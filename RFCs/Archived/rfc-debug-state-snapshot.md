# RFC: 调试状态快照热键（Debug State Snapshot Hotkey）

## 元信息

- 编号：RFC-015
- 状态：Superseded（旧 host 已退役；debug server HTTP API + MCP 提供替代方案）
- 作者：Ring-rs 开发组
- 日期：2026-03-18
- 相关范围：`host::app`、`host::input`、`host::config`、`host::renderer::render_state`、`host::audio`、`vn-runtime::state`
- 前置：无

---

## 1. 背景

当前 Ring-rs 的联调与运行时排障存在一个高频痛点：模型无法直接操作 GUI 窗口，很多问题只能由用户手动复现。  
现有方式通常依赖临时日志埋点（多轮改代码 -> 运行 -> 再补日志），反馈链路长、上下文不完整，尤其在以下场景中效率很低：

- 视觉状态与运行时状态错位（例如文本显示与脚本节点不一致）。
- 输入竞争问题（例如跳过、自动播放、点击推进的状态切换异常）。
- 渲染/音频/脚本三方交互导致的偶发错误（时序相关）。

需要一个“一键冻结现场”的机制：用户在异常瞬间按热键，系统立即导出完整但可序列化的引擎状态快照，以 JSON 文件形式交给模型分析，减少重复埋点与复现成本。

---

## 2. 目标与非目标

### 2.1 目标

- 提供调试热键（默认 `F10`）触发状态快照导出，且仅在调试配置开启时生效。
- 输出结构化 JSON（`serde_json::to_string_pretty`），文件名含时间戳，支持配置输出目录。
- 覆盖关键状态域：`RuntimeState`、`RenderState`、音频播放状态、`AppMode/NavigationStack`、`PlaybackMode`、帧时间信息。
- 明确不可直接序列化对象（GPU 资源、trait object）的降级策略，输出可读的描述性信息而非原始资源。
- 方案可增量落地，先实现核心快照，再扩展字段，不阻塞主循环稳定性。

### 2.2 非目标

- 不实现“状态回放/状态恢复”（snapshot restore）。
- 不导出原始 GPU 纹理、音频 buffer、二进制资源内容。
- 不将该机制作为正式用户功能暴露到发布版 UI（仅调试入口）。
- 不在本 RFC 内引入远程上传、自动 issue 创建、云端存储等能力。

---

## 3. 方案设计

### 3.1 配置模型扩展（`DebugConfig`）

在 `host/src/config/` 的 `DebugConfig` 中新增字段：

- `enable_debug_snapshot: bool`  
  控制是否启用快照功能；`false` 时即使按下热键也不执行导出。
- `snapshot_output_dir: Option<String>`  
  快照输出目录；`None` 时使用默认目录（建议 `./debug-snapshots` 或与日志目录对齐）。
- （可选扩展）`snapshot_hotkey: Option<String>` 或等价键位配置  
  默认 `F10`，后续允许配置化。若首版不做可配置键位，固定 `F10` 并保留字段扩展位。

默认值策略：

- 开发配置可默认启用（便于联调）；发布配置默认关闭。
- 输出目录不存在时自动创建，失败则记录错误并不中断主循环。

### 3.2 触发路径与输入集成

集成点放在 `app/update` 的输入处理路径：

1. `InputManager` 将 `winit` 事件转换为内部输入状态。
2. 在每帧更新中检测“快照热键本帧按下”（edge-triggered，而非按住重复触发）。
3. 若 `debug.enable_debug_snapshot == true`，调用 `AppState` 快照捕获流程。

触发约束：

- 仅在运行态（如 `InGame`）强依赖完整上下文时导出；若在 `MainMenu` 等模式触发，也允许导出但字段可为空/简化。
- 防抖：同一帧最多导出一次，避免输入重复上报造成多文件刷写。

### 3.3 快照数据结构设计

新增可序列化快照根结构（示意）：

- `DebugSnapshot`（`Serialize`）
  - `meta`: 版本、时间戳、平台、构建信息
  - `app`: `AppMode`、`NavigationStack`、`PlaybackMode`
  - `runtime`: `RuntimeSnapshot`
  - `render`: `RenderSnapshot`
  - `audio`: `AudioSnapshot`
  - `timing`: `TimingSnapshot`

设计原则：

- “快照对象”与“运行对象”解耦：由 `from_app_state(&AppState)` 提取必要字段，避免直接给复杂运行时类型加序列化约束。
- 字段稳定优先：优先使用语义化字段名（如 `current_script_path`、`waiting_reason`），便于模型和人类阅读。
- 可扩展：包含 `schema_version`，后续扩字段保持前向兼容。

### 3.4 各子系统采集范围

#### 3.4.1 RuntimeState

采集：

- 当前节点索引（node index）
- 当前脚本路径（script path）
- 变量存储（variable store，按可序列化值导出）
- `WaitingReason`
- 调用栈（call stack）

约束：

- 仅导出逻辑层可观测状态，不包含执行器内部临时借用引用。
- 对潜在大字段（如大文本变量）可增加长度截断策略（可选）。

#### 3.4.2 RenderState

采集：

- 当前背景（资源逻辑路径、显示参数）
- 角色位置信息与状态（slot、pose/expression、可见性、alpha 等）
- 对话状态（当前文本、typewriter 进度、窗口可见性）
- 活动过渡（transition 类型、进度）
- 活动效果（effect 列表与关键参数）

对不可序列化图形资源的处理：

