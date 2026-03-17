use super::*;
use std::str::FromStr;

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
fn test_position_parse_supports_all_variants() {
    let cases = [
        ("right", Position::Right),
        ("NearRight", Position::NearRight),
        ("nearmiddle", Position::NearMiddle),
        ("farleft", Position::FarLeft),
        ("FARRIGHT", Position::FarRight),
        ("farmiddle", Position::FarMiddle),
    ];

    for (raw, expected) in cases {
        assert_eq!(Position::parse(raw), Some(expected), "raw = {raw}");
    }

    assert_eq!(Position::parse("not-a-position"), None);
}

#[test]
fn test_transition_get_duration_and_reversed_wrong_type_returns_none() {
    let t = Transition::with_named_args(
        "Any",
        vec![
            (
                Some("duration".to_string()),
                TransitionArg::String("not-a-number".to_string()),
            ),
            (Some("reversed".to_string()), TransitionArg::Number(1.0)),
        ],
    );

    assert_eq!(t.get_duration(), None);
    assert_eq!(t.get_reversed(), None);
}

#[test]
fn test_transition_get_arg_fallback() {
    let t = Transition::with_named_args(
        "Dissolve",
        vec![(Some("duration".to_string()), TransitionArg::Number(2.0))],
    );
    assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(2.0)));

    let t = Transition::with_args("Dissolve", vec![TransitionArg::Number(1.5)]);
    assert_eq!(t.get_arg("duration", 0), Some(&TransitionArg::Number(1.5)));
}

#[test]
fn test_transition_get_duration_and_reversed() {
    let t = Transition::with_named_args(
        "Fade",
        vec![
            (Some("duration".to_string()), TransitionArg::Number(2.5)),
            (Some("reversed".to_string()), TransitionArg::Bool(true)),
        ],
    );
    assert_eq!(t.get_duration(), Some(2.5));
    assert_eq!(t.get_reversed(), Some(true));

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
