# egui_screens 摘要

## Purpose

按 `AppMode` 组织的 egui UI 页面构建函数集合。每个子模块对应一个界面，返回 `EguiAction` 供 `host_app` 消费。
所有页面通过 `UiRenderContext` 接收统一的渲染上下文（含 `UiLayoutConfig`、`UiAssetCache`、`ScaleContext`、`ScreenDefinitions`、`ConditionContext`），不直接访问 `AppState`。
按钮列表、动作映射、可见性条件和背景切换由 `ScreenDefinitions` 数据驱动（RFC-012）。

## PublicSurface

- 目录：`host/src/egui_screens/`
- 子模块与入口函数：
  - `title::build_title_ui(ctx, &UiRenderContext)` -- 主标题界面。按钮列表、背景切换、可见性条件均从 `ScreenDefinitions` 读取。
  - `ingame::build_ingame_ui(ctx, &RenderState, &UiRenderContext)` -- 游戏中对话框 + 快捷菜单 + 选项。快捷菜单按钮从 `ScreenDefinitions` 读取。
  - `ingame_menu::build_ingame_menu_ui(ctx, &UiRenderContext)` -- 游戏内暂停菜单。按钮列表从 `ScreenDefinitions` 读取。
  - `game_menu::build_game_menu_frame(ctx, title, &UiRenderContext, content_builder)` -- 游戏菜单通用框架（左导航 + 右内容）。背景切换和导航按钮从 `ScreenDefinitions` 读取。
  - `settings::build_settings_content(ui, draft, &UiRenderContext)` -- 设置内容区。
  - `save_load::build_save_load_content(ui, tab, page, save_infos, can_save, &UiRenderContext, thumbnails)` -- 存读档内容区。
  - `history::build_history_content(ui, &[HistoryEvent], &UiRenderContext)` -- 对话历史内容区，接收 `&[HistoryEvent]` 而非 `&AppState`。
  - `confirm::build_confirm_overlay(ctx, dialog, &UiRenderContext)` -- 确认弹窗。
  - `skip_indicator::build_skip_indicator(ctx, &UiRenderContext)` -- 快进指示器。
  - `toast::build_toast_overlay(ctx, toast_manager, &UiRenderContext)` -- Toast 通知浮层。

## KeyFlow

1. `host_app` 根据当前 `AppMode` 调用对应页面构建函数
2. Settings/SaveLoad/History 通过 `build_game_menu_frame` 包裹，共享左侧导航面板
3. 各内容函数通过 `ScaleContext` 将基准 1920×1080 坐标缩放到实际窗口尺寸
4. 使用 `UiAssetCache` 获取 GUI 素材纹理，通过 `NinePatch` 渲染可拉伸元素
5. 返回 `EguiAction`，`host_app` 按需拦截（如 `ShowConfirm`）或转发到 `handle_egui_action`
6. 退出/返回标题/覆盖存档/删除存档操作经 `ShowConfirm` 确认弹窗拦截
7. Skip 指示器在 `InGame` + `Skip` 模式下自动显示
8. 确认弹窗在所有 UI 之上渲染，阻塞其他交互
9. 存读档页面通过 `SaveLoadPage` enum 实现分页（A/Q/1-9），每页 6 slot
10. 保存操作触发帧缓冲截图，下一帧保存为 PNG 缩略图

## Dependencies

- `egui`（UI 构建 API）
- `host::ui::{UiRenderContext, UiLayoutConfig, UiAssetCache, ScaleContext, ScreenDefinitions, ConditionContext, NinePatch}`
- `host::ui::image_slider`（自定义图片滑块 widget）
- `EguiAction`（定义于 `egui_actions.rs`，UI 交互输出，含 StartAtLabel/QuickSave/QuickLoad/ToggleSkip/ToggleAuto/ShowConfirm 等变体）
- 不再依赖 `host::app::AppState`（通过 `UiRenderContext` 接收预求值数据）

## Invariants

- 各页面函数是纯 UI 构建，不直接修改 `AppState`（副作用通过 `EguiAction` 传递）
- 所有布局参数从 `UiLayoutConfig` 读取，不硬编码像素值（基准分辨率常量除外）
- 素材不可用时优雅降级到纯色/无背景渲染

## WhenToReadSource

- 需要修改某个界面的布局或交互时
- 需要新增界面页面时
- 需要理解确认弹窗或快捷菜单的交互流程时

## RelatedDocs

- [host 总览](../host.md)
- [egui_actions 摘要](egui-actions.md)（EguiAction 定义与 handle_egui_action）
- [ui 摘要](ui.md)

## LastVerified

2026-03-18

## Owner

Composer