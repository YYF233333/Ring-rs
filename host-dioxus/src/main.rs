#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

// ── 后端模块（Phase 1 迁移自 host-tauri，无 Tauri 依赖） ──
pub mod audio;
pub mod command_executor;
pub mod config;
pub mod error;
pub mod init;
pub mod layout_config;
pub mod manifest;
pub mod map_data;
pub mod render_state;
pub mod resources;
pub mod save_manager;
pub mod screen_defs;
pub mod state;

pub mod debug_server;

// ── 前端模块（Phase 2） ──
mod components;
mod screens;
mod vn;

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dioxus::desktop::Config;
use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::wry::http;
use dioxus::prelude::*;
use tracing::{error, info};

use components::{ConfirmDialog, PendingConfirm, SkipIndicator, ToastLayer, ToastQueue};
use render_state::{HostScreen, RenderState};
use screens::{HistoryScreen, InGameMenu, SaveLoadScreen, SettingsScreen, TitleScreen};
use state::{AppState, AppStateInner};
use vn::VNScene;

// ---------------------------------------------------------------------------
// 全局 CSS（BEM 命名，编译时嵌入，完整内容见 global.css）
// ---------------------------------------------------------------------------

const GLOBAL_CSS: &str = include_str!("global.css");

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// JS 脚本：窗口 resize 时更新 CSS `--scale-factor`。
/// 等价于 egui host 的 `ScaleContext.scale_uniform = min(w/1920, h/1080)`。
const SCALE_JS: &str = r#"
<script>
(function() {
    function updateScale() {
        var w = window.innerWidth;
        var h = window.innerHeight;
        var s = Math.min(w / 1920, h / 1080);
        document.documentElement.style.setProperty('--scale-factor', s);
    }
    window.addEventListener('resize', updateScale);
    updateScale();
})();
</script>
"#;

fn main() {
    tracing_subscriber::fmt::init();

    let css_head = format!("<style>{GLOBAL_CSS}</style>{SCALE_JS}");

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Ring Engine")
                        .with_inner_size(LogicalSize::new(1280, 720)),
                )
                .with_menu(None)
                .with_custom_head(css_head)
                .with_custom_protocol("ring-asset", ring_asset_handler),
        )
        .launch(App);
}

// ---------------------------------------------------------------------------
// ring-asset custom protocol handler
// ---------------------------------------------------------------------------

/// 小游戏完成结果的全局存储。
///
/// 游戏 iframe 通过 fetch `/__game_complete?result=xxx` 写入，
/// `MinigameOverlay` 轮询读取。
pub static GAME_COMPLETE_RESULT: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn ring_asset_handler(
    _id: dioxus::desktop::wry::WebViewId,
    request: http::Request<Vec<u8>>,
) -> http::Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();
    let raw_path = request.uri().path();
    let path_clean = percent_decode(raw_path.trim_start_matches('/'));

    // ── 小游戏完成端点（iframe 导航触发） ──
    if path_clean.starts_with("__game_complete") {
        let query = request.uri().query().unwrap_or("");
        let result = query
            .split('&')
            .find_map(|kv| {
                let (k, v) = kv.split_once('=')?;
                (k == "result").then(|| percent_decode(v))
            })
            .unwrap_or_default();
        tracing::debug!(result = %result, "game complete via ring-asset");
        if let Ok(mut slot) = GAME_COMPLETE_RESULT.lock() {
            *slot = Some(result);
        }
        // 纯黑空页面，与覆盖层背景一致，无视觉闪烁
        let body = br#"<!DOCTYPE html><html><body style="background:#000;margin:0"></body></html>"#;
        return http::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Cow::from(body.to_vec()))
            .unwrap();
    }

    // ── 虚拟 engine-sdk.js（兼容游戏显式 <script src="../../engine-sdk.js"> 加载） ──
    if path_clean == "engine-sdk.js" {
        return http::Response::builder()
            .status(200)
            .header("Content-Type", "application/javascript")
            .header("Access-Control-Allow-Origin", "*")
            .body(Cow::from(GAME_ENGINE_SDK_JS.as_bytes().to_vec()))
            .unwrap();
    }

    let assets_root = find_assets_root();
    let full_path = assets_root.join(&path_clean);

    tracing::debug!(uri = %uri, resolved = %full_path.display(), "ring-asset request");

    let mime = guess_mime(&path_clean);

    match std::fs::read(&full_path) {
        Ok(bytes) => {
            // games/*/**.html: 自动注入 engine JS SDK（postMessage 桥接）
            let body = if path_clean.starts_with("games/") && mime == "text/html" {
                let html = String::from_utf8_lossy(&bytes);
                let injected = inject_engine_sdk(&html);
                Cow::from(injected.into_bytes())
            } else {
                Cow::from(bytes)
            };
            http::Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .body(body)
                .unwrap()
        }
        Err(e) => {
            tracing::warn!(path = %path_clean, error = %e, "ring-asset 404");
            http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(Cow::from(format!("Not Found: {path_clean}").into_bytes()))
                .unwrap()
        }
    }
}

