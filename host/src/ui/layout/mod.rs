//! # UI 布局配置 + 分辨率缩放
//!
//! 数据驱动的 UI 布局系统。所有布局参数通过 `ui/layout.json` 加载，
//! 配置文件必须存在且字段完整。像素值基于基准分辨率 1920×1080。

use serde::Deserialize;

use crate::resources::{LogicalPath, ResourceManager};

// ─── ScaleContext ─────────────────────────────────────────────────────────────

/// 基准分辨率 → 实际分辨率的缩放上下文。
///
/// 所有 `UiLayoutConfig` 中的像素值均基于 `base` 分辨率定义，
/// 渲染时通过 `ScaleContext` 映射到实际窗口尺寸。
#[derive(Debug, Clone, Copy)]
pub struct ScaleContext {
    pub base_w: f32,
    pub base_h: f32,
    pub actual_w: f32,
    pub actual_h: f32,
    scale_x: f32,
    scale_y: f32,
    scale_uniform: f32,
}

impl ScaleContext {
    pub fn new(base_w: f32, base_h: f32, actual_w: f32, actual_h: f32) -> Self {
        let sx = actual_w / base_w;
        let sy = actual_h / base_h;
        Self {
            base_w,
            base_h,
            actual_w,
            actual_h,
            scale_x: sx,
            scale_y: sy,
            scale_uniform: sx.min(sy),
        }
    }

    /// 缩放水平像素值
    pub fn x(&self, base: f32) -> f32 {
        base * self.scale_x
    }

    /// 缩放垂直像素值
    pub fn y(&self, base: f32) -> f32 {
        base * self.scale_y
    }

    /// 等比缩放（取 min，保持宽高比）
    pub fn uniform(&self, base: f32) -> f32 {
        base * self.scale_uniform
    }

    /// 一次性缩放矩形
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.x(x), self.y(y)),
            egui::vec2(self.x(w), self.y(h)),
        )
    }

    /// 缩放 Vec2
    pub fn vec2(&self, w: f32, h: f32) -> egui::Vec2 {
        egui::vec2(self.x(w), self.y(h))
    }
}

// ─── Colors ───────────────────────────────────────────────────────────────────

/// 颜色配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorConfig {
    /// 强调色（如 #ffffff）
    pub accent: HexColor,
    /// 空闲状态按钮文字色
    pub idle: HexColor,
    /// 悬停色
    pub hover: HexColor,
    /// 选中色
    pub selected: HexColor,
    /// 不可用色
    pub insensitive: HexColor,
    /// 游戏内对话文字色
    pub text: HexColor,
    /// 界面文字色
    pub interface_text: HexColor,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            accent: HexColor("#ffffff".into()),
            idle: HexColor("#888888".into()),
            hover: HexColor("#ff9900".into()),
            selected: HexColor("#ffffff".into()),
            insensitive: HexColor("#7878787f".into()),
            text: HexColor("#000000".into()),
            interface_text: HexColor("#ffffff".into()),
        }
    }
}

/// 十六进制颜色值（如 `"#ff9900"` 或 `"#7878787f"`）
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct HexColor(pub String);

impl HexColor {
    pub fn to_egui(&self) -> egui::Color32 {
        parse_hex_color(&self.0)
    }
}

fn parse_hex_color(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            egui::Color32::from_rgb(r, g, b)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            egui::Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        _ => egui::Color32::WHITE,
    }
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

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            text_size: 33.0,
            name_text_size: 45.0,
            interface_text_size: 33.0,
            label_text_size: 36.0,
            notify_text_size: 24.0,
            title_text_size: 75.0,
        }
    }
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

