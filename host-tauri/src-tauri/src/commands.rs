use tauri::{AppHandle, State, command};
use vn_runtime::{AudioState, SaveData};

use crate::config::AppConfig;
use crate::render_state::PlaybackMode;
use crate::render_state::RenderState;
use crate::save_manager::SaveInfo;
use crate::state::{AppState, HistoryEntry, UserSettings};

/// 初始化游戏——解析脚本并返回初始渲染状态
///
/// `script_path` 优先通过 ResourceManager 解析（相对于 assets_root），
/// 回退为直接文件系统路径。
#[command]
pub fn init_game(state: State<AppState>, script_path: String) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;

    if inner.services.is_some() {
        inner.init_game_from_resource(&script_path)?;
    } else {
        let content = std::fs::read_to_string(&script_path)
            .map_err(|e| format!("读取脚本文件失败 '{script_path}': {e}"))?;
        inner.init_game(&content)?;
    }
    Ok(inner.render_state.clone())
}

/// 每帧 tick——推进打字机和计时器，返回最新渲染状态
#[command]
pub fn tick(state: State<AppState>, dt: f32) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.process_tick(dt);
    Ok(inner.render_state.clone())
}

/// 处理用户点击
#[command]
pub fn click(state: State<AppState>) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.process_click();
    Ok(inner.render_state.clone())
}

/// 处理用户选择
#[command]
pub fn choose(state: State<AppState>, index: usize) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.process_choose(index);
    Ok(inner.render_state.clone())
}

/// 获取当前渲染状态快照
#[command]
pub fn get_render_state(state: State<AppState>) -> Result<RenderState, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.render_state.clone())
}

// ── 存档 ─────────────────────────────────────────────────────────────────────

/// 保存游戏到指定槽位
#[command]
pub fn save_game(state: State<AppState>, slot: u32) -> Result<(), String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;

    let svc = inner.services();
    let rt = inner.runtime.as_ref().ok_or("游戏未启动")?;
    let runtime_state = rt.state().clone();
    let mut save_data = SaveData::new(slot, runtime_state).with_history(rt.history().clone());

    if let Some(ref cm) = inner.render_state.chapter_mark {
        save_data = save_data.with_chapter(&cm.title);
    }
    save_data = save_data.with_audio(AudioState {
        current_bgm: svc.audio.current_bgm_path().map(|s| s.to_string()),
        bgm_looping: true,
    });

    svc.saves.save(&save_data).map_err(|e| e.to_string())
}

/// 加载指定槽位的存档
#[command]
pub fn load_game(state: State<AppState>, slot: u32) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;

    let save_data = inner
        .services()
        .saves
        .load(slot)
        .map_err(|e| e.to_string())?;

    inner.restore_from_save(save_data)?;
    Ok(inner.render_state.clone())
}

/// 列出所有存档信息
#[command]
pub fn list_saves(state: State<AppState>) -> Result<Vec<SaveInfo>, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;

    let svc = inner.services();
    let saves = svc.saves.list_saves();
    let infos: Vec<SaveInfo> = saves
        .iter()
        .filter_map(|(slot, _)| svc.saves.get_save_info(*slot))
        .collect();
    Ok(infos)
}

/// 删除指定槽位的存档
#[command]
pub fn delete_save(state: State<AppState>, slot: u32) -> Result<(), String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner
        .services()
        .saves
        .delete(slot)
        .map_err(|e| e.to_string())
}

// ── 配置 ─────────────────────────────────────────────────────────────────────

/// 获取资源根目录的绝对路径（前端用 convertFileSrc 转换为可访问 URL）
#[command]
pub fn get_assets_root(state: State<AppState>) -> Result<String, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner
        .services()
        .resources
        .base_path()
        .to_string_lossy()
        .to_string())
}

/// 获取当前配置
#[command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.services().config.clone())
}

// ── 用户设置 ─────────────────────────────────────────────────────────────────

/// 获取用户设置
#[command]
pub fn get_user_settings(state: State<AppState>) -> Result<UserSettings, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.user_settings.clone())
}

/// 更新用户设置
#[command]
pub fn update_settings(state: State<AppState>, settings: UserSettings) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;

    inner.text_speed = settings.text_speed;

    let svc = inner.services_mut();
    svc.audio.set_bgm_volume(settings.bgm_volume / 100.0);
    svc.audio.set_sfx_volume(settings.sfx_volume / 100.0);

    inner.user_settings = settings;
    Ok(())
}

// ── 历史记录 ─────────────────────────────────────────────────────────────────

/// 获取对话历史
#[command]
pub fn get_history(state: State<AppState>) -> Result<Vec<HistoryEntry>, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.history.clone())
}

// ── 游戏流程控制 ─────────────────────────────────────────────────────────────

/// 返回标题画面
#[command]
pub fn return_to_title(state: State<AppState>) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.return_to_title();
    Ok(())
}

/// 继续游戏（加载 continue 存档）
#[command]
pub fn continue_game(state: State<AppState>) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;

    let svc = inner.services();
    if !svc.saves.has_continue() {
        return Err("没有 continue 存档".to_string());
    }
    let save_data = svc.saves.load_continue().map_err(|e| e.to_string())?;

    inner.restore_from_save(save_data)?;
    Ok(inner.render_state.clone())
}

