//! # Renderer 模块
//!
//! 渲染系统，负责将游戏状态渲染到屏幕上。
//!
//! ## 渲染层顺序
//!
//! 1. 背景层（Background）
//! 2. 角色层（Characters）
//! 3. UI 层（对话框、选项等）
//! 4. 覆盖层（章节标记、过渡效果等）

use macroquad::prelude::*;
use tracing::warn;
use vn_runtime::command::Position;

use crate::manifest::Manifest;
use crate::resources::ResourceManager;

pub mod animation;
pub mod background_transition;
pub mod character_animation;
mod image_dissolve;
pub mod render_state;
pub mod scene_transition;
mod text_renderer;
mod transition;

pub use animation::{
    Animation, AnimationEvent, AnimationId, AnimationState, AnimationSystem, AnimationTarget,
    EasingFunction, Transform, Vec2 as AnimVec2,
};
// Trait-based 动画系统 API
pub use animation::{
    AnimPropertyKey, Animatable, ObjectId, PropertyAccessor, SimplePropertyAccessor,
};
pub use background_transition::{AnimatableBackgroundTransition, BackgroundTransitionData};
pub use character_animation::{AnimatableCharacter, CharacterAnimData};
pub use image_dissolve::ImageDissolve;
pub use render_state::{CharacterSprite, ChoiceItem, ChoicesState, DialogueState, RenderState};
pub use scene_transition::{
    AnimatableSceneTransition, SceneTransitionManager, SceneTransitionPhase, SceneTransitionType,
};
pub use text_renderer::TextRenderer;
pub use transition::{TransitionManager, TransitionPhase, TransitionType};

/// 渲染器
///
/// 负责将 RenderState 渲染到屏幕上。
pub struct Renderer {
    /// 文本渲染器
    pub text_renderer: TextRenderer,
    /// 过渡效果管理器（用于背景 dissolve 过渡）
    pub transition: TransitionManager,
    /// 场景过渡管理器（用于 changeScene 的 Fade/FadeWhite/Rule 效果）
    pub scene_transition: SceneTransitionManager,
    /// ImageDissolve 效果器（用于 Rule 过渡）
    pub image_dissolve: ImageDissolve,
    /// 设计分辨率（用于坐标计算）
    design_width: f32,
    design_height: f32,
    /// 旧背景路径（用于过渡效果）
    old_background: Option<String>,
}

impl Renderer {
    /// 创建新的渲染器
    pub fn new(design_width: f32, design_height: f32) -> Self {
        Self {
            text_renderer: TextRenderer::new(),
            transition: TransitionManager::new(),
            scene_transition: SceneTransitionManager::new(),
            image_dissolve: ImageDissolve::new(),
            design_width,
            design_height,
            old_background: None,
        }
    }

    /// 异步初始化（加载字体等资源）
    pub async fn init(&mut self, font_path: &str) -> Result<(), String> {
        // 初始化字体
        self.text_renderer.load_font(font_path).await?;

        // 初始化 ImageDissolve shader
        if let Err(e) = self.image_dissolve.init() {
            warn!(error = %e, "ImageDissolve shader 初始化失败，将使用降级方案");
            // 不返回错误，因为有降级方案
        }

        Ok(())
    }

    /// 渲染完整画面
    ///
    /// 纹理从 `resource_manager` 缓存中获取（使用 `peek_texture` 不更新 LRU）。
    pub fn render(
        &mut self,
        state: &RenderState,
        resource_manager: &ResourceManager,
        manifest: &Manifest,
    ) {
        // 清空屏幕
        clear_background(BLACK);

        // 1. 渲染背景（带过渡效果）
        self.render_background_with_transition(state, resource_manager);

        // 2. 渲染角色（从角色自身的动画状态获取变换）
        self.render_characters(state, resource_manager, manifest);

        // 3-5. 渲染 UI 层（仅当 ui_visible 为 true）
        if state.ui_visible {
            // 获取 UI 透明度（用于 changeScene 后的 UI 淡入效果）
            let ui_alpha = self.get_scene_transition_ui_alpha();

            // 3. 渲染对话框
            self.render_dialogue_with_alpha(state, ui_alpha);

            // 4. 渲染选择界面
            self.render_choices_with_alpha(state, ui_alpha);

            // 5. 渲染章节标记
            self.render_chapter_mark_with_alpha(state, ui_alpha);
        }

        // 6. 渲染场景遮罩（changeScene 的 Fade/FadeWhite/Rule 效果）
        self.render_scene_mask(state, resource_manager);
    }

    /// 更新过渡效果
    pub fn update_transition(&mut self, dt: f32) -> bool {
        self.transition.update(dt)
    }

