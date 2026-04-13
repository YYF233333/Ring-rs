---
paths:
  - "host/src/renderer/**"
  - "host/src/rendering_types.rs"
  - "host/src/backend/**"
---

# 渲染与效果（renderer）

## 摘要导航

- [renderer](docs/engine/architecture/module-summaries/host/renderer.md)
- [renderer_render_state](docs/engine/architecture/module-summaries/host/renderer-render-state.md)
- [renderer_animation](docs/engine/architecture/module-summaries/host/renderer-animation.md)
- [renderer_effects](docs/engine/architecture/module-summaries/host/renderer-effects.md)
- [renderer_scene_transition](docs/engine/architecture/module-summaries/host/renderer-scene-transition.md)
- [rendering_types](docs/engine/architecture/module-summaries/host/rendering-types.md)
- [backend](docs/engine/architecture/module-summaries/host/backend.md)
- 效果 capability：[extension_effects_capability.md](docs/engine/reference/extension-effects-capability.md)

## 关键不变量

- `DrawCommand` 使用 `Arc<dyn Texture>`，后端通过 `as_any()` downcast 获取具体 GPU 纹理。
- `NullTexture` / `NullTextureFactory` 用于 headless 测试——渲染逻辑不可假设一定有真 GPU。
- 效果通过 `EffectKind → ResolvedEffect → EffectRequest` 三级解析，支持 capability 回退。
- 动画系统基于 `Animatable` trait，时间驱动（`dt`），不帧数驱动。
- 场景过渡由 `SceneTransitionManager` 管理，支持 `skip_to_end()` 以配合 Skip 模式。

## Do / Don't

- **Do** 新增效果时在 `EffectKind` 注册并实现 `resolve()` 分支。
- **Do** 为 GPU shader 逻辑写 headless 单测（使用 `NullTexture`）。
- **Don't** 在 renderer 中直接访问 `wgpu` 类型——通过 `Texture` trait 抽象。
- **Don't** 在 `backend/` 之外 downcast `Arc<dyn Texture>` 到 `GpuTexture`。
