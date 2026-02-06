//! # Command 模块
//!
//! 定义 Runtime 向 Host 发出的所有指令。
//! Command 是 Runtime 与 Host 之间的**唯一通信方式**。
//!
//! ## 设计原则
//!
//! - **声明式**：Command 描述"做什么"，不描述"怎么做"
//! - **无副作用**：Command 本身不执行任何操作
//! - **引擎无关**：不包含任何 Bevy 或其他引擎的类型

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 过渡效果参数
///
/// 解析器从脚本中提取的过渡效果参数，不解释具体语义。
/// 具体效果的解释由 Host 层负责。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransitionArg {
    /// 数字参数，如 `1.5`
    Number(f64),
    /// 字符串参数，如 `"mask.png"`
    String(String),
    /// 布尔参数，如 `true`
    Bool(bool),
}

/// 过渡效果
///
/// 采用统一函数调用语法，解析器只负责结构提取。
/// 支持位置参数和命名参数（不允许混用）。
///
/// # 示例
///
/// ```text
/// with dissolve             -> Transition { name: "dissolve", args: [] }
/// with Dissolve(1.5)        -> Transition { name: "Dissolve", args: [(None, Number(1.5))] }
/// with Dissolve(duration: 1.5) -> Transition { name: "Dissolve", args: [(Some("duration"), Number(1.5))] }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// 效果名称（大小写敏感）
    pub name: String,
    /// 效果参数列表
    /// - `None` = 位置参数
    /// - `Some(key)` = 命名参数
    pub args: Vec<(Option<String>, TransitionArg)>,
}

impl Transition {
    /// 创建无参数的过渡效果
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    /// 创建带位置参数的过渡效果（兼容旧 API）
    pub fn with_args(name: impl Into<String>, args: Vec<TransitionArg>) -> Self {
        Self {
            name: name.into(),
            args: args.into_iter().map(|a| (None, a)).collect(),
        }
    }

    /// 创建带命名参数的过渡效果
    pub fn with_named_args(
        name: impl Into<String>,
        args: Vec<(Option<String>, TransitionArg)>,
    ) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }

    /// 获取位置参数（按索引）
    pub fn get_positional(&self, index: usize) -> Option<&TransitionArg> {
        self.args
            .iter()
            .filter(|(key, _)| key.is_none())
            .nth(index)
            .map(|(_, v)| v)
    }

    /// 获取命名参数（按 key）
    pub fn get_named(&self, key: &str) -> Option<&TransitionArg> {
        self.args
            .iter()
            .find(|(k, _)| k.as_deref() == Some(key))
            .map(|(_, v)| v)
    }

    /// 获取参数值：优先命名参数，回退到位置参数
    pub fn get_arg(&self, key: &str, positional_index: usize) -> Option<&TransitionArg> {
        self.get_named(key)
            .or_else(|| self.get_positional(positional_index))
    }

    /// 获取 duration 参数（常用辅助方法）
    pub fn get_duration(&self) -> Option<f64> {
        self.get_arg("duration", 0).and_then(|a| match a {
            TransitionArg::Number(n) => Some(*n),
            _ => None,
        })
    }

    /// 获取 reversed 参数（常用辅助方法）
    pub fn get_reversed(&self) -> Option<bool> {
        self.get_arg("reversed", 2).and_then(|a| match a {
            TransitionArg::Bool(b) => Some(*b),
            _ => None,
        })
    }

    /// 判断是否全是位置参数
    pub fn is_all_positional(&self) -> bool {
        self.args.iter().all(|(k, _)| k.is_none())
    }

    /// 判断是否全是命名参数
    pub fn is_all_named(&self) -> bool {
        self.args.is_empty() || self.args.iter().all(|(k, _)| k.is_some())
    }
}

/// 角色立绘位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    /// 左侧
    Left,
    /// 右侧
    Right,
    /// 中央
    Center,
    /// 近左
    NearLeft,
    /// 近右
    NearRight,
    /// 近中
    NearMiddle,
    /// 远左
    FarLeft,
    /// 远右
    FarRight,
    /// 远中
    FarMiddle,
}

impl Position {
    /// 从字符串解析位置（便捷方法）
    pub fn parse(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }
}

impl FromStr for Position {
    type Err = ();

    /// 从字符串解析位置（不区分大小写）
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "center" | "middle" => Ok(Self::Center),
            "nearleft" => Ok(Self::NearLeft),
            "nearright" => Ok(Self::NearRight),
            "nearmiddle" => Ok(Self::NearMiddle),
            "farleft" => Ok(Self::FarLeft),
            "farright" => Ok(Self::FarRight),
            "farmiddle" => Ok(Self::FarMiddle),
            _ => Err(()),
        }
    }
}

/// 选择项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    /// 选项显示文本
    pub text: String,
    /// 跳转目标标签
    pub target_label: String,
}

