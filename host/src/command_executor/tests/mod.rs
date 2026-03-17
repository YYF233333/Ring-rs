use super::*;
use vn_runtime::command::{Choice, Position, Transition};

struct TestCtx {
    executor: CommandExecutor,
    render_state: RenderState,
    resource_manager: ResourceManager,
}

impl TestCtx {
    fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            render_state: RenderState::new(),
            resource_manager: ResourceManager::new("assets", 256),
        }
    }

    fn execute(&mut self, cmd: &Command) -> ExecuteResult {
        self.executor
            .execute(cmd, &mut self.render_state, &self.resource_manager)
    }

    fn execute_batch(&mut self, commands: &[Command]) -> ExecuteResult {
        self.executor
            .execute_batch(commands, &mut self.render_state, &self.resource_manager)
    }
}

mod high_value;
mod low_value;

#[test]
fn test_execute_show_text() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowText {
        speaker: Some("北风".to_string()),
        content: "你好".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::WaitForClick);
    assert!(ctx.render_state.dialogue.is_some());

    let dialogue = ctx.render_state.dialogue.as_ref().unwrap();
    assert_eq!(dialogue.speaker, Some("北风".to_string()));
    assert_eq!(dialogue.content, "你好");
}

#[test]
fn test_execute_show_text_narrator() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowText {
        speaker: None,
        content: "旁白内容".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::WaitForClick);

    let dialogue = ctx.render_state.dialogue.as_ref().unwrap();
    assert_eq!(dialogue.speaker, None);
}

#[test]
fn test_execute_present_choices() {
    let mut ctx = TestCtx::new();

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

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::WaitForChoice { choice_count: 2 });
    assert!(ctx.render_state.choices.is_some());

    let choices = ctx.render_state.choices.as_ref().unwrap();
    assert_eq!(choices.choices.len(), 2);
    assert_eq!(choices.choices[0].text, "选项1");
    assert_eq!(choices.choices[1].target_label, "label2");
}

#[test]
fn test_execute_show_background() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowBackground {
        path: "backgrounds/bg1.png".to_string(),
        transition: None,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(
        ctx.render_state.current_background,
        Some("backgrounds/bg1.png".to_string())
    );
}

#[test]
fn test_execute_show_character() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.render_state.visible_characters.contains_key("char1"));

    let char_sprite = ctx.render_state.visible_characters.get("char1").unwrap();
    assert_eq!(char_sprite.texture_path, "characters/char1.png");
    assert_eq!(char_sprite.position, Position::Center);
}

#[test]
fn test_execute_show_character_reposition_with_dissolve_is_teleport() {
    let mut ctx = TestCtx::new();

    // 先显示角色（无过渡）
    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.effect_requests.is_empty());

    // 位置变更：with dissolve 只应“瞬移”（不触发 Move 动画）
    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Left,
        transition: Some(Transition::simple("dissolve")),
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.effect_requests.is_empty());

    let char_sprite = ctx.render_state.visible_characters.get("char1").unwrap();
    assert_eq!(char_sprite.position, Position::Left);
}

#[test]
fn test_execute_hide_character() {
    let mut ctx = TestCtx::new();

    // 先显示角色
    ctx.render_state.show_character(
        "char1".to_string(),
        "characters/char1.png".to_string(),
        Position::Center,
    );

    let cmd = Command::HideCharacter {
        alias: "char1".to_string(),
        transition: None,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(!ctx.render_state.visible_characters.contains_key("char1"));
}

#[test]
fn test_execute_chapter_mark() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ChapterMark {
        title: "第一章".to_string(),
        level: 1,
    };

    // ChapterMark 是非阻塞的
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.render_state.chapter_mark.is_some());

    let chapter = ctx.render_state.chapter_mark.as_ref().unwrap();
    assert_eq!(chapter.title, "第一章");
    assert_eq!(chapter.level, 1);
    assert_eq!(chapter.alpha, 0.0); // 从 FadeIn 开始
}

