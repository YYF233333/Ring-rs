## Agent 指南

> 目标：让“进仓库的任何模型/协作者”在最少上下文下，快速对齐本项目的**硬约束**、**导航入口**、**常用命令**与**改动边界**。

我们正在开发一个视觉小说引擎，你是项目的lead engineer，遵照以下准则进行项目开发。

---

## 1) 项目目标

- 构建一个 **可运行、结构清晰、可扩展** 的 Visual Novel Engine
- 以“最小人类干预”的方式迭代：要求代码可读、可测、可维护

---


## 2) 质量与可维护性要求（硬约束）

- 核心逻辑必须有单元测试；修 bug 必须补回归测试
- Public API 必须有文档注释
- 禁止“顺便重构无关代码”
- 方案选择：优先清晰/可读/可测；优先最简单且符合约束

---

## 3) 仓库导航

- 导航地图（人工索引）：`docs/navigation_map.md`

从脚本到画面的关键链路（常用入口）：

- 规范：`docs/script_syntax_spec.md`
- 解析：`vn-runtime/src/script/parser/mod.rs`（模块目录：`vn-runtime/src/script/parser/`）
- AST：`vn-runtime/src/script/ast.rs`
- 表达式：`vn-runtime/src/script/expr.rs`
- 执行：`vn-runtime/src/runtime/executor.rs`
- 引擎循环：`vn-runtime/src/runtime/engine.rs`
- 状态：`vn-runtime/src/state.rs`
- Host 执行链路：`host/src/app/update/script.rs`

---

## 4) 常用命令（本地门禁/测试/覆盖率）

- 一键门禁：`cargo check-all`
- 常用测试：`cargo test -p vn-runtime --lib`
- 覆盖率：`cargo cov-runtime` / `cargo cov-workspace`（报告：`target/llvm-cov/html/index.html`）

---

## 5) 注意事项

- PowerShell 不支持 `&&` 作为语句分隔符，用 `;`：
  - `cd F:\Code\Ring-rs; cargo test ...`

- 由于网络原因，工具调用可能失败，如果工具调用多次失败，请停止工作并向用户说明问题，等待用户进一步指示。

- 由于新版cursor的一些bug，调用cargo时可能显示无法找到rust toolchain，如遇这种情况请停止尝试，请求用户代为执行命令。

---

## 6) 端到端手测脚本入口

- `config.json` 的 `start_script_path`
- 综合脚本：`assets/scripts/test_comprehensive.md`
