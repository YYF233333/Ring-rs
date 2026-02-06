//! # Command Executor 模块
//!
//! Command 执行器，负责将 Runtime 发出的 Command 转换为实际操作。
//!
//! ## 设计说明
//!
//! - `CommandExecutor` 接收 `Command`，更新 `RenderState` 和控制音频
//! - 执行器不直接渲染，只更新状态，渲染由 `Renderer` 负责
//! - 角色动画通过 `CharacterAnimationCommand` 传递给主循环，由 AnimationSystem 处理
//! - 场景切换通过 `SceneTransitionCommand` 传递给主循环，由 SceneTransitionManager 处理
//!
//! ## 模块结构
//!
//! - `audio`: 音频命令执行
//! - `background`: 背景命令执行
//! - `character`: 角色命令执行
//! - `ui`: UI 命令执行
//! - `types`: 类型定义

mod audio;
mod background;
mod character;
mod types;
mod ui;

pub use types::*;

use crate::renderer::RenderState;
use crate::resources::ResourceManager;
use tracing::debug;
use vn_runtime::command::Command;

/// Command 执行器
///
/// 负责将 Runtime 发出的 Command 转换为实际的渲染状态更新。
#[derive(Debug)]
pub struct CommandExecutor {
    /// 当前是否有活跃的过渡效果
    transition_active: bool,
    /// 过渡效果计时器
    transition_timer: f32,
    /// 过渡效果总时长
    transition_duration: f32,
    /// 最近一次执行的输出
    pub last_output: CommandOutput,
}

impl CommandExecutor {
    /// 创建新的 Command 执行器
    pub fn new() -> Self {
        Self {
            transition_active: false,
            transition_timer: 0.0,
            transition_duration: 0.0,
            last_output: CommandOutput::default(),
        }
    }

    /// 执行单个 Command
    ///
    /// 根据 Command 类型更新 RenderState。
    /// 返回执行结果，同时更新 `last_output` 以获取过渡和音频信息。
    pub fn execute(
        &mut self,
        command: &Command,
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 重置输出
        self.last_output = CommandOutput::default();

        let result = match command {
            Command::ShowBackground { path, transition } => {
                self.execute_show_background(path, transition.clone(), render_state)
            }
            Command::ChangeScene { path, transition } => {
                // ChangeScene：遮罩过渡 + 切换背景（不再隐式清理 UI/立绘）
                self.execute_change_scene(path, transition.clone(), render_state, resource_manager)
            }
            Command::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => self.execute_show_character(path, alias, *position, transition, render_state),
            Command::HideCharacter { alias, transition } => {
                self.execute_hide_character(alias, transition, render_state)
            }
            Command::ShowText { speaker, content } => {
                self.execute_show_text(speaker.clone(), content, render_state)
            }
            Command::PresentChoices { style, choices } => {
                self.execute_present_choices(style.clone(), choices, render_state)
            }
            Command::ChapterMark { title, level } => {
                self.execute_chapter_mark(title, *level, render_state)
            }
            Command::PlayBgm { path, looping } => self.execute_play_bgm(path, *looping),
            Command::StopBgm { fade_out } => self.execute_stop_bgm(*fade_out),
            Command::PlaySfx { path } => self.execute_play_sfx(path),
            Command::TextBoxHide => self.execute_text_box_hide(render_state),
            Command::TextBoxShow => self.execute_text_box_show(render_state),
            Command::TextBoxClear => self.execute_text_box_clear(render_state),
            Command::ClearCharacters => self.execute_clear_characters(render_state),
        };

        self.last_output.result = result.clone();
        result
    }

    /// 批量执行 Commands
    ///
    /// 执行一组 Commands，返回最后一个需要等待的结果。
    pub fn execute_batch(
        &mut self,
        commands: &[Command],
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        let mut last_result = ExecuteResult::Ok;

        for command in commands {
            let result = self.execute(command, render_state, resource_manager);

            // 记录需要等待的结果
            match &result {
                ExecuteResult::WaitForClick
                | ExecuteResult::WaitForChoice { .. }
                | ExecuteResult::WaitForTime(_) => {
                    last_result = result;
                }
                ExecuteResult::Error(_) => {
                    return result; // 遇到错误立即返回
                }
                _ => {}
            }
        }

        last_result
    }

