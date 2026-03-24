# host/input 摘要

## Purpose

`input` 负责把 winit 事件、回放事件和程序化 UI 结果收敛成 `RuntimeInput`，并维护每帧输入状态。

## PublicSurface

- 入口：`host/src/input/mod.rs`
- 核心类型：`InputManager`
- 关键协作者：`state::InputState`、`ChoiceNavigator`、`recording`
- 关键接口：`process_event`、`inject_replay_events`、`begin_frame`、`update`、`end_frame`、`inject_input`、`inject_ui_result`

## KeyFlow

1. `process_event()` 消费窗口事件并更新按键/鼠标状态；若启用录制，同时写入缓冲区。
2. `inject_replay_events()` 供 `headless` 重放；物理事件更新输入状态，`UIResult` 直接注入待消费输入。
3. `begin_frame()` / `end_frame()` 围住一帧的输入生命周期，保证 just-pressed、滚轮与位移状态按帧清空。
4. `update(waiting, dt)` 根据当前 `WaitingReason` 选择点击、选项、时间等待或无输入分支。
5. Backspace 用于快照回退：由 `modes.rs` 直接检测，不经过 `WaitingReason` 路由；`recording.rs` 的 `key_code_from_name` 已映射 Backspace，供录制/回放一致识别。
6. `inject_ui_result()` 只用于 WebView 等脱离物理输入管线的交互结果，并会同步录制到回放流。

## Invariants

- 输入解释必须受当前 `WaitingReason` 约束，不能把错误类型输入泄漏给 Runtime。
- `begin_frame()` 与 `end_frame()` 必须成对调用。
- 普通 egui 交互优先走物理输入重放；只有无法重放的外部 UI 结果才走 `inject_ui_result()`。

## WhenToReadSource

- 需要修改点击/长按/选项导航语义时。
- 需要排查输入被吞、重复触发或回放不一致时。
- 需要确认某个交互应走物理输入还是 `UIResult` 注入时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app-update.md)
- [host_app 摘要](host-app.md)
- [vn-runtime runtime 摘要](../vn-runtime/runtime.md)

## LastVerified

2026-03-24

## Owner

claude-4.6-opus
