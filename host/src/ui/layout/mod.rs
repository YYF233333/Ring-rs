//! # UI 布局配置 + 分辨率缩放
//!
//! 数据驱动的 UI 布局系统。所有布局参数均可通过 JSON 覆盖，
//! 默认值精确对齐 ref-project `gui.rpy` 的基准分辨率 1920×1080。

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
pub struct ColorConfig {
    /// 强调色（如 #ffffff）
    #[serde(default = "defaults::accent_color")]
    pub accent: HexColor,
    /// 空闲状态按钮文字色
    #[serde(default = "defaults::idle_color")]
    pub idle: HexColor,
    /// 悬停色
    #[serde(default = "defaults::hover_color")]
    pub hover: HexColor,
    /// 选中色
    #[serde(default = "defaults::selected_color")]
    pub selected: HexColor,
    /// 不可用色
    #[serde(default = "defaults::insensitive_color")]
    pub insensitive: HexColor,
    /// 游戏内对话文字色
    #[serde(default = "defaults::text_color")]
    pub text: HexColor,
    /// 界面文字色
    #[serde(default = "defaults::interface_text_color")]
    pub interface_text: HexColor,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            accent: defaults::accent_color(),
            idle: defaults::idle_color(),
            hover: defaults::hover_color(),
            selected: defaults::selected_color(),
            insensitive: defaults::insensitive_color(),
            text: defaults::text_color(),
            interface_text: defaults::interface_text_color(),
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
pub struct FontConfig {
    /// 对话文字字号
    #[serde(default = "defaults::text_size")]
    pub text_size: f32,
    /// 角色名字号
    #[serde(default = "defaults::name_text_size")]
    pub name_text_size: f32,
    /// 界面文字字号
    #[serde(default = "defaults::interface_text_size")]
    pub interface_text_size: f32,
    /// 标签字号
    #[serde(default = "defaults::label_text_size")]
    pub label_text_size: f32,
    /// 通知字号
    #[serde(default = "defaults::notify_text_size")]
    pub notify_text_size: f32,
    /// 标题字号
    #[serde(default = "defaults::title_text_size")]
    pub title_text_size: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            text_size: defaults::text_size(),
            name_text_size: defaults::name_text_size(),
            interface_text_size: defaults::interface_text_size(),
            label_text_size: defaults::label_text_size(),
            notify_text_size: defaults::notify_text_size(),
            title_text_size: defaults::title_text_size(),
        }
    }
}

// ─── Dialogue layout ─────────────────────────────────────────────────────────

/// 对话框布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct DialogueLayoutConfig {
    /// 文本框高度（基准像素）
    #[serde(default = "defaults::textbox_height")]
    pub textbox_height: f32,
    /// 名字 X 偏移
    #[serde(default = "defaults::name_xpos")]
    pub name_xpos: f32,
    /// 名字 Y 偏移
    #[serde(default = "defaults::name_ypos")]
    pub name_ypos: f32,
    /// 名字框边框 (left, top, right, bottom)
    #[serde(default = "defaults::namebox_borders")]
    pub namebox_borders: [f32; 4],
    /// 对话文本 X 偏移
    #[serde(default = "defaults::dialogue_xpos")]
    pub dialogue_xpos: f32,
    /// 对话文本 Y 偏移
    #[serde(default = "defaults::dialogue_ypos")]
    pub dialogue_ypos: f32,
    /// 对话文本最大宽度
    #[serde(default = "defaults::dialogue_width")]
    pub dialogue_width: f32,
}

impl Default for DialogueLayoutConfig {
    fn default() -> Self {
        Self {
            textbox_height: defaults::textbox_height(),
            name_xpos: defaults::name_xpos(),
            name_ypos: defaults::name_ypos(),
            namebox_borders: defaults::namebox_borders(),
            dialogue_xpos: defaults::dialogue_xpos(),
            dialogue_ypos: defaults::dialogue_ypos(),
            dialogue_width: defaults::dialogue_width(),
        }
    }
}

// ─── Choice layout ───────────────────────────────────────────────────────────

