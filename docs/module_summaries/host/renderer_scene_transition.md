# host/renderer/scene_transition 摘要

## Purpose

`renderer/scene_transition` 管理 `changeScene` 的多阶段过渡（Fade/FadeWhite/Rule），并通过动画系统驱动遮罩与 UI 透明度。

## PublicSurface

- 模块入口：`host/src/renderer/scene_transition/mod.rs`
- 核心类型：`SceneTransitionManager`、`SceneTransitionType`、`SceneTransitionPhase`
- 关键能力：`start_*`、`update`、`is_at_midpoint`、`skip_current_phase`、`skip_to_end`

## KeyFlow

1. 调用 `start_fade/start_fade_white/start_rule` 建立过渡上下文与待切背景。
2. 状态机按阶段推进：`FadeIn -> (Blackout) -> FadeOut -> UIFadeIn -> Completed`。
3. 中点阶段通过 `is_at_midpoint` 触发背景切换，保证时机唯一。
4. Skip 路径可按阶段跳过，或 `skip_to_end` 一次完成并返回待切背景。

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
- [app_update 摘要](app_update.md)
- [renderer_animation 摘要](renderer_animation.md)

## LastVerified

2026-03-18

## Owner

Ring-rs 维护者
