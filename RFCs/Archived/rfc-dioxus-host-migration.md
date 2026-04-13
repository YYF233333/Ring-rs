# RFC-033: Dioxus 宿主迁移——消除 IPC 边界与双语言工具链

## 元信息

- 编号：RFC-033
- 状态：**Accepted**
- 作者：claude-4.6-opus
- 日期：2026-04-02
- 完成日期：2026-04-13
- 相关范围：`host/`（功能基线）、`host-dioxus/`（新宿主）、`.cargo/config.toml`、`tools/xtask/`
- 前置：无（独立于 RFC-032；RFC-032 已标记为 Superseded）

---

## 背景

### Tauri 方案的回顾与终止

`host-tauri` 迁移启动半个月以来，基础宿主闭环已跑通（`vn-runtime → CommandExecutor → RenderState → Vue`、34 个 IPC 命令已适配、CSS 动画效果可用）。但开发体验出现了三个结构性问题：

1. **双语言工具链摩擦**：Rust + TypeScript/Vue 需要 cargo + pnpm + vite + biome + vue-tsc 五套独立工具，每次迭代需切换目录和心智模型。在 JS 生态内已无法进一步优化。

2. **IPC 边界成本**：Tauri 的 `invoke` 机制要求所有前后端交互经过 JSON 序列化/反序列化。`RenderState` 的 Rust 结构体定义（`render_state.rs`）必须与 TypeScript 类型定义（`render-state.ts`）手动保持同步。`commands.rs` 中 34 个薄代理函数纯粹是胶水代码。

3. **调试架构矛盾**：为实现浏览器 MCP 无人调试，引入了 `debug_server.rs`（498 行），但它与 Tauri WebView 形成双客户端竞争（`lessons-learned.md` 已记录）。解决方案（`RING_HEADLESS=1`）是权宜之计。

根本原因：**Tauri 的架构天然将 Rust 后端与 WebView 前端隔离为两个独立进程/线程**，而我们的前端是纯渲染层（不持有游戏状态），这个隔离带来的只有成本没有收益。

**决策：host-tauri 前端即日冻结，不再投入新功能开发，等待 host-dioxus 完成后一并废弃。** host-tauri 可作为实现参考，但不构成本次迁移的目标管理基线。

### 迁移目标基线：旧 host（egui）

本次迁移的功能对齐基线是**旧 host**（`host/`，基于 winit + wgpu + egui）。旧 host 是当前功能最完整、经过验证的宿主实现，具体能力基线见 RFC-002（重制体验等价计划）和 `docs/maintenance/host-migration-gap-analysis.md`。

旧 host 已实现的核心能力（即本次迁移的功能目标）：

**视觉演出**：
- 基础转场：dissolve（双层交叉淡化）、fade、fadewhite、rule_mask（WebGL shader 遮罩过渡）
- 角色调度：show/hide/move + 入场淡入 + z-order 层级管理
- sceneEffect 首批 capability：shake/blur/dim（含时长驱动动画，非瞬时值设定）
- sceneEffect 高级 capability：focusPush/pushIn/panRight/resetCamera/skyPan/slowVerticalPan/imageWipe/flashbackIn/flashbackOut（P1 待实现，但架构须支持）
- titleCard：全屏字卡 + 淡入淡出
- changeScene 多阶段过渡状态机（Fade/FadeWhite/Rule）
- cutscene 视频播放（FFmpeg sidecar 解码）

**文本与节奏**：
- wait/pause 等待指令（含 Skip 跳过语义）
- 节奏标签（`{wait}`/`{speed}`/`-->`）
- extend 台词续接
- 窗口显隐控制（textBoxHide/textBoxShow/textBoxClear）
- Skip/Auto/Normal 三模式 + `skip_all_active_effects()` 统一跳过活跃演出

**音频**：
- BGM 播放/停止/交叉淡化
- SFX 一次性播放
- bgmDuck/bgmUnduck
- 三通道混音（music/sound/voice）独立音量控制（P1 待实现）

**系统 UI 与持久化**：
- 核心页面：主菜单、设置、历史、存档/读档、游内菜单（含季节切换）
- 存读档：槽位/缩略图/Continue 生命周期
- 持久化变量域（`$persistent`）
- fullRestart 与首通门控

**宿主 Harness**：
- 后端 authoritative 的宿主编排（HostSessionMode 状态机）
- deterministic 驱动（headless 模式复用同一套 update 链路）
- 小游戏桥接 API（音频、状态读写、完成回传）
- 地图 hit-mask