/// 选项按钮布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceLayoutConfig {
    /// 选项按钮宽度
    #[serde(default = "defaults::choice_button_width")]
    pub button_width: f32,
    /// 选项间距
    #[serde(default = "defaults::choice_spacing")]
    pub spacing: f32,
    /// 选项按钮边框 (left, top, right, bottom)
    #[serde(default = "defaults::choice_button_borders")]
    pub button_borders: [f32; 4],
}

impl Default for ChoiceLayoutConfig {
    fn default() -> Self {
        Self {
            button_width: defaults::choice_button_width(),
            spacing: defaults::choice_spacing(),
            button_borders: defaults::choice_button_borders(),
        }
    }
}

// ─── Quick menu ───────────────────────────────────────────────────────────────

/// 快捷菜单配置
#[derive(Debug, Clone, Deserialize)]
pub struct QuickMenuConfig {
    /// 按钮文字大小
    #[serde(default = "defaults::quick_button_text_size")]
    pub text_size: f32,
    /// 按钮边框 (left, top, right, bottom)
    #[serde(default = "defaults::quick_button_borders")]
    pub button_borders: [f32; 4],
}

impl Default for QuickMenuConfig {
    fn default() -> Self {
        Self {
            text_size: defaults::quick_button_text_size(),
            button_borders: defaults::quick_button_borders(),
        }
    }
}

// ─── Title layout ─────────────────────────────────────────────────────────────

/// 标题画面布局
#[derive(Debug, Clone, Deserialize)]
pub struct TitleLayoutConfig {
    /// 导航按钮 X 偏移
    #[serde(default = "defaults::navigation_xpos")]
    pub navigation_xpos: f32,
    /// 导航按钮间距
    #[serde(default = "defaults::navigation_spacing")]
    pub navigation_spacing: f32,
}

impl Default for TitleLayoutConfig {
    fn default() -> Self {
        Self {
            navigation_xpos: defaults::navigation_xpos(),
            navigation_spacing: defaults::navigation_spacing(),
        }
    }
}

// ─── Game menu layout ─────────────────────────────────────────────────────────

/// 游戏菜单框架布局
#[derive(Debug, Clone, Deserialize)]
pub struct GameMenuLayoutConfig {
    /// 导航面板宽度
    #[serde(default = "defaults::game_menu_nav_width")]
    pub nav_width: f32,
    /// 导航按钮间距
    #[serde(default = "defaults::navigation_spacing")]
    pub navigation_spacing: f32,
}

impl Default for GameMenuLayoutConfig {
    fn default() -> Self {
        Self {
            nav_width: defaults::game_menu_nav_width(),
            navigation_spacing: defaults::navigation_spacing(),
        }
    }
}

// ─── Save/Load layout ────────────────────────────────────────────────────────

/// 存读档布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct SaveLoadLayoutConfig {
    /// 每行列数
    #[serde(default = "defaults::file_slot_cols")]
    pub cols: u32,
    /// 每页行数
    #[serde(default = "defaults::file_slot_rows")]
    pub rows: u32,
    /// 槽位按钮宽度
    #[serde(default = "defaults::slot_button_width")]
    pub slot_width: f32,
    /// 槽位按钮高度
    #[serde(default = "defaults::slot_button_height")]
    pub slot_height: f32,
    /// 缩略图宽度
    #[serde(default = "defaults::thumbnail_width")]
    pub thumbnail_width: f32,
    /// 缩略图高度
    #[serde(default = "defaults::thumbnail_height")]
    pub thumbnail_height: f32,
    /// 槽位间距
    #[serde(default = "defaults::slot_spacing")]
    pub slot_spacing: f32,
    /// 页面按钮间距
    #[serde(default = "defaults::page_spacing")]
    pub page_spacing: f32,
}

impl Default for SaveLoadLayoutConfig {
    fn default() -> Self {
        Self {
            cols: defaults::file_slot_cols(),
            rows: defaults::file_slot_rows(),
            slot_width: defaults::slot_button_width(),
            slot_height: defaults::slot_button_height(),
            thumbnail_width: defaults::thumbnail_width(),
            thumbnail_height: defaults::thumbnail_height(),
            slot_spacing: defaults::slot_spacing(),
            page_spacing: defaults::page_spacing(),
        }
    }
}

