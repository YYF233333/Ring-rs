use super::*;

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
fn test_effective_text_speed_no_override() {
    let mut state = RenderState::new();
    state.start_typewriter(None, "A".to_string(), vec![], false);
    assert_eq!(state.effective_text_speed(20.0), 20.0);
}

#[test]
fn test_effective_text_speed_no_dialogue() {
    let state = RenderState::new();
    assert_eq!(state.effective_text_speed(15.0), 15.0);
}

#[test]
fn test_get_character_anim_some() {
    let mut state = RenderState::new();
    state.show_character(
        "alice".to_string(),
        "alice.png".to_string(),
        Position::Center,
    );
    assert!(state.get_character_anim("alice").is_some());
    assert!(state.get_character_anim("bob").is_none());
}

#[test]
fn test_get_character_anim_mut_some() {
    let mut state = RenderState::new();
    state.show_character(
        "alice".to_string(),
        "alice.png".to_string(),
        Position::Center,
    );
    assert!(state.get_character_anim_mut("alice").is_some());
    assert!(state.get_character_anim_mut("bob").is_none());
}
