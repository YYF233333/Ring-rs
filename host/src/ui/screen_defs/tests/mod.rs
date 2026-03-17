mod high_value;
mod low_value;

use super::*;

// undecided: schema/serde 偏灰
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
