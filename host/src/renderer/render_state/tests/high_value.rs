use super::*;

#[test]
fn test_chapter_mark_animation_lifecycle() {
    let mut state = RenderState::new();

    state.set_chapter_mark("第一章".to_string(), 1);
    assert!(state.chapter_mark.is_some());

    state.update_chapter_mark(0.2);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
    assert!(mark.alpha > 0.0 && mark.alpha < 1.0);

    state.update_chapter_mark(0.3);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::Visible);
    assert_eq!(mark.alpha, 1.0);

    state.update_chapter_mark(1.0);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::Visible);
    assert_eq!(mark.alpha, 1.0);

    state.update_chapter_mark(2.1);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);

    state.update_chapter_mark(0.3);
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);
    assert!(mark.alpha < 1.0 && mark.alpha > 0.0);

    state.update_chapter_mark(0.5);
    assert!(state.chapter_mark.is_none());
}

#[test]
fn test_advance_typewriter_inline_wait_timed() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, Some(2.5));
    let d = state.dialogue.as_ref().unwrap();
    assert_eq!(d.inline_wait.as_ref().unwrap().remaining, Some(2.5));
}

#[test]
fn test_update_inline_wait_timed_countdown() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, Some(1.0));
    assert!(state.has_inline_wait());

    let done = state.update_inline_wait(0.5);
    assert!(!done);
    assert!(state.has_inline_wait());

    let done = state.update_inline_wait(0.6);
    assert!(done);
    assert!(!state.has_inline_wait());
}

#[test]
fn test_extend_dialogue_offsets_inline_effects() {
    let mut state = RenderState::new();
    state.start_typewriter(None, "AB".to_string(), vec![], false);
    state.complete_typewriter();

    state.extend_dialogue(
        "CD",
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(None),
        }],
        false,
    );
    let d = state.dialogue.as_ref().unwrap();
    assert_eq!(d.inline_effects.len(), 1);
    assert_eq!(d.inline_effects[0].position, 3);
}

#[test]
fn test_remove_fading_out_only_removes_fading() {
    let mut state = RenderState::new();
    state.show_character("alice".to_string(), "a.png".to_string(), Position::Center);
    state.show_character("bob".to_string(), "b.png".to_string(), Position::Left);
    state.mark_character_fading_out("alice");

    state.remove_fading_out_characters(&["alice".to_string(), "bob".to_string()]);
    assert!(!state.visible_characters.contains_key("alice"));
    assert!(state.visible_characters.contains_key("bob"));
}

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
fn test_complete_typewriter() {
    let mut state = RenderState::new();

    state.start_typewriter(None, "测试文本".to_string(), vec![], false);
    assert!(!state.is_dialogue_complete());

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
    assert!(dialogue.is_complete);

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
    assert_eq!(chapter.alpha, 0.0);
    assert_eq!(chapter.phase, ChapterMarkPhase::FadeIn);

    state.clear_chapter_mark();
    assert!(state.chapter_mark.is_none());
}

#[test]
fn test_typewriter_effect() {
    let mut state = RenderState::new();

    // 开始打字机效果
    state.start_typewriter(
        Some("北风".to_string()),
        "你好世界".to_string(),
        vec![],
        false,
    );
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
fn test_advance_typewriter_inline_wait_click() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, None);
    let d = state.dialogue.as_ref().unwrap();
    assert!(d.inline_wait.as_ref().unwrap().remaining.is_none());
}

#[test]
fn test_extend_dialogue_appends_content() {
    let mut state = RenderState::new();
    state.start_typewriter(None, "Hello".to_string(), vec![], false);
    state.complete_typewriter();
    assert!(state.is_dialogue_complete());

    state.extend_dialogue(" World", vec![], false);
    let d = state.dialogue.as_ref().unwrap();
    assert_eq!(d.content, "Hello World");
    assert!(!d.is_complete);
    assert_eq!(d.visible_chars, 5);
}
