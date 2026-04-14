# 覆盖率（Coverage）口径与运行指南

本项目当前由 CI 执行 `cargo check-all` 作为质量门禁（含 Rust fmt/clippy/test + 前端 biome/vue-tsc typecheck）；覆盖率本身仍以**本地可复现**为主。覆盖率统一使用 **workspace 口径**（排除工具 crate 与平台胶水代码），用于趋势观察与回归信号。

---

## 工具选择（Windows 优先）

### 推荐：`cargo llvm-cov`

在 Windows 上更稳定、生态更匹配（基于 LLVM 工具链）。

#### 安装

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

#### 生成报告

- `cargo cov`（等价于 `cargo run -p xtask -- cov`）
- HTML 报告：`target/llvm-cov/html/index.html`

---

## 排除策略

### 排除原则

CLAUDE.md 定义了"不值得测试"的代码类型：

- **derive trait**：`Default`/`Clone`/`Debug` 等编译器生成代码
- **serde 序列化**：序列化正确性是 serde 的责任
- **thiserror Display**：`#[error("...")]` 生成的 Display impl
- **简单 getter/setter**：无分支逻辑的单行方法
- **三方框架行为**：Dioxus RSX 组件渲染

这些代码计入覆盖率只会稀释信号，不提供回归价值。

### crate 级排除（`--exclude`）

| 排除 crate | 理由 |
|------------|------|
| `xtask` | 开发工具，不是产品代码 |
| `asset-packer` | 打包工具，不是产品代码 |

### 文件级排除（`--ignore-filename-regex`）

规则定义在 `tools/xtask/cov-ignore-regex.txt` 中（一行一个模式，运行时以 `|` 拼接）。

| 排除文件/目录 | 理由 |
|---------------|------|
| `*/src/lib.rs` | 模块声明与重导出，无可执行逻辑 |
| `vn-runtime/src/error.rs` | thiserror derive，无手写逻辑 |
| `host-dioxus/src/components/` | Dioxus RSX 组件，无法通过 Rust 单元测试覆盖 |
| `host-dioxus/src/screens/` | Dioxus 屏幕组件 |
| `host-dioxus/src/vn/` | Dioxus VN 渲染组件 |
| `host-dioxus/src/main.rs` | 平台入口，Dioxus 启动胶水 |
| `host-dioxus/src/debug_server.rs` | Debug HTTP server，含异步 IO |
