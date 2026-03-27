mod audio;
mod command_executor;
mod commands;
mod config;
#[cfg(debug_assertions)]
mod debug_server;
mod manifest;
mod render_state;
mod resources;
mod save_manager;
mod state;

use state::{AppState, AppStateInner, Services};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri::http;
use tracing::{info, warn};

/// 简易 percent-decode：处理 URL 路径中的 `%XX` 编码（如中文文件名）。
fn percent_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                &input[i + 1..i + 3],
                16,
            ) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// 定位项目根目录。
///
/// 优先查找 `config.json`（release 产物中始终存在），
/// 回退查找 `assets/` 子目录（开发模式兼容）。
/// 开发模式下 Tauri 的 CWD 通常是 `host-tauri/src-tauri/`，需要向上遍历。
fn find_project_root() -> PathBuf {
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
    cfg: &config::AppConfig,
    assets_root: &Path,
    project_root: &Path,
) -> resources::ResourceManager {
    match cfg.asset_source {
        config::AssetSourceType::Fs => {
            info!("资源来源: 文件系统");
            resources::ResourceManager::new(assets_root)
        }
        config::AssetSourceType::Zip => {
            let zip_rel = cfg.zip_path.as_deref().unwrap_or("assets.zip");
            let zip_path = if Path::new(zip_rel).is_relative() {
                project_root.join(zip_rel)
            } else {
                PathBuf::from(zip_rel)
            };
            info!(path = %zip_path.display(), "资源来源: ZIP");
            let source = resources::ZipSource::open(&zip_path).expect("ZIP 资源文件打开失败");
            resources::ResourceManager::with_source(Box::new(source), assets_root)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let shared_inner = Arc::new(Mutex::new(AppStateInner::new()));

    tauri::Builder::default()
        .manage(AppState {
            inner: shared_inner.clone(),
        })
        .register_uri_scheme_protocol("ring-asset", {
            let shared = shared_inner.clone();
            move |_ctx, request| {
                let path_raw = percent_decode(request.uri().path().trim_start_matches('/'));
                let logical = resources::LogicalPath::new(&path_raw);
                let mime = resources::guess_mime_type(logical.as_str());

                let inner = shared.lock().expect("invariant: lock not poisoned");
                let result = if let Some(svc) = inner.services.as_ref() {
                    svc.resources.read_bytes(&logical)
                } else {
                    Err(resources::ResourceError::LoadFailed {
                        path: logical.as_str().to_string(),
                        kind: "protocol".to_string(),
                        message: "services not initialized".to_string(),
                    })
                };
                drop(inner);

                match result {
                    Ok(bytes) => http::Response::builder()
                        .status(200)
                        .header("Content-Type", mime)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Cow::from(bytes))
                        .unwrap(),
                    Err(e) => {
                        warn!(path = %logical, error = %e, "ring-asset 协议资源未找到");
                        let msg = format!("Not Found: {logical}");
                        http::Response::builder()
                            .status(404)
                            .header("Content-Type", "text/plain")
                            .body(Cow::from(msg.into_bytes()))
                            .unwrap()
                    }
                }
            }
        })
        .setup({
            let shared_inner = shared_inner.clone();
            move |_app| {
                let mut inner = shared_inner.lock().expect("invariant: lock not poisoned");

                let project_root = find_project_root();
                info!(root = %project_root.display(), "项目根目录");

                // 尝试加载配置（config.json 不存在时使用默认值）
                let cfg_path = project_root.join("config.json");
                let cfg = config::AppConfig::load(&cfg_path).unwrap_or_else(|e| {
                    info!("使用默认配置 ({e})");
                    config::AppConfig::default()
                });

                // assets_root 相对于项目根目录解析
                let assets_root = if cfg.assets_root.is_relative() {
                    project_root.join(&cfg.assets_root)
                } else {
                    cfg.assets_root.clone()
                };
                info!(assets = %assets_root.display(), "资源根目录");

                let rm = create_resource_manager(&cfg, &assets_root, &project_root);

                // saves_dir 也相对于项目根目录解析
                let saves_dir = if cfg.saves_dir.is_relative() {
                    project_root.join(&cfg.saves_dir)
                } else {
                    cfg.saves_dir.clone()
                };
                let sm = save_manager::SaveManager::new(&saves_dir);

                // 加载 manifest（立绘元数据）——通过 ResourceManager 统一读取（FS/ZIP 透明）
                let manifest_logical = resources::LogicalPath::new(&cfg.manifest_path);
                let manifest = match rm.read_text(&manifest_logical) {
                    Ok(content) => serde_json::from_str::<manifest::Manifest>(&content)
                        .unwrap_or_else(|e| {
                            info!("manifest JSON 解析失败，使用默认 ({e})");
                            manifest::Manifest::with_defaults()
                        }),
                    Err(e) => {
                        info!("manifest 加载失败，使用默认 ({e})");
                        manifest::Manifest::with_defaults()
                    }
                };
                info!(presets = manifest.presets.len(), "Manifest 加载完成");

                // 初始化 AudioManager（headless 状态追踪，无设备依赖）
                let mut am = audio::AudioManager::new();
                am.set_bgm_volume(cfg.audio.bgm_volume);
                am.set_sfx_volume(cfg.audio.sfx_volume);
                info!("AudioManager 初始化成功");

                // 加载持久化变量
                inner.persistent_store = state::PersistentStore::load(&saves_dir);

                inner.services = Some(Services {
                    audio: am,
                    resources: rm,
                    saves: sm,
                    config: cfg,
                    manifest,
                });
                info!("子系统初始化完成");

                drop(inner);

                #[cfg(debug_assertions)]
                {
                    debug_server::start(shared_inner, assets_root);

                    if std::env::var("RING_HEADLESS").is_ok() {
                        info!("Headless 模式：Tauri 窗口已隐藏，请使用浏览器 http://localhost:5173 调试");
                        if let Some(window) = _app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                }

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_game,
            commands::tick,
            commands::click,
            commands::choose,
            commands::get_render_state,
            commands::save_game,
            commands::save_game_with_thumbnail,
            commands::load_game,
            commands::list_saves,
            commands::get_thumbnail,
            commands::delete_save,
            commands::get_assets_root,
            commands::get_config,
            commands::get_user_settings,
            commands::update_settings,
            commands::get_history,
            commands::return_to_title,
            commands::continue_game,
            commands::quit_game,
            commands::finish_cutscene,
            commands::backspace,
            commands::set_playback_mode,
            commands::get_playback_mode,
            commands::frontend_connected,
            commands::log_frontend,
            commands::get_screen_definitions,
            commands::get_ui_assets,
            commands::get_ui_condition_context,
            commands::debug_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
