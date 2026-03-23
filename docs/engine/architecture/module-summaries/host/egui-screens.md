# egui_screens 摘要

## Purpose

`egui_screens` 提供各页面的 egui 构建函数。它们只负责把给定数据渲染成界面，并返回 `EguiAction`。

## PublicSurface

- 目录：`host/src/egui_screens/`
- 主要页面：`title`、`ingame`、`ingame_menu`、`game_menu`、`settings`、`save_load`、`history`
- 叠加层：`confirm`、`skip_indicator`、`toast`

## KeyFlow

1. 页面选择与叠加层编排不再散落在 `host_app`，而是由共享的 `build_ui::build_frame_ui()` 统一调用这些构建函数。
2. 页面函数通过 `UiRenderContext` 读取布局、素材、条件和 `ScreenDefinitions`，不直接持有 `AppState`。
3. Settings / SaveLoad / History 通过 `game_menu` 共享统一框架。
4. `confirm`、`skip_indicator`、`toast` 属于覆盖层；活跃 `ui_modes` 则走独立渲染路径，不属于本目录页面。

## Invariants

- 页面构建函数本身不直接修改应用状态。
- 页面行为优先由 `ScreenDefinitions` 与 `UiRenderContext` 提供的数据驱动。
- 覆盖层顺序由 `build_ui` 统一控制，避免各调用方自行拼装。

## WhenToReadSource

- 需要改某个页面布局或按钮行为时。
- 需要排查 `UiRenderContext` 某项数据如何被页面消费时。
- 需要确认某个元素属于普通页面、覆盖层，还是 `ui_modes` 独立渲染路径时。

## RelatedDocs

- [host_app 摘要](host-app.md)
- [egui_actions 摘要](egui-actions.md)
- [ui 摘要](ui.md)
- [ui_modes 摘要](ui-modes.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
