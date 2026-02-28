# host 模块摘要

## Purpose

`host` 是视觉小说引擎的宿主层：负责窗口与渲染、输入采集、音频播放、资源加载，以及把 `vn-runtime` 输出的 `Command` 落地为实际画面与交互效果。

## PublicSurface

- 对外入口：`host/src/lib.rs`
- 关键公开类型：`AppState`、`Renderer`、`CommandExecutor`、`ResourceManager`、`AudioManager`、`InputManager`、`SaveManager`
- 关键公开能力：主循环更新/绘制、命令执行、资源/音频/存档管理

## KeyFlow

1. `AppState::new` 组装核心子系统（渲染、资源、音频、执行器、UI、会话状态）。
2. 每帧 `app::update` 拉取输入、推进 `vn-runtime`、消费命令并驱动动画/过渡。
3. `command_executor` 将 `Command` 转换为 `RenderState` 变更与外部副作用请求。
4. `app/command_handlers` 消费副作用请求，驱动音频与效果应用器。
5. `app::draw` 通过 `renderer` 把 `RenderState` 渲染到屏幕。

## Submodules

- [app](host/app.md)：应用层组装、生命周期、主循环胶水
- [app_update](host/app_update.md)：每帧更新与模式分发
- [app_command_handlers](host/app_command_handlers.md)：命令副作用处理
- [command_executor](host/command_executor.md)：`Command` 执行与状态变更
- [renderer](host/renderer.md)：渲染主系统与渲染管线
- [renderer_render_state](host/renderer_render_state.md)：渲染状态模型
- [renderer_animation](host/renderer_animation.md)：通用动画系统
- [renderer_effects](host/renderer_effects.md)：统一效果解析与请求
- [renderer_scene_transition](host/renderer_scene_transition.md)：场景切换多阶段过渡
- [resources](host/resources.md)：资源加载、缓存与路径解析
- [audio](host/audio.md)：BGM/SFX 播放与淡入淡出
- [input](host/input.md)：输入采集与 RuntimeInput 转换
- [ui](host/ui.md)：UI 组件与 UI 上下文
- [screens](host/screens.md)：页面层（Title/Settings/SaveLoad/History/Menu）
- [config](host/config.md)：运行配置加载、默认值与校验
- [manifest](host/manifest.md)：立绘元数据与站位配置
- [save_manager](host/save_manager.md)：槽位存档与 Continue 存档

## Invariants

- `host` 不实现脚本语义；脚本执行与等待模型由 `vn-runtime` 负责。
- Runtime 与 Host 的交互保持在 `Command` / `RuntimeInput` 边界。
- 渲染与资源访问以 `RenderState` 和 `ResourceManager` 为中心，避免跨层直接耦合。

## FailureModes

- 资源路径错误或资源缺失，导致纹理/音频加载失败。
- `Command` 执行结果与当前渲染状态不一致，导致等待或展示异常。
- 配置不合法（入口脚本、资源来源、音量范围）导致启动失败或降级。

## WhenToReadSource

- 需要定位某个 `Command` 在 Host 侧的落地路径时。
- 需要排查渲染、音频、输入在特定模式下的边界行为时。
- 需要修改主循环阶段顺序或新增子系统接入点时。

## RelatedDocs

- [摘要索引](../summary_index.md)
- [仓库导航地图](../navigation_map.md)
- [配置说明](../config_guide.md)
- [资源系统文档](../resource_management.md)
- [存档格式](../save_format.md)
- [vn-runtime 模块总览](../module_summaries/vn-runtime.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
