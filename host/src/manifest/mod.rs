//! # Manifest 模块
//!
//! 立绘资源元数据管理，控制角色摆放与缩放。
//!
//! ## 核心概念
//!
//! - **Group**: 立绘组，一组视觉上一致的立绘（如同一角色的不同表情/服装）
//! - **Anchor**: 锚点，立绘的对齐基准点（归一化坐标，0.0-1.0）
//! - **PreScale**: 预处理缩放，载入时应用，使不同尺寸立绘归一化
//! - **Preset**: 站位预设，定义屏幕位置和额外缩放

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::warn;

/// 2D 点（归一化坐标）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Default for Point2D {
    fn default() -> Self {
        Self { x: 0.5, y: 1.0 }
    }
}

/// 立绘组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    /// 锚点（归一化坐标，0.0-1.0）
    /// x: 0.0=左边 0.5=中心 1.0=右边
    /// y: 0.0=顶部 0.5=中心 1.0=底部
    pub anchor: Point2D,
    /// 预处理缩放（载入时应用）
    pub pre_scale: f32,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            anchor: Point2D { x: 0.5, y: 1.0 },
            pre_scale: 1.0,
        }
    }
}

/// 站位预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPreset {
    /// 屏幕 X 坐标（归一化，0.0-1.0）
    pub x: f32,
    /// 屏幕 Y 坐标（归一化，0.0-1.0）
    pub y: f32,
    /// 额外缩放
    pub scale: f32,
}

impl Default for PositionPreset {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.95,
            scale: 1.0,
        }
    }
}

/// 角色配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharactersConfig {
    /// 立绘组定义（group_id -> GroupConfig）
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
    /// 立绘路径到组的映射（sprite_path -> group_id）
    #[serde(default)]
    pub sprites: HashMap<String, String>,
}

/// 默认配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub anchor: Point2D,
    pub pre_scale: f32,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            anchor: Point2D { x: 0.5, y: 1.0 },
            pre_scale: 1.0,
        }
    }
}

/// Manifest 校验警告
#[derive(Debug, Clone)]
pub enum ManifestWarning {
    /// 锚点值超出范围 (0.0 - 1.0)
    InvalidAnchor { context: String, x: f32, y: f32 },
    /// 预缩放值无效 (必须 > 0)
    InvalidPreScale { context: String, value: f32 },
    /// 预设位置超出范围
    InvalidPresetPosition { context: String, value: f32 },
    /// 引用了不存在的组
    UnknownGroup {
        sprite_path: String,
        group_id: String,
    },
}

impl std::fmt::Display for ManifestWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestWarning::InvalidAnchor { context, x, y } => {
                write!(f, "{}: 锚点 ({}, {}) 超出范围 [0.0, 1.0]", context, x, y)
            }
            ManifestWarning::InvalidPreScale { context, value } => {
                write!(f, "{}: 预缩放 {} 必须 > 0", context, value)
            }
            ManifestWarning::InvalidPresetPosition { context, value } => {
                write!(f, "{}: 位置 {} 超出范围 [0.0, 1.0]", context, value)
            }
            ManifestWarning::UnknownGroup {
                sprite_path,
                group_id,
            } => {
                write!(
                    f,
                    "sprite '{}' 引用了不存在的组 '{}'",
                    sprite_path, group_id
                )
            }
        }
    }
}

