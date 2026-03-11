# RFC: Cutscene 视频播放

## 元信息

- 编号：RFC-009
- 状态：Completed
- 作者：Ring-rs 开发组
- 日期：2026-03-12
- 相关范围：`vn-runtime`（AST/Parser/Command/Executor）、`host`（视频播放/渲染/音频）
- 前置：RFC-007 渲染后端迁移已完成（winit + wgpu + egui）
- 关联：RFC-002 P1-4（cutscene 视频播放）

---

## 1. 背景

RFC-002 P1-4 将 cutscene 视频播放列为可选最低优先级项。当前脚本中有 1 处使用：

```markdown
cutscene "audio/ending_HVC_bgm.webm"
```

（`assets/scripts/remake/ring/winter/ending.md:168`）

RFC-007 完成后，渲染后端已迁移至 winit + wgpu + egui，`queue.write_texture()` 动态纹理更新能力已在 PoC 中验证，视频帧注入的架构阻碍已扫清。

当前解析器对 `cutscene` 行直接跳过（不中断流程），需要补全从解析到播放的完整链路。

---

## 2. 目标与非目标

### 2.1 目标

- **G1** — 实现 `cutscene "path"` 命令全链路（AST/Parser/Command/Executor/Host）
- **G2** — 视频全屏播放，音视频同步，播放完毕或玩家操作后恢复脚本流程
- **G3** — 支持跳过（点击/Enter/Esc），Skip 模式下立即跳过
- **G4** — 格式支持 WebM（VP8/VP9 + Vorbis/Opus），架构上不限制未来扩展其他格式
- **G5** — 优雅降级：FFmpeg 不可用时跳过视频、log 警告，剧情流程不中断

### 2.2 非目标

- **不**构建通用视频播放器（不支持暂停/快进/进度条/字幕）
- **不**支持视频循环播放或画中画
- **不**改动 `vn-runtime` 的等待模型（复用 `WaitForSignal`）
- **不**在本 RFC 内实现视频编辑/预处理工具链
- **不**追求硬件加速视频解码（FFmpeg 自身已有，无需额外处理）
- **不**在 `vn-runtime` crate 中引入任何引擎/IO 依赖

---

## 3. 方案选型

### 3.1 候选方案

| 方案 | 组合 | 编译依赖 | 运行时依赖 | WebM 支持 | Windows 构建 | 格式扩展性 |
|------|------|----------|-----------|----------|-------------|-----------|
| **A** | ffmpeg-next (Rust FFmpeg 绑定) | FFmpeg dev libs | FFmpeg shared libs | 全支持 | 问题频发（链接/模式匹配/静态链接） | 全格式 |
| **B** | ffmpeg-sidecar (FFmpeg 子进程) | 无 | FFmpeg 二进制 | 全支持 | 无构建问题 | 全格式 |
| **C** | vk-video (Vulkan Video 硬解) | 无 | Vulkan 驱动 | 仅 H.264 | 依赖驱动 | 受限 |
| **D** | matroska-demuxer + 纯 Rust 解码 | 无 | 无 | VP9 部分，VP8 无成熟实现 | 无问题 | 极受限 |
| **E** | GStreamer 绑定 | GStreamer dev | GStreamer 运行时 (~200MB) | 全支持 | 需安装 GStreamer | 全格式 |
| **F** | 预转帧序列 | 无 | 无 | N/A（构建时预处理） | 无问题 | N/A |

### 3.2 选定方案：B（ffmpeg-sidecar）

**理由**：

1. **零编译复杂度** — 不链接任何 C 库，不影响现有 `cargo check-all` 流程。方案 A 在 Windows 上的构建问题（链接失败、非穷尽模式匹配、静态链接不支持）是硬伤。
2. **全格式支持** — WebM/VP8/VP9 开箱即用，未来增加 MP4/H.264/AV1 等格式零代码改动。方案 C 仅 H.264，方案 D 无成熟 VP8 解码器。
3. **API 简洁** — Iterator 接口逐帧输出 `OutputVideoFrame { width, height, data: Vec<u8>, timestamp, pix_fmt }`，与 wgpu `queue.write_texture()` 对接直白。
4. **运行时依赖可控** — FFmpeg 二进制可随游戏分发（精简构建约 20-30MB），或优雅降级（找不到则跳过视频）。
5. **性能充裕** — 1080p 30fps 管道吞吐约 186 MB/s，OS 管道带宽 >1 GB/s，不构成瓶颈。FFmpeg 自身支持硬件加速解码。

### 3.3 被否决方案的关键原因