### Dioxus 0.7 的关键能力

Dioxus 0.7（2026-03-28 发布 0.7.4）提供了一条消除上述 Tauri 结构性问题的路径：

- **Dioxus Desktop**：底层使用与 Tauri 相同的 wry + tao，但 Rust 逻辑与 UI 代码运行在**同一进程**，无 IPC 序列化。
- **RSX 宏**：在 Rust 中直接编写类 JSX 的声明式 UI，编译期类型检查。
- **Subsecond 热补丁**：编辑 Rust 代码后 <200ms 生效，不丢失应用状态（`dx serve --hotpatch`）。
- **自定义协议**：`with_custom_protocol()` 支持本地资源加载，等价于 Tauri 的 `ring-asset` 协议。
- **GitHub 35.5K stars**，400 贡献者，YC 种子轮资助。

---

## Dioxus 框架评估

### 平台模式说明

Dioxus 是跨平台框架，同一份 RSX 代码可编译到不同目标。本项目使用 **Desktop Mode**：

| | Desktop Mode（本项目使用） | Web Mode（参考） |
|---|---|---|
| 编译产物 | 原生可执行文件（.exe / .app） | WebAssembly (.wasm) |
| Rust 代码运行位置 | 操作系统原生进程 | 浏览器 WASM 沙箱 |
| UI 渲染引擎 | 系统内嵌 WebView（Win=WebView2/Chromium, macOS=WKWebView, Linux=WebKitGTK） | 浏览器本身 |
| 浏览器 API（DOM/WebGL/Canvas） | WebView 内完整可用，但 Rust 侧需通过 `document::eval()` 桥接 JS 执行 | 通过 `web_sys` 直接可用 |
| 系统 API（文件系统/多线程） | 完全可用 | 受 WASM 沙箱限制 |

**关键点**：Desktop Mode 下 WebView 是一个完整的浏览器引擎，**HTML/CSS/JavaScript/WebGL 全部可用**。Rust 代码不能直接调用 `document.getElementById()` 等浏览器 API（因为 Rust 跑在原生进程而非 WASM），但可以通过 `document::eval()` 将 JS 代码发送到 WebView 执行，并通过 `send()`/`recv()` 双向通信。

### 版本稳定性风险

Dioxus 处于快速迭代阶段，每 6-10 个月发布一个大版本，**每个大版本都有 breaking changes**：

| 版本 | 日期 | 破坏性程度 | 关键变化 |
|------|------|------------|----------|
| 0.5 | 2024-03 | 高 | Signal 重写，整个状态管理范式变更 |
| 0.6 | 2024-12 | 中高 | Element 类型变更、prevent_default API 变更 |
| 0.7 | 2025-10 | 中 | Subsecond 热补丁、Fullstack Axum 集成、多处接口调整 |

**评估**：未来 0.8 大概率还会有 breaking changes。但核心收益（消除 IPC 边界、消除双语言工具链）是结构性的，不会因 API 变更而丧失。升级时需要投入适配成本，但这比维护双语言工具链的持续成本更可控。

### 生态与社区

- **组件库匮乏**：无成熟的第三方组件库。对本项目影响有限——VN 引擎的 UI 全部自定义。
- **文档质量参差**：社区称为"death by 1000 papercuts"，小问题多、示例偶尔过时。迁移过程中可能需要翻阅源码。
- **项目可持续性**：YC 种子轮 50 万 + 4 人团队。相比 Tauri（CrabNebula 公司支撑、104K stars）规模较小，但版本发布节奏稳定。

### 已确认的关键能力

| 能力 | 可用性 | 说明 |
|------|--------|------|
| CSS transition/animation | ✅ 完全可用 | WebView 内完整 CSS 支持 |
| WebGL 2.0 + GLSL shader | ✅ 完全可用 | WebView2/WKWebView/WebKitGTK 均支持 WebGL 2.0 |
| HTML5 `<video>` | ✅ 完全可用 | 支持 range request 流式加载 |
| 自定义协议 | ✅ 完全可用 | `with_custom_protocol()` / `with_asynchronous_custom_protocol()` |
| JS 执行 | ✅ 完全可用 | `document::eval()` 双向通信 |
| Signal 状态管理 | ✅ 适合 | fine-grained reactivity + Stores（嵌套状态） |
| Scoped CSS | ⚠️ 需手动管理 | 无内置 scoped，用 BEM 命名 + 全局 CSS 文件 |

---

## 目标与非目标

### 目标

