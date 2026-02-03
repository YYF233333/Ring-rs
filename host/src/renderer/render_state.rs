//! # RenderState 模块
//!
//! 定义渲染状态，管理当前需要渲染的所有元素。

use std::collections::HashMap;
use vn_runtime::command::Position;

/// 遮罩类型
#[derive(Debug, Clone)]
pub enum SceneMaskType {
    /// 纯色遮罩（黑色）
    SolidBlack,
    /// 纯色遮罩（白色）
    SolidWhite,
    /// 图片遮罩（rule-based）
    Rule {
        /// 遮罩图片路径
        mask_path: String,
        /// 是否反向
        reversed: bool,
    },
}

/// UI 淡入时长（秒）
const UI_FADE_DURATION: f32 = 0.2;

/// Rule 效果黑屏停顿时长（秒）
const RULE_BLACKOUT_DURATION: f32 = 0.2;

/// 场景遮罩状态
#[derive(Debug, Clone)]
pub struct SceneMaskState {
    /// 遮罩类型
    pub mask_type: SceneMaskType,
    /// 遮罩透明度（0.0 = 完全透明，1.0 = 完全不透明）
    pub alpha: f32,
    /// 等待切换的新背景（在遮罩中点时切换）
    pub pending_background: Option<String>,
    /// UI 透明度（0.0 = 完全透明，1.0 = 完全不透明）
    pub ui_alpha: f32,
    /// 过渡阶段（0 = 淡入遮罩，1 = 淡出遮罩，2 = UI 淡入）
    pub phase: u8,
    /// 遮罩过渡时长（秒）
    pub duration: f32,
    /// 当前计时器
    pub timer: f32,
    /// ImageDissolve 进度（用于 rule 效果，0.0 - 1.0）
    pub dissolve_progress: f32,
}

impl SceneMaskState {
    pub fn new(mask_type: SceneMaskType, duration: f32) -> Self {
        Self {
            mask_type,
            alpha: 0.0,
            pending_background: None,
            ui_alpha: 0.0,
            phase: 0,
            duration: duration.max(0.01),
            timer: 0.0,
            dissolve_progress: 0.0,
        }
    }

    /// 设置待切换背景（在遮罩中点切换）
    pub fn set_pending_background(&mut self, path: String) {
        self.pending_background = Some(path);
    }

    /// 更新遮罩状态，返回是否完成
    /// 
    /// Rule 效果的 phase 流程：
    /// - phase 0: 旧背景 → 黑屏
    /// - phase 1: 黑屏停顿（0.2s）
    /// - phase 2: 黑屏 → 新背景
    /// - phase 3: UI 淡入
    /// 
    /// Fade/FadeWhite 效果的 phase 流程：
    /// - phase 0: 淡入遮罩
    /// - phase 1: 淡出遮罩
    /// - phase 2: UI 淡入
    pub fn update(&mut self, dt: f32) -> bool {
        self.timer += dt;

        match self.phase {
            0 => {
                // 阶段 0: 淡入遮罩（旧背景 → 遮罩）
                let progress = (self.timer / self.duration).min(1.0);
                self.alpha = progress;
                self.dissolve_progress = progress;
                if progress >= 1.0 {
                    // Rule 进入黑屏停顿阶段，其他直接进入淡出阶段
                    self.phase = match self.mask_type {
                        SceneMaskType::Rule { .. } => 1,  // 黑屏停顿
                        _ => 1,  // 淡出遮罩
                    };
                    self.timer = 0.0;
                    // 保持遮罩完全覆盖
                    self.alpha = 1.0;
                    self.dissolve_progress = 1.0;
                }
                false
            }
            1 => {
                match self.mask_type {
                    SceneMaskType::Rule { .. } => {
                        // Rule: 阶段 1 是黑屏停顿
                        // 保持全黑，不更新 dissolve_progress
                        self.alpha = 1.0;
                        self.dissolve_progress = 1.0;  // 保持全黑
                        if self.timer >= RULE_BLACKOUT_DURATION {
                            self.phase = 2;  // 进入黑屏→新背景阶段
                            self.timer = 0.0;
                            self.dissolve_progress = 0.0;  // 重置为 0，准备从黑屏溶解到新背景
                        }
                        false
                    }
                    _ => {
                        // Fade/FadeWhite: 阶段 1 是淡出遮罩
                        let progress = (self.timer / self.duration).min(1.0);
                        self.alpha = 1.0 - progress;
                        self.dissolve_progress = 1.0 - progress;
                        if progress >= 1.0 {
                            self.phase = 2;  // UI 淡入
                            self.timer = 0.0;
                        }
                        false
                    }
                }
            }
            2 => {
                match self.mask_type {
                    SceneMaskType::Rule { .. } => {
                        // Rule: 阶段 2 是黑屏 → 新背景
                        let progress = (self.timer / self.duration).min(1.0);
                        self.dissolve_progress = progress;
                        if progress >= 1.0 {
                            self.phase = 3;  // UI 淡入
                            self.timer = 0.0;
                        }
                        false
                    }
                    _ => {
                        // Fade/FadeWhite: 阶段 2 是 UI 淡入
                        let progress = (self.timer / UI_FADE_DURATION).min(1.0);
                        self.ui_alpha = progress;
                        progress >= 1.0
                    }
                }
            }
            3 => {
                // Rule: 阶段 3 是 UI 淡入
                let progress = (self.timer / UI_FADE_DURATION).min(1.0);
                self.ui_alpha = progress;
                progress >= 1.0
            }
            _ => true,
        }
    }

