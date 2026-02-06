//! 角色动画命令处理

use crate::command_executor::CharacterAnimationCommand;
use crate::renderer::{AnimatableCharacter, position_to_preset_name};
use macroquad::prelude::screen_width;
use std::rc::Rc;

use super::super::AppState;
use tracing::{info, warn};

/// 处理角色动画命令
pub fn handle_character_animation(app_state: &mut AppState) {
    let anim_cmd = app_state
        .command_executor
        .last_output
        .character_animation
        .clone();

    if let Some(cmd) = anim_cmd {
        match cmd {
            CharacterAnimationCommand::Show { alias, duration } => {
                // 获取角色的动画对象（clone 解除借用）
                let character = app_state.render_state.get_character_anim(&alias).cloned();
                if let Some(character) = character {
                    let object_id = ensure_character_registered(app_state, &alias, &character);

                    // 启动淡入动画
                    if let Err(e) = app_state
                        .animation_system
                        .animate_object::<AnimatableCharacter>(
                            object_id, "alpha", 0.0, 1.0, duration,
                        )
                    {
                        warn!(error = %e, "启动角色淡入动画失败");
                    }
                    info!(alias = %alias, duration = %duration, "角色淡入动画");
                }
            }
            CharacterAnimationCommand::Hide { alias, duration } => {
                // 获取角色的动画对象
                if let Some(&object_id) = app_state.character_object_ids.get(&alias) {
                    // 启动淡出动画
                    if let Err(e) = app_state
                        .animation_system
                        .animate_object::<AnimatableCharacter>(
                            object_id, "alpha", 1.0, 0.0, duration,
                        )
                    {
                        warn!(error = %e, "启动角色淡出动画失败");
                    }
                    info!(alias = %alias, duration = %duration, "角色淡出动画");
                }
            }
            CharacterAnimationCommand::Move {
                alias,
                old_position,
                new_position,
                duration,
            } => {
                // 计算旧位置和新位置的屏幕 X 坐标差（像素偏移）
                let old_preset_name = position_to_preset_name(old_position);
                let new_preset_name = position_to_preset_name(new_position);
                let old_preset = app_state.manifest.get_preset(old_preset_name);
                let new_preset = app_state.manifest.get_preset(new_preset_name);

                let screen_w = screen_width();
                let offset_x = screen_w * (old_preset.x - new_preset.x);

                // 获取角色的动画对象（clone 解除借用）
                let character = app_state.render_state.get_character_anim(&alias).cloned();
                if let Some(character) = character {
                    let object_id = ensure_character_registered(app_state, &alias, &character);

                    // 设置初始偏移（角色视觉上仍在旧位置）
                    character.set("position_x", offset_x);

                    // 动画：从偏移移动到 0（角色平滑移到新位置）
                    if let Err(e) = app_state
                        .animation_system
                        .animate_object::<AnimatableCharacter>(
                            object_id,
                            "position_x",
                            offset_x,
                            0.0,
                            duration,
                        )
                    {
                        warn!(error = %e, "启动角色移动动画失败");
                    }
                    info!(
                        alias = %alias,
                        from = %old_preset_name,
                        to = %new_preset_name,
                        duration = %duration,
                        "角色移动动画"
                    );
                }
            }
        }
    }
}

/// 确保角色已注册到动画系统，返回 ObjectId
fn ensure_character_registered(
    app_state: &mut AppState,
    alias: &str,
    character: &AnimatableCharacter,
) -> crate::renderer::animation::ObjectId {
    if let Some(&id) = app_state.character_object_ids.get(alias) {
        id
    } else {
        let id = app_state
            .animation_system
            .register(Rc::new(character.clone()));
        app_state.character_object_ids.insert(alias.to_string(), id);
        id
    }
}
