# 仓库导航地图（Navigation Map）

> 目标：让“人/模型”在**最少阅读**的前提下，快速定位“该改哪个 crate / 模块 / 文件”。
> 本文是**人工维护**的索引（比自动目录树更有用）。当你重构模块边界时，请顺手更新这里。

## 顶层概览（Workspace）

- **`vn-runtime/`**：纯逻辑 Runtime（脚本解析/执行/状态/存档），**不依赖引擎与 IO**。
- **`host/`**：winit + wgpu + egui 宿主（渲染/音频/输入/资源），把 Runtime 的 `Command` 转换为实际效果。
- **`tools/xtask/`**：本地质量门禁与开发辅助命令（跨平台串行执行）。
- **`tools/asset-packer/`**：资源打包工具（可选工作流）。
- **`assets/`**：游戏资源（背景/立绘/脚本/音频/字体/manifest）。
- **`docs/`**：规范与设计文档（脚本语法、资源管理、存档格式等）。

## 重要文档（建议阅读顺序）

- **摘要入口（vn-runtime + host）**：[summary_index.md](summary_index.md)
- **架构硬约束**：[ARCH.md](../ARCH.md)（Runtime/Host 分离、显式状态、确定性、Command 驱动）
- **RFC 计划索引**：[../RFCs/README.md](../RFCs/README.md)
- **内容制作入门**：[getting_started.md](getting_started.md)（不改代码写脚本/素材 → 测试 → 打包发布）
- **运行配置说明**：[config_guide.md](config_guide.md)（`config.json` 字段含义/默认值/校验规则）
- **脚本语法规范**：[script_syntax_spec.md](script_syntax_spec.md)
- **脚本示例集**：[script_language_showcase.md](script_language_showcase.md)
- **资源系统**：[resource_management.md](resource_management.md)、[manifest_guide.md](manifest_guide.md)
- **存档格式**：[save_format.md](save_format.md)
- **覆盖率与门禁**：[coverage.md](coverage.md)、[CONTRIBUTING.md](../CONTRIBUTING.md)

## `vn-runtime/`：从“脚本”到“Command”的链路

### 摘要优先（先读这些）

- 模块总览：[module_summaries/vn-runtime.md](module_summaries/vn-runtime.md)
- 子模块：[script](module_summaries/vn-runtime/script.md)
- 子模块：[runtime](module_summaries/vn-runtime/runtime.md)
- 子模块：[command](module_summaries/vn-runtime/command.md)
- 子模块：[diagnostic](module_summaries/vn-runtime/diagnostic.md)
- 子模块：[parser](module_summaries/vn-runtime/parser.md)

### 入口与核心文件

- **Command 定义（Runtime → Host）**：`vn-runtime/src/command.rs`
- **输入模型（Host → Runtime）**：`vn-runtime/src/input.rs`
- **显式状态/等待模型**：`vn-runtime/src/state.rs`
- **引擎循环（tick/handle_input/restore）**：`vn-runtime/src/runtime/engine.rs`
- **执行器（AST → Command）**：`vn-runtime/src/runtime/executor.rs`
- **脚本 AST**：`vn-runtime/src/script/ast.rs`
- **脚本解析器**：`vn-runtime/src/script/parser/mod.rs`
- **内联标签解析（节奏标签）**：`vn-runtime/src/script/parser/inline_tags.rs`
- **脚本诊断（静态分析）**：`vn-runtime/src/diagnostic.rs`
- **存档模型**：`vn-runtime/src/save.rs`
- **历史记录**：`vn-runtime/src/history.rs`

### 常见改动：我应该改哪里？

- **新增脚本语法（解析层）**：`vn-runtime/src/script/parser/mod.rs` → `vn-runtime/src/script/ast.rs`
- **新增/修改内联标签（节奏标签）**：`vn-runtime/src/script/parser/inline_tags.rs`
- **把 AST 变成命令（语义层）**：`runtime/executor.rs`
- **新增/修改命令类型（通信契约）**：`command.rs`（同时要改 `host/` 的执行端）
- **调整运行时状态/等待机制**：`state.rs`、`runtime/engine.rs`
- **存档兼容**：`save.rs` + [save_format.md](save_format.md)

## `host/`：把 `Command` 变成“画面/音频/UI”

### 摘要优先（先读这些）