- 用 Dioxus Desktop 替代旧 host（egui）和 host-tauri（Tauri + Vue），消除 IPC 边界和双语言工具链。
- **功能对齐旧 host 的完整能力基线**（见"迁移目标基线"章节），包括视觉演出、文本节奏、音频、系统 UI、宿主 harness。
- 前端 UI 用 Rust RSX 重写，共享 `RenderState` 等核心类型，不再需要手动类型同步。
- RuleTransition 使用 **WebGL shader** 实现（对齐旧 host 的 GPU 遮罩过渡），不沿用 host-tauri 的 CPU Canvas 2D 权宜方案。
- 保持 headless harness 能力（`debug_run_until`、`HarnessTraceBundle`）。
- 构建系统收敛为 `cargo` + `dx`（Dioxus CLI），消除 pnpm / node / biome / vue-tsc 依赖。
- **远期兼容**：架构须支持在 WebView 中嵌入 JS 游戏引擎（Pixi.js/Phaser 等），用于小游戏需求。

### 非目标

- 不使用 Dioxus Native（WGPU/Blitz 模式）。Blitz 的 CSS 支持仍不完整，我们需要完整的 WebView CSS 能力。
- 不在本 RFC 中重新设计 `AppStateInner` / `CommandExecutor` / `RenderState` 的内部逻辑。迁移目标是**同等功能，不同宿主壳**。
- 不追求与旧 host 的像素级一致。保留美术风格与信息层级即可（与 RFC-002 口径一致）。
- 不在本 RFC 中实现 P1 高级 sceneEffect capability（focusPush 等），但架构须预留 capability 注册扩展点。
- 不在本 RFC 中解决 VN+ Hub（RFC-024）的架构问题。
- 不迁移旧 host 的 `EventStream` JSONL 格式和输入录制/回放（已在 RFC-032 中明确为不迁移项）。

---

## 方案设计

### 架构总览

```
旧 host（egui）               host-tauri（冻结）            迁移后（Dioxus Desktop）
┌──────────────────┐  ┌──────────────┐  IPC  ┌──────────┐  ┌────────────────────────────┐
│ 单一 Rust 进程    │  │ Vue Frontend │◄────►│ Tauri RS │  │ 单一 Rust 进程              │
│                  │  │ (TypeScript) │      │ Backend  │  │                            │
│ egui UI ← State  │  │ 24 components│      │ commands │  │ RSX UI ←Signal→ AppStateInner│
│    ↓             │  │ render-state │      │ debug_   │  │    ↓                       │
│ wgpu 渲染        │  │     .ts      │      │ server   │  │ WebView 渲染 (CSS/WebGL)    │
└──────────────────┘  └──────────────┘      └──────────┘  └────────────────────────────┘
  ~15000 行 Rust        ~4300 行 TS/Vue       ~6000 行 RS     ~6000 行 Rust（后端复用）
                        + 6 个 JS 工具                        + ~3000 行 RSX（前端重写）
```

### 目录结构

```
host-dioxus/
├── Cargo.toml          # 单一 crate，依赖 dioxus + vn-runtime
├── Dioxus.toml         # dx CLI 配置
├── src/
│   ├── main.rs         # Dioxus 启动入口
│   ├── app.rs          # 根组件 + 路由
│   │
│   ├── state.rs        # 复用后端逻辑，AppStateInner（适配 Signal 接口）
│   ├── command_executor.rs  # 复用
│   ├── render_state.rs      # 复用（直接用 Rust struct，无类型同步）
│   ├── audio.rs        # 复用
│   ├── resources.rs    # 复用 + 适配 Dioxus custom protocol
│   ├── config.rs       # 复用
│   ├── manifest.rs     # 复用
│   ├── save_manager.rs # 复用
│   ├── error.rs        # 复用
│   ├── headless_cli.rs # 复用
│   │
│   ├── components/     # 通用 UI 组件（RSX）
│   │   ├── confirm_dialog.rs
│   │   ├── toast.rs
│   │   └── skip_indicator.rs
│   ├── screens/        # 系统界面（RSX）
│   │   ├── title.rs
│   │   ├── settings.rs
│   │   ├── save_load.rs
│   │   ├── history.rs
│   │   └── in_game_menu.rs
│   └── vn/             # VN 渲染层（RSX）
│       ├── scene.rs
│       ├── background.rs
│       ├── character.rs
│       ├── dialogue.rs
│       ├── choice.rs
│       ├── transition.rs        # CSS fade 过渡
│       ├── rule_transition.rs   # WebGL shader 遮罩过渡
│       ├── video.rs
│       ├── nvl.rs
│       ├── chapter_mark.rs
│       ├── map_overlay.rs       # 地图 hit-mask
│       ├── minigame.rs          # 小游戏 iframe 容器
│       └── quick_menu.rs
└── assets/
    ├── style.css       # 全局样式（BEM 命名）
    └── shaders/
        └── rule_transition.glsl  # 遮罩过渡 fragment shader
```

