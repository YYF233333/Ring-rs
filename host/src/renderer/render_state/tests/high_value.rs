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
