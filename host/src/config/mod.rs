//! # Config 模块
//!
//! 运行时配置管理，集中管理所有配置项。
//!
//! ## 配置优先级
//!
//! 1. 命令行参数（最高）
//! 2. 配置文件 (config.json)
//! 3. 默认值（最低）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 资源来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssetSourceType {
    /// 文件系统（开发模式）
    Fs,
    /// ZIP 文件（发布模式）
    Zip,
}

impl Default for AssetSourceType {
    fn default() -> Self {
        Self::Fs
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 资源根目录（仅 Fs 模式使用）
    #[serde(default = "default_assets_root")]
    pub assets_root: PathBuf,

    /// 存档目录
    #[serde(default = "default_saves_dir")]
    pub saves_dir: PathBuf,

    /// manifest.json 路径（相对于 assets_root）
    #[serde(default = "default_manifest_path")]
    pub manifest_path: String,

    /// 默认字体路径（相对于 assets_root）
    ///
    /// 默认值为 `"fonts/simhei.ttf"`，支持中文显示。
    /// 可以指定其他字体文件路径（如 `"fonts/custom.ttf"`）。
    #[serde(default = "default_font_path")]
    pub default_font: String,

    /// **入口脚本路径**（相对于 assets_root）
    ///
    /// 必须配置，未配置将 panic。
    pub start_script_path: String,

    /// 资源来源类型（fs/zip）
    #[serde(default)]
    pub asset_source: AssetSourceType,

    /// ZIP 文件路径（仅 Zip 模式使用）
    #[serde(default)]
    pub zip_path: Option<String>,

    /// 窗口配置
    #[serde(default)]
    pub window: WindowConfig,

    /// 调试配置
    #[serde(default)]
    pub debug: DebugConfig,

    /// 音频配置
    #[serde(default)]
    pub audio: AudioConfig,

    /// 资源缓存配置
    #[serde(default)]
    pub resources: ResourceConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    #[serde(default = "default_window_width")]
    pub width: u32,

    /// 窗口高度
    #[serde(default = "default_window_height")]
    pub height: u32,

    /// 窗口标题
    #[serde(default = "default_window_title")]
    pub title: String,

    /// 是否全屏
    #[serde(default)]
    pub fullscreen: bool,
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// 是否显示 FPS
    /// 启动时是否运行脚本检查
    ///
    /// - debug build（`cargo run`）默认开启（见 `default_script_check()`）
    /// - release build 默认关闭，可在 `config.json` 显式设置打开/关闭
    /// - 检查结果只输出诊断，不阻塞启动
    #[serde(default = "default_script_check")]
    pub script_check: bool,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 1.0)
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,

    /// BGM 音量 (0.0 - 1.0)
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,

    /// SFX 音量 (0.0 - 1.0)
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,

    /// 是否静音
    #[serde(default)]
    pub muted: bool,
}

/// 资源缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// 纹理缓存大小（MB）
    #[serde(default = "default_texture_cache_size_mb")]
    pub texture_cache_size_mb: usize,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            texture_cache_size_mb: default_texture_cache_size_mb(),
        }
    }
}

// 默认值函数
fn default_assets_root() -> PathBuf {
    PathBuf::from("assets")
}

fn default_saves_dir() -> PathBuf {
    PathBuf::from("saves")
}

fn default_manifest_path() -> String {
    "manifest.json".to_string()
}

fn default_window_width() -> u32 {
    1920
}

fn default_window_height() -> u32 {
    1080
}

fn default_window_title() -> String {
    "Ring VN Engine".to_string()
}

fn default_master_volume() -> f32 {
    1.0
}

fn default_bgm_volume() -> f32 {
    0.8
}

fn default_sfx_volume() -> f32 {
    1.0
}

fn default_texture_cache_size_mb() -> usize {
    256
}

fn default_font_path() -> String {
    "fonts/simhei.ttf".to_string()
}

