# host_app 摘要

## Purpose

`host_app` 实现 winit `ApplicationHandler` trait，是窗口生命周期和帧循环的入口。负责窗口创建、事件分发、egui UI 编排和每帧渲染驱动。

## PublicSurface

- 文件：`host/src/host_app.rs`
- 关键类型：`HostApp`
- `HostApp::new(config, font_data)` 构造应用实例
- `ApplicationHandler::resumed` 创建窗口与 GPU 后端，初始化 `AppState`
- `ApplicationHandler::window_event` 分发输入/调用 `app::update`/驱动 egui UI/提交渲染帧

## KeyFlow

1. `resumed`：创建 winit 窗口 -> 初始化 `WgpuBackend` -> 创建 `AppState` -> 设置 GPU 资源上下文
2. `window_event(RedrawRequested)`：首帧加载资源/脚本 -> 每帧 `update` -> 构建 sprite draw commands -> egui UI 渲染 -> 处理 `EguiAction` -> 提交帧

## Dependencies

- `host::app`（`AppState`、`update`、`build_game_draw_commands` 等）
- `host::backend::WgpuBackend`
- `egui_actions`、`egui_screens`

## Invariants

- `HostApp` 是 `main.rs` 与 `AppState` 之间的唯一桥梁
- `backend` 和 `app_state` 在 `resumed` 后才初始化（均为 `Option`）

## WhenToReadSource

- 需要修改窗口创建参数、事件处理流程或帧循环编排时

## LastVerified

2026-03-11

## Owner

Ring-rs 维护者