#[test]
fn test_execute_play_bgm() {
    let mut ctx = TestCtx::new();

    let cmd = Command::PlayBgm {
        path: "bgm/music.mp3".to_string(),
        looping: true,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.audio_command.is_some());

    if let Some(AudioCommand::PlayBgm { path, looping, .. }) =
        &ctx.executor.last_output.audio_command
    {
        assert_eq!(path, "bgm/music.mp3");
        assert!(*looping);
    } else {
        panic!("Expected PlayBgm command");
    }
}

#[test]
fn test_execute_stop_bgm() {
    let mut ctx = TestCtx::new();

    let cmd = Command::StopBgm {
        fade_out: Some(1.0),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.audio_command.is_some());

    if let Some(AudioCommand::StopBgm { fade_out }) = &ctx.executor.last_output.audio_command {
        assert_eq!(*fade_out, Some(1.0));
    } else {
        panic!("Expected StopBgm command");
    }
}

#[test]
fn test_execute_play_sfx() {
    let mut ctx = TestCtx::new();

    let cmd = Command::PlaySfx {
        path: "sfx/click.wav".to_string(),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.audio_command.is_some());

    if let Some(AudioCommand::PlaySfx { path }) = &ctx.executor.last_output.audio_command {
        assert_eq!(path, "sfx/click.wav");
    } else {
        panic!("Expected PlaySfx command");
    }
}

#[test]
fn test_execute_bgm_duck() {
    let mut ctx = TestCtx::new();

    let result = ctx.execute(&Command::BgmDuck);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(matches!(
        ctx.executor.last_output.audio_command,
        Some(AudioCommand::BgmDuck)
    ));
}

#[test]
fn test_execute_bgm_unduck() {
    let mut ctx = TestCtx::new();

    let result = ctx.execute(&Command::BgmUnduck);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(matches!(
        ctx.executor.last_output.audio_command,
        Some(AudioCommand::BgmUnduck)
    ));
}

// test_transition_progress 已移除：transition timer 已从 CommandExecutor 删除

#[test]
fn test_execute_batch() {
    let mut ctx = TestCtx::new();

    let commands = vec![
        Command::ShowBackground {
            path: "bg.png".to_string(),
            transition: None,
        },
        Command::ShowText {
            speaker: Some("角色".to_string()),
            content: "对话".to_string(),
            inline_effects: vec![],
            no_wait: false,
        },
    ];

    let result = ctx.execute_batch(&commands);
    // 最后一个需要等待的结果
    assert_eq!(result, ExecuteResult::WaitForClick);
    assert!(ctx.render_state.dialogue.is_some());
    assert_eq!(
        ctx.render_state.current_background,
        Some("bg.png".to_string())
    );
}

