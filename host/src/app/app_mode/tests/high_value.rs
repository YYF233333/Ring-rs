use super::*;

// ============ NavigationStack 状态机与契约 ============

#[test]
fn test_navigation_stack_basic() {
    let mut nav = NavigationStack::new();
    assert_eq!(nav.current(), AppMode::Title);
    assert!(!nav.can_go_back());

    nav.navigate_to(AppMode::Settings);
    assert_eq!(nav.current(), AppMode::Settings);
    assert!(nav.can_go_back());

    nav.go_back();
    assert_eq!(nav.current(), AppMode::Title);
    assert!(!nav.can_go_back());
}

#[test]
fn test_navigation_stack_nested() {
    let mut nav = NavigationStack::new();

    // Title -> InGame (switch, no stack)
    nav.switch_to(AppMode::InGame);
    assert_eq!(nav.current(), AppMode::InGame);
    assert!(!nav.can_go_back());

    // InGame -> InGameMenu -> SaveLoad
    nav.navigate_to(AppMode::InGameMenu);
    nav.navigate_to(AppMode::SaveLoad);
    assert_eq!(nav.depth(), 2);

    // Back to InGameMenu
    nav.go_back();
    assert_eq!(nav.current(), AppMode::InGameMenu);

    // Back to InGame
    nav.go_back();
    assert_eq!(nav.current(), AppMode::InGame);
}

#[test]
fn test_navigation_return_to_title() {
    let mut nav = NavigationStack::new();
    nav.switch_to(AppMode::InGame);
    nav.navigate_to(AppMode::InGameMenu);
    nav.navigate_to(AppMode::SaveLoad);

    nav.return_to_title();
    assert_eq!(nav.current(), AppMode::Title);
    assert!(!nav.can_go_back());
}

#[test]
fn test_go_back_on_empty_stack_returns_none() {
    let mut nav = NavigationStack::new();
    assert_eq!(nav.go_back(), None);
    assert_eq!(nav.current(), AppMode::Title);
}

#[test]
fn test_navigate_to_same_mode_is_noop() {
    let mut nav = NavigationStack::new();
    nav.navigate_to(AppMode::Title);
    assert_eq!(nav.depth(), 0);
    assert_eq!(nav.current(), AppMode::Title);
}

#[test]
fn test_switch_to_clears_stack() {
    let mut nav = NavigationStack::new();
    nav.navigate_to(AppMode::InGameMenu);
    nav.navigate_to(AppMode::Settings);
    assert_eq!(nav.depth(), 2);

    nav.switch_to(AppMode::InGame);
    assert_eq!(nav.depth(), 0);
    assert_eq!(nav.current(), AppMode::InGame);
    assert!(!nav.can_go_back());
}

#[test]
fn test_go_back_returns_previous_mode() {
    let mut nav = NavigationStack::new();
    nav.navigate_to(AppMode::Settings);
    let prev = nav.go_back();
    assert_eq!(prev, Some(AppMode::Title));
}

#[test]
fn test_replace_current_does_not_push() {
    let mut nav = NavigationStack::new();
    nav.navigate_to(AppMode::InGame);
    nav.replace_current(AppMode::Settings);
    assert_eq!(nav.current(), AppMode::Settings);
    assert_eq!(nav.depth(), 1);
    let prev = nav.go_back();
    assert_eq!(prev, Some(AppMode::Title));
}
