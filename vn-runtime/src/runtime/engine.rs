//! # Engine 模块
//!
//! VN Runtime 核心执行引擎。
//!
//! ## 执行模型
//!
//! ```text
//! tick(input) -> (Vec<Command>, WaitingReason)
//! ```
//!
//! 1. 检查当前等待状态
//! 2. 根据 input 决定是否解除等待
//! 3. 若不再等待，继续执行脚本直到下一个阻塞点
//! 4. 返回执行过程中产生的 Command 和新的等待状态

use crate::command::Command;
use crate::error::RuntimeError;
use crate::input::RuntimeInput;
use crate::runtime::executor::Executor;
use crate::script::{Script, ScriptNode};
use crate::state::{RuntimeState, WaitingReason};

/// VN Runtime 执行引擎
///
/// 这是 vn-runtime 的核心类型，负责驱动脚本执行。
///
/// # 使用示例
///
/// ```ignore
/// let script = Script::parse(text)?;
/// let mut runtime = VNRuntime::new(script);
///
/// loop {
///     let (commands, waiting) = runtime.tick(input)?;
///     
///     // Host 执行 commands...
///     
///     // 根据 waiting 采集输入...
/// }
/// ```
pub struct VNRuntime {
    /// 当前脚本
    script: Script,
    /// 运行时状态
    state: RuntimeState,
    /// 节点执行器
    executor: Executor,
}

impl VNRuntime {
    /// 创建新的 Runtime 实例
    ///
    /// # 参数
    ///
    /// - `script`: 已解析的脚本
    pub fn new(script: Script) -> Self {
        let state = RuntimeState::new(&script.id);
        Self {
            script,
            state,
            executor: Executor::new(),
        }
    }

    /// 从保存的状态恢复 Runtime
    ///
    /// # 参数
    ///
    /// - `script`: 脚本（必须与保存时相同）
    /// - `state`: 保存的运行时状态
    pub fn restore(script: Script, state: RuntimeState) -> Self {
        Self {
            script,
            state,
            executor: Executor::new(),
        }
    }

    /// 核心驱动函数
    ///
    /// 根据输入推进脚本执行，返回产生的 Command 和新的等待状态。
    ///
    /// # 参数
    ///
    /// - `input`: Host 传入的输入（可选）
    ///
    /// # 返回
    ///
    /// - `Vec<Command>`: 本次 tick 产生的所有指令
    /// - `WaitingReason`: 新的等待状态
    pub fn tick(
        &mut self,
        input: Option<RuntimeInput>,
    ) -> Result<(Vec<Command>, WaitingReason), RuntimeError> {
        let mut commands = Vec::new();

        // 1. 处理输入，尝试解除等待
        if let Some(input) = input {
            self.handle_input(input)?;
        }

        // 2. 如果仍在等待，直接返回
        if self.state.waiting.is_waiting() {
            return Ok((commands, self.state.waiting.clone()));
        }

        // 3. 继续执行脚本直到阻塞或结束
        loop {
            // 检查是否到达脚本末尾
            let node = match self.script.get_node(self.state.position.node_index) {
                Some(node) => node.clone(),
                None => {
                    // 脚本执行完毕
                    return Ok((commands, WaitingReason::None));
                }
            };

            // 执行当前节点
            let (node_commands, should_wait) =
                self.executor.execute(&node, &mut self.state, &self.script)?;

            commands.extend(node_commands);

            // 前进到下一个节点
            self.state.position.advance();

            // 如果需要等待，停止执行
            if let Some(reason) = should_wait {
                self.state.wait(reason.clone());
                return Ok((commands, reason));
            }
        }
    }

    /// 处理输入，解除等待状态
    fn handle_input(&mut self, input: RuntimeInput) -> Result<(), RuntimeError> {
        match (&self.state.waiting, input) {
            // 点击解除 WaitForClick
            (WaitingReason::WaitForClick, RuntimeInput::Click) => {
                self.state.clear_wait();
                Ok(())
            }

            // 选择解除 WaitForChoice
            (
                WaitingReason::WaitForChoice { choice_count },
                RuntimeInput::ChoiceSelected { index },
            ) => {
                if index >= *choice_count {
                    return Err(RuntimeError::InvalidChoiceIndex {
                        index,
                        max: *choice_count,
                    });
                }

                // 跳转到对应标签
                // 需要从当前节点获取选项信息
                let current_index = self.state.position.node_index.saturating_sub(1);
                if let Some(ScriptNode::Choice { options, .. }) = self.script.get_node(current_index)
                {
                    if let Some(option) = options.get(index) {
                        let target_index = self.script.find_label(&option.target_label).ok_or(
                            RuntimeError::LabelNotFound {
                                label: option.target_label.clone(),
                            },
                        )?;
                        self.state.position.jump_to(target_index);
                    }
                }

                self.state.clear_wait();
                Ok(())
            }

            // 信号解除 WaitForSignal
            (WaitingReason::WaitForSignal(expected_id), RuntimeInput::Signal { id }) => {
                if id == *expected_id {
                    self.state.clear_wait();
                }
                Ok(())
            }

            // WaitForTime 由 Host 处理，收到任何输入都不解除
            (WaitingReason::WaitForTime(_), _) => Ok(()),

            // 不等待时收到输入，忽略
            (WaitingReason::None, _) => Ok(()),

            // 状态不匹配
            (waiting, input) => Err(RuntimeError::StateMismatch {
                expected: format!("{:?}", waiting),
                actual: format!("{:?}", input),
            }),
        }
    }

    /// 获取当前状态（用于存档）
    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    /// 获取当前等待状态
    pub fn waiting(&self) -> &WaitingReason {
        &self.state.waiting
    }

    /// 检查脚本是否执行完毕
    pub fn is_finished(&self) -> bool {
        self.state.position.node_index >= self.script.len() && !self.state.waiting.is_waiting()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::ScriptNode;

    fn create_test_script() -> Script {
        Script::new(
            "test",
            vec![
                ScriptNode::Dialogue {
                    speaker: Some("Test".to_string()),
                    content: "Hello".to_string(),
                },
                ScriptNode::Dialogue {
                    speaker: None,
                    content: "World".to_string(),
                },
            ],
        )
    }

    #[test]
    fn test_runtime_creation() {
        let script = create_test_script();
        let runtime = VNRuntime::new(script);

        assert_eq!(runtime.state().position.node_index, 0);
        assert!(!runtime.state().waiting.is_waiting());
    }

    #[test]
    fn test_runtime_tick_dialogue() {
        let script = create_test_script();
        let mut runtime = VNRuntime::new(script);

        // 第一次 tick，执行第一个对话
        let (commands, waiting) = runtime.tick(None).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            Command::ShowText { speaker: Some(s), content }
            if s == "Test" && content == "Hello"
        ));
        assert!(matches!(waiting, WaitingReason::WaitForClick));

        // 发送点击输入
        let (commands, waiting) = runtime.tick(Some(RuntimeInput::Click)).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            Command::ShowText { speaker: None, content }
            if content == "World"
        ));
        assert!(matches!(waiting, WaitingReason::WaitForClick));
    }

    #[test]
    fn test_runtime_script_end() {
        let script = Script::new(
            "test",
            vec![ScriptNode::ChangeBG {
                path: "bg.png".to_string(),
                transition: None,
            }],
        );
        let mut runtime = VNRuntime::new(script);

        // 执行完毕
        let (commands, waiting) = runtime.tick(None).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(waiting, WaitingReason::None));
        assert!(runtime.is_finished());
    }
}

