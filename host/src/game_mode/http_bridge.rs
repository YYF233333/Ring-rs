//! HTTP Bridge: 本地 HTTP 服务器，统一承担静态资源服务与 Bridge API
//!
//! 替代 `game://` custom protocol 和 wry IPC，为 WebView 小游戏提供：
//! - 静态资源服务（从 `game_dir` 读取文件）
//! - 版本化 REST-like API（`/v1/*`）
//! - JS SDK（`window.engine.*`）注入

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::{info, warn};
use vn_runtime::state::VarValue;

use super::GameCompletion;
use super::bridge::{BridgeResponse, BridgeValue};
use crate::app::AppState;

/// HTTP Bridge 服务器
///
/// 在 `127.0.0.1:0` 上启动本地 HTTP 服务器，同时提供静态资源和 Bridge API。
/// 仅在小游戏运行期间存活，游戏结束后 drop 即关闭。
pub struct BridgeServer {
    server: tiny_http::Server,
    port: u16,
    game_dir: PathBuf,
}

impl std::fmt::Debug for BridgeServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeServer")
            .field("port", &self.port)
            .field("game_dir", &self.game_dir)
            .finish()
    }
}

impl BridgeServer {
    /// 启动 HTTP Bridge 服务器
    ///
    /// 验证游戏目录存在后，在 `127.0.0.1:0`（随机端口）启动 HTTP 服务器。
    pub fn start(game_dir: PathBuf) -> Result<Self, BridgeServerError> {
        let index_path = game_dir.join("index.html");
        if !index_path.exists() {
            return Err(BridgeServerError::AssetsNotFound(
                index_path.to_string_lossy().to_string(),
            ));
        }

        let game_dir = game_dir.canonicalize().map_err(|e| {
            BridgeServerError::AssetsNotFound(format!("{}: {e}", game_dir.display()))
        })?;

        let server = tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| BridgeServerError::BindFailed(e.to_string()))?;

        let port = server
            .server_addr()
            .to_ip()
            .expect("invariant: bound to IP address")
            .port();

        info!(port, game_dir = %game_dir.display(), "HTTP Bridge 已启动");
        Ok(Self {
            server,
            port,
            game_dir,
        })
    }

    /// 获取服务器端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取 WebView 加载 URL
    pub fn game_url(&self) -> String {
        format!("http://127.0.0.1:{}/index.html", self.port)
    }

    /// 轮询并处理所有待处理的 HTTP 请求
    ///
    /// 返回 `Some(GameCompletion)` 当游戏调用 `/v1/complete` 端点。
    pub fn poll(&self, app_state: &mut AppState) -> Option<GameCompletion> {
        let mut completion = None;
        while let Some(request) = self.server.try_recv().ok().flatten() {
            let url = request.url().to_string();
            if url.starts_with("/v1/") {
                if let Some(c) = handle_api(request, &url, &self.game_dir, app_state) {
                    completion = Some(c);
                }
            } else {
                serve_static(request, &url, &self.game_dir);
            }
        }
        completion
    }
}

// ── API 路由 ──

fn handle_api(
    mut request: tiny_http::Request,
    url: &str,
    game_dir: &Path,
    app_state: &mut AppState,
) -> Option<GameCompletion> {
    let mut completion = None;

    let response = match url {
        "/v1/info" => BridgeResponse::ok(Some(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "api_version": "1"
        }))),

        "/v1/audio/play-sound" => match read_body::<AudioPlayReq>(&mut request) {
            Ok(r) => play_sound(&r.name, game_dir, app_state),
            Err(e) => BridgeResponse::err(e),
        },

        "/v1/audio/play-bgm" => match read_body::<PlayBgmReq>(&mut request) {
            Ok(r) => play_bgm(&r.name, r.r#loop, game_dir, app_state),
            Err(e) => BridgeResponse::err(e),
        },

        "/v1/audio/stop-bgm" => {
            if let Some(audio) = app_state.core.audio_manager.as_mut() {
                audio.stop_bgm(None);
            }
            BridgeResponse::ok(None)
        }

        "/v1/state/get" => match read_body::<StateGetReq>(&mut request) {
            Ok(r) => get_state(&r.key, app_state),
            Err(e) => BridgeResponse::err(e),
        },

        "/v1/state/set" => match read_body::<StateSetReq>(&mut request) {
            Ok(r) => {
                if let Some(rt) = app_state.session.vn_runtime.as_mut() {
                    rt.state_mut().set_var(r.key, r.value.into());
                }
                BridgeResponse::ok(None)
            }
            Err(e) => BridgeResponse::err(e),
        },

        "/v1/complete" => match read_body::<CompleteReq>(&mut request) {
            Ok(r) => {
                completion = Some(GameCompletion {
                    result: r.result.into(),
                });
                BridgeResponse::ok(None)
            }
            Err(e) => BridgeResponse::err(e),
        },

        "/v1/log" => match read_body::<LogReq>(&mut request) {
            Ok(r) => {
                match r.level.as_str() {
                    "error" => tracing::error!("[WebView] {}", r.message),
                    "warn" => tracing::warn!("[WebView] {}", r.message),
                    "debug" => tracing::debug!("[WebView] {}", r.message),
                    _ => info!("[WebView] {}", r.message),
                }
                BridgeResponse::ok(None)
            }
            Err(e) => BridgeResponse::err(e),
        },

        _ => BridgeResponse::err(format!("unknown endpoint: {url}")),
    };

    respond_json(request, &response);
    completion
}

// ── 端点处理 ──

