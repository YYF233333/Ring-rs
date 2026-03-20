# RFC: 调试覆盖层（Debug Overlay）

## 元信息

- 编号：RFC-017
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-18
- 相关范围：`host::config`、`host::input`、`host::host_app`、`host::egui_screens`、`host::backend`、`host::renderer::render_state`、`host::audio`、`vn-runtime::state`
- 前置：无

---

## 1. 背景

当前 Ring-rs 在视觉问题排查时存在一个常见痛点：当出现视觉 bug（如立绘错位、对话显示异常、过渡效果卡顿）时，开发者通常需要分别截取游戏画面和导出内部状态（日志、快照等），再人工对照分析。两者分离导致：

- **反馈链路长**：截图与状态 dump 可能不在同一时刻，难以精确对应；
- **上下文割裂**：模型或协作者无法从单张截图直接推断引擎内部状态；
- **开发效率低**：频繁切换工具、重复操作，打断调试节奏。

需要一个**调试覆盖层**：在游戏窗口上直接叠加渲染引擎内部状态，使单次截图即可同时包含“画面问题”与“对应状态”。这样在视觉 bug 发生时，用户按一次截图键即可获得完整现场，极大提升联调与 AI 辅助排障的效率。

---

## 2. 目标与非目标

### 2.1 目标

- 提供可切换显示的调试覆盖层，在游戏画面上叠加关键内部状态信息。
- 使用 **F9** 热键（或可配置）切换覆盖层显隐；仅在 `debug.enable_overlay: true` 时启用，发布版默认关闭。
- 覆盖层基于 **egui** 实现，复用现有 `build_ui` 管线，无需新增渲染路径。
- 信息面板采用可折叠分区，覆盖：脚本状态、对话、视觉状态、音频、性能、模式等。
- 覆盖层半透明、偏置布局（如侧边或角落），尽量不遮挡核心游戏内容。
- 配置项 `debug.enable_overlay: bool`（默认 `false`），与现有 `DebugConfig` 对齐。

### 2.2 非目标

- **非交互式调试器**：不实现单步执行、断点、变量编辑等调试功能。
- **非性能分析器**：不提供 GPU/CPU 细粒度计时、火焰图等 profiling 能力。
- **非主题定制**：覆盖层使用 egui 默认样式，不引入自定义主题系统。
- **非持久化**：覆盖层内容仅用于实时查看，不写入文件（状态快照由 RFC-015 负责）。

---

## 3. 方案设计

### 3.1 配置扩展（`DebugConfig`）

在 `host/src/config/mod.rs` 的 `DebugConfig` 中新增：

```rust
/// 是否启用调试覆盖层（F9 切换显示）
pub enable_overlay: bool,
```

默认值策略：

- `cfg!(debug_assertions)` 时可为 `false`（需显式开启，避免干扰日常开发）；
- release 构建下强制为 `false`，或通过配置显式开启（由实现决定，建议 release 下配置无效）。

可选扩展（首版可不实现）：

- `overlay_hotkey: Option<String>`：覆盖层热键，默认 `F9`。

### 3.2 触发与输入集成

- 在 `host_app.rs` 的 `RedrawRequested` 分支中，在 `app::update` 之后、`backend.render_frame` 之前，检测 F9 本帧按下（`InputManager::is_key_just_pressed(KeyCode::F9)`）。
- 若 `config.debug.enable_overlay == true`，切换覆盖层可见状态（如 `HostApp` 或 `DebugOverlay` 持有的 `visible: bool`）。
- 防抖：同一帧最多切换一次，避免重复触发。

### 3.3 渲染方式

- 覆盖层作为 **egui 窗口/面板**，在 `build_ui` 闭包内、在现有各 screen（title/ingame/save_load 等）之后、toast 之前渲染。
- 当 `visible == true` 时，调用 `DebugOverlay::draw(ctx, &state)`，传入必要的状态引用。
- 使用 `egui::Window` 或 `egui::TopBottomPanel`，设置半透明背景（如 `egui::Color32::from_black_alpha(180)`），置于屏幕一侧或角落，可拖拽调整位置（egui 默认支持）。
- 不修改 `WgpuBackend::render_frame` 签名，仅扩展 `build_ui` 闭包内容。

### 3.4 模块结构

新增模块：`host/src/debug_overlay.rs`（或 `host/src/ui/debug_overlay.rs`，与 `egui_screens` 同级或作为其子模块）。

