//! 存档系统

use crate::renderer::RenderState;
use tracing::{error, info, warn};
use vn_runtime::state::WaitingReason;

use super::AppState;
use super::script_loader::{load_script_by_path_or_id, load_script_from_logical_path};

/// 构建当前游戏状态的存档数据
pub fn build_save_data(app_state: &AppState, slot: u32) -> Option<vn_runtime::SaveData> {
    let runtime = app_state.vn_runtime.as_ref()?;

    // 构建存档数据
    let runtime_state = runtime.state().clone();
    let mut save_data = vn_runtime::SaveData::new(slot, runtime_state);

    // 设置章节标题（如果有）
    if let Some(ref chapter) = app_state.render_state.chapter_mark {
        save_data = save_data.with_chapter(&chapter.title);
    }

    // 设置游戏时长
    let play_time = app_state.play_start_time.elapsed().as_secs();
    save_data.metadata.play_time_secs = play_time;

    // 设置音频状态
    if let Some(ref audio) = app_state.audio_manager {
        save_data = save_data.with_audio(vn_runtime::AudioState {
            current_bgm: audio.current_bgm_path().map(|s| s.to_string()),
            bgm_looping: true, // 假设 BGM 总是循环
        });
    }

    // 设置渲染快照
    let render_snapshot = vn_runtime::RenderSnapshot {
        background: app_state.render_state.current_background.clone(),
        characters: app_state
            .render_state
            .visible_characters
            .iter()
            .map(|(alias, sprite)| vn_runtime::CharacterSnapshot {
                alias: alias.clone(),
                texture_path: sprite.texture_path.clone(),
                position: format!("{:?}", sprite.position),
            })
            .collect(),
    };
    save_data = save_data.with_render(render_snapshot);

    // 设置历史记录
    save_data = save_data.with_history(runtime.history().clone());

    Some(save_data)
}

/// 快速保存（到槽位）
pub fn quick_save(app_state: &mut AppState) {
    // 检查是否有游戏状态（允许从 SaveLoad 界面保存）
    if app_state.vn_runtime.is_none() {
        warn!("只能在游戏中保存");
        return;
    }

    let slot = app_state.current_save_slot;

    let Some(save_data) = build_save_data(app_state, slot) else {
        warn!("没有可保存的游戏状态");
        return;
    };

    // 保存
    match app_state.save_manager.save(&save_data) {
        Ok(()) => info!(slot = slot, "快速保存成功"),
        Err(e) => error!(error = %e, "保存失败"),
    }
}

/// 保存 Continue 存档（用于"继续"功能）
pub fn save_continue(app_state: &mut AppState) {
    // 只在有游戏状态时保存
    if app_state.vn_runtime.is_none() {
        return;
    }

    // 使用槽位 0 作为 Continue 存档的元数据标记
    let Some(save_data) = build_save_data(app_state, 0) else {
        return;
    };

    // 保存 Continue 存档
    match app_state.save_manager.save_continue(&save_data) {
        Ok(()) => info!("Continue 存档保存成功"),
        Err(e) => warn!(error = %e, "Continue 存档保存失败"),
    }
}

