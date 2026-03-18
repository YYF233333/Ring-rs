# RFC: 结构化事件流调试基础设施

## 元信息

- 编号：RFC-018
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-18
- 相关范围：`vn-runtime`（engine、executor、state）、`host`（app、command_executor、input、audio、renderer）
- 前置：无

---

## 1. 背景

当前 Ring-rs 在联调与集成问题排查时，存在一个高频痛点：AI 模型无法直接操作 GUI 窗口，只能依赖用户手动复现问题。现有调试流程通常是：

1. 模型在怀疑位置临时插入 `tracing::debug!` 埋点；
2. 用户复现问题并发送日志；
3. 若信息不足，模型再猜测其他位置补埋点，重复 1–2 步。

该流程存在以下问题：

- **多轮往返**：每次猜测错误都会增加一轮“改代码 → 运行 → 复现 → 发日志”的反馈周期；
- **埋点临时性**：问题解决后埋点往往被移除，同类问题再次出现时需重新猜测；
- **上下文不完整**：`tracing` 输出为人类可读的文本，难以被模型或工具做结构化分析；
- **边界不清晰**：埋点位置依赖开发者经验，容易遗漏关键 pipeline 边界。

Ring-rs 的核心数据流为：

```text
脚本文本 → Parser → AST (ScriptNode) → Executor → Command → CommandExecutor (host) → RenderState/Audio/Effects
```

在 vn-runtime 与 host 的边界、以及 host 内部各子系统（渲染、音频、输入）的边界，存在明确的“事件发生点”。若在这些边界处**永久**输出结构化、机器可解析的事件，则：

- 模型无需每次猜测埋点位置，可直接分析事件流；
- 事件格式统一，便于工具链（如 jq、自定义分析脚本）处理；
- 与 `tracing` 并存：tracing 用于人类可读的详细日志，事件流用于结构化诊断。

---

## 2. 目标与非目标

### 2.1 目标

- 在关键 pipeline 边界建立**永久**的结构化事件输出机制，替代临时 `tracing::debug!` 埋点。
- 事件采用 **JSON Lines** 格式写入文件，便于流式解析与 AI/工具分析。
- 事件流**默认关闭**，通过配置启用；启用时对稳态帧的额外开销可忽略。
- 覆盖以下边界：脚本 tick、Command 产出/执行、状态变更、输入处理、过渡/音频事件。
- 与现有 `tracing` 系统**共存**：tracing 负责人类可读日志，事件流负责机器可解析诊断。

### 2.2 非目标

- **不替代 tracing**：tracing 仍用于开发时的详细调试输出，两者职责不同。
- **不实现遥测/分析系统**：无网络上报、无用户行为统计。
- **不实现网络流式输出**：仅文件输出，不支持 WebSocket/HTTP 推送。
- **不实现运行时事件过滤/订阅**：启用即输出全部事件，不做按类型/字段的动态过滤。
- **不追求帧级细粒度**：事件仅在状态转换边界触发，不每帧输出稳态信息。

---

## 3. 方案设计

### 3.1 事件类型定义

在 pipeline 边界定义以下事件类型，每种对应一个明确的语义：

| 事件类型 | 触发位置 | 典型 data 字段 |
|---------|----------|----------------|
| `script_tick` | Engine tick 执行后 | `node_index`, `commands_count`, `waiting_reason` |
| `command_produced` | Executor 产出 Command 时 | `variant`, 关键字段摘要（如 `speaker`, `text_len`, `path`） |
| `command_executed` | CommandExecutor 执行 Command 后 | `variant`, `result` |
| `state_changed` | 关键状态字段变更时 | `field`, `from`, `to` |
| `input_received` | InputManager 产生 RuntimeInput 时 | `variant`（Click / ChoiceSelected / Signal） |
| `transition_update` | 场景/背景过渡进度更新时 | `type`, `phase`, `progress` |
| `audio_event` | 音频动作执行时 | `action`（play/stop/duck BGM/SE/voice）, `path`, `volume` |

### 3.2 事件格式

每条事件为单行 JSON（JSON Lines），便于流式读取与 `jq` 等工具处理：

```json
{"ts_ms": 12345, "event": "command_produced", "data": {"variant": "ShowText", "speaker": "Alice", "text_len": 42}}
{"ts_ms": 12350, "event": "command_executed", "data": {"variant": "ShowText", "result": "ok"}}
{"ts_ms": 12350, "event": "state_changed", "data": {"field": "waiting", "from": "None", "to": "UserInput"}}
{"ts_ms": 12355, "event": "input_received", "data": {"variant": "Click"}}
{"ts_ms": 12360, "event": "transition_update", "data": {"type": "dissolve", "phase": "in_progress", "progress": 0.5}}
{"ts_ms": 12365, "event": "audio_event", "data": {"action": "play_bgm", "path": "bgm/title.ogg", "volume": 0.8}}
```

