//! # Executor 模块
//!
//! 将 AST 节点转换为 Command。
//!
//! ## 职责
//!
//! - 读取 ScriptNode
//! - 产生对应的 Command
//! - 决定是否需要等待

use crate::command::{Choice, Command, SIGNAL_CUTSCENE, SIGNAL_SCENE_EFFECT, SIGNAL_TITLE_CARD};
use crate::error::RuntimeError;
use crate::input::SignalId;
use crate::script::{EvalError, Script, ScriptNode, evaluate, evaluate_to_bool};
use crate::state::{RuntimeState, WaitingReason};

/// 脚本控制流动作（不经过 Host 命令层）
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptControlFlow {
    /// 调用其他脚本
    Call {
        target_path: String,
        display_label: Option<String>,
    },
    /// 返回调用点
    Return,
}

/// 执行结果
pub struct ExecuteResult {
    /// 产生的命令
    pub commands: Vec<Command>,
    /// 等待原因（如果需要等待）
    pub waiting: Option<WaitingReason>,
    /// 跳转目标（如果需要跳转）
    pub jump_to: Option<usize>,
    /// 脚本控制流动作（callScript / returnFromScript）
    pub script_control: Option<ScriptControlFlow>,
}

impl ExecuteResult {
    /// 创建空结果
    fn empty() -> Self {
        Self {
            commands: Vec::new(),
            waiting: None,
            jump_to: None,
            script_control: None,
        }
    }

    /// 创建带命令的结果
    fn with_commands(commands: Vec<Command>) -> Self {
        Self {
            commands,
            waiting: None,
            jump_to: None,
            script_control: None,
        }
    }

    /// 创建带等待的结果
    fn with_wait(commands: Vec<Command>, waiting: WaitingReason) -> Self {
        Self {
            commands,
            waiting: Some(waiting),
            jump_to: None,
            script_control: None,
        }
    }

    /// 创建跳转结果
    fn with_jump(jump_to: usize) -> Self {
        Self {
            commands: Vec::new(),
            waiting: None,
            jump_to: Some(jump_to),
            script_control: None,
        }
    }

    fn with_script_control(script_control: ScriptControlFlow) -> Self {
        Self {
            commands: Vec::new(),
            waiting: None,
            jump_to: None,
            script_control: Some(script_control),
        }
    }
}

/// 节点执行器
///
/// 负责将单个 ScriptNode 转换为 Command。
pub struct Executor {
    // 未来可添加执行上下文
}

