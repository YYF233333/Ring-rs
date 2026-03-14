# egui_screens 摘要

## Purpose

按 `AppMode` 组织的 egui UI 页面构建函数集合。每个子模块对应一个界面，返回 `EguiAction` 供 `host_app` 消费。
所有页面使用数据驱动的 `UiLayoutConfig` + `UiAssetCache` + `ScaleContext` 系统（RFC-010）。

## PublicSurface

- 目录：`host/src/egui_screens/`
- 子模块与入口函数：
  - `title::build_title_ui` -- 主标题界面（全屏背景 + 左侧中文导航）。根据 `persistent_store.complete_summer` 切换夏/冬篇背景，条件显示冬篇入口按钮。
  - `ingame::build_ingame_ui` -- 游戏中对话框（图片背景 + 名字框 + 快捷菜单 + 选项）
  - `ingame_menu::build_ingame_menu_ui` -- 游戏内暂停菜单（半透明覆盖 + 居中按钮，退出/返回标题经确认弹窗）
  - `game_menu::build_game_menu_frame` -- 游戏菜单通用框架（左侧导航 + 右侧内容），Settings/SaveLoad/History 共用。接收 `is_winter: bool` 参数切换背景。
  - `settings::build_settings_content` -- 设置内容区（自定义 `ImageSlider` widget + 中文标签），接收 `UiAssetCache` 使用 slider 素材。
  - `save_load::build_save_load_content` -- 存档/读档内容区（Tab 切换 + 网格 + 删除按钮 + 覆盖确认 + 分页导航栏 + 缩略图显示）。接收 `SaveLoadPage` 管理 A/Q/1-9 分页，接收 `thumbnails` HashMap 显示存档截图。
  - `history::build_history_content` -- 对话历史内容区（双列布局：角色名 + 对话文本）
  - `confirm::build_confirm_overlay` -- 确认弹窗（模态覆盖 + NinePatch 框架）
  - `skip_indicator::build_skip_indicator` -- 快进指示器（左上角动画提示）
  - `toast::build_toast_overlay` -- Toast 通知浮层，接收 `layout`/`assets`/`scale` 使用 `notify.png` NinePatch 背景。

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
- `host::ui::{UiLayoutConfig, UiAssetCache, ScaleContext, NinePatch}`（数据驱动 UI 基础设施）
- `host::ui::image_slider`（自定义图片滑块 widget）
- `host::app::AppState`（读取游戏状态用于 UI 展示）
- `EguiAction`（UI 交互输出，含 StartWinter/QuickSave/QuickLoad/ToggleSkip/ToggleAuto/ShowConfirm 等变体）

## Invariants

- 各页面函数是纯 UI 构建，不直接修改 `AppState`（副作用通过 `EguiAction` 传递）
- 所有布局参数从 `UiLayoutConfig` 读取，不硬编码像素值（基准分辨率常量除外）
- 素材不可用时优雅降级到纯色/无背景渲染

## WhenToReadSource

- 需要修改某个界面的布局或交互时
- 需要新增界面页面时
- 需要理解确认弹窗或快捷菜单的交互流程时

## LastVerified

2026-03-15

## Owner

Ring-rs 维护者