| 方案 | 否决原因 |
|------|---------|
| A (ffmpeg-next) | Windows 构建问题频发且处于维护模式；为单个 cutscene 引入重量级编译依赖不值得 |
| C (vk-video) | v0.2.0 极早期，仅 H.264，无法解码 WebM |
| D (纯 Rust) | Rust 生态无成熟 VP8/VP9 全帧解码器，WebM 音频轨解码也不完整 |
| E (GStreamer) | 运行时依赖过重（~200MB），安装复杂度远超 FFmpeg 单文件 |
| F (帧序列) | 60s 30fps 1080p ≈ 1800 帧 PNG，资源体积不可接受 |

### 3.4 新增依赖

| 库 | 版本 | 用途 |
|----|------|------|
| ffmpeg-sidecar | latest | FFmpeg 子进程管理、视频帧迭代 |

现有依赖不变：`wgpu`（纹理上传）、`rodio`（音频播放）、`image`（可选：fallback 帧解码）。

---

## 4. 架构设计

### 4.1 整体数据流

```
脚本: cutscene "audio/ending_HVC_bgm.webm"
    ↓ Parser
AST: ScriptNode::Cutscene { path }
    ↓ Executor
Command: Command::Cutscene { path } + WaitForSignal("cutscene")
    ↓ Host CommandExecutor
VideoPlayer::start(resolved_path)
    ↓
┌─ FFmpeg 子进程 (视频) ──────────────────────┐
│  ffmpeg -i input.webm -f rawvideo           │
│         -pix_fmt rgba -v quiet pipe:1       │
│  → stdout 管道输出 RGBA 帧 (带时间戳)        │
└─────────────────────────────────────────────┘
    ↓ 每帧
queue.write_texture() → wgpu Texture → 全屏 quad 渲染
    ↓ 同时
┌─ FFmpeg 子进程 (音频) ──────────────────────┐
│  ffmpeg -i input.webm -f f32le              │
│         -acodec pcm_f32le -ac 2 -ar 44100   │
│         -v quiet pipe:1                     │
│  → stdout 管道输出 PCM 音频数据              │
└─────────────────────────────────────────────┘
    ↓
rodio Sink 播放（duck 现有 BGM）
    ↓ 播放完毕 / 玩家跳过
Signal("cutscene") → Runtime 恢复执行
```

### 4.2 VideoPlayer 模块设计

新增 `host/src/video/` 模块：

```
host/src/video/
├── mod.rs          # VideoPlayer 公开接口
├── decoder.rs      # FFmpeg 子进程管理、帧迭代
└── audio.rs        # 视频音频轨提取与 rodio 播放
```

#### VideoPlayer 状态机

```
Idle → Starting → Playing → Finished
                    ↓
                  Skipped
```

```rust
pub enum VideoState {
    Idle,
    Starting,
    Playing {
        start_time: Instant,
        current_frame: Option<VideoFrame>,
    },
    Finished,
    Skipped,
}

pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,     // RGBA pixels
    pub timestamp: f64,    // seconds
}
```

#### 核心接口

```rust
pub struct VideoPlayer {
    state: VideoState,
    decoder: Option<VideoDecoder>,
    audio: Option<VideoAudio>,
    texture: Option<Arc<dyn Texture>>,
}

impl VideoPlayer {
    /// 开始播放视频
    pub fn start(
        &mut self,
        path: &str,
        texture_ctx: &TextureContext,
        audio_manager: &mut AudioManager,
    ) -> Result<(), VideoError>;

    /// 每帧更新：推进解码、选择当前帧、更新纹理
    /// 返回 true 表示播放中，false 表示结束
    pub fn update(&mut self, dt: f32) -> bool;

    /// 跳过当前视频
    pub fn skip(&mut self);

    /// 获取当前帧的绘制命令（全屏 quad）
    pub fn draw_command(&self, screen_w: f32, screen_h: f32) -> Option<DrawCommand>;

    /// 是否已结束（完成或跳过）
    pub fn is_finished(&self) -> bool;

    /// 清理资源（停止子进程、释放纹理）
    pub fn cleanup(&mut self);
}
```

### 4.3 视频解码策略

**双子进程架构**：

1. **视频进程**：`ffmpeg -i <path> -f rawvideo -pix_fmt rgba -v quiet pipe:1`
   - 输出原始 RGBA 帧到 stdout
   - ffmpeg-sidecar 的 `filter_frames()` 逐帧迭代，提供 `timestamp`

2. **音频进程**：`ffmpeg -i <path> -f f32le -acodec pcm_f32le -ac 2 -ar 44100 -v quiet pipe:1`
   - 输出 f32 PCM 到 stdout
   - 读取后封装为 rodio `Source` 播放

