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
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tracing::info;

/// 从 CWD 向上查找包含 `assets` 子目录的祖先目录，作为项目根目录。
///
/// 开发模式下 Tauri 的 CWD 通常是 `host-tauri/src-tauri/`，
/// 而 assets 位于仓库根目录，需要向上两级才能找到。
fn find_project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut dir: &Path = &cwd;
    loop {
        if dir.join("assets").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    cwd
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let shared_inner = Arc::new(Mutex::new(AppStateInner::new()));

    tauri::Builder::default()
        .manage(AppState {
            inner: shared_inner.clone(),
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

                let rm = resources::ResourceManager::new(&assets_root);

                // saves_dir 也相对于项目根目录解析
                let saves_dir = if cfg.saves_dir.is_relative() {
                    project_root.join(&cfg.saves_dir)
                } else {
                    cfg.saves_dir.clone()
                };
                let sm = save_manager::SaveManager::new(&saves_dir);

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
            commands::load_game,
            commands::list_saves,
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
            commands::debug_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
