use super::*;
use crate::resources::ResourceManager;

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
