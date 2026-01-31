//! # Input 模块
//!
//! 定义 Host 向 Runtime 传递的输入事件。
//!
//! ## 设计说明
//!
//! - `RuntimeInput` 是 Host 采集用户操作后，传递给 Runtime 的抽象输入
//! - Runtime 不直接处理鼠标/键盘事件，只处理语义化的输入
//! - `WaitForTime` 由 Host 处理，Runtime 不需要接收时间流逝事件

use serde::{Deserialize, Serialize};

/// 信号标识符
///
/// 用于 `WaitForSignal` 等待模式，允许外部系统触发 Runtime 继续执行。
pub type SignalId = String;

/// Host 向 Runtime 传递的输入
///
/// Runtime 通过 `tick(input)` 接收这些输入，并根据当前等待状态决定如何处理。
///
/// # 设计说明
///
/// - `Click`：解除 `WaitForClick` 等待
/// - `ChoiceSelected`：解除 `WaitForChoice` 等待，并传递用户选择
/// - `Signal`：解除 `WaitForSignal` 等待
///
/// 注意：`WaitForTime` 由 Host 层处理，Host 等待指定时长后直接调用 `tick(None)`，
/// Runtime 不需要知道时间流逝。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuntimeInput {
    /// 用户点击（解除 `WaitForClick`）
    Click,

    /// 用户选择了某个选项（解除 `WaitForChoice`）
    ///
    /// `index` 是选项的索引（从 0 开始）
    ChoiceSelected { index: usize },

    /// 外部信号（解除 `WaitForSignal`）
    Signal { id: SignalId },
}

impl RuntimeInput {
    /// 创建点击输入
    pub fn click() -> Self {
        Self::Click
    }

    /// 创建选择输入
    pub fn choice(index: usize) -> Self {
        Self::ChoiceSelected { index }
    }

    /// 创建信号输入
    pub fn signal(id: impl Into<SignalId>) -> Self {
        Self::Signal { id: id.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let click = RuntimeInput::click();
        assert_eq!(click, RuntimeInput::Click);

        let choice = RuntimeInput::choice(2);
        assert_eq!(choice, RuntimeInput::ChoiceSelected { index: 2 });

        let signal = RuntimeInput::signal("animation_done");
        assert_eq!(
            signal,
            RuntimeInput::Signal {
                id: "animation_done".to_string()
            }
        );
    }

    #[test]
    fn test_input_serialization() {
        let input = RuntimeInput::ChoiceSelected { index: 1 };
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: RuntimeInput = serde_json::from_str(&json).unwrap();
        assert_eq!(input, deserialized);
    }
}

