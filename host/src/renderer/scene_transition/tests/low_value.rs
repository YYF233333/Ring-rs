use super::*;
use crate::renderer::animation::Animatable;

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
fn test_animatable_scene_transition_trait_property_contract() {
    let state = AnimatableSceneTransition::new();

    assert_eq!(
        state.property_list(),
        &["progress", "mask_alpha", "ui_alpha"]
    );
    assert_eq!(state.get_property("progress"), Some(0.0));
    assert_eq!(state.get_property("mask_alpha"), Some(0.0));
    assert_eq!(state.get_property("ui_alpha"), Some(1.0));
    assert_eq!(state.get_property("unknown"), None);

    assert!(state.set_property("progress", 0.25));
    assert!(state.set_property("mask_alpha", 0.75));
    assert!(state.set_property("ui_alpha", 0.5));
    assert!(!state.set_property("unknown", 1.0));

    assert_eq!(state.progress(), 0.25);
    assert_eq!(state.mask_alpha(), 0.75);
    assert_eq!(state.ui_alpha(), 0.5);
}

#[test]
fn test_animatable_scene_transition_direct_setters_update_fields() {
    let state = AnimatableSceneTransition::new();

    state.set_mask_alpha(0.33);
    state.set_ui_alpha(0.66);

    assert!((state.mask_alpha() - 0.33).abs() < 0.001);
    assert!((state.ui_alpha() - 0.66).abs() < 0.001);
}

#[test]
fn test_scene_transition_update_returns_false_for_idle_and_completed() {
    let mut manager = SceneTransitionManager::new();
    assert!(!manager.update(0.1));

    manager.start_fade(0.1, "done.png".to_string());
    manager.skip_all();
    assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
    assert!(!manager.update(0.1));
}
