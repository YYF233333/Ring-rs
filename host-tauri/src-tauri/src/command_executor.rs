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
    WaitForCutscene {
        video_path: String,
    },
    FullRestart,
    RequestUI {
        key: String,
        mode: String,
    },
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

/// 场景效果请求（从 SceneEffect 命令解析出的参数）
#[derive(Debug, Clone)]
pub struct SceneEffectRequest {
    pub kind: SceneEffectKind,
    pub duration: f32,
}

/// 场景效果类别
#[derive(Debug, Clone)]
pub enum SceneEffectKind {
    Shake { amplitude_x: f32, amplitude_y: f32 },
    Blur,
    BlurOut,
    Dim { level: f32 },
    DimReset,
}

impl SceneEffectRequest {
    fn from_command(name: &str, args: &[(Option<String>, TransitionArg)]) -> Self {
        let name_lower = name.to_lowercase();
        let duration = Self::extract_duration(args);

        if name_lower.contains("blur") {
            if name_lower.contains("out") {
                SceneEffectRequest {
                    kind: SceneEffectKind::BlurOut,
                    duration: duration.unwrap_or(0.5),
                }
            } else {
                SceneEffectRequest {
                    kind: SceneEffectKind::Blur,
                    duration: duration.unwrap_or(0.5),
                }
            }
        } else if name_lower.contains("dim") {
            if name_lower.contains("reset") {
                SceneEffectRequest {
                    kind: SceneEffectKind::DimReset,
                    duration: 0.0,
                }
            } else {
                let level = Self::extract_level(args);
                let dim = (level / 7.0).clamp(0.0, 1.0);
                SceneEffectRequest {
                    kind: SceneEffectKind::Dim { level: dim },
                    duration: 0.0,
                }
            }
        } else {
            let (ax, ay) = if name_lower.contains("vertical") {
                (0.0, 8.0)
            } else if name_lower.contains("bounce") {
                (0.0, 5.0)
            } else {
                (6.0, 4.0)
            };
            SceneEffectRequest {
                kind: SceneEffectKind::Shake {
                    amplitude_x: ax,
                    amplitude_y: ay,
                },
                duration: duration.unwrap_or(0.3),
            }
        }
    }

    fn extract_duration(args: &[(Option<String>, TransitionArg)]) -> Option<f32> {
        for (key, val) in args {
            let is_duration = key.as_deref() == Some("duration")
                || (key.is_none() && matches!(val, TransitionArg::Number(_)));
            if is_duration && let TransitionArg::Number(n) = val {
                return Some(*n as f32);
            }
        }
        None
    }

    fn extract_level(args: &[(Option<String>, TransitionArg)]) -> f32 {
        for (key, val) in args {
            if key.as_deref() == Some("level") && let TransitionArg::Number(n) = val {
                return *n as f32;
            }
        }
        1.0
    }
}

/// 一次 execute 调用的输出
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    pub audio_command: Option<AudioCommand>,
    pub scene_effect_request: Option<SceneEffectRequest>,
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

/// 批量执行的输出
pub struct BatchOutput {
    pub result: ExecuteResult,
    pub audio_commands: Vec<AudioCommand>,
    pub scene_effect_request: Option<SceneEffectRequest>,
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
                        TransitionKind::None => {
                            rs.set_background(path.clone());
                        }
                        _ => {
                            rs.background_transition = Some(BackgroundTransition {
                                old_background: rs.current_background.clone(),
                                new_background: path.clone(),
                                duration,
                            });
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
                                new_background: path.clone(),
                                duration,
                            });
                            rs.set_background(path.clone());
                        }
                        TransitionKind::Fade => {
                            rs.scene_transition = Some(SceneTransition {
                                transition_type: SceneTransitionKind::Fade,
                                phase: SceneTransitionPhaseState::FadeIn,
                                duration,
                                pending_background: Some(path.clone()),
                            });
                        }
                        TransitionKind::FadeWhite => {
                            rs.scene_transition = Some(SceneTransition {
                                transition_type: SceneTransitionKind::FadeWhite,
                                phase: SceneTransitionPhaseState::FadeIn,
                                duration,
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
                let (kind, duration) = transition
                    .as_ref()
                    .map(resolve_transition)
                    .unwrap_or((TransitionKind::None, 0.0));

                if let Some(c) = rs.visible_characters.get_mut(alias) {
                    let is_position_change = c.position != *position;
                    let is_same_texture = c.texture_path == *path;

                    c.texture_path = path.clone();
                    c.position = *position;
                    c.target_alpha = 1.0;

                    if is_position_change && matches!(kind, TransitionKind::Move) {
                        c.transition_duration = Some(duration);
                    } else if is_same_texture && is_position_change {
                        c.transition_duration = None;
                        c.alpha = 1.0;
                    } else {
                        c.transition_duration = if matches!(kind, TransitionKind::None) {
                            c.alpha = 1.0;
                            None
                        } else {
                            Some(duration)
                        };
                    }
                } else {
                    let (start_alpha, trans_dur) = if matches!(kind, TransitionKind::None) {
                        (1.0, None)
                    } else {
                        (0.0, Some(duration))
                    };
                    rs.show_character(alias.clone(), path.clone(), *position);
                    if let Some(c) = rs.visible_characters.get_mut(alias) {
                        c.transition_duration = trans_dur;
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

            Command::SceneEffect { name, args } => {
                self.last_output.scene_effect_request =
                    Some(SceneEffectRequest::from_command(name, args));
                ExecuteResult::Ok
            }

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

            Command::FullRestart => ExecuteResult::FullRestart,

            Command::RequestUI { key, mode, .. } => ExecuteResult::RequestUI {
                key: key.clone(),
                mode: mode.clone(),
            },
        }
    }

    /// 批量执行命令，返回最后一个需要等待的结果及收集的副作用
    pub fn execute_batch(&mut self, cmds: &[Command], rs: &mut RenderState) -> BatchOutput {
        let mut final_result = ExecuteResult::Ok;
        let mut audio_commands = Vec::new();
        let mut scene_effect_request = None;
        for cmd in cmds {
            let result = self.execute(cmd, rs);
            if let Some(audio) = self.last_output.audio_command.take() {
                audio_commands.push(audio);
            }
            if let Some(effect) = self.last_output.scene_effect_request.take() {
                scene_effect_request = Some(effect);
            }
            if result != ExecuteResult::Ok {
                final_result = result;
            }
        }
        BatchOutput {
            result: final_result,
            audio_commands,
            scene_effect_request,
        }
    }
}
