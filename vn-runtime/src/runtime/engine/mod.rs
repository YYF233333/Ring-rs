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
use crate::history::{History, HistoryEvent};
use crate::input::RuntimeInput;
use crate::runtime::executor::{Executor, ScriptControlFlow};
use crate::script::{Script, ScriptNode};
use crate::state::{RuntimeState, WaitingReason};
use std::collections::HashMap;

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
    /// 已注册脚本（key=逻辑路径或脚本 id）
    script_registry: HashMap<String, Script>,
    /// 运行时状态
    state: RuntimeState,
    /// 节点执行器
    executor: Executor,
    /// 历史记录
    history: History,
}

impl VNRuntime {
    /// 创建新的 Runtime 实例
    ///
    /// # 参数
    ///
    /// - `script`: 已解析的脚本
    pub fn new(script: Script) -> Self {
        let mut script_registry = HashMap::new();
        script_registry.insert(script.id.clone(), script.clone());
        let state = RuntimeState::new(&script.id);
        Self {
            script,
            script_registry,
            state,
            history: History::new(),
            executor: Executor::new(),
        }
    }

    /// 从保存的状态恢复 Runtime
    ///
    /// # 参数
    ///
    /// - `script`: 脚本（必须与保存时相同）
    /// - `state`: 保存的运行时状态
    /// - `history`: 历史记录
    pub fn restore(script: Script, state: RuntimeState, history: History) -> Self {
        let mut script_registry = HashMap::new();
        script_registry.insert(script.id.clone(), script.clone());
        Self {
            script,
            script_registry,
            state,
            history,
            executor: Executor::new(),
        }
    }

