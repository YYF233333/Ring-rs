use serde::Serialize;
use vn_runtime::command::{Command, TextMode, Transition, TransitionArg};

use crate::render_state::{
    BackgroundTransition, ChoiceItem, RenderState, SceneTransition, SceneTransitionKind,
    SceneTransitionPhaseState,
};

/// 单条命令的执行结果
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub enum ExecuteResult {
    #[default]
    Ok,
    WaitForClick,
    WaitForChoice {
        choice_count: usize,
    },
    WaitForTime(u64),
    WaitForCutscene {
        video_path: String,
    },
    Error(String),
}

/// 需要由 Host 音频层执行的音频指令
#[derive(Debug, Clone, Serialize)]
pub enum AudioCommand {
    PlayBgm {
        path: String,
        looping: bool,
        fade_in: Option<f32>,
    },
    StopBgm {
        fade_out: Option<f32>,
    },
    BgmDuck,
    BgmUnduck,
    PlaySfx {
        path: String,
    },
}

/// 一次 execute 调用的输出
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    pub result: ExecuteResult,
    pub audio_command: Option<AudioCommand>,
}

/// 过渡效果分类（内部使用）
enum TransitionKind {
    None,
    Dissolve,
    Fade,
    FadeWhite,
    Move,
    Rule { mask_path: String, reversed: bool },
}