impl Default for DialogueLayoutConfig {
    fn default() -> Self {
        Self {
            textbox_height: 278.0,
            name_xpos: 360.0,
            name_ypos: 0.0,
            namebox_borders: [5.0, 5.0, 5.0, 5.0],
            dialogue_xpos: 402.0,
            dialogue_ypos: 75.0,
            dialogue_width: 1116.0,
        }
    }
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

impl Default for ChoiceLayoutConfig {
    fn default() -> Self {
        Self {
            button_width: 1185.0,
            spacing: 33.0,
            button_borders: [150.0, 8.0, 150.0, 8.0],
        }
    }
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

impl Default for QuickMenuConfig {
    fn default() -> Self {
        Self {
            text_size: 21.0,
            button_borders: [15.0, 6.0, 15.0, 0.0],
        }
    }
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

impl Default for TitleLayoutConfig {
    fn default() -> Self {
        Self {
            navigation_xpos: 60.0,
            navigation_spacing: 6.0,
        }
    }
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

impl Default for GameMenuLayoutConfig {
    fn default() -> Self {
        Self {
            nav_width: 420.0,
            navigation_spacing: 6.0,
        }
    }
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

impl Default for SaveLoadLayoutConfig {
    fn default() -> Self {
        Self {
            cols: 3,
            rows: 2,
            slot_width: 414.0,
            slot_height: 309.0,
            thumbnail_width: 384.0,
            thumbnail_height: 216.0,
            slot_spacing: 15.0,
            page_spacing: 0.0,
        }
    }
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

impl Default for HistoryLayoutConfig {
    fn default() -> Self {
        Self {
            entry_height: 210.0,
            name_width: 233.0,
            text_width: 1110.0,
            name_xpos: 233.0,
            text_xpos: 255.0,
        }
    }
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

impl Default for SettingsLayoutConfig {
    fn default() -> Self {
        Self {
            pref_spacing: 15.0,
            pref_button_spacing: 0.0,
        }
    }
}

// ─── Confirm layout ──────────────────────────────────────────────────────────

/// 确认弹窗布局配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmLayoutConfig {
    /// 确认框边框 (left, top, right, bottom)
    pub frame_borders: [f32; 4],
}

impl Default for ConfirmLayoutConfig {
    fn default() -> Self {
        Self {
            frame_borders: [60.0, 60.0, 60.0, 60.0],
        }
    }
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

impl Default for SkipIndicatorConfig {
    fn default() -> Self {
        Self {
            ypos: 15.0,
            frame_borders: [24.0, 8.0, 75.0, 8.0],
        }
    }
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

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            ypos: 68.0,
            frame_borders: [24.0, 8.0, 60.0, 8.0],
        }
    }
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

impl Default for UiAssetPaths {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl UiAssetPaths {
    /// 返回所有 (key, path) 对，用于加载到 UiAssetCache
    pub fn all_entries(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("textbox", &self.textbox),
            ("namebox", &self.namebox),
            ("frame", &self.frame),
            ("main_menu_overlay", &self.main_menu_overlay),
            ("game_menu_overlay", &self.game_menu_overlay),
            ("confirm_overlay", &self.confirm_overlay),
            ("skip", &self.skip),
            ("notify", &self.notify),
            ("main_summer", &self.main_summer),
            ("main_winter", &self.main_winter),
            ("game_menu_bg", &self.game_menu_bg),
            ("button_idle", &self.button_idle),
            ("button_hover", &self.button_hover),
            ("choice_idle", &self.choice_idle),
            ("choice_hover", &self.choice_hover),
            ("slot_idle", &self.slot_idle),
            ("slot_hover", &self.slot_hover),
            ("quick_idle", &self.quick_idle),
            ("quick_hover", &self.quick_hover),
            ("slider_idle_bar", &self.slider_idle_bar),
            ("slider_hover_bar", &self.slider_hover_bar),
            ("slider_idle_thumb", &self.slider_idle_thumb),
            ("slider_hover_thumb", &self.slider_hover_thumb),
        ]
    }
}

// ─── Top-level config ────────────────────────────────────────────────────────

/// 顶层 UI 布局配置
///
/// 所有像素值基于 `base_resolution` (默认 1920×1080)，
/// 运行时通过 [`ScaleContext`] 缩放到实际窗口尺寸。
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

impl Default for UiLayoutConfig {
    fn default() -> Self {
        Self {
            base_width: 1920.0,
            base_height: 1080.0,
            fonts: FontConfig::default(),
            colors: ColorConfig::default(),
            dialogue: DialogueLayoutConfig::default(),
            choice: ChoiceLayoutConfig::default(),
            quick_menu: QuickMenuConfig::default(),
            title: TitleLayoutConfig::default(),
            game_menu: GameMenuLayoutConfig::default(),
            save_load: SaveLoadLayoutConfig::default(),
            history: HistoryLayoutConfig::default(),
            settings: SettingsLayoutConfig::default(),
            confirm: ConfirmLayoutConfig::default(),
            skip_indicator: SkipIndicatorConfig::default(),
            notify: NotifyConfig::default(),
            assets: UiAssetPaths::default(),
        }
    }
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

#[cfg(test)]
mod tests;
