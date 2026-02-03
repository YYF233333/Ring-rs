//! # State 模块
//!
//! 定义 Runtime 的运行时状态和等待模型。
//!
//! ## 设计原则
//!
//! - 所有状态必须**显式建模**
//! - 所有状态必须**可序列化**（支持存档/读档）
//! - 不允许隐式全局状态

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::input::SignalId;

/// 等待原因
///
/// Runtime 在执行过程中可能进入等待状态，需要特定输入才能继续。
/// Host 根据此状态决定如何采集输入。
///
/// # 状态转换
///
/// ```text
/// None          -> 继续执行，不等待
/// WaitForClick  -> 等待用户点击，收到 Click 输入后继续
/// WaitForChoice -> 等待用户选择，收到 ChoiceSelected 输入后继续
/// WaitForTime   -> Host 等待指定时长后调用 tick，Runtime 自动继续
/// WaitForSignal -> 等待外部信号，收到匹配的 Signal 输入后继续
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaitingReason {
    /// 不等待，继续执行
    None,

    /// 等待用户点击
    WaitForClick,

    /// 等待用户选择
    ///
    /// `choice_count` 记录选项数量，用于验证输入合法性
    WaitForChoice { choice_count: usize },

    /// 等待指定时长
    ///
    /// Host 获取此状态后，等待指定时长再调用 tick。
    /// Runtime 不需要知道真实时间流逝。
    WaitForTime(Duration),

    /// 等待外部信号
    WaitForSignal(SignalId),
}

impl WaitingReason {
    /// 是否处于等待状态
    pub fn is_waiting(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// 创建等待点击状态
    pub fn click() -> Self {
        Self::WaitForClick
    }

    /// 创建等待选择状态
    pub fn choice(count: usize) -> Self {
        Self::WaitForChoice {
            choice_count: count,
        }
    }

    /// 创建等待时间状态
    pub fn time(duration: Duration) -> Self {
        Self::WaitForTime(duration)
    }

    /// 创建等待信号状态
    pub fn signal(id: impl Into<SignalId>) -> Self {
        Self::WaitForSignal(id.into())
    }
}

impl Default for WaitingReason {
    fn default() -> Self {
        Self::None
    }
}

/// 脚本变量值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VarValue {
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Bool(bool),
}

/// 脚本执行位置
///
/// 记录当前执行到脚本的哪个位置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPosition {
    /// 脚本标识符（文件名，不含路径和扩展名）
    pub script_id: String,
    /// 脚本完整路径（相对于 assets_root，用于加载脚本）
    ///
    /// 例如：`scripts/chapter1/intro.md`
    ///
    /// 注意：为了向后兼容，此字段可选。如果为空，则尝试从 `script_id` 推断路径。
    #[serde(default)]
    pub script_path: String,
    /// 当前执行的节点索引
    pub node_index: usize,
}

impl ScriptPosition {
    /// 创建新的脚本位置
    pub fn new(script_id: impl Into<String>, node_index: usize) -> Self {
        let id = script_id.into();
        Self {
            script_id: id.clone(),
            script_path: String::new(), // 默认为空，由 host 在加载时设置
            node_index,
        }
    }

    /// 创建带完整路径的脚本位置
    pub fn with_path(
        script_id: impl Into<String>,
        script_path: impl Into<String>,
        node_index: usize,
    ) -> Self {
        Self {
            script_id: script_id.into(),
            script_path: script_path.into(),
            node_index,
        }
    }

    /// 创建默认位置（脚本开头）
    pub fn start(script_id: impl Into<String>) -> Self {
        Self::new(script_id, 0)
    }

    /// 创建带路径的默认位置（脚本开头）
    pub fn start_with_path(script_id: impl Into<String>, script_path: impl Into<String>) -> Self {
        Self::with_path(script_id, script_path, 0)
    }

    /// 设置脚本路径
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.script_path = path.into();
    }

    /// 前进到下一个节点
    pub fn advance(&mut self) {
        self.node_index += 1;
    }

    /// 跳转到指定位置
    pub fn jump_to(&mut self, index: usize) {
        self.node_index = index;
    }
}

/// Runtime 状态
///
/// 这是 Runtime 的**唯一可变状态**，包含所有运行时信息。
/// 所有字段都可序列化，支持存档/读档。
///
/// # 设计说明
///
/// - `position`：脚本执行位置
/// - `variables`：脚本变量（用于条件分支等）
/// - `waiting`：当前等待状态
/// - `visible_characters`：当前显示的角色（用于状态恢复）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeState {
    /// 脚本执行位置
    pub position: ScriptPosition,

    /// 脚本变量
    pub variables: HashMap<String, VarValue>,

    /// 当前等待状态
    pub waiting: WaitingReason,

    /// 当前显示的角色立绘
    /// Key: alias, Value: (path, position)
    pub visible_characters: HashMap<String, (String, crate::command::Position)>,

    /// 当前背景
    pub current_background: Option<String>,
}

impl RuntimeState {
    /// 创建新的运行时状态
    pub fn new(script_id: impl Into<String>) -> Self {
        Self {
            position: ScriptPosition::start(script_id),
            variables: HashMap::new(),
            waiting: WaitingReason::None,
            visible_characters: HashMap::new(),
            current_background: None,
        }
    }

    /// 设置变量
    pub fn set_var(&mut self, name: impl Into<String>, value: VarValue) {
        self.variables.insert(name.into(), value);
    }

    /// 获取变量
    pub fn get_var(&self, name: &str) -> Option<&VarValue> {
        self.variables.get(name)
    }

    /// 进入等待状态
    pub fn wait(&mut self, reason: WaitingReason) {
        self.waiting = reason;
    }

    /// 清除等待状态
    pub fn clear_wait(&mut self) {
        self.waiting = WaitingReason::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waiting_reason() {
        assert!(!WaitingReason::None.is_waiting());
        assert!(WaitingReason::click().is_waiting());
        assert!(WaitingReason::choice(3).is_waiting());
        assert!(WaitingReason::time(Duration::from_secs(1)).is_waiting());
        assert!(WaitingReason::signal("test").is_waiting());
    }

    #[test]
    fn test_script_position() {
        let mut pos = ScriptPosition::start("main");
        assert_eq!(pos.node_index, 0);
        assert_eq!(pos.script_path, "");

        pos.advance();
        assert_eq!(pos.node_index, 1);

        pos.jump_to(10);
        assert_eq!(pos.node_index, 10);

        // 测试带路径的位置
        let pos_with_path = ScriptPosition::start_with_path("intro", "scripts/chapter1/intro.md");
        assert_eq!(pos_with_path.script_id, "intro");
        assert_eq!(pos_with_path.script_path, "scripts/chapter1/intro.md");
    }

    #[test]
    fn test_runtime_state() {
        let mut state = RuntimeState::new("main");
        assert_eq!(state.position.script_id, "main");
        assert_eq!(state.position.node_index, 0);
        assert!(!state.waiting.is_waiting());

        state.set_var("counter", VarValue::Int(42));
        assert_eq!(state.get_var("counter"), Some(&VarValue::Int(42)));

        state.wait(WaitingReason::click());
        assert!(state.waiting.is_waiting());

        state.clear_wait();
        assert!(!state.waiting.is_waiting());
    }

    #[test]
    fn test_state_serialization() {
        let mut state = RuntimeState::new("main");
        state.set_var("name", VarValue::String("test".to_string()));
        state.wait(WaitingReason::choice(3));

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RuntimeState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}
