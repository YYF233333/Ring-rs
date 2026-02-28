# host/screens 摘要

## Purpose

`screens` 定义 Host 页面层（Title/Settings/SaveLoad/History/InGameMenu），承接菜单交互与页面状态。

## PublicSurface

- 模块入口：`host/src/screens/mod.rs`
- 核心页面类型：`TitleScreen`、`SettingsScreen`、`SaveLoadScreen`、`HistoryScreen`、`InGameMenuScreen`

## KeyFlow

1. `app/update/modes` 按当前 `AppMode` 调用对应页面更新逻辑。
2. 页面通过 `ui` 组件与 `UiContext` 计算交互状态。
3. 页面结果驱动导航、设置变更、存档操作等上层行为。

## Dependencies

- 依赖 `ui` 组件库
- 依赖 `app_mode` 的导航状态机
- 与 `save_manager`、`app` 设置/会话状态协作

## Invariants

- 页面层负责交互编排，不直接处理 Runtime 语义执行。
- 导航切换通过 `NavigationStack` 统一管理返回路径。

## FailureModes

- 页面状态与导航栈不一致，导致返回逻辑异常。
- 页面交互结果未正确回写到应用状态，导致 UI 与实际设置不一致。

## WhenToReadSource

- 需要新增页面或调整现有页面流转时。
- 需要排查菜单交互与导航行为时。

## RelatedDocs

- [host 总览](../host.md)
- [ui 摘要](ui.md)
- [app_update 摘要](app_update.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
