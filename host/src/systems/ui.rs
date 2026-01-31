//! UI 更新系统

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

/// 更新对话框系统
pub fn update_dialogue_system(
    dialogue_state: Res<DialogueState>,
    mut speaker_query: Query<&mut Text, (With<SpeakerText>, Without<ContentText>)>,
    mut content_query: Query<&mut Text, (With<ContentText>, Without<SpeakerText>)>,
    mut commands: Commands,
    choice_container: Query<Entity, With<ChoiceContainer>>,
    choice_buttons: Query<Entity, With<ChoiceButton>>,
) {
    // 更新说话者
    if let Ok(mut text) = speaker_query.single_mut() {
        **text = dialogue_state
            .speaker
            .clone()
            .unwrap_or_default();
    }

    // 更新对话内容
    if let Ok(mut text) = content_query.single_mut() {
        **text = dialogue_state.content.clone();
    }

    // 更新选择按钮
    // 先清除旧按钮
    for entity in &choice_buttons {
        commands.entity(entity).despawn();
    }

    // 如果有选项，创建新按钮
    if !dialogue_state.choices.is_empty() {
        if let Ok(container) = choice_container.single() {
            commands.entity(container).with_children(|parent| {
                for choice in &dialogue_state.choices {
                    spawn_choice_button(parent, choice);
                }
            });
        }
    }
}

/// 创建选择按钮
fn spawn_choice_button(parent: &mut ChildSpawnerCommands, choice: &ChoiceItem) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
            ChoiceButton {
                index: choice.index,
            },
        ))
        .with_child((
            Text::new(choice.text.clone()),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

/// 更新角色立绘系统
pub fn update_characters_system(
    // 暂时不需要实现，角色更新在 execute_commands_system 中处理
) {
}
