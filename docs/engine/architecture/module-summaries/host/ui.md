# host/ui 摘要

## Purpose

`ui` 提供 UI 基础设施：数据驱动的布局配置（`UiLayoutConfig`）、分辨率缩放（`ScaleContext`）、
素材缓存（`UiAssetCache`）、NinePatch 九宫格渲染、Toast 通知、UI 上下文、
声明式界面行为定义（`ScreenDefinitions`）和统一渲染上下文（`UiRenderContext`）。
界面渲染使用 egui（在 `main.rs` / `egui_screens/` 中构建）。

## PublicSurface

- 模块入口：`host/src/ui/mod.rs`
- 核心类型：
  - `UiLayoutConfig`：数据驱动的布局参数（对话框/选项/菜单/存档等），从 `ui/layout.json` 加载，配置文件必须存在
  - `ScaleContext`：基准分辨率 → 实际窗口尺寸的缩放映射
  - `UiAssetCache`：将 GUI 图片素材加载为 `egui::TextureHandle` 缓存
  - `NinePatch`/`Borders`：九宫格渲染器，用于可拉伸 UI 元素
  - `UiContext`：存储屏幕尺寸、缩放上下文
  - `ScreenDefinitions`：声明式界面行为定义（按钮列表、动作映射、可见性条件、背景切换），从 `ui/screens.json` 加载，配置文件必须存在
  - `UiRenderContext`：统一渲染上下文，合并 `(layout, assets, scale, screen_defs, conditions)` 为一体
  - `ConditionContext`：条件求值上下文（`has_continue` + `&PersistentStore`）
  - `ToastManager`
- 错误类型：`LayoutConfigError`、`ScreenDefsError`
- 子模块：`layout`、`asset_cache`、`nine_patch`、`toast`、`screen_defs`、`render_context`

## KeyFlow

1. 启动时 `UiLayoutConfig::load()` 从 `ui/layout.json` 加载布局，`ScreenDefinitions::load()` 从 `ui/screens.json` 加载行为定义。两者均返回 `Result`，文件缺失或字段缺失时报错退出。
2. `UiAssetCache::load()` 在 WgpuBackend 初始化后、首帧渲染前加载所有 GUI 素材为 `egui::TextureHandle`。
3. `ScaleContext` 由 `UiContext` 持有，随 winit resize 事件更新。
4. `host_app.rs` 每帧构造 `UiRenderContext`（含预求值的 `ConditionContext`），传给所有 `build_*` 函数。
5. `build_*` 函数从 `UiRenderContext` 读取布局、素材、条件和界面定义，不直接访问 `AppState`。
6. `UiAssetPaths` 定义所有 GUI 素材的逻辑路径。

## Dependencies

- `image` crate（解码 PNG/JPEG 为 RGBA）
- `serde`/`serde_json`（布局配置反序列化）
- `egui`（TextureHandle、ColorImage）
- 被 `app::UiSystems` 消费（layout + asset_cache 存储在 UiSystems 中）

## Invariants

- `UiLayoutConfig` 中的像素值均基于 1920×1080 基准分辨率，渲染时通过 `ScaleContext` 缩放。
- `UiAssetCache` 必须在 `egui::Context` 可用后创建（需要 `ctx.load_texture`）。
- `TextureHandle` 是 `Arc` 引用计数，`UiAssetCache` 持有时纹理不被释放。
- 旧的 `Theme`/`skin` 系统已移除，颜色/尺寸全部由 `UiLayoutConfig` 统一管理。
- 所有配置结构体使用 `#[serde(deny_unknown_fields)]` 拒绝未知字段。
- 默认值存放在外部 JSON 文件中，代码内 `impl Default` 仅供测试使用。

## WhenToReadSource

- 需要新增 UI 布局参数或素材路径时（同时需更新 `assets/ui/layout.json`）。
- 需要理解 NinePatch 渲染或素材加载流程时。
- 需要扩展主题 token 或新增 Toast 类型时。

## RelatedDocs

- [host 总览](../host.md)
- [backend 摘要](backend.md)
- [RFC: 可定制 UI 系统](../../../../../RFCs/Accepted/rfc-customizable-ui-system.md)
- [RFC: UI 行为定制系统](../../../../../RFCs/Accepted/rfc-ui-behavior-customization.md)
- [RFC-013: 配置默认值外部化](../../../../../RFCs/Accepted/rfc-config-externalization.md)
- [UI 行为定制指南](../../../ui/screens-customization.md)

## LastVerified

2026-03-18

## Owner

Composer