    /// 开始背景过渡
    pub fn start_background_transition(
        &mut self,
        old_bg: Option<String>,
        transition: Option<&vn_runtime::command::Transition>,
    ) {
        self.old_background = old_bg;

        if let Some(trans) = transition {
            self.transition.start_from_command(trans);
        } else {
            // 默认使用短暂的 dissolve
            self.transition.start(TransitionType::Dissolve, 0.2);
        }
    }

    /// 跳过当前过渡效果
    pub fn skip_transition(&mut self) {
        self.transition.skip();
        self.old_background = None;
    }

    // ========== 场景过渡管理 (基于动画系统) ==========

    /// 开始 Fade（黑屏）场景过渡
    ///
    /// # 参数
    /// - `duration`: 每个淡入/淡出阶段的时长（秒）
    /// - `pending_background`: 待切换的新背景路径
    pub fn start_scene_fade(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade(duration, pending_background);
    }

    /// 开始 FadeWhite（白屏）场景过渡
    pub fn start_scene_fade_white(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade_white(duration, pending_background);
    }

    /// 开始 Rule（图片遮罩）场景过渡
    pub fn start_scene_rule(
        &mut self,
        duration: f32,
        pending_background: String,
        mask_path: String,
        reversed: bool,
    ) {
        self.scene_transition
            .start_rule(duration, pending_background, mask_path, reversed);
    }

    /// 更新场景过渡
    ///
    /// # 返回
    /// - `true`: 过渡仍在进行中
    /// - `false`: 过渡已完成或处于空闲状态
    pub fn update_scene_transition(&mut self, dt: f32) -> bool {
        self.scene_transition.update(dt)
    }

    /// 检查场景过渡是否处于中间点（可以进行背景切换）
    pub fn is_scene_transition_at_midpoint(&self) -> bool {
        self.scene_transition.is_at_midpoint()
    }

    /// 获取并清除待切换的背景
    pub fn take_pending_background(&mut self) -> Option<String> {
        self.scene_transition.take_pending_background()
    }

    /// 获取场景过渡的 UI 透明度
    pub fn get_scene_transition_ui_alpha(&self) -> f32 {
        if self.scene_transition.is_active() {
            self.scene_transition.ui_alpha()
        } else {
            1.0
        }
    }

    /// 跳过场景过渡的当前阶段
    pub fn skip_scene_transition_phase(&mut self) {
        self.scene_transition.skip_current_phase();
    }

    /// 检查场景过渡是否正在进行
    pub fn is_scene_transition_active(&self) -> bool {
        self.scene_transition.is_active()
    }

    /// 检查场景过渡是否正在 UI 淡入阶段
    pub fn is_scene_transition_ui_fading_in(&self) -> bool {
        self.scene_transition.is_ui_fading_in()
    }

    /// 渲染背景（带过渡效果）
    fn render_background_with_transition(
        &self,
        state: &RenderState,
        resource_manager: &ResourceManager,
    ) {
        // 渲染旧背景（如果正在过渡中）
        if self.transition.is_active() {
            if let Some(ref old_bg_path) = self.old_background {
                if let Some(texture) = resource_manager.peek_texture(old_bg_path) {
                    let alpha = self.transition.old_content_alpha();
                    if alpha > 0.0 {
                        self.draw_texture_fit_with_alpha(&texture, DrawMode::Cover, alpha);
                    }
                }
            }
        }

        // 渲染新背景
        if let Some(ref bg_path) = state.current_background {
            if let Some(texture) = resource_manager.peek_texture(bg_path) {
                let alpha = self.transition.new_content_alpha();
                self.draw_texture_fit_with_alpha(&texture, DrawMode::Cover, alpha);
            }
        }
    }

