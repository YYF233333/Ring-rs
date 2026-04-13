//! Debug HTTP Server — 嵌入式 localhost REST API，供 CC/MCP 交互。
//!
//! 设计原则：
//! - **无第二 tick loop**：仅调用 `AppStateInner` 的 public mutation 方法，
//!   与 keyboard handler 同级，绝不调用 `process_tick()`。
//! - **锁外序列化**：lock → clone → unlock → serialize，最小化锁竞争。
//! - **仅 localhost**：绑定 `127.0.0.1`，不暴露到外部网络。

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::render_state::{HostScreen, PlaybackMode, RenderState};
use crate::state::{AppState, WaitingFor};

// ── 截图通道 ─────────────────────────────────────────────────────────────────

/// 截图请求：HTTP handler 发送，Dioxus UI 线程消费。
pub struct ScreenshotRequest {
    pub reply: oneshot::Sender<Result<ScreenshotData, String>>,
}

/// 截图结果
#[derive(Serialize)]
pub struct ScreenshotData {
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub data_base64: String,
}

/// 创建截图请求通道。返回 (sender, receiver)。
/// - sender 给 HTTP server（clone 到 ServerState）
/// - receiver 给 Dioxus UI spawn loop
pub fn screenshot_channel() -> (
    mpsc::Sender<ScreenshotRequest>,
    mpsc::Receiver<ScreenshotRequest>,
) {
    mpsc::channel(4)
}

// ── 共享状态 ─────────────────────────────────────────────────────────────────

/// axum 路由共享的应用状态
#[derive(Clone)]
struct ServerState {
    app: AppState,
    screenshot_tx: mpsc::Sender<ScreenshotRequest>,
}

// ── 响应类型 ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PingResponse {
    ok: bool,
    waiting: String,
    screen: String,
    script_finished: bool,
}

#[derive(Serialize)]
struct FullStateResponse {
    render_state: RenderState,
    waiting: WaitingFor,
    script_finished: bool,
    playback_mode: PlaybackMode,
    host_screen: HostScreen,
    history_count: usize,
}

#[derive(Serialize)]
struct ActionResponse {
    ok: bool,
    waiting: String,
    screen: String,
    script_finished: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

// ── 请求类型 ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChooseRequest {
    index: usize,
}

#[derive(Deserialize)]
struct AdvanceRequest {
    #[serde(default = "default_max_clicks")]
    max_clicks: usize,
}

fn default_max_clicks() -> usize {
    10
}

#[derive(Deserialize)]
struct NavigateRequest {
    screen: String,
}

#[derive(Deserialize)]
struct StartGameRequest {
    #[serde(default)]
    script: Option<String>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Deserialize)]
struct PlaybackModeRequest {
    mode: String,
}

// ── 路由 ─────────────────────────────────────────────────────────────────────

fn build_router(state: ServerState) -> Router {
    Router::new()
        // 状态查询
        .route("/api/ping", get(handle_ping))
        .route("/api/state", get(handle_state))
        .route("/api/state/dialogue", get(handle_state_dialogue))
        .route("/api/state/scene", get(handle_state_scene))
        .route("/api/state/choices", get(handle_state_choices))
        .route("/api/state/audio", get(handle_state_audio))
        // 截图
        .route("/api/screenshot", get(handle_screenshot))
        // 动作
        .route("/api/click", post(handle_click))
        .route("/api/choose", post(handle_choose))
        .route("/api/advance", post(handle_advance))
        .route("/api/navigate", post(handle_navigate))
        .route("/api/start_game", post(handle_start_game))
        .route("/api/playback_mode", post(handle_playback_mode))
        // 诊断
        .route("/api/diag/transitions", get(handle_diag_transitions))
        .route("/api/diag/typewriter", get(handle_diag_typewriter))
        .with_state(state)
}

// ── 入口 ─────────────────────────────────────────────────────────────────────

/// 启动 debug HTTP server。在 tokio 异步上下文中调用。
///
/// 此函数不会返回（除非绑定失败），应通过 `spawn` 运行。
/// `screenshot_tx` 用于向 Dioxus UI 线程请求截图。
pub async fn run(app_state: AppState, port: u16, screenshot_tx: mpsc::Sender<ScreenshotRequest>) {
    let state = ServerState {
        app: app_state,
        screenshot_tx,
    };
    let router = build_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(port, error = %e, "Debug server 绑定失败");
            return;
        }
    };
    info!(port, "Debug server 已启动: http://127.0.0.1:{port}");

    if let Err(e) = axum::serve(listener, router).await {
        error!(error = %e, "Debug server 运行错误");
    }
}

// ── Handler 辅助 ────────────────────────────────────────────────────────────

/// 获取应用状态锁。Mutex 毒化意味着引擎已 panic（即我们的 bug），此处断言不变量。
fn lock_inner(state: &ServerState) -> std::sync::MutexGuard<'_, crate::state::AppStateInner> {
    state
        .app
        .inner
        .lock()
        .expect("invariant: app state mutex not poisoned")
}

