//! 渲染逻辑
//!
//! 生成 sprite 绘制命令，由 WgpuBackend 消费。
//! UI（对话框、选项、屏幕）由 egui 在 main.rs 渲染循环中构建。

use crate::backend::DrawCommand;

use super::AppState;

/// 为当前游戏状态生成 sprite 绘制命令
///
/// 返回按层级排序的 DrawCommand 列表（背景 → 角色 → 场景效果遮罩）。
/// UI 层（对话/选项/屏幕）由 egui 负责，不在此处生成。
pub fn build_game_draw_commands(app_state: &AppState) -> Vec<DrawCommand> {
    app_state.core.renderer.build_draw_commands(
        &app_state.core.render_state,
        &app_state.core.resource_manager,
        &app_state.session.manifest,
    )
}
