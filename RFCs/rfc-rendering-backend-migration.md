# RFC: 渲染后端迁移 — macroquad → winit + wgpu + egui

## 元信息

- 编号：RFC-007
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-10
- 相关范围：`host`（渲染 / 窗口 / 输入 / UI / 资源加载）
- 前置：PoC 验证通过（`tools/rendering-poc/`）

---

## 1. 背景

当前 `host` 使用 macroquad 0.4 作为唯一渲染后端。macroquad 通过 `#[macroquad::main]` 宏接管窗口创建与 async 主循环，外部代码无法获取底层窗口句柄或 GPU 上下文。

在集成 cutscene 视频播放功能（RFC-002 P1-4）时，这一限制构成了硬性阻碍：

- **窗口封闭**：无法获取原生窗口句柄或 GPU 上下文供视频解码器使用
- **纹理管线封闭**：无法从外部（视频解码帧）注入纹理到渲染管线
- **shader 受限**：仅支持 GLSL，不支持现代 WGSL；自定义渲染管线受限
- **维护性风险**：macroquad 更新节奏慢，社区规模有限，继续 hack 绕过限制会显著增加技术债

同时，macroquad 提供的部分能力已被项目独立实现替代：
- 音频播放已使用 `rodio`
- 图像解码已使用 `image` crate

---

## 2. 目标与非目标

### 2.1 目标

- **G1** — 替换 macroquad，使用 winit + wgpu + egui 作为 host 的渲染/窗口/UI 基础设施
- **G2** — 完全掌控窗口生命周期与 GPU 渲染管线，为视频帧注入扫清障碍
- **G3** — 保持 Runtime/Host 架构分离不变（`vn-runtime` 零改动）
- **G4** — 迁移完成后，现有所有视觉功能（背景/立绘/对话/选项/转场/效果/菜单/存读档）行为等价
- **G5** — ImageDissolve shader 从 GLSL 迁移到 WGSL

### 2.2 非目标

- **不**改动 `vn-runtime`（脚本解析/执行/Command/状态模型不受影响）
- **不**在本 RFC 内实现 cutscene 视频播放（视频功能由后续迭代完成，本 RFC 仅扫清架构阻碍）
- **不**改动 `rodio` 音频系统（已独立于 macroquad）
- **不**改动资源打包工具 `asset-packer`
- **不**追求 WASM 支持（留后续迭代）
- **不**趁机重构无关的 Runtime 逻辑

---

## 3. 方案选型

### 3.1 候选方案

| 方案 | 组合 | 优势 | 劣势 |
|------|------|------|------|
| A | winit + wgpu + egui | 完全窗口掌控；GPU 渲染自由；egui 提供即时模式 UI；三库均为 Rust 顶级项目 | 需自建 2D sprite 管线；迁移工作量中大 |
| B | eframe (egui 官方框架) | 开箱即用；UI 天然适配 | 非为全屏图像密集场景设计；纹理内存管理有已知问题；shader 效果需绕路 |
| C | SDL2 (sdl2 crate) | VN 引擎已有先例；视频集成成熟 | C 依赖增加构建复杂度；shader 需直接 OpenGL；API 风格不 Rustic |
| D | miniquad 直接使用 | 与 macroquad 同生态，迁移成本低 | 不能根本解决窗口封闭问题 |

### 3.2 选定方案：A（winit + wgpu + egui）

**理由**：

1. **窗口完全掌控** — winit 提供原生窗口句柄，事件循环由应用代码拥有
2. **GPU 管线完全自由** — wgpu 可自定义渲染管线、shader、纹理管理，视频帧仅是"又一张纹理"
3. **UI 天然覆盖** — egui 即时模式 UI 适合对话框/菜单/设置等 VN 界面，减少自建 UI 成本
4. **PoC 已验证** — `tools/rendering-poc/` 验证了四项核心能力（见第 4 节）
5. **生态成熟** — winit（Rust 标准窗口库）、wgpu（WebGPU 标准实现）、egui（即时模式 UI 标杆）均为高活跃度项目