fn percent_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16)
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn find_assets_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut dir: &Path = &cwd;
    loop {
        let candidate = dir.join("assets");
        if candidate.is_dir() {
            return candidate;
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    cwd.join("assets")
}

fn guess_mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "html" => "text/html",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------------
// Game engine JS SDK injection (for callGame iframe)
// ---------------------------------------------------------------------------

/// 向游戏 HTML 注入 engine JS SDK。
///
/// 在 `<head>` 标签后（或文档开头）插入 SDK script。
/// SDK 通过 `window.parent.postMessage` 与宿主通信。
fn inject_engine_sdk(html: &str) -> String {
    let sdk_tag = format!("<script>{GAME_ENGINE_SDK_JS}</script>");
    // 在 <head> 后注入，如果没有 <head> 则在开头注入
    if let Some(pos) = html.find("<head>") {
        let insert_pos = pos + "<head>".len();
        format!("{}{sdk_tag}{}", &html[..insert_pos], &html[insert_pos..])
    } else if let Some(pos) = html.find("<HEAD>") {
        let insert_pos = pos + "<HEAD>".len();
        format!("{}{sdk_tag}{}", &html[..insert_pos], &html[insert_pos..])
    } else {
        format!("{sdk_tag}{html}")
    }
}

/// 同源 fetch-based engine JS SDK。
///
/// 提供与旧 host HTTP Bridge SDK 兼容的 `window.engine.*` API，
/// 内部使用同源 fetch 到 `ring-asset` handler 的虚拟端点通信。
///
/// 关键路径：`engine.complete(result)` → fetch `/__game_complete?result=xxx`
/// → ring-asset handler 存入 static → Rust 轮询读取。
const GAME_ENGINE_SDK_JS: &str = r#"
(function() {
    if (window.engine) return;
    window.engine = {
        complete: function(result) {
            var r = result !== undefined && result !== null ? String(result) : "";
            window.location.href = "/__game_complete?result=" + encodeURIComponent(r);
        },
        onComplete: function(result) {
            window.engine.complete(result);
        },
        playSound: function(name) {
            console.log("[engine SDK] playSound: " + name);
        },
        playBGM: function(name, shouldLoop) {
            console.log("[engine SDK] playBGM: " + name);
        },
        stopBGM: function() {
            console.log("[engine SDK] stopBGM");
        },
        getState: function(key) {
            console.warn("[engine SDK] getState not yet supported in iframe mode");
            return Promise.resolve(undefined);
        },
        setState: function(key, value) {
            console.warn("[engine SDK] setState not yet supported in iframe mode");
            return Promise.resolve();
        },
        log: function(level, message) {
            console.log("[game:" + level + "] " + message);
        }
    };

    // 旧 API 兼容：window.ipc.postMessage 映射
    if (!window.ipc) {
        window.ipc = {
            postMessage: function(jsonStr) {
                try {
                    var msg = JSON.parse(jsonStr);
                    if (msg.type === "onComplete") {
                        window.engine.complete(msg.result || msg.data);
                    }
                } catch(e) {}
            }
        };
    }
})();
"#;

// ---------------------------------------------------------------------------
// App 初始化状态
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum InitPhase {
    Loading,
    Ready,
    Error(String),
}

// ---------------------------------------------------------------------------
// Screenshot bridge (debug server ↔ WebView)
// ---------------------------------------------------------------------------

/// 接收来自 debug HTTP server 的截图请求，通过 `document::eval()` 在 WebView 中
/// 执行 JS 截图代码，将结果通过 oneshot 通道回传。
async fn screenshot_bridge(mut rx: tokio::sync::mpsc::Receiver<debug_server::ScreenshotRequest>) {
    while let Some(req) = rx.recv().await {
        // 每个请求启动独立 eval，避免阻塞后续请求
        spawn(async move {
            let mut eval = document::eval(debug_server::SCREENSHOT_JS);
            match eval.recv().await {
                Ok(msg) => {
                    let result: serde_json::Value = msg;
                    if let Some(err) = result.get("error").and_then(|v| v.as_str()) {
                        // receiver 可能已因超时而被丢弃，忽略发送失败
                        let _ = req.reply.send(Err(err.to_string()));
                    } else {
                        let width =
                            result.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
                        let height = result
                            .get("height")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1080) as u32;
                        let data = result
                            .get("data")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        // receiver 可能已因超时而被丢弃，忽略发送失败
                        let _ = req.reply.send(Ok(debug_server::ScreenshotData {
                            format: "png".to_string(),
                            width,
                            height,
                            data_base64: data,
                        }));
                    }
                }
                Err(e) => {
                    // receiver 可能已因超时而被丢弃，忽略发送失败
                    let _ = req.reply.send(Err(format!("eval 失败: {e}")));
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

fn App() -> Element {
    // 全局 AppState：Arc<Mutex<AppStateInner>>
    let app_state = use_context_provider(|| AppState {
        inner: Arc::new(Mutex::new(AppStateInner::new())),
    });

    // 确认弹窗状态（全局 Signal，各页面共享）
    let _pending_confirm: Signal<Option<PendingConfirm>> =
        use_context_provider(|| Signal::new(None));

    // Toast 队列（全局 Signal）
    let _toast_queue: Signal<ToastQueue> =
        use_context_provider(|| Signal::new(ToastQueue::default()));

    // 初始化阶段
    let mut init_phase = use_signal(|| InitPhase::Loading);

    // RenderState signal：tick loop 每帧更新
    let mut render_state = use_signal(RenderState::new);

    // 初始化后端子系统（仅首次 mount）
    let app_state_init = app_state.clone();
    use_hook(move || {
        spawn(async move {
            let result = {
                let mut inner = app_state_init
                    .inner
                    .lock()
                    .expect("invariant: app state mutex not poisoned");
                init::initialize_inner(&mut inner)
            };
            match result {
                Ok(()) => {
                    let debug_port = {
                        let mut inner = app_state_init
                            .inner
                            .lock()
                            .expect("invariant: app state mutex not poisoned");
                        inner.frontend_connected(Some("dioxus-desktop".to_string()));
                        inner
                            .services
                            .as_ref()
                            .map(|s| s.config.debug.resolve_debug_server())
                            .unwrap_or(None)
                    };
                    if let Some(port) = debug_port {
                        let (screenshot_tx, screenshot_rx) = debug_server::screenshot_channel();
                        // HTTP server
                        let app_for_debug = app_state_init.clone();
                        spawn(async move {
                            debug_server::run(app_for_debug, port, screenshot_tx).await;
                        });
                        // 截图桥接：接收 HTTP 请求，通过 document::eval 执行 JS 截图
                        spawn(screenshot_bridge(screenshot_rx));
                    }
                    info!("后端初始化完成");
                    init_phase.set(InitPhase::Ready);
                }
                Err(e) => {
                    error!(error = %e, "后端初始化失败");
                    init_phase.set(InitPhase::Error(e.to_string()));
                }
            }
        });
    });

    // Tick loop：~30 FPS
    let app_state_tick = app_state.clone();
    use_hook(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(33)).await;
                if let Ok(mut inner) = app_state_tick.inner.lock() {
                    inner.process_tick(1.0 / 30.0);
                    render_state.set(inner.render_state.clone());
                }
            }
        });
    });

    // 键盘绑定：JS 监听 → dioxus.send() → Rust recv() 处理
    let app_state_keys = app_state.clone();
    use_hook(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                document.addEventListener("keydown", function(e) {
                    dioxus.send({ type: "down", key: e.key, code: e.code });
                    if (["Escape", " ", "Enter", "Control", "Backspace"].includes(e.key)) {
                        e.preventDefault();
                    }
                });
                document.addEventListener("keyup", function(e) {
                    dioxus.send({ type: "up", key: e.key, code: e.code });
                });
                "#,
            );

            loop {
                let msg: Result<serde_json::Value, _> = eval.recv().await;
                let Ok(msg) = msg else { break };

                let event_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let key = msg.get("key").and_then(|v| v.as_str()).unwrap_or("");

                if let Ok(mut inner) = app_state_keys.inner.lock() {
                    match (event_type, key) {
                        ("down", "Escape") => {
                            let screen = inner.render_state.host_screen.clone();
                            match screen {
                                HostScreen::InGame => {
                                    inner.set_host_screen(HostScreen::InGameMenu);
                                }
                                HostScreen::InGameMenu => {
                                    inner.set_host_screen(HostScreen::InGame);
                                }
                                HostScreen::Save
                                | HostScreen::Load
                                | HostScreen::Settings
                                | HostScreen::History => {
                                    inner.set_host_screen(HostScreen::InGame);
                                }
                                _ => {}
                            }
                        }
                        ("down", " ") | ("down", "Enter")
                            if inner.render_state.host_screen == HostScreen::InGame =>
                        {
                            inner.process_click();
                        }
                        ("down", "Control")
                            if inner.render_state.host_screen == HostScreen::InGame =>
                        {
                            inner.set_playback_mode(render_state::PlaybackMode::Skip);
                        }
                        ("up", "Control")
                            if inner.playback_mode == render_state::PlaybackMode::Skip =>
                        {
                            inner.set_playback_mode(render_state::PlaybackMode::Normal);
                        }
                        ("down", "a") | ("down", "A")
                            if inner.render_state.host_screen == HostScreen::InGame =>
                        {
                            let mode = if inner.playback_mode == render_state::PlaybackMode::Auto {
                                render_state::PlaybackMode::Normal
                            } else {
                                render_state::PlaybackMode::Auto
                            };
                            inner.set_playback_mode(mode);
                        }
                        ("down", "Backspace")
                            if inner.render_state.host_screen == HostScreen::InGame =>
                        {
                            inner.restore_snapshot();
                        }
                        _ => {}
                    }
                }
            }
        });
    });

    // 根据初始化阶段和 host_screen 路由渲染
    let phase = init_phase.read().clone();
    match phase {
        InitPhase::Loading => {
            rsx! {
                div { class: "game-container",
                    div { class: "screen-loading", "Loading..." }
                }
            }
        }
        InitPhase::Error(msg) => {
            rsx! {
                div { class: "game-container",
                    div { class: "screen-error", "{msg}" }
                }
            }
        }
        InitPhase::Ready => {
            let screen = render_state.read().host_screen.clone();
            rsx! {
                div { class: "game-container",
                    match screen {
                        HostScreen::Title => rsx! { TitleScreen { render_state } },
                        HostScreen::InGame => rsx! {
                            VNScene { render_state }
                            SkipIndicator { render_state }
                        },
                        HostScreen::InGameMenu => rsx! {
                            VNScene { render_state }
                            InGameMenu { render_state }
                        },
                        HostScreen::Save | HostScreen::Load => rsx! {
                            SaveLoadScreen { render_state }
                        },
                        HostScreen::Settings => rsx! {
                            SettingsScreen { render_state }
                        },
                        HostScreen::History => rsx! {
                            HistoryScreen { render_state }
                        },
                    }
                    // 确认弹窗（z-index 最高，覆盖所有页面）
                    ConfirmDialog {}
                    // Toast 提示（右上角）
                    ToastLayer {}
                }
            }
        }
    }
}
