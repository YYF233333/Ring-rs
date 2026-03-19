# RFC: Headless 测试模式

## 元信息

- 编号：RFC-019
- 状态：Accepted
- 作者：Ring-rs 开发组
- 日期：2026-03-18（修订：2026-03-19）
- 实现完成：2026-03-19
- 相关范围：`host::main`、`host::host_app`、`host::app`、`host::app::init`、`host::backend`、`host::rendering_types`、`host::audio`、`host::input`
- 前置：RFC-008 (RenderBackend Trait)

### 实现摘要

- `host/src/headless.rs`：run() 入口；AppState::new(config, AppInit { headless: true, event_stream_path })；NullTextureFactory、虚拟屏幕尺寸；InputReplayer::load + drain_until 注入；headless_loop 固定 dt、update_ingame_common、tick_ingame_shared、run_script_tick；退出条件 replay-end/script-finished/max-frames/timeout-sec。
- main.rs：--headless、--replay-input（必填）、--event-stream、--exit-on、--max-frames、--timeout-sec；headless 分支调用 headless::run。
- AppInit 结构体统一构造参数；AudioManager::new_headless()、device_sink 为 Option；update_ingame_common、tick_ingame_shared 提取为 pub(crate) 供 headless 复用。
- 使用说明见 `docs/headless_guide.md`。

---

## 1. 背景

### 1.1 现状

Ring-rs 采用 Runtime/Host 分离架构：`vn-runtime` 负责纯逻辑（脚本解析、执行、状态机），`host` 负责渲染、音频、输入与资源。当前宿主层强依赖 winit + wgpu + egui，启动即创建窗口并初始化 GPU，导致：

- **AI 辅助开发受限**：模型无法直接操作 GUI 窗口，每次验证修复都需用户手动运行并复现；
- **CI/CD 难以覆盖**：无 GPU 的 CI 环境无法运行完整引擎；
- **回归测试成本高**：脚本行为、状态机、命令执行链路的自动化测试依赖真实窗口与 GPU；
- **逻辑层性能分析困难**：无法在隔离环境下测量纯逻辑层耗时，GPU 与窗口事件干扰结果。

### 1.2 RFC-008 已提供的基础

RFC-008 引入了 `Texture` / `TextureFactory` trait 抽象，并明确将 **NullBackend（headless 后端）** 列为目标（G2）。当前实现中：

- `rendering_types` 已定义 `DrawCommand`、`Texture`、`TextureFactory`、`TextureContext`；
- `NullTexture` 与 `NullTextureFactory` 已存在，用于 headless 单元测试；
- `ResourceManager` 通过 `set_texture_context` 注入 `TextureContext`，可接受 `NullTextureFactory`；
- `Renderer::build_draw_commands()` 产出 `Vec<DrawCommand>`，与具体后端解耦；
- `CommandExecutor` 更新 `RenderState`，不依赖 GPU。

因此，**渲染管线已具备 headless 运行的理论基础**。本 RFC 将 headless 从"测试辅助类型"扩展为"完整无窗口运行模式"。

### 1.3 数据流与 headless 边界

更新循环大致为：

```text
update(app_state, dt)
  → 模式分发 (modes)
  → run_script_tick (脚本执行)
  → CommandExecutor 处理 Command
  → 更新 RenderState / 音频 / 效果
  → 过渡 / 动画推进
  → [渲染] build_draw_commands → backend.render_frame  ← headless 下跳过
```

headless 模式下，逻辑链路（脚本 → Command → RenderState 更新 → 过渡/动画）可完整运行；仅窗口创建、GPU 初始化、egui、实际渲染与**真实音频输出**需跳过。音频逻辑仍需保留，用于调试 BGM/SE/duck/fade 等状态机行为（详见 3.1.2）。

### 1.4 核心定位：AI 调试的快速反馈载体

Headless 模式的首要使用场景是配合 RFC-016（输入录制）和 RFC-018（结构化事件流）实现 **AI 自动调试闭环**：

```text
用户录制 → recording.jsonl → headless 快进回放 → events.jsonl → AI 分析
```