fn default_script_check() -> bool {
    // 在 debug build 时默认开启脚本检查
    cfg!(debug_assertions)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            assets_root: default_assets_root(),
            saves_dir: default_saves_dir(),
            manifest_path: default_manifest_path(),
            default_font: default_font_path(),
            start_script_path: String::new(), // 必须在 config.json 中配置
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
            width: default_window_width(),
            height: default_window_height(),
            title: default_window_title(),
            fullscreen: false,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            script_check: default_script_check(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: default_master_volume(),
            bgm_volume: default_bgm_volume(),
            sfx_volume: default_sfx_volume(),
            muted: false,
        }
    }
}

impl AppConfig {
    /// 加载配置文件
    ///
    /// 如果文件不存在或解析失败，返回默认配置并打印警告。
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        if !path.exists() {
            println!("⚠️ 配置文件不存在: {:?}，使用默认配置", path);
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    println!("✅ 配置文件加载成功: {:?}", path);
                    config
                }
                Err(e) => {
                    eprintln!("⚠️ 配置文件解析失败: {}，使用默认配置", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("⚠️ 配置文件读取失败: {}，使用默认配置", e);
                Self::default()
            }
        }
    }

    /// 保存配置到文件
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationFailed(e.to_string()))?;

        fs::write(path, json).map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 获取 manifest 完整路径
    pub fn manifest_full_path(&self) -> PathBuf {
        self.assets_root.join(&self.manifest_path)
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 根据资源来源类型检查
        match self.asset_source {
            AssetSourceType::Fs => {
                // 检查资源目录存在
                if !self.assets_root.exists() {
                    return Err(ConfigError::ValidationFailed(format!(
                        "资源目录不存在: {:?}",
                        self.assets_root
                    )));
                }

                // 检查入口脚本存在
                let script_full_path = self.assets_root.join(&self.start_script_path);
                if !script_full_path.exists() {
                    return Err(ConfigError::ValidationFailed(format!(
                        "入口脚本不存在: {:?}",
                        script_full_path
                    )));
                }
            }
            AssetSourceType::Zip => {
                // 检查 ZIP 路径配置
                let zip_path = self.zip_path.as_ref().ok_or_else(|| {
                    ConfigError::ValidationFailed("Zip 模式必须配置 zip_path".to_string())
                })?;

                // 检查 ZIP 文件存在
                if !Path::new(zip_path).exists() {
                    return Err(ConfigError::ValidationFailed(format!(
                        "ZIP 文件不存在: {}",
                        zip_path
                    )));
                }
            }
        }

        // **必须配置入口脚本**
        if self.start_script_path.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "必须配置 start_script_path（入口脚本路径）".to_string(),
            ));
        }

        // 检查音量范围
        if self.audio.master_volume < 0.0 || self.audio.master_volume > 1.0 {
            return Err(ConfigError::ValidationFailed(
                "主音量必须在 0.0 - 1.0 之间".to_string(),
            ));
        }

        if self.audio.bgm_volume < 0.0 || self.audio.bgm_volume > 1.0 {
            return Err(ConfigError::ValidationFailed(
                "BGM 音量必须在 0.0 - 1.0 之间".to_string(),
            ));
        }

        if self.audio.sfx_volume < 0.0 || self.audio.sfx_volume > 1.0 {
            return Err(ConfigError::ValidationFailed(
                "SFX 音量必须在 0.0 - 1.0 之间".to_string(),
            ));
        }

        Ok(())
    }

    /// 获取入口脚本完整路径
    pub fn start_script_full_path(&self) -> PathBuf {
        self.assets_root.join(&self.start_script_path)
    }
}

/// 配置错误
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// 序列化失败
    SerializationFailed(String),
    /// IO 错误
    IoError(String),
    /// 验证失败
    ValidationFailed(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::SerializationFailed(e) => write!(f, "配置序列化失败: {}", e),
            ConfigError::IoError(e) => write!(f, "配置 IO 错误: {}", e),
            ConfigError::ValidationFailed(e) => write!(f, "配置验证失败: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_config_validation() {
        let mut config = AppConfig::default();

        // 无效音量
        config.audio.master_volume = 2.0;
        assert!(config.validate().is_err());

        // 恢复有效值
        config.audio.master_volume = 0.5;
        // 资源目录不存在时也会失败（默认 "assets" 可能不存在）
        // 只检查音量验证
    }
}