### 3.3 依赖版本

| 库 | 版本 | 用途 |
|----|------|------|
| winit | 0.30 | 窗口管理、事件循环、输入事件 |
| wgpu | 24 | GPU 渲染、纹理管理、shader (WGSL) |
| egui | 0.31 | 即时模式 UI |
| egui-wgpu | 0.31 | egui 到 wgpu 渲染桥接 |
| egui-winit | 0.31 | egui 到 winit 输入桥接 |
| image | 0.24 | 图像解码（不变） |
| rodio | 0.22 | 音频播放（不变） |

---

## 4. PoC 验证结果

PoC 代码位于 `tools/rendering-poc/`，约 290 行，验证结果：

| 验证项 | 方法 | 结果 |
|--------|------|------|
| winit 窗口管理 | `ApplicationHandler` trait + `EventLoop::run_app` | 完全掌控窗口生命周期与事件循环 |
| wgpu 纹理渲染 | 加载 1920x1440 背景图，`create_texture_with_data` → 全屏 quad 渲染 | 正常显示 |
| egui UI 叠加 | `egui::TopBottomPanel` 对话框 + `egui::Window` 控制面板 | 叠加在 wgpu 渲染之上，交互正常 |
| 动态纹理更新 | `queue.write_texture()` 每帧写入新像素数据 | 流畅更新，验证视频帧注入可行性 |

关键技术发现：
- wgpu 24 的 `RenderPass` 默认绑定 encoder 生命周期，需调用 `forget_lifetime()` 获取 `'static` 生命周期以适配 `egui_wgpu::Renderer::render`
- 单个 render pass 内先画 wgpu 自定义内容，再画 egui 叠加层，层级关系清晰

---

## 5. 迁移影响范围

### 5.1 受影响区域

当前 **28 个文件** 引用了 macroquad API，按职责分组：

| 区域 | 文件数 | macroquad 用途 | 迁移目标 |
|------|--------|----------------|----------|
| 主循环/窗口 | 1 (`main.rs`) | `#[macroquad::main]`, `next_frame`, `Conf` | winit `EventLoop` + `ApplicationHandler` |
| 渲染器 | 3 (`renderer/`) | `draw_texture_ex`, `Camera2D`, `clear_background` | wgpu 自定义 2D 渲染管线 |
| 文本渲染 | 1 (`text_renderer.rs`) | `load_ttf_font`, `draw_text_ex`, `measure_text` | egui 文本 或 glyphon/cosmic-text |
| Shader | 1 (`image_dissolve.rs`) | `load_material`, `gl_use_material`, GLSL | wgpu render pipeline + WGSL |
| 资源管理 | 2 (`resources/`) | `Texture2D`, `load_texture_from_bytes` | wgpu `Texture` + 自建缓存 |
| 输入 | 1 (`input/`) | `is_key_pressed`, `mouse_position` | winit `WindowEvent` 键盘/鼠标事件 |
| UI 组件 | 8 (`ui/`) | `draw_rectangle`, `draw_circle`, `mouse_position` | egui 组件 |
| 屏幕页面 | 5 (`screens/`) | 绘制原语 + 输入查询 | egui 页面 |
| 应用层 | 4 (`app/`) | `screen_width`, `get_frame_time`, `get_fps` | winit/wgpu 等价 API |
| 命令处理 | 1 (`extensions/`) | 效果参数中的类型引用 | 适配新类型 |
| 音频加载 | 1 (`resources/`) | `macroquad::audio::load_sound` | 已用 rodio，仅需移除 macroquad 路径 |

### 5.2 不受影响区域

| 区域 | 原因 |
|------|------|
| `vn-runtime/` 全部 | Runtime/Host 分离，Runtime 不依赖任何引擎 API |
| `rodio` 音频播放 | 已独立于 macroquad |
| `image` 图像解码 | 已独立于 macroquad |
| `tools/asset-packer/` | 不涉及渲染 |
| `tools/xtask/` | 不涉及渲染 |
| 脚本/资源文件 | 纯数据，不依赖引擎 |