因此 headless 模式的设计重心是：**以最快速度跑完逻辑帧，产出结构化诊断数据**。不需要模拟真实时间流逝，不需要等待渲染完成，也不允许真实音频播放干扰外界时间流。我们只关心最终的状态转移序列、音频逻辑状态和事件流。

---

## 2. 目标与非目标

### 2.1 目标

- **G1**：提供 headless 运行入口，在无窗口、无 GPU 环境下执行完整引擎状态机。
- **G2**：复用 RFC-008 的 `NullTexture` / `NullTextureFactory`，确保 `ResourceManager`、`Renderer`、`CommandExecutor` 链路在 headless 下可运行。
- **G3**：headless 下**绝不播放真实音频**，但保留完整音频管理逻辑，用于调试 `play_bgm`、`stop_bgm`、`crossfade`、`duck` 等状态机行为（详见 3.1.2）。
- **G4**：headless 模式固定以 RFC-016 的 replay 文件作为**唯一输入源**，不再支持 Auto-advance、默认选项等辅助输入方式。
- **G5**：支持可配置退出条件（脚本结束、回放结束、N 帧、超时）及退出码表示成功/失败。
- **G6**：支持**快进模式**——去掉帧间 sleep，以 CPU 全速推进逻辑帧，最大化 AI 调试反馈速度（详见 3.2.3）。
- **G7**：headless 下**默认启用 RFC-018 事件流**，确保每次回放都产出可供 AI 分析的诊断数据。
- **G8**：与 RFC-015（状态快照）、RFC-016（输入回放）、RFC-018（结构化事件流）协同，形成自动化调试基础设施。

### 2.2 非目标

- **不**实现多渲染后端切换（OpenGL/Vulkan 等），仅扩展 null 路径；
- **不**在 headless 下支持 egui UI 交互（无窗口即无 UI）；
- **不**追求像素级渲染一致性（headless 不渲染）；
- **不**在首期实现 headless 下的视频播放（可留占位或跳过）；
- **不**修改 `vn-runtime`，仅改 host 初始化与运行分支；
- **不**在 headless 下支持真实用户输入或 GUI 辅助输入，必须使用 replay 文件驱动。

---

## 3. 方案设计

### 3.1 NullBackend 组件

#### 3.1.1 渲染侧（已部分存在）

| 组件 | 状态 | 说明 |
|------|------|------|
| `NullTexture` | 已实现 | 实现 `Texture` trait，仅存 width/height/label，`as_any()` 返回 self |
| `NullTextureFactory` | 已实现 | 实现 `TextureFactory`，从 image bytes 解析尺寸或接受任意数据创建 `NullTexture` |
| `TextureContext` | 已支持 | 可注入 `Arc::new(NullTextureFactory)` |

headless 模式下，`ResourceManager::set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)))` 即可，无需 `WgpuBackend`。

#### 3.1.2 音频侧：保留逻辑，禁止真实播放

headless 模式下，**真实音频播放必须被禁止**。原因不是“有没有音频硬件”，而是 headless 的逻辑时间与外界墙钟时间并不保持同步：

- headless 默认以 CPU 全速推进，60 秒脚本可能在 2 秒内跑完；
- 若仍调用真实音频播放，BGM/SE 的实际时长会与逻辑帧进度失配；
- 这会让 `fade`、`duck`、`crossfade` 等行为失真，反而污染调试结果。

因此本 RFC 要求为 headless 提供**无输出的音频管理实现**。推荐形态：

- 保留 `AudioManager` 对外语义与状态字段；
- `play_bgm` / `play_sfx` / `stop_bgm` / `crossfade_bgm` / `duck` / `unduck` / `update(dt)` 等接口仍正常执行；
- 内部只推进逻辑状态机，不向 `rodio`、`cpal` 或系统音频设备提交任何播放请求；
- `current_bgm_path`、音量、fade 进度、duck 倍率等状态完整保留，可用于事件流、快照与断言。

实现形式可以是：

- `HeadlessAudioManager`：与现有 `AudioManager` 共享公共状态与逻辑代码，但输出层为 no-op；
- 或将 `AudioManager` 拆为“状态机 + 输出后端”，GUI 使用 `RodioBackend`，headless 使用 `NoopBackend`。

