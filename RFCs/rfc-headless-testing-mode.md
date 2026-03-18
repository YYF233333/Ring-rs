# RFC: Headless 测试模式

## 元信息

- 编号：RFC-019
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-18
- 相关范围：`host::main`、`host::host_app`、`host::app`、`host::app::init`、`host::backend`、`host::rendering_types`、`host::audio`、`host::input`
- 前置：RFC-008 (RenderBackend Trait)

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

因此，**渲染管线已具备 headless 运行的理论基础**。本 RFC 将 headless 从“测试辅助类型”扩展为“完整无窗口运行模式”。

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

headless 模式下，逻辑链路（脚本 → Command → RenderState 更新 → 过渡/动画）可完整运行；仅窗口创建、GPU 初始化、egui、实际渲染与音频播放需替换为 null 实现或跳过。

---

## 2. 目标与非目标

### 2.1 目标

- **G1**：提供 headless 运行入口，在无窗口、无 GPU、无音频设备环境下执行完整引擎状态机。
- **G2**：复用 RFC-008 的 `NullTexture` / `NullTextureFactory`，确保 `ResourceManager`、`Renderer`、`CommandExecutor` 链路在 headless 下可运行。
- **G3**：提供 null 音频实现（`NullAudioManager` 或等价），使 PlayBgm/PlaySfx 等命令可“执行”但不产生实际声音，并保持状态追踪（如 `current_bgm_path`）以支持存档、快照、事件流。
- **G4**：headless 模式下支持脚本化输入（Auto-advance、可配置默认选项、与 RFC-016 输入回放集成）。
- **G5**：支持可配置退出条件（脚本结束、N 帧、超时）及退出码表示成功/失败。
- **G6**：与 RFC-015（状态快照）、RFC-016（输入回放）、RFC-018（结构化事件流）协同，形成自动化调试基础设施。

### 2.2 非目标

- **不**实现多渲染后端切换（OpenGL/Vulkan 等），仅扩展 null 路径；
- **不**在 headless 下支持 egui UI 交互（无窗口即无 UI）；
- **不**追求像素级渲染一致性（headless 不渲染）；
- **不**在首期实现 headless 下的视频播放（可留占位或跳过）；
- **不**修改 `vn-runtime`，仅改 host 初始化与运行分支。

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

#### 3.1.2 音频侧（需新增）

当前 `AudioManager` 为具体类型，`CoreSystems::audio_manager` 为 `Option<AudioManager>`。音频初始化失败时已为 `None`，command_handlers 已处理 `if let Some(ref mut audio)`。

为 headless 提供两种可选方案：

- **方案 A（简单）**：headless 下使用 `audio_manager: None`。PlayBgm/PlaySfx 等命令被跳过，存档/快照中 BGM 状态可能为空。实现成本最低。
- **方案 B（推荐）**：新增 `NullAudioManager`，实现与 `AudioManager` 相同的公开接口（或通过 trait 抽象），但 play/stop/volume 均为 no-op；内部仍维护 `current_bgm_path`、`bgm_volume`、`duck_multiplier` 等状态，供存档、RFC-015 快照、RFC-018 事件流使用。

本 RFC 推荐 **方案 B**，以保证 headless 下状态完整性与 RFC 协同。若首期为降低复杂度，可先采用方案 A，在后续迭代中引入 `NullAudioManager`。

`NullAudioManager` 设计要点：

- 不调用 `rodio::DeviceSinkBuilder::open_default_sink()`，构造必定成功；
- `play_bgm` / `stop_bgm` / `play_sfx` / `crossfade_bgm` / `duck` / `unduck` 等为 no-op，但更新 `current_bgm_path`、`fade_state` 等内部状态；
- `update(dt)` 推进 fade 状态机，保证逻辑一致性；
- `cache_audio_bytes` 可 no-op 或仅做内存占位（若需验证资源加载路径可保留缓存逻辑）。

#### 3.1.3 输入侧

headless 下无 winit 事件，需替代输入源：

