# host_app 摘要

## Purpose

`host_app` 实现 winit `ApplicationHandler` trait，是窗口模式下的生命周期和帧循环入口。负责窗口创建、事件分发、egui UI 编排和每帧渲染驱动；无窗口回放路径则由 `host/src/headless.rs` 负责镜像执行同一套 `app::update` 与 UI 编排。

## PublicSurface

- 文件：`host/src/host_app.rs`
- 相关入口：`host/src/headless.rs`
- 关键类型：`HostApp`
- `HostApp::new(config, event_stream_path)` 构造应用实例（字体在 `resumed` 中按 `config.default_font` 加载）
- `ApplicationHandler::resumed` 创建窗口，初始化 `AppState`，再创建 GPU 后端并回填 `TextureContext`
- `ApplicationHandler::window_event` 分发输入/调用 `app::update`/驱动 egui UI/提交渲染帧

## KeyFlow

1. 窗口模式：`resumed` 创建 winit 窗口 -> 创建 `AppState` -> 读取默认字体 -> 初始化 `WgpuBackend` -> 设置 GPU 资源上下文
2. 窗口模式：`window_event(RedrawRequested)` 首帧加载资源/脚本 -> 每帧 `update` -> 构建 sprite draw commands -> egui UI 渲染（若 UI 模式活跃：`ui_mode_registry.take_active()` 取出 handler，`UiModeHandler::render()`；若返回 `Completed`/`Cancelled` 则 `complete_active()` 并向 runtime 注入 `RuntimeInput::UIResult`，否则 `restore_active()` 归还以便下一帧继续）-> 处理 `EguiAction` -> 提交帧
3. Headless 模式：`headless::run` 加载 replay -> 以固定步长执行 `app::update` -> 运行 CPU-only egui -> 处理 `EguiAction`
4. 小游戏启动时创建 `BridgeServer`（HTTP 服务器），通过 HTTP URL 加载 WebView；`about_to_wait` 每帧轮询 `bridge.poll()` 处理 HTTP 请求和游戏完成检测

## Dependencies

- `host::app`（`AppState`、`update`、`build_game_draw_commands` 等）
- `host::game_mode::BridgeServer`
- `host::backend::WgpuBackend`
- `egui_actions`、`egui_screens`

## Invariants

- `HostApp` 只负责窗口模式；headless 路径不经过该类型
- `backend` 和 `app_state` 在 `resumed` 后才初始化（均为 `Option`）
- 小游戏活跃期间 `about_to_wait` 通过 `request_redraw()` 保持事件循环轮转，确保 HTTP 请求及时响应

## WhenToReadSource

- 需要修改窗口创建参数、事件处理流程或帧循环编排时

## LastVerified

2026-03-24

## Owner

claude-4.6-opus