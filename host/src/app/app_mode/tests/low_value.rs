use super::*;

// ============ 枚举 default / getter / 辅助 ============

#[test]
fn test_user_settings_default() {
    let settings = UserSettings::default();
    assert!((settings.bgm_volume - 0.8).abs() < 0.001);
    assert!(!settings.muted);
}
