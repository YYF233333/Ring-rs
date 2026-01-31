//! Runtime 驱动系统

use bevy::prelude::*;

use vn_runtime::input::RuntimeInput;

use crate::resources::{PlayerInput, VNCommand, VNState};

/// 驱动 Runtime 执行系统
pub fn tick_runtime_system(
    mut vn_state: ResMut<VNState>,
    mut input_reader: MessageReader<PlayerInput>,
    mut command_writer: MessageWriter<VNCommand>,
) {
    // 收集输入消息
    let inputs: Vec<PlayerInput> = input_reader.read().cloned().collect();

    // 如果没有 Runtime，跳过
    if vn_state.runtime.is_none() {
        return;
    }

    // 处理输入
    for input in &inputs {
        let rt_input = match input {
            PlayerInput::Click => RuntimeInput::Click,
            PlayerInput::Select(idx) => RuntimeInput::ChoiceSelected { index: *idx },
        };

        let runtime = vn_state.runtime.as_mut().unwrap();
        // 调用 Runtime tick
        match runtime.tick(Some(rt_input)) {
            Ok((commands, waiting)) => {
                // 更新等待状态
                vn_state.waiting = waiting;

                // 发送命令消息
                for cmd in commands {
                    command_writer.write(VNCommand(cmd));
                }
            }
            Err(e) => {
                warn!("Runtime error: {:?}", e);
            }
        }
    }

    // 如果处于空闲状态，自动继续执行（用 Click 驱动）
    if vn_state.is_idle() {
        let runtime = vn_state.runtime.as_mut().unwrap();
        match runtime.tick(Some(RuntimeInput::Click)) {
            Ok((commands, waiting)) => {
                vn_state.waiting = waiting;
                for cmd in commands {
                    command_writer.write(VNCommand(cmd));
                }
            }
            Err(e) => {
                warn!("Runtime error: {:?}", e);
            }
        }
    }
}
