//! # 小游戏模式管理
//!
//! 通过 wry (WebView) 嵌入 HTML5 小游戏。
//! 整个模块在 `mini-games` feature gate 下条件编译。

#[cfg(feature = "mini-games")]
pub mod bridge;
#[cfg(feature = "mini-games")]
pub mod lifecycle;

#[cfg(feature = "mini-games")]
pub use lifecycle::{GameCompletion, GameMode, GameModeError, GameModeState, PendingGameLaunch};

/// 检测 mini-games feature 是否可用
pub fn is_available() -> bool {
    cfg!(feature = "mini-games")
}
