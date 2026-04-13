//! 宿主初始化逻辑
//!
//! 从 host-tauri 的 `lib.rs` 提取的通用初始化逻辑，
//! 不依赖任何 Tauri API。

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::audio::AudioManager;
use crate::config::{self, AppConfig};
use crate::error::HostError;
use crate::layout_config::UiLayoutConfig;
use crate::manifest;
use crate::resources::{self, LogicalPath, ResourceManager};
use crate::save_manager::SaveManager;
use crate::screen_defs::ScreenDefinitions;
use crate::state::{AppStateInner, PersistentStore, Services};

/// 简易 percent-decode：处理 URL 路径中的 `%XX` 编码（如中文文件名）。
pub fn percent_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16)
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// 定位项目根目录。
#[cfg(test)]
mod tests {
    use super::percent_decode;

    #[test]
    fn percent_decode_plain_ascii_passthrough() {
        assert_eq!(percent_decode("hello/world.png"), "hello/world.png");
        assert_eq!(percent_decode(""), "");
    }

    #[test]
    fn percent_decode_encoded_chars() {
        // %E4%B8%AD%E6%96%87 = "中文" in UTF-8 percent-encoded
        assert_eq!(percent_decode("%E4%B8%AD%E6%96%87"), "中文");
    }

    #[test]
    fn percent_decode_partial_mixed() {
        // ASCII mix with encoded space (%20)
        assert_eq!(percent_decode("hello%20world"), "hello world");
    }

    #[test]
    fn percent_decode_incomplete_sequence_left_as_is() {
        // A lone '%' at end is treated as literal bytes; result is the original string
        let result = percent_decode("test%");
        assert_eq!(result, "test%");
    }
}

///
/// 优先查找 `config.json`，回退查找 `assets/` 子目录。
pub fn find_project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut dir: &Path = &cwd;
    loop {
        if dir.join("config.json").is_file() || dir.join("assets").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    cwd
}

/// 根据配置创建 ResourceManager（FS 或 ZIP 模式）
fn create_resource_manager(
    cfg: &AppConfig,
    assets_root: &Path,
    project_root: &Path,
) -> Result<ResourceManager, HostError> {
    match cfg.asset_source {
        config::AssetSourceType::Fs => {
            info!("资源来源: 文件系统");
            Ok(ResourceManager::new(assets_root))
        }
        config::AssetSourceType::Zip => {
            let zip_rel = cfg.zip_path.as_deref().unwrap_or("assets.zip");
            let zip_path = if Path::new(zip_rel).is_relative() {
                project_root.join(zip_rel)
            } else {
                PathBuf::from(zip_rel)
            };
            info!(path = %zip_path.display(), "资源来源: ZIP");
            let source = resources::ZipSource::open(&zip_path)?;
            Ok(ResourceManager::with_source(Box::new(source), assets_root))
        }
    }
}

/// 初始化 AppStateInner 的子系统（config、resources、manifest、audio、saves）。
pub fn initialize_inner(inner: &mut AppStateInner) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = find_project_root();
    info!(root = %project_root.display(), "项目根目录");

    let cfg_path = project_root.join("config.json");
    let cfg = AppConfig::load(&cfg_path)?;
    cfg.validate(&project_root)?;

    let assets_root = if cfg.assets_root.is_relative() {
        project_root.join(&cfg.assets_root)
    } else {
        cfg.assets_root.clone()
    };
    info!(assets = %assets_root.display(), "资源根目录");

    let rm = create_resource_manager(&cfg, &assets_root, &project_root)?;

    let manifest_logical = LogicalPath::new(&cfg.manifest_path);
    if !rm.resource_exists(&manifest_logical) {
        return Err(
            HostError::InvalidInput(format!("manifest 不存在: {}", manifest_logical)).into(),
        );
    }
    let start_script_logical = LogicalPath::new(&cfg.start_script_path);
    if !rm.resource_exists(&start_script_logical) {
        return Err(
            HostError::InvalidInput(format!("入口脚本不存在: {}", start_script_logical)).into(),
        );
    }

    let saves_dir = if cfg.saves_dir.is_relative() {
        project_root.join(&cfg.saves_dir)
    } else {
        cfg.saves_dir.clone()
    };
    let sm = SaveManager::new(&saves_dir);

    let manifest_content = rm.read_text(&manifest_logical)?;
    let (mf, manifest_warnings) = manifest::Manifest::parse_and_validate(&manifest_content)?;
    for warning in &manifest_warnings {
        warn!(warning = ?warning, "manifest 校验告警");
    }
    info!(presets = mf.presets.len(), "Manifest 加载完成");

    let mut am = AudioManager::new();
    am.set_bgm_volume(cfg.audio.bgm_volume);
    am.set_sfx_volume(cfg.audio.sfx_volume);
    info!("AudioManager 初始化成功");

    // UI 数据驱动配置加载
    let layout = UiLayoutConfig::load(&rm)?;
    let screen_defs = ScreenDefinitions::load(&rm)?;

    inner.persistent_store = PersistentStore::load(&saves_dir);
    inner.services = Some(Services {
        audio: am,
        resources: rm,
        saves: sm,
        config: cfg,
        manifest: mf,
        layout,
        screen_defs,
    });
    info!("子系统初始化完成");
    Ok(())
}
