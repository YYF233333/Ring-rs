# RFC: 输入录制与 AI 自动调试管线

## 元信息

- 编号：RFC-016
- 状态：Accepted
- 作者：Ring-rs 开发组
- 日期：2026-03-18（修订：2026-03-19）
- 实现完成：2026-03-19
- 相关范围：`host/src/input`、`host/src/app`、`host/src/config`、`vn-runtime` 输入消费边界
- 前置：RFC-018（结构化事件流）、RFC-019（Headless 测试模式）

### 实现摘要

- `host/src/input/recording.rs`：InputEvent、RecordingMeta、RecordingBuffer、RecordingExporter、InputReplayer；WindowEvent→InputEvent 转换。
- `InputManager`：recording_buffer、elapsed_ms；process_input_event、inject_replay_events、recording_snapshot、enable_recording。
- `config.json` / DebugConfig：recording_buffer_size_mb、recording_output_dir。
- F8 热键导出在 app 层（export_recording），录制文件为 JSON Lines，首行 Meta，后续 Event。
- 使用说明见 `docs/headless_guide.md`。

---

## 1. 背景

当前 Ring-rs 的 GUI 类问题（如 egui 覆盖层命中、点击推进、等待状态切换）调试流程：

1. 用户游玩时遇到 bug；
2. 用文字向 AI 描述复现步骤（"我点了这里，然后……"）；
3. AI 猜测原因并修改代码；
4. 用户重新启动、手工复现、反馈结果；
5. 若猜错，重复 2–4。

痛点在于**步骤 2 和 4**：

- **步骤 2 信息丢失**：口述无法精确传递点击坐标、时序间隔、操作顺序，时序相关 bug 几乎不可能通过文字描述复现；
- **步骤 4 人工成本**：每次修改后都需要用户手动重跑游戏验证，AI 无法独立闭环。

理想流程应该是：

1. 用户正常游玩，**引擎后台始终录制输入**（零感知）；
2. 遇到 bug 时，按热键保存录制文件（或引擎崩溃时自动保存）；
3. 用户将录制文件交给 AI；
4. AI 在 headless 模式下（RFC-019）回放录制，结合结构化事件流（RFC-018）自动分析定位问题；
5. AI 修复后再次回放验证，**无需用户介入**。

本 RFC 的核心目的是打通这条**"人类游玩 → 录制导出 → AI 自动重现与调试"**管线。

---

## 2. 目标与非目标

### 2.1 目标

- **G1 后台录制**：引擎运行期间后台持续录制输入事件，对用户游玩体验零干扰。
- **G2 一键导出**：提供热键或崩溃自动保存机制，将当前缓冲区中的最近输入历史导出为文件。
- **G3 AI 可消费的格式**：录制文件采用语义化事件 + JSON Lines 格式，AI 可直接解读操作含义（而非平台原始事件码）。
- **G4 headless 回放**：录制文件可在 RFC-019 headless 模式下作为**唯一输入源**驱动引擎回放，重现用户操作路径。
- **G5 与事件流联动**：回放时配合 RFC-018 结构化事件流，产出机器可解析的诊断数据，支撑 AI 自动 root cause 分析。
- **G6 确定性保障**：回放在相同输入序列下，`vn-runtime` 脚本逻辑层行为可重复。

### 2.2 非目标

- 不追求像素级渲染帧一致性（headless 模式不渲染）。
- 不在本 RFC 中引入脚本化断言 DSL 或完整测试框架。
- 不修改 `vn-runtime` 业务语义，仅在 host 输入源层做注入与替换。
- 不在首期支持跨分辨率自动重映射到 UI 逻辑元素（仅坐标比例缩放）。
- 不以"人类手动回放"作为主要使用场景——回放主要服务于 AI/headless 自动化。

---

## 3. 方案设计

### 3.1 核心工作流

```text
┌─────────────────────────────────────────────────────┐
│  人类游玩阶段                                        │
│                                                     │
│  引擎启动 → InputManager 后台录制到环形缓冲区        │
│         → 用户正常游玩（无感知）                     │
│         → 遇到 bug，按 F8 保存录制文件               │
│         → 或引擎 panic 时自动 dump 缓冲区到文件      │
└─────────────────┬───────────────────────────────────┘
                  │ recording.jsonl
                  ▼
┌─────────────────────────────────────────────────────┐
│  AI 调试阶段                                        │
│                                                     │
│  AI 读取 recording.jsonl，理解用户操作序列           │
│  → 启动 headless 模式 (RFC-019)                     │
│    --replay-input=recording.jsonl                   │
│    （事件流默认启用，可显式覆盖输出路径）            │
│  → 引擎无窗口回放，输出结构化事件流 (RFC-018)        │
│  → AI 分析事件流定位问题                             │
│  → 修复代码后再次回放验证                            │
│  → 闭环，无需用户手动复现                            │
└─────────────────────────────────────────────────────┘
```