**帧调度**：

- 记录播放开始时间 `start_time`
- 每帧计算 `elapsed = Instant::now() - start_time`
- 从解码队列中取 `timestamp <= elapsed` 的最新帧
- 跳过已过期的帧（丢帧策略，保证音视频同步）

### 4.4 音频集成

```rust
struct VideoAudio {
    sink: rodio::Sink,
    // 音频子进程句柄，用于 skip 时终止
    process: Option<Child>,
}
```

播放开始时：
1. 调用 `AudioManager::duck()` 压低当前 BGM
2. 创建音频子进程，PCM 数据 → rodio `Sink`
3. 视频结束/跳过时调用 `AudioManager::unduck()` 恢复 BGM

### 4.5 渲染集成

视频帧渲染复用现有 `SpriteRenderer` 管线：

1. `VideoPlayer::update()` 解码新帧后 → `queue.write_texture()` 更新 wgpu 纹理
2. `VideoPlayer::draw_command()` 返回 `DrawCommand::Sprite`（全屏居中，保持视频宽高比）
3. `WgpuBackend::render_frame()` 中视频绘制在最顶层（覆盖背景/角色/UI）

### 4.6 FFmpeg 可用性检测

```rust
/// 检测 FFmpeg 是否可用
fn detect_ffmpeg() -> FfmpegAvailability {
    // 1. 检查 PATH 中是否有 ffmpeg
    // 2. 检查预设路径（如 ./ffmpeg, ./bin/ffmpeg）
    // 3. 返回路径或 NotFound
}

pub enum FfmpegAvailability {
    Available(PathBuf),
    NotFound,
}
```

应用启动时检测一次，缓存结果。`cutscene` 命令执行时若不可用则 `tracing::warn!` 并直接发送 `Signal("cutscene")` 跳过。

---

## 5. vn-runtime 改动

### 5.1 AST

在 `ScriptNode` 枚举中新增：

```rust
/// 视频过场
///
/// 对应 `cutscene "path"` 语法。
/// 全屏播放视频，播放完毕或跳过后继续执行。
Cutscene {
    /// 视频文件路径（相对于脚本目录）
    path: String,
},
```

`causes_wait()` 返回 `true`（进入 `WaitForSignal`）。

### 5.2 Parser

在 phase2 中匹配 `cutscene` 关键字，解析引号内路径：

```
cutscene "audio/ending_HVC_bgm.webm"
```

### 5.3 Command

新增信号常量和命令变体：

```rust
/// cutscene 播放完成的信号 ID
pub const SIGNAL_CUTSCENE: &str = "cutscene";

/// 播放过场视频
Cutscene {
    /// 视频文件路径
    path: String,
},
```

### 5.4 Executor

AST → Command 映射：

```rust
ScriptNode::Cutscene { path } => {
    let resolved = script.resolve_path(&path);
    commands.push(Command::Cutscene { path: resolved });
    waiting = WaitingReason::WaitForSignal(SIGNAL_CUTSCENE.into());
}
```

---

## 6. Host 改动

### 6.1 CommandExecutor

收到 `Command::Cutscene` 后产出输出事件，由 `app/command_handlers` 消费：

```rust
Command::Cutscene { path } => {
    output_events.push(OutputEvent::StartCutscene { path });
}
```

### 6.2 command_handlers

```rust
OutputEvent::StartCutscene { path } => {
    app_state.video_player.start(&path, &texture_ctx, &mut audio_manager)?;
}
```

### 6.3 主循环集成

在 `app/update` 中：

```rust
// 视频播放中 → 更新视频、拦截普通输入
if app_state.video_player.is_playing() {
    let still_playing = app_state.video_player.update(dt);

    // 玩家点击/Enter/Esc → 跳过
    if input.is_skip_requested() {
        app_state.video_player.skip();
    }

    if !still_playing || app_state.video_player.is_finished() {
        app_state.video_player.cleanup();
        // 发送信号恢复 Runtime
        runtime.handle_input(Input::Signal("cutscene".into()));
    }

    return; // 视频播放期间不执行其他更新
}
```

### 6.4 绘制集成

在 `app/draw` 或 `WgpuBackend::render_frame` 中：

```rust
// 视频播放中 → 只渲染视频帧（覆盖所有其他内容）
if let Some(cmd) = app_state.video_player.draw_command(screen_w, screen_h) {
    // 先清屏为黑色
    // 然后绘制视频帧（居中，保持宽高比）
    video_commands.push(cmd);
}
```

