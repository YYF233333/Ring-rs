//! Renderer 绘制命令生成
//!
//! 将 RenderState 中的背景、角色、场景遮罩转换为 DrawCommand 列表。

use crate::manifest::Manifest;
use crate::rendering_types::{DrawCommand, Texture};
use crate::resources::{LogicalPath, ResourceManager};

use super::scene_transition::SceneTransitionPhase;
use super::{DrawMode, RenderState, Renderer, SceneTransitionType};

impl Renderer {
    /// 计算选项矩形区域（用于点击检测）
    ///
    /// 返回每个选项的 (x, y, width, height) 元组数组。
    pub fn get_choice_rects(&self, choice_count: usize) -> Vec<(f32, f32, f32, f32)> {
        let choice_height = 50.0;
        let choice_spacing = 10.0;
        let total_height = choice_count as f32 * (choice_height + choice_spacing) - choice_spacing;
        let start_y = (self.screen_height - total_height) / 2.0;

        let box_w = self.screen_width * 0.6;
        let box_x = (self.screen_width - box_w) / 2.0;

        (0..choice_count)
            .map(|i| {
                let y = start_y + i as f32 * (choice_height + choice_spacing);
                (box_x, y, box_w, choice_height)
            })
            .collect()
    }

    /// 生成背景绘制命令（带过渡效果）
    pub(super) fn build_background_commands(
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
            && let Some(texture) = resource_manager.peek_texture(&LogicalPath::new(old_bg_path))
        {
            let alpha = self.transition.old_content_alpha();
            if alpha > 0.0 {
                let (dw, dh, x, y) = self.calculate_draw_rect_for(&*texture, DrawMode::Cover);
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
            && let Some(texture) = resource_manager.peek_texture(&LogicalPath::new(bg_path))
        {
            let alpha = self.transition.new_content_alpha();
            let (dw, dh, x, y) = self.calculate_draw_rect_for(&*texture, DrawMode::Cover);
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
    pub(super) fn build_character_commands(
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
            if let Some(texture) =
                resource_manager.peek_texture(&LogicalPath::new(&character.texture_path))
            {
                let group_config = manifest.get_group_config(&character.texture_path);
                let position_name = super::position_to_preset_name(character.position);
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
    pub(super) fn build_scene_mask_commands(
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

                if let Some(mask_texture) =
                    resource_manager.peek_texture(&LogicalPath::new(mask_path))
                {
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
                            ramp: self.dissolve_ramp,
                            reversed: *reversed,
                            overlay_color: [0.0, 0.0, 0.0, overlay_alpha],
                            x: 0.0,
                            y: 0.0,
                            width: sw,
                            height: sh,
                        });
                    }
                } else {
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

    pub(super) fn get_scale_factor(&self) -> f32 {
        let scale_x = self.screen_width / self.design_width;
        let scale_y = self.screen_height / self.design_height;
        scale_x.min(scale_y)
    }

    /// 计算纹理绘制矩形（dest_w, dest_h, x, y）
    pub(super) fn calculate_draw_rect_for(
        &self,
        texture: &dyn Texture,
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