- **Auto-advance**：当 `WaitingReason::WaitForClick` 时，自动注入 Click 输入推进；
- **选项**：当 `WaitingReason::WaitForChoice` 时，使用可配置的默认选项索引（如 `--choice-index 0`）或选项脚本文件；
- **RFC-016 集成**：若启用输入回放，从录制文件注入 `InputEvent`，替代真实输入。

### 3.2 Headless 入口点

#### 3.2.1 触发方式

- **方式 1**：现有二进制 `--headless` 标志。`main.rs` 解析参数，若 `--headless` 则走 headless 分支，否则走现有 winit 事件循环。
- **方式 2**：独立二进制 `host-headless`（如 `cargo run -p host --bin host-headless`）。与主程序共享 `lib`，仅 `main` 不同。

本 RFC 推荐 **方式 1**，实现简单且便于 CI 调用（`./host --headless --script main.vn`）。

#### 3.2.2 初始化流程（headless 分支）

1. 加载 `AppConfig`（与现有路径相同）；
2. 初始化 tracing（与现有路径相同）；
3. **跳过**：`EventLoop::new()`、窗口创建、`WgpuBackend::new()`、egui 初始化；
4. 调用 `AppState::new(config)`（与现有相同）；
5. 调用 `resource_manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)))`；
6. 使用 `init::create_audio_manager_headless()` 或等价，返回 `Some(NullAudioManager)` 或 `None`（若采用方案 A）；
7. 设置 `renderer.set_screen_size(config.window.width as f32, config.window.height as f32)`（使用配置中的逻辑分辨率）；
8. 若指定了 `--script`，加载并启动脚本；否则使用 manifest 默认入口；
9. 进入 headless 主循环。

#### 3.2.3 Headless 主循环

```rust
// 伪代码
let dt = 1.0 / 60.0;  // 固定 60 FPS 或可配置
loop {
    inject_headless_input(&mut app_state);  // Auto-advance / 选项 / RFC-016 回放
    update(&mut app_state, dt);
    // 不调用 backend.render_frame，不调用 egui
    if should_exit(&app_state) { break; }
    std::thread::sleep(duration_from_secs(dt));  // 可选，控制速率
}
```

`should_exit` 条件（可配置组合）：

- 脚本执行完毕（`script_finished`）；
- 达到指定帧数（`--max-frames N`）；
- 超时（`--timeout-sec N`）；
- 特定状态/事件触发（可与 RFC-018 事件流联动）。

### 3.3 Headless 下可用与不可用能力

| 能力 | 可用 | 说明 |
|------|------|------|
| 脚本解析与执行 | ✓ | 完整 |
| Command 生成与处理 | ✓ | 完整 |
| RenderState 更新 | ✓ | 完整 |
| 过渡/动画状态推进 | ✓ | 完整 |
| 变量/状态机逻辑 | ✓ | 完整 |
| 存档序列化/反序列化 | ✓ | 完整（含 BGM 等状态，若用 NullAudioManager） |
| 实际渲染 | ✗ | 无 GPU、无窗口 |
| 真实音频播放 | ✗ | NullAudioManager 不发声 |
| 用户输入 | ✗ | 需脚本化（Auto-advance / 选项 / 回放） |
| egui UI 交互 | ✗ | 无窗口 |

### 3.4 Headless 输出

| 输出类型 | 说明 |
|----------|------|
| 控制台 | `tracing` 日志、脚本进度摘要（可配置 verbosity） |
| RFC-018 事件流 | 若启用，输出结构化 JSON Lines 事件到文件 |
| RFC-015 状态快照 | 若配置快照触发点，在 headless 下同样可导出 |
| 退出码 | 0 = 脚本正常结束，非 0 = 超时/错误/未完成 |

### 3.5 与相关 RFC 的协同

| RFC | 协同方式 |
|-----|----------|
| RFC-015 调试状态快照 | headless 下可按帧/按条件触发快照导出，无需用户按热键；模型可直接分析快照文件 |
| RFC-016 输入录制与回放 | headless 使用回放文件作为输入源，实现“录制一次、headless 复现多次”的自动化调试 |
| RFC-018 结构化事件流 | headless 启用事件流输出，模型可分析完整事件序列，无需人工操作窗口 |