---

## 6. 迁移架构设计

### 6.1 新渲染层架构

```
┌─────────────────────────────────────────────┐
│  main.rs (winit EventLoop + ApplicationHandler)  │
├─────────────────────────────────────────────┤
│  app/update   — 输入采集 + Runtime tick + 命令执行  │
├──────────────────────┬──────────────────────┤
│  wgpu 2D Renderer    │  egui UI Layer       │
│  ┌────────────────┐  │  ┌────────────────┐  │
│  │ Background     │  │  │ DialogueBox    │  │
│  │ Characters     │  │  │ ChoiceMenu     │  │
│  │ SceneTransition│  │  │ Screens(Title/ │  │
│  │ ImageDissolve  │  │  │  Settings/Save/│  │
│  │ (WGSL shader)  │  │  │  History/Menu) │  │
│  │ Video frames   │  │  │ Toast/Modal    │  │
│  └────────────────┘  │  └────────────────┘  │
├──────────────────────┴──────────────────────┤
│  wgpu Device/Queue/Surface                   │
│  winit Window                                │
└─────────────────────────────────────────────┘
```

**渲染顺序**（单个 render pass 内）：
1. Clear → 背景 quad
2. 角色 sprites（按 z-order）
3. 场景转场遮罩 / ImageDissolve
4. egui UI 叠加（对话框、菜单、HUD）

### 6.2 2D 渲染管线

核心是一个 **textured quad batch renderer**：

- 输入：一组 `(texture, position, size, alpha, tint)` 绘制指令
- 管线：单个 wgpu `RenderPipeline`，顶点含 position + UV + color/alpha
- 纹理切换：按纹理分组提交 draw call（或使用 texture array）
- 效果：alpha blend、tint、缩放、裁剪通过 uniform/vertex attribute 实现

### 6.3 纹理管理

沿用现有 `TextureCache` LRU 策略，底层类型从 `macroquad::Texture2D` 替换为 `wgpu::Texture` + `wgpu::TextureView` + `wgpu::BindGroup`。

加载路径不变：`image::open()` → RGBA bytes → `device.create_texture_with_data()` → 缓存。

### 6.4 文本渲染

两种方案：

| 方案 | 优势 | 劣势 |
|------|------|------|
| **egui 文本（推荐）** | 零额外依赖；与 UI 层统一；支持富文本 | 字体渲染质量受 egui 限制；打字机效果需适配 |
| cosmic-text / glyphon | 高质量文本渲染；更精细控制 | 额外依赖；需自建与 wgpu 的集成 |

建议首选 egui 文本，如打字机效果适配困难再回退到 cosmic-text。

### 6.5 输入系统

winit 通过 `WindowEvent` 提供键盘、鼠标事件。迁移映射：

| macroquad | winit |
|-----------|-------|
| `is_key_pressed(KeyCode::Space)` | `WindowEvent::KeyboardInput` + `ElementState::Pressed` |
| `is_key_down(KeyCode::LControl)` | 维护 pressed keys `HashSet`，查询是否包含 |
| `mouse_position()` | `WindowEvent::CursorMoved` → 缓存位置 |
| `is_mouse_button_pressed(Left)` | `WindowEvent::MouseInput` + `ElementState::Pressed` |
| `screen_width()` / `screen_height()` | `window.inner_size()` |
| `get_frame_time()` | `Instant::now()` 逐帧计算 delta |
| `get_time()` | `Instant::elapsed()` |

### 6.6 UI 系统

现有 `host/src/ui/` 手工绘制 UI 组件（button、slider、toggle、panel、modal、list、toast），全部替换为 egui 等价组件。

现有 `host/src/screens/` 各页面（title、settings、save_load、history、ingame_menu）重写为 egui 窗口/面板。

egui 的即时模式特性天然适配 VN 的 UI 模式——每帧根据状态重建 UI，无需维护组件树。

---

## 7. 分阶段实施计划

### Phase 0：基础设施 -- DONE

