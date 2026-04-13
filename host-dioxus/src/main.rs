#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

// ── 后端模块（Phase 1 迁移自 host-tauri，无 Tauri 依赖） ──
pub mod audio;
pub mod command_executor;
pub mod config;
pub mod error;
pub mod headless_cli;
pub mod init;
pub mod manifest;
pub mod render_state;
pub mod resources;
pub mod save_manager;
pub mod state;

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

use components::SkipIndicator;
use render_state::{HostScreen, RenderState};
use screens::{HistoryScreen, InGameMenu, SaveLoadScreen, SettingsScreen, TitleScreen};
use state::{AppState, AppStateInner};
use vn::VNScene;

// ---------------------------------------------------------------------------
// CSS — 全局样式（BEM 命名，内联注入）
// ---------------------------------------------------------------------------

const GLOBAL_CSS: &str = r#"
/* === Reset & Variables === */
* { margin: 0; padding: 0; box-sizing: border-box; }
:root {
    --vn-width: 1280px;
    --vn-height: 720px;
    --vn-bg-color: #000;
    --vn-text-color: #eee;
    --vn-font-body: "Noto Sans SC", "Microsoft YaHei", sans-serif;
    --vn-ease-scene: ease;
}
body {
    background: #000;
    color: var(--vn-text-color);
    font-family: var(--vn-font-body);
    overflow: hidden;
}

/* === Game Container === */
.game-container {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: var(--vn-bg-color);
}

/* === VN Scene === */
.vn-scene {
    position: absolute;
    inset: 0;
    overflow: hidden;
}

.vn-scene__layers {
    position: absolute;
    inset: 0;
}

.vn-scene__dim {
    position: absolute;
    inset: 0;
    background: #000;
    pointer-events: none;
}

/* === Background Layer === */
.vn-background {
    position: absolute;
    inset: 0;
}

.vn-background__img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.vn-background__img--old {
    z-index: 1;
}

.vn-background__img--current {
    z-index: 0;
}

/* === Dialogue Box (ADV) === */
.vn-dialogue {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(0, 0, 0, 0.78);
    padding: 16px 24px 20px;
    min-height: 140px;
    z-index: 50;
    cursor: pointer;
}

.vn-dialogue__name {
    font-size: 1.1em;
    font-weight: bold;
    color: #ffd700;
    margin-bottom: 6px;
}

.vn-dialogue__text {
    font-size: 1.05em;
    line-height: 1.7;
    color: var(--vn-text-color);
    min-height: 60px;
    white-space: pre-wrap;
}

.vn-dialogue__advance {
    display: inline-block;
    margin-left: 4px;
    animation: vn-blink 0.8s ease-in-out infinite;
    color: #aaa;
    font-size: 0.8em;
}

@keyframes vn-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.2; }
}

/* === NVL Panel === */
.vn-nvl {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
}

.vn-nvl__scroll {
    width: 80%;
    max-height: 85%;
    overflow-y: auto;
    padding: 32px;
}

.vn-nvl__entry {
    margin-bottom: 16px;
    line-height: 1.7;
    font-size: 1.05em;
}

.vn-nvl__speaker {
    font-weight: bold;
    color: #ffd700;
    margin-right: 8px;
}

.vn-nvl__text {
    color: var(--vn-text-color);
    white-space: pre-wrap;
}

/* === Character Layer === */
.vn-characters {
    position: absolute;
    inset: 0;
    z-index: 10;
    pointer-events: none;
}

.vn-characters__sprite {
    position: absolute;
    max-height: 100%;
    pointer-events: none;
    user-select: none;
}

/* === Choice Panel === */
.vn-choices {
    position: absolute;
    inset: 0;
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.3);
}

.vn-choices__panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 320px;
    max-width: 60%;
}

.vn-choices__btn {
    padding: 14px 24px;
    background: rgba(20, 20, 50, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: var(--vn-text-color);
    font-size: 1.05em;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
    text-align: left;
}

.vn-choices__btn:hover {
    background: rgba(40, 40, 80, 0.95);
    border-color: rgba(255, 215, 0, 0.6);
}

/* === Transition Overlay (Fade/FadeWhite) === */
.vn-transition-overlay {
    position: absolute;
    inset: 0;
    z-index: 30;
    pointer-events: none;
}

/* === Rule Transition Canvas (WebGL) === */
.vn-rule-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: 30;
    pointer-events: none;
}

/* === Chapter Mark === */
.vn-chapter-mark {
    position: absolute;
    top: 10%;
    left: 0;
    right: 0;
    text-align: center;
    z-index: 55;
    color: #fff;
    text-shadow: 0 2px 8px rgba(0,0,0,0.7);
    pointer-events: none;
    font-weight: bold;
}

