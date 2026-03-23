# host/src/rendering_types.rs

## Purpose

渲染抽象层（RFC-008）。它把 `renderer` / `resources` 与具体 GPU 后端隔开，只暴露纹理工厂、纹理句柄和绘制命令，并保留 `NullTexture` 供 headless 路径与单测复用。

## PublicSurface

- `Texture`：统一纹理接口，含尺寸、估算占用与 `as_any`
- `TextureFactory`：从 RGBA 字节创建纹理
- `TextureContext`：把工厂注入 `ResourceManager`
- `DrawCommand`：`Sprite` / `Rect` / `Dissolve`
- `NullTexture` / `NullTextureFactory`：无 GPU 的测试实现

## KeyFlow

1. `backend::WgpuBackend::texture_context()` 构造 `TextureContext`。
2. `ResourceManager` 通过 `TextureFactory` 把解码后的字节转成 `Arc<dyn Texture>`。
3. `Renderer` 只生产 `DrawCommand`；wgpu 后端在内部再向下转型到 `GpuTexture`。

## Invariants

- `DrawCommand` 始终持有 `Arc<dyn Texture>`，渲染逻辑不直接依赖 `wgpu` 类型。
- 向下转型只发生在 backend 内部。
- `NullTexture` / `NullTextureFactory` 常驻编译产物，不是仅测试可用。

## FailureModes

- 若把非 `GpuTexture` 送入 wgpu backend 的 dissolve/sprite 路径，会在 backend 内部触发 downcast 失败。

## RelatedDocs

- [backend 摘要](backend.md)
- [renderer 摘要](renderer.md)
- [仓库导航地图](../../navigation-map.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4