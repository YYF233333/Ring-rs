//! # UI 布局配置
//!
//! 数据驱动的 UI 布局系统。所有布局参数通过 `ui/layout.json` 加载，
//! 配置文件必须存在且字段完整。像素值基于基准分辨率 1920×1080。
//!
//! 复用自 `host/src/ui/layout/mod.rs`，移除 ScaleContext 和 egui 依赖。
//! Dioxus host 中缩放由 CSS `transform: scale()` 负责，无需 Rust 侧 ScaleContext。

use serde::Deserialize;

use crate::resources::{LogicalPath, ResourceManager};

// ─── Colors ───────────────────────────────────────────────────────────────────

/// 颜色配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorConfig {
    /// 强调色（如 #ffffff）
    pub accent: String,
    /// 空闲状态按钮文字色
    pub idle: String,
    /// 悬停色
    pub hover: String,
    /// 选中色
    pub selected: String,
    /// 不可用色
    pub insensitive: String,
    /// 游戏内对话文字色
    pub text: String,
    /// 界面文字色
    pub interface_text: String,
}

// ─── Font sizes ───────────────────────────────────────────────────────────────

/// 字号配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontConfig {
    /// 对话文字字号
    pub text_size: f32,
    /// 角色名字号
    pub name_text_size: f32,
    /// 界面文字字号
    pub interface_text_size: f32,
    /// 标签字号
    pub label_text_size: f32,
    /// 通知字号
    pub notify_text_size: f32,
    /// 标题字号
    pub title_text_size: f32,
}

// ─── Dialogue layout ─────────────────────────────────────────────────────────

/// 对话框布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueLayoutConfig {
    /// 文本框高度（基准像素）
    pub textbox_height: f32,
    /// 名字 X 偏移
    pub name_xpos: f32,
    /// 名字 Y 偏移
    pub name_ypos: f32,
    /// 名字框边框 (left, top, right, bottom)
    pub namebox_borders: [f32; 4],
    /// 对话文本 X 偏移
    pub dialogue_xpos: f32,
    /// 对话文本 Y 偏移
    pub dialogue_ypos: f32,
    /// 对话文本最大宽度
    pub dialogue_width: f32,
}

// ─── Choice layout ───────────────────────────────────────────────────────────

/// 选项按钮布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceLayoutConfig {
    /// 选项按钮宽度
    pub button_width: f32,
    /// 选项间距
    pub spacing: f32,
    /// 选项按钮边框 (left, top, right, bottom)
    pub button_borders: [f32; 4],
}

// ─── Quick menu ───────────────────────────────────────────────────────────────

/// 快捷菜单配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuickMenuConfig {
    /// 按钮文字大小
    pub text_size: f32,
    /// 按钮边框 (left, top, right, bottom)
    pub button_borders: [f32; 4],
}

// ─── Title layout ─────────────────────────────────────────────────────────────

/// 标题画面布局
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TitleLayoutConfig {
    /// 导航按钮 X 偏移
    pub navigation_xpos: f32,
    /// 导航按钮间距
    pub navigation_spacing: f32,
}

// ─── Game menu layout ─────────────────────────────────────────────────────────

/// 游戏菜单框架布局
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameMenuLayoutConfig {
    /// 导航面板宽度
    pub nav_width: f32,
    /// 导航按钮间距
    pub navigation_spacing: f32,
}

// ─── Save/Load layout ────────────────────────────────────────────────────────

/// 存读档布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveLoadLayoutConfig {
    /// 每行列数
    pub cols: u32,
    /// 每页行数
    pub rows: u32,
    /// 槽位按钮宽度
    pub slot_width: f32,
    /// 槽位按钮高度
    pub slot_height: f32,
    /// 缩略图宽度
    pub thumbnail_width: f32,
    /// 缩略图高度
    pub thumbnail_height: f32,
    /// 槽位间距
    pub slot_spacing: f32,
    /// 页面按钮间距
    pub page_spacing: f32,
}

// ─── History layout ──────────────────────────────────────────────────────────

/// 历史记录布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryLayoutConfig {
    /// 条目高度
    pub entry_height: f32,
    /// 角色名列宽度
    pub name_width: f32,
    /// 对话文本列宽度
    pub text_width: f32,
    /// 角色名 X 偏移
    pub name_xpos: f32,
    /// 对话文本 X 偏移
    pub text_xpos: f32,
}

// ─── Settings layout ─────────────────────────────────────────────────────────

/// 设置页面布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsLayoutConfig {
    /// 设置项间距
    pub pref_spacing: f32,
    /// 设置按钮间距
    pub pref_button_spacing: f32,
}

// ─── Confirm layout ──────────────────────────────────────────────────────────

