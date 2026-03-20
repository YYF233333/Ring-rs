# RFC: 编排层（App Layer）测试基础设施

**状态**：Draft

## 背景

`host/src/app/` 下的编排层模块是引擎的核心粘合层，负责连接 VNRuntime → CommandExecutor → Renderer → AnimationSystem → AudioManager，但目前零测试覆盖：

| 文件 | 行数 | 职责 |
|------|------|------|
| `update/script.rs` | ~343 | 脚本 tick 推进、命令管线执行 |
| `update/modes.rs` | ~349 | PlaybackMode 状态机（Normal/Auto/Skip） |
| `update/scene_transition.rs` | ~33 | 场景过渡控制 |
| `update/mod.rs` | ~75 | update 入口分发 |
| `save.rs` | ~299 | 存档构建与恢复 |
| `script_loader.rs` | ~258 | 脚本加载、预取路径收集 |
| `init.rs` | ~266 | 引擎初始化、脚本检查报告 |

合计约 1600 行代码，全部依赖人工测试和 headless 回放验证。这些模块包含引擎最关键的运行时编排逻辑——PlaybackMode 状态机决定了何时推进对话、何时等待输入；命令管线决定了脚本指令如何逐步转化为渲染效果；存档系统决定了玩家进度能否正确保存与恢复。任何回归都会直接影响终端用户体验，但当前完全没有自动化回归守护。

## 目标

- 为编排层核心逻辑建立可单测的测试通道。
- 优先覆盖高风险路径：PlaybackMode 状态机、命令管线、存档往返。
- 建立可复用的 headless test fixture，降低后续补测试的成本。

## 非目标

- 不追求 100% 覆盖率。
- 不测 GPU、窗口、音频设备、FFmpeg 依赖的路径。
- 不重写编排层架构——测试基础设施应适配现有代码，而非反过来。

## 提案

### 1. 纯函数提取策略

编排层中部分逻辑是纯函数或接近纯函数，可以零架构改动地直接提取并测试：

- **`script_loader.rs`**：`collect_prefetch_paths`、`collect_call_nodes` — 输入 AST 节点，输出路径/节点列表，纯函数，直接可测。
- **`init.rs`**：`report_script_check_results` — 输入诊断结果列表，输出格式化报告，纯函数。
- **`save.rs`**：部分字段构建逻辑（如从 RenderState 提取快照字段）可提取为接受结构体引用、返回数据的纯函数。

纯函数提取是最低风险、最高收益的切入点：不改变生产代码的控制流，不引入新的抽象层，测试本身也最简单直接。

### 2. Headless AppState test fixture

对于无法提取为纯函数的编排逻辑（如 PlaybackMode 推进、tick 管线、存档往返），需要构造一个最小可用的 headless 测试上下文。

**设计原则**：
- 基于已有的 `NullTexture` + `InMemorySource` 构建，复用现有测试基础设施。
- 参考 `command_executor` 测试中的 `TestCtx` 模式，提供 builder API。
- 最小依赖集：`Script` + `VNRuntime` + `CommandExecutor` + `RenderState`。
- 不需要：窗口、GPU 上下文、`AudioManager`（用 stub 替代）、`EguiContext`。

**Builder 模式示例**：

```rust
let ctx = HeadlessTestCtx::builder()
    .with_script("start", "你好世界\n@bg park.png")
    .build();
```

Builder 只暴露测试场景需要的配置项（脚本内容、初始变量等），内部自动构建完整的 AppState 所需依赖。这样当 AppState 结构变更时，只需修改 builder 内部，而非每个测试用例。

### 3. 首批测试场景

按风险等级排序，首批覆盖以下场景：

| 优先级 | 场景 | 类型 | 所在模块 |
|--------|------|------|----------|
| P0 | PlaybackMode（Normal/Auto/Skip）推进时序 | fixture | `update/modes.rs` |
| P0 | `build_save_data` → `restore_from_save_data` 往返一致性 | fixture | `save.rs` |
| P0 | `collect_prefetch_paths` / `collect_call_nodes` 基础功能 | 纯函数 | `script_loader.rs` |
| P1 | `run_script_tick` 命令管线端到端（单 tick 产出正确 Command 序列） | fixture | `update/script.rs` |
| P1 | `report_script_check_results` 格式化输出 | 纯函数 | `init.rs` |
| P1 | `skip_all_active_effects` 收敛正确性（所有动画归位到终态） | fixture | `update/script.rs` |

## 风险

- **fixture 维护成本**：AppState 结构变更时 fixture 需同步更新，若 AppState 频繁变动会导致测试代码大量流转。
- **与生产代码耦合**：fixture 过度依赖内部结构会阻碍后续重构——测试本应守护行为而非实现细节。
- **部分逻辑难以脱离外部系统**：音频淡入淡出时序、FFmpeg 解码等与硬件绑定的路径无法在 headless 环境中测试。

## 风险缓解

- **Builder 模式隔离变更**：fixture 采用 builder 模式，只暴露必要配置项。AppState 内部结构变更时，修改点集中在 builder 实现，而非散落在每个测试用例中。
- **纯函数优先**：能提取为纯函数的逻辑优先用纯函数测试，fixture 仅用于无法提取的集成逻辑。这样最大化了测试的稳定性和可维护性。
- **明确测试分类**：每个测试明确标注属于"纯函数单测"还是"编排层集成测试"，避免混淆测试的职责边界和失败信号含义。
- **硬件依赖路径不测**：音频、GPU、FFmpeg 相关路径明确排除在测试范围之外，用 stub/null 实现替代。

## 迁移计划

分四个阶段，每阶段独立可交付：

1. **纯函数提取与测试**（零架构改动）
   - 从 `script_loader.rs`、`init.rs`、`save.rs` 提取纯函数。
   - 为每个纯函数编写单元测试，放入对应模块的 `tests/high_value.rs`。
   - 验证：`cargo test -p host --lib` 新增测试全部通过。

2. **HeadlessTestCtx fixture 设计与实现**
   - 在 `host/src/app/` 下建立测试 fixture 模块。
   - 实现 builder API，确保可以用最小配置构造可用的测试上下文。
   - 验证：fixture 可成功构造并驱动一次 `tick`。

3. **首批 fixture 测试**
   - PlaybackMode 三种模式的推进逻辑测试。
   - `build_save_data` → `restore_from_save_data` 往返测试。
   - `skip_all_active_effects` 收敛测试。
   - 验证：所有首批测试通过。

4. **逐步扩展**
   - 按风险等级补充更多测试场景（如错误恢复、边界条件）。
   - 根据实际开发节奏按需推进，不设硬性时间表。

## 验收标准

- 至少 3 个纯函数被提取并有对应单测（`collect_prefetch_paths`、`collect_call_nodes`、`report_script_check_results`）。
- `HeadlessTestCtx` 可成功构造并驱动一次 tick，不依赖 GPU/窗口/音频设备。
- PlaybackMode 三种模式（Normal、Auto、Skip）的基础推进逻辑有测试覆盖。
- `build_save_data` → `restore_from_save_data` 往返测试通过，关键字段一致。