### 3.2 录制层级决策

候选方案：

- **方案 A：录制原始 `WindowEvent`**
  - 优点：最忠实底层输入；
  - 缺点：数据体积大、平台细节重、与 `winit` 事件结构强耦合，AI 不可读。
- **方案 B：仅录制 `RuntimeInput`**
  - 优点：最精简，直接贴近 `vn-runtime`；
  - 缺点：丢失鼠标位置与时序细节，难覆盖 UI 命中/悬停类问题。
- **推荐方案：录制 `InputManager` 语义中间层事件**
  - 在 `process_event` 将原始事件映射为 `InputEvent`（按键按下/释放、鼠标按下/释放、移动、滚轮）；
  - 事件携带人类可读上下文（时间戳、坐标、按键名），AI 可直接理解"用户在第 3 秒点击了 (320, 480)"；
  - 在回放时注入到 `InputManager` 同层逻辑，保持与真实路径一致。

### 3.3 数据模型与文件格式

文件格式：JSON Lines（`.jsonl`），UTF-8，每行一条事件。

**文件头（首行元数据）**：

```json
{"meta": {"version": 1, "logical_width": 1280, "logical_height": 720, "dpi_scale": 1.25, "engine_version": "0.1.0", "recorded_at": "2026-03-19T14:30:00Z", "duration_ms": 45000, "entry_script": "main.vn"}}
```

**事件行**：

```json
{"t_ms": 0, "event": {"type": "MouseMove", "x": 640, "y": 360}}
{"t_ms": 1234, "event": {"type": "MousePress", "button": "Left", "x": 320, "y": 480}}
{"t_ms": 1300, "event": {"type": "MouseRelease", "button": "Left", "x": 320, "y": 480}}
{"t_ms": 2345, "event": {"type": "KeyPress", "key": "Space"}}
{"t_ms": 2400, "event": {"type": "KeyRelease", "key": "Space"}}
```

`InputEvent` 首期变体：

- `KeyPress { key }` / `KeyRelease { key }`
- `MouseMove { x, y }`
- `MousePress { button, x, y }` / `MouseRelease { button, x, y }`
- `MouseWheel { delta_x, delta_y }`

AI 可直接阅读此文件理解"用户做了什么"，无需额外工具转换。

其中元数据头的职责应限定为 **可读性增强与运行前校验**，而不是提供第二份启动配置。至少应包含：

- 录制时逻辑尺寸；
- 录制格式版本；
- 用于人类阅读的场景/脚本标识（可选，只作提示，不参与启动决策）；
- 录制总时长。

headless 的真实启动参数仍应来自 `config` / CLI 这一唯一事实源。replay 元数据只用于：

- 帮助人类和 AI 快速理解这份录制大致对应什么场景；
- 在启动前做 guard 校验，发现 replay 与当前运行环境错配时尽早失败。

可选的 guard 示例：

- replay 记录的逻辑尺寸与当前运行配置严重不一致；
- replay 记录的场景/脚本标识与当前启动目标不一致；
- replay 格式版本超出当前程序支持范围。

### 3.4 环形缓冲区录制

后台录制采用**环形缓冲区**（Ring Buffer），按大小控制：

- 默认缓冲区上限 **1 MB**（可通过配置调整）；
- 语义事件体积小（单条约 50–100 bytes），1 MB 约容纳 10000–20000 条事件，正常操作频率下覆盖 5–15 分钟；
- 超出上限时淘汰最早的事件（FIFO）；
- 缓冲区在 `InputManager` 中维护，不涉及磁盘 I/O，对帧率无影响；
- 仅在用户触发导出（热键/崩溃）时才一次性写盘。

按大小而非时间控制的理由：

- 大小上限直接约束内存占用，行为可预测；
- 不同场景输入密度差异大（快速点击 vs 长时间等待），时间窗口无法稳定控制内存；
- 1 MB 默认值在绝大多数场景下足够覆盖"从发现 bug 到按 F8"之间的操作历史。

### 3.5 导出触发机制