/// Runtime 向 Host 发出的指令
///
/// 这是 Runtime 与 Host 之间的**唯一通信方式**。
/// Host 接收 Command 后，将其转换为实际的渲染、音频等操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// 显示/切换背景
    ShowBackground {
        /// 背景图片路径
        path: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 切换场景（可带 rule 遮罩）
    ChangeScene {
        /// 场景图片路径
        path: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 显示角色立绘
    ShowCharacter {
        /// 立绘图片路径
        path: String,
        /// 角色别名（用于后续引用）
        alias: String,
        /// 显示位置
        position: Position,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 隐藏角色立绘
    HideCharacter {
        /// 角色别名
        alias: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 显示对话文本
    ShowText {
        /// 说话者名称（None 表示旁白）
        speaker: Option<String>,
        /// 对话内容
        content: String,
    },

    /// 显示选择分支
    PresentChoices {
        /// 选择界面样式（从表头提取）
        style: Option<String>,
        /// 选项列表
        choices: Vec<Choice>,
    },

    /// 播放背景音乐
    PlayBgm {
        /// 音乐文件路径
        path: String,
        /// 是否循环播放
        looping: bool,
    },

    /// 停止背景音乐
    StopBgm {
        /// 淡出时长（秒），None 表示立即停止
        fade_out: Option<f64>,
    },

    /// 播放音效
    PlaySfx {
        /// 音效文件路径
        path: String,
    },

    /// 章节标记（用于显示章节过渡动画）
    ChapterMark {
        /// 章节标题
        title: String,
        /// 章节级别（1-6，对应 # 到 ######）
        level: u8,
    },

    /// 隐藏对话框（不影响背景/立绘）
    TextBoxHide,

    /// 显示对话框
    TextBoxShow,

    /// 清理对话框内容（对话/选择分支等）
    TextBoxClear,

    /// 清除所有角色立绘
    ClearCharacters,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_transition_simple() {
        let t = Transition::simple("dissolve");
        assert_eq!(t.name, "dissolve");
        assert!(t.args.is_empty());
    }

    #[test]
    fn test_transition_with_args() {
        let t = Transition::with_args("Dissolve", vec![TransitionArg::Number(1.5)]);
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
    }

    #[test]
    fn test_position_from_str() {
        assert_eq!(Position::from_str("left").ok(), Some(Position::Left));
        assert_eq!(Position::from_str("LEFT").ok(), Some(Position::Left));
        assert_eq!(Position::from_str("center").ok(), Some(Position::Center));
        assert_eq!(Position::from_str("middle").ok(), Some(Position::Center));
        assert_eq!(
            Position::from_str("nearleft").ok(),
            Some(Position::NearLeft)
        );
        assert_eq!(Position::from_str("unknown").ok(), None);
    }

    #[test]
    fn test_command_serialization() {
        let cmd = Command::ShowText {
            speaker: Some("羽艾".to_string()),
            content: "你好".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_transition_with_named_args() {
        let t = Transition::with_named_args(
            "Dissolve",
            vec![
                (Some("duration".to_string()), TransitionArg::Number(1.5)),
                (Some("reversed".to_string()), TransitionArg::Bool(true)),
            ],
        );
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 2);
        assert!(t.is_all_named());
        assert!(!t.is_all_positional());
    }

    #[test]
    fn test_transition_get_named() {
        let t = Transition::with_named_args(
            "Fade",
            vec![
                (Some("duration".to_string()), TransitionArg::Number(2.0)),
                (Some("reversed".to_string()), TransitionArg::Bool(false)),
            ],
        );

        // 按 key 获取命名参数
        assert_eq!(t.get_named("duration"), Some(&TransitionArg::Number(2.0)));
        assert_eq!(t.get_named("reversed"), Some(&TransitionArg::Bool(false)));
        assert_eq!(t.get_named("unknown"), None);
    }

    #[test]
    fn test_transition_get_duration_and_reversed_wrong_type_returns_none() {
        let t = Transition::with_named_args(
            "Any",
            vec![
                // duration 不是 Number
                (
                    Some("duration".to_string()),
                    TransitionArg::String("not-a-number".to_string()),
                ),
                // reversed 不是 Bool
                (Some("reversed".to_string()), TransitionArg::Number(1.0)),
            ],
        );

        assert_eq!(t.get_duration(), None);
        assert_eq!(t.get_reversed(), None);
    }

    #[test]
    fn test_transition_get_positional() {
        let t = Transition::with_args(
            "Effect",
            vec![
                TransitionArg::Number(1.0),
                TransitionArg::String("test".to_string()),
                TransitionArg::Bool(true),
            ],
        );

        // 按索引获取位置参数
        assert_eq!(t.get_positional(0), Some(&TransitionArg::Number(1.0)));
        assert_eq!(
            t.get_positional(1),
            Some(&TransitionArg::String("test".to_string()))
        );
        assert_eq!(t.get_positional(2), Some(&TransitionArg::Bool(true)));
        assert_eq!(t.get_positional(3), None);
        assert!(t.is_all_positional());
    }

    #[test]
    fn test_transition_get_arg_fallback() {
        // 命名参数优先
        let t = Transition::with_named_args(
            "Dissolve",
            vec![(Some("duration".to_string()), TransitionArg::Number(2.0))],
        );
        assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(2.0)));

        // 位置参数回退
        let t = Transition::with_args("Dissolve", vec![TransitionArg::Number(1.5)]);
        assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(1.5)));
    }

    #[test]
    fn test_transition_get_duration_and_reversed() {
        // 命名参数
        let t = Transition::with_named_args(
            "Fade",
            vec![
                (Some("duration".to_string()), TransitionArg::Number(2.5)),
                (Some("reversed".to_string()), TransitionArg::Bool(true)),
            ],
        );
        assert_eq!(t.get_duration(), Some(2.5));
        assert_eq!(t.get_reversed(), Some(true));

        // 位置参数
        let t = Transition::with_args(
            "Fade",
            vec![
                TransitionArg::Number(1.0),
                TransitionArg::String("mask".to_string()),
                TransitionArg::Bool(false),
            ],
        );
        assert_eq!(t.get_duration(), Some(1.0));
        assert_eq!(t.get_reversed(), Some(false));
    }

    #[test]
    fn test_transition_serialization_with_named_args() {
        let t = Transition::with_named_args(
            "Dissolve",
            vec![(Some("duration".to_string()), TransitionArg::Number(1.5))],
        );

        let json = serde_json::to_string(&t).unwrap();
        let deserialized: Transition = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deserialized);
        assert_eq!(deserialized.get_duration(), Some(1.5));
    }
}