设计目标：

- **普通 PC headless**：不出声，但音频逻辑状态完整；
- **CI headless**：同样不出声，且不依赖系统音频硬件；
- **事件流一致**：无论本地还是 CI，`audio_event` 都能稳定产出，便于 AI 分析。

#### 3.1.3 输入侧：replay-only

headless 下无 winit 事件，也**不再提供** Auto-advance、默认选项等辅助输入。输入源固定为 RFC-016 的 replay 文件：

- 启动 headless 时必须提供 `replay_input`；
- `InputReplayer` 是 headless 的唯一输入生产者；
- `t_ms <= elapsed_ms` 的事件被注入 `process_input_event(InputEvent)`；
- 若 replay 文件缺失、损坏或为空，headless 启动失败并返回非 0 退出码。

这样可以保证：AI 每次调试的输入路径都来自真实用户录制，而不是“猜测式自动推进”。

### 3.2 Headless 入口点

#### 3.2.1 触发方式

- **方式 1（推荐）**：现有二进制 `--headless` 标志。`main.rs` 解析参数，若 `--headless` 则走 headless 分支，否则走现有 winit 事件循环。
- **方式 2**：独立二进制 `host-headless`（如 `cargo run -p host --bin host-headless`）。与主程序共享 `lib`，仅 `main` 不同。

本 RFC 推荐 **方式 1**，实现简单且便于 AI/CI 调用。

#### 3.2.2 初始化流程（headless 分支）

1. 加载 `AppConfig`（与现有路径相同）；
2. 初始化 tracing（与现有路径相同）；
3. **跳过**：`EventLoop::new()`、窗口创建、`WgpuBackend::new()`、egui 初始化；
4. 调用 `AppState::new(config)`（与现有相同）；
5. 调用 `resource_manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)))`；
6. 构造 headless 音频管理实现，仅推进逻辑状态机，不向真实音频设备发起播放；
7. 设置 `renderer.set_screen_size(config.window.width as f32, config.window.height as f32)`（使用配置中的逻辑分辨率）；
8. 加载并校验 `replay_input` 文件；若缺失则直接报错退出；
9. 初始化 `InputReplayer`，并用 replay 元数据做兼容性 guard 校验（如逻辑尺寸、场景标识、格式版本）；
10. 在未显式指定事件流输出路径时，为 headless 自动生成默认输出路径；
11. 进入 headless 主循环。

#### 3.2.3 Headless 主循环与快进模式

headless 模式的核心设计原则：**我们只关心最终结果（事件流、状态、退出码），不关心实时性**。因此默认以 CPU 全速运行，不插入帧间 sleep。

```rust
// 伪代码
let fixed_dt = 1.0 / 60.0; // 固定逻辑帧步长，保证确定性
loop {
    inject_replay_events(&mut app_state); // 唯一输入源：RFC-016 replay
    update(&mut app_state, fixed_dt);
    // 不调用 backend.render_frame，不调用 egui
    if should_exit(&app_state) { break; }
    // 无 sleep——全速推进
}
```

**快进效果**：一段 60 秒的游玩录制，在 headless 下可能仅需 1–2 秒跑完（取决于逻辑复杂度）。这使 AI 可以在修改代码后快速验证，实现秒级反馈循环。

**速度控制参数（可选）**：

- `--speed=max`（默认）：无 sleep，全速运行；
- `--speed=realtime`：插入 sleep 模拟真实时间流逝（仅用于特殊排查，非默认路径）；
- `--speed=N`（如 `--speed=10`）：以 N 倍速运行（每帧 sleep 缩短为 1/N）。

**固定 dt 的重要性**：无论速度模式如何，逻辑层始终使用固定 `dt`（默认 1/60s）。这保证：
- 同一录制文件多次回放，`vn-runtime` 状态转移序列完全一致；
- 过渡/动画推进不受运行速度影响，逻辑行为可重复。

`should_exit` 条件（可配置组合）：

- 脚本执行完毕（`script_finished`）；
- 回放事件耗尽且引擎进入稳定结束态（`--exit-on=replay-end`，推荐默认）；
- 达到指定帧数（`--max-frames N`）；
- 超时（`--timeout-sec N`，以墙钟时间计）；

