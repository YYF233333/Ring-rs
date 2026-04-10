# Domain Partitions

项目按职责划分为 6 个领域。在拆分大任务、并行审查、或确定改动范围时，以此表为默认边界。

## 领域分区表

| Domain ID | Scope | Summary files |
|-----------|-------|---------------|
| `script-lang` | `vn-runtime/src/script/**`, `command/**`, `diagnostic.rs` | script, command, diagnostic, parser |
| `runtime-engine` | `vn-runtime/src/runtime/**`, `state.rs`, `input.rs`, `save.rs`, `history.rs` | runtime, vn-runtime overview |
| `host-app` | `host/src/app/**`, `command_executor/**`, `host_app.rs`, `egui_actions.rs` | app, app_update, app_command_handlers, command_executor, host_app, egui_actions |
| `renderer` | `host/src/renderer/**`, `rendering_types.rs`, `backend/**` | renderer, render_state, animation, effects, scene_transition, rendering_types, backend |
| `resources` | `host/src/resources/**`, `manifest/**`, `config/**`, `save_manager/**` | resources, manifest, config, save_manager |
| `media-ui` | `host/src/audio/**`, `video/**`, `input/**`, `ui/**`, `egui_screens/**`, `extensions/**` | audio, video, input, ui, egui_screens, extensions |

详细不变量和 Do/Don't 规则见各领域规则文件：`.cursor/rules/domain-{id}.mdc`。

## 使用指南

### 任务分区

1. 将任务映射到受影响的领域。
2. 一个工作单元对应一个领域（除非某领域影响极小，可合并）。
3. 跨领域任务由顶层协调者掌控边界契约（如 Command enum 变更跨 `script-lang` + `host-app`）。

### 分区质量标准

每个分区应有：

- 明确的文件范围，与其他分区最小重叠。
- 具体的预期产出。
- 合理的源码规模（建议每分区 < 5K LOC）。大目录（如 `host/`）须按子系统进一步拆分。

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
