# host/backend 模块摘要

## Purpose

GPU 渲染后端：封装 winit 窗口管理、wgpu GPU 初始化与帧渲染循环、egui UI 集成。
替代了原 macroquad 渲染层（RFC-007）。

## PublicSurface

- `WgpuBackend`：窗口/GPU 生命周期管理、帧渲染入口
- `SpriteRenderer`：2D textured quad batch 渲染器（WGSL shader）
- `DissolveRenderer`：mask-based dissolve 效果渲染器（WGSL shader）
- `GpuTexture`：`Arc`-wrapped wgpu 纹理 + 视图 + 绑定组
- `GpuResourceContext`：共享 GPU 设备/队列/渲染器引用，供 `ResourceManager` 加载纹理
- `DrawCommand`：绘制命令枚举（Sprite / Rect / Dissolve）

## KeyFlow

1. `WgpuBackend::new(window, font_data)` 初始化 GPU、surface、egui 集成。
2. 每帧 `render_frame(build_ui, sprite_commands)` 执行：
   - 清屏 + sprite 命令批量绘制（SpriteRenderer）
   - Dissolve 命令单独绘制（DissolveRenderer）
   - egui UI 叠加层（通过 `build_ui` 闭包构建）
3. `handle_window_event` 转发事件给 egui，返回是否已消费。
4. `gpu_resource_context()` 提供 `GpuResourceContext` 供异步纹理加载。

## Invariants

- 所有 GPU 资源（纹理/缓冲区）通过 `Arc` 共享，跨帧安全。
- `SpriteRenderer` 和 `DissolveRenderer` 共享 `texture_bind_group_layout`。
- egui 事件优先处理；未被 egui 消费的事件才转发给 `InputManager`。

## LastVerified

2026-03-11
