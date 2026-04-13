//! 地图数据模型
//!
//! 定义地图定义格式，从 JSON 文件加载。
//! 地图文件约定路径：`assets/maps/{map_id}.json`

use serde::{Deserialize, Serialize};

/// 地图定义
///
/// 从 `assets/maps/{map_id}.json` 加载。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDefinition {
    /// 地图标题
    pub title: String,
    /// 可选的背景图片路径（相对于 assets）
    #[serde(default)]
    pub background: Option<String>,
    /// 命中检测掩码图路径（同尺寸，每个区域涂唯一纯色）
    #[serde(default)]
    pub hit_mask: Option<String>,
    /// 地图上的位置列表
    pub locations: Vec<MapLocation>,
}

/// 地图上的一个位置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapLocation {
    /// 位置 ID（回传脚本的值）
    pub id: String,
    /// 显示名称
    pub label: String,
    /// 掩码图中此区域的颜色（如 "#FF0000"）
    #[serde(default)]
    pub mask_color: Option<String>,
    /// 位置坐标（基准 1920x1080）
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
