//! 命令处理模块
//!
//! 处理 `CommandExecutor` 产出的副作用：
//! - `effect_applier`：统一动画/过渡效果应用（替代原来的 character + transition 处理）
//! - `audio`：音频命令处理

mod audio;
mod effect_applier;

pub use audio::*;
pub use effect_applier::*;
