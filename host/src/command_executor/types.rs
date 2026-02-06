//! Command Executor 类型定义
//!
//! 定义执行结果、音频命令、效果请求等公开类型。

use crate::renderer::effects::EffectRequest;

/// Command 执行结果
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExecuteResult {
    /// 执行成功，继续
    #[default]
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

/// 命令执行输出
///
/// 每次 `CommandExecutor::execute()` 调用后，`last_output` 包含该命令产生的所有副作用请求。
/// 动画/过渡效果统一通过 `effect_requests` 传递给 `EffectApplier`。
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    /// 执行结果
    pub result: ExecuteResult,
    /// 动画/过渡效果请求（由 EffectApplier 消费）
    ///
    /// 替代原来的 `character_animation` / `scene_transition` / `transition_info` 三个字段，
    /// 统一为一个请求列表。
    pub effect_requests: Vec<EffectRequest>,
    /// 音频命令（如果有）
    pub audio_command: Option<AudioCommand>,
}
