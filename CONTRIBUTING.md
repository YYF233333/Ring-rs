# 贡献与开发指南（单人模式）

本仓库目前以**单人开发、本地运行**为主：暂不强制 CI，但要求每次提交前通过本地质量门禁。

## 必备环境

- Rust：建议使用较新的 Rust（本仓库使用 `edition = "2024"`，因此需要较新的 toolchain；不强制锁定版本）
- Windows：PowerShell（用于 `cargo` alias 的一键门禁）

## 常用命令（推荐记住这几个）

### 一键质量门禁（提交前必跑）

- **全量**（fmt + clippy + workspace tests）：
  - `cargo check-all`
  - 说明：该命令通过 `tools/xtask` 串行执行 `fmt --check` → `clippy` → `test`

### 分步执行（定位问题用）

- **格式化检查**：`cargo fmt-check`
- **Clippy**：`cargo clippy-all`
- **Clippy（严格，warnings 视为错误）**：`cargo clippy-deny`
- **测试（workspace）**：`cargo test-all`
- **测试（runtime）**：`cargo test-runtime`
- **测试（host lib）**：`cargo test-host-lib`

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

## 覆盖率（Coverage）

本仓库的覆盖率目标以 **`vn-runtime` 接近 100%** 为主（`host` 不追求纯覆盖率数值，优先保证关键链路可回归、可 headless 测试）。

- 运行（推荐 `cargo llvm-cov`，Windows 友好）：
  - `cargo cov-runtime`：生成 `vn-runtime` 覆盖率 HTML 报告
  - `cargo cov-workspace`：生成 workspace 覆盖率 HTML 报告（趋势观察）
- 报告位置：`target/llvm-cov/html/index.html`
- 详情见：`docs/coverage.md`

## 仓库导航地图（强烈建议先看）

- `docs/navigation_map.md`：按“常见改动场景”索引到具体 crate/模块/文件，减少无效翻文件。

## 约定

- 新增/修复逻辑时优先补单元测试；需要跨模块链路时补 `host/tests/` 集成测试（阶段 20 会补齐）。
- 尽量把“纯逻辑”放在可测试模块中；渲染/音频/IO 作为薄壳。

