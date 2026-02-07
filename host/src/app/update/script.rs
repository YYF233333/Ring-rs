//! 脚本模式输入与 VNRuntime tick

use tracing::{debug, error, info};
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

use crate::ExecuteResult;

use super::super::AppState;
use super::super::CoreSystems;
use super::super::command_handlers::{apply_effect_requests, handle_audio_command};
use super::super::save::return_to_title_from_game;
use super::super::script_loader::collect_prefetch_paths;

/// 一次性跳过所有活跃的演出效果（动画/过渡/场景过渡/打字机）
///
/// 用于 Skip 模式，确保所有效果正确收敛：
/// - 角色动画：skip_all + 清理淡出完成的角色
/// - 背景过渡（changeBG dissolve）：直接完成
/// - 场景过渡（changeScene）：完全跳过并切换背景
/// - 打字机：立即完成
///
/// 阶段 27：签名从 `&mut AppState` 改为 `&mut CoreSystems`。
pub fn skip_all_active_effects(core: &mut CoreSystems) {
    // 1. 跳过所有角色动画
    if core.animation_system.has_active_animations() {
        core.animation_system.skip_all();
        let _ = core.animation_system.update(0.0);
        cleanup_fading_characters(core);
    }

    // 2. 跳过背景过渡（changeBG dissolve）
    if core.renderer.transition.is_active() {
        core.renderer.transition.skip();
    }

    // 3. 完全跳过场景过渡（changeScene），确保背景切换
    if core.renderer.is_scene_transition_active() {
        if let Some(path) = core.renderer.skip_scene_transition_to_end() {
            core.render_state.set_background(path);
        }
        core.render_state.ui_visible = true;
    }

    // 4. 完成打字机
    if !core.render_state.is_dialogue_complete() {
        core.render_state.complete_typewriter();
    }
}

/// 清理淡出完成的角色（从动画系统注销并从 render_state 移除）
///
/// 阶段 27：签名从 `&mut AppState` 改为 `&mut CoreSystems`。
pub(super) fn cleanup_fading_characters(core: &mut CoreSystems) {
    let fading_out: Vec<String> = core
        .render_state
        .visible_characters
        .iter()
        .filter(|(_, c)| c.fading_out)
        .map(|(alias, _)| alias.clone())
        .collect();

    for alias in &fading_out {
        if let Some(object_id) = core.character_object_ids.remove(alias) {
            core.animation_system.unregister(object_id);
        }
    }
    core.render_state.remove_fading_out_characters(&fading_out);
}

/// 处理脚本模式下的输入
pub fn handle_script_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    // 如果有动画正在进行，跳过所有动画
    if app_state.core.animation_system.has_active_animations() {
        app_state.core.animation_system.skip_all();
        // 应用最终状态
        let _ = app_state.core.animation_system.update(0.0);

        // 清理淡出完成的角色
        cleanup_fading_characters(&mut app_state.core);
        return;
    }

    // 如果转场正在进行（changeBG），允许输入用于跳过转场
    if app_state.core.renderer.transition.is_active() {
        // 跳过转场效果
        app_state.core.renderer.transition.skip();
        return;
    }

    // 如果场景过渡正在进行（changeScene），允许输入用于跳过转场
    if app_state.core.renderer.is_scene_transition_active() {
        // 跳过当前阶段的转场动画
        app_state.core.renderer.skip_scene_transition_phase();

        // 如果跳过后过渡完成，立即恢复 UI 和切换背景
        if !app_state.core.renderer.is_scene_transition_active() {
            // 切换待处理的背景（如果有）
            if let Some(path) = app_state.core.renderer.take_pending_background() {
                app_state.core.render_state.set_background(path);
            }
            // 恢复 UI 可见性
            app_state.core.render_state.ui_visible = true;
        }
        return;
    }

    // 如果对话正在打字，先完成打字
    if !app_state.core.render_state.is_dialogue_complete() {
        app_state.core.render_state.complete_typewriter();
        return;
    }

    // 将输入传递给 VNRuntime
    run_script_tick(app_state, Some(input));
}

/// 执行一次 VNRuntime tick
pub fn run_script_tick(app_state: &mut AppState, input: Option<RuntimeInput>) {
    // 如果是选择输入，先清除选择界面
    if let Some(RuntimeInput::ChoiceSelected { index }) = &input {
        debug!(choice = index + 1, "用户选择了选项");
        app_state.core.render_state.clear_choices();
    }

    // 先执行 tick 并收集结果
    let tick_result = {
        let runtime = match app_state.session.vn_runtime.as_mut() {
            Some(r) => r,
            None => {
                error!("VNRuntime 未初始化");
                return;
            }
        };
        runtime.tick(input)
    };

    // 处理 tick 结果
    match tick_result {
        Ok((commands, waiting)) => {
            debug!(
                commands = commands.len(),
                waiting = ?waiting,
                "tick 返回命令"
            );

            // 收集命令中的资源路径（用于预取统计）
            let prefetch_paths = collect_prefetch_paths(&commands);
            if !prefetch_paths.is_empty() {
                debug!(paths = ?prefetch_paths, "预取资源");
            }

            // 执行所有命令
            for command in &commands {
                debug!(command = ?command, "执行命令");
                let result = app_state.core.command_executor.execute(
                    command,
                    &mut app_state.core.render_state,
                    &app_state.core.resource_manager,
                );

                // 应用动画/过渡效果请求（统一入口）
                apply_effect_requests(&mut app_state.core, &app_state.session.manifest);

                // 处理音频命令
                handle_audio_command(&mut app_state.core, &app_state.config);

                // 检查执行结果
                if let ExecuteResult::Error(e) = result {
                    error!(error = %e, "命令执行失败");
                }
            }

            // 更新等待状态
            app_state.session.waiting_reason = waiting.clone();

            // 如果是选择等待，重置选择索引
            if let WaitingReason::WaitForChoice { choice_count } = &waiting {
                app_state.input_manager.reset_choice(*choice_count);
            }

            // 检查脚本是否执行完毕
            let is_finished = app_state
                .session
                .vn_runtime
                .as_ref()
                .map(|r| r.is_finished())
                .unwrap_or(false);
            if is_finished && !app_state.session.script_finished {
                app_state.session.script_finished = true;
                info!("脚本执行完毕，自动返回主界面");
                // 自动返回主界面，不保存 Continue 存档（避免下次 Continue 直接跳到末尾）
                return_to_title_from_game(app_state, false);
            }

            // 重置打字机计时器
            app_state.session.typewriter_timer = 0.0;
        }
        Err(e) => {
            error!(error = ?e, "Runtime tick 错误");
        }
    }
}