/// 确认弹窗布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmLayoutConfig {
    /// 确认框边框 (left, top, right, bottom)
    pub frame_borders: [f32; 4],
}

// ─── Skip indicator ──────────────────────────────────────────────────────────

/// 快进指示器配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkipIndicatorConfig {
    /// 快进指示器 Y 偏移
    pub ypos: f32,
    /// 快进框架边框 (left, top, right, bottom)
    pub frame_borders: [f32; 4],
}

// ─── Notify ──────────────────────────────────────────────────────────────────

/// 通知配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotifyConfig {
    /// 通知 Y 偏移
    pub ypos: f32,
    /// 通知框架边框 (left, top, right, bottom)
    pub frame_borders: [f32; 4],
}

// ─── Asset paths ─────────────────────────────────────────────────────────────

/// UI 素材路径配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiAssetPaths {
    pub textbox: String,
    pub namebox: String,
    pub frame: String,
    pub main_menu_overlay: String,
    pub game_menu_overlay: String,
    pub confirm_overlay: String,
    pub skip: String,
    pub notify: String,
    pub main_summer: String,
    pub main_winter: String,
    pub game_menu_bg: String,
    pub button_idle: String,
    pub button_hover: String,
    pub choice_idle: String,
    pub choice_hover: String,
    pub slot_idle: String,
    pub slot_hover: String,
    pub quick_idle: String,
    pub quick_hover: String,
    pub slider_idle_bar: String,
    pub slider_hover_bar: String,
    pub slider_idle_thumb: String,
    pub slider_hover_thumb: String,
}

impl UiAssetPaths {
    /// 将逻辑路径转为 ring-asset 协议 URL
    pub fn asset_url(path: &str) -> String {
        format!("http://ring-asset.localhost/{path}")
    }

    /// 将 screens.json 中的资源 key（如 "main_summer"）解析为实际逻辑路径
    pub fn resolve_key(&self, key: &str) -> String {
        match key {
            "main_summer" => self.main_summer.clone(),
            "main_winter" => self.main_winter.clone(),
            "main_menu_overlay" => self.main_menu_overlay.clone(),
            "game_menu_bg" => self.game_menu_bg.clone(),
            "game_menu_overlay" => self.game_menu_overlay.clone(),
            "confirm_overlay" => self.confirm_overlay.clone(),
            // 如果 key 不匹配任何已知 asset，直接当路径使用
            other => other.to_string(),
        }
    }
}

// ─── Top-level config ────────────────────────────────────────────────────────

/// 顶层 UI 布局配置
///
/// 所有像素值基于 `base_resolution` (默认 1920×1080)，
/// 运行时通过 CSS `transform: scale()` 缩放到实际窗口尺寸。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiLayoutConfig {
    /// 基准分辨率宽度
    pub base_width: f32,
    /// 基准分辨率高度
    pub base_height: f32,
    /// 字号
    pub fonts: FontConfig,
    /// 颜色
    pub colors: ColorConfig,
    /// 对话框
    pub dialogue: DialogueLayoutConfig,
    /// 选项按钮
    pub choice: ChoiceLayoutConfig,
    /// 快捷菜单
    pub quick_menu: QuickMenuConfig,
    /// 标题画面
    pub title: TitleLayoutConfig,
    /// 游戏菜单
    pub game_menu: GameMenuLayoutConfig,
    /// 存读档
    pub save_load: SaveLoadLayoutConfig,
    /// 历史
    pub history: HistoryLayoutConfig,
    /// 设置
    pub settings: SettingsLayoutConfig,
    /// 确认弹窗
    pub confirm: ConfirmLayoutConfig,
    /// 快进指示器
    pub skip_indicator: SkipIndicatorConfig,
    /// 通知
    pub notify: NotifyConfig,
    /// 素材路径
    pub assets: UiAssetPaths,
}

/// UI 布局配置加载错误
#[derive(Debug)]
pub enum LayoutConfigError {
    /// 配置文件缺失或读取失败
    NotFound(String),
    /// JSON 解析失败
    ParseFailed(String),
}

impl std::fmt::Display for LayoutConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutConfigError::NotFound(msg) => write!(f, "布局配置加载失败: {}", msg),
            LayoutConfigError::ParseFailed(msg) => write!(f, "布局配置解析失败: {}", msg),
        }
    }
}

impl std::error::Error for LayoutConfigError {}

