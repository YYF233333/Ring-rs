# host/ui 摘要

## Purpose

`ui` 提供页面构建所需的基础设施：布局配置、缩放、GUI 素材缓存、声明式页面定义、Toast，以及统一的渲染上下文。

## PublicSurface

- 入口：`host/src/ui/mod.rs`
- 核心类型：`UiLayoutConfig`、`ScaleContext`、`UiAssetCache`、`UiContext`
- 页面配置类型：`ScreenDefinitions`、`ConditionContext`、`UiRenderContext`
- 其他基础设施：`ToastManager`、`NinePatch`、`image_slider`、`map`

## KeyFlow

1. `AppState::new()` 通过 `ResourceManager` 加载 `UiLayoutConfig` 与 `ScreenDefinitions`。
2. `UiContext` 持有当前逻辑尺寸与 `ScaleContext`，在 resize 时更新。
3. `UiAssetCache` 只能在 egui context 可用后创建，因此由窗口模式首帧或 `headless` UI 初始化阶段补齐。
4. 每帧由 `host_app` / `headless` 构造 `UiRenderContext`，再交给 `build_ui` 和 `egui_screens` 消费。

## Invariants

- UI 布局值基于基准分辨率，经 `ScaleContext` 映射到当前窗口。
- 素材与页面定义走数据驱动配置，而不是把细节硬编码进页面函数。
- `UiRenderContext` 是页面读取 UI 数据的稳定入口；页面本身不应直接依赖 `AppState`。

## WhenToReadSource

- 需要新增 UI 布局参数、页面配置字段或 GUI 素材路径时。
- 需要排查缩放、素材加载或 Toast 生命周期时。
- 需要确认某项页面数据来自 `layout`、`screen_defs` 还是临时渲染上下文时。

## RelatedDocs

- [host 总览](../host.md)
- [egui_screens 摘要](egui-screens.md)
- [egui_actions 摘要](egui-actions.md)
- [UI 行为定制指南](../../../ui/screens-customization.md)
- [RFC: 可定制 UI 系统](../../../../../RFCs/Accepted/rfc-customizable-ui-system.md)
- [RFC: UI 行为定制系统](../../../../../RFCs/Accepted/rfc-ui-behavior-customization.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
