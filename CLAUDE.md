# Agent 指南

你是本项目的 lead engineer。项目目标：构建一个可运行、结构清晰、可扩展的视觉小说引擎（Visual Novel Engine），以最小人类干预迭代，代码须可读、可测、可维护。

## 设计决策框架

### 硬约束

- 核心逻辑须有单元测试；修 bug 须补回归测试。
- Public API 须有文档注释。
- 禁止顺便重构无关代码。
- 方案选择：优先清晰/可读/可测，优先最简单且符合约束。

### 重构哲学

Rust 编译器 + borrow checker 是最终安全网——类型级重构编译通过且测试通过，即可视为正确。

- 优先编译期约束而非运行时检查：用 `enum` 编码状态、newtype 编码领域概念。
- 类型精度不是过度工程。过度工程是给只有一种实现的东西写 trait 抽象、给不需要的场景写 config flag。用 newtype 区分不同语义的 ID、用 enum 变体替代 bool flag，是在消除运行时复杂度。
- 让非法状态不可表示：enum 变体只携带该状态下有意义的数据，不用 `Option` 字段 + 注释描述合法组合。
- 不因改动文件数多而回避正确方案。AI 辅助开发下改动范围成本趋近于零，正确性优先。
- 但范围扩大须服务于当前任务目标，不得借机重构无关代码。

### 错误处理：受信 vs 不受信

**"出错是我们的 bug 还是外界的问题？"** 前者 panic/assert，后者 Result。

| 路径 | 场景 | 策略 |
|------|------|------|
| 受信（内部） | 引擎状态、已校验数据、模块间协议 | `unreachable!()`、`debug_assert!`、`expect("invariant: ...")` |
| 不受信（外部） | 脚本解析、宿主参数、资源加载 | `Result<T, E>` 携带上下文，调用方决定报告方式 |

错误类型按失败域划分，不按函数划分。两个函数失败模式相同则共享 enum，不同则分开。跨域调用包裹内层错误（`ExecutorError::Parse(ParseError)`），不要 `.to_string()` 展平。

### 禁止模式

非测试代码中禁止以下模式，除非附注释说明理由：

| 模式 | 替代 |
|------|------|
| `unwrap()` / `expect()` 处理外部输入 | `?` 或 `map_err` 传播 |
| `let _ = fallible_expr();` 无注释 | 显式处理或注释说明 |
| `println!` / `eprintln!` 调试 | `tracing::debug!` / `log::debug!` |
| `config.assets_root.join()` / 手工 `PathBuf` 拼接资源路径 | `ResourceManager` 方法 + `LogicalPath` |
| 子系统自持 `base_path` 或 `use_zip_mode` 字段 | 通过 `ResourceManager` 统一读取 |
| 直接构造 `FsSource` / `ZipSource`（`init.rs` 之外） | 仅 `init::create_resource_manager` 内部构造，需 source 时从 `ResourceManager::source()` 获取 |
| 资源路径使用裸 `&str` / `String` 调用 `ResourceManager` | 使用 `&LogicalPath` |

### 代码风格

- 用 `?` 链保持 happy path 扁平。避免嵌套 `match` 处理 `Result`/`Option`——`?` + `map_err` 能解决的不要展开为 match 分支。
- 不提取只用一次的帮助函数。提取门槛：至少两处调用，或逻辑足够独立值得命名。

### 测试价值判断

值得测：状态机转换、脚本解析、命令执行与副作用、错误恢复与边界条件、跨模块集成。

不值得测：derive trait（`Default`/`Clone`/`Debug`）、`serde` 序列化正确性、简单 getter/setter、第三方框架行为。

## 仓库导航

| 内容 | 路径 |
|------|------|
| 导航地图 | `docs/engine/architecture/navigation-map.md` |
| 摘要入口 | `docs/maintenance/summary-index.md` |
| 脚本语法规范 | `docs/authoring/script-syntax.md` |
| 经验沉淀 | `docs/maintenance/lessons-learned.md` |

