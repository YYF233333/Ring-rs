//! # 小游戏模式管理
//!
//! 通过 wry (WebView) 嵌入 HTML5 小游戏。

pub mod bridge;
pub mod lifecycle;

pub use lifecycle::{GameCompletion, GameMode, GameModeError, GameModeState, PendingGameLaunch};