| 触发方式 | 场景 | 行为 |
|----------|------|------|
| 热键 `F8` | 用户遇到 bug，手动触发 | 将缓冲区内容写入 `recordings/` 目录，文件名含时间戳 |
| Panic hook | 引擎 panic 崩溃 | 在 panic handler 中尝试 dump 缓冲区（尽力而为） |

导出路径默认：`recordings/input_{timestamp}.jsonl`

导出时在日志中输出文件路径和事件数，方便用户找到文件：

```text
[INFO] Input recording saved: recordings/input_20260319_143000.jsonl (1234 events, 45.0s)
```

### 3.6 模块划分与职责

新增模块：`host/src/input/recording.rs`

- **`InputEvent`**：语义化输入事件枚举，实现 `Serialize`/`Deserialize`。
- **`RecordingBuffer`**：环形缓冲区，接收 `(t_ms, InputEvent)`，维护时间窗口裁剪。
- **`RecordingExporter`**：将缓冲区快照写出为 `.jsonl`，附加元数据头。
- **`InputReplayer`**：读取 `.jsonl` 文件，按时间戳吐出事件，供 headless 回放。

`InputManager` 扩展：

- `recording_buffer: Option<RecordingBuffer>`（游玩时启用）
- `replayer: Option<InputReplayer>`（headless 回放时启用）
- 二者互斥：正常游玩模式下有 buffer 无 replayer，headless 回放下有 replayer 无 buffer。

### 3.7 帧循环集成

**正常游玩模式（录制）**：

1. `process_event(WindowEvent)` → 将原始事件转为 `InputEvent` → 同时写入 `RecordingBuffer`；
2. `begin_frame(dt)` → 推进时间；
3. `update(waiting, dt)` → 正常生成 `RuntimeInput`；
4. `end_frame()` → 清理逐帧状态。

**Headless 回放模式**：

1. 无真实 `WindowEvent`（headless 无窗口）；
2. `begin_frame(dt)` → 推进时间（固定 dt 模式，确保确定性）；
3. `replayer` 取出 `t_ms <= elapsed_ms` 的事件 → 调用统一内部入口 `process_input_event(InputEvent)`；
4. `update(waiting, dt)` → 基于注入的输入状态生成 `RuntimeInput`；
5. `end_frame()` → 清理逐帧状态。

关键：真实输入与回放输入统一经过同一内部处理函数 `process_input_event`，避免逻辑分叉。

### 3.8 时间基准与确定性策略

回放调度采用"时间戳对齐"：

- 每帧累加 elapsed；
- 注入所有 `event.t_ms <= elapsed_ms` 的事件；
- 若某帧跨度较大，允许一次注入多条事件。

`dt` 策略：

- **headless 回放默认固定 `dt`**（16ms）：确保确定性，AI 多次回放结果一致；
- 可配置实时 `dt`：贴近真实运行节奏，但不保证跨机器一致。

确定性边界：

- `vn-runtime` 在相同输入序列 + 固定 dt 下，脚本行为严格确定；
- 渲染帧率不作为一致性判据（headless 不渲染）；
- 音频时序不纳入首期一致性目标。

### 3.9 坐标与窗口尺寸差异

录制时保存逻辑尺寸元数据。回放时：

- 默认按比例映射：`x' = x * replay_width / record_width`
- headless 模式下以录制时的逻辑尺寸作为虚拟窗口尺寸（最简单、最稳定）；
- 若尺寸差异超阈值（> 30%），打印警告日志。

### 3.10 与 RFC-018/019 的协同

本 RFC 是三者联动的关键环节：

```text
RFC-016 (本 RFC)           RFC-019                  RFC-018
输入录制文件      →    headless 回放引擎    →    结构化事件流输出
recording.jsonl        无窗口运行引擎            events.jsonl
                                                    ↓
                                              AI 读取分析
                                              定位 root cause
```

在该联动方案中，`recording.jsonl` 不只是“可回放文件”，而是 **headless 的启动前提**：

- 启动 `--headless` 时必须提供 `--replay-input=<path>`；
- headless 不提供 Auto-advance、默认选项等其他输入路径；
- replay 文件中的元数据负责提供可读提示与 guard 校验信息，事件体负责提供逐步输入序列；真正的启动配置仍来自 `config` / CLI。

**典型 AI 调试命令**：

```bash
ring-rs --headless --replay-input=recording.jsonl --exit-on=replay-end
```

AI 拿到 headless 默认生成或显式指定的 `events.jsonl` 后可以：