- [x] 在 `host/src/` 新建 `backend/` 模块，封装 wgpu 初始化、窗口、渲染管线
- [x] 实现 `WgpuBackend` 结构体：device, queue, surface, egui 集成
- [x] egui 字体初始化：加载 CJK 字体（simhei.ttf）到 `FontDefinitions`，确保中文渲染正常
- [x] 迁移 `main.rs`：从 `#[macroquad::main]` 改为 winit `EventLoop::run_app`
- [x] 主循环 `update → draw → present` 骨架就位
- [x] **门控**：空白窗口 + egui 中文面板可运行（GPU: Vulkan, 183 tests pass）

### Phase 1：2D 渲染管线 -- DONE

- [x] 实现 textured quad batch renderer（WGSL shader + vertex buffer）→ `backend/sprite_renderer.rs`
- [x] 迁移纹理加载：`ResourceManager` 底层从 `Texture2D` 改为 `Arc<GpuTexture>`（同步 API）
- [x] 迁移 `TextureCache`：LRU 策略不变，缓存类型 `Texture2D` → `Arc<GpuTexture>`
- [x] 迁移 `Renderer::render` → `build_draw_commands` 返回 `Vec<DrawCommand>`
- [x] `UiContext::new` 消除 macroquad 依赖
- [x] `main.rs` 集成 AppState + 脚本加载 + 场景过渡驱动
- [x] **门控**：背景纹理通过 sprite pipeline 正确渲染，183 tests pass，clippy clean

### Phase 2：Shader 效果迁移（预计 1 天） -- DONE

- [x] ImageDissolve shader 从 GLSL 迁移到 WGSL
  - 新增 `host/src/backend/dissolve_renderer.rs`：独立的遮罩溶解渲染器
  - WGSL fragment shader：通过灰度 mask 纹理的 r 通道控制逐像素 alpha，支持 progress / ramp / reversed
  - `DissolveRenderer` 共享 `SpriteRenderer` 的 `texture_bind_group_layout`，GpuTexture bind_group 两管线通用
  - 独立 uniform buffer（projection + dissolve params）和 vertex buffer（单个全屏 quad）
- [x] 转场 alpha blend 在新管线中实现
  - Fade / FadeWhite：使用 `DrawCommand::Rect` alpha 叠加（Phase 1 已完成）
  - Rule：使用 `DrawCommand::Dissolve` 遮罩溶解叠加
- [x] 场景转场（`SceneTransitionManager`）适配新渲染后端
  - `Renderer::build_scene_mask_commands` 为 Rule 类型生成 `DrawCommand::Dissolve`
  - FadeIn 阶段：progress 0->1（遮罩逐步覆盖黑色）
  - Blackout 阶段：progress=1（全黑）
  - FadeOut 阶段：progress 1->0（遮罩逐步揭示新背景）
  - 降级：mask 未加载时回退为纯色 Rect
  - `WgpuBackend::render_frame` 在 sprite 绘制后、egui 前处理 Dissolve 命令
- [x] **门控**：编译 + clippy 无警告 + 183 单元测试通过

### Phase 3：输入系统迁移（预计 0.5 天） -- DONE

- [x] `InputManager` 改为消费 winit `WindowEvent`
  - 完全移除 macroquad 依赖，改用 `winit::event::{WindowEvent, ElementState, KeyCode}`
  - 新增 `process_event(&WindowEvent)` 接收键盘/鼠标事件
  - 新增 `begin_frame(dt)` 清除 per-frame 状态并推进内部时钟
  - 内部维护 `HashSet<KeyCode>` pressed_keys / just_pressed_keys 和鼠标状态
  - 暴露 `mouse_position()` / `is_mouse_pressed()` / `is_mouse_just_pressed()` 给外部查询
- [x] 维护 pressed keys / mouse state
- [x] `main.rs` 中 winit 事件转发到 InputManager（egui 未消费时）
  - RedrawRequested 中调用 `begin_frame(dt)` → `update(waiting, dt)` → `run_script_tick`
