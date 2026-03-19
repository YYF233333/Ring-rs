//! # Config 模块
//!
//! 运行时配置管理，集中管理所有配置项。
//!
//! 所有字段均为必填（`Option` 字段需显式写 `null`），
//! 配置文件缺失或字段缺失时直接报错。
//! 默认值存放在外部 `config.json` 文件中，代码不提供运行时回退。

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
    /// 游戏名称（用于打包时命名可执行文件）
    pub name: Option<String>,

    /// 资源根目录（仅 Fs 模式使用）
    pub assets_root: PathBuf,

    /// 存档目录
    pub saves_dir: PathBuf,

    /// manifest.json 路径（相对于 assets_root）
    pub manifest_path: String,

    /// 默认字体路径（相对于 assets_root）
    pub default_font: String,

    /// 入口脚本路径（相对于 assets_root）
    pub start_script_path: String,

    /// 资源来源类型（fs/zip）
    pub asset_source: AssetSourceType,

    /// ZIP 文件路径（仅 Zip 模式使用，Fs 模式写 null）
    pub zip_path: Option<String>,

    /// 窗口配置
    pub window: WindowConfig,

    /// 调试配置
    pub debug: DebugConfig,

    /// 音频配置
    pub audio: AudioConfig,

    /// 资源缓存配置
    pub resources: ResourceConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: u32,

    /// 窗口高度
    pub height: u32,

    /// 窗口标题
    pub title: String,

    /// 是否全屏
    pub fullscreen: bool,
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    /// 启动时是否运行脚本检查
    pub script_check: bool,

    /// 日志等级（null 时使用 info）
    pub log_level: Option<String>,

    /// 日志输出文件路径（null 时输出到控制台）
    pub log_file: Option<String>,

    /// 后台录制缓冲区大小上限（MB），0 则禁用录制
    pub recording_buffer_size_mb: u32,

    /// 录制导出目录
    pub recording_output_dir: String,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 1.0)
    pub master_volume: f32,

    /// BGM 音量 (0.0 - 1.0)
    pub bgm_volume: f32,

    /// SFX 音量 (0.0 - 1.0)
    pub sfx_volume: f32,

    /// 是否静音
    pub muted: bool,
}

/// 资源缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceConfig {
    /// 纹理缓存大小（MB）
    pub texture_cache_size_mb: usize,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            texture_cache_size_mb: 256,
        }
    }
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

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            script_check: cfg!(debug_assertions),
            log_level: None,
            log_file: if cfg!(debug_assertions) {
                None
            } else {
                Some("game.log".to_string())
            },
            recording_buffer_size_mb: 8,
            recording_output_dir: "recordings".to_string(),
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
    /// 加载配置文件
    ///
    /// 配置文件必须存在且所有字段完整，否则返回错误。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed(format!("配置文件 {:?} 读取失败: {}", path, e)))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ConfigError::LoadFailed(format!("配置文件 {:?} 解析失败: {}", path, e)))?;

        info!(path = ?path, "配置文件加载成功");
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationFailed(e.to_string()))?;

        fs::write(path, json).map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 获取 manifest 完整路径
    ///
    /// bootstrap-only: 仅在 ResourceManager 初始化之前使用，后续资源加载需通过 ResourceManager。
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
    ///
    /// bootstrap-only: 仅在 ResourceManager 初始化之前使用，后续资源加载需通过 ResourceManager。
    pub fn start_script_full_path(&self) -> PathBuf {
        self.assets_root.join(&self.start_script_path)
    }
}

/// 配置错误
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// 加载失败（文件缺失、读取错误、解析错误）
    LoadFailed(String),
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
            ConfigError::LoadFailed(e) => write!(f, "配置加载失败: {}", e),
            ConfigError::SerializationFailed(e) => write!(f, "配置序列化失败: {}", e),
            ConfigError::IoError(e) => write!(f, "配置 IO 错误: {}", e),
            ConfigError::ValidationFailed(e) => write!(f, "配置验证失败: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests;