```rust
/// 调试覆盖层
///
/// 在游戏画面上叠加内部状态，用于联调与排障。
/// 仅当 config.debug.enable_overlay 为 true 时可用，F9 切换显示。
pub struct DebugOverlay {
    pub visible: bool,
}

impl DebugOverlay {
    pub fn new() -> Self { ... }

    /// 绘制覆盖层（仅当 visible 时在 build_ui 中调用）
    pub fn draw(
        &self,
        ctx: &egui::Context,
        state: &DebugOverlayState,
    ) { ... }
}

/// 覆盖层所需的状态快照（由 host_app 在每帧构建）
pub struct DebugOverlayState<'a> {
    pub runtime: Option<&'a vn_runtime::VNRuntime>,
    pub render_state: &'a RenderState,
    pub waiting_reason: &'a WaitingReason,
    pub playback_mode: PlaybackMode,
    pub app_mode: AppMode,
    pub navigation_stack: &'a NavigationStack,
    pub audio: &'a AudioManager,
    pub frame_delta: f32,
    pub script_path: Option<&'a str>,
}
```

`draw` 内部使用 `egui::CollapsingHeader` 组织各分区，按需展开/折叠。

### 3.5 信息面板设计

各面板为可折叠区块，默认可全部折叠以减小占用。建议布局：单列垂直排列，或左右两列（根据屏幕宽度自适应）。

#### 3.5.1 脚本状态（Script State）

| 字段         | 来源                         | 说明                         |
|--------------|------------------------------|------------------------------|
| 脚本路径     | `position.script_id` / 加载映射 | 当前执行的脚本逻辑路径       |
| 节点索引     | `position.node_index`        | 当前节点在脚本中的索引       |
| 源码行号     | `Script::get_source_line(node_index)` | 若 host 可访问当前 Script 且含 source_map 则显示；否则可省略 |
| 等待原因     | `WaitingReason`              | `None` / `WaitForClick` / `WaitForChoice` / `WaitForTime` / `WaitForSignal` |
| 调用栈深度   | `call_stack.len()`           | 跨文件调用栈长度             |

无 Runtime 时显示 "无脚本" 或 "—"。

#### 3.5.2 对话（Dialogue）

| 字段           | 来源                         | 说明                     |
|----------------|------------------------------|--------------------------|
| 当前说话人     | `dialogue.speaker`           | 若有                      |
| 显示文本       | `dialogue.content` 截断     | 可限制长度，避免过长      |
| 打字机进度     | `visible_chars` / `total`   | 如 "12 / 45"             |
| 内联效果       | `inline_effects` / `inline_wait` / `effective_cps` | 简要列出活跃项 |

无对话时显示 "—"。

#### 3.5.3 视觉状态（Visual State）

| 字段             | 来源                              | 说明                     |
|------------------|-----------------------------------|--------------------------|
| 当前背景         | `render_state.current_background` | 逻辑路径或 "—"          |
| 可见角色数       | `visible_characters.len()`        | 数量                     |
| 角色列表         | `visible_characters` 迭代         | alias + position 简要   |
| 场景过渡         | `Renderer::scene_transition`      | 类型、阶段、进度         |
| 场景效果         | `scene_effect`                    | shake/blur/dim 等        |

#### 3.5.4 音频（Audio）

| 字段       | 来源                         | 说明                     |
|------------|------------------------------|--------------------------|
| BGM        | `current_bgm_path()`         | 路径，无则 "—"           |
| BGM 音量   | `bgm_volume` / `duck_multiplier` | 当前有效音量、是否 duck |
| 播放中的 SE| 若 `AudioManager` 暴露 API    | 否则可显示 "—" 或占位    |
| 播放中的语音 | 若存在 voice 子系统        | 否则可显示 "—" 或占位    |

注：当前 `AudioManager` 主要暴露 BGM 状态；SE/voice 列表若暂无 API，可留占位或显示 "N/A"。

#### 3.5.5 性能（Performance）

| 字段     | 来源                    | 说明           |
|----------|-------------------------|----------------|
| FPS      | `1.0 / frame_delta`     | 帧率           |
| 帧时间   | `frame_delta * 1000`    | 毫秒           |
| Delta    | `frame_delta`           | 秒             |

数据来自 `WgpuBackend::frame_delta()`，在 `render_frame` 内更新。

#### 3.5.6 模式（Mode）

