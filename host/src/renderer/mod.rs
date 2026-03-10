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

use crate::backend::{DrawCommand, GpuTexture};
use crate::manifest::Manifest;
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
pub mod effects;
mod image_dissolve;
pub mod render_state;
pub mod scene_transition;
mod text_renderer;
mod transition;

pub use animation::{
    AnimPropertyKey, Animatable, ObjectId, PropertyAccessor, SimplePropertyAccessor,
};
pub use animation::{
    Animation, AnimationEvent, AnimationId, AnimationState, AnimationSystem, EasingFunction,
    Transform, Vec2 as AnimVec2,
};
pub use background_transition::{AnimatableBackgroundTransition, BackgroundTransitionData};
pub use character_animation::{AnimatableCharacter, CharacterAnimData};
pub use image_dissolve::ImageDissolve;
pub use render_state::{
    CharacterSprite, ChoiceItem, ChoicesState, DialogueState, RenderState, SceneEffectState,
    TitleCardState,
};
pub use scene_transition::{
    AnimatableSceneTransition, SceneTransitionManager, SceneTransitionPhase, SceneTransitionType,
};
pub use text_renderer::TextRenderer;
pub use transition::{TransitionManager, TransitionPhase, TransitionType};

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

        // 场景效果：震动偏移
        let (shake_x, shake_y) = self.current_shake_offset();

        // 1. 渲染背景（带过渡效果）
        self.build_background_commands(&mut commands, state, resource_manager, shake_x, shake_y);

        // 2. 渲染角色
        self.build_character_commands(
            &mut commands,
            state,
            resource_manager,
            manifest,
            shake_x,
            shake_y,
        );

        // 场景效果：暗化遮罩
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

        // 场景效果：模糊近似（半透明白色叠加模拟）
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

        // 3. 场景遮罩（changeScene 的 Fade/FadeWhite）
        self.build_scene_mask_commands(&mut commands, state, resource_manager);

        commands
    }

    // ============ 场景效果方法 ============

    /// 启动震动效果
    pub fn start_shake(&mut self, amplitude_x: f32, amplitude_y: f32, duration: f32) {
        self.shake = ShakeState {
            active: true,
            amplitude_x,
            amplitude_y,
            elapsed: 0.0,
            duration,
        };
    }

    /// 启动模糊过渡
    pub fn start_blur_transition(&mut self, from: f32, to: f32, duration: f32) {
        self.blur_transition = BlurTransitionState {
            active: true,
            from,
            to,
            elapsed: 0.0,
            duration,
        };
    }

    /// 更新场景效果（每帧调用）
    pub fn update_scene_effects(&mut self, dt: f32, scene_effect: &mut SceneEffectState) -> bool {
        let mut any_active = false;

        if self.shake.active {
            self.shake.elapsed += dt;
            if self.shake.elapsed >= self.shake.duration {
                self.shake.active = false;
                scene_effect.shake_offset_x = 0.0;
                scene_effect.shake_offset_y = 0.0;
            } else {
                let progress = self.shake.elapsed / self.shake.duration;
                let decay = 1.0 - progress;
                let t = self.shake.elapsed * SHAKE_FREQUENCY;
                scene_effect.shake_offset_x = t.sin() * self.shake.amplitude_x * decay;
                scene_effect.shake_offset_y = (t * 1.3).cos() * self.shake.amplitude_y * decay;
                any_active = true;
            }
        }

        if self.blur_transition.active {
            self.blur_transition.elapsed += dt;
            if self.blur_transition.elapsed >= self.blur_transition.duration {
                self.blur_transition.active = false;
                scene_effect.blur_amount = self.blur_transition.to;
            } else {
                let progress =
                    (self.blur_transition.elapsed / self.blur_transition.duration).clamp(0.0, 1.0);
                let smoothed = progress * progress * (3.0 - 2.0 * progress);
                scene_effect.blur_amount = self.blur_transition.from
                    + (self.blur_transition.to - self.blur_transition.from) * smoothed;
                any_active = true;
            }
        }

        any_active
    }

    /// 检查场景效果是否仍在播放
    pub fn is_scene_effect_active(&self) -> bool {
        self.shake.active || self.blur_transition.active
    }

    fn current_shake_offset(&self) -> (f32, f32) {
        if self.shake.active {
            let progress = self.shake.elapsed / self.shake.duration;
            let decay = 1.0 - progress;
            let t = self.shake.elapsed * 30.0;
            (
                t.sin() * self.shake.amplitude_x * decay,
                (t * 1.3).cos() * self.shake.amplitude_y * decay,
            )
        } else {
            (0.0, 0.0)
        }
    }

    /// 更新过渡效果
    pub fn update_transition(&mut self, dt: f32) -> bool {
        self.transition.update(dt)
    }

    /// 开始背景过渡（保留兼容）
    pub fn start_background_transition(
        &mut self,
        old_bg: Option<String>,
        transition: Option<&vn_runtime::command::Transition>,
    ) {
        self.old_background = old_bg;

        if let Some(trans) = transition {
            self.transition.start_from_command(trans);
        } else {
            self.transition.start(TransitionType::Dissolve, 0.2);
        }
    }

    /// 开始背景过渡（阶段 25：基于 ResolvedEffect 的统一入口）
    pub fn start_background_transition_resolved(
        &mut self,
        old_bg: Option<String>,
        effect: &effects::ResolvedEffect,
    ) {
        self.old_background = old_bg;
        self.transition.start_from_resolved(effect);
    }

    /// 跳过当前过渡效果
    pub fn skip_transition(&mut self) {
        self.transition.skip();
        self.old_background = None;
    }

    // ========== 场景过渡管理 ==========

    pub fn start_scene_fade(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade(duration, pending_background);
    }

    pub fn start_scene_fade_white(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade_white(duration, pending_background);
    }

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

    pub fn update_scene_transition(&mut self, dt: f32) -> bool {
        self.scene_transition.update(dt)
    }

    pub fn is_scene_transition_at_midpoint(&self) -> bool {
        self.scene_transition.is_at_midpoint()
    }

    pub fn take_pending_background(&mut self) -> Option<String> {
        self.scene_transition.take_pending_background()
    }

    pub fn get_scene_transition_ui_alpha(&self) -> f32 {
        if self.scene_transition.is_active() {
            self.scene_transition.ui_alpha()
        } else {
            1.0
        }
    }

    pub fn skip_scene_transition_phase(&mut self) {
        self.scene_transition.skip_current_phase();
    }

    pub fn skip_scene_transition_to_end(&mut self) -> Option<String> {
        self.scene_transition.skip_to_end()
    }

    pub fn is_scene_transition_active(&self) -> bool {
        self.scene_transition.is_active()
    }

    pub fn is_scene_transition_ui_fading_in(&self) -> bool {
        self.scene_transition.is_ui_fading_in()
    }

    /// 获取选择框的矩形区域
    pub fn get_choice_rects(&self, choice_count: usize) -> Vec<(f32, f32, f32, f32)> {
        self.text_renderer
            .get_choice_rects(choice_count, self.screen_width, self.screen_height)
    }

    // ============ 内部绘制命令生成 ============

    /// 生成背景绘制命令（带过渡效果）
    fn build_background_commands(
        &self,
        commands: &mut Vec<DrawCommand>,
        state: &RenderState,
        resource_manager: &ResourceManager,
        shake_x: f32,
        shake_y: f32,
    ) {
        // 旧背景（过渡中）
        if self.transition.is_active()
            && let Some(ref old_bg_path) = self.old_background
            && let Some(texture) = resource_manager.peek_texture(old_bg_path)
        {
            let alpha = self.transition.old_content_alpha();
            if alpha > 0.0 {
                let (dw, dh, x, y) = self.calculate_draw_rect_for(&texture, DrawMode::Cover);
                commands.push(DrawCommand::Sprite {
                    texture,
                    x: x + shake_x,
                    y: y + shake_y,
                    width: dw,
                    height: dh,
                    color: [1.0, 1.0, 1.0, alpha],
                });
            }
        }

        // 新（当前）背景
        if let Some(ref bg_path) = state.current_background
            && let Some(texture) = resource_manager.peek_texture(bg_path)
        {
            let alpha = self.transition.new_content_alpha();
            let (dw, dh, x, y) = self.calculate_draw_rect_for(&texture, DrawMode::Cover);
            commands.push(DrawCommand::Sprite {
                texture,
                x: x + shake_x,
                y: y + shake_y,
                width: dw,
                height: dh,
                color: [1.0, 1.0, 1.0, alpha],
            });
        }
    }

    /// 生成角色绘制命令
    fn build_character_commands(
        &self,
        commands: &mut Vec<DrawCommand>,
        state: &RenderState,
        resource_manager: &ResourceManager,
        manifest: &Manifest,
        shake_x: f32,
        shake_y: f32,
    ) {
        let mut characters: Vec<_> = state.visible_characters.iter().collect();
        characters.sort_by_key(|(_, c)| c.z_order);

        let sw = self.screen_width;
        let sh = self.screen_height;
        let base_scale = self.get_scale_factor();

        for (_alias, character) in characters {
            if let Some(texture) = resource_manager.peek_texture(&character.texture_path) {
                let group_config = manifest.get_group_config(&character.texture_path);
                let position_name = position_to_preset_name(character.position);
                let preset = manifest.get_preset(position_name);

                let alpha = character.anim.alpha();
                let (position_x, position_y) = character.anim.position();
                let (scale_x, _scale_y) = character.anim.scale();

                let final_scale = base_scale * group_config.pre_scale * preset.scale * scale_x;

                let dest_w = texture.width() * final_scale;
                let dest_h = texture.height() * final_scale;

                let target_x = sw * preset.x + position_x;
                let target_y = sh * preset.y + position_y;

                let anchor_px_x = dest_w * group_config.anchor.x;
                let anchor_px_y = dest_h * group_config.anchor.y;

                let x = target_x - anchor_px_x + shake_x;
                let y = target_y - anchor_px_y + shake_y;

                commands.push(DrawCommand::Sprite {
                    texture,
                    x,
                    y,
                    width: dest_w,
                    height: dest_h,
                    color: [1.0, 1.0, 1.0, alpha],
                });
            }
        }
    }

    /// 生成场景遮罩绘制命令
    fn build_scene_mask_commands(
        &self,
        commands: &mut Vec<DrawCommand>,
        _state: &RenderState,
        resource_manager: &ResourceManager,
    ) {
        if self.scene_transition.is_mask_complete() {
            return;
        }

        let sw = self.screen_width;
        let sh = self.screen_height;

        match self.scene_transition.transition_type() {
            Some(SceneTransitionType::Fade) => {
                let alpha = self.scene_transition.mask_alpha();
                if alpha > 0.0 {
                    commands.push(DrawCommand::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: sw,
                        height: sh,
                        color: [0.0, 0.0, 0.0, alpha],
                    });
                }
            }
            Some(SceneTransitionType::FadeWhite) => {
                let alpha = self.scene_transition.mask_alpha();
                if alpha > 0.0 {
                    commands.push(DrawCommand::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: sw,
                        height: sh,
                        color: [1.0, 1.0, 1.0, alpha],
                    });
                }
            }
            Some(SceneTransitionType::Rule {
                mask_path,
                reversed,
            }) => {
                let progress = self.scene_transition.progress();
                let phase = self.scene_transition.phase();

                if let Some(mask_texture) = resource_manager.peek_texture(mask_path) {
                    let (dissolve_progress, overlay_alpha) = match phase {
                        SceneTransitionPhase::FadeIn => (progress, 1.0f32),
                        SceneTransitionPhase::Blackout => (1.0, 1.0),
                        SceneTransitionPhase::FadeOut => (1.0 - progress, 1.0),
                        _ => (0.0, 0.0),
                    };
                    if overlay_alpha > 0.0 {
                        commands.push(DrawCommand::Dissolve {
                            mask_texture,
                            progress: dissolve_progress,
                            ramp: self.image_dissolve.ramp(),
                            reversed: *reversed,
                            overlay_color: [0.0, 0.0, 0.0, overlay_alpha],
                            x: 0.0,
                            y: 0.0,
                            width: sw,
                            height: sh,
                        });
                    }
                } else {
                    // 降级：遮罩未加载时使用纯色 Rect
                    let alpha = match phase {
                        SceneTransitionPhase::FadeIn => progress,
                        SceneTransitionPhase::Blackout => 1.0,
                        SceneTransitionPhase::FadeOut => 1.0 - progress,
                        _ => 0.0,
                    };
                    if alpha > 0.0 {
                        commands.push(DrawCommand::Rect {
                            x: 0.0,
                            y: 0.0,
                            width: sw,
                            height: sh,
                            color: [0.0, 0.0, 0.0, alpha],
                        });
                    }
                }
            }
            None => {}
        }
    }

    fn get_scale_factor(&self) -> f32 {
        let scale_x = self.screen_width / self.design_width;
        let scale_y = self.screen_height / self.design_height;
        scale_x.min(scale_y)
    }

    /// 计算纹理绘制矩形（dest_w, dest_h, x, y）
    fn calculate_draw_rect_for(
        &self,
        texture: &GpuTexture,
        mode: DrawMode,
    ) -> (f32, f32, f32, f32) {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let tw = texture.width();
        let th = texture.height();

        match mode {
            DrawMode::Cover => {
                let scale = (sw / tw).max(sh / th);
                let dw = tw * scale;
                let dh = th * scale;
                (dw, dh, (sw - dw) / 2.0, (sh - dh) / 2.0)
            }
            DrawMode::Contain => {
                let scale = (sw / tw).min(sh / th);
                let dw = tw * scale;
                let dh = th * scale;
                (dw, dh, (sw - dw) / 2.0, (sh - dh) / 2.0)
            }
            DrawMode::Stretch => (sw, sh, 0.0, 0.0),
        }
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
