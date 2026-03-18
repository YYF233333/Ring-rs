# host/app/update 摘要

## Purpose

`app/update` 提供 Host 每帧更新主入口，负责按 `AppMode` 分发逻辑，并统一推进过渡、动画、章节标记和音频状态。

## PublicSurface

- 模块入口：`host/src/app/update/mod.rs`
- 对外接口：`update(app_state)`、`handle_script_mode_input`、`run_script_tick`、`skip_all_active_effects`
- 子模块：`modes`、`script`、`scene_transition`

## KeyFlow

1. `update` 先更新 UI 上下文与 Toast。
2. 根据 `navigation.current()` 分发到 `modes::*` 对应界面逻辑。
3. InGame 范围内统一推进背景/场景过渡、动画系统、章节标记状态。
4. 打字机更新：检测 `inline_wait`（定时递减 / 点击等待暂停）、按 `effective_text_speed` 推进字符、`no_wait` 完成后自动触发 Click。
5. 最后统一更新音频管理器状态，保证各模式音频一致推进。

## Dependencies

- 依赖 `app::AppState` 访问 core/ui/session 子系统
- 依赖 `renderer` 进行过渡/动画更新
- 依赖 `app_mode` 提供模式枚举与语义判定

## Invariants

- 每帧只有一个 `AppMode` 分支被执行，状态推进单向且确定。
- InGame 共享更新逻辑与菜单模式更新逻辑分离，避免重复分支。

## FailureModes

- 模式分发缺失或条件错误，导致特定页面不响应输入。
- 场景过渡和脚本推进顺序不一致，造成渲染与等待状态错位。

## WhenToReadSource

- 需要调整推进模式（Normal/Auto/Skip）行为时。
- 需要排查“某模式下更新缺失/重复执行”问题时。

## RelatedDocs

- [host 总览](../host.md)
- [app 摘要](app.md)
- [renderer_scene_transition 摘要](renderer_scene_transition.md)

## LastVerified

2026-03-18

## Owner

Ring-rs 维护者
