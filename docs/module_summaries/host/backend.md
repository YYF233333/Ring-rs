# host/backend 模块摘要

## Purpose

GPU 渲染后端：封装 winit 窗口管理、wgpu GPU 初始化与帧渲染循环、egui UI 集成。

## PublicSurface

- `WgpuBackend`：渲染后端门面，组合 `GpuContext` + `EguiIntegration`，编排帧渲染流程
- `GpuContext`：GPU 设备/队列/表面管理（初始化、resize、帧获取）
- `EguiIntegration`：egui 上下文/状态/渲染器桥接（输入转发、字体加载、UI 渲染）
- `SpriteRenderer`：2D textured quad batch 渲染器（WGSL shader）
- `DissolveRenderer`：mask-based dissolve 效果渲染器（WGSL shader）
- `GpuTexture`：`Arc`-wrapped wgpu 纹理 + 视图 + 绑定组（实现 `rendering_types::Texture` trait）
- `WgpuTextureFactory`：`TextureFactory` 的 wgpu 实现（内部类型，通过 `texture_context()` 暴露）
- `DrawCommand`：re-export from `rendering_types`（使用 `Arc<dyn Texture>`）
- `math`：公共渲染工具（`QuadVertex`、`orthographic_projection`、`quad_vertices`）

## KeyFlow

1. `WgpuBackend::new(window, font_data)` 初始化 `GpuContext`、渲染器、`EguiIntegration`。
2. 每帧 `render_frame(build_ui, sprite_commands)` 执行：
   - 清屏 + sprite 命令批量绘制（SpriteRenderer）
   - Dissolve 命令单独绘制（DissolveRenderer）
   - egui UI 叠加层（通过 `build_ui` 闭包构建）
3. `handle_window_event` 转发事件给 egui，返回是否已消费。
4. `texture_context()` 提供 `TextureContext`（含 `WgpuTextureFactory`）供纹理加载。
5. `SpriteRenderer` / `DissolveRenderer` 通过 `as_any().downcast_ref::<GpuTexture>()` 恢复具体纹理类型以访问 `bind_group`。

## Invariants

- 所有 GPU 资源（纹理/缓冲区）通过 `Arc` 共享，跨帧安全。
- `SpriteRenderer` 和 `DissolveRenderer` 共享 `texture_bind_group_layout` 和 `QuadVertex` 顶点布局。
- egui 事件优先处理；未被 egui 消费的事件才转发给 `InputManager`。

## LastVerified

2026-03-11
