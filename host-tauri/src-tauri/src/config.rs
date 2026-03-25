//! 运行时配置管理
//!
//! 配置文件缺失或字段缺失时使用 Default 回退。

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
pub struct AppConfig {
    /// 游戏名称
    pub name: Option<String>,
    /// 资源根目录
    pub assets_root: PathBuf,
    /// 存档目录
    pub saves_dir: PathBuf,
    /// manifest.json 路径（相对于 assets_root）
    pub manifest_path: String,
    /// 入口脚本路径（相对于 assets_root）
    pub start_script_path: String,
    /// 资源来源类型
    pub asset_source: AssetSourceType,
    /// 窗口配置
    pub window: WindowConfig,
    /// 调试配置
    pub debug: DebugConfig,
    /// 音频配置
    pub audio: AudioConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fullscreen: bool,
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DebugConfig {
    pub log_level: Option<String>,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: None,
            assets_root: PathBuf::from("assets"),
            saves_dir: PathBuf::from("saves"),
            manifest_path: "manifest.json".to_string(),
            start_script_path: String::new(),
            asset_source: AssetSourceType::default(),
            window: WindowConfig::default(),
            debug: DebugConfig::default(),
            audio: AudioConfig::default(),
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

}

/// 配置错误
#[derive(Debug, Clone)]
pub enum ConfigError {
    LoadFailed(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::LoadFailed(e) => write!(f, "配置加载失败: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