/// 解析 Transition 为效果类型和时长
fn resolve_transition(transition: &Transition) -> (TransitionKind, f32) {
    let name = transition.name.to_lowercase();
    let duration = transition.get_duration().map(|d| d as f32);
    match name.as_str() {
        "dissolve" => (TransitionKind::Dissolve, duration.unwrap_or(0.3)),
        "fade" => (TransitionKind::Fade, duration.unwrap_or(0.5)),
        "fadewhite" => (TransitionKind::FadeWhite, duration.unwrap_or(0.5)),
        "move" | "slide" => (TransitionKind::Move, duration.unwrap_or(0.3)),
        "none" => (TransitionKind::None, 0.0),
        "rule" => {
            let mask_path = transition
                .get_arg("mask", 1)
                .and_then(|a| match a {
                    TransitionArg::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let reversed = transition.get_reversed().unwrap_or(false);
            (
                TransitionKind::Rule {
                    mask_path,
                    reversed,
                },
                duration.unwrap_or(0.5),
            )
        }
        _ => (TransitionKind::Dissolve, duration.unwrap_or(0.3)),
    }
}

/// 命令执行器（管理过渡状态）
#[derive(Debug)]
pub struct CommandExecutor {
    pub last_output: CommandOutput,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            last_output: CommandOutput::default(),
        }
    }

    /// 执行单条 Command，就地修改 RenderState
    pub fn execute(&mut self, cmd: &Command, rs: &mut RenderState) -> ExecuteResult {
        self.last_output = CommandOutput::default();

        match cmd {
            Command::ShowBackground {
                path, transition, ..
            } => {
                if let Some(t) = transition {
                    let (kind, duration) = resolve_transition(t);
                    match kind {
                        TransitionKind::Dissolve | TransitionKind::Move => {
                            rs.background_transition = Some(BackgroundTransition {
                                old_background: rs.current_background.clone(),
                                duration,
                                progress: 0.0,
                            });
                            rs.set_background(path.clone());
                        }
                        TransitionKind::None => {
                            rs.set_background(path.clone());
                        }
                        _ => {
                            rs.set_background(path.clone());
                        }
                    }
                } else {
                    rs.set_background(path.clone());
                }
                ExecuteResult::Ok
            }

            Command::ChangeScene {
                path, transition, ..
            } => {
                if let Some(t) = transition {
                    let (kind, duration) = resolve_transition(t);
                    match kind {
                        TransitionKind::Dissolve | TransitionKind::Move => {
                            rs.background_transition = Some(BackgroundTransition {
                                old_background: rs.current_background.clone(),
                                duration,
                                progress: 0.0,
                            });
                            rs.set_background(path.clone());
                        }
                        TransitionKind::Fade => {
                            rs.scene_transition = Some(SceneTransition {
                                transition_type: SceneTransitionKind::Fade,
                                phase: SceneTransitionPhaseState::FadeIn,
                                duration,
                                mask_alpha: 0.0,
                                progress: 0.0,
                                ui_alpha: 0.0,
                                pending_background: Some(path.clone()),
                            });
                        }
                        TransitionKind::FadeWhite => {
                            rs.scene_transition = Some(SceneTransition {
                                transition_type: SceneTransitionKind::FadeWhite,
                                phase: SceneTransitionPhaseState::FadeIn,
                                duration,
                                mask_alpha: 0.0,
                                progress: 0.0,
                                ui_alpha: 0.0,
                                pending_background: Some(path.clone()),
                            });
                        }
                        TransitionKind::Rule {
                            mask_path,
                            reversed,
                        } => {
                            rs.scene_transition = Some(SceneTransition {
                                transition_type: SceneTransitionKind::Rule {
                                    mask_path,
                                    reversed,
                                },
                                phase: SceneTransitionPhaseState::FadeIn,
                                duration,
                                mask_alpha: 0.0,
                                progress: 0.0,
                                ui_alpha: 0.0,
                                pending_background: Some(path.clone()),
                            });
                        }
                        TransitionKind::None => {
                            rs.set_background(path.clone());
                        }
                    }
                } else {
                    rs.set_background(path.clone());
                }
                ExecuteResult::Ok
            }

            Command::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => {
                let (trans_duration, start_alpha) = if let Some(t) = transition {
                    let (_, duration) = resolve_transition(t);
                    (Some(duration), 0.0)
                } else {
                    (None, 1.0)
                };

                if let Some(c) = rs.visible_characters.get_mut(alias) {
                    c.texture_path = path.clone();
                    c.position = *position;
                    c.transition_duration = trans_duration;
                    c.target_alpha = 1.0;
                    if trans_duration.is_none() {
                        c.alpha = 1.0;
                    }
                } else {
                    rs.show_character(alias.clone(), path.clone(), *position);
                    if let Some(c) = rs.visible_characters.get_mut(alias) {
                        c.transition_duration = trans_duration;
                        c.alpha = start_alpha;
                        c.target_alpha = 1.0;
                    }
                }
                ExecuteResult::Ok
            }

            Command::HideCharacter { alias, transition } => {
                if let Some(t) = transition {
                    let (_, duration) = resolve_transition(t);
                    if let Some(c) = rs.visible_characters.get_mut(alias) {
                        c.transition_duration = Some(duration);
                        c.target_alpha = 0.0;
                        c.fading_out = true;
                    }
                } else {
                    rs.hide_character(alias);
                }
                ExecuteResult::Ok
            }

            Command::ShowText {
                speaker,
                content,
                inline_effects,
                no_wait,
            } => {
                rs.start_typewriter(
                    speaker.clone(),
                    content.clone(),
                    inline_effects.clone(),
                    *no_wait,
                );
                if rs.text_mode == TextMode::NVL {
                    rs.nvl_entries.push(crate::render_state::NvlEntry {
                        speaker: speaker.clone(),
                        content: content.clone(),
                        visible_chars: 0,
                        is_complete: false,
                    });
                }
                ExecuteResult::WaitForClick
            }

            Command::ExtendText {
                content,
                inline_effects,
                no_wait,
            } => {
                rs.extend_dialogue(content.clone(), inline_effects.clone(), *no_wait);
                ExecuteResult::WaitForClick
            }

            Command::PresentChoices { choices, style } => {
                rs.clear_dialogue();
                let items = choices
                    .iter()
                    .map(|c| ChoiceItem {
                        text: c.text.clone(),
                        target_label: c.target_label.clone(),
                    })
                    .collect();
                rs.set_choices(items, style.clone());
                let count = choices.len();
                ExecuteResult::WaitForChoice {
                    choice_count: count,
                }
            }

            Command::ChapterMark { title, level } => {
                rs.set_chapter_mark(title.clone(), *level);
                ExecuteResult::Ok
            }

            Command::PlayBgm { path, looping } => {
                self.last_output.audio_command = Some(AudioCommand::PlayBgm {
                    path: path.clone(),
                    looping: *looping,
                    fade_in: None,
                });
                ExecuteResult::Ok
            }

            Command::StopBgm { fade_out } => {
                self.last_output.audio_command = Some(AudioCommand::StopBgm {
                    fade_out: fade_out.map(|f| f as f32),
                });
                ExecuteResult::Ok
            }

            Command::BgmDuck => {
                self.last_output.audio_command = Some(AudioCommand::BgmDuck);
                ExecuteResult::Ok
            }

            Command::BgmUnduck => {
                self.last_output.audio_command = Some(AudioCommand::BgmUnduck);
                ExecuteResult::Ok
            }

            Command::PlaySfx { path } => {
                self.last_output.audio_command = Some(AudioCommand::PlaySfx { path: path.clone() });
                ExecuteResult::Ok
            }

            Command::TextBoxHide => {
                rs.ui_visible = false;
                ExecuteResult::Ok
            }

            Command::TextBoxShow => {
                rs.ui_visible = true;
                ExecuteResult::Ok
            }

            Command::TextBoxClear => {
                rs.clear_dialogue();
                ExecuteResult::Ok
            }

            Command::ClearCharacters => {
                rs.hide_all_characters();
                ExecuteResult::Ok
            }

            Command::SceneEffect { .. } => ExecuteResult::Ok,

            Command::TitleCard { text, duration } => {
                rs.title_card = Some(crate::render_state::TitleCardState {
                    text: text.clone(),
                    duration: *duration as f32,
                    elapsed: 0.0,
                });
                ExecuteResult::Ok
            }

            Command::SetTextMode(mode) => {
                if *mode == TextMode::ADV {
                    rs.nvl_entries.clear();
                }
                rs.text_mode = *mode;
                ExecuteResult::Ok
            }

            Command::Cutscene { path } => ExecuteResult::WaitForCutscene {
                video_path: path.clone(),
            },

            Command::FullRestart | Command::RequestUI { .. } => ExecuteResult::Ok,
        }
    }

    /// 批量执行命令，返回最后一个需要等待的结果
    pub fn execute_batch(&mut self, cmds: &[Command], rs: &mut RenderState) -> ExecuteResult {
        let mut final_result = ExecuteResult::Ok;
        for cmd in cmds {
            let result = self.execute(cmd, rs);
            if result != ExecuteResult::Ok {
                final_result = result;
            }
        }
        final_result
    }
}
