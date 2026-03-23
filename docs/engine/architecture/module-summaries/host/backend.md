# host/backend 模块摘要

## Purpose

`backend` 是 GUI 渲染落地层：封装 `winit + wgpu + egui`，负责窗口事件接入、GPU 帧渲染以及与抽象纹理系统的桥接。

## PublicSurface

- `WgpuBackend`：后端门面，负责初始化、事件转发、resize、逐帧渲染
- `GpuContext`：设备/队列/表面管理
- `EguiIntegration`：egui 输入与渲染桥接
- `SpriteRenderer`、`DissolveRenderer`：消费 `DrawCommand`
- `GpuTexture`：`Texture` 的 wgpu 实现
- `math`：公共顶点与投影工具

## KeyFlow

1. `WgpuBackend::new()` 初始化 `GpuContext`、sprite/dissolve 渲染器与 egui。
2. `texture_context()` 暴露 `TextureFactory` 桥，把资源解码结果接入 GPU 纹理。
3. `render_frame()` 按顺序执行清屏、sprite、dissolve、egui 叠加。
4. `handle_window_event()` 先把窗口事件交给 egui。

## Invariants

- 所有 GPU 资源（纹理/缓冲区）通过 `Arc` 共享，跨帧安全。
- `SpriteRenderer` 和 `DissolveRenderer` 共享 `texture_bind_group_layout` 和 `QuadVertex` 顶点布局。
- egui 事件优先处理；未被 egui 消费的事件才转发给 `InputManager`。

## RelatedDocs

- [host 总览](../host.md)
- [rendering_types 摘要](rendering-types.md)
- [仓库导航地图](../../navigation-map.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4