#### 3.2.4 CLI 协议

为防止启动出“无输入 headless”，CLI 约束应显式编码为协议：

- `--headless` 只能与 `--replay-input=<path>` 一起使用；
- 若提供 `--headless` 但缺少 `--replay-input`，启动阶段直接报错并返回非 0；
- `--replay-input` 在 GUI 模式下无效，避免产生“双输入源”歧义；
- `--event-stream=<path>` 为可选覆盖项；未提供时，headless 自动生成默认事件流路径；
- `--speed`、`--timeout-sec`、`--max-frames`、`--exit-on` 仅在 headless 下生效。

推荐 CLI 形式：

```bash
ring-rs --headless --replay-input=recording.jsonl
```

等价的显式形式：

```bash
ring-rs --headless --replay-input=recording.jsonl --event-stream=events.jsonl --exit-on=replay-end
```

### 3.3 Headless 下可用与不可用能力

| 能力 | PC headless | CI headless | 说明 |
|------|:-----------:|:-----------:|------|
| 脚本解析与执行 | ✓ | ✓ | 完整 |
| Command 生成与处理 | ✓ | ✓ | 完整 |
| RenderState 更新 | ✓ | ✓ | 完整 |
| 过渡/动画状态推进 | ✓ | ✓ | 完整 |
| 变量/状态机逻辑 | ✓ | ✓ | 完整 |
| 音频状态追踪 | ✓ | ✓ | 统一使用 headless 音频逻辑，不依赖真实设备 |
| 存档序列化 | ✓ | ✓ | 本地与 CI 均保留音频逻辑状态 |
| 实际渲染 | ✗ | ✗ | 无 GPU、无窗口 |
| 真实音频播放 | ✗ | ✗ | headless 严禁实际播放 |
| 用户输入 | ✗ | ✗ | 唯一输入源为 replay 文件 |
| egui UI 交互 | ✗ | ✗ | 无窗口 |

### 3.4 Headless 输出

| 输出类型 | 说明 |
|----------|------|
| 控制台 | `tracing` 日志、脚本进度摘要（可配置 verbosity） |
| RFC-018 事件流 | **默认启用**，输出结构化 JSON Lines 事件到文件 |
| RFC-015 状态快照 | 若配置快照触发点，在 headless 下同样可导出 |
| 退出码 | 0 = 脚本正常结束，非 0 = 超时/错误/未完成 |

### 3.5 与相关 RFC 的协同

| RFC | 协同方式 |
|-----|----------|
| RFC-015 调试状态快照 | headless 下可按帧/按条件触发快照导出，无需用户按热键 |
| RFC-016 输入录制与 AI 调试管线 | **核心联动**：人类游玩时录制输入 → headless 以 replay 作为唯一输入源快进回放 → AI 分析事件流。headless 是录制文件的消费端 |
| RFC-018 结构化事件流 | headless 默认启用事件流输出，AI 在快进回放后分析完整事件序列定位问题 |

**典型 AI 调试工作流**：

```bash
# 1. AI 收到用户的录制文件 recording.jsonl
# 2. 快进回放，产出事件流
ring-rs --headless --replay-input=recording.jsonl --exit-on=replay-end
# 3. AI 分析 events.jsonl 定位问题
# 4. AI 修复代码后再次回放验证
ring-rs --headless --replay-input=recording.jsonl --event-stream=events_fixed.jsonl --exit-on=replay-end
# 5. 对比 events.jsonl vs events_fixed.jsonl 确认修复
```

整个流程在秒级完成，无需用户参与。

### 3.6 模块与文件变更

| 模块/文件 | 变更类型 | 说明 |
|-----------|----------|------|
| `host/src/main.rs` | 扩展 | 解析 `--headless`、`--replay-input`、`--speed` 等参数，并对 headless 做必填校验 |
| `host/src/host_app.rs` | 无或最小 | 仅 GUI 路径使用；headless 不创建 HostApp |
| `host/src/app/init.rs` | 微调 | headless 分支跳过 GPU 初始化，并构造 headless 音频实现 |
| `host/src/audio/mod.rs` | 扩展 | 抽出音频逻辑状态与无输出后端，供 headless 复用 |
| `host/src/rendering_types.rs` | 无 | `NullTexture` / `NullTextureFactory` 已存在 |
| `host/src/headless.rs` | 新增 | headless 主循环、输入注入、退出条件、速度控制 |
| `host/src/input/mod.rs` | 扩展 | headless replay 注入接口与必填 replay 校验 |

