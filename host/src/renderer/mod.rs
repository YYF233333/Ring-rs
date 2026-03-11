//! # Renderer 模块
//!
//! 渲染系统，负责将游戏状态转换为绘制命令。
//!
//! ## 渲染层顺序
//!
//! 1. 背景层（Background）
//! 2. 角色层（Characters）
//! 3. 场景效果层（dim/blur 遮罩）
//! 4. 场景过渡遮罩层（Fade/FadeWhite/Rule）
//! 5. UI 层（对话框、选项 — 由 egui 负责）

use crate::manifest::Manifest;
use crate::rendering_types::DrawCommand;
use crate::resources::ResourceManager;
use vn_runtime::command::Position;

// ── 渲染常量 ────────────────────────────────────────────────────────────────

/// 场景效果（dim/blur）可见性阈值
const EFFECT_THRESHOLD: f32 = 0.01;
/// 模糊近似系数：blur_amount * 此值 = 叠加 alpha
const BLUR_APPROX_FACTOR: f32 = 0.3;
/// 震动效果频率（弧度/秒）
const SHAKE_FREQUENCY: f32 = 30.0;
/// 默认设计分辨率
const DEFAULT_DESIGN_WIDTH: f32 = 1920.0;
const DEFAULT_DESIGN_HEIGHT: f32 = 1080.0;

pub mod animation;
pub mod background_transition;
pub mod character_animation;
mod draw_commands;
pub mod effects;
mod image_dissolve;
pub mod render_state;
mod scene_effects;
pub mod scene_transition;
mod text_renderer;
mod transition;

pub use animation::{AnimationSystem, ObjectId};
pub use character_animation::AnimatableCharacter;
pub use render_state::{ChoiceItem, RenderState, SceneEffectState, TitleCardState};
pub use scene_transition::SceneTransitionType;

use image_dissolve::ImageDissolve;
use scene_transition::SceneTransitionManager;
use text_renderer::TextRenderer;
use transition::{TransitionManager, TransitionType};

/// 渲染器
///
/// 负责将 RenderState 转换为 DrawCommand 列表。
pub struct Renderer {
    /// 文本渲染器（保留用于兼容，Phase 4 将迁移到 egui）
    pub text_renderer: TextRenderer,
    /// 过渡效果管理器（用于背景 dissolve 过渡）
    pub transition: TransitionManager,
    /// 场景过渡管理器（用于 changeScene 的 Fade/FadeWhite/Rule 效果）
    pub scene_transition: SceneTransitionManager,
    /// ImageDissolve 效果器（用于 Rule 过渡，Phase 2 迁移到 WGSL）
    pub image_dissolve: ImageDissolve,
    /// 设计分辨率
    design_width: f32,
    design_height: f32,
    /// 当前屏幕尺寸（像素）
    screen_width: f32,
    screen_height: f32,
    /// 旧背景路径（用于过渡效果）
    old_background: Option<String>,
    /// 场景震动效果状态
    shake: ShakeState,
    /// 场景模糊过渡状态
    blur_transition: BlurTransitionState,
}

/// 震动效果状态
#[derive(Debug, Clone, Default)]
struct ShakeState {
    active: bool,
    amplitude_x: f32,
    amplitude_y: f32,
    elapsed: f32,
    duration: f32,
}

