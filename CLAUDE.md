# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

你是本项目的 lead engineer。项目目标：构建一个可运行、结构清晰、可扩展的视觉小说引擎（Visual Novel Engine），以最小人类干预迭代，代码须可读、可测、可维护。

## 架构概览

### Runtime/Host 分离（硬约束）

引擎分为纯逻辑核心和宿主两层，通过 `Command`（Runtime→Host）和 `RuntimeInput`（Host→Runtime）通信，禁止其他耦合：

- **`vn-runtime`**：纯逻辑——脚本解析/执行、状态管理、等待建模、产出 `Command`。禁止 IO/渲染/真实时间依赖。所有状态在 `RuntimeState` 中显式建模且可序列化。
- **`host-dioxus`**：Dioxus 0.7 Desktop 渲染宿主，Rust 全栈（RSX 声明式 UI），无 IPC 边界。执行 `Command` 产生画面/音频/UI。禁止脚本逻辑、直接修改 Runtime 内部状态。

### 核心执行模型

Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动。确定性执行：相同输入序列产出相同 Command 序列。时间等待由 Host 负责。

### Workspace 成员

```
vn-runtime/          # 纯逻辑 Runtime
host-dioxus/         # 宿主（Dioxus Desktop）
tools/xtask/         # 门禁/覆盖率/脚本检查
tools/asset-packer/  # 资源打包
tools/debug-mcp/     # Debug MCP Server（Node.js，封装 HTTP REST API）
```

默认 `cargo run` 执行 `host-dioxus`。

## 常用命令

| 用途 | 命令 |
|------|------|
| 一键门禁（CI 同款） | `cargo check-all`（fmt → clippy --fix → test） |
| 测试 | `cargo test -p vn-runtime --lib` |
| 单个测试 | `cargo test -p vn-runtime --lib test_name` |
| 覆盖率 | `cargo cov`（报告：`target/llvm-cov/html/index.html`） |
| 符号索引（定期） | `cargo gen-symbols` |
| 脚本静态检查 | `cargo script-check [path]` |
| 变异测试 | `cargo mutants` |

### Debug Server（实时交互调试）

host-dioxus 内嵌 HTTP REST API + MCP 封装，允许 CC 在游戏运行时查询状态、驱动操作、截图。

**启用策略**：debug build 默认启用，release 默认关闭。优先级：`RING_DEBUG_SERVER` env > `config.debug.enable_debug_server` > `cfg!(debug_assertions)`。

| 用途 | 命令/操作 |
|------|-----------|
| 启动（debug build 自动） | `cargo run` |
| 强制启用 | `RING_DEBUG_SERVER=1 cargo run` |
| 健康检查 | `curl http://127.0.0.1:9876/api/ping` |
| 查询完整状态 | `curl http://127.0.0.1:9876/api/state` |
| 推进对话 | `curl -X POST http://127.0.0.1:9876/api/click` |
| 选择选项 | `curl -X POST http://127.0.0.1:9876/api/choose -d '{"index":0}'` |
| 批量推进 | `curl -X POST http://127.0.0.1:9876/api/advance -d '{"max_clicks":10}'` |
| 截图 | `curl http://127.0.0.1:9876/api/screenshot` |

MCP 集成：`.mcp.json` 已配置 `ring-debug` server，重启 CC session 后可直接使用 MCP tools。

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
| 新增/修改 Command | `vn-runtime` command 模块 + `host-dioxus` command_executor |
| 新增脚本语法 | 先更新语法规范 → parser + AST + executor + round-trip 测试 |
| 新增 UI 页面 | `host-dioxus` 对应页面 |
| Typewriter 节奏标签 | parser inline_tags + Command InlineEffect + host-dioxus consumer |
| 大批量新增/移动 pub 符号 | 运行 `cargo gen-symbols` 刷新符号索引 |
| 新增 Debug API 端点 | `host-dioxus/src/debug_server.rs` + `tools/debug-mcp/index.js` MCP tool 映射 |

### 常见工作流详细指南

| 工作流 | 文档 |
|--------|------|
| 新增/修改 Command 全管线 | `docs/workflows/cross-module-command-pipeline.md` |
| 扩展脚本语法 | `docs/workflows/script-syntax-extension.md` |
| RFC 完整流程 | `docs/workflows/rfc-workflow.md` |
| 领域分区（大任务拆分） | `docs/workflows/domain-partitions.md` |