    /// 开始过渡效果
    ///
    /// 阶段 25 重构：接受已解析的 duration（不再从 Transition 提取）。
    pub(crate) fn start_transition(&mut self, duration: f32) {
        self.transition_active = true;
        self.transition_timer = 0.0;
        self.transition_duration = duration;

        debug!(duration = self.transition_duration, "开始过渡效果");
    }

    /// 更新过渡效果
    ///
    /// 返回 true 表示过渡效果仍在进行中。
    pub fn update_transition(&mut self, dt: f32) -> bool {
        if !self.transition_active {
            return false;
        }

        self.transition_timer += dt;
        if self.transition_timer >= self.transition_duration {
            self.transition_active = false;
            self.transition_timer = 0.0;
            debug!("过渡效果完成");
            return false;
        }

        true
    }

    /// 获取过渡效果进度 (0.0 - 1.0)
    pub fn get_transition_progress(&self) -> f32 {
        if !self.transition_active || self.transition_duration <= 0.0 {
            return 1.0;
        }
        (self.transition_timer / self.transition_duration).min(1.0)
    }

    /// 检查是否有活跃的过渡效果
    pub fn is_transition_active(&self) -> bool {
        self.transition_active
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_runtime::command::{Choice, Position, Transition};

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        assert!(!executor.is_transition_active());
    }

    #[test]
    fn test_execute_show_text() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowText {
            speaker: Some("北风".to_string()),
            content: "你好".to_string(),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::WaitForClick);
        assert!(render_state.dialogue.is_some());