/// 模糊过渡状态
#[derive(Debug, Clone, Default)]
struct BlurTransitionState {
    active: bool,
    from: f32,
    to: f32,
    elapsed: f32,
    duration: f32,
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
            screen_width: design_width,
            screen_height: design_height,
            old_background: None,
            shake: ShakeState::default(),
            blur_transition: BlurTransitionState::default(),
        }
    }

    /// 更新屏幕尺寸（窗口大小变化时调用）
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// 获取当前屏幕宽度
    pub fn screen_width(&self) -> f32 {
        self.screen_width
    }

    /// 获取当前屏幕高度
    pub fn screen_height(&self) -> f32 {
        self.screen_height
    }

    /// 异步初始化（Phase 2 前为空操作）
    ///
    /// 保留签名用于兼容；ImageDissolve shader 将在 Phase 2 迁移到 WGSL。
    pub fn init(&mut self) {
        // ImageDissolve WGSL shader 将在 Phase 2 初始化
        // TextRenderer 字体加载已迁移到 egui 的 FontDefinitions
    }

    /// 生成完整画面的绘制命令
    ///
    /// 返回按层级排序的 DrawCommand 列表，由 WgpuBackend 消费。
    pub fn build_draw_commands(
        &self,
        state: &RenderState,
        resource_manager: &ResourceManager,
        manifest: &Manifest,
    ) -> Vec<DrawCommand> {
        let mut commands = Vec::with_capacity(16);
        let sw = self.screen_width;
        let sh = self.screen_height;

        let (shake_x, shake_y) = self.current_shake_offset();

        self.build_background_commands(&mut commands, state, resource_manager, shake_x, shake_y);

        self.build_character_commands(
            &mut commands,
            state,
            resource_manager,
            manifest,
            shake_x,
            shake_y,
        );

        if state.scene_effect.dim_level > EFFECT_THRESHOLD {
            let alpha = state.scene_effect.dim_level.clamp(0.0, 1.0);
            commands.push(DrawCommand::Rect {
                x: 0.0,
                y: 0.0,
                width: sw,
                height: sh,
                color: [0.0, 0.0, 0.0, alpha],
            });
        }

        if state.scene_effect.blur_amount > EFFECT_THRESHOLD {
            let alpha = (state.scene_effect.blur_amount * BLUR_APPROX_FACTOR)
                .clamp(0.0, BLUR_APPROX_FACTOR);
            commands.push(DrawCommand::Rect {
                x: 0.0,
                y: 0.0,
                width: sw,
                height: sh,
                color: [1.0, 1.0, 1.0, alpha],
            });
        }

        self.build_scene_mask_commands(&mut commands, state, resource_manager);

        commands
    }
}

/// 将 Position 枚举转换为 manifest 预设名称
pub fn position_to_preset_name(position: Position) -> &'static str {
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
        Self::new(DEFAULT_DESIGN_WIDTH, DEFAULT_DESIGN_HEIGHT)
    }
}

#[cfg(test)]
mod headless_tests {
    use super::*;
    use crate::rendering_types::{NullTexture, NullTextureFactory, TextureContext};
    use crate::resources::ResourceManager;
    use std::sync::Arc;

    fn make_test_resource_manager() -> ResourceManager {
        let mut manager = ResourceManager::new("assets", 256);
        manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));
        manager
    }

    #[test]
    fn test_build_draw_commands_empty_state() {
        let renderer = Renderer::new(1920.0, 1080.0);
        let state = RenderState::default();
        let resource_manager = make_test_resource_manager();
        let manifest = Manifest::with_defaults();

        let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);
        assert!(
            cmds.is_empty(),
            "empty state should produce no draw commands"
        );
    }

    #[test]
    fn test_build_draw_commands_with_background() {
        let renderer = Renderer::new(1920.0, 1080.0);
        let state = RenderState {
            current_background: Some("bg/sky.png".to_string()),
            ..Default::default()
        };

        let mut resource_manager = make_test_resource_manager();
        let tex: Arc<dyn crate::rendering_types::Texture> = Arc::new(NullTexture::new(1920, 1080));
        resource_manager
            .texture_cache_mut()
            .insert("bg/sky.png".to_string(), tex);

        let manifest = Manifest::with_defaults();
        let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);

        assert!(
            !cmds.is_empty(),
            "should produce at least one Sprite command"
        );
        let has_sprite = cmds.iter().any(|c| matches!(c, DrawCommand::Sprite { .. }));
        assert!(has_sprite, "should contain a Sprite command for background");
    }

    #[test]
    fn test_build_draw_commands_with_character() {
        let mut renderer = Renderer::new(1920.0, 1080.0);
        renderer.set_screen_size(1920.0, 1080.0);

        let mut state = RenderState::default();
        state.show_character(
            "hero".to_string(),
            "characters/hero/normal.png".to_string(),
            vn_runtime::command::Position::Center,
        );

        let mut resource_manager = make_test_resource_manager();
        let tex: Arc<dyn crate::rendering_types::Texture> = Arc::new(NullTexture::new(512, 1024));
        resource_manager
            .texture_cache_mut()
            .insert("characters/hero/normal.png".to_string(), tex);

        let manifest = Manifest::with_defaults();
        let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);

        let sprite_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Sprite { .. }))
            .count();
        assert!(
            sprite_count >= 1,
            "should produce at least 1 Sprite for the character"
        );
    }
}