### 状态管理：Signal 替代 IPC

当前 Tauri 架构中，前端通过 `callBackend("get_render_state")` IPC 轮询获取状态。Dioxus 中改为 Signal 直接共享：

```rust
fn App() -> Element {
    let state = use_signal(|| AppStateInner::new());

    let mut state_w = state.clone();
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(16)).await;
            state_w.write().process_tick(1.0 / 60.0);
        }
    });

    let rs = state.read();
    rsx! {
        div { class: "game-container",
            BackgroundLayer { bg: rs.render_state.background.clone() }
            CharacterLayer { characters: rs.render_state.characters.clone() }
            DialogueBox {
                dialogue: rs.render_state.dialogue.clone(),
                on_click: move |_| state.write().process_click(),
            }
        }
    }
}
```

### 资源加载：自定义协议

```rust
fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_custom_protocol("ring-asset", move |request| {
                    let path = request.uri().path();
                    let data = resource_manager.read_bytes(path);
                    // 返回 Response
                })
        )
        .launch(App);
}
```

RSX 中使用：`img { src: "ring-asset://localhost/{path}" }`

### RuleTransition：WebGL Shader 遮罩过渡

对齐旧 host 的 GPU 实现方案，不沿用 host-tauri 的 CPU Canvas 2D 权宜方案。

**Desktop Mode 下 WebGL 可用性**：WebView（WebView2/WKWebView/WebKitGTK）完整支持 WebGL 2.0 + GLSL shader。Rust 侧通过 `document::eval()` 注入 JS 代码初始化 WebGL context 和编译 shader。

**架构**：
- Rust 侧：计算过渡进度（float），通过 `eval.send()` 传递给 WebView
- JS 侧：初始化 WebGL context → 编译 vertex/fragment shader → 绑定遮罩纹理 → 每帧更新 progress uniform → 绘制
- 性能：只需传递一个 float 值，eval 桥接开销可忽略

```rust
fn RuleTransition(mask_path: String, progress: f32) -> Element {
    let canvas_id = use_signal(|| format!("rule-canvas-{}", uuid()));

    // 初始化 WebGL（仅首次 mount）
    use_effect(move || {
        let id = canvas_id.read().clone();
        let mask_url = format!("ring-asset://localhost/{mask_path}");
        document::eval(&format!(r#"
            const canvas = document.getElementById("{id}");
            const gl = canvas.getContext("webgl2");
            // ... 编译 shader、加载遮罩纹理、绑定 uniform ...
            window.__ruleGl_{id} = {{ gl, progressLoc }};
        "#));
    });

    // 每帧更新 progress
    use_effect(move || {
        let id = canvas_id.read().clone();
        document::eval(&format!(r#"
            const state = window.__ruleGl_{id};
            if (state) {{
                state.gl.uniform1f(state.progressLoc, {progress});
                state.gl.drawArrays(state.gl.TRIANGLE_STRIP, 0, 4);
            }}
        "#));
    });

    rsx! {
        canvas {
            id: "{canvas_id}",
            width: "960", height: "540",
            class: "vn-rule-canvas",
        }
    }
}
```

### 视频播放

直接使用 HTML5 `<video>`：

```rust
fn VideoOverlay(video_path: String, on_finished: EventHandler) -> Element {
    rsx! {
        div { class: "vn-video-overlay", onclick: move |_| on_finished.call(()),
            video {
                src: "ring-asset://localhost/{video_path}",
                autoplay: true,
                onended: move |_| on_finished.call(()),
            }
        }
    }
}
```

### 小游戏桥接：JS 引擎兼容性

远期小游戏需求要求在 WebView 中嵌入 JS 游戏引擎（Pixi.js / Phaser 等）。这在 Desktop Mode 下完全可行，因为 WebView 是一个完整的浏览器引擎。

**架构方案**：小游戏运行在 `<iframe>` 中，通过 `ring-asset` 协议加载本地 HTML/JS 资源，Rust ↔ JS 通过 `document::eval()` 的 `send()`/`recv()` 或注入全局函数通信。

