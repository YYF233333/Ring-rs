## Agent 指南

> 目标：让“进仓库的任何模型/协作者”在最少上下文下，快速对齐本项目的**硬约束**、**导航入口**、**常用命令**与**改动边界**。

我们正在开发一个视觉小说引擎，你是项目的lead engineer，遵照以下准则进行项目开发。

---

## 1) 项目目标

- 构建一个 **可运行、结构清晰、可扩展** 的 Visual Novel Engine
- 以“最小人类干预”的方式迭代：要求代码可读、可测、可维护

---

## 2) 总体架构原则（硬约束）

### 2.1 Runtime 与 Host 分离

- **`vn-runtime`**：纯逻辑核心（脚本解析/执行、状态管理、等待建模、产出 `Command`）
- **`host`**：IO/渲染/音频/输入/资源宿主（执行 `Command` 产生画面/音频/UI）
- Runtime **禁止**：引擎 API（macroquad）、IO、真实时间依赖
- Host **禁止**：脚本逻辑；直接修改 Runtime 内部状态

### 2.2 显式状态、确定性执行

- 所有运行状态必须**显式建模**且可序列化（支持存档/读档）
- 不允许隐式全局状态
- 不依赖真实时间推进逻辑（时间等待由 Host 负责）

### 2.3 命令驱动（Command-based）

- Runtime **只产出** `Command`
- Host **只执行** `Command`
- Runtime 不直接渲染/播放音频/等待输入

---

## 3) VN Runtime 核心模型（必须遵守）

### 3.1 `RuntimeState`（唯一可变状态）

- 脚本执行位置（`ScriptPosition`）
- 脚本变量（variables）
- 当前等待状态（`WaitingReason`）
- 以及其他可恢复的显式状态（如已显示角色/背景等）

要求：**可序列化**、可测试；禁止隐式状态。

### 3.2 `WaitingReason`（显式等待模型）

允许的等待原因（示例口径）：

```text
None
WaitForClick
WaitForChoice { choice_count }
WaitForTime(Duration)
WaitForSignal(SignalId)
```

禁止使用隐式 await/sleep 来推进脚本。

### 3.3 执行模型（tick）

- Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动
- 若处于等待：仅处理输入尝试解除等待
- 若不等待：持续推进脚本直到再次阻塞或结束

### 3.4 `RuntimeInput`（Host → Runtime）

典型输入：

```text
Click
ChoiceSelected(index)
Signal(signal_id)
```

说明：`WaitForTime` 由 Host 处理（Host 等待指定时长再调用 tick）。

---

## 4) Command 模型（Runtime → Host）

- `Command` 是 Runtime 与 Host 的**唯一通信方式**
- 要求：**声明式**、不包含引擎类型、不产生副作用

---

## 5) 质量与可维护性要求（硬约束）

- 核心逻辑必须有单元测试；修 bug 必须补回归测试
- Public API 必须有文档注释
- 禁止“顺便重构无关代码”
- 方案选择：优先清晰/可读/可测；优先最简单且符合约束

---

## 6) 仓库导航（强烈建议先读）

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

## 7) 常用命令（本地门禁/测试/覆盖率）

- 一键门禁：`cargo check-all`
- 常用测试：`cargo test -p vn-runtime --lib`
- 覆盖率：`cargo cov-runtime` / `cargo cov-workspace`（报告：`target/llvm-cov/html/index.html`）

---

## 8) Windows PowerShell 注意事项

- PowerShell 不支持 `&&` 作为语句分隔符，用 `;`：
  - `cd F:\Code\Ring-rs; cargo test ...`

---

## 9) 端到端手测脚本入口

- `config.json` 的 `start_script_path`
- 综合脚本：`assets/scripts/test_comprehensive.md`