## 仓库导航

| 内容 | 路径 |
|------|------|
| 导航地图（"改哪里"） | `docs/engine/architecture/navigation-map.md` |
| 符号索引 | `docs/engine/symbol-index.md` |
| 脚本语法规范 | `docs/authoring/script-syntax.md` |
| 经验沉淀 | `docs/maintenance/lessons-learned.md` |
| 架构约束 | `ARCH.md` |
| RFC 索引 | `RFCs/README.md` |
| 工作流指南 | `docs/workflows/` |

## 关键领域约束

### 资源系统

所有资源路径必须使用 `&LogicalPath`，所有读取通过 `ResourceManager` 单例。`FsSource`/`ZipSource` 仅在 `init::create_resource_manager` 内构造。Config 加载后不可变；运行时变更用 `UserSettings`。

### 领域不变量速查

详细不变量和 Do/Don't 规则见 `.cursor/rules/`（Cursor 与 CC 共享）：

| 领域 | 规则文件 | 核心约束摘要 |
|------|----------|-------------|
| 脚本语言 | `.cursor/rules/domain-script-lang.mdc` | 两阶段解析器（Block 识别→语义解析）；AST 无运行时状态 |
| 资源与配置 | `.cursor/rules/domain-resources.mdc` | LogicalPath 强制；Config 加载后不可变 |
| 运行时引擎 | `.cursor/rules/domain-runtime-engine.mdc` | 确定性执行；显式状态；存档向后兼容 |

### 存档兼容

`SaveData` 序列化结构变更须保持向后兼容（新字段加 `#[serde(default)]`）。不兼容变更须走 RFC。

## 人机协作人体工学

### 语言

所有回复使用**简体中文**。代码、命令、标识符保持原样，仅自然语言部分使用中文。

### 多阶段任务的进度同步

连续执行大的多阶段任务时，每完成一个阶段里程碑，须在对话框中向用户简要描述该阶段成果，再执行下一阶段。

### 决策确认（Proactive Confirmation）

**默认姿态：有选择时先问，不要替用户做决定。** 不确定时宁可多问一次，也不要自行判断后事后返工。

须主动使用 `AskUserQuestion` 发起结构化选择题（2-5 选项，每选项附简要说明），避免开放式提问。

#### 必须确认的场景

| 场景 | 示例 |
|------|------|
| 存在 ≥2 个合理技术方案 | "用 enum 还是 trait object 建模？" |
| 任务拆分方式有多种合理选择 | "先做 A 再做 B，还是一起做？" |
| 对现有设计意图拿不准 | "这个 Option 字段是故意的还是遗留的？" |

#### 不需要确认的场景

- 单一明确方案的实现细节
- 遵循已有模式的机械扩展（如 match 分支补全）

## 工作协议

### 符号定位

优先使用 **LSP**（rust-analyzer）进行符号查找、跳转定义、查找引用。`docs/engine/symbol-index.md` 作为离线浏览参考，需要跨模块全局概览时可读取。`cargo gen-symbols` 在大批量 pub API 变更后运行，无需每次编码后刷新。

### Commit 规范

格式：`type(scope): summary`。scope 可省略。summary 用英文，祈使语气，不超过 60 字符。

| type | 含义 |
|------|------|
| feat | 新功能或新指令 |
| fix | Bug 修复 |
| refactor | 重构（不改变行为） |
| test | 仅测试变更 |
| docs | 仅文档变更 |
| chore | 构建/工具/CI 变更 |

Body（可选）：简述 what & why，不超过 3 行。

```
feat(parser): add titleCard block-level instruction

phase1 recognition, phase2 parsing, executor mapping.
Includes round-trip parser tests.
```

### 经验沉淀（Session 学习）

开发中遇到非显而易见的问题时：先查 `docs/maintenance/lessons-learned.md` 确认是否已知；新发现则追加条目（现象/原因/正确做法三段式）。

### RFC 流程

跨模块改造、语法/语义变更、Runtime/Host 协议变更、存档格式变更须走 RFC。详见 `docs/workflows/rfc-workflow.md`。索引：`RFCs/README.md`。

实施 RFC 的 session 须在结束前同步 RFC 状态：实施中标 `Active`；验收标准全部达成后标 `Accepted`，将文件移至 `RFCs/Archived/`，并更新 `RFCs/README.md` 索引。
