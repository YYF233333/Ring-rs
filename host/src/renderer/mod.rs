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
use vn_runtime::command::Position;

use crate::resources::ResourceManager;
use crate::manifest::Manifest;

pub mod render_state;
mod text_renderer;
mod transition;
mod image_dissolve;

pub use render_state::{RenderState, CharacterSprite, DialogueState, ChoiceItem, ChoicesState, SceneMaskState, SceneMaskType};
pub use text_renderer::TextRenderer;
pub use transition::{TransitionManager, TransitionType, TransitionPhase};
pub use image_dissolve::ImageDissolve;

/// 渲染器
///
/// 负责将 RenderState 渲染到屏幕上。
pub struct Renderer {
    /// 文本渲染器
    pub text_renderer: TextRenderer,
    /// 过渡效果管理器
    pub transition: TransitionManager,
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
            eprintln!("⚠️ ImageDissolve shader 初始化失败，将使用降级方案: {}", e);
            // 不返回错误，因为有降级方案
        }
        
        Ok(())
    }

    /// 渲染完整画面
    ///
    /// 纹理从 `resource_manager` 缓存中获取（使用 `peek_texture` 不更新 LRU）。
    pub fn render(&mut self, state: &RenderState, resource_manager: &ResourceManager, manifest: &Manifest) {
        // 清空屏幕
        clear_background(BLACK);

        // 1. 渲染背景（带过渡效果）
        self.render_background_with_transition(state, resource_manager);

        // 2. 渲染角色
        self.render_characters(state, resource_manager, manifest);

        // 3-5. 渲染 UI 层（仅当 ui_visible 为 true）
        if state.ui_visible {
            // 获取 UI 透明度（用于 changeScene 后的 UI 淡入效果）
            let ui_alpha = state.get_effective_ui_alpha();

            // 3. 渲染对话框
            self.render_dialogue_with_alpha(state, ui_alpha);

            // 4. 渲染选择界面
            self.render_choices_with_alpha(state, ui_alpha);

            // 5. 渲染章节标记
            self.render_chapter_mark_with_alpha(state, ui_alpha);
        }

        // 6. 渲染场景遮罩（changeScene 的 Fade/FadeWhite/Rule 效果）
        self.render_scene_mask(state, resource_manager);

        // 7. 渲染过渡效果遮罩（普通 Fade 效果）
        self.transition.render_overlay();
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

    /// 渲染背景（带过渡效果）
    fn render_background_with_transition(&self, state: &RenderState, resource_manager: &ResourceManager) {
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

    /// 渲染背景（不带过渡效果，保留兼容）
    #[allow(dead_code)]
    fn render_background(&self, state: &RenderState, resource_manager: &ResourceManager) {
        if let Some(ref bg_path) = state.current_background {
            if let Some(texture) = resource_manager.peek_texture(bg_path) {
                self.draw_texture_fit(&texture, DrawMode::Cover);
            }
        }
    }

    /// 渲染角色立绘
    ///
    /// 使用 manifest 配置的 anchor + pre_scale + preset 进行布局：
    /// 1. 从 manifest 获取立绘组的 anchor 和 pre_scale
    /// 2. 从 manifest 获取站位预设的 x, y, scale
    /// 3. 计算最终位置和尺寸
    fn render_characters(&self, state: &RenderState, resource_manager: &ResourceManager, manifest: &Manifest) {
        // 按 z_order 排序渲染
        let mut characters: Vec<_> = state.visible_characters.values().collect();
        characters.sort_by_key(|c| c.z_order);

        let screen_w = screen_width();
        let screen_h = screen_height();
        let base_scale = self.get_scale_factor();

        for character in characters {
            // 从 ResourceManager 缓存获取纹理
            if let Some(texture) = resource_manager.peek_texture(&character.texture_path) {
                // 获取立绘组配置
                let group_config = manifest.get_group_config(&character.texture_path);
                
                // 获取站位预设
                let position_name = Self::position_to_preset_name(character.position);
                let preset = manifest.get_preset(&position_name);
                
                // 计算最终缩放：基础缩放 * 预处理缩放 * 站位缩放
                let final_scale = base_scale * group_config.pre_scale * preset.scale;
                
                // 计算渲染尺寸
                let dest_w = texture.width() * final_scale;
                let dest_h = texture.height() * final_scale;
                
                // 计算屏幕目标点（预设位置）
                let target_x = screen_w * preset.x;
                let target_y = screen_h * preset.y;
                
                // 计算立绘锚点在纹理中的像素位置
                let anchor_px_x = dest_w * group_config.anchor.x;
                let anchor_px_y = dest_h * group_config.anchor.y;
                
                // 最终位置：目标点 - 锚点偏移
                let x = target_x - anchor_px_x;
                let y = target_y - anchor_px_y;
                
                // 应用透明度
                let color = Color::new(1.0, 1.0, 1.0, character.alpha);
                
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
            self.text_renderer.render_chapter_title(
                &chapter.title,
                effective_alpha,
            );
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
    fn render_scene_mask(&mut self, state: &RenderState, resource_manager: &ResourceManager) {
        if let Some(ref mask) = state.scene_mask {
            // 遮罩已完成，不需要渲染
            if mask.is_mask_complete() {
                return;
            }

            match &mask.mask_type {
                SceneMaskType::SolidBlack => {
                    // 绘制黑色遮罩
                    if mask.alpha > 0.0 {
                        draw_rectangle(
                            0.0, 0.0,
                            screen_width(), screen_height(),
                            Color::new(0.0, 0.0, 0.0, mask.alpha),
                        );
                    }
                }
                SceneMaskType::SolidWhite => {
                    // 绘制白色遮罩
                    if mask.alpha > 0.0 {
                        draw_rectangle(
                            0.0, 0.0,
                            screen_width(), screen_height(),
                            Color::new(1.0, 1.0, 1.0, mask.alpha),
                        );
                    }
                }
                SceneMaskType::Rule { mask_path, reversed } => {
                    // Rule 遮罩：使用 ImageDissolve shader 实现
                    // 三阶段：phase 0: 旧背景→黑屏，phase 1: 黑屏停顿，phase 2: 黑屏→新背景
                    let mask_texture = resource_manager.peek_texture(mask_path)
                        .unwrap_or_else(|| panic!("Rule 遮罩纹理未找到: {}", mask_path));

                    if !self.image_dissolve.is_initialized() {
                        panic!("ImageDissolve shader 未初始化，无法使用 rule 过渡");
                    }

                    let progress = mask.dissolve_progress;
                    let black_texture = resource_manager.peek_texture("backgrounds/black.png")
                        .unwrap_or_else(|| panic!("缺少黑色背景纹理: backgrounds/black.png"));

                    match mask.phase {
                        0 => {
                            // phase 0: 旧背景 → 黑屏
                            let old_bg_path = state.current_background.as_ref()
                                .unwrap_or_else(|| panic!("Rule 遮罩缺少当前背景"));
                            let old_bg_texture = resource_manager.peek_texture(old_bg_path)
                                .unwrap_or_else(|| panic!("Rule 旧背景纹理未找到: {}", old_bg_path));

                            let (dest_w, dest_h, x, y) =
                                self.calculate_draw_rect(&old_bg_texture, DrawMode::Cover);
                            // 从旧背景溶解到黑色
                            self.image_dissolve.draw(
                                &black_texture,      // 目标：黑色
                                &old_bg_texture,     // 源：旧背景
                                &mask_texture,
                                progress,
                                *reversed,
                                (x, y, dest_w, dest_h),
                            );
                        }
                        1 => {
                            // phase 1: 黑屏停顿 - 绘制纯黑屏
                            draw_rectangle(
                                0.0, 0.0,
                                screen_width(), screen_height(),
                                Color::new(0.0, 0.0, 0.0, 1.0),
                            );
                        }
                        2 => {
                            // phase 2: 黑屏 → 新背景
                            // 注意：此时 pending_background 已在 is_at_midpoint() 时被 take 并设置到 current_background
                            let new_bg_path = state.current_background.as_ref()
                                .unwrap_or_else(|| panic!("Rule 遮罩缺少新背景（current_background）"));
                            let new_bg_texture = resource_manager.peek_texture(new_bg_path)
                                .unwrap_or_else(|| panic!("Rule 新背景纹理未找到: {}", new_bg_path));

                            let (dest_w, dest_h, x, y) =
                                self.calculate_draw_rect(&new_bg_texture, DrawMode::Cover);
                            // 从黑色溶解到新背景（反向溶解）
                            self.image_dissolve.draw(
                                &new_bg_texture,     // 目标：新背景
                                &black_texture,      // 源：黑色
                                &mask_texture,
                                progress,
                                !*reversed,         // 反向，让效果对称
                                (x, y, dest_w, dest_h),
                            );
                        }
                        _ => {}
                    }
                }
            }
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

    /// 绘制纹理以适应屏幕
    fn draw_texture_fit(&self, texture: &Texture2D, mode: DrawMode) {
        self.draw_texture_fit_with_alpha(texture, mode, 1.0);
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