---

## 4. 影响范围

| 模块 | 影响 |
|------|------|
| `host::main` | 新增 headless 分支，参数解析 |
| `host::app` | `AppState::new` 不变，init 需支持 headless 初始化路径 |
| `host::app::init` | 跳过 GPU/窗口，构造 headless 音频与默认事件流 |
| `host::audio` | 抽出/新增 headless 无输出实现，保留音频逻辑状态 |
| `host::backend` | 无改动，headless 不实例化 `WgpuBackend` |
| `host::rendering_types` | 无改动 |
| `host::resources` | 无改动，仅注入 `NullTextureFactory` |
| `host::input` | 可能新增 headless 输入注入 API |
| `vn-runtime` | 无改动 |

---

## 5. 迁移计划

### 阶段 1：最小 headless 骨架

1. 在 `main.rs` 中解析 `--headless`，若存在则跳过 `EventLoop`，直接进入 headless 分支；
2. 实现 `headless::run(config, replay_path)`：创建 `AppState`，注入 `NullTextureFactory`，构造 headless 音频实现；
3. 将 `--replay-input` 设为 headless 必填参数，缺失则直接报错退出；
4. 实现全速 `loop { update(...); }` 主循环，固定 dt，无 sleep；
5. 默认启用事件流，未显式传 `--event-stream` 时自动派生输出路径；
6. 验证：`cargo run -p host -- --headless --replay-input recording.jsonl` 能启动并快速跑完。

### 阶段 2：replay 输入与速度控制

1. 集成 RFC-016 回放作为唯一输入源；
2. 完成 replay 文件加载、元数据校验、坐标映射与错误处理；
3. 实现 `--speed` 参数：`max`（默认）/ `realtime` / 倍速；
4. 移除 Auto-advance、默认选项等辅助输入路径。

### 阶段 3：输出与 RFC 协同

1. 退出码规范化：0 / 非 0；
2. 集成 RFC-018 事件流：headless 下默认启用并输出到文件；
3. 集成 RFC-015 快照：headless 下可按条件触发快照；
4. 支持 `--max-frames`、`--timeout-sec`、`--exit-on` 等退出条件参数。

### 阶段 4：打磨与文档

1. 控制台输出 verbosity 配置；
2. 补充 headless 相关单元测试与集成测试；
3. 文档：CI 配置示例、AI 调试工作流说明；
4. 视实现拆分结果整理音频公共状态代码，避免 GUI/headless 逻辑重复。

---

## 6. 验收标准

- [ ] `cargo run -p host -- --headless --replay-input recording.jsonl` 可在无窗口环境下启动并运行；缺少 replay 文件时应直接失败。
- [ ] headless 下脚本解析、执行、Command 处理、RenderState 更新、过渡/动画推进均正常运行。
- [ ] headless 下 `ResourceManager::load_texture` 使用 `NullTextureFactory`，不依赖 GPU。
- [ ] headless 下绝不向真实音频设备发起播放，但 `play_bgm` / `stop_bgm` / `duck` / `crossfade` 等逻辑状态仍被完整维护。
- [ ] headless 默认全速运行（无帧间 sleep），60 秒录制在 headless 下数秒内跑完。
- [ ] headless 的唯一输入源为 replay 文件，不存在 Auto-advance 或默认选项等辅助输入分支。
- [ ] 退出码正确：脚本正常结束返回 0，超时/错误返回非 0。
- [ ] `cargo check-all` 通过，无新增 clippy 警告。
- [ ] headless 下默认启用事件流；若未显式指定输出路径，系统可自动生成默认文件路径。
- [ ] 文档说明 headless 用法、必填参数、默认事件流行为，以及与 RFC-015/016/018 的协同方式。
