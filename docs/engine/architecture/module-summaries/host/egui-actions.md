# egui_actions 摘要

## Purpose

`egui_actions` 定义 egui 层对应用层的动作协议，把页面构建与实际副作用处理解耦。

## PublicSurface

- 文件：`host/src/egui_actions.rs`
- 核心类型：`EguiAction`
- 关键接口：`handle_egui_action`、`action_def_to_egui`、`button_def_to_egui`

## KeyFlow

1. `egui_screens` 与 `build_ui` 只返回 `EguiAction`，不直接改 `AppState`。
2. `action_def_to_egui` / `button_def_to_egui` 把 `ScreenDefinitions` 中的声明式动作转换为运行时动作，并在按钮层包上 `ShowConfirm`。
3. `handle_egui_action` 统一处理导航、开局/继续、存读档、设置保存、播放模式切换与退出。

## Invariants

- UI 副作用集中在 `handle_egui_action`，页面构建函数保持纯 UI。
- `ShowConfirm` 必须由调用方先拦截；直接传入 `handle_egui_action` 属于错误用法。
- `StartAtLabel(String)` 是当前通用的“从指定标签开始”入口。

## WhenToReadSource

- 需要新增 UI 动作或修改某个动作的副作用时。
- 需要排查按钮配置到 `AppState` 状态变更的映射时。
- 需要确认哪些动作必须经过确认弹窗包装时。

## RelatedDocs

- [host_app 摘要](host-app.md)
- [egui_screens 摘要](egui-screens.md)
- [ui 摘要](ui.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
