# host/renderer/animation 摘要

## Purpose

`renderer/animation` 提供通用动画时间轴系统，统一驱动角色、背景过渡、场景遮罩等对象属性变化。

## PublicSurface

- 模块入口：`host/src/renderer/animation/mod.rs`
- 核心类型：`AnimationSystem`、`Animation`、`AnimationState`、`EasingFunction`
- Trait API：`Animatable`、`ObjectId`、`AnimPropertyKey`、`PropertyAccessor`

## KeyFlow

1. 调用方将实现 `Animatable` 的对象注册到 `AnimationSystem`。
2. 按属性名提交动画（起止值、时长、缓动函数）。
3. 每帧 `update(dt)` 推进时间轴并产生事件。
4. 渲染或逻辑层读取对象属性得到当前插值结果。

## Dependencies

- 被 `renderer`、`scene_transition`、角色/背景动画路径消费

## Invariants

- 动画系统只负责时间与插值，不感知对象业务语义。
- 属性命名是跨模块协作契约，需保持一致。

## FailureModes

- 属性键不匹配导致动画不生效。
- 动画对象注册/注销时机不正确导致悬挂状态或内存增长。

## WhenToReadSource

- 需要新增可动画对象或属性时。
- 需要调优缓动曲线、跳过行为或动画事件处理时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [renderer_scene_transition 摘要](renderer_scene_transition.md)

## LastVerified

2026-03-18

## Owner

Composer