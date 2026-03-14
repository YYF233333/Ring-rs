# host/ui 摘要

## Purpose

`ui` 提供 UI 基础设施：数据驱动的布局配置（`UiLayoutConfig`）、分辨率缩放（`ScaleContext`）、
素材缓存（`UiAssetCache`）、NinePatch 九宫格渲染、Toast 通知和 UI 上下文。
界面渲染使用 egui（在 `main.rs` / `egui_screens/` 中构建）。

## PublicSurface

- 模块入口：`host/src/ui/mod.rs`
- 核心类型：
  - `UiLayoutConfig`：数据驱动的布局参数（对话框/选项/菜单/存档等），JSON 可覆盖，默认值对齐 ref-project `gui.rpy`
  - `ScaleContext`：基准分辨率 → 实际窗口尺寸的缩放映射
  - `UiAssetCache`：将 GUI 图片素材加载为 `egui::TextureHandle` 缓存
  - `NinePatch`/`Borders`：九宫格渲染器，用于可拉伸 UI 元素
  - `UiContext`：存储屏幕尺寸、缩放上下文
  - `ToastManager`
- 子模块：`layout`、`asset_cache`、`nine_patch`、`toast`

## KeyFlow

1. 启动时 `UiLayoutConfig::load()` 尝试从 `ui/layout.json` 加载布局，失败回退默认。
2. `UiAssetCache::load()` 在 WgpuBackend 初始化后、首帧渲染前加载所有 GUI 素材为 `egui::TextureHandle`。
3. `ScaleContext` 由 `UiContext` 持有，随 winit resize 事件更新。
4. 各 egui 页面接收 `&UiLayoutConfig` + `Option<&UiAssetCache>` + `&ScaleContext` 参数，实现数据驱动渲染。
5. `UiAssetPaths` 定义所有 GUI 素材的逻辑路径，支持 JSON 覆盖。

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

## WhenToReadSource

- 需要新增 UI 布局参数或素材路径时。
- 需要理解 NinePatch 渲染或素材加载流程时。
- 需要扩展主题 token 或新增 Toast 类型时。

## RelatedDocs

- [host 总览](../host.md)
- [backend 摘要](backend.md)
- RFC-010: 可定制 UI 系统

## LastVerified

2026-03-15

## Owner

Ring-rs 维护者
