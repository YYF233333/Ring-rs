//! 角色动画命令处理

use crate::command_executor::CharacterAnimationCommand;
use crate::renderer::AnimatableCharacter;
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
                // 获取角色的动画对象并注册到动画系统
                if let Some(character) = app_state.render_state.get_character_anim(&alias) {
                    // 如果角色还没注册到动画系统，先注册
                    let object_id = if let Some(&id) = app_state.character_object_ids.get(&alias) {
                        id
                    } else {
                        // 注册角色到动画系统
                        let id = app_state
                            .animation_system
                            .register(Rc::new(character.clone()));
                        app_state.character_object_ids.insert(alias.clone(), id);
                        id
                    };

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
        }
    }
}
