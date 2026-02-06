//! # Command Executor 模块
//!
//! Command 执行器，负责将 Runtime 发出的 Command 转换为实际操作。
//!
//! ## 设计说明
//!
//! - `CommandExecutor` 接收 `Command`，更新 `RenderState` 和控制音频
//! - 执行器不直接渲染，只更新状态，渲染由 `Renderer` 负责
//! - 动画/过渡效果通过 `EffectRequest` 统一传递给 `EffectApplier`
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
use vn_runtime::command::Command;

/// Command 执行器
///
/// 负责将 Runtime 发出的 Command 转换为实际的渲染状态更新。
/// 动画/过渡效果通过 `last_output.effect_requests` 传递给 `EffectApplier`。
#[derive(Debug)]
pub struct CommandExecutor {
    /// 最近一次执行的输出
    pub last_output: CommandOutput,
}

impl CommandExecutor {
    /// 创建新的 Command 执行器
    pub fn new() -> Self {
        Self {
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
        assert!(executor.last_output.effect_requests.is_empty());
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
        use crate::renderer::effects::{EffectKind, EffectTarget};

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
        assert_eq!(executor.last_output.effect_requests.len(), 1);

        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            EffectTarget::BackgroundTransition { old_background } => {
                assert_eq!(*old_background, Some("old_bg.png".to_string()));
            }
            other => panic!("Expected BackgroundTransition, got {:?}", other),
        }
        assert_eq!(req.effect.kind, EffectKind::Dissolve);
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
        assert!(executor.last_output.effect_requests.is_empty());

        // 位置变更：with dissolve 只应“瞬移”（不触发 Move 动画）
        let cmd = Command::ShowCharacter {
            path: "characters/char1.png".to_string(),
            alias: "char1".to_string(),
            position: Position::Left,
            transition: Some(Transition::simple("dissolve")),
        };
        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);
        assert!(executor.last_output.effect_requests.is_empty());

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

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            crate::renderer::effects::EffectTarget::CharacterMove {
                alias,
                old_position,
                new_position,
            } => {
                assert_eq!(alias, "char1");
                assert_eq!(*old_position, Position::Center);
                assert_eq!(*new_position, Position::Right);
            }
            other => panic!("Expected CharacterMove, got {:?}", other),
        }
        assert!(req.effect.duration_or(0.0) > 0.0);

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

    // test_transition_progress 已移除：transition timer 已从 CommandExecutor 删除

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
        use crate::renderer::effects::EffectTarget;

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

        // dissolve 应产生 CharacterShow 效果请求（alpha 淡入）
        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            EffectTarget::CharacterShow { alias } => {
                assert_eq!(alias, "char1");
            }
            other => panic!("Expected CharacterShow, got {:?}", other),
        }
        assert!(req.effect.duration_or(0.0) > 0.0);
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

        // dissolve 应产生 CharacterHide 效果请求（alpha 淡出）
        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            crate::renderer::effects::EffectTarget::CharacterHide { alias } => {
                assert_eq!(alias, "char1");
            }
            other => panic!("Expected CharacterHide, got {:?}", other),
        }
        assert!(req.effect.duration_or(0.0) > 0.0);
    }

    #[test]
    fn test_show_background_with_dissolve_produces_effect_request() {
        use crate::renderer::effects::{EffectKind, EffectTarget};

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

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            EffectTarget::BackgroundTransition { old_background } => {
                assert_eq!(*old_background, Some("old_bg.png".to_string()));
            }
            other => panic!("Expected BackgroundTransition, got {:?}", other),
        }
        assert_eq!(req.effect.kind, EffectKind::Dissolve);
    }

    #[test]
    fn test_change_scene_fade_produces_scene_transition() {
        use crate::renderer::effects::{EffectKind, EffectTarget};

        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ChangeScene {
            path: "new_bg.png".to_string(),
            transition: Some(Transition::simple("fade")),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::Ok);

        // fade 在 changeScene 上下文应产生 SceneTransition 效果请求
        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            EffectTarget::SceneTransition { pending_background } => {
                assert_eq!(pending_background, "new_bg.png");
            }
            other => panic!("Expected SceneTransition, got {:?}", other),
        }
        assert_eq!(req.effect.kind, EffectKind::Fade);
        // duration 未显式指定时为 None；EffectApplier 会使用 defaults::FADE_DURATION
        assert!(
            req.effect
                .duration_or(crate::renderer::effects::defaults::FADE_DURATION)
                > 0.0
        );
    }

    #[test]
    fn test_change_scene_dissolve_produces_background_transition() {
        use crate::renderer::effects::{EffectKind, EffectTarget};

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

        // dissolve 在 changeScene 上下文应产生 BackgroundTransition 效果请求
        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            EffectTarget::BackgroundTransition { old_background } => {
                assert_eq!(*old_background, Some("old_bg.png".to_string()));
            }
            other => panic!("Expected BackgroundTransition, got {:?}", other),
        }
        assert_eq!(req.effect.kind, EffectKind::Dissolve);
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

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            crate::renderer::effects::EffectTarget::SceneTransition { pending_background } => {
                assert_eq!(pending_background, "new_bg.png");
            }
            other => panic!("Expected SceneTransition, got {:?}", other),
        }
        match &req.effect.kind {
            crate::renderer::effects::EffectKind::Rule {
                mask_path,
                reversed,
            } => {
                assert!(mask_path.contains("wipe.png") || mask_path.contains("masks"));
                assert!(*reversed);
            }
            other => panic!("Expected Rule effect, got {:?}", other),
        }
        assert!((req.effect.duration_or(0.0) - 0.8).abs() < 0.01);
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

        // 应产生 CharacterShow 效果请求（alpha 淡入），而非场景过渡
        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let req = &executor.last_output.effect_requests[0];
        match &req.target {
            crate::renderer::effects::EffectTarget::CharacterShow { alias } => {
                assert_eq!(alias, "char1");
            }
            other => panic!(
                "Expected CharacterShow for fade on character, got {:?}",
                other
            ),
        }
        assert!(req.effect.duration_or(0.0) > 0.0);
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

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let dur = executor.last_output.effect_requests[0]
            .effect
            .duration_or(0.0);
        assert!(
            (dur - 2.0).abs() < 0.01,
            "Character dissolve duration should be 2.0, got {}",
            dur
        );

        // 背景：dissolve(2.0)
        let cmd = Command::ShowBackground {
            path: "bg.png".to_string(),
            transition: Some(Transition::with_args(
                "dissolve",
                vec![vn_runtime::command::TransitionArg::Number(2.0)],
            )),
        };
        executor.execute(&cmd, &mut render_state, &resource_manager);

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        assert_eq!(
            executor.last_output.effect_requests[0].effect.duration,
            Some(2.0)
        );

        // changeScene fade(2.0)
        let cmd = Command::ChangeScene {
            path: "bg2.png".to_string(),
            transition: Some(Transition::with_args(
                "fade",
                vec![vn_runtime::command::TransitionArg::Number(2.0)],
            )),
        };
        executor.execute(&cmd, &mut render_state, &resource_manager);

        assert_eq!(executor.last_output.effect_requests.len(), 1);
        let scene_dur = executor.last_output.effect_requests[0]
            .effect
            .duration_or(0.0);
        assert!(
            (scene_dur - 2.0).abs() < 0.01,
            "Scene fade duration should be 2.0, got {}",
            scene_dur
        );
    }
}
