//! Debug HTTP server (debug builds only).
//!
//! Mirrors Tauri IPC commands as HTTP endpoints so the frontend can run
//! in a regular browser for Agent-driven debugging via browser MCP.
//!
//! NOTE: The dispatch logic intentionally duplicates the thin wrappers in
//! `commands.rs`. This is acceptable because commands are trivial (lock →
//! call method → serialize) and the debug server is not shipped in release.

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, State as AxState};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use base64::Engine as _;
use tower_http::cors::CorsLayer;

use crate::render_state::{HostScreen, PlaybackMode};
use crate::state::{AppStateInner, UserSettings};

type SharedState = Arc<Mutex<AppStateInner>>;

#[derive(Clone)]
struct ServerCtx {
    state: SharedState,
}

/// 在独立线程启动 debug HTTP server（端口 9528）。
pub fn start(state: SharedState) {
    let ctx = ServerCtx { state };

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("invariant: tokio runtime builds");

        rt.block_on(async {
            let app = Router::new()
                .route("/api/{command}", post(handle_command))
                .route("/assets/{*path}", get(handle_asset))
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

async fn handle_asset(
    Path(path): Path<String>,
    AxState(ctx): AxState<ServerCtx>,
) -> impl IntoResponse {
    let logical = crate::resources::LogicalPath::new(&path);
    let mime = crate::resources::guess_mime_type(logical.as_str());
    let inner = match ctx.state.lock() {
        Ok(inner) => inner,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("state lock failed: {error}"),
            )
                .into_response();
        }
    };

    let bytes = inner
        .services()
        .resources
        .read_bytes(&logical)
        .map_err(|e| e.to_string());

    match bytes {
        Ok(bytes) => (StatusCode::OK, [("Content-Type", mime)], bytes).into_response(),
        Err(error) => (StatusCode::NOT_FOUND, error).into_response(),
    }
}

fn dispatch(
    command: &str,
    state: &SharedState,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match command {
        "init_game" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let script_path = args["scriptPath"]
                .as_str()
                .ok_or("missing scriptPath")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.delete_continue().map_err(|e| e.to_string())?;
            if inner.services.is_some() {
                inner.init_game_from_resource(&script_path).map_err(|e| e.to_string())?;
            } else {
                let content = std::fs::read_to_string(&script_path)
                    .map_err(|e| format!("读取脚本文件失败 '{script_path}': {e}"))?;
                inner.init_game(&content).map_err(|e| e.to_string())?;
            }
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "init_game_at_label" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let script_path = args["scriptPath"]
                .as_str()
                .ok_or("missing scriptPath")?
                .to_string();
            let label = args["label"].as_str().ok_or("missing label")?.to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.delete_continue().map_err(|e| e.to_string())?;
            inner.init_game_from_resource_at_label(&script_path, &label).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "tick" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let dt = args["dt"].as_f64().unwrap_or(0.016) as f32;
            if !(dt >= 0.0 && dt.is_finite()) {
                return Err(format!("参数校验失败: dt 必须为非负有限数，实际为 {dt}"));
            }
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.process_tick(dt);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "click" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.process_click();
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "choose" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let index = args["index"].as_u64().ok_or("missing index")? as usize;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.process_choose(index);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "get_render_state" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "save_game" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.save_to_slot(slot).map_err(|e| e.to_string())?;
            Ok(serde_json::Value::Null)
        }

        "load_game" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            let save_data = inner
                .services()
                .saves
                .load(slot)
                .map_err(|e| e.to_string())?;
            inner.restore_from_save(save_data).map_err(|e| e.to_string())?;
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

        "save_game_with_thumbnail" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let thumbnail_base64 = args["thumbnail_base64"]
                .as_str()
                .ok_or("missing thumbnail_base64")?;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            let png_bytes = base64::engine::general_purpose::STANDARD
                .decode(thumbnail_base64)
                .map_err(|e| format!("base64 decode: {e}"))?;
            inner.save_to_slot_with_thumbnail(slot, &png_bytes).map_err(|e| e.to_string())?;
            Ok(serde_json::Value::Null)
        }

        "get_thumbnail" => {
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let inner = state.lock().map_err(|e| e.to_string())?;
            let b64 = inner.services().saves.load_thumbnail_base64(slot);
            Ok(serde_json::to_value(&b64).unwrap_or_default())
        }

        "delete_save" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let slot = args["slot"].as_u64().ok_or("missing slot")? as u32;
            let inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
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
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let settings: UserSettings = serde_json::from_value(args["settings"].clone())
                .map_err(|e| format!("Invalid settings: {e}"))?;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
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
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let save_continue = args["saveContinue"].as_bool().unwrap_or(false);
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.return_to_title(save_continue);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "continue_game" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            let svc = inner.services();
            if !svc.saves.has_continue() {
                return Err("没有 continue 存档".to_string());
            }
            let save_data = svc.saves.load_continue().map_err(|e| e.to_string())?;
            inner.restore_from_save(save_data).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "quit_game" => Ok(serde_json::Value::Null),

        "finish_cutscene" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.finish_cutscene();
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "submit_ui_result" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let key = args["key"].as_str().ok_or("missing key")?.to_string();
            let value = args["value"].clone();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.handle_ui_result(key, value).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "backspace" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            if inner.restore_snapshot() {
                Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
            } else {
                Err("没有可回退的快照".to_string())
            }
        }

        "set_playback_mode" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let mode: PlaybackMode = serde_json::from_value(args["mode"].clone())
                .map_err(|e| format!("invalid mode: {e}"))?;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.set_playback_mode(mode);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
        }

        "get_playback_mode" => {
            let inner = state.lock().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&inner.playback_mode).unwrap_or_default())
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
            let client_label = args["clientLabel"].as_str().map(|s| s.to_string());
            let session = inner.frontend_connected(client_label);
            Ok(serde_json::to_value(&session).unwrap_or_default())
        }

        "set_host_screen" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let screen: HostScreen = serde_json::from_value(args["screen"].clone())
                .map_err(|e| format!("invalid screen: {e}"))?;
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            inner.set_host_screen(screen);
            Ok(serde_json::to_value(&inner.render_state).unwrap_or_default())
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
            let assets = full
                .get("assets")
                .cloned()
                .ok_or("layout.json missing assets")?;
            let colors = full
                .get("colors")
                .cloned()
                .ok_or("layout.json missing colors")?;
            Ok(serde_json::json!({
                "assets": assets,
                "colors": colors,
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
                .map(|(k, v)| {
                    let json_val = match v {
                        vn_runtime::state::VarValue::Bool(b) => serde_json::Value::Bool(*b),
                        vn_runtime::state::VarValue::Int(i) => serde_json::json!(*i),
                        vn_runtime::state::VarValue::Float(f) => serde_json::json!(*f),
                        vn_runtime::state::VarValue::String(s) => {
                            serde_json::Value::String(s.clone())
                        }
                    };
                    (k.clone(), json_val)
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
                "host_screen": format!("{:?}", inner.host_screen),
                "history_count": inner.history.len(),
                "has_audio": inner.services.is_some(),
                "current_bgm": inner.services().audio.current_bgm_path().map(String::from),
                "user_settings": inner.user_settings,
            }))
        }

        "debug_run_until" => {
            let client_token = args["clientToken"]
                .as_str()
                .ok_or("missing clientToken")?
                .to_string();
            let dt = args["dt"].as_f64().unwrap_or(1.0 / 60.0) as f32;
            let max_steps = args["maxSteps"].as_u64().unwrap_or(600) as usize;
            if !(dt >= 0.0 && dt.is_finite()) {
                return Err(format!("参数校验失败: dt 必须为非负有限数，实际为 {dt}"));
            }
            if max_steps > 100_000 {
                return Err(format!(
                    "参数校验失败: max_steps 不能超过 100000，实际为 {max_steps}"
                ));
            }
            let stop_on_wait = args["stopOnWait"].as_bool().unwrap_or(true);
            let stop_on_script_finished = args["stopOnScriptFinished"].as_bool().unwrap_or(true);
            let mut inner = state.lock().map_err(|e| e.to_string())?;
            inner.assert_owner(&client_token).map_err(|e| e.to_string())?;
            let bundle =
                inner.debug_run_until(dt, max_steps, stop_on_wait, stop_on_script_finished);
            Ok(serde_json::to_value(&bundle).unwrap_or_default())
        }

        _ => Err(format!("Unknown command: {command}")),
    }
}
