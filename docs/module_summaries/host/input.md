# host/input 摘要

## Purpose

`input` 负责采集 macroquad 输入事件，并根据 `WaitingReason` 转换为 `RuntimeInput`，含点击防抖和长按快进策略。

## PublicSurface

- 模块入口：`host/src/input/mod.rs`
- 核心类型：`InputManager`
- 关键接口：`update`、`reset_choice`、`set_choice_rects`、`inject_input`

## KeyFlow

1. `update(waiting, dt)` 根据等待原因选择输入处理分支。
2. Click 等待态支持单击与长按重复触发，带防抖窗口。
3. Choice 等待态维护选项索引、鼠标悬停和键盘导航。
4. 支持外部注入输入（信号桥接场景）。

## Dependencies

- 依赖 `macroquad::prelude` 读取键鼠状态
- 依赖 `vn_runtime::{RuntimeInput, WaitingReason}`

## Invariants

- 输入解释必须与当前等待态一致，避免“错误类型输入”泄漏到 Runtime。
- 选择索引受 `choice_count` 约束，防止越界。

## FailureModes

- 防抖与长按阈值配置不当，导致推进过快或不响应。
- 选择框矩形不同步，导致鼠标悬停/点击错位。

## WhenToReadSource

- 需要调整推进节奏（点击、长按、选择确认）时。
- 需要排查“输入被吞/重复触发”问题时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app_update.md)
- [vn-runtime runtime 摘要](../vn-runtime/runtime.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