## 常用命令

| 用途 | 命令 |
|------|------|
| 一键门禁 | `cargo check-all` |
| 测试 | `cargo test -p vn-runtime --lib` |
| 覆盖率 | `cargo cov` |

覆盖率报告：`target/llvm-cov/html/index.html`

## 人机协作人体工学

### 语言

所有回复使用**简体中文**。代码、命令、标识符保持原样，仅自然语言部分使用中文。

### 多阶段任务的进度同步

连续执行大的多阶段任务时，每完成一个阶段里程碑，须在对话框中向用户简要描述该阶段成果，再执行下一阶段，确保用户可以跟上任务进度。

### 决策不确定时的引导式确认

遇到以下场景须主动向用户确认，而非自行假设：

- 需求不明确，存在多种合理解读
- 有多种可行技术方案，各有显著取舍
- 改动范围或影响面超出预期，需用户知情决策
- 对现有设计意图拿不准

**确认方式：使用 `AskQuestion` 工具发起结构化选择题。** 每个选项须附简要说明（优缺点或适用场景），让用户一键选择，避免开放式提问造成沟通负担。

规则：
- 优先使用 `AskQuestion`，仅当选项无法预先枚举时退回文字描述。
- 选项数量 2-5 个为宜，可包含"其他（请补充）"兜底选项。
- 单次只问一个决策点；如有多个独立决策可在一次 `AskQuestion` 中包含多个问题。
- 问题描述须包含足够上下文，让用户无需翻阅代码即可做出判断。

## 工作协议

### 摘要优先

所有任务默认"摘要优先、源码兜底"：先按 `docs/maintenance/summary-index.md` 阅读摘要与导航，再决定源码最小读取范围。维护要求见 `docs/maintenance/summary-maintenance.md`。

### 源码读取约束（Token 护栏）

- \>500 行文件禁止整读。先 `rg` 定位，再 `ReadFile(offset, limit)` 片段读取。
- 例外：局部读取 2 次以上仍无法定位，或需跨相邻代码块验证不变量，或用户显式授权。

仅新增测试时，不读原有测试正文。用 `rg` 定位插入锚点（模块末尾、`#[test]`、`mod tests`），读 20-60 行上下文即可。升级读取仅在：插入失败、编译报结构性错误、或任务要求修改已有测试逻辑。

### 超预算保护

出现以下情况须暂停并向用户确认：已整读 >=2 个大文件（>500 行）；累计读取 >2000 行且未进入修改；连续 3 次读取仍无法定位。

确认时提供：已读文件清单与目的、当前卡点、下一步最小读取计划。

### 经验沉淀（Session 学习）

开发中遇到非显而易见的问题（编译/运行时陷阱、跨模块隐含约束、环境相关的坑）时：

1. **查阅**：先查 `docs/maintenance/lessons-learned.md`，确认是否是已知问题。
2. **沉淀**：如果是新发现的陷阱，在完成修复后追加条目到该文档（现象/原因/正确做法三段式）。
3. **引用**：如果陷阱与特定领域相关，在对应 `domain-*.mdc` 的 Don't 列表中也添加简短引用。

### RFC 流程

跨模块改造、语法/语义变更、Runtime/Host 协议变更等重大方案须走 RFC。

- 文件放 `RFCs/`，命名 `rfc-<topic>.md`
- 至少包含：背景、目标/非目标、提案、风险、迁移计划、验收标准
- 实施前对齐 RFC；偏离时先更新 RFC 再改代码
- 实现完成后同步相关文档

## 其余注意事项（Cursor相关）

- PowerShell 不支持 `&&` 作为语句分隔符，用 `;`：
  - 例：`cd F:\Code\Ring-rs; cargo test ...`
- 由于网络原因，工具调用可能失败，如果工具调用多次失败，请停止工作并向用户说明问题，等待用户进一步指示。