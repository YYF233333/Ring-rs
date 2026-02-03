//! 更新逻辑

use crate::renderer::RenderState;
use crate::screens::history::HistoryAction;
use crate::screens::ingame_menu::InGameMenuAction;
use crate::screens::save_load::SaveLoadAction;
use crate::screens::settings::SettingsAction;
use crate::screens::title::TitleAction;
use crate::{AppMode, SaveLoadTab};
use macroquad::prelude::*;
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

use super::command_handlers::{
    apply_transition_effect, handle_audio_command, handle_character_animation,
    handle_scene_transition,
};
use super::save::{
    load_continue, load_game, quick_load, quick_save, return_to_title_from_game, start_new_game,
};
use super::script_loader::collect_prefetch_paths;
use super::{AppState, USER_SETTINGS_PATH};
use crate::ExecuteResult;

/// 更新场景过渡状态（基于 AnimationSystem）
///
/// 多阶段流程由 SceneTransitionManager 管理：
/// - Fade/FadeWhite: FadeIn → FadeOut → UIFadeIn → Completed
/// - Rule: FadeIn → Blackout → FadeOut → UIFadeIn → Completed
pub fn update_scene_transition(
    renderer: &mut crate::Renderer,
    render_state: &mut RenderState,
    dt: f32,
) {
    // 记录过渡开始前的状态
    let was_active = renderer.is_scene_transition_active();

    if !was_active {
        return;
    }

    // 更新场景过渡
    renderer.update_scene_transition(dt);

    // 在中间点时切换背景
    if renderer.is_scene_transition_at_midpoint() {
        if let Some(path) = renderer.take_pending_background() {
            render_state.set_background(path);
        }
    }

    // 当进入 UI 淡入阶段时，恢复 UI 可见性
    if renderer.is_scene_transition_ui_fading_in() && !render_state.ui_visible {
        render_state.ui_visible = true;
    }

    // 过渡完成时恢复 UI（包括被跳过的情况）
    if !renderer.is_scene_transition_active() {
        render_state.ui_visible = true;
    }
}

/// 更新逻辑
pub fn update(app_state: &mut AppState) {
    let dt = get_frame_time();

    // 更新 UI 上下文
    app_state.ui_context.update();

    // 更新 Toast
    app_state.toast_manager.update(dt);

    // 切换调试模式（全局可用）
    if is_key_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
        println!(
            "🔧 调试模式: {}",
            if app_state.host_state.debug_mode {
                "开启"
            } else {
                "关闭"
            }
        );
    }

    // 根据当前模式处理更新
    let current_mode = app_state.navigation.current();
    match current_mode {
        AppMode::Title => update_title(app_state),
        AppMode::InGame => update_ingame(app_state, dt),
        AppMode::InGameMenu => update_ingame_menu(app_state),
        AppMode::SaveLoad => update_save_load(app_state),
        AppMode::Settings => update_settings(app_state),
        AppMode::History => update_history(app_state),
    }

    // 游戏进行时的通用更新（过渡效果、音频等）
    if current_mode.is_in_game() {
        // 更新过渡效果
        app_state.command_executor.update_transition(dt);
        app_state.renderer.update_transition(dt);

        // 更新场景过渡状态（基于动画系统）
        update_scene_transition(&mut app_state.renderer, &mut app_state.render_state, dt);

        // 更新动画系统
        let _events = app_state.animation_system.update(dt);

        // 检测淡出完成的角色并移除
        let completed_fadeouts: Vec<String> = app_state
            .render_state
            .visible_characters
            .iter()
            .filter(|(_alias, char)| {
                // 检查角色是否标记为淡出且透明度已降到 0
                if char.fading_out {
                    let alpha = char.anim.alpha();
                    alpha <= 0.01
                } else {
                    false
                }
            })
            .map(|(alias, _)| alias.clone())
            .collect();

        // 移除淡出完成的角色，并从动画系统注销
        for alias in &completed_fadeouts {
            if let Some(object_id) = app_state.character_object_ids.remove(alias) {
                app_state.animation_system.unregister(object_id);
            }
        }
        app_state
            .render_state
            .remove_fading_out_characters(&completed_fadeouts);
    }

    // 更新音频状态（所有模式都需要）
    if let Some(ref mut audio_manager) = app_state.audio_manager {
        audio_manager.update(dt);
    }
}

/// 更新主标题界面
fn update_title(app_state: &mut AppState) {
    // 初始化界面
    if app_state.title_screen.needs_init() {
        app_state.title_screen.init(
            &app_state.save_manager,
            &app_state.ui_context.theme,
            app_state.ui_context.screen_width,
            app_state.ui_context.screen_height,
        );
    }

    // 处理用户操作
    match app_state.title_screen.update(&app_state.ui_context) {
        TitleAction::StartGame => {
            // 开始新游戏时删除旧的 Continue 存档
            let _ = app_state.save_manager.delete_continue();
            start_new_game(app_state);
        }
        TitleAction::Continue => {
            // 读取专用 Continue 存档
            if app_state.title_screen.has_continue() {
                load_continue(app_state);
            }
        }
        TitleAction::LoadGame => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        TitleAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        TitleAction::Exit => {
            app_state.host_state.stop();
        }
        TitleAction::None => {}
    }
}