- 不直接序列化 `Arc<dyn Texture>`、GPU handle 等。
- 导出描述性元数据：`texture_path`、`width`、`height`、`format_hint`（若可得）。

#### 3.4.3 AudioManager

采集：

- 当前 BGM（逻辑路径、播放位置、循环状态）
- 正在播放的 SE/voice 列表（逻辑路径、通道、音量、是否循环）

不采集：

- 音频解码器内部状态、底层设备句柄。

#### 3.4.4 App 导航与播放控制

采集：

- `AppMode`
- `NavigationStack`
- `PlaybackMode`（`Normal/Auto/Skip`）

价值：

- 快速判定“用户操作层”状态是否与脚本推进策略一致，定位输入与运行时策略冲突。

#### 3.4.5 帧时间信息

采集：

- 当前帧 `delta_time`
- 平滑 FPS（或最近窗口平均 FPS）
- 快照触发帧号（若可得）

价值：

- 排查时序敏感问题（低帧率导致过渡/输入异常）。

### 3.5 输出与文件命名

输出格式：

- `serde_json::to_string_pretty(&snapshot)` 生成可读 JSON。
- 编码 UTF-8。

输出路径策略：

- 目录：`snapshot_output_dir` 或默认目录。
- 文件名：`debug_snapshot_{timestamp}.json`。
- 时间戳建议：`YYYYMMDD_HHMMSS_mmm`（本地时间或 UTC，需在字段中明确）。

错误处理策略（外部 I/O 属于不受信路径）：

- 目录创建失败、文件写入失败、序列化失败均返回 `Result` 并记录结构化日志。
- 不中断主循环，不影响游戏继续运行。

### 3.6 API 与职责边界

建议新增职责：

- `AppState::capture_debug_snapshot(&self) -> Result<DebugSnapshot, SnapshotError>`
- `AppState::write_debug_snapshot(&self) -> Result<PathBuf, SnapshotError>`

`SnapshotError` 按失败域拆分（示例）：

- `SnapshotError::Serialize(serde_json::Error)`
- `SnapshotError::Io(std::io::Error)`
- `SnapshotError::InvalidConfig(String)`

边界约束：

- `vn-runtime` 保持纯逻辑，不依赖 host 的文件系统实现；快照落盘在 `host` 层完成。
- `vn-runtime` 如需辅助导出结构，可提供只读提取接口，但不承担 I/O。

### 3.7 可测试性与回归策略

单元测试建议：

- 快照结构构建：给定最小 `AppState`，断言关键字段存在且值正确。
- 不可序列化资源降级：断言导出描述字段而非 panic。
- 文件命名与目录策略：断言默认目录、自定义目录、时间戳命名格式。

集成测试建议：

- 模拟输入热键触发，验证仅在 `enable_debug_snapshot = true` 时生成文件。
- 验证同帧单次触发与连续帧触发行为。
- 验证写盘失败时不会中断 update loop（可通过 mock 或临时只读目录）。

---

## 4. 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host::config` | `DebugConfig` 新增快照开关与输出目录配置 | 低：配置反序列化兼容性需校验默认值 |
| `host::input` | 增加快照热键事件识别（默认 `F10`） | 低：需避免与现有快捷键冲突 |
| `host::app` | 在 `app/update` 集成触发逻辑；新增快照构建/写盘 API | 中：需保证失败不影响主循环 |
| `host::renderer::render_state` | 提供可导出的描述性快照字段 | 中：字段映射不完整会降低排障价值 |
| `host::audio` | 导出 BGM/SE/voice 当前播放状态 | 低：主要是只读映射 |
| `vn-runtime::state` | 暴露可读的运行时状态提取入口（如有必要） | 低：只读接口风险可控 |

---

## 5. 迁移计划

阶段 1：数据结构与配置落地

1. 定义 `DebugSnapshot` 及子结构，完成 `Serialize`。
2. 扩展 `DebugConfig`：`enable_debug_snapshot`、`snapshot_output_dir`。
3. 增加默认配置与配置兼容测试。

阶段 2：快照提取与写盘

1. 在 `AppState` 实现从各子系统提取快照字段。
2. 实现 JSON pretty 序列化与文件落盘。
3. 接入日志与错误类型，确保失败可观测且不崩溃。

阶段 3：热键接入与行为验证

1. 在输入处理链路增加 `F10` 触发检测。
2. 在 `app/update` 中按调试开关执行快照导出。
3. 完成单元/集成测试，验证触发条件与失败回退。

阶段 4：文档与使用说明

1. 在调试文档补充“如何抓取快照并提供给模型”。
2. 记录 JSON schema 示例，便于后续自动分析工具接入。

---

## 6. 验收标准

- [ ] `DebugConfig` 新增 `enable_debug_snapshot: bool` 与 `snapshot_output_dir: Option<String>`，且默认值与历史配置兼容。
- [ ] 按下 `F10` 且 `enable_debug_snapshot=true` 时，生成 `debug_snapshot_{timestamp}.json` 文件；关闭开关时不生成文件。
- [ ] 快照 JSON 至少包含：`runtime`、`render`、`audio`、`app_mode/navigation`、`playback_mode`、`timing` 六类字段。
- [ ] 不可直接序列化资源（如 `Arc<dyn Texture>`）以描述性字段输出，流程无 panic。
- [ ] 写盘或序列化失败时主循环继续运行，且有可检索日志。
- [ ] 增加覆盖触发逻辑与快照字段完整性的测试（单元/集成）。
- [ ] 执行 `cargo check-all` 通过。
- [ ] RFC 对应实现文档更新（调试说明与快照字段说明）完成。
