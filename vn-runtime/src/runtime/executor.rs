//! # Executor 模块
//!
//! 将 AST 节点转换为 Command。
//!
//! ## 职责
//!
//! - 读取 ScriptNode
//! - 产生对应的 Command
//! - 决定是否需要等待

use crate::command::{Choice, Command};
use crate::error::RuntimeError;
use crate::script::{Script, ScriptNode};
use crate::state::{RuntimeState, WaitingReason};

/// 执行结果
pub struct ExecuteResult {
    /// 产生的命令
    pub commands: Vec<Command>,
    /// 等待原因（如果需要等待）
    pub waiting: Option<WaitingReason>,
    /// 跳转目标（如果需要跳转）
    pub jump_to: Option<usize>,
}

impl ExecuteResult {
    /// 创建空结果
    fn empty() -> Self {
        Self {
            commands: Vec::new(),
            waiting: None,
            jump_to: None,
        }
    }

    /// 创建带命令的结果
    fn with_commands(commands: Vec<Command>) -> Self {
        Self {
            commands,
            waiting: None,
            jump_to: None,
        }
    }

    /// 创建带等待的结果
    fn with_wait(commands: Vec<Command>, waiting: WaitingReason) -> Self {
        Self {
            commands,
            waiting: Some(waiting),
            jump_to: None,
        }
    }

    /// 创建跳转结果
    fn with_jump(jump_to: usize) -> Self {
        Self {
            commands: Vec::new(),
            waiting: None,
            jump_to: Some(jump_to),
        }
    }
}

/// 节点执行器
///
/// 负责将单个 ScriptNode 转换为 Command。
pub struct Executor {
    // 未来可添加执行上下文
}

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

            ScriptNode::Dialogue { speaker, content } => {
                Ok(ExecuteResult::with_wait(
                    vec![Command::ShowText {
                        speaker: speaker.clone(),
                        content: content.clone(),
                    }],
                    WaitingReason::WaitForClick,
                ))
            }

            ScriptNode::ChangeBG { path, transition } => {
                // 解析路径（相对于脚本目录）
                let resolved_path = script.resolve_path(path);
                // 更新状态
                state.current_background = Some(resolved_path.clone());

                Ok(ExecuteResult::with_commands(vec![Command::ShowBackground {
                    path: resolved_path,
                    transition: transition.clone(),
                }]))
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
                    new_transition.args = t.args.iter().map(|(key, arg)| {
                        if let Some(k) = key {
                            // 命名参数：mask 参数需要解析路径
                            if k == "mask" {
                                if let crate::command::TransitionArg::String(s) = arg {
                                    return (Some(k.clone()), crate::command::TransitionArg::String(script.resolve_path(s)));
                                }
                            }
                        }
                        (key.clone(), arg.clone())
                    }).collect();
                    new_transition
                });

                Ok(ExecuteResult::with_commands(vec![Command::ChangeScene {
                    path: resolved_path,
                    transition: resolved_transition,
                }]))
            }

            ScriptNode::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => {
                // 解析路径（相对于脚本目录）
                let resolved_path = script.resolve_path(path);
                // 更新状态
                state
                    .visible_characters
                    .insert(alias.clone(), (resolved_path.clone(), *position));

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

            ScriptNode::UIAnim { effect } => {
                Ok(ExecuteResult::with_commands(vec![Command::UIAnimation {
                    effect: effect.clone(),
                }]))
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

            ScriptNode::Goto { target_label } => {
                // 查找标签位置
                let target_index = script.find_label(target_label).ok_or_else(|| {
                    RuntimeError::LabelNotFound {
                        label: target_label.clone(),
                    }
                })?;

                Ok(ExecuteResult::with_jump(target_index))
            }
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Position;
    use crate::script::ChoiceOption;

    #[test]
    fn test_execute_dialogue() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "");

        let node = ScriptNode::Dialogue {
            speaker: Some("Test".to_string()),
            content: "Hello".to_string(),
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(matches!(
            &result.commands[0],
            Command::ShowText { speaker: Some(s), content }
            if s == "Test" && content == "Hello"
        ));
        assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
    }

    #[test]
    fn test_execute_show_character() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "");

        let node = ScriptNode::ShowCharacter {
            path: "char.png".to_string(),
            alias: "test_char".to_string(),
            position: Position::Center,
            transition: None,
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(result.waiting.is_none());

        // 验证状态更新
        assert!(state.visible_characters.contains_key("test_char"));
    }

    #[test]
    fn test_execute_choice() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "");

        let node = ScriptNode::Choice {
            style: Some("横排".to_string()),
            options: vec![
                ChoiceOption {
                    text: "选项A".to_string(),
                    target_label: "label_a".to_string(),
                },
                ChoiceOption {
                    text: "选项B".to_string(),
                    target_label: "label_b".to_string(),
                },
            ],
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(matches!(
            &result.commands[0],
            Command::PresentChoices { choices, .. } if choices.len() == 2
        ));
        assert!(matches!(
            result.waiting,
            Some(WaitingReason::WaitForChoice { choice_count: 2 })
        ));
    }

    #[test]
    fn test_execute_label_no_command() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "");

        let node = ScriptNode::Label {
            name: "test".to_string(),
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert!(result.commands.is_empty());
        assert!(result.waiting.is_none());
    }

    #[test]
    fn test_execute_goto() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new(
            "test",
            vec![
                ScriptNode::Label { name: "start".to_string() },
                ScriptNode::Dialogue { speaker: None, content: "Hello".to_string() },
                ScriptNode::Label { name: "end".to_string() },
            ],
            "",
        );

        let node = ScriptNode::Goto {
            target_label: "end".to_string(),
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert!(result.commands.is_empty());
        assert!(result.waiting.is_none());
        assert_eq!(result.jump_to, Some(2)); // 跳转到 "end" 标签
    }

    #[test]
    fn test_execute_play_bgm() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "scripts");

        // BGM: 有 loop 标识
        let node = ScriptNode::PlayAudio {
            path: "../bgm/music.mp3".to_string(),
            is_bgm: true,
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(matches!(
            &result.commands[0],
            Command::PlayBgm { path, looping: true }
            if path == "scripts/../bgm/music.mp3"
        ));
    }

    #[test]
    fn test_execute_play_sfx() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "scripts");

        // SFX: 无 loop 标识
        let node = ScriptNode::PlayAudio {
            path: "../sfx/click.mp3".to_string(),
            is_bgm: false,
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(matches!(
            &result.commands[0],
            Command::PlaySfx { path }
            if path == "scripts/../sfx/click.mp3"
        ));
    }

    #[test]
    fn test_execute_stop_bgm() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "");

        let node = ScriptNode::StopBgm;

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(result.commands.len(), 1);
        assert!(matches!(
            &result.commands[0],
            Command::StopBgm { fade_out: Some(_) }
        ));
    }

    #[test]
    fn test_path_resolution() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![], "assets/scripts");

        let node = ScriptNode::ChangeBG {
            path: "../backgrounds/bg.jpg".to_string(),
            transition: None,
        };

        let result = executor.execute(&node, &mut state, &script).unwrap();

        assert!(matches!(
            &result.commands[0],
            Command::ShowBackground { path, .. }
            if path == "assets/scripts/../backgrounds/bg.jpg"
        ));
    }
}

