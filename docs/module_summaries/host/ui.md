# host/ui 摘要

## Purpose

`ui` 提供主题系统、Toast 通知和 UI 上下文等基础设施。界面渲染已迁移到 egui（在 `main.rs` 中构建）。
旧的 macroquad UI 组件（button/list/modal/panel/slider/toggle/tab/scroll）已在 RFC-007 Phase 5 中移除。

## PublicSurface

- 模块入口：`host/src/ui/mod.rs`
- 核心类型：`UiContext`、`Theme`（token 分层 + 自定义 `Color` 类型）、`ToastManager`
- 子模块：`theme`、`theme_loader`、`toast`、`skin`

## KeyFlow

1. 启动时通过 `theme_loader` 加载默认主题 + 覆盖文件（缺失时回退默认主题并诊断）。
2. `UiContext` 存储主题和屏幕尺寸，由 winit resize 事件更新。
3. `ToastManager` 管理通知队列，每帧 `update(dt)` 推进计时和淡出。
4. egui 界面代码在 `main.rs` 中通过 `build_toast_overlay` 渲染 Toast。

## Dependencies

- 不依赖外部渲染库（自定义 `Color` 类型替代 macroquad::Color）
- 被 `app` 消费（UiContext / ToastManager 存储在 UiSystems 中）

## Invariants

- `UiContext` 屏幕尺寸由外部（winit）驱动，非自轮询。
- Theme 的 `Color` 类型为 `ui::theme::Color`（RGBA f32），与 egui `Color32` 独立。

## WhenToReadSource

- 需要扩展主题 token 或新增 Toast 类型时。
- 需要理解 egui 与 Theme 的映射关系时。

## RelatedDocs

- [host 总览](../host.md)
- [backend 摘要](backend.md)

## LastVerified

2026-03-11

## Owner

Ring-rs 维护者
