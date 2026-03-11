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
| 直接构造 `FsSource` / `ZipSource`（`init.rs` 之外） | `create_resource_source()` 统一入口 |
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
| 导航地图 | `docs/navigation_map.md` |
| 摘要入口 | `docs/summary_index.md` |
| 脚本语法规范 | `docs/script_syntax_spec.md` |

## 常用命令

| 用途 | 命令 |
|------|------|
| 一键门禁 | `cargo check-all` |
| 测试 | `cargo test -p vn-runtime --lib` |
| 覆盖率 | `cargo cov-runtime` / `cargo cov-workspace` |

覆盖率报告：`target/llvm-cov/html/index.html`

## 工作协议

### 摘要优先

所有任务默认"摘要优先、源码兜底"：先按 `docs/summary_index.md` 阅读摘要与导航，再决定源码最小读取范围。维护要求见 `docs/summary_maintenance.md`。

### 源码读取约束（Token 护栏）

- \>500 行文件禁止整读。先 `rg` 定位，再 `ReadFile(offset, limit)` 片段读取。
- 例外：局部读取 2 次以上仍无法定位，或需跨相邻代码块验证不变量，或用户显式授权。

仅新增测试时，不读原有测试正文。用 `rg` 定位插入锚点（模块末尾、`#[test]`、`mod tests`），读 20-60 行上下文即可。升级读取仅在：插入失败、编译报结构性错误、或任务要求修改已有测试逻辑。

### 超预算保护

出现以下情况须暂停并向用户确认：已整读 >=2 个大文件（>500 行）；累计读取 >2000 行且未进入修改；连续 3 次读取仍无法定位。

确认时提供：已读文件清单与目的、当前卡点、下一步最小读取计划。

### 输出复盘

读写超过 10 个文件时，最终回复须包含：实际读写顺序、源码阅读必要性（必要/可优化/不必要）、大文件整读理由。

### RFC 流程

跨模块改造、语法/语义变更、Runtime/Host 协议变更等重大方案须走 RFC。

- 文件放 `RFCs/`，命名 `rfc-<topic>.md`
- 至少包含：背景、目标/非目标、提案、风险、迁移计划、验收标准
- 实施前对齐 RFC；偏离时先更新 RFC 再改代码
- 实现完成后同步相关文档