/// 从 locked inner 提取动作摘要响应。
fn action_summary(inner: &crate::state::AppStateInner) -> ActionResponse {
    ActionResponse {
        ok: true,
        waiting: format!("{:?}", inner.waiting),
        screen: format!("{:?}", inner.render_state.host_screen),
        script_finished: inner.script_finished,
    }
}

fn err_json(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            ok: false,
            error: msg.into(),
        }),
    )
}

// ── GET /api/ping ────────────────────────────────────────────────────────────

async fn handle_ping(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(PingResponse {
        ok: true,
        waiting: format!("{:?}", inner.waiting),
        screen: format!("{:?}", inner.render_state.host_screen),
        script_finished: inner.script_finished,
    })
}

// ── GET /api/state ───────────────────────────────────────────────────────────

async fn handle_state(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    let resp = FullStateResponse {
        render_state: inner.render_state.clone(),
        waiting: inner.waiting.clone(),
        script_finished: inner.script_finished,
        playback_mode: inner.playback_mode.clone(),
        host_screen: inner.render_state.host_screen.clone(),
        history_count: inner.history.len(),
    };
    drop(inner);
    Json(resp)
}

// ── GET /api/state/dialogue ──────────────────────────────────────────────────

async fn handle_state_dialogue(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(serde_json::json!({
        "dialogue": inner.render_state.dialogue,
        "text_mode": inner.render_state.text_mode,
        "nvl_entries": inner.render_state.nvl_entries,
    }))
}

// ── GET /api/state/scene ─────────────────────────────────────────────────────

async fn handle_state_scene(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(serde_json::json!({
        "current_background": inner.render_state.current_background,
        "visible_characters": inner.render_state.visible_characters,
        "scene_effect": inner.render_state.scene_effect,
        "background_transition": inner.render_state.background_transition,
        "scene_transition": inner.render_state.scene_transition,
        "title_card": inner.render_state.title_card,
        "chapter_mark": inner.render_state.chapter_mark,
    }))
}

// ── GET /api/state/choices ───────────────────────────────────────────────────

async fn handle_state_choices(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(serde_json::json!({
        "waiting_for_choice": inner.waiting == WaitingFor::Choice,
        "choices": inner.render_state.choices,
    }))
}

// ── GET /api/state/audio ─────────────────────────────────────────────────────

async fn handle_state_audio(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(serde_json::json!({
        "audio": inner.render_state.audio,
    }))
}

// ── POST /api/click ──────────────────────────────────────────────────────────

async fn handle_click(State(state): State<ServerState>) -> impl IntoResponse {
    let mut inner = lock_inner(&state);
    inner.process_click();
    Json(action_summary(&inner))
}

// ── POST /api/choose ─────────────────────────────────────────────────────────

async fn handle_choose(
    State(state): State<ServerState>,
    Json(req): Json<ChooseRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut inner = lock_inner(&state);
    if inner.waiting != WaitingFor::Choice {
        return Err(err_json(
            StatusCode::CONFLICT,
            format!("当前非选择状态，waiting={:?}", inner.waiting),
        ));
    }
    let choice_count = inner
        .render_state
        .choices
        .as_ref()
        .map(|c| c.choices.len())
        .unwrap_or(0);
    if req.index >= choice_count {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            format!(
                "选项索引 {} 超出范围 (共 {} 个选项)",
                req.index, choice_count
            ),
        ));
    }
    inner.process_choose(req.index);
    Ok(Json(action_summary(&inner)))
}

// ── POST /api/advance ────────────────────────────────────────────────────────

async fn handle_advance(
    State(state): State<ServerState>,
    Json(req): Json<AdvanceRequest>,
) -> impl IntoResponse {
    let max = req.max_clicks.min(100); // 硬上限防止意外
    let mut inner = lock_inner(&state);
    let mut clicks = 0;
    let initial_waiting = inner.waiting.clone();

    for _ in 0..max {
        // 只在 Click 等待态下推进
        if inner.waiting != WaitingFor::Click {
            break;
        }
        inner.process_click();
        clicks += 1;
        // 如果等待态变化了（变成 Choice/Signal/etc），停止
        if inner.waiting != WaitingFor::Click && inner.waiting != WaitingFor::Nothing {
            break;
        }
    }

    Json(serde_json::json!({
        "ok": true,
        "clicks_performed": clicks,
        "initial_waiting": format!("{:?}", initial_waiting),
        "current_waiting": format!("{:?}", inner.waiting),
        "screen": format!("{:?}", inner.render_state.host_screen),
        "script_finished": inner.script_finished,
    }))
}

// ── POST /api/navigate ───────────────────────────────────────────────────────

async fn handle_navigate(
    State(state): State<ServerState>,
    Json(req): Json<NavigateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let screen = match req.screen.to_lowercase().as_str() {
        "title" => HostScreen::Title,
        "ingame" | "in_game" => HostScreen::InGame,
        "ingamemenu" | "in_game_menu" => HostScreen::InGameMenu,
        "save" => HostScreen::Save,
        "load" => HostScreen::Load,
        "settings" => HostScreen::Settings,
        "history" => HostScreen::History,
        other => {
            return Err(err_json(
                StatusCode::BAD_REQUEST,
                format!("未知 screen: {other}"),
            ));
        }
    };
    let mut inner = lock_inner(&state);
    inner.set_host_screen(screen);
    Ok(Json(action_summary(&inner)))
}

