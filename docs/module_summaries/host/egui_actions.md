# egui_actions 摘要

## Purpose

定义 `EguiAction` 枚举及其处理函数，将 egui UI 层的交互意图与 `AppState` 的状态变更解耦。

## PublicSurface

- 文件：`host/src/egui_actions.rs`
- `EguiAction` 枚举：`None`、`StartGame`、`StartAtLabel(String)`、`ContinueGame`、`NavigateTo`、`ReplaceTo`、`GoBack`、`ReturnToGame`、`ReturnToTitle`、`Exit`、`ApplySettings`、`OpenSave`、`OpenLoad`、`SaveToSlot`、`LoadFromSlot`、`DeleteSlot`、`QuickSave`、`QuickLoad`、`ToggleSkip`、`ToggleAuto`、`ShowConfirm { message, on_confirm }`
- `handle_egui_action(app_state, action, save_load_tab, event_loop)` 消费 `EguiAction` 并执行对应状态变更
- `action_def_to_egui(action: &ActionDef) -> EguiAction` 将声明式 `ActionDef` 转换为 `EguiAction`
- `button_def_to_egui(button: &ButtonDef) -> EguiAction` 转换含 confirm 包装的按钮动作
- 注：`StartWinter` 已泛化为 `StartAtLabel(String)`（RFC-012）

## KeyFlow

1. `egui_screens` 中的 UI 构建函数返回 `EguiAction`
2. `host_app` 拦截 `ShowConfirm` 变体，弹出确认对话框；用户确认后转发内嵌的 `on_confirm` 动作
3. 其余动作由 `handle_egui_action` 翻译为 `AppState` 操作（导航、存档、设置保存等）

## Dependencies

- `host::app`（`AppState`、存档/加载/重启函数）
- `winit::event_loop::ActiveEventLoop`（用于 `Exit` 动作）

## Invariants

- `EguiAction` 是 UI 层到应用层的单向通信通道
- 所有 UI 交互副作用集中在 `handle_egui_action` 中处理

## WhenToReadSource

- 需要添加新的 UI 交互动作时
- 需要修改现有动作的副作用逻辑时

## LastVerified

2026-03-15

## Owner

Ring-rs 维护者
