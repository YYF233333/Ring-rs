mod high_value;
mod low_value;

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

fn unique_temp_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{}-{}", prefix, nanos))
}

fn full_config_json(start_script_path: &str) -> String {
    format!(
        r#"{{
  "name": null,
  "assets_root": "assets",
  "saves_dir": "saves",
  "manifest_path": "manifest.json",
  "default_font": "fonts/simhei.ttf",
  "start_script_path": "{}",
  "asset_source": "fs",
  "zip_path": null,
  "window": {{ "width": 1920, "height": 1080, "title": "Ring VN Engine", "fullscreen": false }},
  "debug": {{ "script_check": true, "log_level": null, "log_file": null, "recording_buffer_size_mb": 8, "recording_output_dir": "recordings" }},
  "audio": {{ "master_volume": 1.0, "bgm_volume": 0.8, "sfx_volume": 1.0, "muted": false }},
  "resources": {{ "texture_cache_size_mb": 256 }}
}}"#,
        start_script_path
    )
}