// ─── History layout ──────────────────────────────────────────────────────────

/// 历史记录布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryLayoutConfig {
    /// 条目高度
    #[serde(default = "defaults::history_height")]
    pub entry_height: f32,
    /// 角色名列宽度
    #[serde(default = "defaults::history_name_width")]
    pub name_width: f32,
    /// 对话文本列宽度
    #[serde(default = "defaults::history_text_width")]
    pub text_width: f32,
    /// 角色名 X 偏移
    #[serde(default = "defaults::history_name_xpos")]
    pub name_xpos: f32,
    /// 对话文本 X 偏移
    #[serde(default = "defaults::history_text_xpos")]
    pub text_xpos: f32,
}

impl Default for HistoryLayoutConfig {
    fn default() -> Self {
        Self {
            entry_height: defaults::history_height(),
            name_width: defaults::history_name_width(),
            text_width: defaults::history_text_width(),
            name_xpos: defaults::history_name_xpos(),
            text_xpos: defaults::history_text_xpos(),
        }
    }
}

// ─── Settings layout ─────────────────────────────────────────────────────────

/// 设置页面布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLayoutConfig {
    /// 设置项间距
    #[serde(default = "defaults::pref_spacing")]
    pub pref_spacing: f32,
    /// 设置按钮间距
    #[serde(default = "defaults::pref_button_spacing")]
    pub pref_button_spacing: f32,
}

impl Default for SettingsLayoutConfig {
    fn default() -> Self {
        Self {
            pref_spacing: defaults::pref_spacing(),
            pref_button_spacing: defaults::pref_button_spacing(),
        }
    }
}

// ─── Confirm layout ──────────────────────────────────────────────────────────

/// 确认弹窗布局配置
#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmLayoutConfig {
    /// 确认框边框 (left, top, right, bottom)
    #[serde(default = "defaults::confirm_frame_borders")]
    pub frame_borders: [f32; 4],
}

impl Default for ConfirmLayoutConfig {
    fn default() -> Self {
        Self {
            frame_borders: defaults::confirm_frame_borders(),
        }
    }
}

// ─── Skip indicator ──────────────────────────────────────────────────────────

/// 快进指示器配置
#[derive(Debug, Clone, Deserialize)]
pub struct SkipIndicatorConfig {
    /// 快进指示器 Y 偏移
    #[serde(default = "defaults::skip_ypos")]
    pub ypos: f32,
    /// 快进框架边框 (left, top, right, bottom)
    #[serde(default = "defaults::skip_frame_borders")]
    pub frame_borders: [f32; 4],
}

impl Default for SkipIndicatorConfig {
    fn default() -> Self {
        Self {
            ypos: defaults::skip_ypos(),
            frame_borders: defaults::skip_frame_borders(),
        }
    }
}

// ─── Notify ──────────────────────────────────────────────────────────────────

/// 通知配置
#[derive(Debug, Clone, Deserialize)]
pub struct NotifyConfig {
    /// 通知 Y 偏移
    #[serde(default = "defaults::notify_ypos")]
    pub ypos: f32,
    /// 通知框架边框 (left, top, right, bottom)
    #[serde(default = "defaults::notify_frame_borders")]
    pub frame_borders: [f32; 4],
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            ypos: defaults::notify_ypos(),
            frame_borders: defaults::notify_frame_borders(),
        }
    }
}

// ─── Asset paths ─────────────────────────────────────────────────────────────

/// UI 素材路径配置
#[derive(Debug, Clone, Deserialize)]
pub struct UiAssetPaths {
    #[serde(default = "defaults::asset_textbox")]
    pub textbox: String,
    #[serde(default = "defaults::asset_namebox")]
    pub namebox: String,
    #[serde(default = "defaults::asset_frame")]
    pub frame: String,
    #[serde(default = "defaults::asset_main_menu_overlay")]
    pub main_menu_overlay: String,
    #[serde(default = "defaults::asset_game_menu_overlay")]
    pub game_menu_overlay: String,
    #[serde(default = "defaults::asset_confirm_overlay")]
    pub confirm_overlay: String,
    #[serde(default = "defaults::asset_skip")]
    pub skip: String,
    #[serde(default = "defaults::asset_notify")]
    pub notify: String,

