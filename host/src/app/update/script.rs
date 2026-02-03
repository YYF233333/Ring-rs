//! 脚本模式输入与 VNRuntime tick

use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

use crate::ExecuteResult;

use super::super::AppState;
use super::super::command_handlers::{
    apply_transition_effect, handle_audio_command, handle_character_animation,
    handle_scene_transition,
};
use super::super::save::return_to_title_from_game;
use super::super::script_loader::collect_prefetch_paths;

/// 处理脚本模式下的输入
pub fn handle_script_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    // 如果有动画正在进行，跳过所有动画
    if app_state.animation_system.has_active_animations() {
        app_state.animation_system.skip_all();
        // 应用最终状态
        let _ = app_state.animation_system.update(0.0);

        // 清理淡出完成的角色
        let fading_out: Vec<String> = app_state
            .render_state
            .visible_characters
            .iter()
            .filter(|(_, c)| c.fading_out)
            .map(|(alias, _)| alias.clone())
            .collect();

        // 从动画系统注销并移除
        for alias in &fading_out {
            if let Some(object_id) = app_state.character_object_ids.remove(alias) {
                app_state.animation_system.unregister(object_id);
            }
        }
        app_state
            .render_state
            .remove_fading_out_characters(&fading_out);
        return;
    }

    // 如果转场正在进行（changeBG），允许输入用于跳过转场
    if app_state.renderer.transition.is_active() {
        // 跳过转场效果
        app_state.renderer.transition.skip();
        return;
    }

    // 如果场景过渡正在进行（changeScene），允许输入用于跳过转场
    if app_state.renderer.is_scene_transition_active() {
        // 跳过当前阶段的转场动画
        app_state.renderer.skip_scene_transition_phase();

        // 如果跳过后过渡完成，立即恢复 UI 和切换背景
        if !app_state.renderer.is_scene_transition_active() {
            // 切换待处理的背景（如果有）
            if let Some(path) = app_state.renderer.take_pending_background() {
                app_state.render_state.set_background(path);
            }
            // 恢复 UI 可见性
            app_state.render_state.ui_visible = true;
        }
        return;
    }

    // 如果对话正在打字，先完成打字
    if !app_state.render_state.is_dialogue_complete() {
        app_state.render_state.complete_typewriter();
        return;
    }

    // 将输入传递给 VNRuntime
    run_script_tick(app_state, Some(input));
}

/// 执行一次 VNRuntime tick
pub fn run_script_tick(app_state: &mut AppState, input: Option<RuntimeInput>) {
    // 如果是选择输入，先清除选择界面
    if let Some(RuntimeInput::ChoiceSelected { index }) = &input {
        println!("📜 用户选择了选项 {}", index + 1);
        app_state.render_state.clear_choices();
    }

    // 先执行 tick 并收集结果
    let tick_result = {
        let runtime = match app_state.vn_runtime.as_mut() {
            Some(r) => r,
            None => {
                eprintln!("❌ VNRuntime 未初始化");
                return;
            }
        };
        runtime.tick(input)
    };

    // 处理 tick 结果
    match tick_result {
        Ok((commands, waiting)) => {
            println!(
                "📜 tick 返回 {} 条命令, 等待状态: {:?}",
                commands.len(),
                waiting
            );

            // 收集命令中的资源路径（用于预取统计）
            let prefetch_paths = collect_prefetch_paths(&commands);
            if !prefetch_paths.is_empty() {
                println!("  📦 预取资源: {:?}", prefetch_paths);
            }

            // 执行所有命令
            for command in &commands {
                println!("  ▶️ {:?}", command);
                let result = app_state.command_executor.execute(
                    command,
                    &mut app_state.render_state,
                    &app_state.resource_manager,
                );

                // 应用过渡效果
                apply_transition_effect(app_state);

                // 处理音频命令
                handle_audio_command(app_state);

                // 处理角色动画命令
                handle_character_animation(app_state);

                // 处理场景切换命令
                handle_scene_transition(app_state);

                // 检查执行结果
                if let ExecuteResult::Error(e) = result {
                    eprintln!("  ❌ 命令执行失败: {}", e);
                }
            }

            // 更新等待状态
            app_state.waiting_reason = waiting.clone();

            // 如果是选择等待，重置选择索引
            if let WaitingReason::WaitForChoice { choice_count } = &waiting {
                app_state.input_manager.reset_choice(*choice_count);
            }

            // 检查脚本是否执行完毕
            let is_finished = app_state
                .vn_runtime
                .as_ref()
                .map(|r| r.is_finished())
                .unwrap_or(false);
            if is_finished && !app_state.script_finished {
                app_state.script_finished = true;
                println!("📜 脚本执行完毕，自动返回主界面");
                // 自动返回主界面，不保存 Continue 存档（避免下次 Continue 直接跳到末尾）
                return_to_title_from_game(app_state, false);
            }

            // 重置打字机计时器
            app_state.typewriter_timer = 0.0;
        }
        Err(e) => {
            eprintln!("❌ Runtime tick 错误: {:?}", e);
        }
    }
}
