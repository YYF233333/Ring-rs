use super::*;
use crate::resources::ResourceManager;

#[test]
fn condition_parse_cases() {
    let cases = [
        (r#""$has_continue""#, ConditionDef::HasContinue),
        (
            r#""$persistent.complete_summer""#,
            ConditionDef::PersistentVar("complete_summer".into()),
        ),
        (
            r#""!$persistent.complete_summer""#,
            ConditionDef::NotPersistentVar("complete_summer".into()),
        ),
        (r#""true""#, ConditionDef::Always),
        (r#""""#, ConditionDef::Always),
        (r#""unknown_condition""#, ConditionDef::Always),
    ];
    for (json, expected) in cases {
        let cond: ConditionDef = serde_json::from_str(json).unwrap();
        assert_eq!(cond, expected, "failed for {}", json);
    }
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
fn action_parse_unknown_variant_errors() {
    let result = serde_json::from_str::<ActionDef>(r#""unknown_action""#);
    assert!(result.is_err());
}

#[test]
fn action_parse_object_unknown_field_errors() {
    let result = serde_json::from_str::<ActionDef>(r#"{"start_at_label": "X", "extra_key": 1}"#);
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
fn screen_defs_error_display() {
    let not_found = ScreenDefsError::NotFound("ui/screens.json 不存在".into());
    assert!(not_found.to_string().contains("界面配置加载失败"));
    assert!(not_found.to_string().contains("ui/screens.json"));

    let parse_failed = ScreenDefsError::ParseFailed("unexpected token".into());
    assert!(parse_failed.to_string().contains("界面配置解析失败"));
    assert!(parse_failed.to_string().contains("unexpected token"));
}

#[test]
fn screen_definitions_load_not_found() {
    let manager = ResourceManager::new("__nonexistent_screen_defs_test_path", 0);
    let result = ScreenDefinitions::load(&manager);
    let err = result.expect_err("load should fail when file missing");
    match &err {
        ScreenDefsError::NotFound(msg) => assert!(msg.contains("ui/screens.json")),
        ScreenDefsError::ParseFailed(_) => panic!("expected NotFound, got ParseFailed"),
    }
}

#[test]
fn conditional_asset_resolve() {
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
        Some("main_summer"),
        "no var set -> fallback to when=None"
    );

    let mut store_match = PersistentStore::empty();
    store_match
        .variables
        .insert("complete_summer".into(), VarValue::Bool(true));
    let ctx_match = ConditionContext {
        has_continue: false,
        persistent: &store_match,
    };
    assert_eq!(
        ConditionalAsset::resolve(&assets, &ctx_match),
        Some("main_winter"),
        "var truthy -> first matching asset"
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
fn condition_evaluate_always() {
    let store = PersistentStore::empty();
    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert!(ConditionDef::Always.evaluate(&ctx));
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

    // 键存在但为 Bool(false)，is_var_truthy 应返回 false
    store
        .variables
        .insert("off_flag".into(), VarValue::Bool(false));
    let ctx2 = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert!(!ConditionDef::PersistentVar("off_flag".into()).evaluate(&ctx2));
    assert!(ConditionDef::NotPersistentVar("off_flag".into()).evaluate(&ctx2));
}
