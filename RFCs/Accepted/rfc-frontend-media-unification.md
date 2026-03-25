# RFC: 前端媒体统一——动画模型收敛与音频前端化

## 元信息

- 编号：RFC-029
- 状态：Implemented
- 作者：claude-4.6-opus
- 日期：2026-03-25
- 相关范围：host-tauri（state.rs / render_state.rs / audio.rs / command_executor.rs）、前端（Vue 组件 / composables）
- 前置：无

---

## 背景

当前前后端在媒体表现（动画和音频）上混用了三种驱动模式，导致架构不一致和实际缺陷：

### 问题 1：动画驱动模式混乱

| 模式 | 使用者 | 后端做什么 | 前端做什么 |
|------|--------|-----------|-----------|
| A：后端逐帧驱动 | scene_transition、background_transition（progress）、打字机 | 每帧计算 progress/mask_alpha 推给前端 | 直接映射到样式 |
| B：后端声明目标 | 角色立绘 fade/move | 一次性设置 target_alpha + transition_duration | CSS transition 自主过渡 |
| C：前端完全自主 | video cutscene | 设置"播放什么" | HTML5 原生播放，完成后回调 |

问题表现：
- **BackgroundLayer.vue 不使用后端推送的 `progress`**——它用 `duration` 设 CSS transition，后端每帧算 progress 是浪费。
- **TransitionOverlay.vue 中后端推送的 `mask_alpha` 与 CSS `transition` 在"打架"**——两边同时试图控制同一个值。
- 模式 A 的动画流畅度受 tick 频率（IPC 往返延迟）限制，而模式 B/C 使用浏览器原生合成器线程，天然 60fps。

### 问题 2：音频在后端播放导致生命周期不对齐

音频由 Rust 后端通过 rodio 播放，WebView 关闭/刷新后音频继续播放，后端状态不随前端重置（已通过 `reset_session` 临时缓解）。此外，rodio 是原生依赖（cpal 音频驱动），阻碍 Web 部署。

### 共同根因

两个问题的根因相同：**没有统一的"后端控制决策，前端控制表现"分界原则**。动画和音频的表现细节应由前端自主管理，后端只需声明"应该是什么状态"并按预期时长计时来控制脚本流程。

---

## 目标与非目标

### 目标

1. **统一动画驱动为"声明式目标"模式**：后端不再逐帧推送 progress/mask_alpha，改为声明目标状态 + 持续时长。前端用 CSS transition / Web Audio 自主实现过渡。
2. **音频播放迁移到前端**：RenderState 新增音频状态字段，前端通过 Web Audio API 播放 BGM/SFX。后端 AudioManager 退化为 headless 状态追踪器。
3. **消除 rodio 原生依赖**：host-tauri 不再需要 rodio/cpal，为 Web 部署扫清最后的硬障碍。
4. **保持后端对脚本流程的唯一控制权**：动画/音频信号完成的判定仍由后端计时器负责，前端不参与脚本流程决策。

### 非目标

- **vn-runtime WASM 编译**：虽然 vn-runtime 已近乎 WASM 就绪，但全客户端方案不在本 RFC 范围内。
- **多用户/服务端部署**：debug_server 提升为生产 web server 是另一个话题。
- **存档系统改造**：SaveManager 的存储后端抽象留待 Web 部署 RFC。
- **打字机模式变更**：打字机是模式 A 中合理使用逐帧推进的场景（visible_chars 需要精确控制每字符出现时机），本 RFC 不改动打字机逻辑。

---

## 方案设计

### 设计原则：后端控制决策，前端控制表现

```
后端（权威）                           前端（执行）
├─ 决定"播放什么 BGM"                  ├─ 用 Web Audio 实际播放
├─ 决定"目标过渡是什么"                 ├─ 用 CSS transition 实现过渡
├─ 按 duration 计时决定信号何时解除      ├─ 在 duration 内完成视觉/听觉过渡
└─ 门控用户输入（WaitingFor 状态机）     └─ 响应 RenderState 变化做 diff
```

后端和前端使用同一个时间源（前端传入的 dt），因此计时器自然同步。即使极端情况下有微小偏差（≤1帧/16ms），也不会影响体验，因为：
- 后端计时器到期 → 解除信号 → 产出新命令 → 要等下一帧 tick 才返回前端
- 前端 CSS transition 到期 → 下一帧恰好收到新 RenderState → 视觉同帧切换

### 一、动画模型统一

#### 1.1 背景 dissolve 过渡

**当前**：后端每帧推 `progress`，前端不用它（用 CSS transition）。

