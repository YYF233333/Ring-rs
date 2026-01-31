//! 命令执行系统

use bevy::prelude::*;
use vn_runtime::Command;
use crate::components::*;
use crate::resources::*;

/// 执行 Runtime 命令系统
pub fn execute_commands_system(
    mut command_reader: MessageReader<VNCommand>,
    mut dialogue_state: ResMut<DialogueState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    background_query: Query<Entity, With<Background>>,
    character_query: Query<(Entity, &Character)>,
) {
    // 收集命令
    let cmds: Vec<_> = command_reader.read().cloned().collect();
    
    for VNCommand(cmd) in cmds {
        match &cmd {
            Command::ShowText { speaker, content } => {
                dialogue_state.speaker = speaker.clone();
                dialogue_state.content = content.clone();
                dialogue_state.visible = true;
                dialogue_state.choices.clear();
            }

            Command::ShowBackground { path, .. } => {
                // 移除旧背景
                for entity in &background_query {
                    commands.entity(entity).despawn();
                }

                // 加载新背景
                let path_owned = path.clone();
                let texture = asset_server.load(path_owned);
                commands
                    .spawn((
                        Sprite::from_image(texture),
                        Transform::from_xyz(0.0, 0.0, -10.0),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Background,
                    ));
            }

            Command::ShowCharacter { path, alias, position, .. } => {
                // 检查是否已存在同别名的角色
                let existing = character_query
                    .iter()
                    .find(|(_, c)| c.alias == *alias)
                    .map(|(e, _)| e);

                if let Some(entity) = existing {
                    commands.entity(entity).despawn();
                }

                // 计算位置
                let x = position_to_x(position);

                // 加载角色立绘
                let path_owned = path.clone();
                let texture = asset_server.load(path_owned);
                commands
                    .spawn((
                        Sprite::from_image(texture),
                        Transform::from_xyz(x, 0.0, 0.0),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Character {
                            alias: alias.clone(),
                            position: position.clone(),
                        },
                    ));
            }

            Command::HideCharacter { alias, .. } => {
                // 查找并移除角色
                for (entity, character) in &character_query {
                    if character.alias == *alias {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }

            Command::PresentChoices { choices, .. } => {
                dialogue_state.choices = choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| ChoiceItem {
                        text: c.text.clone(),
                        index: i,
                    })
                    .collect();
            }

            _ => {} // 其他命令暂不处理
        }
    }
}

/// 将 Position 转换为 X 坐标
fn position_to_x(position: &vn_runtime::command::Position) -> f32 {
    use vn_runtime::command::Position;
    match *position {
        Position::FarLeft => -500.0,
        Position::Left => -300.0,
        Position::NearLeft => -150.0,
        Position::NearMiddle | Position::Center => 0.0,
        Position::NearRight => 150.0,
        Position::Right => 300.0,
        Position::FarRight => 500.0,
        Position::FarMiddle => 0.0,
    }
}