    // Backgrounds
    #[serde(default = "defaults::asset_main_summer")]
    pub main_summer: String,
    #[serde(default = "defaults::asset_main_winter")]
    pub main_winter: String,
    #[serde(default = "defaults::asset_game_menu_bg")]
    pub game_menu_bg: String,

    // Buttons
    #[serde(default = "defaults::asset_button_idle")]
    pub button_idle: String,
    #[serde(default = "defaults::asset_button_hover")]
    pub button_hover: String,
    #[serde(default = "defaults::asset_choice_idle")]
    pub choice_idle: String,
    #[serde(default = "defaults::asset_choice_hover")]
    pub choice_hover: String,
    #[serde(default = "defaults::asset_slot_idle")]
    pub slot_idle: String,
    #[serde(default = "defaults::asset_slot_hover")]
    pub slot_hover: String,
    #[serde(default = "defaults::asset_quick_idle")]
    pub quick_idle: String,
    #[serde(default = "defaults::asset_quick_hover")]
    pub quick_hover: String,

    // Slider
    #[serde(default = "defaults::asset_slider_idle_bar")]
    pub slider_idle_bar: String,
    #[serde(default = "defaults::asset_slider_hover_bar")]
    pub slider_hover_bar: String,
    #[serde(default = "defaults::asset_slider_idle_thumb")]
    pub slider_idle_thumb: String,
    #[serde(default = "defaults::asset_slider_hover_thumb")]
    pub slider_hover_thumb: String,
}