### 6.5 Skip 模式兼容

```rust
// Skip 模式下立即跳过 cutscene
if playback_mode == PlaybackMode::Skip {
    app_state.video_player.skip();
    app_state.video_player.cleanup();
    runtime.handle_input(Input::Signal("cutscene".into()));
}
```

---

## 7. 错误处理

| 失败场景 | 策略 | 分类 |
|---------|------|------|
| FFmpeg 二进制不存在 | `warn!` + 立即发送 Signal 跳过 | 不受信（外部） |
| 视频文件不存在 | `warn!` + 立即发送 Signal 跳过 | 不受信（外部） |
| FFmpeg 子进程崩溃 | 检测退出码，`warn!` + cleanup + Signal 跳过 | 不受信（外部） |
| 帧解码失败（管道断裂） | 视为播放结束，cleanup + Signal | 不受信（外部） |
| 纹理创建失败 | 跳过当前帧，继续尝试后续帧 | 不受信（外部） |
| 音频提取失败 | `warn!` + 仅播放视频（静音） | 不受信（外部） |

核心原则：视频播放失败**绝不阻塞剧情流程**。所有失败路径都最终发送 `Signal("cutscene")` 恢复 Runtime。

---

## 8. 分阶段实施计划

### Phase 0：vn-runtime 链路 -- DONE

- [x] AST：新增 `ScriptNode::Cutscene { path }`
- [x] `causes_wait()` 返回 `true`
- [x] Parser：解析 `cutscene "path"` 语法
- [x] Command：新增 `SIGNAL_CUTSCENE` 常量 + `Command::Cutscene { path }`
- [x] Executor：AST → Command + `WaitForSignal("cutscene")`
- [x] Diagnostic：新增 `ResourceType::Video`，`extract_from_nodes` 提取视频资源引用
- [x] Host CommandExecutor：`Command::Cutscene` 分支（暂为 noop，Phase 2 实现）
- [x] 单元测试：8 个新增（6 parser + 2 executor）
- [x] **门控**：`cargo check-all` 通过（fmt + clippy + 496 tests）

### Phase 1：视频解码基础设施 -- DONE

- [x] 添加 `ffmpeg-sidecar` 依赖（`default-features = false`，不含 download_ffmpeg）
- [x] 新建 `host/src/video/` 模块（mod.rs / decoder.rs / audio.rs）
- [x] 实现 FFmpeg 可用性检测（`detect_ffmpeg()`：vendor → bin → PATH）
- [x] 实现 `VideoDecoder`：后台线程 + FfmpegCommand + filter_frames → mpsc channel
- [x] 实现 `VideoAudio`：后台线程 FFmpeg PCM 提取，`take_samples()` 供 Phase 2 集成
- [x] 实现 `VideoPlayer` 状态机（Idle/Playing/Finished/Skipped）+ 时间戳帧调度
- [x] FFmpeg 直出 RGBA（`-pix_fmt rgba`），消除 CPU 侧格式转换
- [x] Windows `CREATE_NO_WINDOW` 防止弹出控制台窗口
- [x] **门控**：`cargo check-all` 通过（fmt + clippy + 503 tests）

### Phase 2：渲染与主循环集成（预计 1 天） -- DONE

- [x] `script.rs` 拦截 `Command::Cutscene`，启动 `VideoPlayer`，duck BGM
- [x] `update` 循环集成：视频帧推进、音频启动、完成检测
- [x] `update_ingame` 视频播放期间输入拦截 + 跳过（Esc/Enter/Space/Click/Ctrl/Skip模式）
- [x] `draw` 集成：`WgpuBackend` 视频纹理上传 + 全屏渲染（信箱模式保持宽高比）
- [x] 音频集成：`AudioManager::play_video_audio()` 通过 rodio SamplesBuffer 播放 PCM
- [x] `finish_cutscene()` 统一清理：cleanup + unduck + signal
- [x] **门控**：`cargo check-all` 通过

### Phase 3：优雅降级与验收（预计 0.5 天） -- DONE

- [x] FFmpeg 不可用时的优雅降级（start 失败 → warn → 下一帧 finish_cutscene → 恢复 Runtime）
- [x] 视频文件不存在时的优雅降级（同上路径）
- [x] 子进程异常退出处理（decoder 线程 error log → finished 标记 → is_done → finish_cutscene）
- [x] 音频提取失败降级（静音播放视频）
- [x] 端到端验收：cutscene 视频正常播放、跳过、恢复流程
- [x] 更新文档（导航地图、模块摘要、summary_index）
- [x] **门控**：`cargo check-all` 通过