/// 资源清单
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// 角色配置
    #[serde(default)]
    pub characters: CharactersConfig,
    /// 站位预设
    #[serde(default)]
    pub presets: HashMap<String, PositionPreset>,
    /// 默认配置
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl Manifest {
    /// 从文件加载 Manifest（文件系统模式）
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("无法读取 manifest 文件: {} - {}", path, e))?;

        serde_json::from_str(&content).map_err(|e| format!("无法解析 manifest JSON: {}", e))
    }

    /// 从字节数据加载 Manifest（ZIP 模式）
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("无法将字节转换为 UTF-8 字符串: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("无法解析 manifest JSON: {}", e))
    }

    /// 创建带默认预设的空 Manifest
    pub fn with_defaults() -> Self {
        let mut presets = HashMap::new();

        // 默认的九宫格站位
        presets.insert(
            "left".to_string(),
            PositionPreset {
                x: 0.15,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "nearleft".to_string(),
            PositionPreset {
                x: 0.25,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "farleft".to_string(),
            PositionPreset {
                x: 0.08,
                y: 0.90,
                scale: 0.85,
            },
        );
        presets.insert(
            "center".to_string(),
            PositionPreset {
                x: 0.50,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "nearmiddle".to_string(),
            PositionPreset {
                x: 0.40,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "farmiddle".to_string(),
            PositionPreset {
                x: 0.50,
                y: 0.90,
                scale: 0.85,
            },
        );
        presets.insert(
            "right".to_string(),
            PositionPreset {
                x: 0.85,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "nearright".to_string(),
            PositionPreset {
                x: 0.75,
                y: 0.95,
                scale: 1.0,
            },
        );
        presets.insert(
            "farright".to_string(),
            PositionPreset {
                x: 0.92,
                y: 0.90,
                scale: 0.85,
            },
        );

        Self {
            characters: CharactersConfig::default(),
            presets,
            defaults: DefaultsConfig::default(),
        }
    }

    /// 验证 Manifest 配置有效性
    ///
    /// 返回所有验证警告/错误
    pub fn validate(&self) -> Vec<ManifestWarning> {
        let mut warnings = Vec::new();

        // 验证锚点范围 (0.0 - 1.0)
        let validate_point = |p: &Point2D, context: &str| -> Option<ManifestWarning> {
            if p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0 {
                Some(ManifestWarning::InvalidAnchor {
                    context: context.to_string(),
                    x: p.x,
                    y: p.y,
                })
            } else {
                None
            }
        };

        // 验证默认锚点
        if let Some(w) = validate_point(&self.defaults.anchor, "defaults.anchor") {
            warnings.push(w);
        }

        // 验证默认预缩放
        if self.defaults.pre_scale <= 0.0 {
            warnings.push(ManifestWarning::InvalidPreScale {
                context: "defaults.pre_scale".to_string(),
                value: self.defaults.pre_scale,
            });
        }

        // 验证组配置
        for (group_id, config) in &self.characters.groups {
            let ctx = format!("characters.groups.{}", group_id);

            if let Some(w) = validate_point(&config.anchor, &format!("{}.anchor", ctx)) {
                warnings.push(w);
            }

            if config.pre_scale <= 0.0 {
                warnings.push(ManifestWarning::InvalidPreScale {
                    context: format!("{}.pre_scale", ctx),
                    value: config.pre_scale,
                });
            }
        }

        // 验证预设
        for (preset_name, preset) in &self.presets {
            let ctx = format!("presets.{}", preset_name);

            if preset.x < 0.0 || preset.x > 1.0 {
                warnings.push(ManifestWarning::InvalidPresetPosition {
                    context: format!("{}.x", ctx),
                    value: preset.x,
                });
            }

            if preset.y < 0.0 || preset.y > 1.0 {
                warnings.push(ManifestWarning::InvalidPresetPosition {
                    context: format!("{}.y", ctx),
                    value: preset.y,
                });
            }

            if preset.scale <= 0.0 {
                warnings.push(ManifestWarning::InvalidPreScale {
                    context: format!("{}.scale", ctx),
                    value: preset.scale,
                });
            }
        }

        // 验证 sprite 映射引用的组是否存在
        for (sprite_path, group_id) in &self.characters.sprites {
            if !self.characters.groups.contains_key(group_id) {
                warnings.push(ManifestWarning::UnknownGroup {
                    sprite_path: sprite_path.clone(),
                    group_id: group_id.clone(),
                });
            }
        }

        warnings
    }

    /// 加载并验证 Manifest，打印警告
    pub fn load_and_validate(path: &str) -> Result<Self, String> {
        let manifest = Self::load(path)?;

        let warnings = manifest.validate();
        for warning in &warnings {
            warn!(warning = %warning, "Manifest 警告");
        }

        Ok(manifest)
    }

    /// 获取立绘的组配置
    ///
    /// 查找顺序：
    /// 1. sprites 显式映射
    /// 2. 路径目录名推导
    /// 3. 文件名前缀推导
    /// 4. 返回默认配置
    pub fn get_group_config(&self, sprite_path: &str) -> GroupConfig {
        // 1. 显式映射
        if let Some(group_id) = self.characters.sprites.get(sprite_path) {
            if let Some(config) = self.characters.groups.get(group_id) {
                return config.clone();
            }
        }

        // 2. 路径推导
        let group_id = self.infer_group_id(sprite_path);
        if let Some(config) = self.characters.groups.get(&group_id) {
            return config.clone();
        }

        // 3. 返回默认
        GroupConfig {
            anchor: self.defaults.anchor,
            pre_scale: self.defaults.pre_scale,
        }
    }

    /// 从路径推导 group_id
    ///
    /// 规则：
    /// 1. 取父目录名（如果不是 "characters"）
    /// 2. 否则取文件名前缀（`-` 或 `_` 之前的部分）
    fn infer_group_id(&self, sprite_path: &str) -> String {
        let path = Path::new(sprite_path);

        // 尝试从父目录推导
        if let Some(parent) = path.parent() {
            if let Some(parent_name) = parent.file_name() {
                let parent_str = parent_name.to_string_lossy();
                // 如果父目录不是通用目录名，使用它
                if !matches!(
                    parent_str.as_ref(),
                    "characters" | "sprites" | "images" | "assets"
                ) {
                    return parent_str.to_string();
                }
            }
        }

        // 从文件名推导
        if let Some(stem) = path.file_stem() {
            let stem_str = stem.to_string_lossy();
            // 查找分隔符
            for sep in ['-', '_', ' '] {
                if let Some(pos) = stem_str.find(sep) {
                    return stem_str[..pos].to_string();
                }
            }
            // 无分隔符，返回完整 stem
            return stem_str.to_string();
        }

        // 兜底
        "default".to_string()
    }

    /// 获取站位预设
    pub fn get_preset(&self, position_name: &str) -> PositionPreset {
        self.presets.get(position_name).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_group_id() {
        let manifest = Manifest::with_defaults();

        // 从文件名推导
        assert_eq!(
            manifest.infer_group_id("characters/北风-日常服.png"),
            "北风"
        );
        assert_eq!(manifest.infer_group_id("characters/路汐_笑颜.png"), "路汐");
        assert_eq!(manifest.infer_group_id("characters/测试.png"), "测试");

        // 从目录推导
        assert_eq!(manifest.infer_group_id("北风/日常服.png"), "北风");
    }

    #[test]
    fn test_get_group_config_explicit() {
        let mut manifest = Manifest::with_defaults();

        // 添加显式配置
        manifest.characters.groups.insert(
            "北风".to_string(),
            GroupConfig {
                anchor: Point2D { x: 0.5, y: 0.9 },
                pre_scale: 0.8,
            },
        );
        manifest
            .characters
            .sprites
            .insert("characters/北风-日常服.png".to_string(), "北风".to_string());

        let config = manifest.get_group_config("characters/北风-日常服.png");
        assert!((config.anchor.y - 0.9).abs() < 0.01);
        assert!((config.pre_scale - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_get_group_config_inferred() {
        let mut manifest = Manifest::with_defaults();

        // 只添加组配置，不添加显式映射
        manifest.characters.groups.insert(
            "北风".to_string(),
            GroupConfig {
                anchor: Point2D { x: 0.5, y: 0.85 },
                pre_scale: 0.75,
            },
        );

        // 通过文件名推导应该能找到
        let config = manifest.get_group_config("characters/北风-惊讶.png");
        assert!((config.pre_scale - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_get_group_config_default() {
        let manifest = Manifest::with_defaults();

        // 未配置的立绘应返回默认值
        let config = manifest.get_group_config("characters/未知角色.png");
        assert!((config.anchor.x - 0.5).abs() < 0.01);
        assert!((config.anchor.y - 1.0).abs() < 0.01);
        assert!((config.pre_scale - 1.0).abs() < 0.01);
    }
}
