use tauri::{AppHandle, Manager, State, command};

use crate::config::AppConfig;
use crate::protocol::{parse_host_screen, parse_playback_mode};
use crate::render_state::{PlaybackMode, RenderState};
use crate::save_manager::SaveInfo;
use crate::state::{AppState, FrontendSession, HarnessTraceBundle, HistoryEntry, UserSettings};

/// 初始化游戏——解析脚本并返回初始渲染状态
///
/// `script_path` 优先通过 ResourceManager 解析（相对于 assets_root），
/// 回退为直接文件系统路径。
#[command]
pub fn init_game(
    state: State<AppState>,
    client_token: String,
    script_path: String,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.delete_continue()?;

    if inner.services.is_some() {
        inner.init_game_from_resource(&script_path)?;
    } else {
        let content = std::fs::read_to_string(&script_path)
            .map_err(|e| format!("读取脚本文件失败 '{script_path}': {e}"))?;
        inner.init_game(&content)?;
    }
    Ok(inner.render_state.clone())
}

#[command]
pub fn init_game_at_label(
    state: State<AppState>,
    client_token: String,
    script_path: String,
    label: String,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.delete_continue()?;
    inner.init_game_from_resource_at_label(&script_path, &label)?;
    Ok(inner.render_state.clone())
}

/// 每帧 tick——推进打字机和计时器，返回最新渲染状态
#[command]
pub fn tick(state: State<AppState>, client_token: String, dt: f32) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.process_tick(dt);
    Ok(inner.render_state.clone())
}

/// 处理用户点击
#[command]
pub fn click(state: State<AppState>, client_token: String) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.process_click();
    Ok(inner.render_state.clone())
}

/// 处理用户选择
#[command]
pub fn choose(
    state: State<AppState>,
    client_token: String,
    index: usize,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
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
pub fn save_game(state: State<AppState>, client_token: String, slot: u32) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.save_to_slot(slot)
}

/// 加载指定槽位的存档
#[command]
pub fn load_game(
    state: State<AppState>,
    client_token: String,
    slot: u32,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;

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

/// 保存游戏并附带缩略图（base64 编码的 PNG）
#[command]
pub fn save_game_with_thumbnail(
    state: State<AppState>,
    client_token: String,
    slot: u32,
    thumbnail_base64: String,
) -> Result<(), String> {
    use base64::Engine as _;

    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(&thumbnail_base64)
        .map_err(|e| format!("base64 解码失败: {e}"))?;
    inner.save_to_slot_with_thumbnail(slot, &png_bytes)
}

/// 获取指定槽位的缩略图（base64 编码的 PNG）
#[command]
pub fn get_thumbnail(state: State<AppState>, slot: u32) -> Result<Option<String>, String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.services().saves.load_thumbnail_base64(slot))
}

/// 删除指定槽位的存档
#[command]
pub fn delete_save(state: State<AppState>, client_token: String, slot: u32) -> Result<(), String> {
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
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
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    client_token: String,
    settings: UserSettings,
) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;

    inner.text_speed = settings.text_speed;

    let svc = inner.services_mut();
    svc.audio.set_bgm_volume(settings.bgm_volume / 100.0);
    svc.audio.set_sfx_volume(settings.sfx_volume / 100.0);

    let window = app.get_webview_window("main").ok_or("主窗口不存在")?;
    window
        .set_fullscreen(settings.fullscreen)
        .map_err(|e| format!("设置全屏失败: {e}"))?;

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
pub fn return_to_title(
    state: State<AppState>,
    client_token: String,
    save_continue: Option<bool>,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.return_to_title(save_continue.unwrap_or(false));
    Ok(inner.render_state.clone())
}

/// 继续游戏（加载 continue 存档）
#[command]
pub fn continue_game(state: State<AppState>, client_token: String) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;

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
pub fn finish_cutscene(
    state: State<AppState>,
    client_token: String,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.finish_cutscene();
    Ok(inner.render_state.clone())
}

// ── UI 交互结果 ──────────────────────────────────────────────────────────────

/// 前端 UI 模式完成交互后回传结果（requestUI / callGame / showMap）
#[command]
pub fn submit_ui_result(
    state: State<AppState>,
    client_token: String,
    key: String,
    value: serde_json::Value,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.handle_ui_result(key, value)?;
    Ok(inner.render_state.clone())
}

// ── 快照回退 ─────────────────────────────────────────────────────────────────

/// 回退到上一个快照（Backspace）
#[command]
pub fn backspace(state: State<AppState>, client_token: String) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    if inner.restore_snapshot() {
        Ok(inner.render_state.clone())
    } else {
        Err("没有可回退的快照".to_string())
    }
}

// ── 播放模式 ─────────────────────────────────────────────────────────────────

/// 设置播放模式
#[command]
pub fn set_playback_mode(
    state: State<AppState>,
    client_token: String,
    mode: String,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.set_playback_mode(parse_playback_mode(&mode)?);
    Ok(inner.render_state.clone())
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

/// 前端（重新）连接通知——领取 / 抢占当前会话 owner，并返回当前渲染投影。
///
/// 前端 mount 时调用。覆盖浏览器刷新、WebView 重建、HMR 热重载等场景。
#[command]
pub fn frontend_connected(
    state: State<AppState>,
    client_label: Option<String>,
) -> Result<FrontendSession, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(inner.frontend_connected(client_label))
}

#[command]
pub fn set_host_screen(
    state: State<AppState>,
    client_token: String,
    screen: String,
) -> Result<RenderState, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    inner.set_host_screen(parse_host_screen(&screen)?);
    Ok(inner.render_state.clone())
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
    let assets = full
        .get("assets")
        .cloned()
        .ok_or("layout.json 缺少 assets 字段")?;
    let colors = full
        .get("colors")
        .cloned()
        .ok_or("layout.json 缺少 colors 字段")?;
    Ok(serde_json::json!({
        "assets": assets,
        "colors": colors,
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
        "host_screen": format!("{:?}", inner.host_screen),
        "history_count": inner.history.len(),
        "has_audio": inner.services.is_some(),
        "current_bgm": inner.services().audio.current_bgm_path().map(String::from),
        "user_settings": inner.user_settings,
    }))
}

#[command]
pub fn debug_run_until(
    state: State<AppState>,
    client_token: String,
    dt: f32,
    max_steps: usize,
    stop_on_wait: Option<bool>,
    stop_on_script_finished: Option<bool>,
) -> Result<HarnessTraceBundle, String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.assert_owner(&client_token)?;
    Ok(inner.debug_run_until(
        dt,
        max_steps,
        stop_on_wait.unwrap_or(true),
        stop_on_script_finished.unwrap_or(true),
    ))
}
