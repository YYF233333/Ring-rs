# host/renderer/animation 摘要

## Purpose

`renderer/animation` 提供 trait-based 时间轴系统，用统一接口驱动角色、场景过渡等对象的 `f32` 属性变化；它只管理插值与生命周期，不理解对象业务语义。

## PublicSurface

- 模块入口：`host/src/renderer/animation/mod.rs`
- 核心类型：`AnimationSystem`、`Animation`、`AnimationState`、`AnimationEvent`、`EasingFunction`
- Trait API：`Animatable`、`ObjectId`、`AnimPropertyKey`、`PropertyAccessor`

## KeyFlow

1. 调用方注册实现 `Animatable` 的对象，获得 `ObjectId`。
2. 动画系统按属性名启动插值，并在 `update(dt)` 中推进。
3. 对象自己通过 `get_property` / `set_property` 暴露和承接变化。

## Dependencies

- 被 `renderer`、`scene_transition`、角色/背景动画路径消费

## Invariants

- 动画系统只负责时间与插值。
- 属性名是跨模块契约，调用方与对象实现必须一致。

## FailureModes

- 属性键不匹配导致动画不生效。
- 动画对象注册/注销时机不正确导致悬挂状态或内存增长。

## WhenToReadSource

- 需要新增可动画对象或属性时。
- 需要调优缓动曲线、跳过行为或动画事件处理时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [renderer_scene_transition 摘要](renderer-scene-transition.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4