- 对比正常流与异常流的事件差异；
- 定位 Command 执行失败点；
- 检查状态变更是否符合预期；
- 修复后重新回放验证。

### 3.11 错误处理与可观测性

外部不受信路径返回 `Result` 并附上下文：

- `ReplayLoadError`（文件不存在、JSON 解析失败、版本不兼容）
- `RecordExportError`（路径不可写、磁盘失败）

内部不变量（如回放游标越界）使用 `debug_assert!`/`unreachable!` 保护。

日志：

- 录制导出：文件路径、事件总数、时间跨度；
- 回放开始/结束：总事件数、已注入数、总耗时；
- 回放中断原因（若有）。

### 3.12 配置

`DebugConfig` 新增字段：

```rust
pub struct DebugConfig {
    /// 后台录制缓冲区大小上限（MB），设为 0 禁用录制。默认 1MB。
    pub recording_buffer_size_mb: u32,
    /// 录制导出目录。默认 "recordings/"。
    pub recording_output_dir: Option<String>,
    /// headless 模式的回放输入文件路径（headless 时必填）。
    pub replay_input: Option<String>,
}
```

命令行参数映射：

- `--replay-input=<path>`：headless 模式必填，指定唯一输入源
- `--recording-buffer-kb=<size>`：覆盖缓冲区大小（单位 KB）
- `--no-recording`：禁用后台录制

优先级：CLI > 配置文件 > 默认值。

---

## 4. 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host/src/input/recording.rs`（新增） | `InputEvent`、`RecordingBuffer`、`RecordingExporter`、`InputReplayer` | 中（新子系统，需测试覆盖） |
| `host/src/input/mod.rs` | `InputManager` 增加 buffer/replayer 集成与统一注入入口 | 中（输入主路径，需回归） |
| `host/src/app/mod.rs` | 启动期按配置初始化录制/回放，panic hook 注册 | 低-中 |
| `host/src/config` | `DebugConfig` 新字段 | 低（默认值无破坏性） |
| `vn-runtime` | 不改 | 无 |

---

## 5. 迁移计划

### 阶段 A：数据结构与文件读写

- 定义 `InputEvent` 枚举与 serde 逻辑；
- 实现 `RecordingBuffer`（环形缓冲区）+ 单元测试；
- 实现 `RecordingExporter`（`.jsonl` 写出）+ 单元测试；
- 实现 `InputReplayer`（`.jsonl` 读取与时间戳调度）+ 单元测试。

### 阶段 B：InputManager 接入（录制侧）

- 提取统一内部入口 `process_input_event(InputEvent)`；
- 在 `process_event` 中同时写入 `RecordingBuffer`；
- 实现 F8 热键触发导出。

### 阶段 C：Headless 回放接入（依赖 RFC-019）

- `InputManager` 支持 replayer 注入；
- headless 模式启动时根据 `--replay-input` 初始化 replayer；
- 联调固定 dt + 事件流输出。

### 阶段 D：Panic dump 与配置集成

- 注册 panic hook，尝试 dump 录制缓冲区；
- `DebugConfig` 新增字段与 CLI 映射；
- 补充使用说明文档。

兼容性：

- 新字段均为 `Option` 或有默认值，旧配置文件无需修改；
- 录制文件包含版本号，后续格式演进可向后兼容。

---

## 6. 验收标准

- [ ] 引擎正常运行时后台持续录制输入到环形缓冲区，对帧率无可感知影响。
- [ ] 按 F8 热键可导出缓冲区为合法 `.jsonl` 文件，含元数据头与时间戳语义事件。
- [ ] 导出的 `.jsonl` 文件可被人类和 AI 直接阅读理解操作含义。
- [ ] 在 headless 模式下通过 `--replay-input` 加载录制文件，且 replay 是唯一输入源，可在无窗口/无人工操作下驱动脚本推进。
- [ ] headless 回放 + 固定 dt 下，同一录制文件多次回放产生的 `vn-runtime` 状态转移序列一致。
- [ ] 回放时 headless 默认同时输出结构化事件流（RFC-018），形成完整调试管线。
- [ ] 文件解析/写入失败时提供结构化错误与上下文，不因单次失败导致应用崩溃。
- [ ] 引擎 panic 时尝试 dump 缓冲区内容到文件（尽力而为，不保证 100% 成功）。
- [ ] 为新模块补充单元测试，并通过 `cargo check-all`。
- [ ] 文档更新：RFC 索引项、调试管线使用说明。