- [x] `UiContext` 输入源保持 Phase 1 的 no-op 状态（Phase 4 统一迁移到 egui）
  - macroquad UI 组件（buttons, sliders 等）仍保留 macroquad imports（dead code）
- [x] **门控**：编译 + clippy 无警告 + 183 单元测试通过

### Phase 4：UI 与屏幕迁移（预计 3-4 天） -- DONE

- [x] 接入完整 `app::update()` 循环到 winit main loop
  - 打字机效果（typewriter timer + advance）自动推进
  - PlaybackMode 处理（Normal/Auto/Skip + Ctrl held）
  - 信号完成检测（scene_transition / scene_effect / title_card）
  - WaitForTime 计时推进
  - 淡出角色自动移除
  - `modes.rs` macroquad 按键调用全部替换为 `InputManager`
  - `update/mod.rs` 不再依赖 `get_frame_time()`，由 main.rs 传入 dt
  - `InputManager::begin_frame(dt)` / `end_frame()` 分离避免 per-frame 状态提前清除
- [x] egui 模式分发 UI 系统 (`build_mode_ui`)
  - `EguiAction` 枚举统一处理所有 UI 动作（StartGame/Continue/Navigate/GoBack/Exit 等）
  - `handle_egui_action` 在 render_frame 后处理，避免闭包借用冲突
  - Title 页面：New Game / Continue / Load / Settings / Exit 按钮
  - InGame 页面：对话框（打字机）+ 选项（高亮选中）
  - InGameMenu 页面：Resume / Save / Load / Settings / History / Return to Title / Exit
  - Settings 页面：当前值展示 + Back 按钮（编辑功能 TODO）
  - SaveLoad / History：stub 页面
- [x] `host/src/ui/` 旧 macroquad 组件清理 -- DONE (Phase 5)
- [x] Settings 页面实现编辑功能（滑块、开关）
  - Text Speed / Auto Delay / BGM Volume / SFX Volume 滑块编辑
  - Muted 开关
  - Apply & Back / Cancel 按钮
  - 应用设置时同步音频管理器 + 持久化到 user_settings.json
- [x] SaveLoad 页面实现完整存读档 UI
  - Save / Load 标签页切换
  - 20 槽位滚动列表，显示章节、时间、游玩时长
  - Save / Load / Delete 按钮；空槽位 / 无游戏状态自动禁用
  - 操作后 Toast 反馈
- [x] History 页面实现对话历史浏览
  - 从 VNRuntime History 读取 Dialogue + ChapterMark 事件
  - 滚动列表，自动滚到底部
  - 说话者金色高亮，章节标记分隔线
- [x] Toast 覆盖层适配 egui
  - 右上角浮动通知，按类型着色（Success/Error/Warning/Info）
  - 淡出动画（alpha 渐变）
  - 在所有 AppMode 上层渲染
- [x] ESC 键统一返回（InGameMenu / SaveLoad / Settings / History）
- [x] **门控**：`cargo check-all` 通过（fmt + clippy + 381 tests）

### Phase 5：清理与验收 -- DONE

- [x] 移除 `macroquad` 依赖（`host/Cargo.toml`）
- [x] 移除所有 `use macroquad::` 引用
  - 已移除的模块/文件：`screens/` 模块（5 个 screen 实现）、`ui/` 下 8 个旧组件模块（button/list/modal/panel/scroll/slider/tab/toggle）、`draw_rounded_rect` 系列函数
  - 自定义 `Color` 类型替代 `macroquad::prelude::Color`（`ui/theme.rs`）
  - `TextRenderer` 简化为选项布局计算器（`get_choice_rects` 接收显式屏幕尺寸）
  - `ImageDissolve` 简化为 ramp 参数管理（渲染由 `DissolveRenderer` 处理）
  - `builtin_effects.rs` 的 `screen_width/height` 改为从 `Renderer` 获取
  - `UiSystems` 移除旧 screen 字段，`modes.rs` 非 InGame 模式改为 no-op