    /// 判断是否处于中间状态（可以进行场景切换）
    /// 对于 Fade/FadeWhite：phase 1 刚开始时
    /// 对于 Rule：phase 2 刚开始时（黑屏停顿结束后）
    pub fn is_at_midpoint(&self) -> bool {
        match self.mask_type {
            SceneMaskType::Rule { .. } => self.phase == 2 && self.timer < 0.01,
            _ => self.phase == 1 && self.timer < 0.01,
        }
    }

    /// 判断是否正在进行 UI 淡入
    /// 对于 Fade/FadeWhite：phase 2
    /// 对于 Rule：phase 3
    pub fn is_ui_fading_in(&self) -> bool {
        match self.mask_type {
            SceneMaskType::Rule { .. } => self.phase == 3,
            _ => self.phase == 2,
        }
    }

    /// 判断遮罩是否已完成（不再需要渲染）
    pub fn is_mask_complete(&self) -> bool {
        match self.mask_type {
            SceneMaskType::Rule { .. } => self.phase >= 3,
            _ => self.phase >= 2,
        }
    }

    /// 跳过当前阶段的转场动画
    /// 
    /// - phase 0（遮罩淡入）：立刻跳到遮罩完全显现的状态
    ///   - Rule: 跳到 phase 2 开始（黑屏停顿结束，准备显示新背景）
    ///   - Fade/FadeWhite: 跳到 phase 1 开始（遮罩完全显现，准备淡出）
    /// - phase 1（遮罩淡出/黑屏停顿）：立刻完成整个转场（phase 2/3 结束）
    /// - phase 2（黑屏→新背景/UI淡入）：立刻完成整个转场
    pub fn skip_current_phase(&mut self) {
        match self.phase {
            0 => {
                // phase 0: 遮罩淡入 → 立刻跳到遮罩完全显现
                match self.mask_type {
                    SceneMaskType::Rule { .. } => {
                        // Rule: 跳到 phase 2 开始（黑屏停顿结束，准备显示新背景）
                        // 这样背景会在 is_at_midpoint() 时切换
                        self.phase = 2;
                        self.alpha = 1.0;
                        self.dissolve_progress = 0.0;  // 准备从黑屏溶解到新背景
                        self.timer = 0.0;
                    }
                    _ => {
                        // Fade/FadeWhite: 跳到 phase 1 开始（遮罩完全显现，准备淡出）
                        // 这样背景会在 is_at_midpoint() 时切换
                        self.phase = 1;
                        self.alpha = 1.0;  // 遮罩完全显现
                        self.dissolve_progress = 1.0;
                        self.timer = 0.0;
                    }
                }
            }
            1 => {
                // phase 1: 遮罩淡出/黑屏停顿 → 立刻完成整个转场
                match self.mask_type {
                    SceneMaskType::Rule { .. } => {
                        // Rule: 跳到 phase 3（UI淡入）的结束状态
                        self.phase = 3;
                        self.alpha = 0.0;
                        self.dissolve_progress = 1.0;  // 新背景完全显示
                        self.ui_alpha = 1.0;
                        self.timer = 0.0;
                    }
                    _ => {
                        // Fade/FadeWhite: 跳到 phase 2（UI淡入）的结束状态
                        self.phase = 2;
                        self.alpha = 0.0;
                        self.dissolve_progress = 0.0;
                        self.ui_alpha = 1.0;
                        self.timer = 0.0;
                    }
                }
            }
            2 => {
                // phase 2: 黑屏→新背景/UI淡入 → 立刻完成
                match self.mask_type {
                    SceneMaskType::Rule { .. } => {
                        // Rule: 跳到 phase 3（UI淡入）的结束状态
                        self.phase = 3;
                        self.dissolve_progress = 1.0;
                        self.ui_alpha = 1.0;
                        self.timer = 0.0;
                    }
                    _ => {
                        // Fade/FadeWhite: 已经是 phase 2（UI淡入），直接完成
                        self.ui_alpha = 1.0;
                        self.timer = 0.0;
                    }
                }
            }
            _ => {
                // phase 3 或更高，已经完成，无需处理
            }
        }
    }

    /// 获取当前 UI 透明度
    pub fn get_ui_alpha(&self) -> f32 {
        self.ui_alpha
    }
}

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

    /// 场景遮罩状态（用于 changeScene 过渡）
    pub scene_mask: Option<SceneMaskState>,
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
            scene_mask: None,
        }
    }
}

