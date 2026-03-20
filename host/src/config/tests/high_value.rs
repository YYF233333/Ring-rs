use super::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_config_validation_invalid_master_volume() {
    let (mut config, _env) = make_valid_fs_config();
    config.audio.master_volume = 2.0;
    assert!(config.validate().is_err());
}

#[test]
fn test_load_missing_file_returns_error() {
    let missing = unique_temp_path("ring-config-test-missing");
    let result = AppConfig::load(&missing);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::LoadFailed(_)));
}

#[test]
fn test_load_parse_error_returns_error() {
    let parse_err_file = TempPath::file("ring-config-test-parse-error.json");
    fs::write(parse_err_file.as_path(), "{ this is not json").unwrap();
    let result = AppConfig::load(parse_err_file.as_path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::LoadFailed(_)));
}

#[test]
fn test_load_missing_field_returns_error() {
    let incomplete_file = TempPath::file("ring-config-test-incomplete.json");
    fs::write(
        incomplete_file.as_path(),
        r#"{"start_script_path":"entry.md"}"#,
    )
    .unwrap();
    let result = AppConfig::load(incomplete_file.as_path());
    assert!(result.is_err());
}

#[test]
fn test_load_unknown_field_returns_error() {
    let bad_file = TempPath::file("ring-config-test-unknown-field.json");
    let mut json = full_config_json("entry.md");
    json = json.replace(
        r#""resources": { "texture_cache_size_mb": 256 }"#,
        r#""resources": { "texture_cache_size_mb": 256 }, "bogus_field": true"#,
    );
    fs::write(bad_file.as_path(), json).unwrap();
    let result = AppConfig::load(bad_file.as_path());
    assert!(result.is_err());
}

#[test]
fn test_load_complete_json_succeeds() {
    let ok_file = TempPath::file("ring-config-test-ok.json");
    fs::write(ok_file.as_path(), full_config_json("entry.md")).unwrap();
    let loaded = AppConfig::load(ok_file.as_path()).unwrap();
    assert_eq!(loaded.start_script_path, "entry.md");
}

#[test]
fn test_save_io_and_ok() {
    let config = AppConfig::default();
    let io_err_dir = TempPath::dir("ring-config-test-save-dir");
    let err = config.save(io_err_dir.as_path()).unwrap_err();
    assert_is_io_error(err);

    let (config, _env) = make_valid_fs_config();
    let save_file = TempPath::file("ring-config-test-save-ok.json");
    config.save(save_file.as_path()).unwrap();
    assert!(save_file.as_path().exists());
}

#[test]
fn test_validate_fs_assets_root_missing() {
    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: unique_temp_path("ring-config-test-no-such-assets"),
        start_script_path: "start.md".to_string(),
        ..Default::default()
    };
    assert_validation_failed_contains(&config, "资源目录不存在");
}

#[test]
fn test_validate_fs_script_missing() {
    let temp_root = TempPath::dir("ring-config-test-assets-script-missing");
    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: temp_root.path.clone(),
        start_script_path: "nope.md".to_string(),
        ..Default::default()
    };
    assert_validation_failed_contains(&config, "入口脚本不存在");
}

#[test]
fn test_validate_fs_empty_start_script_path() {
    let temp_root = TempPath::dir("ring-config-test-assets-empty-entry");
    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: temp_root.path.clone(),
        start_script_path: "".to_string(),
        ..Default::default()
    };
    assert_validation_failed_contains(&config, "start_script_path");
}

#[test]
fn test_validate_fs_volume_out_of_range_branches() {
    let (mut master_config, _env1) = make_valid_fs_config();
    master_config.audio.master_volume = 2.0;
    assert_validation_failed_contains(&master_config, "主音量");

    let (mut bgm_config, _env2) = make_valid_fs_config();
    bgm_config.audio.bgm_volume = -0.1;
    assert_validation_failed_contains(&bgm_config, "BGM 音量");

    let (mut sfx_config, _env3) = make_valid_fs_config();
    sfx_config.audio.sfx_volume = 1.1;
    assert_validation_failed_contains(&sfx_config, "SFX 音量");
}

#[test]
fn test_validate_fs_ok() {
    let (config, _env) = make_valid_fs_config();
    assert!(config.validate().is_ok());
}

#[test]
fn test_validate_fs_accepts_volume_boundaries() {
    let (mut min_config, _env1) = make_valid_fs_config();
    min_config.audio.master_volume = 0.0;
    min_config.audio.bgm_volume = 0.0;
    min_config.audio.sfx_volume = 0.0;
    assert!(min_config.validate().is_ok());

    let (mut max_config, _env2) = make_valid_fs_config();
    max_config.audio.master_volume = 1.0;
    max_config.audio.bgm_volume = 1.0;
    max_config.audio.sfx_volume = 1.0;
    assert!(max_config.validate().is_ok());
}

#[test]
fn test_validate_fs_rejects_negative_and_over_max_per_audio_channel() {
    let (mut master_negative, _env1) = make_valid_fs_config();
    master_negative.audio.master_volume = -0.01;
    assert_validation_failed_contains(&master_negative, "主音量");

    let (mut bgm_over_max, _env2) = make_valid_fs_config();
    bgm_over_max.audio.bgm_volume = 1.01;
    assert_validation_failed_contains(&bgm_over_max, "BGM 音量");

    let (mut sfx_negative, _env3) = make_valid_fs_config();
    sfx_negative.audio.sfx_volume = -0.01;
    assert_validation_failed_contains(&sfx_negative, "SFX 音量");
}

#[test]
fn test_validate_zip_requires_zip_path() {
    let config = AppConfig {
        asset_source: AssetSourceType::Zip,
        zip_path: None,
        start_script_path: "entry.md".to_string(),
        ..Default::default()
    };
    assert_validation_failed_contains(&config, "zip_path");
}

#[test]
fn test_validate_zip_missing_zip_file() {
    let config = AppConfig {
        asset_source: AssetSourceType::Zip,
        zip_path: Some(
            unique_temp_path("ring-config-test-missing.zip")
                .to_string_lossy()
                .into(),
        ),
        start_script_path: "entry.md".to_string(),
        ..Default::default()
    };
    assert_validation_failed_contains(&config, "ZIP 文件不存在");
}

#[test]
fn test_validate_zip_ok() {
    let zip = TempPath::file("ring-config-test-ok.zip");
    fs::write(zip.as_path(), b"").unwrap();
    let config = AppConfig {
        asset_source: AssetSourceType::Zip,
        zip_path: Some(zip.to_string_lossy_owned()),
        start_script_path: "entry.md".to_string(),
        ..Default::default()
    };
    assert!(config.validate().is_ok());
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
