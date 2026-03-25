//! 资源清单（Manifest）
//!
//! 立绘元数据管理：角色组配置、锚点、预缩放、站位预设。

use crate::resources::normalize_logical_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    pub anchor: Point2D,
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
    pub x: f32,
    pub y: f32,
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
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
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

/// 资源清单
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub characters: CharactersConfig,
    #[serde(default)]
    pub presets: HashMap<String, PositionPreset>,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl Manifest {
    /// 从文件加载
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取 manifest 文件: {} - {}", path, e))?;
        serde_json::from_str(&content).map_err(|e| format!("无法解析 manifest JSON: {}", e))
    }

    /// 从字节数据加载
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("无法将字节转换为 UTF-8 字符串: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("无法解析 manifest JSON: {}", e))
    }

    /// 创建带默认预设的空 Manifest
    pub fn with_defaults() -> Self {
        let mut presets = HashMap::new();
        for (name, x, y, scale) in [
            ("left", 0.15, 0.95, 1.0),
            ("nearleft", 0.25, 0.95, 1.0),
            ("farleft", 0.08, 0.90, 0.85),
            ("center", 0.50, 0.95, 1.0),
            ("nearmiddle", 0.40, 0.95, 1.0),
            ("farmiddle", 0.50, 0.90, 0.85),
            ("right", 0.85, 0.95, 1.0),
            ("nearright", 0.75, 0.95, 1.0),
            ("farright", 0.92, 0.90, 0.85),
        ] {
            presets.insert(name.to_string(), PositionPreset { x, y, scale });
        }

        Self {
            characters: CharactersConfig::default(),
            presets,
            defaults: DefaultsConfig::default(),
        }
    }

    /// 获取立绘的组配置
    ///
    /// 查找顺序：显式映射 → 路径推导 → 默认配置
    pub fn get_group_config(&self, sprite_path: &str) -> GroupConfig {
        let normalized_path = normalize_logical_path(sprite_path);

        if let Some(group_id) = self
            .characters
            .sprites
            .get(sprite_path)
            .or_else(|| self.characters.sprites.get(&normalized_path))
            && let Some(config) = self.characters.groups.get(group_id)
        {
            return config.clone();
        }

        let group_id = self.infer_group_id(&normalized_path);
        if let Some(config) = self.characters.groups.get(&group_id) {
            return config.clone();
        }

        GroupConfig {
            anchor: self.defaults.anchor,
            pre_scale: self.defaults.pre_scale,
        }
    }

    fn infer_group_id(&self, sprite_path: &str) -> String {
        let path = Path::new(sprite_path);

        if let Some(parent) = path.parent()
            && let Some(parent_name) = parent.file_name()
        {
            let parent_str = parent_name.to_string_lossy();
            if !matches!(
                parent_str.as_ref(),
                "characters" | "sprites" | "images" | "assets"
            ) {
                return parent_str.to_string();
            }
        }

        if let Some(stem) = path.file_stem() {
            let stem_str = stem.to_string_lossy();
            for sep in ['-', '_', ' '] {
                if let Some(pos) = stem_str.find(sep) {
                    return stem_str[..pos].to_string();
                }
            }
            return stem_str.to_string();
        }

        "default".to_string()
    }

    /// 获取站位预设
    pub fn get_preset(&self, position_name: &str) -> PositionPreset {
        self.presets.get(position_name).cloned().unwrap_or_default()
    }
}