**改为**：RenderState 中 `BackgroundTransition` 仅保留声明字段：

```rust
pub struct BackgroundTransition {
    pub old_background: Option<String>,
    pub new_background: String,
    pub duration: f32,
}
```

- 删除 `progress` 字段（前端从未使用）。
- 后端 `update_background_transition(dt)` 保留内部计时器（不推到 RenderState），到期后清除 `background_transition = None`。
- 前端行为不变——已经在用 `duration` 做 CSS transition。

#### 1.2 场景遮罩过渡（scene_transition）

**当前**：后端每帧推 `mask_alpha` / `ui_alpha` / `progress`，5 阶段状态机。

**改为**：RenderState 中 `SceneTransition` 简化为声明式：

```rust
pub struct SceneTransition {
    pub transition_type: SceneTransitionKind,
    pub phase: SceneTransitionPhase,
    pub duration: f32,
    pub pending_background: Option<String>,
}

pub enum SceneTransitionPhase {
    FadeIn,
    Hold,
    FadeOut,
    Completed,
}
```

- 删除 `mask_alpha`、`ui_alpha`、`progress`（由前端 CSS 管理）。
- 后端状态机仍按 duration 计时推进 phase，但不再计算渐变值。
- 前端根据 `phase` + `duration` 设置 CSS transition（例如 phase=FadeIn → `opacity: 1; transition: opacity ${duration}s`）。
- 后端仍按 `phase == Completed` 判定信号解除，保持脚本流程控制权。

#### 1.3 角色动画（已是模式 B，无需改动）

`CharacterSprite` 的 `target_alpha` + `transition_duration` 模式已经正确，不改动。

#### 1.4 打字机（保留模式 A）

打字机的 `visible_chars` 是逐字符精确控制，与 CPS（每秒字符数）、inline effects（等待、变速）紧密耦合，不适合改为声明式。保留当前逐帧推进模式。

### 二、音频前端化

#### 2.1 RenderState 新增音频状态

```rust
/// 音频声明式状态——后端描述"应该播什么"，前端负责实际播放
#[derive(Debug, Clone, Serialize)]
pub struct AudioState {
    /// 当前应播放的 BGM（None 表示静音）
    pub bgm: Option<BgmState>,
    /// 本帧需要播放的一次性音效（前端播放后忽略，下帧清空）
    pub sfx_queue: Vec<SfxRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BgmState {
    pub path: String,
    pub looping: bool,
    pub volume: f32,     // 0.0–1.0，已含 duck 计算
}

#[derive(Debug, Clone, Serialize)]
pub struct SfxRequest {
    pub path: String,
    pub volume: f32,
}
```

将 `AudioState` 作为 `RenderState` 的新字段：

```rust
pub struct RenderState {
    // ... 现有字段 ...
    pub audio: AudioState,
}
```

#### 2.2 后端 AudioManager 改为 headless 状态追踪

- 移除 rodio 依赖（`Cargo.toml` 中删除 `rodio`）。
- `AudioManager` 只保留状态追踪：`current_bgm_path`、`bgm_volume`、`sfx_volume`、`duck_multiplier`、`muted`。
- `play_bgm()`、`stop_bgm()`、`duck()`、`unduck()` 只更新内部状态，不做 I/O。
- 新增 `to_audio_state() -> AudioState` 方法，每帧被 `process_tick` 调用，写入 `RenderState.audio`。
- `sfx_queue` 在 `to_audio_state()` 调用后清空，确保每个音效只出现一帧。
- 移除 `unsafe impl Send`（不再需要 cpal 的线程绕过）。
- 移除 `audio_cache`（前端通过 URL 直接加载音频文件）。

#### 2.3 前端音频 composable

新增 `composables/useAudio.ts`：

```typescript
// 响应式监听 RenderState.audio 变化
// diff bgm 状态：path 变化 → crossfade；volume 变化 → 平滑调整；null → fade out
// sfx_queue 有新条目 → 播放一次性音效
// 使用 Web Audio API (AudioContext)
```

核心逻辑：
- 维护一个 `AudioContext` 和当前 BGM 的 `HTMLAudioElement`（或 `AudioBufferSourceNode`）。
- `watch(renderState.audio.bgm)` → 检测 `path` 变化 → crossfade 到新曲目。
- `watch(renderState.audio.bgm.volume)` → `gainNode.gain.linearRampToValueAtTime()`。
- `watch(renderState.audio.sfx_queue)` → 逐条播放。
- 用户首次交互后 `AudioContext.resume()`（浏览器 autoplay policy）。

#### 2.4 音量设置同步

