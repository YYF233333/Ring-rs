# host 模块摘要

## Purpose

`host` 是宿主层：负责窗口/渲染、输入、音频、UI 与资源访问，并把 `vn-runtime` 产出的 `Command` 落地为实际画面和交互。窗口模式与 `headless` 模式共用同一套应用更新链路。

## PublicSurface

- 入口：`host/src/lib.rs`
- 关键公开类型：`AppState`、`Renderer`、`CommandExecutor`、`ResourceManager`、`AudioManager`、`InputManager`、`UiModeRegistry`、`SaveManager`

## KeyFlow

1. `AppState::new` 组装 core/ui/session 三组状态，并注册内建扩展与 UI mode。
2. `app::update` 按当前模式推进输入、Runtime、过渡、视频与音频。
3. `command_executor` 负责 `Command -> RenderState 变更 + 副作用输出`。
4. `app/command_handlers` 消费副作用输出，驱动音频与效果 capability。
5. 窗口模式由 `host_app` + `build_ui` 驱动渲染与 egui；`headless` 复用同一套 update/UI 编排但跳过 GPU。

## Submodules

- [app](host/app.md)：应用状态组装与主循环胶水
- [app_update](host/app-update.md)：每帧更新与模式分发
- [app_command_handlers](host/app-command-handlers.md)：音频与效果副作用处理
- [extensions](host/extensions.md)：效果 capability 注册与调度
- [command_executor](host/command-executor.md)：`Command` 执行与状态变更
- [rendering_types](host/rendering-types.md)：渲染抽象层（Texture/DrawCommand/TextureContext）
- [renderer](host/renderer.md)：渲染主系统与渲染管线
- [renderer_render_state](host/renderer-render-state.md)：渲染状态模型
- [renderer_animation](host/renderer-animation.md)：通用动画系统
- [renderer_effects](host/renderer-effects.md)：统一效果解析与请求
- [renderer_scene_transition](host/renderer-scene-transition.md)：场景切换多阶段过渡
- [resources](host/resources.md)：资源加载、缓存与路径解析
- [audio](host/audio.md)：音频状态与播放控制
- [video](host/video.md)：cutscene 视频播放
- [game_mode](host/game-mode.md)：小游戏 HTTP Bridge 与 WebView 生命周期
- [input](host/input.md)：输入采集与 `RuntimeInput` 转换
- [ui](host/ui.md)：UI 配置、素材缓存与渲染上下文
- [backend](host/backend.md)：winit + wgpu + egui 后端
- [config](host/config.md)：运行配置加载、默认值与校验
- [manifest](host/manifest.md)：立绘元数据与站位配置
- [save_manager](host/save-manager.md)：槽位存档与 Continue 存档
- [host_app](host/host-app.md)：窗口生命周期与帧驱动
- [egui_actions](host/egui-actions.md)：UI 动作枚举与应用层处理
- [egui_screens](host/egui-screens.md)：各页面的 egui 构建函数

## Invariants

- `host` 不实现脚本语义，只消费 `Command` / `RuntimeInput` 边界。
- `command_executor` 不直接操作外部系统；音频和效果副作用交给 `app/command_handlers`。
- 窗口模式与 `headless` 模式应尽量共享 update/UI 逻辑，避免行为分叉。

## WhenToReadSource

- 需要定位某个 `Command` 的 Host 落地路径时。
- 需要调整主循环阶段顺序、模式切换或 UI/小游戏接入点时。
- 需要排查窗口模式与 `headless` 行为是否一致时。

## RelatedDocs

- [摘要索引](../../../maintenance/summary-index.md)
- [仓库导航地图](../navigation-map.md)
- [配置说明](../../../authoring/config.md)
- [资源系统文档](../../../authoring/resources.md)
- [存档格式](../../reference/save-format.md)
- [vn-runtime 模块总览](vn-runtime.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4