字段约定：

- `ts_ms`：自进程启动或会话开始的毫秒时间戳（用于排序与关联）。
- `event`：事件类型字符串。
- `data`：与事件类型相关的结构化对象，仅包含诊断所需的关键字段，不包含完整业务数据（如不输出完整对话文本，仅 `text_len`）。

### 3.3 实现架构

#### 3.3.1 核心类型

```rust
/// 可序列化的事件枚举，按类型携带对应 data
#[derive(Serialize)]
pub enum EngineEvent {
    ScriptTick { node_index: usize, commands_count: usize, waiting_reason: String },
    CommandProduced { variant: String, /* 各 variant 关键字段 */ },
    CommandExecuted { variant: String, result: String },
    StateChanged { field: String, from: String, to: String },
    InputReceived { variant: String },
    TransitionUpdate { transition_type: String, phase: String, progress: f32 },
    AudioEvent { action: String, path: Option<String>, volume: Option<f32> },
}

/// 事件流写入器：启用时写入文件，禁用时 no-op
pub struct EventStream {
    enabled: bool,
    writer: Option<BufWriter<File>>,
    start_ms: u64,
}
```

#### 3.3.2 传递方式

- **Option A（推荐）**：`EventStream` 作为 `AppState` 或 `GameContext` 的字段，在 `run_script_tick`、`CommandExecutor::execute`、`InputManager::update` 等路径以 `&EventStream` 或 `&mut EventStream` 传递。
- **Option B**：类似 `tracing` 的 thread-local 或全局 subscriber，通过 `EventStream::current()` 获取。首版建议 Option A，依赖注入更清晰、可测试性更好。

#### 3.3.3 埋点位置（约 10–15 处）

| 模块 | 位置 | 事件类型 |
|------|------|----------|
| `vn-runtime::engine` | `tick()` 返回后 | `script_tick` |
| `vn-runtime::executor` | 每产出一个 Command | `command_produced` |
| `host::command_executor` | `execute()` 每个分支返回后 | `command_executed` |
| `host::app` / `vn-runtime::state` | `waiting_reason` 变更时 | `state_changed` |
| `host::app` / `vn-runtime::state` | `background`、`dialogue` 等关键字段变更时 | `state_changed` |
| `host::input` | `InputManager::update` 返回 `Some(input)` 时 | `input_received` |
| `host::renderer` | 场景/背景过渡进度更新时 | `transition_update` |
| `host::command_executor::audio` | 各音频命令执行时 | `audio_event` |

**约束**：埋点仅位于模块边界，不进入 tight loop（如每帧渲染、打字机逐字输出）。事件在**状态转换**时触发，稳态帧无事件输出。

#### 3.3.4 禁用时的性能

- 启用检查：`if !event_stream.is_enabled() { return; }`，单次 bool 判断。
- 禁用时：不分配、不序列化、不写文件，开销可忽略。
- 启用时：仅在有事件时进行 JSON 序列化 + 文件写入，频率与状态转换一致，非每帧。

### 3.4 配置扩展

在 `DebugConfig` 中新增字段，与现有 `log_level`、`log_file` 并列：

```rust
// host/src/config/mod.rs - DebugConfig 扩展

/// 事件流输出文件路径（None 时禁用事件流）
pub event_stream_file: Option<String>,
```

或采用更显式的双字段形式：

```rust
/// 是否启用结构化事件流
pub enable_event_stream: bool,
/// 事件流输出文件路径（enable_event_stream 为 true 时必填）
pub event_stream_file: Option<String>,
```

**推荐**：使用 `event_stream_file: Option<String>` 单字段——`Some(path)` 即启用并写入该路径，`None` 即禁用。与 `log_file` 的语义一致，配置更简洁。

**config.json 示例**：

```json
{
  "debug": {
    "script_check": true,
    "log_level": "info",
    "log_file": null,
    "event_stream_file": "events.jsonl"
  }
}
```

默认值：`event_stream_file: None`，即默认禁用。

### 3.5 与 tracing 的关系

| 维度 | tracing | 事件流 |
|------|---------|--------|
| 受众 | 人类开发者 | 模型、工具、脚本 |
| 格式 | 人类可读文本 | JSON Lines |
| 粒度 | 可任意细（含 debug/info 等级） | 仅 pipeline 边界 |
| 用途 | 开发时详细排障 | 集成问题结构化诊断 |
| 配置 | `log_level`、`log_file` | `event_stream_file` |

两者**并存**，不互相替代。开发者在需要时可同时开启 tracing 与事件流，分别用于不同分析场景。

---

## 4. 影响范围

