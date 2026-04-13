#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

// ── 后端模块（Phase 1 迁移自 host-tauri，无 Tauri 依赖） ──
pub mod audio;
pub mod command_executor;
pub mod config;
pub mod error;
pub mod headless_cli;
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
// CSS — 全局样式（BEM 命名，内联注入）
// ---------------------------------------------------------------------------

const GLOBAL_CSS: &str = r#"
/* === Reset & Variables === */
* { margin: 0; padding: 0; box-sizing: border-box; }
:root {
    /* ── 基准分辨率（所有像素值基于此坐标系） ── */
    --vn-base-w: 1920;
    --vn-base-h: 1080;
    --vn-bg-color: #000;
    --vn-font-body: "Noto Sans SC", "Microsoft YaHei", sans-serif;
    --vn-ease-scene: ease;
    --scale-factor: 1;

    /* ── 颜色 token（来自 layout.json colors） ── */
    --ui-accent: #ffffff;
    --ui-idle: #888888;
    --ui-hover: #ff9900;
    --ui-selected: #ffffff;
    --ui-insensitive: #7878787f;
    --ui-text: #000000;
    --ui-interface-text: #ffffff;

    /* ── 字号 token（来自 layout.json fonts，基准 1920×1080） ── */
    --font-text: 33px;
    --font-name: 45px;
    --font-interface: 33px;
    --font-label: 36px;
    --font-notify: 24px;
    --font-title: 75px;
    --font-quick: 21px;
}
body {
    background: #000;
    color: var(--ui-interface-text);
    font-family: var(--vn-font-body);
    overflow: hidden;
    width: 100vw;
    height: 100vh;
}

/* === Game Container (1920×1080 基准 + transform 缩放，居中对齐) === */
.game-container {
    position: fixed;
    left: 50%;
    top: 50%;
    width: 1920px;
    height: 1080px;
    overflow: hidden;
    background: var(--vn-bg-color);
    transform: translate(-50%, -50%) scale(var(--scale-factor));
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
    height: 278px;
    z-index: 50;
    cursor: pointer;
    user-select: none;
    /* NinePatch textbox 背景 */
    border-image-source: url("http://ring-asset.localhost/gui/textbox.png");
    border-image-slice: 30 30 30 30 fill;
    border-image-width: 30px;
    border-style: solid;
    background: transparent;
}

.vn-dialogue__name {
    position: absolute;
    left: 360px;
    top: 0px;
    font-size: var(--font-name);
    font-weight: bold;
    color: var(--ui-accent);
    white-space: nowrap;
    text-shadow: 1px 1px 2px rgba(0,0,0,0.9), 0 0 8px rgba(0,0,0,0.5);
    /* NinePatch namebox 背景 */
    border-image-source: url("http://ring-asset.localhost/gui/namebox.png");
    border-image-slice: 5 5 5 5 fill;
    border-image-width: 5px;
    border-style: solid;
    background: transparent;
    padding: 2px 10px;
}

.vn-dialogue__text {
    position: absolute;
    left: 402px;
    top: 75px;
    max-width: 1116px;
    font-size: var(--font-text);
    line-height: 1.7;
    color: var(--ui-text);
    white-space: pre-wrap;
}

.vn-dialogue__advance {
    display: inline-block;
    margin-left: 4px;
    animation: vn-blink 0.8s ease-in-out infinite;
    color: var(--ui-idle);
    font-size: 0.8em;
}

@keyframes vn-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.2; }
}

/* 背景 dissolve 淡出动画（新创建元素无法用 CSS transition，需 @keyframes） */
@keyframes vn-dissolve-out {
    from { opacity: 1; }
    to { opacity: 0; }
}

/* 遮罩淡入动画（Fade/FadeWhite FadeIn 阶段） */
@keyframes vn-overlay-fadein {
    from { opacity: 0; }
    to { opacity: 1; }
}

/* === NVL Panel === */
.vn-nvl {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.706);
    z-index: 50;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    cursor: pointer;
    user-select: none;
}

.vn-nvl__scroll {
    width: calc(100% - 240px);
    max-height: 100%;
    overflow-y: auto;
    padding: 80px 0 40px 0;
    margin: 0 120px;
}

.vn-nvl__entry {
    margin-bottom: 16px;
    line-height: 1.5;
}

.vn-nvl__speaker {
    font-size: var(--font-name);
    font-weight: bold;
    color: var(--ui-accent);
    margin-right: 4px;
}

.vn-nvl__speaker::after {
    content: "：";
}

.vn-nvl__text {
    font-size: var(--font-text);
    color: var(--ui-text);
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
}

.vn-choices__panel {
    display: flex;
    flex-direction: column;
    gap: 33px;
    width: 1185px;
}

