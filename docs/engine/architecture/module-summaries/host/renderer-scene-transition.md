# host/renderer/scene_transition 摘要

## Purpose

`renderer/scene_transition` 专管 `changeScene` 的多阶段转场。它把 Fade/FadeWhite/Rule 统一建模为状态机，并用动画系统驱动遮罩属性与 `ui_alpha`。

## PublicSurface

- 模块入口：`host/src/renderer/scene_transition/mod.rs`
- 核心类型：`SceneTransitionManager`、`SceneTransitionType`、`SceneTransitionPhase`
- 关键接口：`start_*`、`update`、`is_at_midpoint`、`skip_current_phase`、`skip_to_end`

## KeyFlow

1. `start_*` 记录待切背景并启动 `FadeIn`。
2. 状态机推进为 `FadeIn -> (Blackout) -> FadeOut -> UIFadeIn -> Completed`。
3. 调用方在 `is_at_midpoint()` 为真时取走 `pending_background` 完成背景切换。
4. Skip 既可跳当前阶段，也可 `skip_to_end()` 直接返回待切背景。

## Dependencies

- 依赖 `renderer/animation` 作为底层时间轴
- 被 `renderer` 与 `app/update/scene_transition` 驱动

## Invariants

- 中点切换必须只触发一次，且在遮罩完全覆盖时发生。
- 过渡结束后 `ui_alpha` 回到可见状态。

## FailureModes

- 中点判定错误导致背景切换过早或丢失。
- 跳过逻辑处理不全导致过渡状态卡住。

## WhenToReadSource

- 需要修改 `changeScene` 阶段语义、黑屏停顿或 UI 淡入策略时。
- 需要排查 Skip 模式下过渡状态异常时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [app_update 摘要](app-update.md)
- [renderer_animation 摘要](renderer-animation.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4