| 字段         | 来源                    | 说明                     |
|--------------|-------------------------|--------------------------|
| AppMode      | `navigation.current()`  | Title / InGame / ...     |
| NavigationStack | `navigation`         | 栈深度或简要路径         |
| PlaybackMode | `session.playback_mode` | Normal / Auto / Skip     |

### 3.6 集成点

在 `host_app.rs` 的 `build_ui` 闭包中，在 `egui_screens::toast::build_toast_overlay` 之前插入：

```rust
// Debug overlay (F9 toggle, only when config.debug.enable_overlay)
if app_state.config.debug.enable_overlay && debug_overlay.visible {
    debug_overlay::draw(ctx, &debug_overlay_state);
}
```

`debug_overlay_state` 在闭包外、`render_frame` 调用前构建，从 `app_state` 和 `backend` 中提取所需引用。

### 3.7 布局与样式

- 使用 `egui::Window::new("Debug Overlay")`，`id` 固定以便 egui 记住位置。
- 默认锚定在屏幕右上角或右侧，宽度约 280–320 逻辑像素。
- 背景色：`egui::Color32::from_black_alpha(200)` 或类似，保证可读性的同时半透明。
- 字体使用 egui 默认，不引入额外字体资源。
- 若内容过多可启用滚动区域（`egui::ScrollArea`）。

---

## 4. 影响范围

| 模块 / 文件                    | 变更类型     | 说明                                   |
|--------------------------------|--------------|----------------------------------------|
| `host/src/config/mod.rs`       | 扩展         | `DebugConfig` 新增 `enable_overlay`     |
| `host/src/input/mod.rs`        | 无           | 已有 `is_key_just_pressed`，无需改动   |
| `host/src/host_app.rs`         | 扩展         | F9 检测、`DebugOverlay` 状态、`build_ui` 内调用 |
| `host/src/debug_overlay.rs`    | 新增         | `DebugOverlay` 与 `draw` 实现          |
| `host/src/lib.rs`              | 可选         | 若 `debug_overlay` 为独立模块则 `pub use` |
| `host/src/backend/mod.rs`      | 无           | 不修改 `render_frame` 签名              |
| `host/src/renderer/render_state` | 无         | 只读访问现有字段                       |
| `host/src/audio/mod.rs`       | 可选         | 若需 SE/voice 列表，可后续扩展 API      |
| `vn-runtime`                  | 无           | 仅读取 `RuntimeState`、`Script` 等     |

---

## 5. 迁移计划

1. **阶段 1**：配置与热键
   - 在 `DebugConfig` 中新增 `enable_overlay`，默认 `false`。
   - 在 `HostApp` 中增加 `DebugOverlay { visible: false }` 状态。
   - 在 `RedrawRequested` 中检测 F9，当 `enable_overlay` 为 true 时切换 `visible`。

2. **阶段 2**：覆盖层骨架
   - 新增 `host/src/debug_overlay.rs`，实现 `DebugOverlay` 与 `DebugOverlayState`。
   - 实现空 `draw`（仅显示标题或占位），在 `build_ui` 中集成。

3. **阶段 3**：各信息面板
   - 按 3.5 节顺序实现：脚本状态 → 对话 → 视觉状态 → 音频 → 性能 → 模式。
   - 源码行号若 host 暂无法访问 `Script`，可先显示 "—"，待 Runtime 暴露相应 API 后补充。
   - 每完成一块可单独验证显示正确性。

4. **阶段 4**：打磨
   - 调整布局、透明度、折叠默认状态。
   - 补充文档注释，更新 `config_guide.md` 中 `debug.enable_overlay` 说明。
   - 若需要，在 `docs/engine/architecture/navigation-map.md` 中补充 `debug_overlay` 入口。

---

## 6. 验收标准

- [ ] `config.json` 中 `debug.enable_overlay: true` 时，F9 可切换覆盖层显示。
- [ ] `enable_overlay: false` 时，F9 无效果，覆盖层不渲染。
- [ ] 覆盖层包含 6 个可折叠区块：脚本状态、对话、视觉状态、音频、性能、模式。
- [ ] 各区块数据与当前 `AppState` / `RenderState` / `RuntimeState` 一致。
- [ ] 覆盖层半透明，不完全遮挡游戏内容，可拖拽移动。
- [ ] `cargo check-all` 通过，无新增 clippy 警告。
- [ ] 覆盖层代码有基本文档注释，符合项目规范。
- [ ] release 构建下，`enable_overlay` 默认关闭或配置不生效（由实现决定并文档化）。
