//! 输入处理系统

use bevy::prelude::*;

use crate::components::ChoiceButton;
use crate::resources::{PlayerInput, VNState};

/// 输入处理系统
pub fn input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    vn_state: Res<VNState>,
    mut input_writer: MessageWriter<PlayerInput>,
    interaction_query: Query<(&Interaction, &ChoiceButton), Changed<Interaction>>,
) {
    // 处理选择按钮点击
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            input_writer.write(PlayerInput::Select(button.index));
            return;
        }
    }

    // 处理点击/按键继续
    if vn_state.is_waiting_for_click() || vn_state.is_idle() {
        if mouse.just_pressed(MouseButton::Left)
            || keyboard.just_pressed(KeyCode::Space)
            || keyboard.just_pressed(KeyCode::Enter)
        {
            input_writer.write(PlayerInput::Click);
        }
    }
}
