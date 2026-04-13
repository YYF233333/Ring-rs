# 仓库导航地图（Navigation Map）

> 目标：让"人/模型"在**最少阅读**的前提下，快速定位"该改哪个 crate / 模块 / 文件"。
> 本文是**人工维护**的索引（比自动目录树更有用）。当你重构模块边界时，请顺手更新这里。

## 顶层概览（Workspace）

- **`vn-runtime/`**：纯逻辑 Runtime（脚本解析/执行/状态/存档），**不依赖引擎与 IO**。
- **`host-dioxus/`**：Dioxus Desktop 宿主。Rust 全栈（RSX 声明式 UI），无 IPC 边界。默认 `cargo run` 执行此 crate。
- **`tools/xtask/`**：本地自检与 CI 共用的质量门禁、覆盖率和开发辅助命令入口。
- **`tools/asset-packer/`**：资源打包工具（可选工作流）。
- **`assets/`**：游戏资源（背景/立绘/脚本/音频/字体/manifest）。
- **`docs/`**：规范与设计文档（脚本语法、资源管理、存档格式等）。

## 重要文档（建议阅读顺序）

- **架构硬约束**：[ARCH.md](../../../ARCH.md)（Runtime/Host 分离、显式状态、确定性、Command 驱动）
- **RFC 计划索引**：[RFC 索引](../../../RFCs/README.md)
- **内容制作入门**：[Getting Started](../../authoring/getting-started.md)（不改代码写脚本/素材 → 测试 → 打包发布）
- **运行配置说明**：[config 配置说明](../../authoring/config.md)（`config.json` 字段含义/默认值/校验规则）
- **脚本语法规范**：[脚本语法规范](../../authoring/script-syntax.md)
- **资源系统**：[资源系统与打包](../../authoring/resources.md)、[manifest 指南](../../authoring/manifest.md)
- **存档格式**：[save format](../reference/save-format.md)
- **覆盖率与门禁**：[coverage.md](../../testing/coverage.md)、[CONTRIBUTING.md](../../../CONTRIBUTING.md)

## `vn-runtime/`：从"脚本"到"Command"的链路

### 入口与核心文件

- **Command 定义（Runtime → Host）**：`vn-runtime/src/command/mod.rs`
- **输入模型（Host → Runtime）**：`vn-runtime/src/input.rs`
- **显式状态/等待模型**：`vn-runtime/src/state.rs`
- **引擎循环（tick/handle_input/restore）**：`vn-runtime/src/runtime/engine/mod.rs`
- **执行器（AST → Command）**：`vn-runtime/src/runtime/executor/mod.rs`
- **脚本 AST**：`vn-runtime/src/script/ast/mod.rs`
- **脚本解析器**：`vn-runtime/src/script/parser/mod.rs`
- **阶段 2 解析（块 → ScriptNode）**：`vn-runtime/src/script/parser/phase2/`（`mod.rs` 分发；`display.rs` / `control.rs` / `dialogue.rs` / `misc.rs` 按域拆分）
- **内联标签解析（节奏标签）**：`vn-runtime/src/script/parser/inline_tags.rs`
- **脚本诊断（静态分析）**：`vn-runtime/src/diagnostic/mod.rs`
- **存档模型**：`vn-runtime/src/save.rs`
- **历史记录**：`vn-runtime/src/history.rs`

### 常见改动：我应该改哪里？

- **新增脚本语法（解析层）**：`vn-runtime/src/script/parser/mod.rs` → `vn-runtime/src/script/ast/mod.rs`
- **新增/修改内联标签（节奏标签）**：`vn-runtime/src/script/parser/inline_tags.rs`
- **把 AST 变成命令（语义层）**：`runtime/executor/mod.rs`
- **新增/修改命令类型（通信契约）**：`command/mod.rs`（同时要改 `host-dioxus/` 的执行端）
- **调整运行时状态/等待机制**：`state.rs`、`runtime/engine/mod.rs`
- **存档兼容**：`save.rs` + [save-format.md](../reference/save-format.md)

## `host-dioxus/`：把 "Command" 变成"画面/音频/UI"（Dioxus Desktop）

> Rust 全栈（RSX 声明式 UI），无 IPC 边界。默认 `cargo run` 执行此 crate。

### 目录结构