    /// 注册一个可被 callScript 调用的脚本
    ///
    /// `logical_path` 应为相对 assets_root 的规范化路径，例如 `scripts/remake/main.md`。
    pub fn register_script(&mut self, logical_path: impl Into<String>, script: Script) {
        let key = logical_path.into();
        self.script_registry.insert(key, script.clone());
        // 兼容按 script_id 查询（用于旧状态）
        self.script_registry
            .entry(script.id.clone())
            .or_insert(script);
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
                    // 当前脚本执行完毕：
                    // - 若在调用栈中，则自动 return 到调用点
                    // - 否则视为入口脚本结束
                    if !self.state.call_stack.is_empty() {
                        self.handle_script_control(ScriptControlFlow::Return)?;
                        continue;
                    }
                    return Ok((commands, WaitingReason::None));
                }
            };

            // 执行当前节点
            let result = self
                .executor
                .execute(&node, &mut self.state, &self.script)?;

            // 记录历史事件
            for cmd in &result.commands {
                self.record_history(cmd);
            }

            commands.extend(result.commands);

            if let Some(control) = result.script_control {
                self.handle_script_control(control)?;
                continue;
            }

            // 处理跳转（记录历史）
            if let Some(target) = result.jump_to {
                // 从当前节点获取跳转目标标签（用于历史记录）
                if let ScriptNode::Goto { target_label } = &node {
                    self.history.push(HistoryEvent::jump(target_label.clone()));
                }
                self.state.position.jump_to(target);
                continue; // 继续执行跳转目标
            }

            // 前进到下一个节点
            self.state.position.advance();

            // 如果需要等待，停止执行
            if let Some(reason) = result.waiting {
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
                if let Some(ScriptNode::Choice { options, .. }) =
                    self.script.get_node(current_index)
                {
                    // 记录选择事件到历史
                    let option_texts: Vec<String> =
                        options.iter().map(|o| o.text.clone()).collect();
                    self.history
                        .push(HistoryEvent::choice_made(option_texts, index));

                    if let Some(option) = options.get(index) {
                        // 记录跳转事件
                        self.history
                            .push(HistoryEvent::jump(option.target_label.clone()));

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

            // WaitForTime: Click 可以打断等待（用于 wait 指令的交互打断）
            (WaitingReason::WaitForTime(_), RuntimeInput::Click) => {
                self.state.clear_wait();
                Ok(())
            }
            // WaitForTime: 其他输入忽略
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

    fn handle_script_control(&mut self, control: ScriptControlFlow) -> Result<(), RuntimeError> {
        match control {
            ScriptControlFlow::Call {
                target_path,
                display_label,
            } => {
                let resolved_path = self.script.resolve_path(&target_path);
                let target_script = self.script_registry.get(&resolved_path).cloned().ok_or(
                    RuntimeError::ScriptNotLoaded {
                        path: resolved_path.clone(),
                    },
                )?;

                let return_position = crate::state::ScriptPosition::with_path(
                    self.state.position.script_id.clone(),
                    self.state.position.script_path.clone(),
                    self.state.position.node_index + 1,
                );
                self.state.call_stack.push(return_position);

                // `callScript [label](path)` 中的 label 仅用于展示，不参与入口寻址。
                let target_index = 0;

                self.history.push(HistoryEvent::jump(format!(
                    "call {}{}",
                    resolved_path,
                    display_label
                        .as_ref()
                        .map(|l| format!(" [{}]", l))
                        .unwrap_or_default()
                )));

                self.script = target_script;
                self.state.position.script_id = self.script.id.clone();
                self.state.position.script_path = resolved_path;
                self.state.position.node_index = target_index;
                Ok(())
            }
            ScriptControlFlow::Return => {
                let return_position =
                    self.state
                        .call_stack
                        .pop()
                        .ok_or_else(|| RuntimeError::InvalidState {
                            message: "returnFromScript 但调用栈为空".to_string(),
                        })?;

                let next_script = if !return_position.script_path.is_empty() {
                    self.script_registry
                        .get(&return_position.script_path)
                        .cloned()
                        .or_else(|| {
                            self.script_registry
                                .get(&return_position.script_id)
                                .cloned()
                        })
                        .ok_or_else(|| RuntimeError::ScriptNotLoaded {
                            path: return_position.script_path.clone(),
                        })?
                } else {
                    self.script_registry
                        .get(&return_position.script_id)
                        .cloned()
                        .ok_or(RuntimeError::ScriptNotLoaded {
                            path: return_position.script_id.clone(),
                        })?
                };

                self.history.push(HistoryEvent::jump("return".to_string()));

                self.script = next_script;
                self.state.position = return_position;
                Ok(())
            }
        }
    }

    /// 获取当前状态（用于存档）
    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    /// 获取可变状态引用
    pub fn state_mut(&mut self) -> &mut RuntimeState {
        &mut self.state
    }

    /// 恢复状态（用于读档）
    ///
    /// 将 Runtime 状态恢复到指定状态。
    /// 注意：调用方需要确保 state 中的 script_id 与当前加载的脚本匹配。
    pub fn restore_state(&mut self, state: RuntimeState) {
        self.state = state;
    }

    /// 获取当前等待状态
    pub fn waiting(&self) -> &WaitingReason {
        &self.state.waiting
    }

    /// 检查脚本是否执行完毕
    pub fn is_finished(&self) -> bool {
        self.state.position.node_index >= self.script.len() && !self.state.waiting.is_waiting()
    }

    /// 获取历史记录
    pub fn history(&self) -> &History {
        &self.history
    }

    /// 恢复历史记录（用于读档）
    pub fn restore_history(&mut self, history: History) {
        self.history = history;
    }

    /// 根据 Command 记录历史事件
    fn record_history(&mut self, cmd: &Command) {
        match cmd {
            Command::ShowText { speaker, content } => {
                self.history
                    .push(HistoryEvent::dialogue(speaker.clone(), content.clone()));
            }
            Command::ChapterMark { title, .. } => {
                self.history.push(HistoryEvent::chapter_mark(title.clone()));
            }
            Command::ShowBackground { path, .. } => {
                self.history
                    .push(HistoryEvent::background_change(path.clone()));
            }
            Command::PlayBgm { path, .. } => {
                self.history
                    .push(HistoryEvent::bgm_change(Some(path.clone())));
            }
            Command::StopBgm { .. } => {
                self.history.push(HistoryEvent::bgm_change(None));
            }
            // 其他命令不记录历史（角色显示/隐藏、音效等）
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