```rust
fn MiniGameOverlay(game_path: String, on_complete: EventHandler<GameResult>) -> Element {
    // 监听小游戏完成事件
    use_future(move || async move {
        let result = document::eval(r#"
            await dioxus.recv()  // 等待 iframe postMessage 转发的完成信号
        "#).recv::<GameResult>().await;
        on_complete.call(result);
    });

    rsx! {
        div { class: "vn-minigame-overlay",
            iframe {
                src: "ring-asset://localhost/{game_path}/index.html",
                width: "100%", height: "100%",
            }
        }
    }
}
```

旧 host 的小游戏桥接 API 面（音频控制、状态读写、完成回传）须在 `engine-sdk.js` 中完整恢复，不得像 host-tauri 那样缩水。

### CSS 策略

不使用 Vue 的 `<style scoped>`，也不引入 Tailwind。替代方案：

1. **全局 CSS 文件**（`assets/style.css`）：定义所有 `.vn-*` / `.screen-*` 样式。
2. **BEM 命名约定**：`vn-dialogue__text`、`vn-character--entering` 等，避免样式冲突。
3. **CSS 变量**：保留当前 `--vn-ease-scene`、`--vn-font-body` 等设计 token。
4. **CSS 动画**：`@keyframes`、`transition` 属性直接可用（WebView 内完整 CSS 支持）。

### 构建与开发

```toml
# Dioxus.toml
[application]
name = "ring-engine"
default_platform = "desktop"
```

| 操作 | 旧 host | host-tauri | Dioxus 后 |
|---|---|---|---|
| 开发运行 | `cargo run -p host` | `cd host-tauri; pnpm tauri dev` | `dx serve` |
| 热重载 | 无 | Vite HMR (JS) + cargo rebuild (RS) | `dx serve --hotpatch`（<200ms） |
| 发布构建 | `cargo build -p host --release` | `cd host-tauri; pnpm tauri build` | `dx build --release` |
| 格式化 | `cargo fmt` | `cargo fmt` + `pnpm check:write` | `cargo fmt` |
| Lint | `cargo clippy` | `cargo clippy` + biome | `cargo clippy` |
| 类型检查 | `rustc` | `rustc` + `vue-tsc` | `rustc` |

### Headless / 调试

`headless_cli.rs` 和 `debug_run_until()` 逻辑完全复用。由于 UI 和后端在同一进程，不再存在"双客户端竞争"问题。

| 场景 | 方案 |
|---|---|
| 逻辑调试（Agent） | `headless_cli` → `HarnessTraceBundle` JSON（不变） |
| 视觉调试（Agent） | `dx serve` 启动后，浏览器 MCP 连接 Dioxus 的 dev server |
| 视觉调试（人类） | `dx serve`，直接看窗口 |

关键改善：Dioxus 的 dev server 在开发模式下会在 localhost 暴露 WebSocket 热重载端口。浏览器 MCP 可以直接连接这个端口查看实时 UI，**无需额外的 debug_server**。

---

## 与旧 host 的功能对齐清单

以下基于 `host-migration-gap-analysis.md` 和 RFC-002，列出本次迁移需要对齐的旧 host 能力。**对齐标准是玩家可感知行为，不是实现细节的一比一复制**（如不复制 wgpu/egui/winit 的平台落地细节）。

### 必须对齐（PoC + Phase 1-2）

| 领域 | 旧 host 能力 | Dioxus 实现路径 |
|------|-------------|----------------|
| 背景 dissolve | 双层交叉淡化 | CSS transition: opacity + 双 `<img>` 层 |
| 角色 show/hide | 入场淡入 + z-order | CSS transition + RSX 动态排序 |
| 场景过渡状态机 | Fade/FadeWhite/Rule 多阶段 | CSS overlay + WebGL shader（Rule） |
| rule_mask 过渡 | GPU shader 遮罩 | WebGL 2.0 shader via eval 桥接 |
| sceneEffect 基础 | shake/blur/dim（时长驱动） | CSS animation + filter |
| titleCard | 全屏字卡 + 淡入淡出 | CSS animation |
| Skip 语义 | skip_all_active_effects() 统一收敛 | Signal 驱动，清除所有活跃动画状态 |
| 文本/节奏 | wait/pause/节奏标签/extend | 复用 vn-runtime 逻辑 |
| 音频 | BGM/SFX/duck/crossfade | 复用 audio.rs |
| 视频 | cutscene | HTML5 `<video>` |
| 存读档 | 槽位/缩略图/Continue | 复用 save_manager.rs |
| 系统 UI | 主菜单/设置/历史/存读档/游内菜单 | RSX 重写 |
| 宿主 authority | HostSessionMode 状态机 | 复用 state.rs |
| 资源访问 | FS/ZIP 透明 | custom protocol + ResourceManager |
| config/manifest | strict bootstrap | 复用 config.rs/manifest.rs |
| headless | deterministic 驱动 | 复用 headless_cli.rs |

