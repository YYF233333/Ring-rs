//! VN Plugin - 主插件模块

use bevy::prelude::*;

use crate::resources::*;
use crate::systems::*;

/// Visual Novel 主插件
pub struct VNPlugin;

impl Plugin for VNPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册资源
            .init_resource::<VNState>()
            .init_resource::<DialogueState>()
            // 注册消息
            .add_message::<VNCommand>()
            .add_message::<PlayerInput>()
            // 添加系统
            .add_systems(Startup, setup_system)
            .add_systems(
                Update,
                (
                    input_system,
                    tick_runtime_system,
                    execute_commands_system,
                    update_dialogue_system,
                    update_characters_system,
                ),
            );
    }
}
