# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

你是本项目的 lead engineer。项目目标：构建一个可运行、结构清晰、可扩展的视觉小说引擎（Visual Novel Engine），以最小人类干预迭代，代码须可读、可测、可维护。

## 架构概览

### Runtime/Host 分离（硬约束）

引擎分为纯逻辑核心和宿主两层，通过 `Command`（Runtime→Host）和 `RuntimeInput`（Host→Runtime）通信，禁止其他耦合：

- **`vn-runtime`**：纯逻辑——脚本解析/执行、状态管理、等待建模、产出 `Command`。禁止 IO/渲染/真实时间依赖。所有状态在 `RuntimeState` 中显式建模且可序列化。
- **`host`**（旧宿主）：winit + wgpu + egui 渲染宿主，执行 `Command` 产生画面/音频/UI。禁止脚本逻辑、直接修改 Runtime 内部状态。
- **`host-tauri`**（新宿主）：Tauri 2 + Vue 3 + TypeScript。Rust 后端持有所有游戏状态，前端只渲染。IPC 命令须是 thin proxy（lock→call→return）。`render_state.rs`（Rust）↔ `render-state.ts`（TS）须保持同步。

### 核心执行模型

Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动。确定性执行：相同输入序列产出相同 Command 序列。时间等待由 Host 负责。

### Workspace 成员

```
vn-runtime/          # 纯逻辑 Runtime
host/                # 旧宿主（winit/wgpu/egui）
host-tauri/src-tauri # 新宿主 Rust 后端
host-tauri/          # 新宿主 Vue 前端
tools/xtask/         # 门禁/覆盖率/脚本检查
tools/asset-packer/  # 资源打包
```

默认 `cargo run` 执行 `host-tauri/src-tauri`。

## 常用命令

| 用途 | 命令 |
|------|------|
| 一键门禁（CI 同款） | `cargo check-all`（fmt → clippy --fix → biome + vue-tsc → test） |
| 前端检查 | `cargo fe-check`（仅 biome + vue-tsc） |
| 测试 | `cargo test -p vn-runtime --lib` |
| 单个测试 | `cargo test -p vn-runtime --lib test_name` |
| 覆盖率 | `cargo cov`（报告：`target/llvm-cov/html/index.html`） |
| 符号索引 | `cargo gen-symbols` |
| 脚本静态检查 | `cargo script-check [path]` |
| Tauri 开发 | `cd host-tauri; pnpm tauri dev` |
| 前端格式化+lint | `pnpm -C host-tauri check:write` |
| 前端类型检查 | `pnpm -C host-tauri typecheck` |
| 变异测试 | `cargo mutants` |

## 设计决策框架

### 硬约束

- 核心逻辑须有单元测试；修 bug 须补回归测试。
- Public API 须有文档注释。
- 禁止顺便重构无关代码。
- 方案选择：优先清晰/可读/可测，优先最简单且符合约束。

### 重构哲学

Rust 编译器 + borrow checker 是最终安全网——类型级重构编译通过且测试通过，即可视为正确。

- 优先编译期约束而非运行时检查：用 `enum` 编码状态、newtype 编码领域概念。
- 让非法状态不可表示：enum 变体只携带该状态下有意义的数据，不用 `Option` 字段 + 注释描述合法组合。
- 不因改动文件数多而回避正确方案，但范围扩大须服务于当前任务目标。

### 错误处理：受信 vs 不受信

**"出错是我们的 bug 还是外界的问题？"** 前者 panic/assert，后者 Result。

| 路径 | 场景 | 策略 |
|------|------|------|
| 受信（内部） | 引擎状态、已校验数据、模块间协议 | `unreachable!()`、`debug_assert!`、`expect("invariant: ...")` |
| 不受信（外部） | 脚本解析、宿主参数、资源加载 | `Result<T, E>` 携带上下文，调用方决定报告方式 |

错误类型按失败域划分，不按函数划分。跨域调用包裹内层错误（`ExecutorError::Parse(ParseError)`），不要 `.to_string()` 展平。

### 禁止模式

非测试代码中禁止以下模式，除非附注释说明理由：

| 模式 | 替代 |
|------|------|
| `unwrap()` / `expect()` 处理外部输入 | `?` 或 `map_err` 传播 |
| `let _ = fallible_expr();` 无注释 | 显式处理或注释说明 |
| `println!` / `eprintln!` 调试 | `tracing::debug!` / `log::debug!` |
| 手工 `PathBuf` 拼接资源路径 | `ResourceManager` 方法 + `LogicalPath` |
| 子系统自持 `base_path` 或 `use_zip_mode` 字段 | 通过 `ResourceManager` 统一读取 |
| 直接构造 `FsSource` / `ZipSource`（`init.rs` 之外） | 仅 `init::create_resource_manager` 内部构造 |
| 资源路径使用裸 `&str` / `String` | 使用 `&LogicalPath` |

### 代码风格

- 用 `?` 链保持 happy path 扁平。避免嵌套 `match` 处理 `Result`/`Option`。
- 不提取只用一次的帮助函数。提取门槛：至少两处调用，或逻辑足够独立值得命名。

