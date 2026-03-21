//! 脚本模式输入与 VNRuntime tick

use std::path::PathBuf;

use tracing::{debug, error, info, warn};
use vn_runtime::command::{Command, SIGNAL_CUTSCENE};
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

use crate::ExecuteResult;
use crate::event_stream::command_variant_name;
use crate::resources::LogicalPath;
use crate::video::VideoError;

use super::super::AppState;
use super::super::CoreSystems;
use super::super::command_handlers::{apply_effect_requests, handle_audio_command};
use super::super::save::return_to_title_from_game;
use super::super::script_loader::collect_prefetch_paths;

/// 跳过场景效果时使用的大 dt（保证单步完成）
const SKIP_LARGE_DT: f32 = 999.0;

/// 一次性跳过所有活跃的演出效果（动画/过渡/场景过渡/打字机）
///
/// 用于 Skip 模式，确保所有效果正确收敛：
/// - 角色动画：skip_all + 清理淡出完成的角色
/// - 背景过渡（changeBG dissolve）：直接完成
/// - 场景过渡（changeScene）：完全跳过并切换背景
/// - 打字机：立即完成
pub fn skip_all_active_effects(core: &mut CoreSystems) {
    // 1. 跳过所有角色动画
    if core.animation_system.has_active_animations() {
        core.animation_system.skip_all();
        // update(0.0) 将已跳过的动画状态刷新到对象；返回值为"是否仍有活跃动画"，此处不需要
        let _ = core.animation_system.update(0.0);
        super::cleanup_fading_characters(core);
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

    // 4. 跳过场景效果（shake/blur）
    if core.renderer.is_scene_effect_active() {
        core.renderer
            .update_scene_effects(SKIP_LARGE_DT, &mut core.render_state.scene_effect);
    }

    // 5. 跳过标题字卡
    if core.render_state.title_card.is_some() {
        core.render_state.title_card = None;
    }

    // 6. 完成打字机
    if !core.render_state.is_dialogue_complete() {
        core.render_state.complete_typewriter();
    }
}

/// 处理脚本模式下的输入
pub fn handle_script_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    app_state
        .event_stream
        .on_input_received(&format!("{input:?}"));

    // 如果有动画正在进行，跳过所有动画
    if app_state.core.animation_system.has_active_animations() {
        app_state.core.animation_system.skip_all();
        // 同上：刷新跳过后的状态，返回值不需要
        let _ = app_state.core.animation_system.update(0.0);

        // 清理淡出完成的角色
        super::cleanup_fading_characters(&mut app_state.core);
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

    // 如果对话处于内联点击等待（{wait}），跳过当前等待点继续打字
    if app_state.core.render_state.is_inline_click_wait() {
        app_state.core.render_state.clear_inline_wait();
        return;
    }

    // 如果对话正在打字（含定时内联等待），先完成打字
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

            let node_index = app_state
                .session
                .vn_runtime
                .as_ref()
                .map(|r| r.state().position.node_index)
                .unwrap_or(0);
            app_state.event_stream.on_script_tick(
                node_index,
                commands.len(),
                &format!("{waiting:?}"),
            );

            // 收集命令中的资源路径（用于预取统计）
            let prefetch_paths = collect_prefetch_paths(&commands);
            if !prefetch_paths.is_empty() {
                debug!(paths = ?prefetch_paths, "预取资源");
            }

            // 执行所有命令
            for command in &commands {
                debug!(command = ?command, "执行命令");

                app_state
                    .event_stream
                    .on_command_produced(&command_variant_name(command), &format!("{command:?}"));

                // FullRestart：持久化 persistent_variables，清空会话，返回标题
                if matches!(command, Command::FullRestart) {
                    info!("收到 FullRestart 命令，持久化变量并返回标题");
                    let persistent_vars = app_state
                        .session
                        .vn_runtime
                        .as_ref()
                        .map(|r| r.state().persistent_variables.clone())
                        .unwrap_or_default();
                    app_state.persistent_store.merge_from(&persistent_vars);
                    app_state.persistent_store.save_or_log();
                    return_to_title_from_game(app_state, false);
                    return;
                }

                // RequestUI：按 mode 分发处理
                if let Command::RequestUI {
                    key, mode, params, ..
                } = command
                {
                    if mode == "call_game" {
                        #[cfg(feature = "mini-games")]
                        {
                            if let Some(game_id) = params.get("game_id").and_then(|v| {
                                if let vn_runtime::state::VarValue::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            }) {
                                info!(
                                    game_id = %game_id,
                                    "callGame: 设置待启动小游戏请求"
                                );
                                app_state.pending_game_launch =
                                    Some(crate::game_mode::PendingGameLaunch {
                                        game_id,
                                        request_key: key.clone(),
                                        params: params.clone(),
                                    });
                            }
                        }
                        #[cfg(not(feature = "mini-games"))]
                        {
                            warn!("callGame 需要 mini-games feature，当前未启用");
                            app_state
                                .input_manager
                                .inject_input(RuntimeInput::ui_result(
                                    key.clone(),
                                    vn_runtime::state::VarValue::String(String::new()),
                                ));
                        }
                    } else if mode == "show_map"
                        && let Some(map_id) = params.get("map_id").and_then(|v| {
                            if let vn_runtime::state::VarValue::String(s) = v {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                    {
                        let map_path = format!("maps/{}.json", map_id);
                        let logical = LogicalPath::new(&map_path);
                        match app_state.core.resource_manager.read_text(&logical) {
                            Ok(json_text) => {
                                match serde_json::from_str::<crate::ui::map::MapDefinition>(
                                    &json_text,
                                ) {
                                    Ok(definition) => {
                                        let map_state = crate::ui::map::MapDisplayState::new(
                                            definition,
                                            key.clone(),
                                        );
                                        app_state.core.render_state.map_display = Some(map_state);
                                        debug!(map_id, "地图加载成功");
                                    }
                                    Err(e) => {
                                        warn!(
                                            map_id,
                                            error = %e,
                                            "地图 JSON 解析失败"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(
                                    map_id,
                                    error = %e,
                                    "地图文件加载失败"
                                );
                            }
                        }
                    }
                    continue;
                }

                // Cutscene：启动视频播放器，duck BGM
                if let Command::Cutscene { path } = command {
                    info!(path = %path, "收到 Cutscene 命令，启动视频播放");
                    match resolve_video_path(app_state, path) {
                        Ok((resolved_path, temp_file)) => {
                            match app_state.core.video_player.start(&resolved_path, temp_file) {
                                Ok(()) => {
                                    if let Some(ref mut audio) = app_state.core.audio_manager {
                                        audio.duck();
                                    }
                                }
                                Err(e) => {
                                    warn!(error = %e, "视频播放启动失败，跳过 cutscene");
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "视频路径解析失败，跳过 cutscene");
                        }
                    }
                    continue;
                }

                let result = app_state.core.command_executor.execute(
                    command,
                    &mut app_state.core.render_state,
                    &app_state.core.resource_manager,
                );
                let effect_count = app_state
                    .core
                    .command_executor
                    .last_output
                    .effect_requests
                    .len();
                if effect_count > 0 {
                    debug!(command = ?command, effect_count, "命令产出效果请求");
                }

                // 应用动画/过渡效果请求（统一入口）
                apply_effect_requests(
                    &app_state.extension_registry,
                    &mut app_state.core,
                    &app_state.session.manifest,
                );

                // 处理音频命令
                if let Some(ref audio_cmd) =
                    app_state.core.command_executor.last_output.audio_command
                {
                    let audio_action = format!("{audio_cmd:?}")
                        .split_once(['(', '{', ' '])
                        .map(|(n, _)| n.to_string())
                        .unwrap_or_else(|| format!("{audio_cmd:?}"));
                    let audio_path = match audio_cmd {
                        crate::command_executor::AudioCommand::PlayBgm { path, .. } => {
                            Some(path.as_str())
                        }
                        crate::command_executor::AudioCommand::PlaySfx { path } => {
                            Some(path.as_str())
                        }
                        _ => None,
                    };
                    app_state
                        .event_stream
                        .on_audio_event(&audio_action, audio_path, None);
                }
                handle_audio_command(&mut app_state.core, &app_state.config);

                app_state
                    .event_stream
                    .on_command_executed(&command_variant_name(command), &format!("{result:?}"));

                if let ExecuteResult::Error(e) = result {
                    error!(error = %e, "命令执行失败");
                }
            }

            // 更新等待状态
            let old_waiting =
                std::mem::replace(&mut app_state.session.waiting_reason, waiting.clone());
            if std::mem::discriminant(&old_waiting) != std::mem::discriminant(&waiting) {
                app_state.event_stream.on_state_changed(
                    "waiting_reason",
                    &format!("{old_waiting:?}"),
                    &format!("{waiting:?}"),
                );
            }

            // 如果是选择等待，重置选择索引
            if let WaitingReason::WaitForChoice { choice_count } = &waiting {
                app_state.input_manager.reset_choice(*choice_count);
            }

            // 如果是时间等待，初始化 wait_timer
            if let WaitingReason::WaitForTime(duration) = &waiting {
                app_state.session.wait_timer = duration.as_secs_f32();
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

/// 结束 cutscene 播放，发送信号恢复 Runtime 并 unduck BGM。
pub fn finish_cutscene(app_state: &mut AppState) {
    app_state.core.video_player.cleanup();
    if let Some(ref mut audio) = app_state.core.audio_manager {
        audio.unduck();
    }
    run_script_tick(app_state, Some(RuntimeInput::signal(SIGNAL_CUTSCENE)));
}

/// 解析视频路径为真实文件系统路径。
///
/// 通过 `ResourceManager::materialize_to_fs` 统一处理 FS/ZIP 模式。
fn resolve_video_path(
    app_state: &mut AppState,
    logical_path: &str,
) -> Result<(PathBuf, Option<PathBuf>), VideoError> {
    let path = LogicalPath::new(logical_path);
    let temp_dir = std::env::temp_dir().join("ring-vn-video");

    app_state
        .core
        .resource_manager
        .materialize_to_fs(&path, &temp_dir)
        .map_err(|e| VideoError::FileNotFound(format!("{} ({})", path, e)))
}
