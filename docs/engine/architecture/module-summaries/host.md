# host 模块摘要

## Purpose

`host` 是视觉小说引擎的宿主层（winit + wgpu + egui）：负责窗口与渲染、输入采集、音频播放、资源加载，以及把 `vn-runtime` 输出的 `Command` 落地为实际画面与交互效果。除窗口模式外，也提供 `headless` 回放入口用于无窗口自动化调试。

## PublicSurface

- 对外入口：`host/src/lib.rs`
- 关键公开类型：`AppState`、`Renderer`、`CommandExecutor`、`ResourceManager`、`AudioManager`、`InputManager`、`SaveManager`
- 关键公开能力：主循环更新/绘制、命令执行、资源/音频/存档管理

## KeyFlow

1. `AppState::new` 组装核心子系统（渲染、资源、音频、执行器、UI、会话状态）。
2. 每帧 `app::update` 拉取输入、推进 `vn-runtime`、消费命令并驱动动画/过渡。
3. `command_executor` 将 `Command` 转换为 `RenderState` 变更与外部副作用请求。
4. `app/command_handlers` 消费副作用请求，驱动音频与效果应用器。
5. 窗口模式由 `main.rs` + `host_app` 通过 `backend::WgpuBackend` 渲染 sprite 命令 + egui UI 到屏幕；headless 模式由 `headless.rs` 复用同一套 update/UI 逻辑但跳过 GPU。

## Submodules

- [app](host/app.md)：应用层组装、生命周期、主循环胶水
- [app_update](host/app-update.md)：每帧更新与模式分发
- [app_command_handlers](host/app-command-handlers.md)：命令副作用处理
- [extensions](host/extensions.md)：扩展 API 与 capability 注册表
- [command_executor](host/command-executor.md)：`Command` 执行与状态变更
- [rendering_types](host/rendering-types.md)：渲染抽象层（Texture/DrawCommand/TextureContext）
- [renderer](host/renderer.md)：渲染主系统与渲染管线
- [renderer_render_state](host/renderer-render-state.md)：渲染状态模型
- [renderer_animation](host/renderer-animation.md)：通用动画系统
- [renderer_effects](host/renderer-effects.md)：统一效果解析与请求
- [renderer_scene_transition](host/renderer-scene-transition.md)：场景切换多阶段过渡
- [resources](host/resources.md)：资源加载、缓存与路径解析
- [audio](host/audio.md)：BGM/SFX 播放与淡入淡出
- [video](host/video.md)：Cutscene 视频播放（RFC-009）
- [game_mode](host/game-mode.md)：小游戏模式（HTTP Bridge + WebView 嵌入）
- [input](host/input.md)：winit 事件驱动的输入采集与 RuntimeInput 转换
- [ui](host/ui.md)：主题、Toast、UI 上下文（布局/素材；页面构建见 `egui_screens/`，由 `host_app`/`main` 驱动渲染）
- [backend](host/backend.md)：winit + wgpu + egui 渲染后端（SpriteRenderer / DissolveRenderer / GpuTexture）
- [config](host/config.md)：运行配置加载、默认值与校验
- [manifest](host/manifest.md)：立绘元数据与站位配置
- [save_manager](host/save-manager.md)：槽位存档与 Continue 存档
- [host_app](host/host-app.md)：winit ApplicationHandler、窗口生命周期与帧驱动
- [egui_actions](host/egui-actions.md)：EguiAction 枚举与 UI 动作分发
- [egui_screens](host/egui-screens.md)：egui 页面构建（title/ingame/settings/save_load/history 等）

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

- [摘要索引](../../../maintenance/summary-index.md)
- [仓库导航地图](../navigation-map.md)
- [配置说明](../../../authoring/config.md)
- [资源系统文档](../../../authoring/resources.md)
- [存档格式](../../reference/save-format.md)
- [vn-runtime 模块总览](vn-runtime.md)

## LastVerified

2026-03-23

## Owner

claude-4.6-opus