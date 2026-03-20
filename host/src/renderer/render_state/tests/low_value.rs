use super::*;

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
fn test_advance_typewriter_does_not_advance_past_completion() {
    let mut state = RenderState::new();
    state.start_typewriter(None, "A".to_string(), vec![], false);

    assert!(state.advance_typewriter());
    assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 1);

    assert!(state.advance_typewriter());
    assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 1);
}

#[test]
fn test_update_chapter_mark_uses_linear_fraction_for_fade_in_and_out() {
    let mut state = RenderState::new();
    state.set_chapter_mark("第一章".to_string(), 1);

    assert!(state.update_chapter_mark(0.2));
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
    assert!((mark.alpha - 0.5).abs() < 0.01);

    assert!(state.update_chapter_mark(0.2));
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::Visible);
    assert!((mark.alpha - 1.0).abs() < 0.01);

    assert!(state.update_chapter_mark(3.0));
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);

    assert!(state.update_chapter_mark(0.3));
    let mark = state.chapter_mark.as_ref().unwrap();
    assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);
    assert!((mark.alpha - 0.5).abs() < 0.01);
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

// ============ advance_typewriter 内联效果测试 ============

#[test]
fn test_advance_typewriter_set_cps_absolute() {
    let mut state = RenderState::new();
    state.start_typewriter(
        None,
        "AB".to_string(),
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsAbsolute(30.0),
        }],
        false,
    );

    state.advance_typewriter();
    let d = state.dialogue.as_ref().unwrap();
    assert!(matches!(d.effective_cps, Some(EffectiveCps::Absolute(n)) if (n - 30.0).abs() < 0.01));
}

#[test]
fn test_advance_typewriter_set_cps_relative() {
    let mut state = RenderState::new();
    state.start_typewriter(
        None,
        "AB".to_string(),
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsRelative(2.0),
        }],
        false,
    );

    state.advance_typewriter();
    let d = state.dialogue.as_ref().unwrap();
    assert!(matches!(d.effective_cps, Some(EffectiveCps::Relative(m)) if (m - 2.0).abs() < 0.01));
}

#[test]
fn test_advance_typewriter_reset_cps() {
    let mut state = RenderState::new();
    state.start_typewriter(
        None,
        "ABC".to_string(),
        vec![
            InlineEffect {
                position: 1,
                kind: InlineEffectKind::SetCpsAbsolute(50.0),
            },
            InlineEffect {
                position: 2,
                kind: InlineEffectKind::ResetCps,
            },
        ],
        false,
    );

    state.advance_typewriter();
    assert!(state.dialogue.as_ref().unwrap().effective_cps.is_some());

    state.advance_typewriter();
    assert!(state.dialogue.as_ref().unwrap().effective_cps.is_none());
}

#[test]
fn test_advance_typewriter_returns_bool() {
    let mut state = RenderState::new();
    state.start_typewriter(None, "A".to_string(), vec![], false);
    assert!(state.advance_typewriter());
}

#[test]
fn test_advance_typewriter_no_dialogue_returns_true() {
    let mut state = RenderState::new();
    assert!(state.advance_typewriter());
}

// ============ inline_wait 状态管理测试 ============

#[test]
fn test_has_inline_wait() {
    let mut state = RenderState::new();
    assert!(!state.has_inline_wait());
    start_tw_with_wait(&mut state, Some(1.0));
    assert!(state.has_inline_wait());
}

#[test]
fn test_is_inline_click_wait_vs_timed() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, Some(1.0));
    assert!(!state.is_inline_click_wait());

    start_tw_with_wait(&mut state, None);
    assert!(state.is_inline_click_wait());
}

#[test]
fn test_clear_inline_wait() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, None);
    assert!(state.has_inline_wait());
    state.clear_inline_wait();
    assert!(!state.has_inline_wait());
}

#[test]
fn test_update_inline_wait_click_wait_not_consumed_by_time() {
    let mut state = RenderState::new();
    start_tw_with_wait(&mut state, None);
    let done = state.update_inline_wait(999.0);
    assert!(!done);
    assert!(state.has_inline_wait());
}

// ============ effective_text_speed 测试 ============

#[test]
fn test_effective_text_speed_absolute() {
    let mut state = RenderState::new();
    state.start_typewriter(
        None,
        "AB".to_string(),
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsAbsolute(50.0),
        }],
        false,
    );
    state.advance_typewriter();
    assert_eq!(state.effective_text_speed(20.0), 50.0);
}

#[test]
fn test_effective_text_speed_relative() {
    let mut state = RenderState::new();
    state.start_typewriter(
        None,
        "AB".to_string(),
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsRelative(3.0),
        }],
        false,
    );
    state.advance_typewriter();
    assert!((state.effective_text_speed(20.0) - 60.0).abs() < 0.01);
}
