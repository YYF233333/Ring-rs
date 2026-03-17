use super::*;

#[test]
fn test_inject_input() {
    let mut manager = InputManager::new();
    manager.inject_input(RuntimeInput::Click);
    assert!(manager.pending_input.is_some());

    let result = manager.update(&WaitingReason::WaitForClick, 0.016);
    assert_eq!(result, Some(RuntimeInput::Click));
    assert!(manager.pending_input.is_none());
}

#[test]
fn test_begin_frame_advances_time() {
    let mut manager = InputManager::new();
    manager.begin_frame(0.016);
    assert!((manager.current_time - 0.016).abs() < 0.001);
    manager.begin_frame(0.033);
    assert!((manager.current_time - 0.049).abs() < 0.001);
}

#[test]
fn test_end_frame_clears_per_frame_state() {
    let mut manager = InputManager::new();
    manager.mouse_just_pressed = true;
    manager.just_pressed_keys.insert(KeyCode::Space);
    manager.end_frame();
    assert!(!manager.mouse_just_pressed);
    assert!(!manager.is_key_just_pressed(KeyCode::Space));
}

#[test]
fn test_update_waiting_none_returns_none() {
    let mut manager = InputManager::new();
    let result = manager.update(&WaitingReason::None, 0.016);
    assert!(result.is_none());
}

#[test]
fn test_update_waiting_for_signal_returns_none() {
    let mut manager = InputManager::new();
    let result = manager.update(
        &WaitingReason::WaitForSignal(vn_runtime::input::SignalId::new("sig")),
        0.016,
    );
    assert!(result.is_none());
}

#[test]
fn test_update_waiting_for_time_no_click_returns_none() {
    let mut manager = InputManager::new();
    let result = manager.update(
        &WaitingReason::WaitForTime(std::time::Duration::from_secs(1)),
        0.016,
    );
    assert!(result.is_none());
}

#[test]
fn test_update_click_via_mouse_just_pressed() {
    let mut manager = InputManager::new();
    // 确保时间超过防抖阈值 (CLICK_DEBOUNCE_SECONDS = 0.15)
    manager.current_time = 1.0;
    manager.last_click_time = 0.0; // 距上次点击超过 0.15s
    manager.mouse_just_pressed = true;

    let result = manager.update(&WaitingReason::WaitForClick, 0.016);
    assert_eq!(result, Some(RuntimeInput::Click));
}

#[test]
fn test_debounce_prevents_rapid_click() {
    let mut manager = InputManager::new();
    manager.current_time = 0.05; // only 0.05s since start
    manager.last_click_time = 0.0; // 0.05 - 0.0 = 0.05 < 0.15, should be blocked
    manager.mouse_just_pressed = true;

    let result = manager.update(&WaitingReason::WaitForClick, 0.016);
    assert!(result.is_none(), "debounce should block rapid click");
}

#[test]
fn test_choice_keyboard_navigation_down() {
    let mut manager = InputManager::new();
    manager.choice_count = 3;
    manager.selected_index = 0;
    manager.just_pressed_keys.insert(KeyCode::ArrowDown);

    let result = manager.update(&WaitingReason::WaitForChoice { choice_count: 3 }, 0.016);
    assert!(result.is_none());
    assert_eq!(manager.selected_index, 1);
}

#[test]
fn test_choice_keyboard_navigation_up_saturates() {
    let mut manager = InputManager::new();
    manager.choice_count = 3;
    manager.selected_index = 0;
    manager.just_pressed_keys.insert(KeyCode::ArrowUp);

    manager.update(&WaitingReason::WaitForChoice { choice_count: 3 }, 0.016);
    assert_eq!(manager.selected_index, 0);
}

#[test]
fn test_choice_keyboard_w_s_navigation() {
    let mut manager = InputManager::new();
    manager.choice_count = 3;
    manager.selected_index = 1;
    manager.just_pressed_keys.insert(KeyCode::KeyS);

    manager.update(&WaitingReason::WaitForChoice { choice_count: 3 }, 0.016);
    assert_eq!(manager.selected_index, 2);
}

#[test]
fn test_choice_enter_confirms_selection() {
    let mut manager = InputManager::new();
    manager.choice_count = 3;
    manager.selected_index = 2;
    manager.current_time = 1.0;
    manager.last_click_time = 0.0;
    manager.just_pressed_keys.insert(KeyCode::Enter);

    let result = manager.update(&WaitingReason::WaitForChoice { choice_count: 3 }, 0.016);
    assert_eq!(result, Some(RuntimeInput::ChoiceSelected { index: 2 }));
}

#[test]
fn test_choice_mouse_hover_in_rect() {
    let mut manager = InputManager::new();
    let rects = vec![(100.0, 200.0, 300.0, 50.0)];
    manager.mouse_position = (200.0, 220.0);
    let hovered = manager.handle_choice_hover(&rects);
    assert!(hovered);
    assert_eq!(manager.hovered_index, Some(0));
}

#[test]
fn test_choice_mouse_hover_outside_rect() {
    let mut manager = InputManager::new();
    let rects = vec![(100.0, 200.0, 300.0, 50.0)];
    manager.mouse_position = (0.0, 0.0);
    let hovered = manager.handle_choice_hover(&rects);
    assert!(!hovered);
    assert!(manager.hovered_index.is_none());
}

#[test]
fn test_choice_mouse_click_selects_hovered() {
    let mut manager = InputManager::new();
    manager.choice_count = 2;
    manager.choice_rects = vec![(0.0, 0.0, 100.0, 50.0), (0.0, 60.0, 100.0, 50.0)];
    manager.mouse_position = (50.0, 30.0);
    manager.hovered_index = Some(0);
    manager.current_time = 1.0;
    manager.last_click_time = 0.0;
    manager.mouse_just_pressed = true;

    let result = manager.update(&WaitingReason::WaitForChoice { choice_count: 2 }, 0.016);
    assert_eq!(result, Some(RuntimeInput::ChoiceSelected { index: 0 }));
}

#[test]
fn test_update_waiting_for_time_click_passes_debounce() {
    let mut manager = InputManager::new();
    manager.current_time = 1.0;
    manager.last_click_time = 0.0;
    manager.mouse_just_pressed = true;

    let result = manager.update(
        &WaitingReason::WaitForTime(std::time::Duration::from_secs(1)),
        0.016,
    );
    assert_eq!(result, Some(RuntimeInput::Click));
}
