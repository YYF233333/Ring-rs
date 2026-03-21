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
use std::collections::HashMap;
use std::str::FromStr;

use crate::state::VarValue;

/// changeScene 过渡完成的信号 ID
///
/// 当 changeScene 带过渡效果时，Runtime 进入 `WaitForSignal(SIGNAL_SCENE_TRANSITION)`,
/// Host 在过渡动画播放完毕后发送此信号以解除等待。
pub const SIGNAL_SCENE_TRANSITION: &str = "scene_transition";

/// sceneEffect 动画完成的信号 ID
///
/// 当 sceneEffect 带 duration 时，Runtime 进入 `WaitForSignal(SIGNAL_SCENE_EFFECT)`,
/// Host 在动画完成后发送此信号以解除等待。
pub const SIGNAL_SCENE_EFFECT: &str = "scene_effect";

/// titleCard 显示完成的信号 ID
pub const SIGNAL_TITLE_CARD: &str = "title_card";

/// cutscene 播放完成的信号 ID
///
/// 当 cutscene 命令执行时，Runtime 进入 `WaitForSignal(SIGNAL_CUTSCENE)`,
/// Host 在视频播放完毕或被跳过后发送此信号以解除等待。
pub const SIGNAL_CUTSCENE: &str = "cutscene";

/// 内联效果（对话文本中的节奏控制标签）
///
/// 标记在纯文本的字符位置上，由打字机推进时触发。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineEffect {
    /// 触发位置（纯文本中的字符索引，0-based）
    pub position: usize,
    /// 效果类型
    pub kind: InlineEffectKind,
}

/// 内联效果类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineEffectKind {
    /// `{wait}` 或 `{wait Ns}` -- 在此位置暂停打字机
    ///
    /// `None` = 等待点击；`Some(n)` = 暂停 n 秒后继续
    Wait(Option<f64>),
    /// `{speed N}` -- 设置绝对字速（字符/秒）
    SetCpsAbsolute(f64),
    /// `{speed Nx}` -- 设置相对字速（基础速度的倍率）
    SetCpsRelative(f64),
    /// `{/speed}` -- 重置字速到用户默认
    ResetCps,
}

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

/// 文本显示模式
///
/// 控制对话文本的显示方式。
/// - `ADV`（默认）：底部对话框，一次显示一句
/// - `NVL`：全屏半透明背景，文本逐行累积
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextMode {
    /// ADV 模式：底部对话框，一次一句
    #[default]
    ADV,
    /// NVL 模式：全屏文本累积
    NVL,
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
        /// 对话内容（纯文本，标签已剥离）
        content: String,
        /// 内联效果列表
        inline_effects: Vec<InlineEffect>,
        /// 是否自动推进（行尾 `-->` 修饰符）
        no_wait: bool,
    },

    /// 台词续接（不清屏追加文本）
    ExtendText {
        /// 追加文本（纯文本）
        content: String,
        /// 内联效果列表
        inline_effects: Vec<InlineEffect>,
        /// 是否自动推进
        no_wait: bool,
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

    /// 压低 BGM 音量（duck）
    BgmDuck,

    /// 恢复 BGM 音量（unduck）
    BgmUnduck,

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

    /// 场景效果（镜头语言）
    ///
    /// Host 收到此命令后应按 `name` 分发到对应的效果处理器。
    /// 参数格式复用 `Transition` 的 args 结构。
    SceneEffect {
        /// 效果名称（如 "shakeSmall", "blurIn"）
        name: String,
        /// 效果参数
        args: Vec<(Option<String>, TransitionArg)>,
    },

    /// 章节字卡
    ///
    /// Host 收到后全屏居中显示文字，淡入淡出后自动消失。
    TitleCard {
        /// 显示文本
        text: String,
        /// 显示时长（秒）
        duration: f64,
    },

    /// 完整重启游戏会话
    ///
    /// Host 收到此命令后应：
    /// 1. 将 `RuntimeState.persistent_variables` 持久化到 `saves/persistent.json`
    /// 2. 清空当前游戏会话
    /// 3. 返回标题画面
    FullRestart,

    /// 播放过场视频
    ///
    /// Host 收到后全屏播放指定视频。播放完毕或玩家跳过后，
    /// 发送 `Signal("cutscene")` 恢复 Runtime。
    /// FFmpeg 不可用或文件不存在时应优雅降级（跳过 + 警告）。
    Cutscene {
        /// 视频文件路径
        path: String,
    },

    /// 切换文本显示模式
    ///
    /// Host 收到后切换对话渲染方式。
    /// 切换到 NVL 时开始累积对话文本。
    /// 切换到 ADV 时清空累积的 NVL 文本。
    SetTextMode(TextMode),

    /// 请求 Host 展示自定义 UI 并等待用户交互
    ///
    /// Host 收到后应根据 `mode` 展示对应 UI，用户完成交互后
    /// 通过 `RuntimeInput::UIResult { key, value }` 回传结果。
    RequestUI {
        /// 请求标识符（用于匹配响应）
        key: String,
        /// UI 模式标识（Host 据此选择展示哪种 UI）
        mode: String,
        /// 模式特定参数
        params: HashMap<String, VarValue>,
    },
}

#[cfg(test)]
mod tests;
