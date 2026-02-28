# host/renderer 摘要

## Purpose

`renderer` 负责将 `RenderState` 绘制为屏幕画面，管理背景过渡、场景切换遮罩、文本渲染与角色绘制顺序。

## PublicSurface

- 模块入口：`host/src/renderer/mod.rs`
- 核心类型：`Renderer`、`DrawMode`
- 关键公开子系统：`animation`、`effects`、`render_state`、`scene_transition`
- 对外接口：`render`、`update_transition`、`start_scene_*`、`skip_scene_transition_to_end`

## KeyFlow

1. `render` 按层级绘制：背景 -> 角色 -> UI（对话/选项/章节标记）-> 场景遮罩。
2. 普通背景过渡由 `TransitionManager` 驱动 alpha 混合。
3. `changeScene` 场景过渡由 `SceneTransitionManager` 管理多阶段流程。
4. Rule 过渡场景下，`ImageDissolve` shader 根据进度渲染遮罩效果。

## Dependencies

- 依赖 `resources::ResourceManager` 获取纹理
- 依赖 `manifest::Manifest` 获取立绘锚点/站位配置
- 依赖 `render_state` 与动画/过渡子模块

## Invariants

- `Renderer` 只消费状态，不生成脚本语义状态。
- 渲染层级固定，避免 UI 与角色/遮罩层次错乱。

## FailureModes

- 纹理缺失导致背景或角色不可见。
- Rule 过渡缺少遮罩纹理或 shader 未初始化导致异常。
- 场景过渡中点处理不当导致背景切换时机错误。

## WhenToReadSource

- 需要调整渲染层级或缩放/锚点布局规则时。
- 需要排查 `changeScene`、Rule、Skip 相关视觉问题时。

## RelatedDocs

- [host 总览](../host.md)
- [renderer_render_state 摘要](renderer_render_state.md)
- [renderer_animation 摘要](renderer_animation.md)
- [renderer_effects 摘要](renderer_effects.md)
- [renderer_scene_transition 摘要](renderer_scene_transition.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