/// 更新游戏进行中
fn update_ingame(app_state: &mut AppState, dt: f32) {
    // ESC 打开系统菜单
    if is_key_pressed(KeyCode::Escape) {
        app_state.ingame_menu.mark_needs_init();
        app_state.navigation.navigate_to(AppMode::InGameMenu);
        return;
    }

    // 开发者快捷键（后续考虑 feature gate）
    #[cfg(debug_assertions)]
    {
        if is_key_pressed(KeyCode::F5) {
            quick_save(app_state);
        }
        if is_key_pressed(KeyCode::F9) {
            quick_load(app_state);
        }
    }

    // 使用 InputManager 处理游戏输入（传入 dt 用于长按快进）
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.waiting_reason, dt)
    {
        handle_script_mode_input(app_state, input);
    }

    // 同步选择索引到 RenderState
    if let Some(ref mut choices) = app_state.render_state.choices {
        let choice_rects = app_state.renderer.get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        choices.selected_index = app_state.input_manager.selected_index;
        choices.hovered_index = app_state.input_manager.hovered_index;
    }

    // 更新打字机效果
    if let Some(ref dialogue) = app_state.render_state.dialogue {
        if !dialogue.is_complete {
            app_state.typewriter_timer += dt * app_state.user_settings.text_speed;
            while app_state.typewriter_timer >= 1.0 {
                app_state.typewriter_timer -= 1.0;
                if app_state.render_state.advance_typewriter() {
                    break;
                }
            }
        }
    }
}

/// 更新游戏内菜单
fn update_ingame_menu(app_state: &mut AppState) {
    if app_state.ingame_menu.needs_init() {
        app_state.ingame_menu.init(&app_state.ui_context);
    }

    match app_state.ingame_menu.update(&app_state.ui_context) {
        InGameMenuAction::Resume => {
            app_state.navigation.go_back();
        }
        InGameMenuAction::Save => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Save);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Load => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        InGameMenuAction::History => {
            app_state.history_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::History);
        }
        InGameMenuAction::ReturnToTitle => {
            // 用户主动返回，保存 Continue 存档
            return_to_title_from_game(app_state, true);
        }
        InGameMenuAction::Exit => {
            app_state.host_state.stop();
        }
        InGameMenuAction::None => {}
    }
}

/// 更新存档/读档界面
fn update_save_load(app_state: &mut AppState) {
    if app_state.save_load_screen.needs_init() {
        app_state
            .save_load_screen
            .init(&app_state.ui_context, &app_state.save_manager);
    }
    if app_state.save_load_screen.needs_refresh() {
        app_state
            .save_load_screen
            .refresh_saves(&app_state.save_manager);
    }

    match app_state.save_load_screen.update(&app_state.ui_context) {
        SaveLoadAction::Back => {
            app_state.navigation.go_back();
        }
        SaveLoadAction::Save(slot) => {
            app_state.current_save_slot = slot;
            quick_save(app_state);
            app_state
                .toast_manager
                .success(format!("已保存到槽位 {}", slot));
            app_state
                .save_load_screen
                .refresh_saves(&app_state.save_manager);
        }
        SaveLoadAction::Load(slot) => {
            load_game(app_state, slot);
            app_state
                .toast_manager
                .success(format!("已读取槽位 {}", slot));
        }
        SaveLoadAction::Delete(slot) => {
            if app_state.save_manager.delete(slot).is_ok() {
                app_state.toast_manager.info(format!("已删除槽位 {}", slot));
                app_state
                    .save_load_screen
                    .refresh_saves(&app_state.save_manager);
            } else {
                app_state.toast_manager.error("删除失败");
            }
        }
        SaveLoadAction::None => {}
    }
}

/// 更新设置界面
fn update_settings(app_state: &mut AppState) {
    if app_state.settings_screen.needs_init() {
        app_state
            .settings_screen
            .init(&app_state.ui_context, &app_state.user_settings);
    }

    match app_state.settings_screen.update(&app_state.ui_context) {
        SettingsAction::Back => {
            app_state.navigation.go_back();
        }
        SettingsAction::Apply => {
            // 应用设置
            app_state.user_settings = app_state.settings_screen.settings().clone();

            // 应用音量
            if let Some(ref mut audio) = app_state.audio_manager {
                audio.set_bgm_volume(app_state.user_settings.bgm_volume);
                audio.set_sfx_volume(app_state.user_settings.sfx_volume);
                audio.set_muted(app_state.user_settings.muted);
            }

            // 保存设置
            if let Err(e) = app_state.user_settings.save(USER_SETTINGS_PATH) {
                eprintln!("⚠️ 保存用户设置失败: {}", e);
                app_state.toast_manager.error("设置保存失败");
            } else {
                app_state.toast_manager.success("设置已保存");
            }

            app_state.navigation.go_back();
        }
        SettingsAction::None => {}
    }
}

/// 更新历史界面
fn update_history(app_state: &mut AppState) {
    if app_state.history_screen.needs_init() {
        if let Some(ref runtime) = app_state.vn_runtime {
            app_state
                .history_screen
                .init(&app_state.ui_context, runtime.history());
        }
    }

    match app_state.history_screen.update(&app_state.ui_context) {
        HistoryAction::Back => {
            app_state.navigation.go_back();
        }
        HistoryAction::None => {}
    }
}

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

    // 脚本执行完毕后已自动返回主界面，这里不再处理

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