.vn-choices__btn {
    width: 100%;
    padding: 0;
    cursor: pointer;
    font-size: var(--font-text);
    color: #cccccc;
    text-align: center;
    transition: color 0.15s;
    /* NinePatch choice_idle 背景 */
    border-image-source: url("http://ring-asset.localhost/gui/button/choice_idle_background.png");
    border-image-slice: 8 150 8 150 fill;
    border-image-width: 8px 150px 8px 150px;
    border-style: solid;
    background: transparent;
}

.vn-choices__btn:hover {
    color: #ffffff;
    border-image-source: url("http://ring-asset.localhost/gui/button/choice_hover_background.png");
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
    top: 30px;
    left: 30px;
    z-index: 55;
    color: #fff;
    font-size: var(--font-label);
    font-weight: bold;
    padding: 10px 20px;
    border-radius: 4px;
    pointer-events: none;
    text-shadow: 0 2px 8px rgba(0,0,0,0.7);
}

/* === Title Card === */
.vn-title-card {
    position: absolute;
    inset: 0;
    z-index: 70;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.vn-title-card__text {
    color: #fff;
    font-size: var(--font-title);
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

/* === Quick Menu（对话框内部底边居中） === */
.vn-quick-menu {
    position: absolute;
    bottom: 4px;  /* 对话框内部底边，留 4px 边距 */
    left: 50%;
    transform: translateX(-50%);
    z-index: 55;  /* 高于对话框 z-index:50 */
    display: flex;
    gap: 0;
}

.vn-quick-menu__btn {
    padding: 4px 0;
    width: 90px;
    background: transparent;
    border: none;
    color: var(--ui-idle);
    font-size: var(--font-quick);
    cursor: pointer;
    transition: color 0.15s;
    text-align: center;
}

.vn-quick-menu__btn:hover {
    color: var(--ui-hover);
}

.vn-quick-menu__btn--active {
    color: var(--ui-hover);
}

/* === Map Overlay (showMap) === */
.vn-map-overlay {
    position: absolute;
    inset: 0;
    z-index: 75;
    background: rgba(0, 0, 0, 0.85);
    user-select: none;
}

.vn-map-overlay__bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.vn-map-overlay__title {
    position: absolute;
    top: 40px;
    left: 50%;
    transform: translateX(-50%);
    font-size: var(--font-label);
    font-weight: bold;
    color: #fff;
    text-shadow: 0 2px 8px rgba(0,0,0,0.7);
    z-index: 1;
}

.vn-map-overlay__btn {
    position: absolute;
    transform: translate(-50%, -50%);
    width: 200px;
    height: 50px;
    border: 1.5px solid rgb(100, 149, 237);
    border-radius: 8px;
    background: rgba(40, 40, 60, 0.8);
    color: #ccc;
    font-size: 20px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    z-index: 1;
}

.vn-map-overlay__btn:hover {
    background: rgba(80, 80, 120, 0.9);
    color: #fff;
}

.vn-map-overlay__btn--disabled {
    border-color: rgb(80, 80, 80);
    background: rgba(60, 60, 60, 0.7);
    color: rgb(100, 100, 100);
    cursor: not-allowed;
}

/* === Minigame Overlay (callGame iframe) === */
.vn-minigame-overlay {
    position: absolute;
    inset: 0;
    z-index: 75;
    background: #000;
}

.vn-minigame-overlay__frame {
    width: 100%;
    height: 100%;
    border: none;
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
    background: rgba(0, 0, 0, 0.706);
    display: flex;
    align-items: center;
    justify-content: center;
}

.screen-ingame-menu__panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 260px;
}

.screen-ingame-menu__btn {
    width: 260px;
    height: 49px;
    padding: 0;
    background: rgba(30, 30, 60, 0.39);
    border: none;
    border-radius: 4px;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    text-align: center;
}

.screen-ingame-menu__btn:hover {
    background: rgba(60, 60, 100, 0.59);
    color: var(--ui-hover);
}

/* === Title Screen === */
.screen-title {
    position: absolute;
    inset: 0;
    overflow: hidden;
}

.screen-title__bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 0;
}

.screen-title__overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 1;
    pointer-events: none;
}

