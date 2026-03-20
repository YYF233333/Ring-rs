# RFC: 测试分层与维护策略

**状态**：Accepted（已完成）

## 背景

当前仓库测试已经覆盖 `vn-runtime` 与 `host` 的关键逻辑，但不同测试的回归信号强度差异很大：

- 一类测试在守护不变量、状态机、边界条件、外部契约与跨模块链路。
- 一类测试主要在验证构造器、默认值、`Display`、getter、纯 `serde` roundtrip，更多是在提升机械覆盖率。
- 还有一部分测试命中了真实行为路径，但断言仍偏浅，暂时难以快速判定。

同时，现有 `mutants.out` 已证明 `vn-runtime` 的真实薄弱点主要集中在 parser 前端与少数 runtime 控制流分支，而不是整个测试面均匀薄弱。

## 目标

- 在仓库内引入统一的测试分层口径。
- 把 `high_value` 与 `low_value` 测试拆成独立测试子模块，降低核心回归集被低信号测试淹没的风险。
- 把 `undecided` 明确保留在原测试根模块，后续再逐步提升或淘汰。
- 为后续新增测试提供落点规则，避免继续无序增长。

## 非目标

- 不追求一次性重写全部测试。
- 不把所有 helper/冒烟测试都删除。
- 不引入新的测试框架、测试运行器或 CI 门禁。
- 不重新跑 `cargo mutants`；当前策略直接消费已有 `mutants.out`。

## 提案

### 1. 采用三档分类

- `high_value`
  - 不变量：变量域隔离、状态恢复、一次性消费语义、缓存/资源失效策略。
  - 状态机：等待态、过渡阶段、skip 分支、回退/恢复。
  - 外部契约：Runtime -> Host 命令转译、资源来源抽象、配置 schema、扩展能力兼容。
  - 错误分类与边界条件：非法输入、缺失依赖、损坏文件、跨脚本恢复。
  - `vn-runtime` 中能稳定杀死 mutants 的测试也优先并入这一层。

- `low_value`
  - 构造器/默认值。
  - getter/setter、简单 helper。
  - `Display` / `Debug` / 字符串格式化。
  - 纯 `serde` roundtrip。
  - 单纯的路径文件名格式或枚举映射测试。

- `undecided`
  - 命中了真实路径，但断言只停在 “`is_ok()` / `is_err()` / 有结果”。
  - helper 边界测试与集成冒烟测试之间的中间地带。
  - 当前变更热度不足以判断是否应升级为核心测试。

### 2. 采用统一模块结构

对已有 companion `tests.rs` 的模块，统一改成：

```rust
mod high_value;
mod low_value;

use super::*;

// undecided tests 留在 tests.rs 根模块
```

目录布局示例：

```text
src/foo/tests.rs
src/foo/tests/high_value.rs
src/foo/tests/low_value.rs
```

说明：

- `high_value.rs` 与 `low_value.rs` 只承载“已明确归类”的测试。
- 暂时无法确定的测试继续留在 `tests.rs` 根文件中，避免为了分层而强行定级。
- 内联 `#[cfg(test)] mod tests` 的文件，只在分类收益明显时再迁出；小模块可暂时维持原状。

### 3. 用现有 `mutants.out` 作为佐证，而不是重新运行

- `missed.txt` 用于识别“语义断言不足”的真实缺口。
- `outcomes.json` / `mutants.json` / `diff/` 用于定位源文件、函数与变异类型。
- `timeout` 仅作为潜在死循环或卡死路径信号，不等同于必须补新测试。
- `unviable` 视为噪音，不纳入测试价值评估。

### 4. 新增测试默认落点

- 新增或修 bug 时，优先进入 `high_value`。
- 若测试只是在保留 schema/格式兼容、简单 API 形状，可放入 `low_value`。
- 若当前无法确认长期价值，可先留在 `tests.rs` 根模块，待下一轮审计再处理。

## 风险

- 错误分类会把未来重要的测试降权。
- 机械搬迁测试时，容易丢失共享 helper 或导入。
- 大文件测试分层后，review 体验可能先变好，但 grep 路径会变多。

## 风险缓解

- `undecided` 明确保留，不强迫一次性分类到底。
- 每次迁移只做“搬家+最小导入修复”，不顺手改断言逻辑。
- 先迁移混杂最严重、价值最清晰的测试入口，再处理长尾。

## 迁移计划