- `update_settings` IPC 仍然将音量写入后端 AudioManager 的状态。
- 后端 `to_audio_state()` 计算最终音量 = `bgm_volume × duck_multiplier × (muted ? 0 : 1)`。
- 前端只关心最终的 `volume` 值，不需要知道 duck/mute 的内部计算。

#### 2.5 存档兼容

- `save_game` 仍然从后端 `AudioManager` 取 `current_bgm_path`——逻辑不变。
- `restore_from_save` 恢复 `current_bgm_path` 到 AudioManager → 下一帧 `to_audio_state()` 输出到 RenderState → 前端自动开始播放恢复的 BGM。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host-tauri/src-tauri/src/render_state.rs` | 新增 AudioState、简化 BackgroundTransition/SceneTransition 字段 | 低——结构体简化 |
| `host-tauri/src-tauri/src/state.rs` | update_background_transition/update_scene_transition 内部化进度、process_tick 生成 audio 状态 | 低——逻辑简化 |
| `host-tauri/src-tauri/src/audio.rs` | 重构为 headless-only，移除 rodio | 中——大量代码删除 |
| `host-tauri/src-tauri/src/command_executor.rs` | AudioCommand 保留但不再驱动真实播放 | 低 |
| `host-tauri/src-tauri/Cargo.toml` | 移除 rodio 依赖 | 低 |
| `host-tauri/src/types/render-state.ts` | 新增 AudioState 类型、简化过渡类型 | 低 |
| `host-tauri/src/vn/BackgroundLayer.vue` | 移除对 progress 的引用（当前已不使用） | 极低 |
| `host-tauri/src/vn/TransitionOverlay.vue` | 改为根据 phase + duration 设 CSS transition | 中——需重写过渡样式逻辑 |
| `host-tauri/src/composables/useAudio.ts` | **新增**——Web Audio 播放、BGM/SFX 管理 | 中——新代码 |
| `host-tauri/src/vn/VNScene.vue` | 集成 useAudio | 低 |
| 3 份摘要文档 | 同步更新 | 低 |

---

## 迁移计划

分两个阶段实施，每个阶段独立可交付、可验证：

### 阶段 1：动画模型统一

1. 简化 `BackgroundTransition`：移除 `progress` 字段，后端保留内部计时器。
2. 简化 `SceneTransition`：移除 `mask_alpha`/`ui_alpha`/`progress`，保留 phase + duration。
3. 重写 `TransitionOverlay.vue`：根据 phase 设 CSS transition。
4. 同步 TypeScript 类型定义。
5. 验证：正常过渡、Skip 跳过、存档/加载期间的过渡。

### 阶段 2：音频前端化

1. RenderState 新增 `AudioState`，state.rs 中 `process_tick` 末尾写入。
2. 重构 `AudioManager` 为 headless 状态追踪器，移除 rodio。
3. 实现 `useAudio.ts` composable。
4. 集成到 VNScene，验证 BGM/SFX/duck/crossfade/存档恢复。
5. 移除 `Cargo.toml` 中的 rodio 依赖。

### 向后兼容

- **存档格式**：不受影响。`SaveData` 中的 `AudioState`（vn-runtime 侧）只记录 `current_bgm` 和 `bgm_looping`，与播放实现无关。
- **脚本语法**：不受影响。`@bgm`、`@sfx`、`@stopbgm` 等指令的解析和 Command 生成不变。
- **现有 host（macroquad）**：本 RFC 只改 host-tauri，不影响旧 host。

---

## 验收标准

- [ ] `BackgroundTransition` 和 `SceneTransition` 中不再有 `progress`/`mask_alpha`/`ui_alpha` 字段
- [ ] 背景 dissolve 和场景过渡的视觉效果与改动前一致（CSS transition 驱动）
- [ ] Skip 模式下过渡被正确跳过（后端清除状态 → 前端 CSS 中断）
- [ ] `rodio` 依赖从 `Cargo.toml` 中移除
- [ ] `AudioManager` 无真实音频设备引用，无 `unsafe impl Send`
- [ ] RenderState 包含 `AudioState` 字段，BGM/SFX 状态正确反映当前音频意图
- [ ] 前端 `useAudio.ts` 能正确播放/停止/crossfade BGM，播放 SFX
- [ ] 存档后加载，BGM 能正确恢复播放
- [ ] Duck/unduck 音量变化在前端平滑过渡
- [ ] 浏览器 autoplay policy 正确处理（首次交互后 AudioContext resume）
- [ ] `cargo check -p host-tauri` 通过
- [ ] TypeScript 类型 `render-state.ts` 与 Rust 结构同步
- [ ] 相关摘要文档更新