.screen-title__nav {
    position: absolute;
    left: 60px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.screen-title__btn {
    width: 240px;
    height: 49px;
    padding: 0;
    cursor: pointer;
    border: none;
    background: transparent;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    text-align: left;
    transition: color 0.15s;
}

.screen-title__btn:hover {
    color: var(--ui-hover);
}

/* === Save/Load Screen（嵌入 GameMenuFrame） === */
.save-load__tabs {
    display: flex;
    gap: 20px;
    margin-bottom: 16px;
}

.save-load__tab {
    background: transparent;
    border: none;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    cursor: pointer;
    transition: color 0.15s;
}

.save-load__tab:hover { color: var(--ui-hover); }
.save-load__tab--active { color: var(--ui-accent); }

.save-load__grid {
    display: grid;
    grid-template-columns: repeat(3, 414px);
    grid-template-rows: repeat(2, 309px);
    gap: 15px;
}

.save-load__slot {
    position: relative;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    /* NinePatch slot_idle */
    border-image-source: url("http://ring-asset.localhost/gui/button/slot_idle_background.png");
    border-image-slice: 15 15 15 15 fill;
    border-image-width: 15px;
    border-style: solid;
    background: transparent;
}

.save-load__slot:hover {
    border-image-source: url("http://ring-asset.localhost/gui/button/slot_hover_background.png");
}

.save-load__thumb {
    width: 384px;
    height: 216px;
    object-fit: cover;
    margin: 0 auto;
}

.save-load__slot-info {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.save-load__slot-chapter {
    font-size: 24px;
    color: var(--ui-interface-text);
}

.save-load__slot-time {
    font-size: 20px;
    color: var(--ui-idle);
}

.save-load__slot-empty {
    font-size: 24px;
    color: var(--ui-idle);
    text-align: center;
    padding: 40px 0;
}

.save-load__delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    color: var(--ui-idle);
    font-size: 24px;
    cursor: pointer;
    transition: color 0.15s;
    z-index: 5;
}

.save-load__delete-btn:hover {
    color: #ff5050;
}

.save-load__pagination {
    display: flex;
    gap: 4px;
    justify-content: center;
    margin-top: 16px;
}

.save-load__page-btn {
    padding: 4px 12px;
    background: transparent;
    border: none;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    cursor: pointer;
    transition: color 0.15s;
}

.save-load__page-btn:hover { color: var(--ui-hover); }
.save-load__page-btn--active { color: var(--ui-accent); }

/* === Settings Screen（嵌入 GameMenuFrame） === */
.settings__body {
    display: flex;
    flex-direction: column;
    gap: 15px;
}

.settings__row {
    display: flex;
    align-items: center;
    gap: 12px;
}

.settings__label {
    width: 200px;
    font-size: var(--font-interface);
    color: var(--ui-interface-text);
    text-align: right;
}

.settings__slider {
    width: 400px;
    height: 16px;
    accent-color: var(--ui-hover);
}

.settings__value {
    min-width: 100px;
    font-size: 28px;
    color: var(--ui-interface-text);
    text-align: left;
}

.settings__checkbox-label {
    font-size: var(--font-interface);
    color: var(--ui-interface-text);
    cursor: pointer;
}

.settings__apply-row {
    margin-top: 20px;
    display: flex;
    justify-content: center;
}

.settings__apply-btn {
    width: 160px;
    height: 49px;
    border: none;
    border-radius: 4px;
    background: rgba(40, 40, 70, 1);
    color: var(--ui-accent);
    font-size: var(--font-interface);
    cursor: pointer;
    transition: background 0.15s;
}

.settings__apply-btn:hover {
    background: rgba(60, 60, 100, 1);
}

/* === History Screen（嵌入 GameMenuFrame） === */
.history__scroll {
    height: 100%;
    overflow-y: auto;
}

.history__entry {
    display: flex;
    gap: 16px;
    line-height: 1.7;
    padding: 4px 0;
}

.history__name {
    width: 233px;
    min-width: 233px;
    font-size: var(--font-name);
    font-weight: bold;
    color: var(--ui-accent);
    text-align: right;
}

.history__text {
    flex: 1;
    max-width: 1110px;
    font-size: var(--font-interface);
    color: var(--ui-interface-text);
}

.history__empty {
    font-size: var(--font-interface);
    color: var(--ui-idle);
    text-align: center;
    margin-top: 60px;
}

/* === Skip/Auto Indicator === */
.skip-indicator {
    position: absolute;
    top: 15px;
    left: 10px;
    z-index: 100;
    padding: 4px 16px;
    font-size: var(--font-notify);
    font-weight: bold;
    color: #fff;
    pointer-events: none;
}

.skip-indicator--skip {
    background: rgba(20, 60, 40, 0.78);
    border-radius: 3px;
}

.skip-indicator--auto {
    background: rgba(20, 40, 80, 0.78);
    border-radius: 3px;
}

.skip-indicator__arrows {
    display: inline-block;
    animation: skip-arrow-cycle 1s steps(3, end) infinite;
}

@keyframes skip-arrow-cycle {
    0%   { content: "›"; }
    33%  { content: "››"; }
    66%  { content: "›››"; }
    100% { content: "›"; }
}

/* === GameMenuFrame（左导航 + 右内容通用框架） === */
.game-menu {
    position: absolute;
    inset: 0;
    overflow: hidden;
}

.game-menu__bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 0;
}

