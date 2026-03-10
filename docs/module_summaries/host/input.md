# host/input 摘要

## Purpose

`input` 基于 winit 事件驱动的输入管理器，根据 `WaitingReason` 将键鼠事件转换为 `RuntimeInput`，含点击防抖和长按快进策略。

## PublicSurface

- 模块入口：`host/src/input/mod.rs`
- 核心类型：`InputManager`
- 关键接口：`process_event`、`begin_frame`、`end_frame`、`update`、`set_choice_rects`、`inject_input`
- 公开查询：`is_key_just_pressed_pub`、`is_key_down_pub`、`mouse_position`、`is_mouse_pressed`

## KeyFlow

1. `process_event(WindowEvent)` 接收 winit 事件，更新内部键鼠状态（HashSet）。
2. `begin_frame(dt)` 推进内部时间计数器。
3. `update(waiting, dt)` 根据等待原因选择输入处理分支。
4. `end_frame()` 清除 per-frame 的 "just pressed" 状态。
5. egui 事件优先处理；未被 egui 消费的事件才转发给 InputManager。

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
- [app_update 摘要](app_update.md)
- [vn-runtime runtime 摘要](../vn-runtime/runtime.md)

## LastVerified

2026-03-11

## Owner

Ring-rs 维护者