### 延后对齐（P1-P2，架构须预留扩展点）

| 领域 | 旧 host 能力 | 说明 |
|------|-------------|------|
| sceneEffect 高级 | focusPush/panRight/skyPan 等 | 架构预留 capability 注册点 |
| 三通道混音 | music/sound/voice 独立音量 | P1 |
| 地图 hit-mask | 复杂热点区域 | `map_overlay.rs` 预留 |
| 小游戏桥接 | 完整 API 面 | `minigame.rs` + `engine-sdk.js` |
| 选项键盘导航 | ArrowUp/Down/Enter | P2 |
| 历史信息密度 | 章节/事件语义 | P2 |

### 明确不迁移

| 项目 | 原因 |
|------|------|
| EventStream JSONL | 已被 debug_snapshot + trace bundle 替代 |
| 输入录制/回放 | RFC-032 已明确舍弃 |
| egui 即时模式 UI | 技术栈替代，非功能 gap |
| wgpu 渲染管线 | 由 WebView CSS/WebGL 替代 |
| ExtensionRegistry/CapabilityId | 低优先级架构差距，非玩家可见 blocker |

---

## 影响范围

| 模块 | 改动 | 风险 |
|---|---|---|
| `host/`（旧 host） | 不改动，保持为功能参考基线 | 无 |
| `host-tauri/` | 冻结，迁移完成后整体删除 | 低 |
| `host-dioxus/src/*.rs`（后端） | 从 host-tauri 后端迁移，移除 Tauri 特有依赖 | 低——核心逻辑不变 |
| `host-dioxus/src/vn/*.rs`（渲染层） | RSX 重写，对齐旧 host 视觉语义 | 中——最大工作量所在 |
| `host-dioxus/src/screens/*.rs`（系统 UI） | RSX 重写旧 host 各页面 | 中 |
| `.cargo/config.toml` | 更新 alias | 低 |
| `tools/xtask/` | 移除 pnpm/biome/vue-tsc 调用 | 低——简化 |
| CI (`check-all.yml`) | 移除 Node/pnpm 步骤 | 低——简化 |
| 根 `Cargo.toml` workspace | 替换 `host-tauri/src-tauri` 为 `host-dioxus` | 低 |

---

## 已知风险与缓解

| 风险 | 严重度 | 缓解措施 |
|------|--------|----------|
| dx CLI 与 Cargo workspace 兼容性 | 中 | PoC 阶段优先验证 `dx serve` 在 workspace 中的工作情况 |
| Dioxus 0.8 breaking changes | 中 | 核心收益（消除 IPC/双语言）是结构性的，升级成本可控 |
| WebView2 WebGL 性能低于原生 Chrome（低端集显约 50%） | 低 | 960×540 遮罩过渡 shader 不是性能密集场景 |
| eval 桥接延迟影响 WebGL shader 初始化 | 低 | 仅初始化时一次性注入，运行时只传 float |
| Dioxus 项目可持续性（小团队） | 低 | YC 背书 + 稳定发布节奏；底层 wry/tao 与 Tauri 共享 |
| 无 Scoped CSS | 低 | BEM 命名约定足够，VN 引擎 CSS 规模有限 |

---

## 迁移计划

### Phase 0：PoC 验证（1-2 天）

在新分支上创建 `host-dioxus/` 骨架，验证：
- [x] Dioxus Desktop 窗口能正常启动（含 Cargo workspace 兼容性）
- [ ] `dx serve` 在 workspace 中工作正常（未测试，不影响后续阶段）
- [x] `ring-asset` 自定义协议能加载本地图片
- [x] CSS transition/animation 在 WebView 中正常工作
- [x] WebGL 2.0 shader 可在 WebView 中编译运行（GLSL fragment shader）
- [x] RuleTransition 的 WebGL 遮罩过渡可通过 eval 桥接驱动
- [x] HTML5 `<video>` 能播放
- [x] `AppStateInner` 能通过 Signal 驱动 UI 更新

