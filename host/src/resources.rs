//! 资源定义模块

use bevy::prelude::*;
use vn_runtime::{
    runtime::VNRuntime,
    state::WaitingReason,
    Command as RTCommand,
};

/// VN 引擎状态资源
#[derive(Resource)]
pub struct VNState {
    /// Runtime 实例
    pub runtime: Option<VNRuntime>,
    /// 当前等待原因
    pub waiting: WaitingReason,
    /// 待处理的命令队列
    pub pending_commands: Vec<RTCommand>,
}

impl Default for VNState {
    fn default() -> Self {
        Self {
            runtime: None,
            waiting: WaitingReason::None,
            pending_commands: Vec::new(),
        }
    }
}

impl VNState {
    /// 检查是否正在等待玩家点击
    pub fn is_waiting_for_click(&self) -> bool {
        matches!(self.waiting, WaitingReason::WaitForClick)
    }

    /// 检查是否正在等待玩家选择
    pub fn is_waiting_for_choice(&self) -> bool {
        matches!(self.waiting, WaitingReason::WaitForChoice { .. })
    }

    /// 检查是否处于空闲状态（可以继续执行）
    pub fn is_idle(&self) -> bool {
        matches!(self.waiting, WaitingReason::None)
    }
}

/// 对话框状态
#[derive(Resource, Default)]
pub struct DialogueState {
    /// 当前说话者
    pub speaker: Option<String>,
    /// 当前对话内容
    pub content: String,
    /// 是否显示对话框
    pub visible: bool,
    /// 选项列表（用于选择分支）
    pub choices: Vec<ChoiceItem>,
}

/// 选择项
#[derive(Clone)]
pub struct ChoiceItem {
    pub text: String,
    pub index: usize,
}

/// VN 命令消息（从 Runtime 到 Host）
#[derive(Message, Clone)]
pub struct VNCommand(pub RTCommand);

/// 玩家输入消息
#[derive(Message, Clone)]
pub enum PlayerInput {
    /// 点击（继续）
    Click,
    /// 选择选项
    Select(usize),
}