// ========== 效果矩阵测试 ==========
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

    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: Some(Transition::simple("dissolve")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    // dissolve 应产生 CharacterShow 效果请求（alpha 淡入）
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
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
    let mut ctx = TestCtx::new();

    // 先显示角色
    ctx.render_state.show_character(
        "char1".to_string(),
        "characters/char1.png".to_string(),
        Position::Center,
    );

    let cmd = Command::HideCharacter {
        alias: "char1".to_string(),
        transition: Some(Transition::simple("dissolve")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    // dissolve 应产生 CharacterHide 效果请求（alpha 淡出）
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
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

    let mut ctx = TestCtx::new();

    ctx.render_state.set_background("old_bg.png".to_string());

    let cmd = Command::ShowBackground {
        path: "new_bg.png".to_string(),
        transition: Some(Transition::simple("dissolve")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
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

    let mut ctx = TestCtx::new();

    let cmd = Command::ChangeScene {
        path: "new_bg.png".to_string(),
        transition: Some(Transition::simple("fade")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    // fade 在 changeScene 上下文应产生 SceneTransition 效果请求
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
    match &req.target {
        EffectTarget::SceneTransition { pending_background } => {
            assert_eq!(pending_background, "new_bg.png");
        }
        other => panic!("Expected SceneTransition, got {:?}", other),
    }
    assert_eq!(req.capability_id, "effect.fade");
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

    let mut ctx = TestCtx::new();

    ctx.render_state.set_background("old_bg.png".to_string());

    let cmd = Command::ChangeScene {
        path: "new_bg.png".to_string(),
        transition: Some(Transition::simple("dissolve")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    // dissolve 在 changeScene 上下文应产生 BackgroundTransition 效果请求
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
    match &req.target {
        EffectTarget::BackgroundTransition { old_background } => {
            assert_eq!(*old_background, Some("old_bg.png".to_string()));
        }
        other => panic!("Expected BackgroundTransition, got {:?}", other),
    }
    assert_eq!(req.effect.kind, EffectKind::Dissolve);
}

#[test]
fn test_fade_on_character_is_alpha_not_scene_mask() {
    // fade 在立绘上下文中等价于 dissolve（alpha 淡入），不是黑屏遮罩
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: Some(Transition::simple("fade")),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    // 应产生 CharacterShow 效果请求（alpha 淡入），而非场景过渡
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
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

// ========== 边界测试补充 ==========

#[test]
fn test_scene_effect_shake_no_duration() {
    use vn_runtime::command::TransitionArg;

    let mut ctx = TestCtx::new();
    let cmd = Command::SceneEffect {
        name: "shake".to_string(),
        args: vec![(Some("amplitude".to_string()), TransitionArg::Number(10.0))],
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    assert_eq!(
        ctx.executor.last_output.effect_requests[0].effect.duration,
        None
    );
}

#[test]
fn test_scene_effect_extra_params_forwarded() {
    use vn_runtime::command::TransitionArg;

    let mut ctx = TestCtx::new();
    let cmd = Command::SceneEffect {
        name: "blur".to_string(),
        args: vec![
            (Some("duration".to_string()), TransitionArg::Number(0.5)),
            (Some("amount".to_string()), TransitionArg::Number(3.0)),
            (Some("animated".to_string()), TransitionArg::Bool(true)),
        ],
    };

    ctx.execute(&cmd);
    let req = &ctx.executor.last_output.effect_requests[0];
    assert!(req.params.contains_key("amount"));
    assert!(req.params.contains_key("animated"));
    // duration is in params as it's built from the effect
    // but "duration" from args is filtered out of extra_params in execute_scene_effect
}

#[test]
fn test_title_card_sets_render_state_and_effect() {
    use crate::renderer::effects::EffectTarget;

    let mut ctx = TestCtx::new();
    let cmd = Command::TitleCard {
        text: "Chapter 1".to_string(),
        duration: 3.0,
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    let tc = ctx.render_state.title_card.as_ref().unwrap();
    assert_eq!(tc.text, "Chapter 1");
    assert!((tc.duration - 3.0).abs() < 0.01);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    assert!(matches!(
        &ctx.executor.last_output.effect_requests[0].target,
        EffectTarget::TitleCard { text } if text == "Chapter 1"
    ));
}

#[test]
fn test_play_bgm_default_fade_in() {
    let mut ctx = TestCtx::new();
    let cmd = Command::PlayBgm {
        path: "bgm/test.mp3".to_string(),
        looping: false,
    };

    ctx.execute(&cmd);
    if let Some(AudioCommand::PlayBgm {
        looping, fade_in, ..
    }) = &ctx.executor.last_output.audio_command
    {
        assert!(!*looping);
        assert_eq!(*fade_in, Some(0.5));
    } else {
        panic!("Expected PlayBgm");
    }
}

#[test]
fn test_stop_bgm_no_fade() {
    let mut ctx = TestCtx::new();
    let cmd = Command::StopBgm { fade_out: None };

    ctx.execute(&cmd);
    if let Some(AudioCommand::StopBgm { fade_out }) = &ctx.executor.last_output.audio_command {
        assert_eq!(*fade_out, None);
    } else {
        panic!("Expected StopBgm");
    }
}

#[test]
fn test_show_character_duplicate_same_position_same_texture() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    ctx.execute(&cmd);
    assert!(ctx.render_state.visible_characters.contains_key("char1"));

    // Show same character, same position, same texture: diff_only path (alpha >= 0.99)
    ctx.execute(&cmd);
    assert!(ctx.executor.last_output.effect_requests.is_empty());
    assert_eq!(
        ctx.render_state
            .visible_characters
            .get("char1")
            .unwrap()
            .position,
        Position::Center
    );
}

#[test]
fn test_show_character_texture_change_same_position() {
    let mut ctx = TestCtx::new();

    let cmd1 = Command::ShowCharacter {
        path: "characters/char1_a.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    ctx.execute(&cmd1);

    let cmd2 = Command::ShowCharacter {
        path: "characters/char1_b.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    ctx.execute(&cmd2);

    // Texture should be updated
    let sprite = ctx.render_state.visible_characters.get("char1").unwrap();
    assert_eq!(sprite.texture_path, "characters/char1_b.png");
    assert_eq!(sprite.position, Position::Center);
}

#[test]
fn test_hide_nonexistent_character_no_panic() {
    let mut ctx = TestCtx::new();

    let cmd = Command::HideCharacter {
        alias: "nonexistent".to_string(),
        transition: None,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
}

#[test]
fn test_hide_character_with_dissolve_marks_fading() {
    let mut ctx = TestCtx::new();

    ctx.render_state.show_character(
        "char1".to_string(),
        "characters/char1.png".to_string(),
        Position::Center,
    );

    let cmd = Command::HideCharacter {
        alias: "char1".to_string(),
        transition: Some(Transition::simple("dissolve")),
    };
    ctx.execute(&cmd);

    // Character should still be in visible_characters (fading out)
    assert!(ctx.render_state.visible_characters.contains_key("char1"));
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
}

#[test]
fn test_change_scene_no_transition_immediate() {
    let mut ctx = TestCtx::new();
    ctx.render_state.set_background("old.png".to_string());

    let cmd = Command::ChangeScene {
        path: "new.png".to_string(),
        transition: None,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(
        ctx.render_state.current_background,
        Some("new.png".to_string())
    );
    assert!(ctx.executor.last_output.effect_requests.is_empty());
}

#[test]
fn test_change_scene_fade_white() {
    use crate::renderer::effects::{EffectKind, EffectTarget};

    let mut ctx = TestCtx::new();
    let cmd = Command::ChangeScene {
        path: "new_bg.png".to_string(),
        transition: Some(Transition::simple("fadeWhite")),
    };

    ctx.execute(&cmd);
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);

    let req = &ctx.executor.last_output.effect_requests[0];
    assert!(matches!(
        &req.target,
        EffectTarget::SceneTransition { pending_background } if pending_background == "new_bg.png"
    ));
    assert_eq!(req.effect.kind, EffectKind::FadeWhite);
}

#[test]
fn test_text_box_hide_show_clear() {
    let mut ctx = TestCtx::new();
    assert!(ctx.render_state.ui_visible);

    ctx.execute(&Command::TextBoxHide);
    assert!(!ctx.render_state.ui_visible);

    ctx.execute(&Command::TextBoxShow);
    assert!(ctx.render_state.ui_visible);

    // Set up some dialogue first
    let cmd = Command::ShowText {
        speaker: Some("A".to_string()),
        content: "hello".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    ctx.execute(&cmd);
    assert!(ctx.render_state.dialogue.is_some());

    ctx.execute(&Command::TextBoxClear);
    assert!(ctx.render_state.dialogue.is_none());
}

#[test]
fn test_clear_characters() {
    let mut ctx = TestCtx::new();

    ctx.render_state.show_character(
        "a".to_string(),
        "characters/a.png".to_string(),
        Position::Left,
    );
    ctx.render_state.show_character(
        "b".to_string(),
        "characters/b.png".to_string(),
        Position::Right,
    );
    assert_eq!(ctx.render_state.visible_characters.len(), 2);

    ctx.execute(&Command::ClearCharacters);
    assert!(ctx.render_state.visible_characters.is_empty());
}

#[test]
fn test_extend_text() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowText {
        speaker: Some("A".to_string()),
        content: "hello".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    ctx.execute(&cmd);

    let cmd = Command::ExtendText {
        content: " world".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::WaitForClick);
}