- 模块总览：[module_summaries/host.md](module_summaries/host.md)
- 子模块：[app](module_summaries/host/app.md)
- 子模块：[app_update](module_summaries/host/app_update.md)
- 子模块：[app_command_handlers](module_summaries/host/app_command_handlers.md)
- 子模块：[extensions](module_summaries/host/extensions.md)
- 子模块：[command_executor](module_summaries/host/command_executor.md)
- 子模块：[renderer](module_summaries/host/renderer.md)
- 子模块：[renderer_render_state](module_summaries/host/renderer_render_state.md)
- 子模块：[renderer_animation](module_summaries/host/renderer_animation.md)
- 子模块：[renderer_effects](module_summaries/host/renderer_effects.md)
- 子模块：[renderer_scene_transition](module_summaries/host/renderer_scene_transition.md)
- 子模块：[resources](module_summaries/host/resources.md)
- 子模块：[audio](module_summaries/host/audio.md)
- 子模块：[input](module_summaries/host/input.md)
- 子模块：[ui](module_summaries/host/ui.md)
- 子模块：[backend](module_summaries/host/backend.md)
- 子模块：[config](module_summaries/host/config.md)
- 子模块：[manifest](module_summaries/host/manifest.md)
- 子模块：[save_manager](module_summaries/host/save_manager.md)
- 子模块：[host_app](module_summaries/host/host_app.md)
- 子模块：[egui_actions](module_summaries/host/egui_actions.md)
- 子模块：[egui_screens](module_summaries/host/egui_screens.md)

### 应用层（App：生命周期/主循环胶水）

- **入口（尽量薄）**：`host/src/main.rs`（仅 `main()` + 配置 + EventLoop 启动）
- **HostApp（ApplicationHandler）**：`host/src/host_app.rs`（窗口生命周期、事件分发）
- **EguiAction（UI 动作枚举）**：`host/src/egui_actions.rs`
- **egui 页面构建**：`host/src/egui_screens/`（title/ingame/settings/save_load/history/toast/helpers）
- **AppState 与组装**：`host/src/app/mod.rs`
- **启动引导（资源预加载/按需加载扫描）**：`host/src/app/bootstrap.rs`
- **初始化拆分（资源/音频/manifest/脚本/设置等）**：`host/src/app/init.rs`
- **每帧更新（已模块化）**：`host/src/app/update/`
  - `host/src/app/update/mod.rs`：聚合入口 `update(app_state)`
  - `host/src/app/update/modes.rs`：按 `AppMode` 分发（Title/Menu/Settings/History…）
  - `host/src/app/update/script.rs`：脚本输入 + runtime tick + 命令执行链路；阶段26新增 `skip_all_active_effects()`（Skip 模式收敛入口）
  - `host/src/app/update/scene_transition.rs`：场景过渡驱动
- **绘制**：`host/src/app/draw.rs`
- **存档操作（quick save/load 等）**：`host/src/app/save.rs`
- **脚本加载与扫描**：`host/src/app/script_loader.rs`
- **命令侧的“外部系统处理器”**：`host/src/app/command_handlers/`（音频/转场/角色动画等）

### 执行层（CommandExecutor：Command → RenderState + 外部输出事件）

- **核心执行器**：`host/src/command_executor/mod.rs`
- **执行器类型（输出事件/命令载荷）**：`host/src/command_executor/types.rs`
- **UI 命令执行（TextBox/ChapterMark/ClearCharacters）**：`host/src/command_executor/ui.rs`
- **背景/场景命令执行**：`host/src/command_executor/background.rs`

> 直觉对齐：
> - `command_executor` 更偏“把 Command 翻译成**状态变更 + 需要外部系统执行的输出**”
> - `app/command_handlers` 更偏“消费输出，驱动**音频/过渡/动画系统**做事”

### 渲染抽象层 / 后端 / 资源 / 音频 / UI

- **渲染抽象层（RFC-008）**：`host/src/rendering_types.rs`
  - **Texture trait**：纹理抽象接口（`width/height/size_bytes/as_any`）
  - **TextureFactory trait**：纹理创建工厂接口
  - **TextureContext**：持有 `Arc<dyn TextureFactory>`，注入到 ResourceManager
  - **DrawCommand**：绘制命令（Sprite/Rect/Dissolve），使用 `Arc<dyn Texture>`
  - **NullTexture / NullTextureFactory**：headless 后端，用于无 GPU 环境的单元测试
- **GPU 后端（winit + wgpu + egui）**：`host/src/backend/`
  - **WgpuBackend**：渲染后端门面，编排帧渲染流程
  - **WgpuTextureFactory**：`TextureFactory` 的 wgpu 实现（内部类型）
  - **GpuContext**：GPU 设备/队列/表面管理
  - **EguiIntegration**：egui 输入/输出/渲染桥接
  - **SpriteRenderer**：2D textured quad batch 渲染器（WGSL shader，通过 downcast 访问 GpuTexture）
  - **DissolveRenderer**：mask-based dissolve 效果渲染器（WGSL shader，通过 downcast 访问 GpuTexture）
  - **GpuTexture**：wgpu 纹理封装，实现 `Texture` trait
  - **math**：公共渲染工具（QuadVertex、orthographic_projection、quad_vertices）
