//! AppState 初始化拆分
//!
//! 目标：把 `AppState::new` 中"资源/音频/manifest/脚本/用户设置"等初始化逻辑按职责拆开，
//! 让 `app/mod.rs` 保持可读，后续扩展更容易定位修改点。

use super::persistent::PersistentStore;
use crate::manifest::Manifest;
use crate::resources::path::{extract_base_dir, normalize_logical_path};
use crate::resources::{LogicalPath, ResourceManager, ZipSource, extract_script_id};
use crate::save_manager::SaveManager;
use crate::{AppConfig, AssetSourceType, AudioManager, UserSettings};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use vn_runtime::{Parser, analyze_script, extract_resource_references};

use super::script_loader::{scan_scripts, scan_scripts_from_zip};

/// Create the resource source from config (centralized source creation).
pub fn create_resource_source(config: &AppConfig) -> Arc<dyn crate::resources::ResourceSource> {
    let assets_root = assets_root_string(config);
    match config.asset_source {
        AssetSourceType::Fs => Arc::new(crate::resources::FsSource::new(&assets_root)),
        AssetSourceType::Zip => {
            let zip_path = config
                .zip_path
                .as_ref()
                .expect("Zip mode requires zip_path");
            Arc::new(ZipSource::new(zip_path))
        }
    }
}

pub fn assets_root_string(config: &AppConfig) -> String {
    config.assets_root.to_string_lossy().to_string()
}

pub fn saves_dir_string(config: &AppConfig) -> String {
    config.saves_dir.to_string_lossy().to_string()
}

pub fn window_size(config: &AppConfig) -> (f32, f32) {
    (config.window.width as f32, config.window.height as f32)
}

pub fn create_resource_manager(config: &AppConfig) -> ResourceManager {
    let assets_root = assets_root_string(config);

    match config.asset_source {
        AssetSourceType::Fs => {
            info!(assets_root = %assets_root, "资源来源: 文件系统");
            ResourceManager::new(&assets_root, config.resources.texture_cache_size_mb)
        }
        AssetSourceType::Zip => {
            let zip_path = config.zip_path.as_ref().expect("Zip 模式必须配置 zip_path");
            info!(zip_path = %zip_path, "资源来源: ZIP 文件");
            ResourceManager::with_source(
                Arc::new(ZipSource::new(zip_path)),
                config.resources.texture_cache_size_mb,
            )
        }
    }
}

pub fn create_audio_manager(_config: &AppConfig) -> Option<AudioManager> {
    match AudioManager::new() {
        Ok(am) => {
            info!("Audio system initialized");
            Some(am)
        }
        Err(e) => {
            warn!(error = %e, "Audio system initialization failed");
            None
        }
    }
}

pub fn load_manifest(config: &AppConfig, resource_manager: &ResourceManager) -> Manifest {
    let manifest_path = LogicalPath::new(&config.manifest_path);
    match resource_manager.read_text(&manifest_path) {
        Ok(content) => match Manifest::load_from_bytes(content.as_bytes()) {
            Ok(m) => {
                info!(path = %manifest_path, "Manifest loaded");
                m
            }
            Err(e) => {
                warn!(path = %manifest_path, error = %e, "Manifest parse failed, using defaults");
                Manifest::with_defaults()
            }
        },
        Err(e) => {
            warn!(path = %manifest_path, error = %e, "Manifest read failed, using defaults");
            Manifest::with_defaults()
        }
    }
}

pub fn create_save_manager(config: &AppConfig) -> SaveManager {
    let saves_dir = saves_dir_string(config);
    let save_manager = SaveManager::new(&saves_dir);
    info!(saves_dir = %saves_dir, "存档管理器初始化成功");
    save_manager
}

/// 加载持久化变量 store（文件不存在时返回空 store）
pub fn load_persistent_store(config: &AppConfig) -> PersistentStore {
    let saves_dir = saves_dir_string(config);
    PersistentStore::load(&saves_dir)
}

pub fn scan_script_list(config: &AppConfig, resource_manager: &ResourceManager) -> Vec<PathBuf> {
    let scripts = match config.asset_source {
        AssetSourceType::Fs => scan_scripts(&config.assets_root),
        AssetSourceType::Zip => scan_scripts_from_zip(resource_manager),
    };
    info!(count = scripts.len(), "发现脚本文件");
    scripts
}

pub fn load_user_settings(settings_path: &str) -> UserSettings {
    let settings = UserSettings::load(settings_path);
    info!("用户设置加载完成");
    settings
}

/// 运行脚本检查（Dev Mode 自动诊断）
///
/// 在 `debug.script_check = true` 时运行（debug build 默认开启），检查所有脚本的：
/// - 语法错误
/// - 未定义的跳转目标
/// - 资源文件是否存在
///
/// 只输出警告，不阻塞启动。
pub fn run_script_check(
    config: &AppConfig,
    scripts: &[PathBuf],
    resource_manager: &ResourceManager,
) {
    // 检查是否需要运行
    if !config.debug.script_check {
        return;
    }

    info!("Dev Mode: 运行脚本检查...");

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut scripts_checked = 0;

    for script_path in scripts {
        // 读取脚本内容
        let logical_path = normalize_logical_path(&script_path.to_string_lossy());
        let script_id = extract_script_id(&logical_path);
        let content = match resource_manager.read_text(&LogicalPath::new(&logical_path)) {
            Ok(c) => c,
            Err(e) => {
                warn!(script_id = %script_id, error = %e, "脚本无法读取");
                total_warnings += 1;
                continue;
            }
        };

        // 计算 base_path
        let base_path = extract_base_dir(&logical_path);

        // 解析脚本
        let mut parser = Parser::new();
        let script = match parser.parse_with_base_path(&script_id, &content, &base_path) {
            Ok(s) => s,
            Err(e) => {
                error!(script_id = %script_id, error = %e, "脚本解析失败");
                total_errors += 1;
                continue;
            }
        };

        // 输出解析警告
        for warning in parser.warnings() {
            warn!(script_id = %script_id, warning = %warning, "解析警告");
            total_warnings += 1;
        }

        // 运行诊断分析
        let diag = analyze_script(&script);
        for d in &diag.diagnostics {
            if d.level == vn_runtime::DiagnosticLevel::Error {
                total_errors += 1;
                if let Some(line) = d.line {
                    error!(script_id = %script_id, line = line, message = %d.message, "诊断错误");
                } else {
                    error!(script_id = %script_id, message = %d.message, "诊断错误");
                }
            } else {
                total_warnings += 1;
                if let Some(line) = d.line {
                    warn!(script_id = %script_id, line = line, message = %d.message, "诊断警告");
                } else {
                    warn!(script_id = %script_id, message = %d.message, "诊断警告");
                }
            }
        }

        let refs = extract_resource_references(&script);
        for r in refs {
            let lp = LogicalPath::new(&r.resolved_path);
            if !resource_manager.resource_exists(&lp) {
                warn!(
                    script_id = %script_id,
                    resource_type = %r.resource_type,
                    path = %r.resolved_path,
                    "Resource not found"
                );
                total_warnings += 1;
            }
        }

        scripts_checked += 1;
    }

    // 输出汇总
    if total_errors > 0 || total_warnings > 0 {
        warn!(
            scripts = scripts_checked,
            errors = total_errors,
            warnings = total_warnings,
            "脚本检查完成"
        );
        if total_errors > 0 {
            warn!("发现错误，建议修复后再继续");
        }
    } else {
        info!(scripts = scripts_checked, "脚本检查通过");
    }
}