// ── POST /api/start_game ─────────────────────────────────────────────────────

async fn handle_start_game(
    State(state): State<ServerState>,
    Json(req): Json<StartGameRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut inner = lock_inner(&state);

    let script_path = req
        .script
        .or_else(|| {
            inner
                .services
                .as_ref()
                .map(|s| s.config.start_script_path.clone())
        })
        .unwrap_or_default();

    let result = if let Some(label) = req.label {
        inner.init_game_from_resource_at_label(&script_path, &label)
    } else {
        inner.init_game_from_resource(&script_path)
    };

    match result {
        Ok(()) => Ok(Json(action_summary(&inner))),
        Err(e) => Err(err_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("游戏启动失败: {e}"),
        )),
    }
}

// ── POST /api/playback_mode ──────────────────────────────────────────────────

async fn handle_playback_mode(
    State(state): State<ServerState>,
    Json(req): Json<PlaybackModeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mode = match req.mode.to_lowercase().as_str() {
        "normal" => PlaybackMode::Normal,
        "auto" => PlaybackMode::Auto,
        "skip" => PlaybackMode::Skip,
        other => {
            return Err(err_json(
                StatusCode::BAD_REQUEST,
                format!("未知 playback mode: {other}"),
            ));
        }
    };
    let mut inner = lock_inner(&state);
    inner.set_playback_mode(mode);
    Ok(Json(action_summary(&inner)))
}

// ── GET /api/diag/transitions ────────────────────────────────────────────────

async fn handle_diag_transitions(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    Json(serde_json::json!({
        "background_transition_active": inner.render_state.background_transition.is_some(),
        "scene_transition_active": inner.render_state.scene_transition.is_some(),
        "cutscene_active": inner.render_state.cutscene.is_some(),
        "title_card_active": inner.render_state.title_card.is_some(),
        "background_transition": inner.render_state.background_transition,
        "scene_transition": inner.render_state.scene_transition,
    }))
}

// ── GET /api/diag/typewriter ─────────────────────────────────────────────────

async fn handle_diag_typewriter(State(state): State<ServerState>) -> impl IntoResponse {
    let inner = lock_inner(&state);
    let (visible, total, complete) = if let Some(ref d) = inner.render_state.dialogue {
        let total = d.content.chars().count();
        let visible = d.visible_chars;
        (visible, total, visible >= total)
    } else {
        (0, 0, true)
    };
    Json(serde_json::json!({
        "has_dialogue": inner.render_state.dialogue.is_some(),
        "visible_chars": visible,
        "total_chars": total,
        "complete": complete,
        "typewriter_timer": inner.typewriter_timer,
    }))
}

// ── GET /api/screenshot ──────────────────────────────────────────────────────

async fn handle_screenshot(
    State(state): State<ServerState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let (tx, rx) = oneshot::channel();
    state
        .screenshot_tx
        .send(ScreenshotRequest { reply: tx })
        .await
        .map_err(|_| err_json(StatusCode::SERVICE_UNAVAILABLE, "截图服务不可用"))?;

    match tokio::time::timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(Ok(data))) => Ok(Json(data)),
        Ok(Ok(Err(e))) => Err(err_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("截图失败: {e}"),
        )),
        Ok(Err(_)) => Err(err_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            "截图响应通道关闭",
        )),
        Err(_) => Err(err_json(StatusCode::GATEWAY_TIMEOUT, "截图超时 (10s)")),
    }
}

// ── 截图 JS 代码 ────────────────────────────────────────────────────────────

/// 注入到 WebView 中的截图 JS 代码。
///
/// 使用 html2canvas（动态加载），捕获 `.game-container` 元素。
/// 返回 JSON: `{ width, height, data }` 其中 data 是 base64 PNG。
pub const SCREENSHOT_JS: &str = r#"
(async function() {
    try {
        // 动态加载 html2canvas（仅首次）
        if (!window.__html2canvasLoaded) {
            await new Promise((resolve, reject) => {
                const script = document.createElement('script');
                script.src = 'https://cdnjs.cloudflare.com/ajax/libs/html2canvas/1.4.1/html2canvas.min.js';
                script.onload = () => { window.__html2canvasLoaded = true; resolve(); };
                script.onerror = (e) => reject(new Error('html2canvas load failed'));
                document.head.appendChild(script);
            });
        }

        const container = document.querySelector('.game-container');
        if (!container) {
            dioxus.send({ error: 'game-container not found' });
            return;
        }

        const canvas = await html2canvas(container, {
            useCORS: true,
            allowTaint: false,
            scale: 1,
            width: 1920,
            height: 1080,
            logging: false,
        });

        const dataUrl = canvas.toDataURL('image/png');
        // 去掉 "data:image/png;base64," 前缀
        const base64 = dataUrl.split(',')[1];
        dioxus.send({ width: canvas.width, height: canvas.height, data: base64 });
    } catch (e) {
        dioxus.send({ error: e.message || String(e) });
    }
})();
"#;
