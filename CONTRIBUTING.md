# 贡献与开发指南（单人模式）

本仓库目前以**单人开发、本地运行**为主：暂不强制 CI，但要求每次提交前通过本地质量门禁。

## 必备环境

- Rust：建议使用较新的 Rust（本仓库使用 `edition = "2024"`，因此需要较新的 toolchain；不强制锁定版本）
- Windows：PowerShell（用于 `cargo` alias 的一键门禁）

## 常用命令（推荐记住这几个）

### 一键质量门禁（提交前必跑）

- **全量**（fmt + clippy + workspace tests）：
  - `cargo check-all`

### 分步执行（定位问题用）

- **格式化检查**：`cargo fmt-check`
- **Clippy**：`cargo clippy-all`
- **Clippy（严格，warnings 视为错误）**：`cargo clippy-deny`
- **测试（workspace）**：`cargo test-all`
- **测试（runtime）**：`cargo test-runtime`
- **测试（host lib）**：`cargo test-host-lib`

### 资源打包

- `cargo pack -- --help`

## 约定

- 新增/修复逻辑时优先补单元测试；需要跨模块链路时补 `host/tests/` 集成测试（阶段 20 会补齐）。
- 尽量把“纯逻辑”放在可测试模块中；渲染/音频/IO 作为薄壳。