/// 从存档数据恢复游戏状态
pub fn restore_from_save_data(app_state: &mut AppState, save_data: vn_runtime::SaveData) -> bool {
    // 加载对应的脚本（优先使用 script_path，回退到 script_id）
    let script_path = &save_data.runtime_state.position.script_path;
    let script_id = &save_data.runtime_state.position.script_id;

    info!(path = %script_path, id = %script_id, "尝试加载脚本");
    if !load_script_by_path_or_id(app_state, script_path, script_id) {
        error!("找不到脚本");
        // 尝试使用 start_script_path 作为后备
        info!("尝试使用 start_script_path 作为后备");
        let start_path = app_state.config.start_script_path.clone();
        if !load_script_from_logical_path(app_state, &start_path) {
            error!("后备脚本加载也失败");
            return false;
        }
    }

    // 恢复 Runtime 状态和历史记录
    if let Some(ref mut runtime) = app_state.vn_runtime {
        runtime.restore_state(save_data.runtime_state);
        runtime.restore_history(save_data.history);
    }

    // 恢复渲染状态
    app_state.render_state = RenderState::new();
    app_state.character_object_ids.clear(); // 清除旧的对象 ID 映射
    app_state.render_state.current_background = save_data.render.background;
    for char_snap in save_data.render.characters {
        // 尝试解析 position（简化处理，默认 Center）
        let position = vn_runtime::Position::Center;
        app_state.render_state.show_character(
            char_snap.alias.clone(),
            char_snap.texture_path,
            position,
        );
        // 恢复角色时设置为完全不透明（存档的角色应该是可见的）
        if let Some(anim) = app_state.render_state.get_character_anim(&char_snap.alias) {
            anim.set_alpha(1.0);
        }
    }

    // 恢复音频状态
    if let Some(ref mut audio) = app_state.audio_manager
        && let Some(ref bgm_path) = save_data.audio.current_bgm
    {
        audio.play_bgm(bgm_path, save_data.audio.bgm_looping, Some(0.5));
    }

    // 设置游戏状态
    app_state.script_finished = false;
    app_state.waiting_reason = WaitingReason::WaitForClick;
    app_state.play_start_time = std::time::Instant::now(); // 重置开始时间

    true
}

/// 快速读取（从槽位）
pub fn quick_load(app_state: &mut AppState) -> bool {
    let slot = app_state.current_save_slot;

    // 读取存档
    let save_data = match app_state.save_manager.load(slot) {
        Ok(data) => data,
        Err(e) => {
            error!(slot = slot, error = %e, "读取失败");
            return false;
        }
    };

    if restore_from_save_data(app_state, save_data) {
        info!(slot = slot, "快速读取成功");
        true
    } else {
        false
    }
}

/// 从游戏状态返回主界面
/// 用于脚本执行完毕或用户主动返回时清理状态并跳转到 Title
///
/// # 参数
/// - `should_save_continue`: 是否保存 Continue 存档。脚本执行完毕时应该为 `false`，用户主动返回时为 `true`
pub fn return_to_title_from_game(app_state: &mut AppState, should_save_continue: bool) {
    // 只在用户主动返回时保存 Continue 存档
    // 脚本执行完毕时不保存，避免下次 Continue 直接跳到末尾
    if should_save_continue {
        save_continue(app_state);
    }

    // 停止音乐
    if let Some(ref mut audio) = app_state.audio_manager {
        audio.stop_bgm(Some(0.5));
    }

    // 清理游戏状态
    app_state.vn_runtime = None;
    app_state.render_state = RenderState::new();
    app_state.script_finished = false;

    // 返回标题
    app_state.navigation.return_to_title();
    app_state.title_screen.mark_needs_init();
}

/// 开始新游戏（使用 config.start_script_path）
pub fn start_new_game(app_state: &mut AppState) {
    use super::update::run_script_tick;
    use crate::AppMode;

    // 使用配置的入口脚本（逻辑路径）
    let script_path = app_state.config.start_script_path.clone();

    if load_script_from_logical_path(app_state, &script_path) {
        app_state.render_state = RenderState::new();
        app_state.script_finished = false;
        app_state.play_start_time = std::time::Instant::now();

        // 执行第一次 tick
        run_script_tick(app_state, None);

        // 切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
        info!(script = %script_path, "开始新游戏");
    } else {
        app_state.toast_manager.error("无法加载入口脚本");
    }
}

/// 读取存档（槽位）
pub fn load_game(app_state: &mut AppState, slot: u32) {
    use crate::AppMode;

    app_state.current_save_slot = slot;
    if quick_load(app_state) {
        // 成功读档后切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
    }
}

/// 读取 Continue 存档
pub fn load_continue(app_state: &mut AppState) {
    use crate::AppMode;

    // 读取 Continue 存档
    let save_data = match app_state.save_manager.load_continue() {
        Ok(data) => data,
        Err(e) => {
            error!(error = %e, "Continue 读取失败");
            app_state.toast_manager.error("Continue 存档读取失败");
            return;
        }
    };

    // 恢复游戏状态
    if restore_from_save_data(app_state, save_data) {
        // 成功读档后切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
        info!("继续游戏");
    }
}
