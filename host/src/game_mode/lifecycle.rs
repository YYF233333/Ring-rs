//! 小游戏生命周期状态机
//!
//! 管理 WebView 的创建、运行和销毁。

use std::collections::HashMap;

use tracing::info;
use vn_runtime::state::VarValue;
use wry::WebViewBuilder;

use super::http_bridge::BridgeServer;

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

/// 小游戏完成结果
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

    /// 启动小游戏：通过 HTTP Bridge 创建 WebView
    ///
    /// WebView 通过 `http://127.0.0.1:{PORT}/index.html` 加载游戏页面，
    /// JS SDK 通过 init_script 注入 `window.engine.*` API。
    pub fn start<W: wry::raw_window_handle::HasWindowHandle>(
        &mut self,
        window: &W,
        window_size: (u32, u32),
        launch: &PendingGameLaunch,
        bridge: &BridgeServer,
    ) -> Result<wry::WebView, GameModeError> {
        if self.is_active() {
            return Err(GameModeError::AlreadyRunning);
        }

        let webview = WebViewBuilder::new()
            .with_url(bridge.game_url())
            .with_initialization_script(super::http_bridge::js_sdk_init_script())
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
        info!(
            game_id = %launch.game_id,
            port = bridge.port(),
            "WebView 小游戏已启动 (HTTP Bridge)"
        );

        Ok(webview)
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