impl Default for UiAssetPaths {
    fn default() -> Self {
        Self {
            textbox: defaults::asset_textbox(),
            namebox: defaults::asset_namebox(),
            frame: defaults::asset_frame(),
            main_menu_overlay: defaults::asset_main_menu_overlay(),
            game_menu_overlay: defaults::asset_game_menu_overlay(),
            confirm_overlay: defaults::asset_confirm_overlay(),
            skip: defaults::asset_skip(),
            notify: defaults::asset_notify(),
            main_summer: defaults::asset_main_summer(),
            main_winter: defaults::asset_main_winter(),
            game_menu_bg: defaults::asset_game_menu_bg(),
            button_idle: defaults::asset_button_idle(),
            button_hover: defaults::asset_button_hover(),
            choice_idle: defaults::asset_choice_idle(),
            choice_hover: defaults::asset_choice_hover(),
            slot_idle: defaults::asset_slot_idle(),
            slot_hover: defaults::asset_slot_hover(),
            quick_idle: defaults::asset_quick_idle(),
            quick_hover: defaults::asset_quick_hover(),
            slider_idle_bar: defaults::asset_slider_idle_bar(),
            slider_hover_bar: defaults::asset_slider_hover_bar(),
            slider_idle_thumb: defaults::asset_slider_idle_thumb(),
            slider_hover_thumb: defaults::asset_slider_hover_thumb(),
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
pub struct UiLayoutConfig {
    /// 基准分辨率宽度
    #[serde(default = "defaults::base_width")]
    pub base_width: f32,
    /// 基准分辨率高度
    #[serde(default = "defaults::base_height")]
    pub base_height: f32,
    /// 字号
    #[serde(default)]
    pub fonts: FontConfig,
    /// 颜色
    #[serde(default)]
    pub colors: ColorConfig,
    /// 对话框
    #[serde(default)]
    pub dialogue: DialogueLayoutConfig,
    /// 选项按钮
    #[serde(default)]
    pub choice: ChoiceLayoutConfig,
    /// 快捷菜单
    #[serde(default)]
    pub quick_menu: QuickMenuConfig,
    /// 标题画面
    #[serde(default)]
    pub title: TitleLayoutConfig,
    /// 游戏菜单
    #[serde(default)]
    pub game_menu: GameMenuLayoutConfig,
    /// 存读档
    #[serde(default)]
    pub save_load: SaveLoadLayoutConfig,
    /// 历史
    #[serde(default)]
    pub history: HistoryLayoutConfig,
    /// 设置
    #[serde(default)]
    pub settings: SettingsLayoutConfig,
    /// 确认弹窗
    #[serde(default)]
    pub confirm: ConfirmLayoutConfig,
    /// 快进指示器
    #[serde(default)]
    pub skip_indicator: SkipIndicatorConfig,
    /// 通知
    #[serde(default)]
    pub notify: NotifyConfig,
    /// 素材路径
    #[serde(default)]
    pub assets: UiAssetPaths,
}

impl Default for UiLayoutConfig {
    fn default() -> Self {
        Self {
            base_width: defaults::base_width(),
            base_height: defaults::base_height(),
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

impl UiLayoutConfig {
    /// 从 `ResourceManager` 加载布局配置。
    ///
    /// 尝试读取 `ui/layout.json`，失败时回退到默认值。
    pub fn load(resource_manager: &ResourceManager) -> Self {
        let path = LogicalPath::new("ui/layout.json");
        match resource_manager.read_text_optional(&path) {
            Some(content) => match serde_json::from_str::<Self>(&content) {
                Ok(config) => {
                    tracing::info!("UI layout config loaded from ui/layout.json");
                    config
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse ui/layout.json, using defaults");
                    Self::default()
                }
            },
            None => {
                tracing::info!("No ui/layout.json found, using default layout");
                Self::default()
            }
        }
    }
}

// ─── Default values (from gui.rpy) ───────────────────────────────────────────

mod defaults {
    pub fn base_width() -> f32 {
        1920.0
    }
    pub fn base_height() -> f32 {
        1080.0
    }

    // Font sizes
    pub fn text_size() -> f32 {
        33.0
    }
    pub fn name_text_size() -> f32 {
        45.0
    }
    pub fn interface_text_size() -> f32 {
        33.0
    }
    pub fn label_text_size() -> f32 {
        36.0
    }
    pub fn notify_text_size() -> f32 {
        24.0
    }
    pub fn title_text_size() -> f32 {
        75.0
    }

    // Colors
    pub fn accent_color() -> super::HexColor {
        super::HexColor("#ffffff".into())
    }
    pub fn idle_color() -> super::HexColor {
        super::HexColor("#888888".into())
    }
    pub fn hover_color() -> super::HexColor {
        super::HexColor("#ff9900".into())
    }
    pub fn selected_color() -> super::HexColor {
        super::HexColor("#ffffff".into())
    }
    pub fn insensitive_color() -> super::HexColor {
        super::HexColor("#7878787f".into())
    }
    pub fn text_color() -> super::HexColor {
        super::HexColor("#000000".into())
    }
    pub fn interface_text_color() -> super::HexColor {
        super::HexColor("#ffffff".into())
    }

    // Dialogue
    pub fn textbox_height() -> f32 {
        278.0
    }
    pub fn name_xpos() -> f32 {
        360.0
    }
    pub fn name_ypos() -> f32 {
        0.0
    }
    pub fn namebox_borders() -> [f32; 4] {
        [5.0, 5.0, 5.0, 5.0]
    }
    pub fn dialogue_xpos() -> f32 {
        402.0
    }
    pub fn dialogue_ypos() -> f32 {
        75.0
    }
    pub fn dialogue_width() -> f32 {
        1116.0
    }

    // Choice
    pub fn choice_button_width() -> f32 {
        1185.0
    }
    pub fn choice_spacing() -> f32 {
        33.0
    }
    pub fn choice_button_borders() -> [f32; 4] {
        [150.0, 8.0, 150.0, 8.0]
    }

    // Quick menu
    pub fn quick_button_text_size() -> f32 {
        21.0
    }
    pub fn quick_button_borders() -> [f32; 4] {
        [15.0, 6.0, 15.0, 0.0]
    }

    // Title
    pub fn navigation_xpos() -> f32 {
        60.0
    }
    pub fn navigation_spacing() -> f32 {
        6.0
    }

    // Game menu
    pub fn game_menu_nav_width() -> f32 {
        420.0
    }

    // Save/Load
    pub fn file_slot_cols() -> u32 {
        3
    }
    pub fn file_slot_rows() -> u32 {
        2
    }
    pub fn slot_button_width() -> f32 {
        414.0
    }
    pub fn slot_button_height() -> f32 {
        309.0
    }
    pub fn thumbnail_width() -> f32 {
        384.0
    }
    pub fn thumbnail_height() -> f32 {
        216.0
    }
    pub fn slot_spacing() -> f32 {
        15.0
    }
    pub fn page_spacing() -> f32 {
        0.0
    }

    // History
    pub fn history_height() -> f32 {
        210.0
    }
    pub fn history_name_xpos() -> f32 {
        233.0
    }
    pub fn history_name_width() -> f32 {
        233.0
    }
    pub fn history_text_xpos() -> f32 {
        255.0
    }
    pub fn history_text_width() -> f32 {
        1110.0
    }

    // Settings
    pub fn pref_spacing() -> f32 {
        15.0
    }
    pub fn pref_button_spacing() -> f32 {
        0.0
    }

    // Confirm
    pub fn confirm_frame_borders() -> [f32; 4] {
        [60.0, 60.0, 60.0, 60.0]
    }

    // Skip indicator
    pub fn skip_ypos() -> f32 {
        15.0
    }
    pub fn skip_frame_borders() -> [f32; 4] {
        [24.0, 8.0, 75.0, 8.0]
    }

    // Notify
    pub fn notify_ypos() -> f32 {
        68.0
    }
    pub fn notify_frame_borders() -> [f32; 4] {
        [24.0, 8.0, 60.0, 8.0]
    }

    // Asset paths
    pub fn asset_textbox() -> String {
        "gui/textbox.png".into()
    }
    pub fn asset_namebox() -> String {
        "gui/namebox.png".into()
    }
    pub fn asset_frame() -> String {
        "gui/frame.png".into()
    }
    pub fn asset_main_menu_overlay() -> String {
        "gui/overlay/main_menu.png".into()
    }
    pub fn asset_game_menu_overlay() -> String {
        "gui/overlay/game_menu.png".into()
    }
    pub fn asset_confirm_overlay() -> String {
        "gui/overlay/confirm.png".into()
    }
    pub fn asset_skip() -> String {
        "gui/skip.png".into()
    }
    pub fn asset_notify() -> String {
        "gui/notify.png".into()
    }
    pub fn asset_main_summer() -> String {
        "gui/main_summer.jpg".into()
    }
    pub fn asset_main_winter() -> String {
        "gui/main_winter.jpg".into()
    }
    pub fn asset_game_menu_bg() -> String {
        "gui/game_menu.png".into()
    }
    pub fn asset_button_idle() -> String {
        "gui/button/idle_background.png".into()
    }
    pub fn asset_button_hover() -> String {
        "gui/button/hover_background.png".into()
    }
    pub fn asset_choice_idle() -> String {
        "gui/button/choice_idle_background.png".into()
    }
    pub fn asset_choice_hover() -> String {
        "gui/button/choice_hover_background.png".into()
    }
    pub fn asset_slot_idle() -> String {
        "gui/button/slot_idle_background.png".into()
    }
    pub fn asset_slot_hover() -> String {
        "gui/button/slot_hover_background.png".into()
    }
    pub fn asset_quick_idle() -> String {
        "gui/button/quick_idle_background.png".into()
    }
    pub fn asset_quick_hover() -> String {
        "gui/button/quick_hover_background.png".into()
    }
    pub fn asset_slider_idle_bar() -> String {
        "gui/slider/horizontal_idle_bar.png".into()
    }
    pub fn asset_slider_hover_bar() -> String {
        "gui/slider/horizontal_hover_bar.png".into()
    }
    pub fn asset_slider_idle_thumb() -> String {
        "gui/slider/horizontal_idle_thumb.png".into()
    }
    pub fn asset_slider_hover_thumb() -> String {
        "gui/slider/horizontal_hover_thumb.png".into()
    }
}

#[cfg(test)]
mod tests;