**PoC 结论：通过（2026-04-04）。** 7/8 项验证通过，所有关键能力确认可用。

**发现的平台差异**（已沉淀至 `docs/maintenance/lessons-learned.md`）：

1. **自定义协议 URL 格式**：Windows 上 wry 使用 `http://{name}.localhost/` 格式，不是 `{name}://localhost/`。
2. **CSS 加载**：`with_custom_head("<style>...")` 内联 CSS，外部 `<link>` 在 `cargo run` 下不可靠。
3. **WebGL eval 桥接**：Rust 设全局变量，JS 用 `requestAnimationFrame` 自主渲染循环。不从 Rust 逐帧 eval（延迟导致闪烁）。

### Phase 1：后端迁移（1 天）— 完成（2026-04-10）

将后端文件迁移到 `host-dioxus/src/`：
- [x] `state.rs`、`command_executor.rs`、`render_state.rs`、`audio.rs`、`resources.rs`、`config.rs`、`manifest.rs`、`save_manager.rs`、`error.rs`、`headless_cli.rs`、`init.rs`
- [x] 移除 Tauri 特有依赖（`tauri::State`、`#[command]`），改为普通 Rust 模块接口
- [x] 不再需要 `commands.rs`（IPC 薄代理）和 `debug_server.rs`
- [x] 23 个 Command handler 全部就位
- [x] `AppState { inner: Arc<Mutex<AppStateInner>> }` 适配 Dioxus `use_context`

### Phase 2：前端重写（3-5 天）— 完成（2026-04-13）

按优先级重写为 RSX，**对齐旧 host 的视觉语义**。

**架构决策**：
- 状态接入：`AppState` 通过 `use_context_provider` 注入，tick loop（30 FPS）每帧 clone `RenderState` 到 `Signal`
- CSS 策略：单一内联 CSS（BEM 命名），1920×1080 基准 + `transform: scale()` 等比缩放
- Skip 模式：`.skip-mode` class 零化所有 `transition-duration` / `animation-duration`
- 数据驱动 UI：`screen_defs.rs`（screens.json）+ `layout_config.rs`（layout.json）

**最终文件结构**：
- `src/vn/`（13 个文件）：scene / background / character / dialogue / nvl / choice / transition / rule_transition / chapter_mark / title_card / video / quick_menu / audio_bridge
- `src/screens/`（5 个文件）：title / in_game_menu / save_load / settings / history
- `src/components/`（4 个文件）：skip_indicator / confirm_dialog / game_menu_frame / toast
- 后端模块（11 个文件）：state / command_executor / render_state / audio / resources / config / manifest / save_manager / error / headless_cli / init
- 数据驱动模块（2 个文件）：screen_defs / layout_config

**Phase 2 延后项状态**：
- [x] 音频桥接 — `vn/audio_bridge.rs`：JS Web Audio API via eval（BGM crossfade + SFX 一次性播放）
- [x] 键盘绑定 — `main.rs`：eval send/recv 双向通信（Escape/Ctrl-Skip/Space/Enter/A-Auto/Backspace-rollback）
- [x] Toast — `components/toast.rs`：4 种类型 + 2.8s 自动淡出（Phase 4 实现）
- [x] ConfirmDialog — `components/confirm_dialog.rs`：模态确认弹窗 + NinePatch frame 背景（Phase 4 实现）
- [ ] MapOverlay / MiniGame — placeholder 预留（延后对齐，架构已预留）

### Phase 3：构建集成与清理（1 天）— 完成（2026-04-13）

- [x] 更新 `.cargo/config.toml` alias（添加 `test-dioxus`；`default-members` 已指向 `host-dioxus`）
- [x] 确认 `tools/xtask/` 无需修改（check-all 已是纯 Rust：fmt → clippy → test）
- [x] CSS 已内联于 `main.rs`（BEM 命名全局样式）
- [x] 更新 `docs/engine/architecture/navigation-map.md`（新增 `host-dioxus/` 章节）
- [x] 更新 `docs/maintenance/lessons-learned.md`（新增 3 条 Dioxus 经验）
- [x] 更新 `docs/maintenance/summary-index.md`（添加 `host-dioxus` 条目）
- [x] 删除 `host-tauri/` 目录（2026-04-13，含 `.claude/rules/domain-host-tauri-*.md` 和 settings pnpm 权限清理）

### Phase 4：UI 精细化对齐 egui host — 完成（2026-04-13）

将 Dioxus host 的占位符级 UI 升级为与 egui host 完全功能等价的正式界面。