impl UiLayoutConfig {
    /// 测试用默认值（不依赖资源文件）
    #[cfg(test)]
    pub fn default_for_tests() -> Self {
        Self {
            base_width: 1920.0,
            base_height: 1080.0,
            fonts: FontConfig {
                text_size: 33.0,
                name_text_size: 45.0,
                interface_text_size: 33.0,
                label_text_size: 36.0,
                notify_text_size: 24.0,
                title_text_size: 75.0,
            },
            colors: ColorConfig {
                accent: "#ffffff".into(),
                idle: "#888888".into(),
                hover: "#ff9900".into(),
                selected: "#ffffff".into(),
                insensitive: "#7878787f".into(),
                text: "#000000".into(),
                interface_text: "#ffffff".into(),
            },
            dialogue: DialogueLayoutConfig {
                textbox_height: 278.0,
                name_xpos: 360.0,
                name_ypos: 0.0,
                namebox_borders: [5.0, 5.0, 5.0, 5.0],
                dialogue_xpos: 402.0,
                dialogue_ypos: 75.0,
                dialogue_width: 1116.0,
            },
            choice: ChoiceLayoutConfig {
                button_width: 1185.0,
                spacing: 33.0,
                button_borders: [150.0, 8.0, 150.0, 8.0],
            },
            quick_menu: QuickMenuConfig {
                text_size: 21.0,
                button_borders: [15.0, 6.0, 15.0, 0.0],
            },
            title: TitleLayoutConfig {
                navigation_xpos: 60.0,
                navigation_spacing: 6.0,
            },
            game_menu: GameMenuLayoutConfig {
                nav_width: 420.0,
                navigation_spacing: 6.0,
            },
            save_load: SaveLoadLayoutConfig {
                cols: 3,
                rows: 2,
                slot_width: 414.0,
                slot_height: 309.0,
                thumbnail_width: 384.0,
                thumbnail_height: 216.0,
                slot_spacing: 15.0,
                page_spacing: 0.0,
            },
            history: HistoryLayoutConfig {
                entry_height: 210.0,
                name_width: 233.0,
                text_width: 1110.0,
                name_xpos: 233.0,
                text_xpos: 255.0,
            },
            settings: SettingsLayoutConfig {
                pref_spacing: 15.0,
                pref_button_spacing: 0.0,
            },
            confirm: ConfirmLayoutConfig {
                frame_borders: [60.0, 60.0, 60.0, 60.0],
            },
            skip_indicator: SkipIndicatorConfig {
                ypos: 15.0,
                frame_borders: [24.0, 8.0, 75.0, 8.0],
            },
            notify: NotifyConfig {
                ypos: 68.0,
                frame_borders: [24.0, 8.0, 60.0, 8.0],
            },
            assets: UiAssetPaths {
                textbox: "gui/textbox.png".into(),
                namebox: "gui/namebox.png".into(),
                frame: "gui/frame.png".into(),
                main_menu_overlay: "gui/overlay/main_menu.png".into(),
                game_menu_overlay: "gui/overlay/game_menu.png".into(),
                confirm_overlay: "gui/overlay/confirm.png".into(),
                skip: "gui/skip.png".into(),
                notify: "gui/notify.png".into(),
                main_summer: "gui/main_summer.jpg".into(),
                main_winter: "gui/main_winter.jpg".into(),
                game_menu_bg: "gui/game_menu.png".into(),
                button_idle: "gui/button/idle_background.png".into(),
                button_hover: "gui/button/hover_background.png".into(),
                choice_idle: "gui/button/choice_idle_background.png".into(),
                choice_hover: "gui/button/choice_hover_background.png".into(),
                slot_idle: "gui/button/slot_idle_background.png".into(),
                slot_hover: "gui/button/slot_hover_background.png".into(),
                quick_idle: "gui/button/quick_idle_background.png".into(),
                quick_hover: "gui/button/quick_hover_background.png".into(),
                slider_idle_bar: "gui/slider/horizontal_idle_bar.png".into(),
                slider_hover_bar: "gui/slider/horizontal_hover_bar.png".into(),
                slider_idle_thumb: "gui/slider/horizontal_idle_thumb.png".into(),
                slider_hover_thumb: "gui/slider/horizontal_hover_thumb.png".into(),
            },
        }
    }

    /// 从 `ResourceManager` 加载布局配置。
    ///
    /// 配置文件 `ui/layout.json` 必须存在且所有字段完整，否则返回错误。
    pub fn load(resource_manager: &ResourceManager) -> Result<Self, LayoutConfigError> {
        let path = LogicalPath::new("ui/layout.json");
        let content = resource_manager
            .read_text_optional(&path)
            .ok_or_else(|| LayoutConfigError::NotFound("ui/layout.json 不存在".into()))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| LayoutConfigError::ParseFailed(format!("ui/layout.json: {e}")))?;

        tracing::info!("UI layout config loaded from ui/layout.json");
        Ok(config)
    }
}
