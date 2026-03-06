# host/ui 摘要

## Purpose

`ui` 提供可复用 UI 组件与共享 UI 上下文，支撑主菜单、设置、存档等页面的绘制与交互。

## PublicSurface

- 模块入口：`host/src/ui/mod.rs`
- 核心类型：`UiContext`、`Theme`（阶段29新增 token 分层）
- 组件子模块：`button`、`list`、`modal`、`panel`、`toast`
- 阶段29新增：`slider`、`toggle`、`tab`、`scroll`
- 阶段29新增：`theme_loader`（主题覆盖）、`skin`（皮肤协议加载）

## KeyFlow

1. `UiContext::update` 每帧刷新屏幕尺寸和鼠标状态。
2. 启动时通过 `theme_loader` 加载默认主题 + 覆盖文件（缺失时回退默认主题并诊断）。
3. 页面层（`screens`）读取上下文与组件 API 进行交互判定与绘制。
4. Toast/Modal 等组件统一由 UI 上下文数据驱动；皮肤配置通过 `UiContext.skin` 可选注入。

## Dependencies

- 依赖 `macroquad` 进行基础绘制与输入读取
- 被 `app` 与 `screens` 广泛消费

## Invariants

- `UiContext` 是页面层共享上下文，避免各页面重复维护输入状态。
- 组件尽量保持纯视图与轻状态，页面负责业务编排。
- 页面不应再内联实现通用控件（阶段29后 `slider/toggle/tab/scroll` 统一在 `ui` 层）。

## FailureModes

- 上下文未按帧更新导致点击/悬停判定滞后。
- 组件样式与交互状态不一致导致 UI 反馈异常。

## WhenToReadSource

- 需要新增通用 UI 组件或调整主题系统时。
- 需要排查特定页面交互命中问题时。

## RelatedDocs

- [host 总览](../host.md)
- [screens 摘要](screens.md)

## LastVerified

2026-03-06

## Owner

Ring-rs 维护者