- [x] `cargo check-all` 通过（fmt + clippy + 181 host tests + 200 runtime tests）
- [ ] 端到端手动验收：prologue -> summer -> winter 主线跑通
- [ ] Skip/Auto/存档/读档/设置 闭环验收
- [ ] 移除或归档 `tools/rendering-poc/`
- [ ] 更新文档（`ARCH.md`、导航地图、模块摘要）

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 打字机效果在 egui 文本渲染中难以精确控制（逐字显示、速度标签） | 对话体验降级 | 可回退到 cosmic-text 自建文本渲染；或用 egui `Label` 逐帧截取可见文本 |
| wgpu 初始化在部分老旧 GPU/驱动上失败 | 无法运行 | wgpu 支持 Vulkan/DX12/Metal/GL 多后端自动回退；可指定后端 |
| egui 高分辨率纹理内存占用（已知 issue：单张 2880x1800 约 41MB） | 内存压力 | 沿用现有 TextureCache LRU 驱逐策略；控制同时加载纹理数 |
| 迁移过程中功能回归 | 功能断裂 | 分阶段推进，每阶段有门控验收；保留 macroquad 代码直到确认可删 |
| 依赖版本兼容性（egui/wgpu/winit 三库版本矩阵） | 编译失败 | PoC 已锁定兼容版本组合；使用 workspace 统一管理 |

---

## 9. 视频集成展望（后续迭代）

本 RFC 完成后，cutscene 视频播放（RFC-002 P1-4）的集成路径将被打通：

```
视频解码器 (ffmpeg-next / re_video)
    ↓ 输出 RGBA 帧
queue.write_texture() → wgpu Texture
    ↓
全屏 quad 渲染（复用 2D 管线）
    ↓
音频轨道 → rodio 播放
```

具体视频解码方案选型将在独立的迭代中决策，不在本 RFC 范围内。

---

## 10. ARCH.md 兼容性

| 架构约束 | 兼容性 |
|----------|--------|
| Runtime/Host 分离 | 完全兼容。vn-runtime 零改动，迁移仅影响 Host |
| Runtime 禁止引擎 API | 不受影响。Runtime 从未依赖 macroquad |
| Command 驱动 | 不受影响。Command 类型、执行语义不变 |
| 显式状态 / 确定性 | 不受影响。RenderState 模型不变，仅底层绘制实现替换 |

Host 层 ARCH.md 中"macroquad 宿主"的表述需在迁移完成后更新为"winit + wgpu + egui 宿主"。

---

## 11. 验收标准（Definition of Done）

- [ ] `host/Cargo.toml` 不再包含 `macroquad` 依赖
- [ ] `cargo check-all` 通过（fmt / clippy / test）
- [ ] 端到端主线可跑通（prologue → summer → winter）
- [ ] 所有转场效果（fade / ImageDissolve / changeScene）视觉等价
- [ ] 对话打字机效果（含节奏标签 {wait} / {speed} / --> / extend）行为等价
- [ ] Skip / Auto / Normal 三种推进模式正常
- [ ] 所有菜单页面（Title / Settings / SaveLoad / History / InGameMenu）可交互
- [ ] 存档 / 读档 / Continue 功能正常
- [ ] 音频播放不受影响
- [ ] `ARCH.md`、`docs/navigation_map.md`、相关模块摘要已更新

---

## 12. 时间预估

| 阶段 | 预估 | 累计 |
|------|------|------|
| Phase 0: 基础设施 | 1-2 天 | 1-2 天 |
| Phase 1: 2D 渲染管线 | 2-3 天 | 3-5 天 |
| Phase 2: Shader 迁移 | 1 天 | 4-6 天 |
| Phase 3: 输入迁移 | 0.5 天 | 4.5-6.5 天 |
| Phase 4: UI/屏幕迁移 | 3-4 天 | 7.5-10.5 天 |
| Phase 5: 清理与验收 | 1 天 | 8.5-11.5 天 |

总计约 **9-12 工作日**，取决于打字机效果适配和 UI 页面复杂度。