    /// 渲染角色立绘
    ///
    /// 使用 manifest 配置的 anchor + pre_scale + preset 进行布局：
    /// 1. 从 manifest 获取立绘组的 anchor 和 pre_scale
    /// 2. 从 manifest 获取站位预设的 x, y, scale
    /// 3. 从角色的 AnimatableCharacter 获取动画变换（透明度等）
    /// 4. 计算最终位置和尺寸
    fn render_characters(
        &self,
        state: &RenderState,
        resource_manager: &ResourceManager,
        manifest: &Manifest,
    ) {
        // 按 z_order 排序渲染
        let mut characters: Vec<_> = state.visible_characters.iter().collect();
        characters.sort_by_key(|(_, c)| c.z_order);

        let screen_w = screen_width();
        let screen_h = screen_height();
        let base_scale = self.get_scale_factor();

        for (_alias, character) in characters {
            // 从 ResourceManager 缓存获取纹理
            if let Some(texture) = resource_manager.peek_texture(&character.texture_path) {
                // 获取立绘组配置
                let group_config = manifest.get_group_config(&character.texture_path);

                // 获取站位预设
                let position_name = Self::position_to_preset_name(character.position);
                let preset = manifest.get_preset(&position_name);

                // 从角色动画状态获取属性
                let alpha = character.anim.alpha();
                let (position_x, position_y) = character.anim.position();
                let (scale_x, _scale_y) = character.anim.scale();

                // 计算最终缩放：基础缩放 * 预处理缩放 * 站位缩放 * 动画缩放
                let final_scale = base_scale * group_config.pre_scale * preset.scale * scale_x;

                // 计算渲染尺寸
                let dest_w = texture.width() * final_scale;
                let dest_h = texture.height() * final_scale;

                // 计算屏幕目标点（预设位置 + 动画位置偏移）
                let target_x = screen_w * preset.x + position_x;
                let target_y = screen_h * preset.y + position_y;

                // 计算立绘锚点在纹理中的像素位置
                let anchor_px_x = dest_w * group_config.anchor.x;
                let anchor_px_y = dest_h * group_config.anchor.y;

                // 最终位置：目标点 - 锚点偏移
                let x = target_x - anchor_px_x;
                let y = target_y - anchor_px_y;

                // 应用透明度
                let color = Color::new(1.0, 1.0, 1.0, alpha);

                draw_texture_ex(
                    &texture,
                    x,
                    y,
                    color,
                    DrawTextureParams {
                        dest_size: Some(vec2(dest_w, dest_h)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// 将 Position 枚举转换为预设名称
    fn position_to_preset_name(position: Position) -> &'static str {
        match position {
            Position::Left => "left",
            Position::NearLeft => "nearleft",
            Position::FarLeft => "farleft",
            Position::Center => "center",
            Position::NearMiddle => "nearmiddle",
            Position::FarMiddle => "farmiddle",
            Position::Right => "right",
            Position::NearRight => "nearright",
            Position::FarRight => "farright",
        }
    }

    /// 渲染对话框（带透明度）
    fn render_dialogue_with_alpha(&self, state: &RenderState, alpha: f32) {
        if let Some(ref dialogue) = state.dialogue {
            self.text_renderer.render_dialogue_box_with_alpha(
                dialogue.speaker.as_deref(),
                &dialogue.content,
                dialogue.visible_chars,
                alpha,
            );
        }
    }

    /// 渲染章节标记（带透明度）
    fn render_chapter_mark_with_alpha(&self, state: &RenderState, alpha: f32) {
        if let Some(ref chapter) = state.chapter_mark {
            // 章节标记有自己的 alpha，与 UI alpha 相乘
            let effective_alpha = chapter.alpha * alpha;
            self.text_renderer
                .render_chapter_title(&chapter.title, effective_alpha);
        }
    }

    /// 渲染选择界面（带透明度）
    fn render_choices_with_alpha(&self, state: &RenderState, alpha: f32) {
        if let Some(ref choices_state) = state.choices {
            self.text_renderer.render_choices_with_alpha(
                &choices_state.choices,
                choices_state.selected_index,
                choices_state.hovered_index,
                alpha,
            );
        }
    }

    /// 渲染场景遮罩（用于 changeScene 的 Fade/FadeWhite/Rule 效果）
    ///
    /// 基于 AnimationSystem 驱动 shader 变量实现场景过渡。
    fn render_scene_mask(&mut self, state: &RenderState, resource_manager: &ResourceManager) {
        // 遮罩已完成，不需要渲染
        if self.scene_transition.is_mask_complete() {
            return;
        }

        match self.scene_transition.transition_type() {
            Some(SceneTransitionType::Fade) => {
                // 绘制黑色遮罩
                let alpha = self.scene_transition.mask_alpha();
                if alpha > 0.0 {
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        Color::new(0.0, 0.0, 0.0, alpha),
                    );
                }
            }
            Some(SceneTransitionType::FadeWhite) => {
                // 绘制白色遮罩
                let alpha = self.scene_transition.mask_alpha();
                if alpha > 0.0 {
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        Color::new(1.0, 1.0, 1.0, alpha),
                    );
                }
            }
            Some(SceneTransitionType::Rule {
                mask_path,
                reversed,
            }) => {
                // Rule 遮罩：使用 ImageDissolve shader 实现
                let mask_texture = resource_manager
                    .peek_texture(mask_path)
                    .unwrap_or_else(|| panic!("Rule 遮罩纹理未找到: {}", mask_path));

                if !self.image_dissolve.is_initialized() {
                    panic!("ImageDissolve shader 未初始化，无法使用 rule 过渡");
                }

                let progress = self.scene_transition.progress();
                let black_texture = resource_manager
                    .peek_texture("backgrounds/black.png")
                    .unwrap_or_else(|| panic!("缺少黑色背景纹理: backgrounds/black.png"));

                let phase = self.scene_transition.phase();
                let reversed = *reversed;

                match phase {
                    SceneTransitionPhase::FadeIn => {
                        // phase FadeIn: 旧背景 → 黑屏
                        let old_bg_path = state
                            .current_background
                            .as_ref()
                            .unwrap_or_else(|| panic!("Rule 遮罩缺少当前背景"));
                        let old_bg_texture = resource_manager
                            .peek_texture(old_bg_path)
                            .unwrap_or_else(|| panic!("Rule 旧背景纹理未找到: {}", old_bg_path));

                        let (dest_w, dest_h, x, y) =
                            self.calculate_draw_rect(&old_bg_texture, DrawMode::Cover);
                        // 从旧背景溶解到黑色
                        self.image_dissolve.draw(
                            &black_texture,  // 目标：黑色
                            &old_bg_texture, // 源：旧背景
                            &mask_texture,
                            progress,
                            reversed,
                            (x, y, dest_w, dest_h),
                        );
                    }
                    SceneTransitionPhase::Blackout => {
                        // phase Blackout: 黑屏停顿 - 绘制纯黑屏
                        draw_rectangle(
                            0.0,
                            0.0,
                            screen_width(),
                            screen_height(),
                            Color::new(0.0, 0.0, 0.0, 1.0),
                        );
                    }
                    SceneTransitionPhase::FadeOut => {
                        // phase FadeOut: 黑屏 → 新背景
                        let new_bg_path = state
                            .current_background
                            .as_ref()
                            .unwrap_or_else(|| panic!("Rule 遮罩缺少新背景（current_background）"));
                        let new_bg_texture = resource_manager
                            .peek_texture(new_bg_path)
                            .unwrap_or_else(|| panic!("Rule 新背景纹理未找到: {}", new_bg_path));

                        let (dest_w, dest_h, x, y) =
                            self.calculate_draw_rect(&new_bg_texture, DrawMode::Cover);
                        // 从黑色溶解到新背景（反向溶解）
                        self.image_dissolve.draw(
                            &new_bg_texture, // 目标：新背景
                            &black_texture,  // 源：黑色
                            &mask_texture,
                            progress,
                            !reversed, // 反向，让效果对称
                            (x, y, dest_w, dest_h),
                        );
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }

    /// 获取选择框的矩形区域
    pub fn get_choice_rects(&self, choice_count: usize) -> Vec<(f32, f32, f32, f32)> {
        self.text_renderer.get_choice_rects(choice_count)
    }

    /// 获取当前缩放因子
    fn get_scale_factor(&self) -> f32 {
        let scale_x = screen_width() / self.design_width;
        let scale_y = screen_height() / self.design_height;
        scale_x.min(scale_y)
    }

    /// 绘制纹理以适应屏幕（带透明度）
    fn draw_texture_fit_with_alpha(&self, texture: &Texture2D, mode: DrawMode, alpha: f32) {
        let (dest_w, dest_h, x, y) = self.calculate_draw_rect(texture, mode);

        let color = Color::new(1.0, 1.0, 1.0, alpha);
        draw_texture_ex(
            texture,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dest_w, dest_h)),
                ..Default::default()
            },
        );
    }

    /// 计算纹理绘制矩形（dest_w, dest_h, x, y）
    fn calculate_draw_rect(&self, texture: &Texture2D, mode: DrawMode) -> (f32, f32, f32, f32) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let tex_w = texture.width();
        let tex_h = texture.height();

        match mode {
            DrawMode::Cover => {
                // 覆盖模式：填满屏幕，可能裁剪
                let scale = (screen_w / tex_w).max(screen_h / tex_h);
                let dest_w = tex_w * scale;
                let dest_h = tex_h * scale;
                let x = (screen_w - dest_w) / 2.0;
                let y = (screen_h - dest_h) / 2.0;
                (dest_w, dest_h, x, y)
            }
            DrawMode::Contain => {
                // 包含模式：完整显示，可能有黑边
                let scale = (screen_w / tex_w).min(screen_h / tex_h);
                let dest_w = tex_w * scale;
                let dest_h = tex_h * scale;
                let x = (screen_w - dest_w) / 2.0;
                let y = (screen_h - dest_h) / 2.0;
                (dest_w, dest_h, x, y)
            }
            DrawMode::Stretch => {
                // 拉伸模式：完全填满
                (screen_w, screen_h, 0.0, 0.0)
            }
        }
    }
}

/// 绘制模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    /// 覆盖模式：填满屏幕，保持宽高比，可能裁剪
    Cover,
    /// 包含模式：完整显示，保持宽高比，可能有黑边
    Contain,
    /// 拉伸模式：完全填满，不保持宽高比
    Stretch,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}
