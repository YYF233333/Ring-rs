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

// ============ AppMode / SaveLoadPage / UserSettings（自 low_value 迁入）===========

#[test]
fn test_input_capture() {
    assert_eq!(AppMode::Title.default_input_capture(), InputCapture::Menu);
    assert_eq!(AppMode::InGame.default_input_capture(), InputCapture::Game);
    assert_eq!(
        AppMode::InGameMenu.default_input_capture(),
        InputCapture::Menu
    );
}

#[test]
fn test_is_in_game_modes() {
    assert!(!AppMode::Title.is_in_game());
    assert!(AppMode::InGame.is_in_game());
    assert!(AppMode::InGameMenu.is_in_game());
    assert!(!AppMode::SaveLoad.is_in_game());
    assert!(!AppMode::Settings.is_in_game());
    assert!(AppMode::History.is_in_game());
}

#[test]
fn save_load_page_first_slot_manual() {
    assert_eq!(SaveLoadPage::Manual(1).first_slot(), 1);
    assert_eq!(SaveLoadPage::Manual(2).first_slot(), 7);
    assert_eq!(SaveLoadPage::Manual(9).first_slot(), 49);
}

#[test]
fn save_load_page_first_slot_special() {
    assert_eq!(SaveLoadPage::Quick.first_slot(), 55);
    assert_eq!(SaveLoadPage::Auto.first_slot(), 61);
}

#[test]
fn save_load_page_slot_range() {
    assert_eq!(SaveLoadPage::Manual(1).slot_range(), 1..=6);
    assert_eq!(SaveLoadPage::Manual(2).slot_range(), 7..=12);
    assert_eq!(SaveLoadPage::Quick.slot_range(), 55..=60);
    assert_eq!(SaveLoadPage::Auto.slot_range(), 61..=66);
}

#[test]
fn save_load_page_label() {
    assert_eq!(SaveLoadPage::Manual(1).label(), "1");
    assert_eq!(SaveLoadPage::Manual(9).label(), "9");
    assert_eq!(SaveLoadPage::Manual(10).label(), "?");
    assert_eq!(SaveLoadPage::Quick.label(), "Q");
    assert_eq!(SaveLoadPage::Auto.label(), "A");
}

#[test]
fn save_load_page_all_pages_count() {
    let pages = SaveLoadPage::all_pages();
    assert_eq!(pages.len(), 11);
    assert_eq!(pages[0], SaveLoadPage::Auto);
    assert_eq!(pages[1], SaveLoadPage::Quick);
    assert_eq!(pages[2], SaveLoadPage::Manual(1));
}

#[test]
fn save_load_page_prev_next() {
    assert_eq!(SaveLoadPage::Auto.prev(), None);
    assert_eq!(SaveLoadPage::Auto.next(), Some(SaveLoadPage::Quick));
    assert_eq!(SaveLoadPage::Quick.prev(), Some(SaveLoadPage::Auto));
    assert_eq!(SaveLoadPage::Quick.next(), Some(SaveLoadPage::Manual(1)));
    assert_eq!(SaveLoadPage::Manual(9).next(), None);
    assert_eq!(
        SaveLoadPage::Manual(5).prev(),
        Some(SaveLoadPage::Manual(4))
    );
    assert_eq!(
        SaveLoadPage::Manual(5).next(),
        Some(SaveLoadPage::Manual(6))
    );
}

#[test]
fn test_is_overlay() {
    assert!(!AppMode::Title.is_overlay());
    assert!(!AppMode::InGame.is_overlay());
    assert!(AppMode::InGameMenu.is_overlay());
    assert!(AppMode::History.is_overlay());
    assert!(!AppMode::SaveLoad.is_overlay());
    assert!(!AppMode::Settings.is_overlay());
}

#[test]
fn test_is_fullscreen_ui() {
    assert!(AppMode::Title.is_fullscreen_ui());
    assert!(!AppMode::InGame.is_fullscreen_ui());
    assert!(!AppMode::InGameMenu.is_fullscreen_ui());
    assert!(AppMode::SaveLoad.is_fullscreen_ui());
    assert!(AppMode::Settings.is_fullscreen_ui());
    assert!(!AppMode::History.is_fullscreen_ui());
}

#[test]
fn user_settings_load_nonexistent_returns_default() {
    let s = UserSettings::load("__nonexistent_settings__.json");
    assert!((s.bgm_volume - 0.8).abs() < 0.001);
}

#[test]
fn user_settings_save_load_round_trip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("test_settings.json");
    let path_str = path.to_str().expect("temp path is valid UTF-8");

    let settings = UserSettings {
        bgm_volume: 0.42,
        ..Default::default()
    };
    settings.save(path_str).unwrap();

    let loaded = UserSettings::load(path_str);
    assert!((loaded.bgm_volume - 0.42).abs() < 0.001);
}
