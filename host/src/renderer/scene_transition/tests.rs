use super::*;

#[test]
fn test_animatable_scene_transition() {
    let state = AnimatableSceneTransition::new();

    assert_eq!(state.progress(), 0.0);
    assert_eq!(state.mask_alpha(), 0.0);
    assert_eq!(state.ui_alpha(), 1.0);

    state.set_progress(0.5);
    assert_eq!(state.progress(), 0.5);

    state.reset();
    assert_eq!(state.progress(), 0.0);
    assert_eq!(state.ui_alpha(), 0.0);
}

#[test]
fn test_scene_transition_manager_creation() {
    let manager = SceneTransitionManager::new();
    assert_eq!(manager.phase(), SceneTransitionPhase::Idle);
    assert!(!manager.is_active());
}

#[test]
fn test_fade_transition() {
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(0.5, "new_bg.png".to_string());

    assert!(manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);

    // 模拟完成 FadeIn
    for _ in 0..10 {
        manager.update(0.1);
    }

    // 应该进入 FadeOut 或更后的阶段
    assert!(matches!(
        manager.phase(),
        SceneTransitionPhase::FadeOut
            | SceneTransitionPhase::UIFadeIn
            | SceneTransitionPhase::Completed
    ));
}

#[test]
fn test_rule_transition() {
    let mut manager = SceneTransitionManager::new();
    manager.start_rule(0.3, "new_bg.png".to_string(), "mask.png".to_string(), false);

    assert!(manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);
    assert!(manager.transition_type().is_some());
}

#[test]
fn test_skip_all() {
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(1.0, "new_bg.png".to_string());

    assert!(manager.is_active());
    manager.skip_all();

    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
    assert_eq!(manager.ui_alpha(), 1.0);
}

#[test]
fn test_midpoint_detected_for_fade_and_consumed_by_take_pending_background() {
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(0.2, "new_bg.png".to_string());

    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);
    assert!(!manager.is_at_midpoint());
    assert_eq!(manager.pending_background(), Some("new_bg.png"));

    // 推进足够的时间：FadeIn 完成并进入 FadeOut（且刚开始）
    manager.update(0.25);

    assert_eq!(manager.phase(), SceneTransitionPhase::FadeOut);
    assert!(manager.is_at_midpoint());

    // 一旦消费 pending_background，中间点不应再次触发
    assert_eq!(
        manager.take_pending_background().as_deref(),
        Some("new_bg.png")
    );
    assert!(manager.pending_background().is_none());
    assert!(!manager.is_at_midpoint());
}

#[test]
fn test_midpoint_detected_for_rule_after_blackout_and_consumed() {
    let mut manager = SceneTransitionManager::new();
    manager.start_rule(0.1, "new_bg.png".to_string(), "mask.png".to_string(), false);

    // 先完成 FadeIn → Blackout
    manager.update(0.2);
    assert_eq!(manager.phase(), SceneTransitionPhase::Blackout);
    assert!(!manager.is_at_midpoint());

    // 再推进超过黑屏停顿：进入 FadeOut（起点）
    manager.update(RULE_BLACKOUT_DURATION + 0.01);
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeOut);
    assert!(manager.is_at_midpoint());

    assert_eq!(
        manager.take_pending_background().as_deref(),
        Some("new_bg.png")
    );
    assert!(manager.pending_background().is_none());
    assert!(!manager.is_at_midpoint());
}

// ===== 阶段 26 新增：skip_to_end / 逐阶段跳过语义测试 =====

#[test]
fn test_skip_to_end_fade() {
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(1.0, "target_bg.png".to_string());

    assert!(manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);

    // skip_to_end 应返回 pending_background 并完成过渡
    let bg = manager.skip_to_end();
    assert_eq!(bg.as_deref(), Some("target_bg.png"));
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
    assert_eq!(manager.ui_alpha(), 1.0);
    // pending_background 已被取走
    assert!(manager.pending_background().is_none());
}

#[test]
fn test_skip_to_end_fade_white() {
    let mut manager = SceneTransitionManager::new();
    manager.start_fade_white(0.5, "white_bg.png".to_string());

    assert!(manager.is_active());

    let bg = manager.skip_to_end();
    assert_eq!(bg.as_deref(), Some("white_bg.png"));
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
}

#[test]
fn test_skip_to_end_rule() {
    let mut manager = SceneTransitionManager::new();
    manager.start_rule(
        0.5,
        "rule_bg.png".to_string(),
        "mask.png".to_string(),
        false,
    );

    assert!(manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);

    // skip_to_end 应返回 pending_background 并完成过渡
    let bg = manager.skip_to_end();
    assert_eq!(bg.as_deref(), Some("rule_bg.png"));
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
    assert_eq!(manager.ui_alpha(), 1.0);
    assert!(manager.pending_background().is_none());
}

#[test]
fn test_skip_to_end_during_fade_out() {
    // 即使在 FadeOut 阶段（背景可能已被 take），skip_to_end 也应安全完成
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(0.2, "mid_bg.png".to_string());

    // 推进到 FadeOut 阶段
    manager.update(0.25);
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeOut);

    // 模拟中间点背景已被消费
    let _ = manager.take_pending_background();

    // skip_to_end 应返回 None（背景已消费）并安全完成
    let bg = manager.skip_to_end();
    assert!(bg.is_none());
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
}

#[test]
fn test_skip_current_phase_fade_ensures_midpoint() {
    // 验证：逐阶段跳过 Fade 时，FadeIn → FadeOut 跳转后 midpoint 可被检测
    let mut manager = SceneTransitionManager::new();
    manager.start_fade(1.0, "phase_bg.png".to_string());

    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);
    assert!(!manager.is_at_midpoint());

    // 跳过 FadeIn → 直接进入 FadeOut
    manager.skip_current_phase();
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeOut);

    // 此时 midpoint 应被检测到（mask_alpha == 1.0，pending_background 仍在）
    assert!(manager.is_at_midpoint());
    assert_eq!(manager.pending_background(), Some("phase_bg.png"));

    // 消费背景
    let bg = manager.take_pending_background();
    assert_eq!(bg.as_deref(), Some("phase_bg.png"));
    assert!(!manager.is_at_midpoint());

    // 再跳过 FadeOut → Completed
    manager.skip_current_phase();
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
}

#[test]
fn test_skip_current_phase_rule_ensures_midpoint() {
    // 验证：逐阶段跳过 Rule 时，FadeIn → FadeOut 跳转后 midpoint 可被检测
    let mut manager = SceneTransitionManager::new();
    manager.start_rule(
        1.0,
        "rule_phase_bg.png".to_string(),
        "mask.png".to_string(),
        false,
    );

    assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);

    // 跳过 FadeIn → 直接进入 FadeOut（Rule 跳过 Blackout）
    manager.skip_current_phase();
    assert_eq!(manager.phase(), SceneTransitionPhase::FadeOut);

    // midpoint 应被检测到（progress == 0.0，pending_background 仍在）
    assert!(manager.is_at_midpoint());
    assert_eq!(manager.pending_background(), Some("rule_phase_bg.png"));

    // 消费背景
    let bg = manager.take_pending_background();
    assert_eq!(bg.as_deref(), Some("rule_phase_bg.png"));

    // 跳过 FadeOut → Completed
    manager.skip_current_phase();
    assert!(!manager.is_active());
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
}
