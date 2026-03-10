use super::*;
use std::str::FromStr;

#[test]
fn test_transition_simple() {
    let t = Transition::simple("dissolve");
    assert_eq!(t.name, "dissolve");
    assert!(t.args.is_empty());
}

#[test]
fn test_transition_with_args() {
    let t = Transition::with_args("Dissolve", vec![TransitionArg::Number(1.5)]);
    assert_eq!(t.name, "Dissolve");
    assert_eq!(t.args.len(), 1);
}

#[test]
fn test_position_from_str() {
    assert_eq!(Position::from_str("left").ok(), Some(Position::Left));
    assert_eq!(Position::from_str("LEFT").ok(), Some(Position::Left));
    assert_eq!(Position::from_str("center").ok(), Some(Position::Center));
    assert_eq!(Position::from_str("middle").ok(), Some(Position::Center));
    assert_eq!(
        Position::from_str("nearleft").ok(),
        Some(Position::NearLeft)
    );
    assert_eq!(Position::from_str("unknown").ok(), None);
}

#[test]
fn test_command_serialization() {
    let cmd = Command::ShowText {
        speaker: Some("羽艾".to_string()),
        content: "你好".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };

    let json = serde_json::to_string(&cmd).unwrap();
    let deserialized: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, deserialized);
}

#[test]
fn test_transition_with_named_args() {
    let t = Transition::with_named_args(
        "Dissolve",
        vec![
            (Some("duration".to_string()), TransitionArg::Number(1.5)),
            (Some("reversed".to_string()), TransitionArg::Bool(true)),
        ],
    );
    assert_eq!(t.name, "Dissolve");
    assert_eq!(t.args.len(), 2);
    assert!(t.is_all_named());
    assert!(!t.is_all_positional());
}

#[test]
fn test_transition_get_named() {
    let t = Transition::with_named_args(
        "Fade",
        vec![
            (Some("duration".to_string()), TransitionArg::Number(2.0)),
            (Some("reversed".to_string()), TransitionArg::Bool(false)),
        ],
    );

    // 按 key 获取命名参数
    assert_eq!(t.get_named("duration"), Some(&TransitionArg::Number(2.0)));
    assert_eq!(t.get_named("reversed"), Some(&TransitionArg::Bool(false)));
    assert_eq!(t.get_named("unknown"), None);
}

#[test]
fn test_transition_get_duration_and_reversed_wrong_type_returns_none() {
    let t = Transition::with_named_args(
        "Any",
        vec![
            // duration 不是 Number
            (
                Some("duration".to_string()),
                TransitionArg::String("not-a-number".to_string()),
            ),
            // reversed 不是 Bool
            (Some("reversed".to_string()), TransitionArg::Number(1.0)),
        ],
    );

    assert_eq!(t.get_duration(), None);
    assert_eq!(t.get_reversed(), None);
}

#[test]
fn test_transition_get_positional() {
    let t = Transition::with_args(
        "Effect",
        vec![
            TransitionArg::Number(1.0),
            TransitionArg::String("test".to_string()),
            TransitionArg::Bool(true),
        ],
    );

    // 按索引获取位置参数
    assert_eq!(t.get_positional(0), Some(&TransitionArg::Number(1.0)));
    assert_eq!(
        t.get_positional(1),
        Some(&TransitionArg::String("test".to_string()))
    );
    assert_eq!(t.get_positional(2), Some(&TransitionArg::Bool(true)));
    assert_eq!(t.get_positional(3), None);
    assert!(t.is_all_positional());
}

#[test]
fn test_transition_get_arg_fallback() {
    // 命名参数优先
    let t = Transition::with_named_args(
        "Dissolve",
        vec![(Some("duration".to_string()), TransitionArg::Number(2.0))],
    );
    assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(2.0)));

    // 位置参数回退
    let t = Transition::with_args("Dissolve", vec![TransitionArg::Number(1.5)]);
    assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(1.5)));
}

#[test]
fn test_transition_get_duration_and_reversed() {
    // 命名参数
    let t = Transition::with_named_args(
        "Fade",
        vec![
            (Some("duration".to_string()), TransitionArg::Number(2.5)),
            (Some("reversed".to_string()), TransitionArg::Bool(true)),
        ],
    );
    assert_eq!(t.get_duration(), Some(2.5));
    assert_eq!(t.get_reversed(), Some(true));

    // 位置参数
    let t = Transition::with_args(
        "Fade",
        vec![
            TransitionArg::Number(1.0),
            TransitionArg::String("mask".to_string()),
            TransitionArg::Bool(false),
        ],
    );
    assert_eq!(t.get_duration(), Some(1.0));
    assert_eq!(t.get_reversed(), Some(false));
}

#[test]
fn test_transition_serialization_with_named_args() {
    let t = Transition::with_named_args(
        "Dissolve",
        vec![(Some("duration".to_string()), TransitionArg::Number(1.5))],
    );

    let json = serde_json::to_string(&t).unwrap();
    let deserialized: Transition = serde_json::from_str(&json).unwrap();
    assert_eq!(t, deserialized);
    assert_eq!(deserialized.get_duration(), Some(1.5));
}
