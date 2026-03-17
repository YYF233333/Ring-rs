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