### Post-launch 修复

- [x] 性能优化：FFmpeg 直出 RGBA 消除 `rgb_to_rgba` CPU 转换（47% CPU → <5%）
- [x] 内存泄漏修复：unbounded channel → `sync_channel(2)` 有界帧缓冲
- [x] 跳过死锁修复：`stop()` 先 drop receiver 再 join 解码线程，解除 `sync_channel::send()` 阻塞
- [x] 发布打包：`detect_ffmpeg()` 新增可执行文件同目录检测；`asset-packer release` 自动复制 FFmpeg 二进制
- [x] ZIP 模式字体加载：打包后从 ZIP 读取字体文件，不再依赖 `assets/` 目录存在

---

## 9. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| FFmpeg 二进制体积（~100MB 完整版） | 游戏包体增大 | 使用精简构建（仅含 VP8/VP9/Vorbis/Opus 解码器，约 20-30MB）；或标注为可选依赖 |
| 用户环境无 FFmpeg | 视频不可播放 | 优雅降级 + 启动时检测 + 文档说明；可附带 ffmpeg 二进制或提供下载指引 |
| 管道吞吐在极低端 PC 上不足 | 视频卡顿/丢帧 | 丢帧策略保证音频流畅；1080p 30fps 吞吐需求仅 ~186MB/s，远低于管道上限 |
| FFmpeg 子进程启动延迟 | 视频开始前短暂黑屏 | 可在 `cutscene` 命令收到后立即启动子进程（预热），前置黑场 transition 天然掩盖 |
| 音视频同步漂移 | 播放体验降级 | 基于墙钟时间调度帧显示 + 丢帧策略；对 VN cutscene（通常 <3 分钟）漂移极小 |

---

## 10. FFmpeg 分发策略（已确认）

**决策：随包体分发。**

### 10.1 仓库位置

FFmpeg 二进制放入 `vendor/ffmpeg/`，按平台组织：

```
vendor/ffmpeg/
├── win-x64/
│   └── ffmpeg.exe
├── linux-x64/    # 未来
│   └── ffmpeg
└── README.md     # 版本、来源、许可证说明
```

`vendor/ffmpeg/` 加入 `.gitignore`（二进制不入 Git），由开发者手动放置或通过脚本下载。

### 10.2 构建与打包

- **开发阶段**：`detect_ffmpeg()` 搜索顺序：`vendor/ffmpeg/{platform}/` → 可执行文件同目录 → `bin/` → 系统 PATH
- **发布打包**：`cargo run -p asset-packer -- release` 自动检测并复制 FFmpeg 二进制到发行版目录（与游戏 exe 同级）
- **运行时**：`detect_ffmpeg()` 通过 `std::env::current_exe()` 检测可执行文件同目录，发布版 FFmpeg 放在 exe 旁即可

---

## 11. 未来扩展路径

本架构为后续增强留有空间，但均不在本 RFC 范围内：

- **多格式支持**：ffmpeg-sidecar 天然支持所有 FFmpeg 格式，零代码改动
- **视频预加载**：提前启动解码子进程，减少播放延迟
- **硬件加速指定**：通过 FFmpeg 参数 `-hwaccel auto` 启用
- **WASM 支持**：未来可回退到浏览器原生 `<video>` 标签或 WebCodecs API
- **去 FFmpeg 依赖**：若纯 Rust VP8/VP9 解码器成熟，可替换 decoder 层而不影响上层接口

---

## 12. 验收标准（Definition of Done）

- [x] `cutscene "path"` 在 vn-runtime 中完整解析与执行
- [x] `ending.md` 的 cutscene 视频正常播放（全屏、音视频同步）
- [x] 播放期间点击/Enter/Esc 可跳过
- [x] Skip 模式下立即跳过
- [x] 播放完毕/跳过后脚本流程正常恢复（`stopBGM` → `end`）
- [x] FFmpeg 不可用时优雅降级（warn + 跳过，不崩溃）
- [x] 视频文件不存在时优雅降级
- [x] `cargo check-all` 通过
- [x] 导航地图、模块摘要已更新

---

## 13. 时间预估

| 阶段 | 预估 | 累计 |
|------|------|------|
| Phase 0: vn-runtime 链路 | 0.5 天 | 0.5 天 |
| Phase 1: 视频解码基础设施 | 1 天 | 1.5 天 |
| Phase 2: 渲染与主循环集成 | 1 天 | 2.5 天 |
| Phase 3: 优雅降级与验收 | 0.5 天 | 3 天 |

总计约 **3 工作日**。
