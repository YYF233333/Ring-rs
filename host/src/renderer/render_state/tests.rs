use super::*;
use vn_runtime::command::Position;

#[test]
fn test_render_state_default() {
    let state = RenderState::new();
    assert!(state.current_background.is_none());
    assert!(state.visible_characters.is_empty());
    assert!(state.dialogue.is_none());
    assert!(state.chapter_mark.is_none());
    assert!(state.choices.is_none());
    assert!(state.ui_visible);
}

#[test]
fn test_set_background() {
    let mut state = RenderState::new();
    state.set_background("bg.png".to_string());
    assert_eq!(state.current_background, Some("bg.png".to_string()));

    state.clear_background();
    assert!(state.current_background.is_none());
}

#[test]
fn test_show_hide_character() {
    let mut state = RenderState::new();

    // 显示角色
    state.show_character("alice".to_string(), "alice.png".to_string(), Position::Left);
    assert!(state.visible_characters.contains_key("alice"));
    assert_eq!(
        state.visible_characters.get("alice").unwrap().texture_path,
        "alice.png"
    );

    // 隐藏角色
    state.hide_character("alice");
    assert!(!state.visible_characters.contains_key("alice"));
}

#[test]
fn test_hide_all_characters() {
    let mut state = RenderState::new();

    state.show_character("alice".to_string(), "alice.png".to_string(), Position::Left);
    state.show_character("bob".to_string(), "bob.png".to_string(), Position::Right);
    assert_eq!(state.visible_characters.len(), 2);

    state.hide_all_characters();
    assert!(state.visible_characters.is_empty());
}

#[test]
fn test_character_fading_out() {
    let mut state = RenderState::new();

    state.show_character(
        "alice".to_string(),
        "alice.png".to_string(),
        Position::Center,
    );

    // 标记为淡出
    state.mark_character_fading_out("alice");
    assert!(state.visible_characters.get("alice").unwrap().fading_out);

    // 移除淡出完成的角色
    state.remove_fading_out_characters(&["alice".to_string()]);
    assert!(!state.visible_characters.contains_key("alice"));
}

#[test]
fn test_typewriter_effect() {
    let mut state = RenderState::new();

    // 开始打字机效果
    state.start_typewriter(Some("北风".to_string()), "你好世界".to_string());
    let dialogue = state.dialogue.as_ref().unwrap();
    assert_eq!(dialogue.visible_chars, 0);
    assert!(!dialogue.is_complete);
    assert!(!state.is_dialogue_complete());

    // 推进一个字符
    state.advance_typewriter();
    assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 1);

    // 推进直到完成
    while !state.is_dialogue_complete() {
        state.advance_typewriter();
    }
    assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 4); // "你好世界" = 4 个字符
    assert!(state.is_dialogue_complete());
}

#[test]
fn test_complete_typewriter() {
    let mut state = RenderState::new();

    state.start_typewriter(None, "测试文本".to_string());
    assert!(!state.is_dialogue_complete());

    // 立即完成
    state.complete_typewriter();
    assert!(state.is_dialogue_complete());
    assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 4);
}

#[test]
fn test_set_dialogue() {
    let mut state = RenderState::new();

    state.set_dialogue(Some("说话者".to_string()), "内容".to_string());
    let dialogue = state.dialogue.as_ref().unwrap();
    assert_eq!(dialogue.speaker, Some("说话者".to_string()));
    assert_eq!(dialogue.content, "内容");
    assert!(dialogue.is_complete); // set_dialogue 直接显示全部

    state.clear_dialogue();
    assert!(state.dialogue.is_none());
}

#[test]
fn test_chapter_mark() {
    let mut state = RenderState::new();

    state.set_chapter_mark("第一章".to_string(), 1);
    let chapter = state.chapter_mark.as_ref().unwrap();
    assert_eq!(chapter.title, "第一章");
    assert_eq!(chapter.level, 1);
    assert_eq!(chapter.alpha, 0.0); // 从 FadeIn 开始
    assert_eq!(chapter.phase, ChapterMarkPhase::FadeIn);

    state.clear_chapter_mark();
    assert!(state.chapter_mark.is_none());
}

#[test]
fn test_chapter_mark_animation_lifecycle() {
    let mut state = RenderState::new();

    state.set_chapter_mark("第一章".to_string(), 1);
    assert!(state.chapter_mark.is_some());

    // FadeIn 阶段 (FADE_IN_DURATION = 0.4s)
    state.update_chapter_mark(0.2);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
    assert!(mark.alpha > 0.0 && mark.alpha < 1.0);

    // 完成 FadeIn → Visible (累计 0.2 + 0.3 = 0.5 > 0.4)
    state.update_chapter_mark(0.3);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::Visible);
    assert_eq!(mark.alpha, 1.0);

    // Visible 期间保持 (VISIBLE_DURATION = 3.0s)
    state.update_chapter_mark(1.0);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::Visible);
    assert_eq!(mark.alpha, 1.0);

    // 完成 Visible → FadeOut (需要再过 2.1s 来超过 3.0s)
    state.update_chapter_mark(2.1);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);

    // FadeOut 阶段开始时 timer 被重置为 0，这个 update 后 timer=0.0 (刚进入)
    // alpha 应该接近 1.0 因为刚进入 FadeOut
    // 继续推进
    state.update_chapter_mark(0.3);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);
    assert!(mark.alpha < 1.0 && mark.alpha > 0.0);

    // 完成 FadeOut → 自动消失 (FADE_OUT_DURATION = 0.6s)
    state.update_chapter_mark(0.5);
    assert!(state.chapter_mark.is_none());
}

#[test]
fn test_chapter_mark_overlap_replace() {
    let mut state = RenderState::new();

    // 设置第一个
    state.set_chapter_mark("第一章".to_string(), 1);
    state.update_chapter_mark(0.5); // 进入 Visible 阶段

    // 设置第二个（覆盖第一个）
    state.set_chapter_mark("第二章".to_string(), 1);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.title, "第二章");
    assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
    assert_eq!(mark.alpha, 0.0);
}

#[test]
fn test_choices() {
    let mut state = RenderState::new();

    let choices = vec![
        ChoiceItem {
            text: "选项A".to_string(),
            target_label: "labelA".to_string(),
        },
        ChoiceItem {
            text: "选项B".to_string(),
            target_label: "labelB".to_string(),
        },
    ];

    state.set_choices(choices, Some("default".to_string()));
    let choices_state = state.choices.as_ref().unwrap();
    assert_eq!(choices_state.choices.len(), 2);
    assert_eq!(choices_state.style, Some("default".to_string()));
    assert_eq!(choices_state.selected_index, 0);

    state.clear_choices();
    assert!(state.choices.is_none());
}

#[test]
fn test_character_z_order() {
    let mut state = RenderState::new();

    state.show_character("first".to_string(), "first.png".to_string(), Position::Left);
    state.show_character(
        "second".to_string(),
        "second.png".to_string(),
        Position::Right,
    );

    // 后添加的角色 z_order 更大
    assert_eq!(state.visible_characters.get("first").unwrap().z_order, 0);
    assert_eq!(state.visible_characters.get("second").unwrap().z_order, 1);
}