        let dialogue = render_state.dialogue.as_ref().unwrap();
        assert_eq!(dialogue.speaker, Some("北风".to_string()));
        assert_eq!(dialogue.content, "你好");
    }

    #[test]
    fn test_execute_show_text_narrator() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowText {
            speaker: None,
            content: "旁白内容".to_string(),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::WaitForClick);

        let dialogue = render_state.dialogue.as_ref().unwrap();
        assert_eq!(dialogue.speaker, None);
    }

    #[test]
    fn test_execute_present_choices() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::PresentChoices {
            style: None,
            choices: vec![
                Choice {
                    text: "选项1".to_string(),
                    target_label: "label1".to_string(),
                },
                Choice {
                    text: "选项2".to_string(),
                    target_label: "label2".to_string(),
                },
            ],
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::WaitForChoice { choice_count: 2 });
        assert!(render_state.choices.is_some());

        let choices = render_state.choices.as_ref().unwrap();
        assert_eq!(choices.choices.len(), 2);
        assert_eq!(choices.choices[0].text, "选项1");
        assert_eq!(choices.choices[1].target_label, "label2");
    }

    #[test]
    fn test_execute_show_background() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowBackground {
            path: "backgrounds/bg1.png".to_string(),
            transition: None,
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert_eq!(
            render_state.current_background,
            Some("backgrounds/bg1.png".to_string())
        );
    }

    #[test]
    fn test_execute_show_background_with_transition() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 先设置旧背景
        render_state.set_background("old_bg.png".to_string());

        let transition = Transition::simple("dissolve");
        let cmd = Command::ShowBackground {
            path: "new_bg.png".to_string(),
            transition: Some(transition),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(
            executor
                .last_output
                .transition_info
                .has_background_transition
        );
        assert_eq!(
            executor.last_output.transition_info.old_background,
            Some("old_bg.png".to_string())
        );
    }

    #[test]
    fn test_execute_show_character() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: None,
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(render_state.visible_characters.contains_key("char1"));

        let char_sprite = render_state.visible_characters.get("char1").unwrap();
        assert_eq!(char_sprite.texture_path, "characters/char1.png");
        assert_eq!(char_sprite.position, Position::Center);
    }

    #[test]
    fn test_execute_show_character_reposition_with_dissolve_is_teleport() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 先显示角色（无过渡）
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: None,
        };
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.character_animation.is_none());

        // 位置变更：with dissolve 只应“瞬移”（不触发 Move 动画）
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Left,
            transition: Some(Transition::simple("dissolve")),
        };
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.character_animation.is_none());

        let char_sprite = render_state.visible_characters.get("char1").unwrap();
        assert_eq!(char_sprite.position, Position::Left);
    }

    #[test]
    fn test_execute_show_character_reposition_with_move_triggers_animation() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 先显示角色（无过渡）
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: None,
        };
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // 位置变更：with move 应触发 Move 动画
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Right,
            transition: Some(Transition::simple("move")),
        };
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        match executor.last_output.character_animation.as_ref() {
            Some(CharacterAnimationCommand::Move {
                alias,
                old_position,
                new_position,
                duration,
            }) => {
                assert_eq!(alias, "char1");
                assert_eq!(*old_position, Position::Center);
                assert_eq!(*new_position, Position::Right);
                assert!(*duration > 0.0);
            }
            other => panic!("Expected Move animation, got {:?}", other),
        }

        let char_sprite = render_state.visible_characters.get("char1").unwrap();
        assert_eq!(char_sprite.position, Position::Right);
    }

    #[test]
    fn test_execute_hide_character() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 先显示角色
        render_state.show_character(
            "char1".to_string(),
            "characters/char1.png".to_string(),
            Position::Center,
        );

        let cmd = Command::HideCharacter {
            alias: "char1".to_string(),
            transition: None,
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(!render_state.visible_characters.contains_key("char1"));
    }

    #[test]
    fn test_execute_chapter_mark() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ChapterMark {
            title: "第一章".to_string(),
            level: 1,
        };

        // 阶段 24：ChapterMark 是非阻塞的
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(render_state.chapter_mark.is_some());

        let chapter = render_state.chapter_mark.as_ref().unwrap();
        assert_eq!(chapter.title, "第一章");
        assert_eq!(chapter.level, 1);
        assert_eq!(chapter.alpha, 0.0); // 从 FadeIn 开始
    }

    #[test]
    fn test_execute_play_bgm() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::PlayBgm {
            path: "bgm/music.mp3".to_string(),
            looping: true,
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.audio_command.is_some());

        if let Some(AudioCommand::PlayBgm { path, looping, .. }) =
            &executor.last_output.audio_command
        {
            assert_eq!(path, "bgm/music.mp3");
            assert!(*looping);
        } else {
            panic!("Expected PlayBgm command");
        }
    }

    #[test]
    fn test_execute_stop_bgm() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::StopBgm {
            fade_out: Some(1.0),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.audio_command.is_some());

        if let Some(AudioCommand::StopBgm { fade_out }) = &executor.last_output.audio_command {
            assert_eq!(*fade_out, Some(1.0));
        } else {
            panic!("Expected StopBgm command");
        }
    }

    #[test]
    fn test_execute_play_sfx() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::PlaySfx {
            path: "sfx/click.wav".to_string(),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.audio_command.is_some());

        if let Some(AudioCommand::PlaySfx { path }) = &executor.last_output.audio_command {
            assert_eq!(path, "sfx/click.wav");
        } else {
            panic!("Expected PlaySfx command");
        }
    }

    #[test]
    fn test_transition_progress() {
        let mut executor = CommandExecutor::new();

        // 未激活时进度为 1.0
        assert_eq!(executor.get_transition_progress(), 1.0);

        // 开始过渡（阶段 25：直接传 duration）
        executor.start_transition(0.3);
        assert!(executor.is_transition_active());

        // 更新一半
        executor.update_transition(0.15);
        let progress = executor.get_transition_progress();
        assert!(progress > 0.0 && progress < 1.0);

        // 完成过渡
        executor.update_transition(0.2);
        assert!(!executor.is_transition_active());
        assert_eq!(executor.get_transition_progress(), 1.0);
    }

    #[test]
    fn test_execute_batch() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let commands = vec![
            Command::ShowBackground {
                path: "bg.png".to_string(),
                transition: None,
            },
            Command::ShowText {
                speaker: Some("角色".to_string()),
                content: "对话".to_string(),
            },
        ];

        let result = executor.execute_batch(&commands, &mut render_state, &resource_manager);
        // 最后一个需要等待的结果
        assert_eq!(result, ExecuteResult::WaitForClick);
        assert!(render_state.dialogue.is_some());
        assert_eq!(render_state.current_background, Some("bg.png".to_string()));
    }

    // ========== 阶段 25：效果矩阵测试 ==========
    // 验证同名效果在不同 target 上的解析一致性

    #[test]
    fn test_dissolve_consistency_background_vs_character() {
        // 同一个 `dissolve(0.5)`：背景和立绘的解析结果应一致
        use crate::renderer::effects;

        let transition = Transition::with_args(
            "dissolve",
            vec![vn_runtime::command::TransitionArg::Number(0.5)],
        );
        let effect = effects::resolve(&transition);

        // 解析结果唯一
        assert_eq!(effect.kind, effects::EffectKind::Dissolve);
        assert_eq!(effect.duration, Some(0.5));

        // 立绘上下文：duration_or(CHARACTER_ALPHA_DURATION) = 0.5（显式值优先）
        assert_eq!(
            effect.duration_or(effects::defaults::CHARACTER_ALPHA_DURATION),
            0.5
        );
        // 背景上下文：duration_or(BACKGROUND_DISSOLVE_DURATION) = 0.5（显式值优先）
        assert_eq!(
            effect.duration_or(effects::defaults::BACKGROUND_DISSOLVE_DURATION),
            0.5
        );
    }

    #[test]
    fn test_dissolve_default_duration_background_vs_character() {
        // `dissolve`（无参数）的默认值在不同上下文中一致
        use crate::renderer::effects;

        let transition = Transition::simple("dissolve");
        let effect = effects::resolve(&transition);

        assert_eq!(effect.duration, None);
        // 立绘和背景的默认 dissolve 时长都是 0.3
        assert_eq!(
            effect.duration_or(effects::defaults::CHARACTER_ALPHA_DURATION),
            effects::defaults::CHARACTER_ALPHA_DURATION
        );
        assert_eq!(
            effect.duration_or(effects::defaults::BACKGROUND_DISSOLVE_DURATION),
            effects::defaults::BACKGROUND_DISSOLVE_DURATION
        );
        // 两者应相等
        assert_eq!(
            effects::defaults::CHARACTER_ALPHA_DURATION,
            effects::defaults::BACKGROUND_DISSOLVE_DURATION
        );
    }

    #[test]
    fn test_show_character_with_dissolve_produces_alpha_animation() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: Some(Transition::simple("dissolve")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // dissolve 应产生 Show 动画（alpha 淡入）
        match executor.last_output.character_animation.as_ref() {
            Some(CharacterAnimationCommand::Show { alias, duration }) => {
                assert_eq!(alias, "char1");
                assert!(*duration > 0.0);
            }
            other => panic!("Expected Show animation, got {:?}", other),
        }
    }

    #[test]
    fn test_hide_character_with_dissolve_produces_alpha_animation() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 先显示角色
        render_state.show_character(
            "char1".to_string(),
            "characters/char1.png".to_string(),
            Position::Center,
        );

        let cmd = Command::HideCharacter {
            alias: "char1".to_string(),
            transition: Some(Transition::simple("dissolve")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // dissolve 应产生 Hide 动画（alpha 淡出）
        match executor.last_output.character_animation.as_ref() {
            Some(CharacterAnimationCommand::Hide { alias, duration }) => {
                assert_eq!(alias, "char1");
                assert!(*duration > 0.0);
            }
            other => panic!("Expected Hide animation, got {:?}", other),
        }
    }

    #[test]
    fn test_show_background_with_dissolve_produces_transition_info() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        render_state.set_background("old_bg.png".to_string());

        let cmd = Command::ShowBackground {
            path: "new_bg.png".to_string(),
            transition: Some(Transition::simple("dissolve")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        let ti = &executor.last_output.transition_info;
        assert!(ti.has_background_transition);
        assert_eq!(ti.old_background, Some("old_bg.png".to_string()));

        // 应携带 ResolvedEffect
        let effect = ti.effect.as_ref().expect("Should have ResolvedEffect");
        use crate::renderer::effects::EffectKind;
        assert_eq!(effect.kind, EffectKind::Dissolve);
    }

    #[test]
    fn test_change_scene_fade_produces_scene_transition() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ChangeScene {
            path: "new_bg.png".to_string(),
            transition: Some(Transition::simple("fade")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // fade 在 changeScene 上下文应产生 Fade 场景过渡
        match executor.last_output.scene_transition.as_ref() {
            Some(SceneTransitionCommand::Fade {
                duration,
                pending_background,
            }) => {
                assert_eq!(pending_background, "new_bg.png");
                assert!(*duration > 0.0);
            }
            other => panic!("Expected SceneTransitionCommand::Fade, got {:?}", other),
        }
    }

    #[test]
    fn test_change_scene_dissolve_produces_transition_info() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        render_state.set_background("old_bg.png".to_string());

        let cmd = Command::ChangeScene {
            path: "new_bg.png".to_string(),
            transition: Some(Transition::simple("dissolve")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // dissolve 在 changeScene 上下文应产生 transition_info（交叉淡化）
        assert!(executor.last_output.scene_transition.is_none());
        assert!(
            executor
                .last_output
                .transition_info
                .has_background_transition
        );

        let effect = executor
            .last_output
            .transition_info
            .effect
            .as_ref()
            .expect("Should have ResolvedEffect");
        use crate::renderer::effects::EffectKind;
        assert_eq!(effect.kind, EffectKind::Dissolve);
    }

    #[test]
    fn test_change_scene_rule_produces_scene_transition() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ChangeScene {
            path: "new_bg.png".to_string(),
            transition: Some(Transition::with_named_args(
                "rule",
                vec![
                    (
                        Some("duration".to_string()),
                        vn_runtime::command::TransitionArg::Number(0.8),
                    ),
                    (
                        Some("mask".to_string()),
                        vn_runtime::command::TransitionArg::String("masks/wipe.png".to_string()),
                    ),
                    (
                        Some("reversed".to_string()),
                        vn_runtime::command::TransitionArg::Bool(true),
                    ),
                ],
            )),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        match executor.last_output.scene_transition.as_ref() {
            Some(SceneTransitionCommand::Rule {
                duration,
                pending_background,
                mask_path,
                reversed,
            }) => {
                assert_eq!(pending_background, "new_bg.png");
                assert!((duration - 0.8).abs() < 0.01);
                assert!(mask_path.contains("wipe.png") || mask_path.contains("masks"));
                assert!(*reversed);
            }
            other => panic!("Expected SceneTransitionCommand::Rule, got {:?}", other),
        }
    }

    #[test]
    fn test_fade_on_character_is_alpha_not_scene_mask() {
        // fade 在立绘上下文中等价于 dissolve（alpha 淡入），不是黑屏遮罩
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: Some(Transition::simple("fade")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // 应产生 Show 动画（alpha 淡入），而非场景过渡
        match executor.last_output.character_animation.as_ref() {
            Some(CharacterAnimationCommand::Show { alias, duration }) => {
                assert_eq!(alias, "char1");
                assert!(*duration > 0.0);
            }
            other => panic!(
                "Expected Show animation for fade on character, got {:?}",
                other
            ),
        }
        assert!(executor.last_output.scene_transition.is_none());
    }

    #[test]
    fn test_explicit_duration_overrides_default_for_all_targets() {
        // 显式 duration 应在所有 target 上生效
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        // 立绘：dissolve(2.0)
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Center,
            transition: Some(Transition::with_args(
                "dissolve",
                vec![vn_runtime::command::TransitionArg::Number(2.0)],
            )),
        };
        executor.execute(&cmd, &mut render_state, &resource_manager);

        if let Some(CharacterAnimationCommand::Show { duration, .. }) =
            &executor.last_output.character_animation
        {
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Character dissolve duration should be 2.0, got {}",
                duration
            );
        } else {
            panic!("Expected Show animation");
        }

        // 背景：dissolve(2.0)
        let cmd = Command::ShowBackground {
            path: "bg.png".to_string(),
            transition: Some(Transition::with_args(
                "dissolve",
                vec![vn_runtime::command::TransitionArg::Number(2.0)],
            )),
        };
        executor.execute(&cmd, &mut render_state, &resource_manager);

        let effect = executor
            .last_output
            .transition_info
            .effect
            .as_ref()
            .unwrap();
        assert_eq!(effect.duration, Some(2.0));

        // changeScene fade(2.0)
        let cmd = Command::ChangeScene {
            path: "bg2.png".to_string(),
            transition: Some(Transition::with_args(
                "fade",
                vec![vn_runtime::command::TransitionArg::Number(2.0)],
            )),
        };
        executor.execute(&cmd, &mut render_state, &resource_manager);

        if let Some(SceneTransitionCommand::Fade { duration, .. }) =
            &executor.last_output.scene_transition
        {
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Scene fade duration should be 2.0, got {}",
                duration
            );
        } else {
            panic!("Expected SceneTransitionCommand::Fade");
        }
    }
}
