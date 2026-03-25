//! Debug HTTP server (debug builds only).
//!
//! Mirrors Tauri IPC commands as HTTP endpoints so the frontend can run
//! in a regular browser for Agent-driven debugging via browser MCP.
//!
//! NOTE: The dispatch logic intentionally duplicates the thin wrappers in
//! `commands.rs`. This is acceptable because commands are trivial (lock →
//! call method → serialize) and the debug server is not shipped in release.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, State as AxState};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::render_state::PlaybackMode;
use crate::state::{AppStateInner, UserSettings};

type SharedState = Arc<Mutex<AppStateInner>>;

#[derive(Clone)]
struct ServerCtx {
    state: SharedState,
}

/// 在独立线程启动 debug HTTP server（端口 9528）。
pub fn start(state: SharedState, assets_root: PathBuf) {
    let ctx = ServerCtx { state };

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("invariant: tokio runtime builds");

        rt.block_on(async {
            let app = Router::new()
                .route("/api/{command}", post(handle_command))
                .nest_service("/assets", ServeDir::new(&assets_root))
                .layer(CorsLayer::permissive())
                .with_state(ctx);

            let listener = tokio::net::TcpListener::bind("127.0.0.1:9528")
                .await
                .expect("invariant: debug port 9528 available");
            tracing::info!("Debug HTTP server: http://127.0.0.1:9528");
            axum::serve(listener, app).await.ok();
        });
    });
}

