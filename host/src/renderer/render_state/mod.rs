//! # RenderState 模块
//!
//! 定义渲染状态，管理当前需要渲染的所有元素。

mod dialogue;
mod scene;

use std::collections::HashMap;

pub use dialogue::{DialogueState, EffectiveCps, InlineWait};

use super::character_animation::AnimatableCharacter;
use crate::ui::map::MapDisplayState;
use vn_runtime::command::{Position, TextMode};

/// 渲染状态
///
/// 存储当前帧需要渲染的所有元素状态。
#[derive(Debug, Clone)]
pub struct RenderState {
    /// 当前背景图片路径
    pub current_background: Option<String>,

    /// 可见角色列表（alias -> CharacterSprite）
    pub visible_characters: HashMap<String, CharacterSprite>,

    /// 当前对话状态
    pub dialogue: Option<DialogueState>,

    /// 当前章节标记（用于显示章节过渡）
    pub chapter_mark: Option<ChapterMarkState>,

    /// 当前选择界面状态
    pub choices: Option<ChoicesState>,

    /// UI 是否可见（用于 changeScene 时隐藏 UI）
    pub ui_visible: bool,

    /// 当前标题字卡状态
    pub title_card: Option<TitleCardState>,

    /// 当前场景效果状态（shake/blur/dim 等）
    pub scene_effect: SceneEffectState,

    /// 当前文本显示模式
    pub text_mode: TextMode,

    /// NVL 模式下的累积对话条目
    pub nvl_entries: Vec<NvlEntry>,

    /// 地图显示状态（showMap 激活时非 None）
    pub map_display: Option<MapDisplayState>,
}

/// NVL 模式下的单条对话
#[derive(Debug, Clone)]
pub struct NvlEntry {
    /// 说话者名称
    pub speaker: Option<String>,
    /// 对话文本
    pub content: String,
    /// 当前可见字符数
    pub visible_chars: usize,
    /// 是否显示完成
    pub is_complete: bool,
}

/// 标题字卡状态
#[derive(Debug, Clone)]
pub struct TitleCardState {
    /// 显示文本
    pub text: String,
    /// 总显示时长（秒）
    pub duration: f32,
    /// 已经过的时间（秒）
    pub elapsed: f32,
}

/// 场景效果状态（镜头语言）
#[derive(Debug, Clone, Default)]
pub struct SceneEffectState {
    /// 震动 X 偏移（像素）
    pub shake_offset_x: f32,
    /// 震动 Y 偏移（像素）
    pub shake_offset_y: f32,
    /// 模糊程度（0.0 = 无模糊，1.0 = 全模糊）
    pub blur_amount: f32,
    /// 暗化程度（0.0 = 正常，1.0 = 全黑）
    pub dim_level: f32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            current_background: None,
            visible_characters: HashMap::new(),
            dialogue: None,
            chapter_mark: None,
            choices: None,
            ui_visible: true,
            title_card: None,
            scene_effect: SceneEffectState::default(),
            text_mode: TextMode::default(),
            nvl_entries: Vec::new(),
            map_display: None,
        }
    }
}

impl RenderState {
    /// 创建空的渲染状态
    pub fn new() -> Self {
        Self::default()
    }
}

/// 角色立绘状态
///
/// 存储角色立绘的基本信息和动画状态。
#[derive(Debug, Clone)]
pub struct CharacterSprite {
    /// 纹理路径
    pub texture_path: String,
    /// 位置预设
    pub position: Position,
    /// 渲染顺序（越大越靠前）
    pub z_order: i32,
    /// 是否正在淡出（淡出完成后将被移除）
    pub fading_out: bool,
    /// 动画状态（透明度、位置、缩放等）
    pub anim: AnimatableCharacter,
}

/// 章节标记显示阶段
#[derive(Debug, Clone, PartialEq)]
pub enum ChapterMarkPhase {
    /// 淡入阶段
    FadeIn,
    /// 持续显示阶段
    Visible,
    /// 淡出阶段
    FadeOut,
}

/// 章节标记状态
///
/// 非阻塞、固定持续时间、不受快进影响。
/// 在左上角异步显示，固定时间后自动消失。
#[derive(Debug, Clone)]
pub struct ChapterMarkState {
    /// 章节标题
    pub title: String,
    /// 章节级别
    pub level: u8,
    /// 透明度（用于淡入淡出）
    pub alpha: f32,
    /// 当前阶段计时器（秒）
    pub timer: f32,
    /// 当前显示阶段
    pub phase: ChapterMarkPhase,
}

/// ChapterMark 时间常量
impl ChapterMarkState {
    /// 淡入时长（秒）
    const FADE_IN_DURATION: f32 = 0.4;
    /// 持续显示时长（秒）
    const VISIBLE_DURATION: f32 = 3.0;
    /// 淡出时长（秒）
    const FADE_OUT_DURATION: f32 = 0.6;
}

/// 选择项
#[derive(Debug, Clone)]
pub struct ChoiceItem {
    /// 选项文本
    pub text: String,
    /// 目标标签
    pub target_label: String,
}

/// 选择界面状态
#[derive(Debug, Clone)]
pub struct ChoicesState {
    /// 选项列表
    pub choices: Vec<ChoiceItem>,
    /// 样式
    pub style: Option<String>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 鼠标悬停索引
    pub hovered_index: Option<usize>,
}

#[cfg(test)]
mod tests;