1. 先生成仓库级审计清单，建立统一口径。
2. 先迁移混杂最明显的 companion `tests.rs`。
3. 对 `vn-runtime` 优先关注 parser / engine / executor / diagnostic。
4. 对 `host` 优先关注 command executor / render state / resources / save manager / scene transition。
5. 对小型内联测试模块，只在收益明确时再迁出。

## 执行情况（已完成的迁移）

- **审计与文档**：已建立 `docs/maintenance/test-audit-inventory.md`（按测试函数粒度的分类样本与口径），本 RFC 作为分层与维护策略的正式说明。
- **结构迁移**：全仓库原 companion `tests.rs` 已统一迁入 `tests/mod.rs`，共 19 个入口。
- **分层完成范围**：以下 19 个 `tests/mod.rs` 均已增加 `mod high_value; mod low_value;` 并完成分类迁移，`undecided` 仅在有争议时保留在根模块。

| crate | 模块路径 | 说明 |
|-------|----------|------|
| vn-runtime | `command/tests` | 契约/position/transition 高价值，serialization/getter 低价值 |
| vn-runtime | `diagnostic/tests` | 分析/跳转/资源引用/源图 高价值，Display/merge 低价值 |
| vn-runtime | `runtime/engine/tests` | 状态机/等待/调用栈/错误 高价值，构造/getter 低价值 |
| vn-runtime | `runtime/executor/tests` | 命令执行/路径/条件/信号 高价值，简单命令 低价值 |
| vn-runtime | `script/parser/tests` | 语法/错误/transition/条件/表达式 高价值，extract/格式 低价值 |
| vn-runtime | `script/expr/tests` | 求值/错误/短路/类型 高价值，EvalError Display 低价值 |
| vn-runtime | `script/ast/tests` | 节点契约/源图/路径解析 高价值，getter/容器 低价值 |
| host | `command_executor/tests` | Runtime→Host 契约 高价值，简单转发 低价值 |
| host | `renderer/render_state/tests` | 状态机/inline effect 高价值，默认/getter 低价值 |
| host | `resources/tests` | 资源抽象/错误 高价值，创建类 低价值 |
| host | `save_manager/tests` | 错误恢复/多文件 高价值，格式化 低价值 |
| host | `config/tests` | 校验/load/validate 高价值，默认/serde/Display 低价值 |
| host | `input/tests` | 输入契约/防抖/选项导航 高价值，构造/getter 低价值 |
| host | `manifest/tests` | infer/load 错误/validate 高价值，默认/schema/Display 低价值 |
| host | `app/app_mode/tests` | 导航栈/go_back/switch 高价值，save_load_page/getter 低价值 |
| host | `extensions/tests` | 兼容性/能力/dispatch 高价值，CapabilityId/getter 低价值 |
| host | `ui/layout/tests` | 无效 JSON/hex 回退 高价值，ScaleContext/Display 低价值 |
| host | `ui/screen_defs/tests` | 错误/条件求值/资源回退 高价值，action_parse/schema 低价值；**1 个 undecided**（`condition_parse_cases`）留 mod.rs |
| host | `renderer/scene_transition/tests` | 阶段/skip/midpoint 高价值，构造/getter 低价值 |
| host | `renderer/headless_tests` | build_draw_commands 链路/dim/blur/阈值 高价值，几何/getter/枚举映射 低价值 |

- **验证**：`cargo test -p vn-runtime --lib`（288 通过）、`cargo test -p host --lib`（527 通过），无重复测试。
- **后续**：内联 `#[cfg(test)] mod tests` 的模块（如 `state`、`history`、`save` 等）仍保留原位，仅在收益明确时再迁出；`undecided` 留待后续审计或替换。
- **headless_tests**：已补做分层，由原 `headless_tests.rs` 改为 `headless_tests/mod.rs` + `high_value.rs`（6：build_draw_commands 链路、dim/blur/阈值）+ `low_value.rs`（12：get_choice_rects、get_scale_factor、calculate_draw_rect、screen_size、position_to_preset_name）。

## 验收标准

- 仓库内已有统一的测试分层文档。
- 首批混杂测试入口已出现 `high_value.rs` / `low_value.rs` 子模块。
- `undecided` 明确保留在根测试模块，而不是被随意塞进任一层。
- 后续新增测试可以按本 RFC 直接判断落点。

截至当前执行，上述标准已满足：19 个 companion 测试入口均已完成分层，仅 `screen_defs` 在根模块保留 1 个 undecided；新增测试按“高价值 → high_value，低价值 → low_value，难判定 → 根模块”落点即可。
