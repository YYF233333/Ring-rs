# host/input 摘要

## Purpose

`input` 基于 winit 事件驱动：`InputManager` 作为编排器，组合 `InputState`（设备状态、防抖、长按）与 `ChoiceNavigator`（选择分支导航），根据 `WaitingReason` 将键鼠事件转换为 `RuntimeInput`。

## PublicSurface

- 模块入口：`host/src/input/mod.rs`（`InputManager` 编排；内部组合 `state::InputState`、`ChoiceNavigator`）
- `host/src/input/state.rs`：`InputState`（键鼠状态、防抖、长按等）
- `host/src/input/choice_navigator.rs`：`ChoiceNavigator`（选项索引/悬停/确认导航）
- 子模块：`recording`（`host/src/input/recording.rs`）— 录制/回放：`InputEvent`、`RecordingBuffer`、`RecordingExporter`、`InputReplayer`（职责与路径不变）
- 核心类型：`InputManager`
- 关键接口：`process_event`、`process_input_event`、`inject_replay_events`、`begin_frame`、`end_frame`、`update`、`set_choice_rects`、`inject_input`、`inject_ui_result`、`suppress_mouse_click`、`recording_snapshot`、`enable_recording`
- 公开查询：`is_key_just_pressed`、`is_key_down`、`mouse_position`、`is_mouse_pressed`、`is_mouse_just_pressed`

## KeyFlow

1. `process_event(WindowEvent)` 接收 winit 事件，可转为语义 `InputEvent` 写入 `recording_buffer`（若启用），并委托 `InputState` 更新键鼠状态。
2. `process_input_event(InputEvent)` 为录制/回放共用入口，直接更新 `InputState`。
3. `begin_frame(dt)` 推进内部时间计数器（含 `elapsed_ms`），主要作用于 `InputState`。
4. `update(waiting, dt)` 由 `InputManager` 编排：结合 `InputState` 与 `ChoiceNavigator`，按等待原因选择分支并填充 `pending_input`。
5. `end_frame()` 清除 per-frame 的 "just pressed" 状态（`InputState`）。
6. egui 事件优先处理；当 egui 交互元素处于指针下方时可调用 `suppress_mouse_click()` 抑制本帧点击，未被 egui 消费的事件才转发给 InputManager。
7. 录制：`enable_recording(size_mb)` 启用环形缓冲区；`recording_snapshot()` 供导出。回放：`inject_replay_events(events)` 注入事件（headless 用）。导出见 `RecordingExporter`；加载见 `InputReplayer`。
8. `WaitForUIResult` 与 `WaitForSignal` 相同处理：不采集 winit 输入，结果由上层程序化注入。普通 egui UI 交互（地图选择等）用 `inject_input`；WebView 等脱离引擎输入管线的交互用 `inject_ui_result`。
9. `inject_ui_result(key, value)` 同时将 `UIResult` 事件写入录制缓冲区并注入 `pending_input`，仅用于 WebView 小游戏等无法通过物理输入录制重现的场景。回放时 `inject_replay_events` 遇到 `InputEvent::UIResult` 会转换为 `RuntimeInput::UIResult` 注入。

## Dependencies

- 依赖 `winit::event`、`winit::keyboard` 接收事件
- 依赖 `vn_runtime::{RuntimeInput, WaitingReason}`

## Invariants

- 输入解释必须与当前等待态一致，避免"错误类型输入"泄漏到 Runtime。
- 选择索引受 `choice_count` 约束，防止越界。
- `begin_frame` / `end_frame` 必须成对调用，确保 per-frame 状态正确清除。

## FailureModes

- 防抖与长按阈值配置不当，导致推进过快或不响应。
- 选择框矩形不同步，导致鼠标悬停/点击错位。

## WhenToReadSource

- 需要调整推进节奏（点击、长按、选择确认）时。
- 需要排查"输入被吞/重复触发"问题时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app-update.md)
- [vn-runtime runtime 摘要](../vn-runtime/runtime.md)

## LastVerified

2026-03-22

## Owner

claude-4.6-opus