#[allow(clippy::new_without_default)]
impl Executor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {}
    }

    /// 执行单个脚本节点
    ///
    /// # 返回
    ///
    /// `ExecuteResult` 包含：
    /// - `commands`: 产生的指令
    /// - `waiting`: 如果需要等待，返回等待原因
    /// - `jump_to`: 如果需要跳转，返回目标位置
    pub fn execute(
        &mut self,
        node: &ScriptNode,
        state: &mut RuntimeState,
        script: &Script,
    ) -> Result<ExecuteResult, RuntimeError> {
        match node {
            ScriptNode::Chapter { title, level } => {
                Ok(ExecuteResult::with_commands(vec![Command::ChapterMark {
                    title: title.clone(),
                    level: *level,
                }]))
            }

            ScriptNode::Label { .. } => {
                // 标签节点不产生 Command，只是跳转目标
                Ok(ExecuteResult::empty())
            }

            ScriptNode::Dialogue {
                speaker,
                content,
                inline_effects,
                no_wait,
            } => Ok(ExecuteResult::with_wait(
                vec![Command::ShowText {
                    speaker: speaker.clone(),
                    content: content.clone(),
                    inline_effects: inline_effects.clone(),
                    no_wait: *no_wait,
                }],
                WaitingReason::WaitForClick,
            )),

            ScriptNode::Extend {
                content,
                inline_effects,
                no_wait,
            } => Ok(ExecuteResult::with_wait(
                vec![Command::ExtendText {
                    content: content.clone(),
                    inline_effects: inline_effects.clone(),
                    no_wait: *no_wait,
                }],
                WaitingReason::WaitForClick,
            )),

            ScriptNode::ChangeBG { path, transition } => {
                // 解析路径（相对于脚本目录）
                let resolved_path = script.resolve_path(path);
                // 更新状态
                state.current_background = Some(resolved_path.clone());

                Ok(ExecuteResult::with_commands(vec![
                    Command::ShowBackground {
                        path: resolved_path,
                        transition: transition.clone(),
                    },
                ]))
            }

            ScriptNode::ChangeScene { path, transition } => {
                // 解析路径（相对于脚本目录）
                let resolved_path = script.resolve_path(path);
                // 更新状态
                state.current_background = Some(resolved_path.clone());

                // 解析 transition 中的路径参数（如 rule 效果的 mask 路径）
                let resolved_transition = transition.as_ref().map(|t| {
                    let mut new_transition = t.clone();
                    // 遍历参数，解析路径类型的字符串
                    new_transition.args = t
                        .args
                        .iter()
                        .map(|(key, arg)| {
                            if let Some(k) = key {
                                // 命名参数：mask 参数需要解析路径
                                if k == "mask"
                                    && let crate::command::TransitionArg::String(s) = arg
                                {
                                    return (
                                        Some(k.clone()),
                                        crate::command::TransitionArg::String(
                                            script.resolve_path(s),
                                        ),
                                    );
                                }
                            }
                            (key.clone(), arg.clone())
                        })
                        .collect();
                    new_transition
                });

                let commands = vec![Command::ChangeScene {
                    path: resolved_path,
                    transition: resolved_transition.clone(),
                }];

                if resolved_transition.is_some() {
                    Ok(ExecuteResult::with_wait(
                        commands,
                        WaitingReason::WaitForSignal(SignalId::new(
                            crate::command::SIGNAL_SCENE_TRANSITION,
                        )),
                    ))
                } else {
                    Ok(ExecuteResult::with_commands(commands))
                }
            }

            ScriptNode::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => {
                // 如果 path 为 None，尝试从已绑定的别名中查找
                let resolved_path = if let Some(p) = path {
                    // 有路径：解析路径（相对于脚本目录）并绑定别名
                    let resolved = script.resolve_path(p);
                    // 更新状态
                    state
                        .visible_characters
                        .insert(alias.clone(), (resolved.clone(), *position));
                    resolved
                } else {
                    // 无路径：从已绑定的别名中查找
                    // 先获取已绑定的路径（避免借用冲突）
                    let existing_path = state
                        .visible_characters
                        .get(alias)
                        .map(|(path, _)| path.clone());

                    match existing_path {
                        Some(path) => {
                            // 找到已绑定的路径，更新位置
                            state
                                .visible_characters
                                .insert(alias.clone(), (path.clone(), *position));
                            path
                        }
                        None => {
                            // 别名未绑定，报错
                            return Err(RuntimeError::InvalidState {
                                message: format!(
                                    "别名 '{}' 尚未绑定立绘，无法改变位置。请先使用 'show <img src=\"...\"> as {} at ...' 绑定立绘。",
                                    alias, alias
                                ),
                            });
                        }
                    }
                };

                Ok(ExecuteResult::with_commands(vec![Command::ShowCharacter {
                    path: resolved_path,
                    alias: alias.clone(),
                    position: *position,
                    transition: transition.clone(),
                }]))
            }

            ScriptNode::HideCharacter { alias, transition } => {
                // 更新状态
                state.visible_characters.remove(alias);

                Ok(ExecuteResult::with_commands(vec![Command::HideCharacter {
                    alias: alias.clone(),
                    transition: transition.clone(),
                }]))
            }

            ScriptNode::Choice { style, options } => {
                let choices: Vec<Choice> = options
                    .iter()
                    .map(|opt| Choice {
                        text: opt.text.clone(),
                        target_label: opt.target_label.clone(),
                    })
                    .collect();

                let choice_count = choices.len();

                Ok(ExecuteResult::with_wait(
                    vec![Command::PresentChoices {
                        style: style.clone(),
                        choices,
                    }],
                    WaitingReason::WaitForChoice { choice_count },
                ))
            }

            ScriptNode::PlayAudio { path, is_bgm } => {
                // 解析路径（相对于脚本目录）
                let resolved_path = script.resolve_path(path);

                if *is_bgm {
                    // BGM: 循环播放，有 loop 标识
                    Ok(ExecuteResult::with_commands(vec![Command::PlayBgm {
                        path: resolved_path,
                        looping: true,
                    }]))
                } else {
                    // SFX: 播放一次
                    Ok(ExecuteResult::with_commands(vec![Command::PlaySfx {
                        path: resolved_path,
                    }]))
                }
            }

            ScriptNode::StopBgm => {
                // 停止 BGM，默认使用淡出效果（1秒）
                Ok(ExecuteResult::with_commands(vec![Command::StopBgm {
                    fade_out: Some(1.0),
                }]))
            }

            ScriptNode::BgmDuck => Ok(ExecuteResult::with_commands(vec![Command::BgmDuck])),

            ScriptNode::BgmUnduck => Ok(ExecuteResult::with_commands(vec![Command::BgmUnduck])),

            ScriptNode::Goto { target_label } => {
                // 查找标签位置
                let target_index =
                    script
                        .find_label(target_label)
                        .ok_or_else(|| RuntimeError::LabelNotFound {
                            label: target_label.clone(),
                        })?;

                Ok(ExecuteResult::with_jump(target_index))
            }

            ScriptNode::CallScript {
                path,
                display_label,
            } => Ok(ExecuteResult::with_script_control(
                ScriptControlFlow::Call {
                    target_path: path.clone(),
                    display_label: display_label.clone(),
                },
            )),

            ScriptNode::ReturnFromScript => Ok(ExecuteResult::with_script_control(
                ScriptControlFlow::Return,
            )),

            ScriptNode::SetVar { name, value } => {
                let val = evaluate(value, state).map_err(eval_error_to_runtime)?;
                if let Some(bare) = name.strip_prefix("persistent.") {
                    state.set_persistent_var(bare, val);
                } else {
                    state.set_var(name.clone(), val);
                }
                Ok(ExecuteResult::empty())
            }

            ScriptNode::FullRestart => Ok(ExecuteResult::with_commands(vec![Command::FullRestart])),

            ScriptNode::Conditional { branches } => {
                // 顺序求值每个分支条件，找到第一个为真的分支
                self.execute_conditional(branches, state, script)
            }

            ScriptNode::TextBoxHide => Ok(ExecuteResult::with_commands(vec![Command::TextBoxHide])),

            ScriptNode::TextBoxShow => Ok(ExecuteResult::with_commands(vec![Command::TextBoxShow])),

            ScriptNode::TextBoxClear => {
                Ok(ExecuteResult::with_commands(vec![Command::TextBoxClear]))
            }

            ScriptNode::ClearCharacters => {
                // 清除状态中的所有角色
                state.visible_characters.clear();
                Ok(ExecuteResult::with_commands(vec![Command::ClearCharacters]))
            }

            ScriptNode::Wait { duration } => Ok(ExecuteResult::with_wait(
                vec![],
                WaitingReason::WaitForTime(std::time::Duration::from_secs_f64(*duration)),
            )),

            ScriptNode::Pause => Ok(ExecuteResult::with_wait(
                vec![],
                WaitingReason::WaitForClick,
            )),

            ScriptNode::SceneEffect { effect } => {
                let has_duration = effect.get_named("duration").is_some();
                let cmd = Command::SceneEffect {
                    name: effect.name.clone(),
                    args: effect.args.clone(),
                };
                if has_duration {
                    Ok(ExecuteResult::with_wait(
                        vec![cmd],
                        WaitingReason::WaitForSignal(SignalId::new(SIGNAL_SCENE_EFFECT)),
                    ))
                } else {
                    Ok(ExecuteResult::with_commands(vec![cmd]))
                }
            }

            ScriptNode::TitleCard { text, duration } => Ok(ExecuteResult::with_wait(
                vec![Command::TitleCard {
                    text: text.clone(),
                    duration: *duration,
                }],
                WaitingReason::WaitForSignal(SignalId::new(SIGNAL_TITLE_CARD)),
            )),

            ScriptNode::Cutscene { path } => {
                let resolved = script.resolve_path(path);
                Ok(ExecuteResult::with_wait(
                    vec![Command::Cutscene { path: resolved }],
                    WaitingReason::WaitForSignal(SignalId::new(SIGNAL_CUTSCENE)),
                ))
            }
        }
    }

    /// 执行条件分支
    ///
    /// 返回需要执行的分支体产生的命令和等待状态
    fn execute_conditional(
        &mut self,
        branches: &[crate::script::ast::ConditionalBranch],
        state: &mut RuntimeState,
        script: &Script,
    ) -> Result<ExecuteResult, RuntimeError> {
        // 找到第一个条件为真的分支
        for branch in branches {
            let should_execute = match &branch.condition {
                Some(condition) => {
                    evaluate_to_bool(condition, state).map_err(eval_error_to_runtime)?
                }
                None => true, // else 分支，无条件执行
            };

            if should_execute {
                // 执行该分支体中的所有节点
                return self.execute_branch_body(&branch.body, state, script);
            }
        }

        // 没有分支被执行
        Ok(ExecuteResult::empty())
    }

    /// 执行分支体
    fn execute_branch_body(
        &mut self,
        body: &[ScriptNode],
        state: &mut RuntimeState,
        script: &Script,
    ) -> Result<ExecuteResult, RuntimeError> {
        let mut all_commands = Vec::new();

        for node in body {
            let result = self.execute(node, state, script)?;
            all_commands.extend(result.commands);

            // 如果遇到跳转，立即返回
            if result.jump_to.is_some() {
                return Ok(ExecuteResult {
                    commands: all_commands,
                    waiting: None,
                    jump_to: result.jump_to,
                    script_control: None,
                });
            }

            if result.script_control.is_some() {
                return Ok(ExecuteResult {
                    commands: all_commands,
                    waiting: None,
                    jump_to: None,
                    script_control: result.script_control,
                });
            }

            // 如果遇到等待，返回当前状态
            if result.waiting.is_some() {
                return Ok(ExecuteResult {
                    commands: all_commands,
                    waiting: result.waiting,
                    jump_to: None,
                    script_control: None,
                });
            }
        }

        Ok(ExecuteResult::with_commands(all_commands))
    }
}

/// 将 EvalError 转换为 RuntimeError
fn eval_error_to_runtime(e: EvalError) -> RuntimeError {
    RuntimeError::EvalError {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests;
