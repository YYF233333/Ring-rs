use super::*;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempPath {
    path: PathBuf,
    is_dir: bool,
}

impl TempPath {
    fn file(prefix: &str) -> Self {
        Self {
            path: unique_temp_path(prefix),
            is_dir: false,
        }
    }

    fn dir(prefix: &str) -> Self {
        let path = unique_temp_path(prefix);
        fs::create_dir_all(&path).unwrap();
        Self { path, is_dir: true }
    }

    fn as_path(&self) -> &Path {
        &self.path
    }

    fn to_string_lossy_owned(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if self.is_dir {
            let _ = fs::remove_dir_all(&self.path);
        } else {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn make_valid_fs_config() -> (AppConfig, TempPath) {
    let root = TempPath::dir("ring-config-test-assets");
    fs::write(root.as_path().join("start.md"), "ok").unwrap();

    let config = AppConfig {
        asset_source: AssetSourceType::Fs,
        assets_root: root.path.clone(),
        start_script_path: "start.md".to_string(),
        ..Default::default()
    };
    (config, root)
}

fn assert_validation_failed_contains(config: &AppConfig, needle: &str) {
    match config.validate().unwrap_err() {
        ConfigError::ValidationFailed(msg) => assert!(msg.contains(needle), "msg={msg}"),
        other => panic!("expected ValidationFailed, got: {other:?}"),
    }
}

fn assert_is_io_error(err: ConfigError) {
    match err {
        ConfigError::IoError(_) => {}
        other => panic!("expected IoError, got: {other:?}"),
    }
}

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

    // 反序列化
    let loaded: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.window.width, config.window.width);
}

#[test]
fn test_config_validation_invalid_master_volume() {
    let (mut config, _env) = make_valid_fs_config();
    config.audio.master_volume = 2.0;
    assert!(config.validate().is_err());
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{}-{}", prefix, nanos))
}

fn minimal_config_json(start_script_path: &str) -> String {
    format!(r#"{{"start_script_path":"{}"}}"#, start_script_path)
}

#[test]
fn test_load_branches() {
    let missing = unique_temp_path("ring-config-test-missing");
    let loaded_missing = AppConfig::load(&missing);
    assert_eq!(
        loaded_missing.window.width,
        AppConfig::default().window.width
    );

    let parse_err_file = TempPath::file("ring-config-test-parse-error.json");
    fs::write(parse_err_file.as_path(), "{ this is not json").unwrap();
    let loaded_parse_err = AppConfig::load(parse_err_file.as_path());
    assert_eq!(
        loaded_parse_err.window.height,
        AppConfig::default().window.height
    );

    let read_err_dir = TempPath::dir("ring-config-test-read-error-dir");
    let loaded_read_err = AppConfig::load(read_err_dir.as_path());
    assert_eq!(
        loaded_read_err.window.title,
        AppConfig::default().window.title
    );

    let ok_file = TempPath::file("ring-config-test-ok.json");
    fs::write(ok_file.as_path(), minimal_config_json("entry.md")).unwrap();
    let loaded = AppConfig::load(ok_file.as_path());
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
fn test_config_error_display() {
    let cases = [
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
