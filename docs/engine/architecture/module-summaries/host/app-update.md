# host/app/update 摘要

## Purpose

`app/update` 是每帧更新入口：负责按 `AppMode` 分发逻辑，并统一推进 InGame 共享状态、cutscene、场景过渡与音频。

## PublicSurface

- 入口：`host/src/app/update/mod.rs`
- 关键接口：`update(app_state, dt)`、`handle_script_mode_input`、`run_script_tick`、`skip_all_active_effects`
- 主要子模块：`modes`、`script`、`scene_transition`

## KeyFlow

1. `update()` 先刷新 `UiContext` 与 `ToastManager`，再按当前 `AppMode` 分发。
2. `modes.rs` 只保留模式相关逻辑；非 InGame 页面基本是 no-op，InGame 下负责 Normal/Auto/Skip、打字机与等待态输入。Backspace 在 ESC 等逻辑之前检测，调用 `snapshot::rollback` 做快照回退。
3. `tick_ingame_shared()` 在模式分发后统一推进背景/场景过渡、WaitForTime、scene effect、title card、video 与动画清理。
4. `script.rs` 负责 Runtime tick、`Command` 执行、`RequestUI` 路由、cutscene 启停与结束信号回传；`run_script_tick` 在停止点推进前自动保存快照。
5. 音频 `update(dt)` 在所有模式下统一放在 `update()` 末尾推进。

## Invariants

- 每帧只执行一个 `AppMode` 分支；共享的 InGame 时间推进收敛在 `tick_ingame_shared()`。
- `skip_all_active_effects()` 是 Skip 模式的统一收敛入口。
- `RequestUI` 与 `Cutscene` 的上层编排在 `script.rs`，不在 `command_executor` 内完成。

## WhenToReadSource

- 需要修改 Normal/Auto/Skip 的推进语义时。
- 需要排查等待态、scene signal 或 cutscene 恢复链路时。
- 需要确认某个 UI mode / 小游戏请求在哪一帧被激活时。

## RelatedDocs

- [host 总览](../host.md)
- [app 摘要](app.md)
- [command_executor 摘要](command-executor.md)
- [renderer_scene_transition 摘要](renderer-scene-transition.md)

## LastVerified

2026-03-24

## Owner

claude-4.6-opus
