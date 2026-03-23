# host_app 摘要

## Purpose

`host_app` 是窗口模式入口，负责 winit 生命周期、每帧驱动、GPU 后端接线，以及小游戏 WebView 的窗口内集成。

## PublicSurface

- 文件：`host/src/host_app.rs`
- 核心类型：`HostApp`
- 关键接口：`HostApp::new`、`ApplicationHandler::{resumed, window_event, about_to_wait}`
- 配套入口：`host/src/headless.rs`

## KeyFlow

1. `resumed()` 创建窗口，构造 `AppState`，读取默认字体，初始化 `WgpuBackend`，并把 `TextureContext` 回填给 `ResourceManager`。
2. `window_event(RedrawRequested)` 首帧延迟加载资源、脚本与 UI 素材；其后每帧执行 `app::update()`、补齐渲染资源、生成 draw commands。
3. UI 帧构建已抽到 `build_ui::build_frame_ui()`，窗口模式与 `headless` 共用；`host_app` 只负责调度、消费 `EguiAction`、确认弹窗与缩略图截图。
4. 若有活跃 `UiModeHandler`，本帧先 `take_active()` 渲染，再按结果 `restore_active()` 或 `complete_active()`，并在完成时注入 `RuntimeInput::UIResult`。
5. `about_to_wait()` 轮询小游戏 `BridgeServer`，在 WebView 完成后回传结果并请求重绘。

## Invariants

- `backend` 与 `app_state` 只在 `resumed()` 之后有效。
- 窗口模式的 UI 构建应复用 `build_ui`，避免与 `headless` 分叉。
- 小游戏轮询放在 `about_to_wait()`，避免依赖 `RedrawRequested` 才能处理 HTTP 请求。

## WhenToReadSource

- 需要修改窗口创建参数、帧循环顺序或截图保存时。
- 需要排查小游戏 WebView 生命周期、UI mode 渲染或确认弹窗处理时。
- 需要比较 windowed 与 `headless` 的 UI 调度差异时。

## RelatedDocs

- [host 总览](../host.md)
- [app 摘要](app.md)
- [egui_actions 摘要](egui-actions.md)
- [egui_screens 摘要](egui-screens.md)
- [game_mode 摘要](game-mode.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
