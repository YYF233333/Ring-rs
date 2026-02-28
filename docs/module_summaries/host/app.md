# host/app 摘要

## Purpose

`app` 负责 Host 应用层编排：初始化子系统、维护 `AppState`、组织每帧更新与绘制、承接存档与脚本加载流程。

## PublicSurface

- 模块入口：`host/src/app/mod.rs`
- 关键类型：`AppState`、`CoreSystems`、`UiSystems`、`GameSession`
- 关键子模块：`bootstrap`、`init`、`draw`、`save`、`script_loader`、`update`、`command_handlers`

## KeyFlow

1. `AppState::new` 创建资源/音频/渲染/执行器并加载 manifest、脚本列表与用户设置。
2. `update` 路径推进输入、Runtime tick、命令执行和过渡/动画系统。
3. `draw` 路径将当前 `RenderState` 交给 `Renderer` 输出画面。
4. `save` 与 `script_loader` 提供会话存档与脚本加载辅助能力。

## Dependencies

- 依赖 `renderer`、`resources`、`audio`、`input`、`screens`、`ui`、`save_manager`
- 依赖 `vn-runtime` 提供脚本执行核心与等待模型

## Invariants

- `AppState` 是 Host 主循环的状态聚合根，子系统职责分层明确（core/ui/session）。
- 脚本语义执行不在 `app` 内实现，只做编排与驱动。

## FailureModes

- 初始化阶段资源或配置加载失败，导致运行时降级或无法启动。
- 子系统状态未按阶段推进，导致 UI/渲染/输入状态不同步。

## WhenToReadSource

- 需要调整主循环阶段或插入新子系统时。
- 需要排查初始化顺序与运行时状态组装问题时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app_update.md)
- [app_command_handlers 摘要](app_command_handlers.md)
- [仓库导航地图](../../navigation_map.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
