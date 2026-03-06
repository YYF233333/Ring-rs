//! # UI 皮肤配置

use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UiSkinConfig {
    #[serde(default)]
    pub icons: IconMapping,
    #[serde(default)]
    pub button: ButtonSkin,
    #[serde(default)]
    pub panel: PanelSkin,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IconMapping {
    pub info: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ButtonSkin {
    pub normal: Option<String>,
    pub hover: Option<String>,
    pub pressed: Option<String>,
    pub disabled: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PanelSkin {
    pub background: Option<String>,
    pub border: Option<String>,
}

pub fn load_skin(path: &Path) -> Option<UiSkinConfig> {
    if !path.exists() {
        warn!(path = ?path, "UI 皮肤文件不存在，将使用默认绘制");
        return None;
    }

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            warn!(path = ?path, error = %e, "读取 UI 皮肤文件失败，将使用默认绘制");
            return None;
        }
    };

    match serde_json::from_str::<UiSkinConfig>(&content) {
        Ok(cfg) => {
            info!(path = ?path, "UI 皮肤配置加载成功");
            Some(cfg)
        }
        Err(e) => {
            warn!(path = ?path, error = %e, "解析 UI 皮肤文件失败，将使用默认绘制");
            None
        }
    }
}