/* === Title Card === */
.vn-title-card {
    position: absolute;
    inset: 0;
    z-index: 70;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.vn-title-card__text {
    color: #fff;
    font-size: 2em;
    font-weight: 300;
    letter-spacing: 0.15em;
    text-align: center;
    max-width: 70%;
}

/* === Video Overlay === */
.vn-video-overlay {
    position: absolute;
    inset: 0;
    z-index: 80;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
}

.vn-video-overlay__video {
    max-width: 100%;
    max-height: 100%;
}

/* === Quick Menu === */
.vn-quick-menu {
    position: absolute;
    bottom: 0;
    right: 0;
    z-index: 45;
    display: flex;
    gap: 2px;
    padding: 4px;
}

.vn-quick-menu__btn {
    padding: 4px 10px;
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.75em;
    border-radius: 2px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
}

.vn-quick-menu__btn:hover {
    background: rgba(40, 40, 80, 0.8);
    color: #fff;
}

.vn-quick-menu__btn--active {
    background: rgba(80, 60, 20, 0.8);
    color: #ffd700;
    border-color: rgba(255, 215, 0, 0.4);
}

/* === Skip Mode: instantly resolve all CSS transitions === */
.skip-mode *, .skip-mode *::before, .skip-mode *::after {
    transition-duration: 0s !important;
    animation-duration: 0s !important;
}

/* === In-Game Menu === */
.screen-ingame-menu {
    position: absolute;
    inset: 0;
    z-index: 90;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(4px);
}

.screen-ingame-menu__panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 240px;
}

.screen-ingame-menu__btn {
    padding: 14px 32px;
    background: rgba(20, 20, 50, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: var(--vn-text-color);
    font-size: 1.05em;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
    text-align: center;
}

.screen-ingame-menu__btn:hover {
    background: rgba(40, 40, 80, 0.95);
}

/* === Title Screen === */
.screen-title {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #1a1a2e;
}

.screen-title__heading {
    font-size: 2.5em;
    margin-bottom: 40px;
    color: #eee;
}

.screen-title__btn {
    padding: 12px 40px;
    margin: 8px;
    cursor: pointer;
    border: 1px solid #555;
    background: #2a2a4e;
    color: #eee;
    border-radius: 4px;
    font-size: 1.1em;
    min-width: 200px;
    transition: background 0.2s;
}

.screen-title__btn:hover {
    background: #3a3a6e;
}

/* === Save/Load Screen === */
.screen-save-load {
    position: absolute;
    inset: 0;
    background: #1a1a2e;
    display: flex;
    flex-direction: column;
    padding: 24px;
}

.screen-save-load__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.screen-save-load__header h2 { font-size: 1.5em; }

.screen-save-load__back-btn,
.screen-settings__back-btn,
.screen-history__back-btn {
    padding: 8px 20px;
    background: #2a2a4e;
    border: 1px solid #555;
    color: #eee;
    border-radius: 4px;
    cursor: pointer;
}

.screen-save-load__grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    flex: 1;
}

.screen-save-load__slot {
    background: rgba(255,255,255,0.05);
    border: 1px solid #333;
    border-radius: 6px;
    padding: 8px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 120px;
    transition: border-color 0.2s;
}

