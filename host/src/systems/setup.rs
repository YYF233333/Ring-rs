//! 初始化系统

use bevy::prelude::*;

use crate::components::*;

/// 初始化系统：创建摄像机和 UI
pub fn setup_system(mut commands: Commands) {
    // 创建 2D 摄像机
    commands.spawn((
        Camera2d,
        MainCamera,
    ));

    // 创建对话框 UI
    spawn_dialogue_box(&mut commands);
}

/// 创建对话框 UI
fn spawn_dialogue_box(commands: &mut Commands) {
    // 对话框容器（底部）
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            DialogueBox,
        ))
        .with_children(|parent| {
            // 选择容器（对话框上方）
            parent.spawn((
                Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                ChoiceContainer,
            ));

            // 对话框背景
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ))
                .with_children(|parent| {
                    // 说话者名字
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.5)),
                        SpeakerText,
                    ));

                    // 对话内容
                    parent.spawn((
                        Text::new("点击开始..."),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        ContentText,
                    ));
                });
        });
}
