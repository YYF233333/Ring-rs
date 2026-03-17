use super::*;

#[test]
fn action_parse_start_at_label() {
    let action: ActionDef = serde_json::from_str(r#"{"start_at_label": "Winter"}"#).unwrap();
    assert_eq!(action, ActionDef::StartAtLabel("Winter".into()));
}

#[test]
fn action_parse_all_string_variants() {
    let cases = [
        ("start_game", ActionDef::StartGame),
        ("continue_game", ActionDef::ContinueGame),
        ("open_load", ActionDef::OpenLoad),
        ("open_save", ActionDef::OpenSave),
        ("navigate_settings", ActionDef::NavigateSettings),
        ("navigate_history", ActionDef::NavigateHistory),
        ("replace_settings", ActionDef::ReplaceSettings),
        ("replace_history", ActionDef::ReplaceHistory),
        ("quick_save", ActionDef::QuickSave),
        ("quick_load", ActionDef::QuickLoad),
        ("toggle_skip", ActionDef::ToggleSkip),
        ("toggle_auto", ActionDef::ToggleAuto),
        ("go_back", ActionDef::GoBack),
        ("return_to_title", ActionDef::ReturnToTitle),
        ("return_to_game", ActionDef::ReturnToGame),
        ("exit", ActionDef::Exit),
    ];
    for (input, expected) in cases {
        let json = format!("\"{input}\"");
        let parsed: ActionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected, "failed for {input}");
    }
}

#[test]
fn button_def_full() {
    let json = r#"{
            "label": "冬篇",
            "action": {"start_at_label": "Winter"},
            "visible": "$persistent.complete_summer",
            "confirm": "确定？"
        }"#;
    let btn: ButtonDef = serde_json::from_str(json).unwrap();
    assert_eq!(btn.label, "冬篇");
    assert_eq!(btn.action, ActionDef::StartAtLabel("Winter".into()));
    assert_eq!(
        btn.visible,
        Some(ConditionDef::PersistentVar("complete_summer".into()))
    );
    assert_eq!(btn.confirm, Some("确定？".into()));
}

#[test]
fn button_def_minimal() {
    let json = r#"{"label": "开始", "action": "start_game"}"#;
    let btn: ButtonDef = serde_json::from_str(json).unwrap();
    assert_eq!(btn.label, "开始");
    assert_eq!(btn.action, ActionDef::StartGame);
    assert!(btn.visible.is_none());
    assert!(btn.confirm.is_none());
}

#[test]
fn screen_defs_error_display() {
    let not_found = ScreenDefsError::NotFound("ui/screens.json 不存在".into());
    assert!(not_found.to_string().contains("界面配置加载失败"));
    assert!(not_found.to_string().contains("ui/screens.json"));

    let parse_failed = ScreenDefsError::ParseFailed("unexpected token".into());
    assert!(parse_failed.to_string().contains("界面配置解析失败"));
    assert!(parse_failed.to_string().contains("unexpected token"));
}

#[test]
fn condition_evaluate_always() {
    let store = PersistentStore::empty();
    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert!(ConditionDef::Always.evaluate(&ctx));
}
