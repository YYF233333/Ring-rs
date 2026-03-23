# host/renderer 摘要

## Purpose

`renderer` 把 `RenderState` 转成 `DrawCommand`，并维护背景过渡、场景遮罩、角色层级与镜头效果。对话框等 UI 文本不在这里绘制，而是由 egui 路径处理。

## PublicSurface

- 模块入口：`host/src/renderer/mod.rs`
- 核心类型：`Renderer`、`DrawMode`
- 关键子模块：`draw_commands`、`scene_effects`、`transition`、`scene_transition`
- 常用接口：`build_draw_commands`、背景过渡更新、`start_scene_*`、`skip_scene_transition_to_end`

## KeyFlow

1. `build_draw_commands` 依次生成背景、角色、场景效果遮罩与场景过渡遮罩。
2. 背景 dissolve 由 `TransitionManager` 驱动。
3. `changeScene` 的 Fade/FadeWhite/Rule 由 `SceneTransitionManager` 负责阶段推进。
4. backend 只消费产出的 `DrawCommand`，不回写渲染状态。

## Dependencies

- 依赖 `rendering_types::{DrawCommand, Texture}`（渲染抽象层，不直接依赖 backend）
- 依赖 `resources::ResourceManager` 获取纹理（返回 `Arc<dyn Texture>`）
- 依赖 `manifest::Manifest` 获取立绘锚点/站位配置
- 依赖 `render_state` 与动画/过渡子模块

## Invariants

- `Renderer` 只消费状态，不承载脚本语义。
- 渲染层级固定：背景 -> 角色 -> 场景效果 -> 场景过渡；UI 叠加在 backend/egui 路径完成。

## FailureModes

- 纹理缺失导致背景或角色不可见。
- Rule 过渡缺少遮罩纹理或后端无法消费遮罩命令，导致视觉异常。
- 场景过渡中点处理不当导致背景切换时机错误。

## WhenToReadSource

- 需要调整渲染层级或缩放/锚点布局规则时。
- 需要排查 `changeScene`、Rule、Skip 相关视觉问题时。

## RelatedDocs

- [host 总览](../host.md)
- [renderer_render_state 摘要](renderer-render-state.md)
- [renderer_animation 摘要](renderer-animation.md)
- [renderer_effects 摘要](renderer-effects.md)
- [renderer_scene_transition 摘要](renderer-scene-transition.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4