```
host-dioxus/src/
├── main.rs              # 入口 + App 根组件 + tick loop + 全局 CSS（1920×1080 基准）+ 资源协议
├── state.rs             # AppStateInner（tick/click/choose/save/load + execute_action + condition_context）
├── command_executor.rs  # 23 个 Command handler → RenderState 更新
├── render_state.rs      # RenderState：后端→前端的数据契约
├── screen_defs.rs       # 数据驱动 UI：从 screens.json 加载按钮/条件/动作定义
├── layout_config.rs     # 布局配置：从 layout.json 加载字号/颜色/尺寸/资产路径
├── audio.rs             # AudioManager（headless 状态跟踪）
├── config.rs            # 配置加载与校验
├── manifest.rs          # 角色 manifest 解析
├── resources.rs         # ResourceManager（FS/ZIP 透明访问）
├── save_manager.rs      # 存读档管理
├── init.rs              # 后端初始化（含 layout + screen_defs 加载）
├── error.rs             # 错误类型
├── headless_cli.rs      # 无头测试 harness
├── vn/                  # VN 渲染层（RSX 组件）
│   ├── scene.rs         # VNScene 容器（shake/blur/dim + skip-mode）
│   ├── background.rs    # 背景双层交叉淡化
│   ├── character.rs     # 立绘（CSS transition 驱动位置/透明度）
│   ├── dialogue.rs      # ADV 对话框（NinePatch 背景 + 打字机效果）
│   ├── nvl.rs           # NVL 全屏文本
│   ├── choice.rs        # 选项面板（NinePatch 背景）
│   ├── transition.rs    # Fade/FadeWhite CSS 遮罩过渡
│   ├── rule_transition.rs # WebGL shader 遮罩过渡
│   ├── chapter_mark.rs  # 章节标记
│   ├── title_card.rs    # 全屏字卡
│   ├── video.rs         # HTML5 视频 cutscene
│   ├── quick_menu.rs    # 底部快捷菜单（数据驱动）
│   └── audio_bridge.rs  # JS Web Audio API 桥接
├── screens/             # 系统 UI 页面（RSX 组件，数据驱动）
│   ├── title.rs         # 标题画面（条件背景+条件按钮）
│   ├── in_game_menu.rs  # 游内暂停菜单
│   ├── save_load.rs     # 存读档（GameMenuFrame + Tab + A/Q/1-9 分页）
│   ├── settings.rs      # 设置（GameMenuFrame + 静音 + 应用按钮）
│   └── history.rs       # 对话历史（GameMenuFrame + 双列布局）
└── components/          # 通用 UI 组件
    ├── skip_indicator.rs    # SKIP/AUTO 模式指示器
    ├── confirm_dialog.rs    # 模态确认弹窗
    ├── game_menu_frame.rs   # 游戏菜单通用框架（左导航+右内容）
    └── toast.rs             # Toast 提示（4 种类型 + 自动淡出）
```

### 关键数据流

```
AppStateInner ──tick loop 30fps──→ clone RenderState ──Signal──→ RSX 组件
     ↑                                                              │
     └───── onclick/onkeydown ── lock Mutex ── process_click() ─────┘
```

## 开发工作流（质量门禁/覆盖率）

- **一键门禁**：`cargo check-all`（本地自检与 CI 共用；由 `tools/xtask` 串行执行 fmt → clippy --fix → test）
- **构建工具链**：纯 `cargo`，不再需要 Node.js / pnpm / biome / vue-tsc
- **脚本检查**：`cargo script-check`（检查脚本语法/label/资源引用）
- **Dev Mode 自动脚本检查**：Host 启动时基于 `config.json` 的 `debug.script_check` 自动运行（debug build 默认开启）
- **覆盖率**：`cargo cov`，报告：`target/llvm-cov/html/index.html`

## "不要读/不要改"的目录（常见噪音）

- **构建产物**：`target/`（巨大、与定位问题无关）
- **分发产物**：`dist/`、根目录的 `*.zip`（通常由打包流程生成）
- **本地存档**：`saves/`（调试用数据，不是代码）

## 当你想做 X（快速索引）

- **想加/改脚本语法** → [脚本语法规范](../../authoring/script-syntax.md) + `vn-runtime/src/script/*`
- **想加一个新 Command** → `vn-runtime/src/command/mod.rs` + `host-dioxus/src/command_executor.rs`
- **想改 UI 页面** → `host-dioxus/src/screens/` + `host-dioxus/src/components/`
- **想改资源路径解析/打包/缓存** → `host-dioxus/src/resources.rs` + [资源系统与打包](../../authoring/resources.md)
- **想改存档/兼容** → `vn-runtime/src/save.rs` + `host-dioxus/src/save_manager.rs` + [save format](../reference/save-format.md)