| 模块 | 改动类型 | 风险 |
|------|----------|------|
| `host::config` | `DebugConfig` 新增 `event_stream_file` | 低：默认 `None`，反序列化兼容 |
| `host::app` / `host::init` | 构造 `EventStream`，传入 `AppState` | 低 |
| `vn-runtime::engine` | 接收 `&EventStream`，tick 后 emit | 低：需 engine 接口扩展 |
| `vn-runtime::executor` | 接收 `&EventStream`，产出 Command 时 emit | 低：executor 当前无 IO，需通过参数传入 |
| `host::command_executor` | 接收 `&EventStream`，execute 后 emit | 低 |
| `host::input` | 接收 `&EventStream`，update 返回 Some 时 emit | 低 |
| `host::renderer`（过渡相关） | 接收 `&EventStream`，进度更新时 emit | 低 |
| `host::command_executor::audio` | 接收 `&EventStream`，各音频命令 emit | 低 |

**跨 crate 注意**：`vn-runtime` 为纯逻辑 crate，不应依赖 `std::fs` 等 IO。因此 `EventStream` 的**定义与实现**应放在 `host`，`vn-runtime` 仅接收一个 trait 或回调：

```rust
// vn-runtime 侧：定义最小接口，避免 IO 依赖
pub trait EventSink: Send + Sync {
    fn emit(&self, event: &EngineEvent);
}

// host 侧：实现 EventSink，内部持有文件写入逻辑
impl EventSink for EventStream { ... }
```

或更简单：`EngineEvent` 定义在 `vn-runtime`（可序列化、无 IO），`EventStream` 完全在 `host`，`engine::tick` 的调用方（host）在 tick 返回后根据返回值 emit 事件。这样 vn-runtime 无需新增依赖，仅需在 `ExecuteResult` 或类似结构中暴露足够信息供 host 构造事件。

**推荐**：`EngineEvent` 与 `EventStream` 均放在 `host` 的 `debug` 或 `event_stream` 子模块。vn-runtime 的 engine/executor 不直接 emit，而是由 host 的 `run_script_tick` 在调用 engine 前后、以及处理 Command 时统一 emit。这样 vn-runtime 零改动，所有埋点在 host 与 host 对 runtime 的调用边界完成。

---

## 5. 迁移计划

### Phase 1：基础设施（1–2 天）

1. 在 `host` 中新增 `event_stream` 模块，定义 `EngineEvent` 枚举与 `EventStream` 结构。
2. 扩展 `DebugConfig`：`event_stream_file: Option<String>`，默认 `None`。
3. 在 `main.rs` / `init` 中，根据配置创建 `EventStream` 并传入 `AppState`。
4. 实现 `EventStream::emit()`，支持启用时 JSON Lines 写入、禁用时 no-op。
5. 单元测试：启用时写入临时文件，验证格式与内容；禁用时验证无写入。

### Phase 2：核心事件埋点（2–3 天）

1. 在 `run_script_tick` 中 emit `script_tick`（基于 tick 返回值）。
2. 在 Command 处理循环中 emit `command_produced`（host 侧从 `ExecuteResult` 的 commands 迭代）。
3. 在 `CommandExecutor::execute` 各分支 emit `command_executed`。
4. 在 `waiting_reason` 变更路径 emit `state_changed`。
5. 在 `InputManager::update` 返回 `Some` 时 emit `input_received`。

### Phase 3：扩展事件（1–2 天）

1. 在场景/背景过渡逻辑中 emit `transition_update`。
2. 在 `command_executor::audio` 各命令中 emit `audio_event`。
3. 视需要补充 `state_changed` 的其他字段（如 `background`、`dialogue`）。

### Phase 4：文档与验收

1. 更新 `config_guide.md`，说明 `event_stream_file` 的用法。
2. 在 `docs/` 中新增 `event_stream_spec.md`（可选），描述各事件类型的 data 字段规范。
3. 完成验收标准中的各项检查。

---

## 6. 验收标准

- [ ] `DebugConfig` 新增 `event_stream_file: Option<String>`，默认 `None`，与现有配置反序列化兼容。
- [ ] 当 `event_stream_file` 为 `Some(path)` 时，事件写入该路径；为 `None` 时，所有 emit 调用为 no-op。
- [ ] 至少覆盖以下事件类型：`script_tick`、`command_produced`、`command_executed`、`state_changed`（含 `waiting`）、`input_received`。
- [ ] 输出格式为 JSON Lines，每行可被 `serde_json::from_str` 解析。
- [ ] 禁用时，`cargo bench` 或等价性能测试显示无明显开销（可设定阈值，如 <0.5% 帧时间增加）。
- [ ] `config_guide.md` 已更新，包含 `event_stream_file` 的说明与示例。
- [ ] vn-runtime 保持无 IO 依赖，`EventStream` 与 emit 逻辑均在 host 内。
