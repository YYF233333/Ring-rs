# egui_screens 摘要

## Purpose

按 `AppMode` 组织的 egui UI 页面构建函数集合。每个子模块对应一个界面，返回 `EguiAction` 供 `host_app` 消费。

## PublicSurface

- 目录：`host/src/egui_screens/`
- 子模块与入口函数：
  - `title::build_title_ui` -- 主标题界面（开始/继续/设置/退出）
  - `ingame::build_ingame_ui` -- 游戏中对话框与选项
  - `ingame_menu::build_ingame_menu_ui` -- 游戏内暂停菜单
  - `settings::build_settings_ui` -- 设置界面（音量/文字速度/全屏等）
  - `save_load::build_save_load_ui` -- 存档/读档界面
  - `history::build_history_ui` -- 对话历史回看
  - `toast::build_toast_overlay` -- Toast 通知浮层
  - `helpers` -- 公共 UI 常量（`DARK_BG`、`PANEL_BG`、`GOLD`）与辅助函数（`dark_frame`、`panel_frame`、`menu_btn`）

## KeyFlow

1. `host_app` 根据当前 `AppMode` 调用对应的 `build_*_ui` 函数
2. 各函数在 egui `Context` 上构建 UI，返回用户触发的 `EguiAction`
3. `build_toast_overlay` 在所有页面上层叠加 Toast 通知

## Dependencies

- `egui`（UI 构建 API）
- `host::app::AppState`（读取游戏状态用于 UI 展示）
- `EguiAction`（UI 交互输出）

## Invariants

- 各页面函数是纯 UI 构建，不直接修改 `AppState`（副作用通过 `EguiAction` 传递）
- `helpers` 模块提供统一视觉风格常量，确保各页面外观一致

## WhenToReadSource

- 需要修改某个界面的布局或交互时
- 需要新增界面页面时

## LastVerified

2026-03-11

## Owner

Ring-rs 维护者
