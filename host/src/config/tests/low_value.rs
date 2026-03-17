use super::*;
use std::path::PathBuf;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.window.width, 1920);
    assert_eq!(config.window.height, 1080);
}

#[test]
fn test_config_serialization() {
    let config = AppConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();

    let loaded: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.window.width, config.window.width);
}

#[test]
fn test_manifest_full_path() {
    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: PathBuf::from("a"),
        manifest_path: "b/manifest.json".to_string(),
        ..Default::default()
    };
    assert_eq!(
        config.manifest_full_path(),
        PathBuf::from("a").join("b/manifest.json")
    );
}

#[test]
fn test_start_script_full_path() {
    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: PathBuf::from("assets_root"),
        start_script_path: "scripts/main.md".to_string(),
        ..Default::default()
    };
    assert_eq!(
        config.start_script_full_path(),
        PathBuf::from("assets_root").join("scripts/main.md")
    );
}

#[test]
fn test_config_error_display() {
    let cases = [
        (ConfigError::LoadFailed("x".to_string()), "配置加载失败"),
        (
            ConfigError::SerializationFailed("x".to_string()),
            "配置序列化失败",
        ),
        (ConfigError::IoError("y".to_string()), "配置 IO 错误"),
        (
            ConfigError::ValidationFailed("z".to_string()),
            "配置验证失败",
        ),
    ];

    for (err, expected) in cases {
        assert!(err.to_string().contains(expected));
    }
}