### 测试价值判断

值得测：状态机转换、脚本解析、命令执行与副作用、错误恢复与边界条件、跨模块集成。

不值得测：derive trait、`serde` 序列化正确性、简单 getter/setter、第三方框架行为。

## 常见改动的多文件同步要求

| 改动 | 须同步的文件 |
|------|-------------|
| 新增/修改 Command | `vn-runtime` command 模块 + `host` command_executor + `host-tauri` command_executor + `render_state.rs` + `render-state.ts` |
| 新增脚本语法 | 先更新语法规范 → parser + AST + executor + round-trip 测试 |
| 新增 UI 页面 | `host` egui_screens + `host-tauri` screens/*.vue |
| 修改 RenderState | `host-tauri` render_state.rs（Rust）↔ render-state.ts（TS）双向同步 |
| Typewriter 节奏标签 | parser inline_tags + Command InlineEffect + host-side consumer |
| 新增/移动 pub 符号 | 完成后运行 `cargo gen-symbols` 刷新符号索引 |

## 仓库导航

| 内容 | 路径 |
|------|------|
| 导航地图（"改哪里"） | `docs/engine/architecture/navigation-map.md` |
| 摘要入口 | `docs/maintenance/summary-index.md` |
| 符号索引 | `docs/engine/symbol-index.md` |
| 脚本语法规范 | `docs/authoring/script-syntax.md` |
| 经验沉淀 | `docs/maintenance/lessons-learned.md` |
| 架构约束 | `ARCH.md` |
| RFC 索引 | `RFCs/README.md` |

## 关键领域约束

### 资源系统

所有资源路径必须使用 `&LogicalPath`，所有读取通过 `ResourceManager` 单例。`FsSource`/`ZipSource` 仅在 `init::create_resource_manager` 内构造。Config 加载后不可变；运行时变更用 `UserSettings`。

### 渲染与效果

`DrawCommand` 使用 `Arc<dyn Texture>`，后端通过 `as_any()` 下转。效果三级：`EffectKind → ResolvedEffect → EffectRequest`，带 capability fallback。动画基于 `dt` 时间驱动，不基于帧。`NullTexture`/`NullTextureFactory` 用于无 GPU 测试。

### Tauri 前端

所有引擎调用通过 `useEngine()` composable（不直接 `callBackend`）。日志用 `createLogger()`（不用 `console.log`）。URL 通过 `resolveAssetSrc()`/`useAssets()`。VN 组件只 emit 事件，`App.vue` 负责调用后端。

## 人机协作人体工学

### 语言

所有回复使用**简体中文**。代码、命令、标识符保持原样，仅自然语言部分使用中文。

### 多阶段任务的进度同步

连续执行大的多阶段任务时，每完成一个阶段里程碑，须在对话框中向用户简要描述该阶段成果，再执行下一阶段。

### 决策不确定时的引导式确认

遇到需求不明确、多种技术方案各有取舍、改动范围超预期、或对现有设计意图拿不准时，须主动向用户确认。使用 `AskUserQuestion` 工具发起结构化选择题（2-5 个选项，每个附简要说明），避免开放式提问。

## 工作协议

### 摘要优先

所有任务默认"摘要优先、源码兜底"：先按 `docs/maintenance/summary-index.md` 阅读摘要与导航，再决定源码最小读取范围。维护要求见 `docs/maintenance/summary-maintenance.md`。

### 符号索引

编码任务开始前，先读 `docs/engine/symbol-index.md`。该文件由 `cargo gen-symbols` 自动生成，列出所有 pub 符号的名称、类型、行号，以及 enum 变体名和 trait 方法名。工作完成后如新增/删除/移动了 pub 符号，须刷新符号索引。

### 源码读取约束（Token 护栏）

- \>500 行文件禁止整读。先 `rg` 定位，再 `ReadFile(offset, limit)` 片段读取。
- 例外：局部读取 2 次以上仍无法定位，或需跨相邻代码块验证不变量，或用户显式授权。
- 仅新增测试时，不读原有测试正文。用 `rg` 定位插入锚点，读 20-60 行上下文即可。

### 超预算保护

出现以下情况须暂停并向用户确认：已整读 >=2 个大文件（>500 行）；累计读取 >2000 行且未进入修改；连续 3 次读取仍无法定位。确认时提供：已读文件清单与目的、当前卡点、下一步最小读取计划。

### 经验沉淀（Session 学习）

开发中遇到非显而易见的问题时：先查 `docs/maintenance/lessons-learned.md` 确认是否已知；新发现则追加条目（现象/原因/正确做法三段式）。

### RFC 流程

跨模块改造、语法/语义变更、Runtime/Host 协议变更等重大方案须走 RFC。文件放 `RFCs/`，命名 `rfc-<topic>.md`，至少包含：背景、目标/非目标、提案、风险、迁移计划、验收标准。实施前对齐 RFC；偏离时先更新 RFC 再改代码。
