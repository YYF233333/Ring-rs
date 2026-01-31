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
///
/// # 示例
///
/// ```text
/// with dissolve           -> Transition { name: "dissolve", args: [] }
/// with Dissolve(1.5)      -> Transition { name: "Dissolve", args: [Number(1.5)] }
/// with rule("mask.png")   -> Transition { name: "rule", args: [String("mask.png")] }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// 效果名称（大小写敏感）
    pub name: String,
    /// 效果参数列表
    pub args: Vec<TransitionArg>,
}

impl Transition {
    /// 创建无参数的过渡效果
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    /// 创建带参数的过渡效果
    pub fn with_args(name: impl Into<String>, args: Vec<TransitionArg>) -> Self {
        Self {
            name: name.into(),
            args,
        }
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
    /// 从字符串解析位置
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "center" | "middle" => Some(Self::Center),
            "nearleft" => Some(Self::NearLeft),
            "nearright" => Some(Self::NearRight),
            "nearmiddle" => Some(Self::NearMiddle),
            "farleft" => Some(Self::FarLeft),
            "farright" => Some(Self::FarRight),
            "farmiddle" => Some(Self::FarMiddle),
            _ => None,
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

    /// 触发 UI 动画
    UIAnimation {
        /// 动画效果
        effect: Transition,
    },

    /// 章节标记（用于显示章节过渡动画）
    ChapterMark {
        /// 章节标题
        title: String,
        /// 章节级别（1-6，对应 # 到 ######）
        level: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_simple() {
        let t = Transition::simple("dissolve");
        assert_eq!(t.name, "dissolve");
        assert!(t.args.is_empty());
    }

    #[test]
    fn test_transition_with_args() {
        let t = Transition::with_args(
            "Dissolve",
            vec![TransitionArg::Number(1.5)],
        );
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
    }

    #[test]
    fn test_position_from_str() {
        assert_eq!(Position::from_str("left"), Some(Position::Left));
        assert_eq!(Position::from_str("LEFT"), Some(Position::Left));
        assert_eq!(Position::from_str("center"), Some(Position::Center));
        assert_eq!(Position::from_str("middle"), Some(Position::Center));
        assert_eq!(Position::from_str("nearleft"), Some(Position::NearLeft));
        assert_eq!(Position::from_str("unknown"), None);
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
}

