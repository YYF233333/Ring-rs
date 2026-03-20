use super::*;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.window.width, 1920);
    assert_eq!(config.window.height, 1080);
}
