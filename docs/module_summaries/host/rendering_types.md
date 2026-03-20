# host/src/rendering_types.rs

## Purpose

渲染抽象层（RFC-008）。定义 `Texture` trait、`TextureFactory` trait、`DrawCommand` 枚举以及 `TextureContext`，
将 `renderer/` 和 `resources/` 模块与 wgpu 具体类型解耦。同时提供 `NullTexture` / `NullTextureFactory` 用于 headless 测试。

## PublicSurface

| 类型 | 用途 |
|------|------|
| `trait Texture` | 纹理抽象接口（`width`/`height`/`width_u32`/`height_u32`/`size_bytes`/`as_any`） |
| `trait TextureFactory` | 纹理创建工厂接口（`create_texture`） |
| `TextureContext` | 持有 `Arc<dyn TextureFactory>`，注入到 `ResourceManager` |
| `DrawCommand` | 绘制命令枚举（`Sprite`/`Rect`/`Dissolve`），使用 `Arc<dyn Texture>` |
| `NullTexture` | Headless 纹理实现（仅存储尺寸） |
| `NullTextureFactory` | Headless 纹理工厂（创建 `NullTexture`） |

## KeyFlow

```
ResourceManager --set_texture_context()--> TextureContext --create_texture()--> Arc<dyn Texture>
Renderer --build_draw_commands()--> Vec<DrawCommand> { Arc<dyn Texture> }
WgpuBackend --render_frame()--> SpriteRenderer/DissolveRenderer (downcast to GpuTexture)
```

## Dependencies

- `std::any::Any`（downcast 支持）
- 无外部 crate 依赖

## Invariants

- `Texture` trait 要求 `Send + Sync + Debug + 'static`
- `GpuTexture`（`backend/gpu_texture.rs`）实现 `Texture` trait
- `DrawCommand` 使用 `Arc<dyn Texture>` 而非具体的 `Arc<GpuTexture>`
- Backend 内部通过 `as_any().downcast_ref::<GpuTexture>()` 恢复具体类型
- `NullTexture` / `NullTextureFactory` 无条件编译（不限于 `#[cfg(test)]`）

## FailureModes

- Backend downcast 失败（传入非 `GpuTexture` 的纹理到 wgpu backend）：panic with expect message

## RelatedDocs

- [backend 摘要](backend.md)
- [renderer 摘要](renderer.md)
- [仓库导航地图](../../navigation_map.md)

## LastVerified

2026-03-20

## Owner

Composer