//! 小游戏生命周期状态机
//!
//! 管理 WebView 的创建、运行和销毁。

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use tracing::{info, warn};
use vn_runtime::state::VarValue;
use wry::WebViewBuilder;

use super::bridge::BridgeRequest;

/// 游戏模式状态
#[derive(Debug, Default)]
pub enum GameModeState {
    /// 空闲，无活跃小游戏
    #[default]
    Idle,
    /// 小游戏运行中
    Running {
        /// 游戏 ID
        game_id: String,
        /// 请求 key（用于回传 UIResult）
        request_key: String,
    },
}

/// 待启动的小游戏请求（由 script.rs 设置，host_app.rs 消费）
#[derive(Debug, Clone)]
pub struct PendingGameLaunch {
    pub game_id: String,
    pub request_key: String,
    pub params: HashMap<String, VarValue>,
}

/// 小游戏完成结果（通过 channel 从 IPC handler 传回主线程）
pub struct GameCompletion {
    pub result: VarValue,
}

/// 游戏模式管理器
#[derive(Debug)]
pub struct GameMode {
    /// 当前状态
    pub state: GameModeState,
}

impl GameMode {
    pub fn new() -> Self {
        Self {
            state: GameModeState::Idle,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, GameModeState::Running { .. })
    }

    /// 启动小游戏：创建 WebView 并返回完成事件接收端
    ///
    /// 使用 wry custom protocol（`game://`）加载游戏资源，
    /// 避免 `file://` 路径中 Windows 盘符冒号导致 `http::Uri` 解析失败。
    pub fn start<W: wry::raw_window_handle::HasWindowHandle>(
        &mut self,
        window: &W,
        window_size: (u32, u32),
        launch: &PendingGameLaunch,
        assets_root: &Path,
    ) -> Result<(wry::WebView, mpsc::Receiver<GameCompletion>), GameModeError> {
        if self.is_active() {
            return Err(GameModeError::AlreadyRunning);
        }

        let game_dir = assets_root.join("games").join(&launch.game_id);
        let index_path = game_dir.join("index.html");

        if !index_path.exists() {
            return Err(GameModeError::AssetsNotFound(
                index_path.to_string_lossy().to_string(),
            ));
        }

        let game_dir_abs = game_dir
            .canonicalize()
            .map_err(|e| GameModeError::AssetsNotFound(format!("{}: {}", game_dir.display(), e)))?;

        let (tx, rx) = mpsc::channel::<GameCompletion>();

        let init_script = r#"
            window.engine = {
                onComplete(result) {
                    window.ipc.postMessage(JSON.stringify({ type: "onComplete", result: result }));
                },
                log(level, message) {
                    window.ipc.postMessage(JSON.stringify({ type: "log", level: level, message: message }));
                }
            };
        "#;

        let webview = WebViewBuilder::new()
            .with_custom_protocol("game".to_string(), make_asset_handler(game_dir_abs))
            .with_url("game://localhost/index.html")
            .with_initialization_script(init_script)
            .with_ipc_handler(move |request| {
                let body = request.body();
                match serde_json::from_str::<BridgeRequest>(body) {
                    Ok(BridgeRequest::OnComplete { result }) => {
                        let var_value: VarValue = result.into();
                        let _ = tx.send(GameCompletion { result: var_value });
                    }
                    Ok(BridgeRequest::Log { level, message }) => {
                        info!(level = %level, "[WebView] {}", message);
                    }
                    Ok(other) => {
                        warn!(?other, "Unhandled BridgeRequest");
                    }
                    Err(e) => {
                        warn!(error = %e, body = %body, "Failed to parse BridgeRequest");
                    }
                }
            })
            .with_bounds(wry::Rect {
                position: wry::dpi::Position::Physical(wry::dpi::PhysicalPosition::new(0, 0)),
                size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
                    window_size.0,
                    window_size.1,
                )),
            })
            .with_devtools(cfg!(debug_assertions))
            .build_as_child(window)
            .map_err(|e| GameModeError::WebViewCreationFailed(e.to_string()))?;

        self.state = GameModeState::Running {
            game_id: launch.game_id.clone(),
            request_key: launch.request_key.clone(),
        };
        info!(game_id = %launch.game_id, "WebView 小游戏已启动 (custom protocol)");

        Ok((webview, rx))
    }

    /// 小游戏完成，清理状态并返回 request_key
    pub fn complete(&mut self) -> Option<String> {
        if let GameModeState::Running { request_key, .. } =
            std::mem::replace(&mut self.state, GameModeState::Idle)
        {
            info!("GameMode: 小游戏结束");
            Some(request_key)
        } else {
            None
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self::new()
    }
}

/// 构造 custom protocol 的资源文件处理闭包
///
/// 将 `game://localhost/<path>` 映射到 `game_dir/<path>` 并返回文件内容。
/// Windows 上 WebView2 会将 `game://` 翻译为 `http://game.localhost/`。
fn make_asset_handler(
    game_dir: PathBuf,
) -> impl Fn(wry::WebViewId<'_>, wry::http::Request<Vec<u8>>) -> wry::http::Response<Cow<'static, [u8]>>
+ 'static {
    move |_id, request: wry::http::Request<Vec<u8>>| {
        let uri_path = request.uri().path();
        let relative = uri_path.strip_prefix('/').unwrap_or(uri_path);
        let file_path = game_dir.join(relative);

        let (data, mime) = if file_path.is_file() {
            let data = std::fs::read(&file_path).unwrap_or_default();
            let mime = mime_from_ext(file_path.extension().and_then(|e| e.to_str()));
            (data, mime)
        } else {
            warn!(path = %file_path.display(), "Game asset not found");
            (b"Not Found".to_vec(), "text/plain")
        };

        wry::http::Response::builder()
            .header("Content-Type", mime)
            .body(Cow::from(data))
            .unwrap()
    }
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

/// 游戏模式错误
#[derive(Debug, thiserror::Error)]
pub enum GameModeError {
    #[error("another game is already running")]
    AlreadyRunning,
    #[error("WebView not available on this platform")]
    WebViewNotAvailable,
    #[error("game assets not found: {0}")]
    AssetsNotFound(String),
    #[error("WebView creation failed: {0}")]
    WebViewCreationFailed(String),
}
