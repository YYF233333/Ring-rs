use super::*;

#[test]
fn test_reset_choice() {
    let mut manager = InputManager::new();
    manager.choice.selected_index = 5;
    manager.reset_choice(3);
    assert_eq!(manager.choice.selected_index, 0);
    assert_eq!(manager.choice.choice_count, 3);
}

#[test]
fn test_is_key_just_pressed() {
    let mut manager = InputManager::new();
    assert!(!manager.is_key_just_pressed(KeyCode::Space));
    manager.state.just_pressed_keys.insert(KeyCode::Space);
    assert!(manager.is_key_just_pressed(KeyCode::Space));
}

#[test]
fn test_is_key_down() {
    let mut manager = InputManager::new();
    assert!(!manager.is_key_down(KeyCode::Enter));
    manager.state.pressed_keys.insert(KeyCode::Enter);
    assert!(manager.is_key_down(KeyCode::Enter));
}