fn play_sound(name: &str, game_dir: &Path, app_state: &mut AppState) -> BridgeResponse {
    let Some(audio) = app_state.core.audio_manager.as_mut() else {
        return BridgeResponse::ok(None);
    };
    let file_path = game_dir.join(name);
    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let cache_key = format!("game:{name}");
            audio.cache_audio_bytes(&cache_key, bytes);
            audio.play_sfx(&cache_key);
            BridgeResponse::ok(None)
        }
        Err(e) => BridgeResponse::err(format!("audio file not found: {e}")),
    }
}

fn play_bgm(
    name: &str,
    looping: bool,
    game_dir: &Path,
    app_state: &mut AppState,
) -> BridgeResponse {
    let Some(audio) = app_state.core.audio_manager.as_mut() else {
        return BridgeResponse::ok(None);
    };
    let file_path = game_dir.join(name);
    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let cache_key = format!("game:{name}");
            audio.cache_audio_bytes(&cache_key, bytes);
            audio.play_bgm(&cache_key, looping, None);
            BridgeResponse::ok(None)
        }
        Err(e) => BridgeResponse::err(format!("audio file not found: {e}")),
    }
}

fn get_state(key: &str, app_state: &AppState) -> BridgeResponse {
    let value = app_state
        .session
        .vn_runtime
        .as_ref()
        .and_then(|rt| rt.state().get_var(key).cloned());

    let json_value = value
        .map(|v| var_to_json(&v))
        .unwrap_or(serde_json::Value::Null);
    BridgeResponse::ok(Some(serde_json::json!({ "value": json_value })))
}

fn var_to_json(value: &VarValue) -> serde_json::Value {
    match value {
        VarValue::Int(n) => serde_json::Value::Number((*n).into()),
        VarValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        VarValue::String(s) => serde_json::Value::String(s.clone()),
        VarValue::Bool(b) => serde_json::Value::Bool(*b),
    }
}

// ── 静态资源服务 ──

fn serve_static(request: tiny_http::Request, url_path: &str, game_dir: &Path) {
    let relative = url_path.strip_prefix('/').unwrap_or(url_path);

    if relative.contains("..") {
        let response = tiny_http::Response::from_string("Forbidden").with_status_code(403);
        let _ = request.respond(response);
        return;
    }

    let file_path = game_dir.join(relative);

    let (data, mime, status) = if file_path.is_file() {
        let data = std::fs::read(&file_path).unwrap_or_default();
        let mime = mime_from_ext(file_path.extension().and_then(|e| e.to_str()));
        (data, mime, 200)
    } else {
        warn!(path = %file_path.display(), "Game asset not found");
        (b"Not Found".to_vec(), "text/plain", 404)
    };

    let response = tiny_http::Response::from_data(data)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes("Content-Type", mime).expect("invariant: valid header"),
        );
    let _ = request.respond(response);
}

fn mime_from_ext(ext: Option<&str>) -> &'static str {
    match ext {
        Some("html" | "htm") => "text/html",
        Some("js" | "mjs") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
}

// ── 请求体类型 ──

#[derive(Deserialize)]
struct AudioPlayReq {
    name: String,
}

#[derive(Deserialize)]
struct PlayBgmReq {
    name: String,
    #[serde(default = "default_loop")]
    r#loop: bool,
}

fn default_loop() -> bool {
    true
}

#[derive(Deserialize)]
struct StateGetReq {
    key: String,
}

#[derive(Deserialize)]
struct StateSetReq {
    key: String,
    value: BridgeValue,
}

#[derive(Deserialize)]
struct CompleteReq {
    result: BridgeValue,
}

#[derive(Deserialize)]
struct LogReq {
    level: String,
    message: String,
}

// ── 辅助函数 ──

fn read_body<T: serde::de::DeserializeOwned>(
    request: &mut tiny_http::Request,
) -> Result<T, String> {
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|e| format!("failed to read request body: {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("invalid JSON: {e}"))
}

fn respond_json(request: tiny_http::Request, response: &BridgeResponse) {
    let json = serde_json::to_string(response)
        .unwrap_or_else(|_| r#"{"success":false,"error":"internal serialization error"}"#.into());
    let http_response = tiny_http::Response::from_string(json).with_header(
        tiny_http::Header::from_bytes("Content-Type", "application/json")
            .expect("invariant: valid header"),
    );
    let _ = request.respond(http_response);
}

/// JS SDK init_script
///
/// 注入 `window.engine.*` API，使用 `location.origin` 作为基地址。
pub fn js_sdk_init_script() -> &'static str {
    r#"
(function() {
    const BASE = location.origin + '/v1';

    async function call(endpoint, body) {
        const resp = await fetch(BASE + '/' + endpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body || {}),
        });
        const data = await resp.json();
        if (!data.success) throw new Error(data.error || 'Bridge call failed');
        return data.data;
    }

    window.engine = {
        async playSound(name) {
            await call('audio/play-sound', { name: name });
        },
        async playBGM(name, shouldLoop) {
            await call('audio/play-bgm', { name: name, loop: shouldLoop !== false });
        },
        async stopBGM() {
            await call('audio/stop-bgm');
        },
        async getState(key) {
            var result = await call('state/get', { key: key });
            return result ? result.value : undefined;
        },
        async setState(key, value) {
            await call('state/set', { key: key, value: value });
        },
        async complete(result) {
            await call('complete', { result: result !== undefined ? result : null });
        },
        log: function(level, message) {
            call('log', { level: level, message: message }).catch(function() {});
        },
    };
})();
"#
}

/// HTTP Bridge 服务器错误
#[derive(Debug, thiserror::Error)]
pub enum BridgeServerError {
    #[error("failed to bind HTTP server: {0}")]
    BindFailed(String),
    #[error("game assets not found: {0}")]
    AssetsNotFound(String),
}