**基础设施**：
- [x] 基准分辨率 1920×1080 + CSS `transform: scale()` 等比缩放（等价于 egui `ScaleContext`）
- [x] 数据驱动 UI：新增 `screen_defs.rs`（从 `screens.json` 加载 `ConditionDef`/`ActionDef`/`ButtonDef`/`ScreenDefinitions`）、`layout_config.rs`（从 `layout.json` 加载 `UiLayoutConfig`）
- [x] CSS 变量体系（颜色 7 + 字号 7 token），NinePatch 通过 CSS `border-image` 实现
- [x] `execute_action()` + `condition_context()` 实现 ActionDef → app 操作完整映射

**界面对齐**：
- [x] 对话框：NinePatch textbox/namebox 背景，颜色翻转为图片背景黑字
- [x] 快捷菜单：数据驱动（screens.json 7 个中文按钮），对话框上方居中
- [x] 标题画面：条件背景图（summer/winter）+ overlay + 数据驱动按钮列表（条件显隐）
- [x] 选项面板：NinePatch choice_idle/hover 背景，宽 1185px，间距 33px
- [x] 确认弹窗：`ConfirmDialog` 组件，NinePatch frame 面板，接入所有危险操作
- [x] GameMenuFrame：通用框架（背景图+overlay+左导航+右内容），供 Save/Load/Settings/History 复用
- [x] Save/Load：嵌入 GameMenuFrame，Tab 切换，A/Q/1-9 分页，NinePatch slot，删除按钮
- [x] 游内菜单：数据驱动 7 个中文按钮，确认弹窗接入
- [x] Settings：嵌入 GameMenuFrame，静音复选框 + 应用按钮，滑块参数对齐
- [x] History：嵌入 GameMenuFrame，双列布局（角色名右对齐+对话文本）
- [x] Toast 提示：4 种类型 + 2.8s 自动淡出
- [x] Skip 指示器：中文文案 + 绿色背景
- [x] TitleCard/ChapterMark/NVL 样式对齐

### 向后兼容

- `vn-runtime` 完全不受影响
- 旧 `host`（egui）不受影响（保留为参考基线）
- 存档格式不变（`SaveData` 来自 `vn-runtime`）
- 脚本格式不变

---

## 验收标准

- [x] 游戏可以从标题 → 新游戏 → 对话推进 → 选项 → 存档 → 读档 → 返回标题完整流程运行
- [x] 背景 dissolve 为双层交叉淡化（对齐旧 host，非 host-tauri 的瞬时切换）
- [x] 角色入场淡入 + z-order 层级排序正确（对齐旧 host）
- [x] CSS 过渡动画（fade in/out）视觉效果与旧 host 等价
- [x] RuleTransition 使用 WebGL shader 实现遮罩过渡（对齐旧 host GPU 方案）
- [x] sceneEffect 基础 capability（shake/blur/dim）有时长驱动动画（对齐旧 host，非 host-tauri 的瞬时值）
- [x] Skip 模式可统一收敛所有活跃演出（对齐旧 host 的 `skip_all_active_effects()`）
- [x] HTML5 视频播放正常
- [x] `ring-asset` 自定义协议正确加载 FS 和 ZIP 资源
- [x] `cargo test -p host-dioxus` 通过（headless harness，29 个测试）
- [x] 构建工具链中不包含 Node.js / pnpm / biome / vue-tsc
- [x] `cargo check-all` 一条命令通过全部门禁（365 个测试）
- [x] 无 IPC 薄代理层、无手动类型同步文件
- [x] 文档已更新：module summaries、navigation map、lessons-learned
- [x] UI 数据驱动：screens.json（按钮/条件/动作）+ layout.json（布局参数/颜色/字号/资产路径）
- [x] 所有系统界面功能完全对齐 egui host（按钮数量/标签/条件显隐/页面切换逻辑）

---

## 相关 RFC

| RFC | 标题 | 状态 | 与本 RFC 关系 |
|-----|------|------|-------------|
| RFC-002 | ref-project 重制体验等价计划 | Active | 功能对齐基线来源 |
| RFC-032 | host-tauri Harness 能力对齐 | Superseded | Tauri 特有部分已被替代 |
| RFC-004 | 扩展 API 与 Mod 化效果管理 | Active | capability 注册表框架（延后对齐） |
| RFC-009 | Cutscene 视频播放 | Accepted | 视频播放方案参考 |
| RFC-024 | VN+ Hub 架构 | Proposed | 非目标，不在本 RFC 范围 |
