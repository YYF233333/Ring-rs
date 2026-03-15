use super::*;

#[test]
fn condition_parse_has_continue() {
    let cond: ConditionDef = serde_json::from_str(r#""$has_continue""#).unwrap();
    assert_eq!(cond, ConditionDef::HasContinue);
}

#[test]
fn condition_parse_persistent_var() {
    let cond: ConditionDef = serde_json::from_str(r#""$persistent.complete_summer""#).unwrap();
    assert_eq!(cond, ConditionDef::PersistentVar("complete_summer".into()));
}

#[test]
fn condition_parse_not_persistent_var() {
    let cond: ConditionDef = serde_json::from_str(r#""!$persistent.complete_summer""#).unwrap();
    assert_eq!(
        cond,
        ConditionDef::NotPersistentVar("complete_summer".into())
    );
}

#[test]
fn condition_parse_true() {
    let cond: ConditionDef = serde_json::from_str(r#""true""#).unwrap();
    assert_eq!(cond, ConditionDef::Always);
}

#[test]
fn action_parse_simple_string() {
    let action: ActionDef = serde_json::from_str(r#""start_game""#).unwrap();
    assert_eq!(action, ActionDef::StartGame);
}

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
fn action_parse_unknown_variant_errors() {
    let result = serde_json::from_str::<ActionDef>(r#""unknown_action""#);
    assert!(result.is_err());
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
fn screen_definitions_missing_field_returns_error() {
    let json = r#"{
            "title": {
                "background": [],
                "overlay": null,
                "buttons": [
                    {"label": "Play", "action": "start_game"}
                ]
            }
        }"#;
    let result = serde_json::from_str::<ScreenDefinitions>(json);
    assert!(
        result.is_err(),
        "missing ingame_menu/quick_menu/game_menu should fail"
    );
}

#[test]
fn screen_definitions_default_matches_hardcoded() {
    let defs = ScreenDefinitions::default();

    // Title: 6 buttons
    assert_eq!(defs.title.buttons.len(), 6);
    assert_eq!(defs.title.buttons[0].label, "开始游戏");
    assert_eq!(defs.title.buttons[0].action, ActionDef::StartGame);
    assert_eq!(defs.title.buttons[1].label, "冬篇");
    assert_eq!(
        defs.title.buttons[1].action,
        ActionDef::StartAtLabel("Winter".into())
    );
    assert_eq!(defs.title.buttons[5].label, "退出");
    assert_eq!(defs.title.buttons[5].confirm, Some("确定退出游戏？".into()));

    // Title background: winter (conditional) + summer (fallback)
    assert_eq!(defs.title.background.len(), 2);
    assert_eq!(defs.title.background[0].asset, "main_winter");
    assert_eq!(defs.title.background[1].asset, "main_summer");

    // Ingame menu: 7 buttons
    assert_eq!(defs.ingame_menu.buttons.len(), 7);
    assert_eq!(defs.ingame_menu.buttons[0].label, "继续");

    // Quick menu: 7 buttons
    assert_eq!(defs.quick_menu.buttons.len(), 7);
    assert_eq!(defs.quick_menu.buttons[0].label, "历史");

    // Game menu: 6 nav + return
    assert_eq!(defs.game_menu.nav_buttons.len(), 6);
    assert_eq!(defs.game_menu.return_button.label, "返回");
}

#[test]
fn conditional_asset_resolve_fallback() {
    let assets = vec![
        ConditionalAsset {
            when: Some(ConditionDef::PersistentVar("complete_summer".into())),
            asset: "main_winter".into(),
        },
        ConditionalAsset {
            when: None,
            asset: "main_summer".into(),
        },
    ];

    let store = PersistentStore::empty();
    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert_eq!(
        ConditionalAsset::resolve(&assets, &ctx),
        Some("main_summer")
    );
}

#[test]
fn conditional_asset_resolve_match() {
    let assets = vec![
        ConditionalAsset {
            when: Some(ConditionDef::PersistentVar("complete_summer".into())),
            asset: "main_winter".into(),
        },
        ConditionalAsset {
            when: None,
            asset: "main_summer".into(),
        },
    ];

    let mut store = PersistentStore::empty();
    store
        .variables
        .insert("complete_summer".into(), VarValue::Bool(true));
    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert_eq!(
        ConditionalAsset::resolve(&assets, &ctx),
        Some("main_winter")
    );
}

#[test]
fn condition_evaluate_has_continue() {
    let store = PersistentStore::empty();
    let ctx_true = ConditionContext {
        has_continue: true,
        persistent: &store,
    };
    let ctx_false = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert!(ConditionDef::HasContinue.evaluate(&ctx_true));
    assert!(!ConditionDef::HasContinue.evaluate(&ctx_false));
}

#[test]
fn condition_evaluate_persistent_var() {
    let mut store = PersistentStore::empty();
    store
        .variables
        .insert("complete_summer".into(), VarValue::Bool(true));

    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };

    assert!(ConditionDef::PersistentVar("complete_summer".into()).evaluate(&ctx));
    assert!(!ConditionDef::PersistentVar("nonexistent".into()).evaluate(&ctx));
    assert!(!ConditionDef::NotPersistentVar("complete_summer".into()).evaluate(&ctx));
    assert!(ConditionDef::NotPersistentVar("nonexistent".into()).evaluate(&ctx));
}
