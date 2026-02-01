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
use std::collections::HashMap;
use vn_runtime::command::Position;

pub mod render_state;
mod text_renderer;
mod transition;

pub use render_state::{RenderState, CharacterSprite, DialogueState, ChoiceItem, ChoicesState};
pub use text_renderer::TextRenderer;
pub use transition::{TransitionManager, TransitionType, TransitionPhase};

/// 渲染器
///
/// 负责将 RenderState 渲染到屏幕上。
pub struct Renderer {
    /// 文本渲染器
    pub text_renderer: TextRenderer,
    /// 过渡效果管理器
    pub transition: TransitionManager,
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
            design_width,
            design_height,
            old_background: None,
        }
    }

    /// 异步初始化（加载字体等资源）
    pub async fn init(&mut self, font_path: &str) -> Result<(), String> {
        self.text_renderer.load_font(font_path).await
    }

    /// 渲染完整画面
    pub fn render(&self, state: &RenderState, textures: &HashMap<String, Texture2D>) {
        // 清空屏幕
        clear_background(BLACK);

        // 1. 渲染背景（带过渡效果）
        self.render_background_with_transition(state, textures);

        // 2. 渲染角色
        self.render_characters(state, textures);

        // 3. 渲染对话框
        self.render_dialogue(state);

        // 4. 渲染选择界面
        self.render_choices(state);

        // 5. 渲染章节标记
        self.render_chapter_mark(state);

        // 6. 渲染过渡效果遮罩（Fade 效果）
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
    fn render_background_with_transition(&self, state: &RenderState, textures: &HashMap<String, Texture2D>) {
        // 渲染旧背景（如果正在过渡中）
        if self.transition.is_active() {
            if let Some(ref old_bg_path) = self.old_background {
                if let Some(texture) = textures.get(old_bg_path) {
                    let alpha = self.transition.old_content_alpha();
                    if alpha > 0.0 {
                        self.draw_texture_fit_with_alpha(texture, DrawMode::Cover, alpha);
                    }
                }
            }
        }

        // 渲染新背景
        if let Some(ref bg_path) = state.current_background {
            if let Some(texture) = textures.get(bg_path) {
                let alpha = self.transition.new_content_alpha();
                self.draw_texture_fit_with_alpha(texture, DrawMode::Cover, alpha);
            }
        }
    }

    /// 渲染背景（不带过渡效果，保留兼容）
    #[allow(dead_code)]
    fn render_background(&self, state: &RenderState, textures: &HashMap<String, Texture2D>) {
        if let Some(ref bg_path) = state.current_background {
            if let Some(texture) = textures.get(bg_path) {
                self.draw_texture_fit(texture, DrawMode::Cover);
            }
        }
    }

    /// 渲染角色立绘
    fn render_characters(&self, state: &RenderState, textures: &HashMap<String, Texture2D>) {
        // 按 z_order 排序渲染
        let mut characters: Vec<_> = state.visible_characters.values().collect();
        characters.sort_by_key(|c| c.z_order);

        for character in characters {
            if let Some(texture) = textures.get(&character.texture_path) {
                let (x, y) = self.position_to_screen_coords(character.position, texture);
                
                // 应用透明度
                let color = Color::new(1.0, 1.0, 1.0, character.alpha);
                
                draw_texture_ex(
                    texture,
                    x,
                    y,
                    color,
                    DrawTextureParams {
                        dest_size: Some(self.scale_character_size(texture)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// 渲染对话框
    fn render_dialogue(&self, state: &RenderState) {
        if let Some(ref dialogue) = state.dialogue {
            self.text_renderer.render_dialogue_box(
                dialogue.speaker.as_deref(),
                &dialogue.content,
                dialogue.visible_chars,
            );
        }
    }

    /// 渲染章节标记
    fn render_chapter_mark(&self, state: &RenderState) {
        if let Some(ref chapter) = state.chapter_mark {
            self.text_renderer.render_chapter_title(
                &chapter.title,
                chapter.alpha,
            );
        }
    }

    /// 渲染选择界面
    fn render_choices(&self, state: &RenderState) {
        if let Some(ref choices_state) = state.choices {
            self.text_renderer.render_choices(
                &choices_state.choices,
                choices_state.selected_index,
                choices_state.hovered_index,
            );
        }
    }

    /// 获取选择框的矩形区域
    pub fn get_choice_rects(&self, choice_count: usize) -> Vec<(f32, f32, f32, f32)> {
        self.text_renderer.get_choice_rects(choice_count)
    }

    /// 将 Position 枚举转换为屏幕坐标
    fn position_to_screen_coords(&self, position: Position, texture: &Texture2D) -> (f32, f32) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let scale = self.get_scale_factor();
        let char_w = texture.width() * scale * 0.8; // 角色缩放比例
        let char_h = texture.height() * scale * 0.8;

        // 角色底部对齐屏幕底部
        let y = screen_h - char_h;

        let x = match position {
            Position::Left => screen_w * 0.1,
            Position::NearLeft => screen_w * 0.2,
            Position::FarLeft => screen_w * 0.05,
            Position::Center => (screen_w - char_w) / 2.0,
            Position::NearMiddle => screen_w * 0.35,
            Position::FarMiddle => screen_w * 0.4,
            Position::Right => screen_w * 0.9 - char_w,
            Position::NearRight => screen_w * 0.8 - char_w,
            Position::FarRight => screen_w * 0.95 - char_w,
        };

        (x, y)
    }

    /// 计算角色缩放后的尺寸
    fn scale_character_size(&self, texture: &Texture2D) -> Vec2 {
        let scale = self.get_scale_factor() * 0.8; // 角色相对于背景稍小
        vec2(texture.width() * scale, texture.height() * scale)
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
        let screen_w = screen_width();
        let screen_h = screen_height();
        let tex_w = texture.width();
        let tex_h = texture.height();

        let (dest_w, dest_h, x, y) = match mode {
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
        };

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