- **渲染逻辑**：`host/src/renderer/`
  - **Renderer struct + 顶层编排**：`host/src/renderer/mod.rs`
  - **绘制命令生成**：`host/src/renderer/draw_commands.rs`（背景/角色/场景遮罩 -> DrawCommand）
  - **场景效果与过渡**：`host/src/renderer/scene_effects.rs`（shake/blur/dissolve/fade 过渡）
  - **统一效果解析与请求**：`host/src/renderer/effects/`（EffectKind、ResolvedEffect、resolve()、EffectRequest、EffectTarget）
  - **动画系统**：`host/src/renderer/animation/`（AnimationSystem、Animatable trait）
- **资源管理**：`host/src/resources/`（路径、来源、缓存、TextureContext）
- **音频系统**：`host/src/audio/`（mod.rs: 结构/音量/duck; playback.rs: BGM/SFX 播放与淡入淡出）
- **UI 基础设施**：`host/src/ui/`（theme/toast/skin）
- **输入（winit 事件驱动）**：`host/src/input/`
- **配置/manifest/save manager**：`host/src/config/`、`host/src/manifest/`、`host/src/save_manager/`

### 常见改动：节奏标签 / 打字机行为

- **内联标签数据模型**：`vn-runtime/src/command/mod.rs`（`InlineEffect`、`InlineEffectKind`）
- **打字机 inline_wait / effective_cps / no_wait**：`host/src/renderer/render_state/mod.rs`（`DialogueState` 扩展字段、`advance_typewriter`、`extend_dialogue`）
- **节奏标签帧更新**：`host/src/app/update/modes.rs`（inline_wait 定时器、effective_cps 倍率、no_wait 自动推进）
- **点击 inline_wait 跳过**：`host/src/app/update/script.rs`（`is_inline_click_wait` 判定分支）

### 常见改动：推进模式 / Skip / Auto（阶段 26）

- **推进模式状态**：`host/src/app/app_mode.rs`（`PlaybackMode::{Normal,Auto,Skip}`；UserSettings 的 `auto_delay`；Auto 开关不持久化）
- **推进控制主循环**：`host/src/app/update/modes.rs`（Ctrl 按住临时 Skip；Auto 的节拍与推进条件）
- **统一跳过入口（收敛语义）**：`host/src/app/update/script.rs::skip_all_active_effects()`（动画/changeBG/changeScene/打字机）
- **changeScene 完整跳过（不丢背景）**：
  - `host/src/renderer/scene_transition.rs::SceneTransitionManager::skip_to_end()`
  - `host/src/renderer/mod.rs::Renderer::skip_scene_transition_to_end()`

## 开发工作流（质量门禁/覆盖率）

- **一键门禁**：`cargo check-all`（由 `tools/xtask` 串行执行 fmt/clippy/test）
- **脚本检查**：`cargo script-check`（检查脚本语法/label/资源引用）
- **Dev Mode 自动脚本检查**：Host 启动时基于 `config.json` 的 `debug.script_check` 自动运行（debug build 默认开启）
- **覆盖率**：
  - `cargo cov-runtime`（主看 `vn-runtime`）
  - `cargo cov-workspace`（趋势观察）
  - 报告：`target/llvm-cov/html/index.html`

## “不要读/不要改”的目录（常见噪音）

- **构建产物**：`target/`（巨大、与定位问题无关）
- **分发产物**：`dist/`、根目录的 `*.zip`（通常由打包流程生成）
- **本地存档**：`saves/`（调试用数据，不是代码）

## 当你想做 X（快速索引）

- **想加/改脚本语法** → [script_syntax_spec.md](script_syntax_spec.md) + `vn-runtime/src/script/*`
- **想加一个新 Command** → `vn-runtime/src/command.rs` + `host/src/command_executor/*`
- **想改 UI 页面** → `host/src/egui_screens/`（各页面 UI 构建）+ `host/src/ui/*`（主题/Toast）
- **想改资源路径解析/打包/缓存** → `host/src/resources/*` + [resource_management.md](resource_management.md)
- **想改存档/兼容** → `vn-runtime/src/save.rs` + `host/src/app/save.rs` + [save_format.md](save_format.md)

