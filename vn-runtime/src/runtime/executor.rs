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
    /// - `Vec<Command>`: 产生的指令
    /// - `Option<WaitingReason>`: 如果需要等待，返回等待原因
    pub fn execute(
        &mut self,
        node: &ScriptNode,
        state: &mut RuntimeState,
        _script: &Script,
    ) -> Result<(Vec<Command>, Option<WaitingReason>), RuntimeError> {
        let mut commands = Vec::new();
        let mut waiting = None;

        match node {
            ScriptNode::Chapter { title, level } => {
                commands.push(Command::ChapterMark {
                    title: title.clone(),
                    level: *level,
                });
            }

            ScriptNode::Label { .. } => {
                // 标签节点不产生 Command，只是跳转目标
            }

            ScriptNode::Dialogue { speaker, content } => {
                commands.push(Command::ShowText {
                    speaker: speaker.clone(),
                    content: content.clone(),
                });
                waiting = Some(WaitingReason::WaitForClick);
            }

            ScriptNode::ChangeBG { path, transition } => {
                // 更新状态
                state.current_background = Some(path.clone());

                commands.push(Command::ShowBackground {
                    path: path.clone(),
                    transition: transition.clone(),
                });
            }

            ScriptNode::ChangeScene { path, transition } => {
                // 更新状态
                state.current_background = Some(path.clone());

                commands.push(Command::ChangeScene {
                    path: path.clone(),
                    transition: transition.clone(),
                });
            }

            ScriptNode::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => {
                // 更新状态
                state
                    .visible_characters
                    .insert(alias.clone(), (path.clone(), *position));

                commands.push(Command::ShowCharacter {
                    path: path.clone(),
                    alias: alias.clone(),
                    position: *position,
                    transition: transition.clone(),
                });
            }

            ScriptNode::HideCharacter { alias, transition } => {
                // 更新状态
                state.visible_characters.remove(alias);

                commands.push(Command::HideCharacter {
                    alias: alias.clone(),
                    transition: transition.clone(),
                });
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

                commands.push(Command::PresentChoices {
                    style: style.clone(),
                    choices,
                });

                waiting = Some(WaitingReason::WaitForChoice { choice_count });
            }

            ScriptNode::UIAnim { effect } => {
                commands.push(Command::UIAnimation {
                    effect: effect.clone(),
                });
            }
        }

        Ok((commands, waiting))
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
        let script = Script::new("test", vec![]);

        let node = ScriptNode::Dialogue {
            speaker: Some("Test".to_string()),
            content: "Hello".to_string(),
        };

        let (commands, waiting) = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            Command::ShowText { speaker: Some(s), content }
            if s == "Test" && content == "Hello"
        ));
        assert!(matches!(waiting, Some(WaitingReason::WaitForClick)));
    }

    #[test]
    fn test_execute_show_character() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![]);

        let node = ScriptNode::ShowCharacter {
            path: "char.png".to_string(),
            alias: "test_char".to_string(),
            position: Position::Center,
            transition: None,
        };

        let (commands, waiting) = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(waiting.is_none());

        // 验证状态更新
        assert!(state.visible_characters.contains_key("test_char"));
    }

    #[test]
    fn test_execute_choice() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![]);

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

        let (commands, waiting) = executor.execute(&node, &mut state, &script).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            Command::PresentChoices { choices, .. } if choices.len() == 2
        ));
        assert!(matches!(
            waiting,
            Some(WaitingReason::WaitForChoice { choice_count: 2 })
        ));
    }

    #[test]
    fn test_execute_label_no_command() {
        let mut executor = Executor::new();
        let mut state = RuntimeState::new("test");
        let script = Script::new("test", vec![]);

        let node = ScriptNode::Label {
            name: "test".to_string(),
        };

        let (commands, waiting) = executor.execute(&node, &mut state, &script).unwrap();

        assert!(commands.is_empty());
        assert!(waiting.is_none());
    }
}