headless 模式是上述自动化调试能力的**运行载体**：在无人工交互的前提下，通过脚本化输入 + 事件流 + 快照，实现闭环验证。

### 3.6 模块与文件变更

| 模块/文件 | 变更类型 | 说明 |
|-----------|----------|------|
| `host/src/main.rs` | 扩展 | 解析 `--headless` 等参数，分支初始化与主循环 |
| `host/src/host_app.rs` | 无或最小 | 仅 GUI 路径使用；headless 不创建 HostApp |
| `host/src/app/init.rs` | 扩展 | `create_audio_manager_headless()` 或 `create_audio_manager(..., headless: bool)` |
| `host/src/audio/mod.rs` | 扩展 | 新增 `NullAudioManager`（若采用方案 B） |
| `host/src/rendering_types.rs` | 无 | `NullTexture` / `NullTextureFactory` 已存在 |
| `host/src/headless.rs` | 新增 | headless 主循环、输入注入、退出条件判断 |
| `host/src/input/mod.rs` | 扩展 | headless 输入注入接口（或放在 `headless.rs` 内） |

---

## 4. 影响范围

| 模块 | 影响 |
|------|------|
| `host::main` | 新增 headless 分支，参数解析 |
| `host::app` | `AppState::new` 不变，init 需支持 headless 音频 |
| `host::app::init` | 新增 headless 音频创建路径 |
| `host::audio` | 新增 `NullAudioManager`（可选） |
| `host::backend` | 无改动，headless 不实例化 `WgpuBackend` |
| `host::rendering_types` | 无改动 |
| `host::resources` | 无改动，仅注入 `NullTextureFactory` |
| `host::input` | 可能新增 headless 输入注入 API |
| `vn-runtime` | 无改动 |

---

## 5. 迁移计划

### 阶段 1：最小 headless 骨架

1. 在 `main.rs` 中解析 `--headless`，若存在则跳过 `EventLoop`，直接进入 headless 分支；
2. 实现 `headless::run(config)`：创建 `AppState`，注入 `NullTextureFactory`，`audio_manager: None`（方案 A）；
3. 实现固定 dt 的 `loop { update(...); }`，无输入注入，脚本需无交互或自推进；
4. 退出条件：仅 `script_finished`；
5. 验证：`cargo run -p host -- --headless` 能启动并跑完无交互脚本。

### 阶段 2：输入与音频

1. 实现 Auto-advance：`WaitForClick` 时自动注入 Click；
2. 实现选项默认索引：`WaitForChoice` 时使用 `--choice-index` 或配置；
3. （可选）实现 `NullAudioManager`，替换 `None`，保证状态追踪；
4. （可选）集成 RFC-016 回放作为输入源。

### 阶段 3：输出与 RFC 协同

1. 退出码规范化：0 / 非 0；
2. 集成 RFC-018 事件流（若已实现）：headless 下可启用并输出到文件；
3. 集成 RFC-015 快照（若已实现）：headless 下可按条件触发快照；
4. 文档与 CI：在 `CONTRIBUTING.md` 或 CI 配置中说明 headless 用法。

### 阶段 4：打磨

1. 支持 `--max-frames`、`--timeout-sec` 等参数；
2. 控制台输出 verbosity 配置；
3. 补充 headless 相关单元测试与集成测试。

---

## 6. 验收标准

- [ ] `cargo run -p host -- --headless` 可在无窗口环境下启动；
- [ ] headless 下脚本解析、执行、Command 处理、RenderState 更新、过渡/动画推进均正常运行；
- [ ] headless 下 `ResourceManager::load_texture` 使用 `NullTextureFactory`，不依赖 GPU；
- [ ] headless 下 `WaitForClick` 可由 Auto-advance 自动推进；
- [ ] headless 下 `WaitForChoice` 可使用可配置默认选项或回放文件；
- [ ] 退出码正确：脚本正常结束返回 0，超时/错误返回非 0；
- [ ] `cargo check-all` 通过，无新增 clippy 警告；
- [ ] 文档说明 headless 用法、参数、与 RFC-015/016/018 的协同方式；
- [ ] （可选）`NullAudioManager` 实现后，headless 下存档/快照包含正确 BGM 状态。