.screen-save-load__slot:hover { border-color: #888; }
.screen-save-load__slot--filled { border-color: #556; }

.screen-save-load__thumb {
    width: 100%;
    max-height: 80px;
    object-fit: cover;
    border-radius: 4px;
    margin-bottom: 6px;
}

.screen-save-load__slot-label {
    font-size: 0.85em;
    color: #aaa;
}

.screen-save-load__pagination {
    display: flex;
    gap: 6px;
    justify-content: center;
    margin-top: 12px;
}

.screen-save-load__page-btn {
    padding: 6px 12px;
    background: #2a2a4e;
    border: 1px solid #444;
    color: #aaa;
    border-radius: 4px;
    cursor: pointer;
}

.screen-save-load__page-btn--active {
    background: #3a3a6e;
    color: #fff;
    border-color: #777;
}

/* === Settings Screen === */
.screen-settings {
    position: absolute;
    inset: 0;
    background: #1a1a2e;
    display: flex;
    flex-direction: column;
    padding: 24px;
}

.screen-settings__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
}

.screen-settings__header h2 { font-size: 1.5em; }

.screen-settings__body {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 500px;
}

.screen-settings__row {
    display: flex;
    align-items: center;
    gap: 12px;
}

.screen-settings__row label {
    min-width: 140px;
    color: #ccc;
}

.screen-settings__row input[type="range"] {
    flex: 1;
    accent-color: #ffd700;
}

.screen-settings__row span {
    min-width: 50px;
    text-align: right;
    color: #aaa;
    font-size: 0.9em;
}

/* === History Screen === */
.screen-history {
    position: absolute;
    inset: 0;
    background: #1a1a2e;
    display: flex;
    flex-direction: column;
    padding: 24px;
}

.screen-history__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.screen-history__header h2 { font-size: 1.5em; }

.screen-history__scroll {
    flex: 1;
    overflow-y: auto;
    padding-right: 8px;
}

.screen-history__entry {
    padding: 8px 0;
    border-bottom: 1px solid rgba(255,255,255,0.08);
    line-height: 1.6;
}

.screen-history__speaker {
    font-weight: bold;
    color: #ffd700;
    margin-right: 8px;
}

.screen-history__text {
    color: #ddd;
}

.screen-history__empty {
    color: #666;
    text-align: center;
    margin-top: 40px;
}

/* === Skip/Auto Indicator === */
.skip-indicator {
    position: fixed;
    top: 12px;
    left: 12px;
    z-index: 100;
    padding: 4px 12px;
    border-radius: 3px;
    font-size: 0.8em;
    font-weight: bold;
    letter-spacing: 0.1em;
    pointer-events: none;
}

.skip-indicator--skip {
    background: rgba(200, 50, 50, 0.8);
    color: #fff;
}

.skip-indicator--auto {
    background: rgba(50, 120, 200, 0.8);
    color: #fff;
}

/* === Loading / Error === */
.screen-loading {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
    color: #888;
    font-size: 1.2em;
}

.screen-error {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1a0000;
    color: #f44;
    font-size: 1.1em;
    padding: 40px;
    text-align: center;
}
"#;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    tracing_subscriber::fmt::init();

    let css_head = format!("<style>{GLOBAL_CSS}</style>");

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Ring Engine")
                        .with_inner_size(LogicalSize::new(1280, 720)),
                )
                .with_custom_head(css_head)
                .with_custom_protocol("ring-asset", ring_asset_handler),
        )
        .launch(App);
}

// ---------------------------------------------------------------------------
// ring-asset custom protocol handler
// ---------------------------------------------------------------------------

fn ring_asset_handler(
    _id: dioxus::desktop::wry::WebViewId,
    request: http::Request<Vec<u8>>,
) -> http::Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();
    let raw_path = request.uri().path();
    let path_clean = percent_decode(raw_path.trim_start_matches('/'));
    let assets_root = find_assets_root();
    let full_path = assets_root.join(&path_clean);

    tracing::debug!(uri = %uri, resolved = %full_path.display(), "ring-asset request");

    let mime = guess_mime(&path_clean);

    match std::fs::read(&full_path) {
        Ok(bytes) => http::Response::builder()
            .status(200)
            .header("Content-Type", mime)
            .header("Access-Control-Allow-Origin", "*")
            .body(Cow::from(bytes))
            .unwrap(),
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
// App 初始化状态
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum InitPhase {
    Loading,
    Ready,
    Error(String),
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

fn App() -> Element {
    // 全局 AppState：Arc<Mutex<AppStateInner>>
    let app_state = use_context_provider(|| AppState {
        inner: Arc::new(Mutex::new(AppStateInner::new())),
    });

    // 初始化阶段
    let mut init_phase = use_signal(|| InitPhase::Loading);

    // RenderState signal：tick loop 每帧更新
    let mut render_state = use_signal(RenderState::new);

    // 初始化后端子系统（仅首次 mount）
    let app_state_init = app_state.clone();
    use_hook(move || {
        spawn(async move {
            let result = {
                let mut inner = app_state_init.inner.lock().unwrap();
                init::initialize_inner(&mut inner)
            };
            match result {
                Ok(()) => {
                    {
                        let mut inner = app_state_init.inner.lock().unwrap();
                        inner.frontend_connected(Some("dioxus-desktop".to_string()));
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
                        ("down", " ") | ("down", "Enter") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                inner.process_click();
                            }
                        }
                        ("down", "Control") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                inner.set_playback_mode(
                                    render_state::PlaybackMode::Skip,
                                );
                            }
                        }
                        ("up", "Control") => {
                            if inner.playback_mode
                                == render_state::PlaybackMode::Skip
                            {
                                inner.set_playback_mode(
                                    render_state::PlaybackMode::Normal,
                                );
                            }
                        }
                        ("down", "a") | ("down", "A") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                let mode = if inner.playback_mode
                                    == render_state::PlaybackMode::Auto
                                {
                                    render_state::PlaybackMode::Normal
                                } else {
                                    render_state::PlaybackMode::Auto
                                };
                                inner.set_playback_mode(mode);
                            }
                        }
                        ("down", "Backspace") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                inner.restore_snapshot();
                            }
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
                }
            }
        }
    }
}
