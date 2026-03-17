use super::*;

#[test]
fn test_input_manager_creation() {
    let manager = InputManager::new();
    assert_eq!(manager.selected_index, 0);
    assert_eq!(manager.choice_count, 0);
    assert!(manager.pending_input.is_none());
}

#[test]
fn test_reset_choice() {
    let mut manager = InputManager::new();
    manager.selected_index = 5;
    manager.reset_choice(3);
    assert_eq!(manager.selected_index, 0);
    assert_eq!(manager.choice_count, 3);
}

#[test]
fn test_mouse_position_default() {
    let manager = InputManager::new();
    assert_eq!(manager.mouse_position(), (0.0, 0.0));
}

#[test]
fn test_is_mouse_pressed_default_false() {
    let manager = InputManager::new();
    assert!(!manager.is_mouse_pressed());
}

#[test]
fn test_is_mouse_just_pressed_default_false() {
    let manager = InputManager::new();
    assert!(!manager.is_mouse_just_pressed());
}

#[test]
fn test_suppress_mouse_click() {
    let mut manager = InputManager::new();
    manager.mouse_just_pressed = true;
    assert!(manager.is_mouse_just_pressed());
    manager.suppress_mouse_click();
    assert!(!manager.is_mouse_just_pressed());
}

#[test]
fn test_get_selected_index() {
    let mut manager = InputManager::new();
    manager.selected_index = 3;
    assert_eq!(manager.get_selected_index(), 3);
}

#[test]
fn test_is_key_just_pressed() {
    let mut manager = InputManager::new();
    assert!(!manager.is_key_just_pressed(KeyCode::Space));
    manager.just_pressed_keys.insert(KeyCode::Space);
    assert!(manager.is_key_just_pressed(KeyCode::Space));
}

#[test]
fn test_is_key_down() {
    let mut manager = InputManager::new();
    assert!(!manager.is_key_down(KeyCode::Enter));
    manager.pressed_keys.insert(KeyCode::Enter);
    assert!(manager.is_key_down(KeyCode::Enter));
}

#[test]
fn test_is_clicking_with_mouse() {
    let mut manager = InputManager::new();
    assert!(!manager.is_clicking());
    manager.mouse_just_pressed = true;
    assert!(manager.is_clicking());
}

#[test]
fn test_is_clicking_with_space() {
    let mut manager = InputManager::new();
    manager.just_pressed_keys.insert(KeyCode::Space);
    assert!(manager.is_clicking());
}

#[test]
fn test_set_choice_rects() {
    let mut manager = InputManager::new();
    let rects = vec![(10.0, 20.0, 100.0, 50.0)];
    manager.set_choice_rects(rects);
    assert_eq!(manager.choice_rects, vec![(10.0, 20.0, 100.0, 50.0)]);
}
