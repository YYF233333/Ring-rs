//! 运行时配置管理
//!
//! 开发期与运行期共用的严格配置契约。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// 资源来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AssetSourceType {
    /// 文件系统（开发模式）
    #[default]
    Fs,
    /// ZIP 文件（发布模式）
    Zip,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub name: Option<String>,
    pub assets_root: PathBuf,
    pub saves_dir: PathBuf,
    pub manifest_path: String,
    pub default_font: String,
    pub start_script_path: String,
    pub asset_source: AssetSourceType,
    pub zip_path: Option<String>,
    pub window: WindowConfig,
    pub debug: DebugConfig,
    pub audio: AudioConfig,
    pub resources: ResourceConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fullscreen: bool,
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub script_check: bool,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
    pub recording_buffer_size_mb: u32,
    pub recording_output_dir: String,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
}

/// 资源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceConfig {
    pub texture_cache_size_mb: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: None,
            assets_root: PathBuf::from("assets"),
            saves_dir: PathBuf::from("saves"),
            manifest_path: "manifest.json".to_string(),
            default_font: "fonts/simhei.ttf".to_string(),
            start_script_path: String::new(),
            asset_source: AssetSourceType::default(),
            zip_path: None,
            window: WindowConfig::default(),
            debug: DebugConfig::default(),
            audio: AudioConfig::default(),
            resources: ResourceConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            title: "Ring VN Engine".to_string(),
            fullscreen: false,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 0.8,
            sfx_volume: 1.0,
            muted: false,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            script_check: true,
            log_level: Some("info".to_string()),
            log_file: None,
            recording_buffer_size_mb: 8,
            recording_output_dir: "recordings".to_string(),
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            texture_cache_size_mb: 256,
        }
    }
}

impl AppConfig {
    /// 加载配置文件，缺失时返回错误
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed(format!("配置文件 {:?} 读取失败: {}", path, e)))?;
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ConfigError::LoadFailed(format!("配置文件 {:?} 解析失败: {}", path, e)))?;
        info!(path = ?path, "配置文件加载成功");
        Ok(config)
    }

    pub fn validate(&self, project_root: &Path) -> Result<(), ConfigError> {
        let assets_root = if self.assets_root.is_relative() {
            project_root.join(&self.assets_root)
        } else {
            self.assets_root.clone()
        };

        match self.asset_source {
            AssetSourceType::Fs => {
                if !assets_root.is_dir() {
                    return Err(ConfigError::ValidationFailed(format!(
                        "资源目录不存在: {:?}",
                        assets_root
                    )));
                }
            }
            AssetSourceType::Zip => {
                let zip_rel = self.zip_path.as_deref().ok_or_else(|| {
                    ConfigError::ValidationFailed(
                        "asset_source=zip 时必须提供 zip_path".to_string(),
                    )
                })?;
                let zip_path = if Path::new(zip_rel).is_relative() {
                    project_root.join(zip_rel)
                } else {
                    PathBuf::from(zip_rel)
                };
                if !zip_path.is_file() {
                    return Err(ConfigError::ValidationFailed(format!(
                        "ZIP 资源文件不存在: {:?}",
                        zip_path
                    )));
                }
            }
        }

        if self.manifest_path.trim().is_empty() {
            return Err(ConfigError::ValidationFailed(
                "manifest_path 不能为空".to_string(),
            ));
        }
        if self.start_script_path.trim().is_empty() {
            return Err(ConfigError::ValidationFailed(
                "start_script_path 不能为空".to_string(),
            ));
        }
        if self.window.width == 0 || self.window.height == 0 {
            return Err(ConfigError::ValidationFailed(
                "window.width / window.height 必须大于 0".to_string(),
            ));
        }
        for (name, value) in [
            ("audio.master_volume", self.audio.master_volume),
            ("audio.bgm_volume", self.audio.bgm_volume),
            ("audio.sfx_volume", self.audio.sfx_volume),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(ConfigError::ValidationFailed(format!(
                    "{name} 必须位于 0.0..=1.0，实际为 {value}"
                )));
            }
        }
        if self.debug.recording_buffer_size_mb == 0 {
            return Err(ConfigError::ValidationFailed(
                "debug.recording_buffer_size_mb 必须大于 0".to_string(),
            ));
        }
        if self.resources.texture_cache_size_mb == 0 {
            return Err(ConfigError::ValidationFailed(
                "resources.texture_cache_size_mb 必须大于 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// 配置错误
#[derive(Debug, Clone)]
pub enum ConfigError {
    LoadFailed(String),
    ValidationFailed(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::LoadFailed(e) => write!(f, "配置加载失败: {}", e),
            ConfigError::ValidationFailed(e) => write!(f, "配置校验失败: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config_json(assets_root: &str) -> String {
        format!(
            r#"{{
  "name": "Ring VN Engine",
  "assets_root": "{assets_root}",
  "saves_dir": "saves",
  "manifest_path": "manifest.json",
  "default_font": "fonts/simhei.ttf",
  "start_script_path": "scripts/main.md",
  "asset_source": "fs",
  "zip_path": null,
  "window": {{
    "width": 1280, "height": 720,
    "title": "Ring VN Engine", "fullscreen": false
  }},
  "debug": {{
    "script_check": true, "log_level": "info", "log_file": null,
    "recording_buffer_size_mb": 8, "recording_output_dir": "recordings"
  }},
  "audio": {{
    "master_volume": 1.0, "bgm_volume": 0.8, "sfx_volume": 1.0, "muted": false
  }},
  "resources": {{ "texture_cache_size_mb": 256 }}
}}"#
        )
    }

    #[test]
    fn load_rejects_unknown_fields() {
        let dir = std::env::temp_dir().join("ring_host_dioxus_config_unknown");
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("config.json");
        let mut json: serde_json::Value =
            serde_json::from_str(&valid_config_json("assets")).unwrap();
        json.as_object_mut()
            .unwrap()
            .insert("unexpected".to_string(), serde_json::Value::Bool(true));
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let error = AppConfig::load(&config_path).unwrap_err().to_string();
        assert!(error.contains("unexpected"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn validate_rejects_empty_start_script_path() {
        let root = std::env::temp_dir().join("ring_host_dioxus_config_validate");
        let assets = root.join("assets");
        std::fs::create_dir_all(&assets).unwrap();

        let mut config = AppConfig::default();
        config.assets_root = assets;
        config.saves_dir = root.join("saves");
        config.start_script_path.clear();

        let error = config.validate(&root).unwrap_err().to_string();
        assert!(error.contains("start_script_path"));
        std::fs::remove_dir_all(&root).ok();
    }
}
