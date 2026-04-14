# Domain Partitions

项目按职责划分为领域。在拆分大任务、并行审查、或确定改动范围时，以此表为默认边界。

## 领域分区表

| Domain ID | Scope | 核心约束 |
|-----------|-------|----------|
| `script-lang` | `vn-runtime/src/script/**`, `command/**`, `diagnostic.rs` | 两阶段解析器；AST 无运行时状态 |
| `runtime-engine` | `vn-runtime/src/runtime/**`, `state.rs`, `input.rs`, `save.rs`, `history.rs` | 确定性执行；显式状态；存档向后兼容 |
| `host-state` | `host-dioxus/src/state/**`, `command_executor.rs`, `render_state.rs` | Command 执行、游戏状态管理、tick 循环 |
| `host-ui` | `host-dioxus/src/vn/**`, `screens/**`, `components/**`, `main.rs` | Dioxus RSX 组件、屏幕导航、布局 |
| `resources` | `host-dioxus/src/resources.rs`, `manifest.rs`, `config.rs`, `save_manager.rs`, `init.rs` | LogicalPath 强制；Config 加载后不可变 |
| `host-infra` | `host-dioxus/src/audio.rs`, `debug_server.rs`, `error.rs` | 音频播放、调试 API、错误类型 |

详细不变量和 Do/Don't 规则见 `.claude/rules/domain-*.md`（Cursor 侧 `.cursor/rules/domain-*.mdc`）。

## 使用指南

### 任务分区

1. 将任务映射到受影响的领域。
2. 一个工作单元对应一个领域（除非某领域影响极小，可合并）。
3. 跨领域任务由顶层协调者掌控边界契约（如 Command enum 变更跨 `script-lang` + `host-state`）。

### 分区质量标准

每个分区应有：

- 明确的文件范围，与其他分区最小重叠。
- 具体的预期产出。
- 合理的源码规模（建议每分区 < 5K LOC）。

### 适合并行的任务类型

| 任务形态 | 拓扑 |
|----------|------|
| 全仓审查、审计、批量测试更新 | 按领域并行 |
| 有跨文件依赖的重构/重命名 | 顺序或单一执行 |
| 调试/根因分析 | 单一执行（假设链天然串行） |
| 跨多领域的功能实现 | 先设计契约，再按领域并行实现 |

### 不适合拆分的情况

- 需要分区间频繁交互。
- 多个分区重度修改相同文件。
- 依赖一个尚未确定的全局设计决策。