.game-menu__overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 1;
    pointer-events: none;
}

.game-menu__nav {
    position: absolute;
    left: 0;
    top: 0;
    width: 420px;
    height: 100%;
    z-index: 2;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
    padding-left: 60px;
}

.game-menu__nav-buttons {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.game-menu__nav-btn {
    width: 340px;
    height: 49px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    text-align: left;
    cursor: pointer;
    transition: color 0.15s;
}

.game-menu__nav-btn:hover {
    color: var(--ui-hover);
}

.game-menu__nav-btn--active {
    color: var(--ui-accent);
}

.game-menu__return-btn {
    position: absolute;
    bottom: 40px;
    left: 60px;
    width: 340px;
    height: 49px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--ui-idle);
    font-size: var(--font-interface);
    text-align: left;
    cursor: pointer;
    transition: color 0.15s;
}

.game-menu__return-btn:hover {
    color: var(--ui-hover);
}

.game-menu__content {
    position: absolute;
    left: 440px;
    top: 40px;
    right: 40px;
    bottom: 40px;
    z-index: 2;
}

.game-menu__title {
    font-size: 45px;
    color: var(--ui-interface-text);
    margin-bottom: 16px;
    font-weight: normal;
}

.game-menu__body {
    height: calc(100% - 61px);
    overflow: hidden;
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

/* === Confirm Dialog === */
.confirm-overlay {
    position: absolute;
    inset: 0;
    z-index: 200;
    background: rgba(0, 0, 0, 0.706);
    display: flex;
    align-items: center;
    justify-content: center;
}

.confirm-panel {
    width: 600px;
    height: 300px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    /* NinePatch frame 背景 */
    border-image-source: url("http://ring-asset.localhost/gui/frame.png");
    border-image-slice: 60 60 60 60 fill;
    border-image-width: 60px;
    border-style: solid;
    background: transparent;
}

.confirm-panel__message {
    font-size: var(--font-interface);
    color: var(--ui-accent);
    text-align: center;
    margin-bottom: 30px;
}

.confirm-panel__buttons {
    display: flex;
    gap: 80px;
}

.confirm-panel__btn {
    width: 140px;
    height: 49px;
    border: none;
    border-radius: 4px;
    background: rgb(40, 40, 70);
    color: var(--ui-accent);
    font-size: var(--font-interface);
    cursor: pointer;
    transition: background 0.15s;
}

.confirm-panel__btn:hover {
    background: rgb(60, 60, 100);
}

/* === Toast === */
.toast-layer {
    position: absolute;
    top: 68px;
    right: 20px;
    z-index: 210;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
}

.toast {
    padding: 10px 16px;
    border-radius: 6px;
    font-size: var(--font-notify);
    color: #fff;
    animation: toast-show 2.8s ease-out forwards;
}

.toast--info    { background: rgba(40, 40, 80, 0.9); }
.toast--success { background: rgba(30, 80, 40, 0.9); }
.toast--warning { background: rgba(100, 80, 20, 0.9); }
.toast--error   { background: rgba(100, 30, 30, 0.9); }

@keyframes toast-show {
    0%   { opacity: 1; }
    82%  { opacity: 1; }  /* 2.5s / 2.8s ≈ 89%, 留 0.3s 淡出 */
    100% { opacity: 0; }
}
"#;

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
                        let _ = req.reply.send(Ok(debug_server::ScreenshotData {
                            format: "png".to_string(),
                            width,
                            height,
                            data_base64: data,
                        }));
                    }
                }
                Err(e) => {
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
                let mut inner = app_state_init.inner.lock().unwrap();
                init::initialize_inner(&mut inner)
            };
            match result {
                Ok(()) => {
                    let debug_port = {
                        let mut inner = app_state_init.inner.lock().unwrap();
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
                        ("down", " ") | ("down", "Enter") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                inner.process_click();
                            }
                        }
                        ("down", "Control") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                inner.set_playback_mode(render_state::PlaybackMode::Skip);
                            }
                        }
                        ("up", "Control") => {
                            if inner.playback_mode == render_state::PlaybackMode::Skip {
                                inner.set_playback_mode(render_state::PlaybackMode::Normal);
                            }
                        }
                        ("down", "a") | ("down", "A") => {
                            if inner.render_state.host_screen == HostScreen::InGame {
                                let mode =
                                    if inner.playback_mode == render_state::PlaybackMode::Auto {
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
                    // 确认弹窗（z-index 最高，覆盖所有页面）
                    ConfirmDialog {}
                    // Toast 提示（右上角）
                    ToastLayer {}
                }
            }
        }
    }
}
