//! # 小游戏模式管理
//!
//! 通过 wry (WebView) 嵌入 HTML5 小游戏，使用 HTTP Bridge 提供资源和 API。

pub mod bridge;
pub mod http_bridge;
pub mod lifecycle;

pub use http_bridge::BridgeServer;
pub use lifecycle::{GameCompletion, GameMode, GameModeError, GameModeState, PendingGameLaunch};
