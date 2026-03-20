use super::*;
use vn_runtime::command::{InlineEffect, InlineEffectKind, Position};

/// 启动打字机并在位置 1 设置 Wait 效果并推进一次，用于 inline_wait 相关测试。
fn start_tw_with_wait(state: &mut RenderState, wait_secs: Option<f64>) {
    state.start_typewriter(
        None,
        "A".to_string(),
        vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(wait_secs),
        }],
        false,
    );
    state.advance_typewriter();
}

mod high_value;
mod low_value;
