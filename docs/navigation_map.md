# 仓库导航地图（Navigation Map）

> 目标：让“人/模型”在**最少阅读**的前提下，快速定位“该改哪个 crate / 模块 / 文件”。
> 本文是**人工维护**的索引（比自动目录树更有用）。当你重构模块边界时，请顺手更新这里。

## 顶层概览（Workspace）

- **`vn-runtime/`**：纯逻辑 Runtime（脚本解析/执行/状态/存档），**不依赖引擎与 IO**。
- **`host/`**：macroquad 宿主（渲染/音频/输入/资源），把 Runtime 的 `Command` 转换为实际效果。
- **`tools/xtask/`**：本地质量门禁与开发辅助命令（跨平台串行执行）。
- **`tools/asset-packer/`**：资源打包工具（可选工作流）。
- **`assets/`**：游戏资源（背景/立绘/脚本/音频/字体/manifest）。
- **`docs/`**：规范与设计文档（脚本语法、资源管理、存档格式等）。

## 重要文档（建议阅读顺序）

- **架构硬约束**：[PLAN.md](../PLAN.md)（Runtime/Host 分离、显式状态、确定性、Command 驱动）
- **开发路线图**：[ROADMAP.md](../ROADMAP.md)
- **内容制作入门**：[getting_started.md](getting_started.md)（不改代码写脚本/素材 → 测试 → 打包发布）
- **运行配置说明**：[config_guide.md](config_guide.md)（`config.json` 字段含义/默认值/校验规则）
- **脚本语法规范**：[script_syntax_spec.md](script_syntax_spec.md)
- **脚本示例集**：[script_language_showcase.md](script_language_showcase.md)
- **资源系统**：[resource_management.md](resource_management.md)、[manifest_guide.md](manifest_guide.md)
- **存档格式**：[save_format.md](save_format.md)
- **覆盖率与门禁**：[coverage.md](coverage.md)、[CONTRIBUTING.md](../CONTRIBUTING.md)

## `vn-runtime/`：从“脚本”到“Command”的链路

### 入口与核心文件

- **Command 定义（Runtime → Host）**：`vn-runtime/src/command.rs`
- **输入模型（Host → Runtime）**：`vn-runtime/src/input.rs`
- **显式状态/等待模型**：`vn-runtime/src/state.rs`
- **引擎循环（tick/handle_input/restore）**：`vn-runtime/src/runtime/engine.rs`
- **执行器（AST → Command）**：`vn-runtime/src/runtime/executor.rs`
- **脚本 AST**：`vn-runtime/src/script/ast.rs`
- **脚本解析器**：`vn-runtime/src/script/parser.rs`
- **脚本诊断（静态分析）**：`vn-runtime/src/diagnostic.rs`
- **存档模型**：`vn-runtime/src/save.rs`
- **历史记录**：`vn-runtime/src/history.rs`

### 常见改动：我应该改哪里？

- **新增脚本语法（解析层）**：`script/parser.rs` → `script/ast.rs`
- **把 AST 变成命令（语义层）**：`runtime/executor.rs`
- **新增/修改命令类型（通信契约）**：`command.rs`（同时要改 `host/` 的执行端）
- **调整运行时状态/等待机制**：`state.rs`、`runtime/engine.rs`
- **存档兼容**：`save.rs` + [save_format.md](save_format.md)

## `host/`：把 `Command` 变成“画面/音频/UI”

### 应用层（App：生命周期/主循环胶水）

- **入口（尽量薄）**：`host/src/main.rs`
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

### 渲染/资源/音频/UI/屏幕

- **渲染系统**：`host/src/renderer/`
  - **统一效果解析与请求**：`host/src/renderer/effects/`（EffectKind、ResolvedEffect、resolve()、EffectRequest、EffectTarget）
  - **动画系统**：`host/src/renderer/animation/`（AnimationSystem、Animatable trait）
- **资源管理**：`host/src/resources/`（路径、来源、缓存、错误）
- **音频系统**：`host/src/audio/`
- **屏幕（UI 页面）**：`host/src/screens/`（title/settings/save_load/history…）
- **UI 组件**：`host/src/ui/`（button/list/modal/panel/theme/toast）
- **输入**：`host/src/input/`
- **配置/manifest/save manager**：`host/src/config/`、`host/src/manifest/`、`host/src/save_manager/`

### 常见改动：推进模式 / Skip / Auto（阶段 26）

- **推进模式状态**：`host/src/app_mode.rs`（`PlaybackMode::{Normal,Auto,Skip}`；UserSettings 的 `auto_delay/auto_mode`）
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
- **想改 UI 页面** → `host/src/screens/*` + `host/src/ui/*`
- **想改资源路径解析/打包/缓存** → `host/src/resources/*` + [resource_management.md](resource_management.md)
- **想改存档/兼容** → `vn-runtime/src/save.rs` + `host/src/app/save.rs` + [save_format.md](save_format.md)

