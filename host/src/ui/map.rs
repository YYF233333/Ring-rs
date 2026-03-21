//! # 地图数据模型
//!
//! 定义地图定义格式，从 JSON 文件加载。
//! 地图文件约定路径：`assets/maps/{map_id}.json`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 地图定义
///
/// 从 `assets/maps/{map_id}.json` 加载。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDefinition {
    /// 地图标题
    pub title: String,
    /// 可选的背景图片路径（相对于 assets）
    #[serde(default)]
    pub background: Option<String>,
    /// 地图上的位置列表
    pub locations: Vec<MapLocation>,
}

/// 地图上的一个位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLocation {
    /// 位置 ID（回传脚本的值）
    pub id: String,
    /// 显示名称
    pub label: String,
    /// 位置坐标（基准 1920x1080，百分比）
    pub x: f32,
    pub y: f32,
    /// 是否默认可用（可被脚本变量覆盖）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 可用性条件（变量名，值为 true 时可用）
    #[serde(default)]
    pub condition: Option<String>,
}

fn default_true() -> bool {
    true
}

/// 地图显示状态（Host 持有）
#[derive(Debug, Clone)]
pub struct MapDisplayState {
    /// 地图定义
    pub definition: MapDefinition,
    /// 请求的 key（用于回传 UIResult）
    pub request_key: String,
    /// 当前悬停的位置索引
    pub hovered_index: Option<usize>,
    /// 位置可用性缓存（按条件求值结果）
    pub availability: Vec<bool>,
}

impl MapDisplayState {
    pub fn new(definition: MapDefinition, request_key: String) -> Self {
        let availability = definition.locations.iter().map(|loc| loc.enabled).collect();
        Self {
            definition,
            request_key,
            hovered_index: None,
            availability,
        }
    }

    /// 使用变量状态更新位置可用性
    pub fn update_availability(&mut self, variables: &HashMap<String, bool>) {
        for (i, loc) in self.definition.locations.iter().enumerate() {
            self.availability[i] = if let Some(ref cond) = loc.condition {
                variables.get(cond).copied().unwrap_or(loc.enabled)
            } else {
                loc.enabled
            };
        }
    }
}
