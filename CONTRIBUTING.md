# 贡献与开发指南

本仓库当前采用“**本地自检 + CI 兜底**”的质量门禁模式：开发者本地建议先跑一遍 `cargo check-all`，远端由 CI 统一执行并阻止不符合门禁的提交合入。

## 必备环境

- Rust：建议使用较新的 Rust（本仓库使用 `edition = "2024"`，因此需要较新的 toolchain；不强制锁定版本）
- Windows：PowerShell（用于 `cargo` alias 的一键门禁）

## CI 门禁

本仓库不再使用 pre-commit hook。质量门禁统一交给 CI 执行：

- CI 运行 `cargo check-all`
- 随后执行 `git diff --exit-code`

这样可以同时覆盖：

- `cargo fmt --all` 产生的格式化改动
- `cargo clippy --fix` 产生的自动修复改动
- `cargo test --workspace` 的回归验证

本地开发时仍强烈建议在提交前手动运行 `cargo check-all`，这样可以在推送前尽早看到格式化、clippy 和测试问题。

## 常用命令（推荐记住这几个）

### 一键质量门禁（本地自检）

- **全量**（fmt + clippy + workspace tests，与 CI 共用同一门禁命令）：
  - `cargo check-all`
  - 说明：该命令通过 `tools/xtask` 串行执行 `fmt --all`（直接应用）→ `clippy --fix` → `test`

### 分步执行（定位问题用）

- **格式化检查**：`cargo fmt-check`
- **Clippy**：`cargo clippy-all`
- **Clippy（严格，warnings 视为错误）**：`cargo clippy-deny`
- **测试（workspace）**：`cargo test-all`
- **测试（runtime）**：`cargo test-runtime`
- **测试（host-dioxus lib）**：`cargo test-dioxus`

### 资源打包

- `cargo pack -- --help`

### 脚本检查（Script Check）

静态检查脚本文件，发现问题无需运行游戏：

- `cargo script-check`：检查 `assets/scripts/` 下所有脚本
- `cargo script-check <path>`：检查指定文件或目录

检查内容：
- 语法错误（解析失败）
- 未定义的跳转目标（goto/choice 引用不存在的 label）
- 资源文件是否存在（背景/立绘/音频）

诊断输出包含精确行号，便于快速定位问题：
```
[ERROR] script.md:42: 未定义的跳转目标: **missing_label**
```

### Dev Mode 自动诊断

在 **debug build**（`cargo run`）时 `debug.script_check` 默认值为 `true`，
Host 启动会自动运行脚本检查并输出诊断结果；release build 默认值为 `false`。

- 默认只输出警告，不阻塞启动
- 可在 `config.json` 中配置开关：
  - 开启：`"debug": { "script_check": true }`
  - 关闭：`"debug": { "script_check": false }`

## 指标追踪（Metrics）

除门禁外，每次推送到 `main` 还会触发 `metrics` workflow（`.github/workflows/metrics.yml`），自动采集以下指标并更新 GitHub Pages 趋势仪表盘：

- 测试覆盖率（workspace / vn-runtime / host-dioxus）
- 二进制大小（release / stripped）
- 构建耗时（debug / release clean build）
- 代码行数（tokei Rust + 有效生产代码 LOC）
- 依赖数量（直接 / 传递）
- 测试数量（passed / failed / ignored）

该 workflow 不阻塞 PR 合入，仅用于长期趋势观察。

## 覆盖率（Coverage）

本仓库的覆盖率目标以 **`vn-runtime` 接近 100%** 为主（`host-dioxus` 不追求纯覆盖率数值，优先保证关键链路可回归、可测试）。

- 运行：`cargo cov` 生成 workspace 覆盖率 HTML 报告
- 报告位置：`target/llvm-cov/html/index.html`
- 详情见：[docs/testing/coverage.md](docs/testing/coverage.md)

## 仓库导航地图（强烈建议先看）

- [docs/engine/architecture/navigation-map.md](docs/engine/architecture/navigation-map.md)：按“常见改动场景”索引到具体 crate/模块/文件，减少无效翻文件。
- [docs/README.md](docs/README.md)：文档总入口，按读者与任务分流到 authoring / engine / testing / maintenance。

## 约定

- 新增/修复逻辑时优先补单元测试。
- 尽量把”纯逻辑”放在可测试模块中；渲染/音频/IO 作为薄壳。