/// 退出应用
#[command]
pub fn quit_game(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

// ── 视频过场 ─────────────────────────────────────────────────────────────────

/// 前端视频播放完成（或被跳过）后调用
#[command]
pub fn finish_cutscene(state: State<AppState>) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.finish_cutscene();
    Ok(inner.render_state.clone())
}

// ── 快照回退 ─────────────────────────────────────────────────────────────────

/// 回退到上一个快照（Backspace）
#[command]
pub fn backspace(state: State<AppState>) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    if inner.restore_snapshot() {
        Ok(inner.render_state.clone())
    } else {
        Err("没有可回退的快照".to_string())
    }
}

// ── 播放模式 ─────────────────────────────────────────────────────────────────

/// 设置播放模式
#[command]
pub fn set_playback_mode(state: State<AppState>, mode: String) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.playback_mode = match mode.as_str() {
        "auto" => PlaybackMode::Auto,
        "skip" => PlaybackMode::Skip,
        _ => PlaybackMode::Normal,
    };
    inner.auto_timer = 0.0;
    inner.render_state.playback_mode = inner.playback_mode.clone();
    Ok(())
}

/// 获取当前播放模式
#[command]
pub fn get_playback_mode(state: State<AppState>) -> Result<String, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    let mode = match inner.playback_mode {
        PlaybackMode::Normal => "normal",
        PlaybackMode::Auto => "auto",
        PlaybackMode::Skip => "skip",
    };
    Ok(mode.to_string())
}

// ── 前端生命周期 ─────────────────────────────────────────────────────────────

/// 前端（重新）连接通知——重置后端会话状态，确保无残留音频或游戏状态。
///
/// 前端 mount 时调用。覆盖浏览器刷新、WebView 重建、HMR 热重载等场景。
#[command]
pub fn frontend_connected(state: State<AppState>) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.return_to_title();
    Ok(())
}

// ── 前端日志转发 ─────────────────────────────────────────────────────────────

/// 接收前端日志并输出到 Rust tracing
#[command]
pub fn log_frontend(level: String, module: String, message: String, data: Option<String>) {
    let data_str = data.as_deref().unwrap_or("");
    match level.as_str() {
        "error" => tracing::error!(target: "frontend", module = %module, "{message} {data_str}"),
        "warn" => tracing::warn!(target: "frontend", module = %module, "{message} {data_str}"),
        "info" => tracing::info!(target: "frontend", module = %module, "{message} {data_str}"),
        _ => tracing::debug!(target: "frontend", module = %module, "{message} {data_str}"),
    }
}

// ── UI 配置 ──────────────────────────────────────────────────────────────────

/// 返回 screens.json 全文（按钮/动作/条件可见性定义）
#[command]
pub fn get_screen_definitions(state: State<AppState>) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    let rm = &inner.services().resources;
    let path = crate::resources::LogicalPath::new("ui/screens.json");
    let text = rm.read_text(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("screens.json 解析失败: {e}"))
}

/// 返回 layout.json 中的 assets（素材 key → 逻辑路径）和 colors 部分
#[command]
pub fn get_ui_assets(state: State<AppState>) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    let rm = &inner.services().resources;
    let path = crate::resources::LogicalPath::new("ui/layout.json");
    let text = rm.read_text(&path).map_err(|e| e.to_string())?;
    let full: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("layout.json 解析失败: {e}"))?;
    Ok(serde_json::json!({
        "assets": full.get("assets").cloned().unwrap_or(serde_json::Value::Object(Default::default())),
        "colors": full.get("colors").cloned().unwrap_or(serde_json::Value::Object(Default::default())),
    }))
}

/// 返回 UI 条件求值上下文（screens.json 的 visible 条件所需）
#[command]
pub fn get_ui_condition_context(state: State<AppState>) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    let svc = inner.services();
    let has_continue = svc.saves.has_continue();
    let persistent: serde_json::Map<String, serde_json::Value> = inner
        .persistent_store
        .variables
        .iter()
        .map(|(k, v)| {
            let json_val = match v {
                vn_runtime::state::VarValue::Bool(b) => serde_json::Value::Bool(*b),
                vn_runtime::state::VarValue::Int(i) => serde_json::json!(*i),
                vn_runtime::state::VarValue::Float(f) => serde_json::json!(*f),
                vn_runtime::state::VarValue::String(s) => serde_json::Value::String(s.clone()),
            };
            (k.clone(), json_val)
        })
        .collect();
    Ok(serde_json::json!({
        "has_continue": has_continue,
        "persistent": persistent,
    }))
}

// ── 调试快照 ─────────────────────────────────────────────────────────────────

/// 返回完整的内部状态快照（供 Agent 调试用）
#[command]
pub fn debug_snapshot(state: State<AppState>) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "has_runtime": inner.runtime.is_some(),
        "waiting": inner.waiting,
        "script_finished": inner.script_finished,
        "render_state": inner.render_state,
        "playback_mode": format!("{:?}", inner.playback_mode),
        "history_count": inner.history.len(),
        "has_audio": inner.services.is_some(),
        "current_bgm": inner.services().audio.current_bgm_path().map(String::from),
        "user_settings": inner.user_settings,
    }))
}