async fn handle_command(
    Path(command): Path<String>,
    AxState(ctx): AxState<ServerCtx>,
    body: String,
) -> impl IntoResponse {
    let args: serde_json::Value = if body.is_empty() {
        serde_json::Value::Object(Default::default())
    } else {
        match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")).into_response();
            }
        }
    };

    match dispatch(&command, &ctx.state, &args) {
        Ok(value) => axum::Json(value).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

fn dispatch(
    command: &str,
    state: &SharedState,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match command {
        "init_game" => {
            let script_path = args["scriptPath"]
                .as_str()
                .ok_or("missing scriptPath")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            if inner.services.is_some() {
                inner.init_game_from_resource(&script_path)?;
            } else {
                let content = std::fs::read_to_string(&script_path)
                    .map_err(|e| format!("读取脚本文件失败 '{script_path}': {e}"))?;
                inner.init_game(&content)?;
            }
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "tick" => {
            let dt = args["dt"].as_f64().unwrap_or(0.016) as f32;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.process_tick(dt);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "click" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.process_click();
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "choose" => {
            let index = args["index"].as_u64().ok_or("missing index")? as usize;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.process_choose(index);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "get_render_state" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "save_game" => {
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let inner = state.lock().map_err(|e| e.to_string())?;
            let svc = inner.services();
            let rt = inner.runtime.as_ref().ok_or("游戏未启动")?;
            let runtime_state = rt.state().clone();
            let mut save_data =
                vn_runtime::SaveData::new(slot, runtime_state).with_history(rt.history().clone());
            if let Some(ref cm) = inner.render_state.chapter_mark {
                save_data = save_data.with_chapter(&cm.title);
            }
            save_data = save_data.with_audio(vn_runtime::AudioState {
                current_bgm: svc.audio.current_bgm_path().map(|s| s.to_string()),
                bgm_looping: true,
            });
            svc.saves.save(&save_data).map_err(|e| e.to_string())?;
            Ok(serde_json::Value::Null)
        }

        "load_game" => {
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            let save_data = inner
                .services()
                .saves
                .load(slot)
                .map_err(|e| e.to_string())?;
            inner.restore_from_save(save_data)?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "list_saves" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let svc = inner.services();
            let saves = svc.saves.list_saves();
            let infos: Vec<_> = saves
                .iter()
                .filter_map(|(slot, _)| svc.saves.get_save_info(*slot))
                .collect();
            Ok(serde_json::to_value(&infos).unwrap_or_default())
        }

        "delete_save" => {
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let inner = state.lock().map_err(|e| e.to_string())?;
            inner
                .services()
                .saves
                .delete(slot)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::Value::Null)
        }

        "get_assets_root" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::Value::String(
                inner
                    .services()
                    .resources
                    .base_path()
                    .to_string_lossy()
                    .to_string(),
            ))
        }

        "get_config" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let cfg = inner.services().config.clone();
            Ok(serde_json::to_value(&cfg).unwrap_or_default())
        }

        "get_user_settings" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.user_settings).unwrap_or_default())
        }

        "update_settings" => {
            let settings: UserSettings = serde_json::from_value(args["settings"].clone())
                .map_err(|e| format!("Invalid settings: {e}"))?;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.text_speed = settings.text_speed;
            let svc = inner.services_mut();
            svc.audio.set_bgm_volume(settings.bgm_volume / 100.0);
            svc.audio.set_sfx_volume(settings.sfx_volume / 100.0);
            inner.user_settings = settings;
            Ok(serde_json::Value::Null)
        }

        "get_history" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.history).unwrap_or_default())
        }

        "return_to_title" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.return_to_title();
            Ok(serde_json::Value::Null)
        }

        "continue_game" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            let svc = inner.services();
            if !svc.saves.has_continue() {
                return Err("没有 continue 存档".to_string());
            }
            let save_data = svc.saves.load_continue().map_err(|e| e.to_string())?;
            inner.restore_from_save(save_data)?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "quit_game" => Ok(serde_json::Value::Null),

        "finish_cutscene" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.finish_cutscene();
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "backspace" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            if inner.restore_snapshot() {
                Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
            } else {
                Err("没有可回退的快照".to_string())
            }
        }

        "set_playback_mode" => {
            let mode_str = args["mode"].as_str().unwrap_or("normal");
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.playback_mode = match mode_str {
                "auto" => PlaybackMode::Auto,
                "skip" => PlaybackMode::Skip,
                _ => PlaybackMode::Normal,
            };
            inner.auto_timer = 0.0;
            inner.render_state.playback_mode = inner.playback_mode.clone();
            Ok(serde_json::Value::Null)
        }

        "get_playback_mode" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let mode = match inner.playback_mode {
                PlaybackMode::Normal => "normal",
                PlaybackMode::Auto => "auto",
                PlaybackMode::Skip => "skip",
            };
            Ok(serde_json::Value::String(mode.to_string()))
        }

        "log_frontend" => {
            let level = args["level"].as_str().unwrap_or("debug");
            let module = args["module"].as_str().unwrap_or("unknown");
            let message = args["message"].as_str().unwrap_or("");
            let data = args["data"].as_str().unwrap_or("");
            match level {
                "error" => {
                    tracing::error!(target: "frontend", module = %module, "{message} {data}")
                }
                "warn" => {
                    tracing::warn!(target: "frontend", module = %module, "{message} {data}")
                }
                "info" => {
                    tracing::info!(target: "frontend", module = %module, "{message} {data}")
                }
                _ => tracing::debug!(target: "frontend", module = %module, "{message} {data}"),
            }
            Ok(serde_json::Value::Null)
        }

        "frontend_connected" => {
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.return_to_title();
            Ok(serde_json::Value::Null)
        }

        "get_screen_definitions" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let rm = &inner.services().resources;
            let path = crate::resources::LogicalPath::new("ui/screens.json");
            let text = rm.read_text(&path).map_err(|e| e.to_string())?;
            let val: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("screens.json parse: {e}"))?;
            Ok(val)
        }

        "get_ui_assets" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let rm = &inner.services().resources;
            let path = crate::resources::LogicalPath::new("ui/layout.json");
            let text = rm.read_text(&path).map_err(|e| e.to_string())?;
            let full: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("layout.json parse: {e}"))?;
            Ok(serde_json::json!({
                "assets": full.get("assets").cloned().unwrap_or_default(),
                "colors": full.get("colors").cloned().unwrap_or_default(),
            }))
        }

        "get_ui_condition_context" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            let svc = inner.services();
            let has_continue = svc.saves.has_continue();
            let persistent: serde_json::Map<String, serde_json::Value> = inner
                .persistent_store
                .variables
                .iter()
                .filter_map(|(k, v)| {
                    let json_val = match v {
                        vn_runtime::state::VarValue::Bool(b) => serde_json::Value::Bool(*b),
                        vn_runtime::state::VarValue::Int(i) => serde_json::json!(*i),
                        vn_runtime::state::VarValue::Float(f) => serde_json::json!(*f),
                        vn_runtime::state::VarValue::String(s) => {
                            serde_json::Value::String(s.clone())
                        }
                    };
                    Some((k.clone(), json_val))
                })
                .collect();
            Ok(serde_json::json!({
                "has_continue": has_continue,
                "persistent": persistent,
            }))
        }

        "debug_snapshot" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "has_runtime": inner.runtime.is_some(),
                "waiting": inner.waiting,
                "script_finished": inner.script_finished,
                "render_state": inner.render_state,
                "playback_mode": format!("{:?}", inner.playback_mode),
                "history_count": inner.history.len(),
                "has_audio": inner.services.is_some(),
                "current_bgm": inner.services().audio.current_bgm_path().map(String::from),
                "user_settings": inner.user_settings,
            }))
        }

        _ => Err(format!("Unknown command: {command}")),
    }
}