impl RenderState {
    /// 创建空的渲染状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取有效的 UI 透明度
    ///
    /// 如果有场景遮罩且处于 UI 淡入阶段，返回遮罩的 ui_alpha；
    /// 否则返回 1.0（完全不透明）。
    pub fn get_effective_ui_alpha(&self) -> f32 {
        if let Some(ref mask) = self.scene_mask {
            if mask.is_ui_fading_in() {
                return mask.get_ui_alpha();
            }
        }
        1.0
    }

    /// 设置背景
    pub fn set_background(&mut self, path: String) {
        self.current_background = Some(path);
    }

    /// 清除背景
    pub fn clear_background(&mut self) {
        self.current_background = None;
    }

    /// 显示角色
    ///
    /// 注意：动画效果由 AnimationSystem 管理，这里只创建角色数据。
    pub fn show_character(&mut self, alias: String, texture_path: String, position: Position) {
        let z_order = self.visible_characters.len() as i32;

        self.visible_characters.insert(
            alias,
            CharacterSprite {
                texture_path,
                position,
                z_order,
                fading_out: false,
            },
        );
    }

    /// 隐藏角色（立即移除）
    pub fn hide_character(&mut self, alias: &str) {
        self.visible_characters.remove(alias);
    }

    /// 标记角色为淡出状态
    ///
    /// 角色会在动画完成后被 `remove_fading_out_characters` 移除。
    pub fn mark_character_fading_out(&mut self, alias: &str) {
        if let Some(character) = self.visible_characters.get_mut(alias) {
            character.fading_out = true;
        }
    }

    /// 移除所有标记为淡出且动画已完成的角色
    ///
    /// 应在动画系统更新后调用，传入已完成淡出的角色列表。
    pub fn remove_fading_out_characters(&mut self, completed_aliases: &[String]) {
        for alias in completed_aliases {
            if let Some(character) = self.visible_characters.get(alias) {
                if character.fading_out {
                    self.visible_characters.remove(alias);
                }
            }
        }
    }

    /// 隐藏所有角色
    pub fn hide_all_characters(&mut self) {
        self.visible_characters.clear();
    }

    /// 设置对话
    pub fn set_dialogue(&mut self, speaker: Option<String>, content: String) {
        self.dialogue = Some(DialogueState {
            speaker,
            content: content.clone(),
            visible_chars: content.chars().count(), // 默认显示全部
            is_complete: true,
        });
    }

    /// 开始打字机效果
    pub fn start_typewriter(&mut self, speaker: Option<String>, content: String) {
        self.dialogue = Some(DialogueState {
            speaker,
            content,
            visible_chars: 0,
            is_complete: false,
        });
    }

    /// 推进打字机效果（返回是否完成）
    pub fn advance_typewriter(&mut self) -> bool {
        if let Some(ref mut dialogue) = self.dialogue {
            let total_chars = dialogue.content.chars().count();
            if dialogue.visible_chars < total_chars {
                dialogue.visible_chars += 1;
                dialogue.is_complete = dialogue.visible_chars >= total_chars;
            }
            dialogue.is_complete
        } else {
            true
        }
    }

    /// 完成打字机效果（立即显示全部文本）
    pub fn complete_typewriter(&mut self) {
        if let Some(ref mut dialogue) = self.dialogue {
            dialogue.visible_chars = dialogue.content.chars().count();
            dialogue.is_complete = true;
        }
    }

    /// 清除对话
    pub fn clear_dialogue(&mut self) {
        self.dialogue = None;
    }

    /// 检查对话是否完成
    pub fn is_dialogue_complete(&self) -> bool {
        self.dialogue.as_ref().map_or(true, |d| d.is_complete)
    }

    /// 设置章节标记
    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 1.0,
            timer: 0.0,
        });
    }

    /// 清除章节标记
    pub fn clear_chapter_mark(&mut self) {
        self.chapter_mark = None;
    }

    /// 设置选择界面
    pub fn set_choices(&mut self, choices: Vec<ChoiceItem>, style: Option<String>) {
        self.choices = Some(ChoicesState {
            choices,
            style,
            selected_index: 0,
            hovered_index: None,
        });
    }

    /// 清除选择界面
    pub fn clear_choices(&mut self) {
        self.choices = None;
    }
}

/// 角色立绘状态
///
/// 存储角色立绘的基本信息。动画（透明度、位置等）由 AnimationSystem 统一管理。
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
}

/// 对话状态
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// 说话者名称（None 表示旁白）
    pub speaker: Option<String>,
    /// 对话内容
    pub content: String,
    /// 当前可见字符数（用于打字机效果）
    pub visible_chars: usize,
    /// 是否显示完成
    pub is_complete: bool,
}

/// 章节标记状态
#[derive(Debug, Clone)]
pub struct ChapterMarkState {
    /// 章节标题
    pub title: String,
    /// 章节级别
    pub level: u8,
    /// 透明度（用于淡入淡出）
    pub alpha: f32,
    /// 计时器（用于动画）
    pub timer: f32,
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
