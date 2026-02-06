//! Command Executor 类型定义
//!
//! 定义执行结果、音频命令、过渡信息等公开类型。

use crate::renderer::effects::ResolvedEffect;

/// Command 执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum ExecuteResult {
    /// 执行成功，继续
    Ok,
    /// 执行成功，需要等待用户输入（对话显示完成后）
    WaitForClick,
    /// 执行成功，需要等待用户选择
    WaitForChoice { choice_count: usize },
    /// 执行成功，需要等待指定时长（毫秒）
    WaitForTime(u64),
    /// 资源加载中
    Loading,
    /// 执行失败
    Error(String),
}

impl Default for ExecuteResult {
    fn default() -> Self {
        Self::Ok
    }
}

/// 音频命令
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// 播放 BGM
    PlayBgm {
        path: String,
        looping: bool,
        fade_in: Option<f32>,
    },
    /// 停止 BGM
    StopBgm { fade_out: Option<f32> },
    /// 播放 SFX
    PlaySfx { path: String },
}

/// 过渡效果信息
///
/// 阶段 25 重构：使用 `ResolvedEffect` 替代 raw `Transition`，
/// 避免下游再次解析效果参数。
#[derive(Debug, Clone, Default)]
pub struct TransitionInfo {
    /// 是否有背景过渡
    pub has_background_transition: bool,
    /// 旧背景路径
    pub old_background: Option<String>,
    /// 已解析的过渡效果
    pub effect: Option<ResolvedEffect>,
}

/// 角色动画命令
#[derive(Debug, Clone)]
pub enum CharacterAnimationCommand {
    /// 显示角色（淡入）
    Show { alias: String, duration: f32 },
    /// 隐藏角色（淡出）
    Hide { alias: String, duration: f32 },
    /// 移动角色到新位置（位置变更动画）
    Move {
        alias: String,
        old_position: vn_runtime::command::Position,
        new_position: vn_runtime::command::Position,
        duration: f32,
    },
}

/// 场景切换命令
///
/// 由 main.rs 调用 `Renderer.start_scene_*()` 方法处理
#[derive(Debug, Clone)]
pub enum SceneTransitionCommand {
    /// Fade（黑屏）过渡
    Fade {
        duration: f32,
        pending_background: String,
    },
    /// FadeWhite（白屏）过渡
    FadeWhite {
        duration: f32,
        pending_background: String,
    },
    /// Rule（图片遮罩）过渡
    Rule {
        duration: f32,
        pending_background: String,
        mask_path: String,
        reversed: bool,
    },
}

/// 命令执行输出
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    /// 执行结果
    pub result: ExecuteResult,
    /// 过渡信息
    pub transition_info: TransitionInfo,
    /// 音频命令（如果有）
    pub audio_command: Option<AudioCommand>,
    /// 角色动画命令（如果有）
    pub character_animation: Option<CharacterAnimationCommand>,
    /// 场景切换命令（如果有）
    pub scene_transition: Option<SceneTransitionCommand>,
}
