# 经验沉淀（Lessons Learned）

> 本文档记录开发过程中反复出现的问题模式和解决方案。
> Agent 和人类开发者在遇到相关场景时应优先查阅此文档，避免重复踩坑。
>
> **维护规则**：发现新陷阱后，在对应分类下追加条目。每条须包含：现象、原因、正确做法。

---

## 跨模块管线

### 新增 Command 变体后 host 侧 panic

- **现象**：新增 `Command::Foo` 后，脚本执行到该指令时 host 侧 panic（`unreachable!` 或 match 不完备）。
- **原因**：`CommandExecutor::execute()` 中遗漏了对新变体的 match arm。
- **正确做法**：新增 Command 变体后，立即在 `host/src/command_executor/` 中补处理分支。使用 `rg "Command::" host/src/command_executor/` 验证覆盖度。参考 [cross-module-command-pipeline SKILL](../../.cursor/skills/cross-module-command-pipeline/SKILL.md)。

### Signal 常量字符串不匹配

- **现象**：Command 需要 host 回信号解除等待，但 runtime 一直卡在等待状态。
- **原因**：`SIGNAL_*` 常量在 `runtime/executor/mod.rs`（等待端）和 `command_executor`（发射端）中拼写不一致。
- **正确做法**：Signal ID 统一定义为常量，两端引用同一常量。改动后搜索 `SIGNAL_` 确认匹配。

---

## 脚本解析

### Phase 1 不识别导致指令静默丢失

- **现象**：新增指令在测试中不产出任何 Command，也不报错。
- **原因**：Phase 1 未识别该行，将其视为空行或旁白静默跳过。Phase 2 永远看不到它。
- **正确做法**：先确认 phase1 对该行产生了正确的 `Block` 变体。用 `parse_phase1()` 单独测试。

### 中文标点容错遗漏

- **现象**：使用中文冒号 `：` 的对话行解析失败，但英文冒号 `:` 正常。
- **原因**：解析逻辑只匹配了一种标点。
- **正确做法**：所有面向内容作者的标点都须同时支持中英文变体（`：`/`:`、`"`/`"`/`"` 等）。新增解析逻辑后须补容错测试。

### Source map 漂移

- **现象**：诊断报错的行号偏移，指向错误位置。
- **原因**：多行结构（choice 块、if/else 块）改变了行分组方式，但 source map 未同步更新。
- **正确做法**：修改 phase1/phase2 行分组后，运行 diagnostic 测试验证行号准确性。

---

## 资源与路径

### 手工拼接路径导致 Zip 模式失败

- **现象**：开发模式（文件系统）正常，但打包后（zip 模式）找不到资源。
- **原因**：使用 `config.assets_root.join()` 或 `PathBuf` 手工拼接路径，绕过了 `ResourceManager` 的统一路径解析。
- **正确做法**：所有资源访问通过 `ResourceManager` 方法 + `LogicalPath`，禁止裸字符串路径调用。

### 资源路径相对于脚本文件

- **现象**：脚本中引用的图片/音频路径在测试中找不到。
- **原因**：资源路径是相对于脚本文件位置的，测试中未设置 `base_path`。
- **正确做法**：parser 测试使用 `parse_with_base_path`；`extract_resource_references` 须处理新节点类型。

---

## 渲染与效果

### changeBG 后立即 changeScene 背景闪烁

- **现象**：先 `changeBG` 再 `changeScene`，过渡期间背景短暂消失。
- **原因**：`changeBG` 设置了 `current_background`，但 `changeScene` 在过渡开始时会清除/覆盖它。
- **正确做法**：`changeScene` 的 `new_bg` 参数已包含目标背景，不需要先 `changeBG`。

### Headless 测试中假设 GPU 存在

- **现象**：新增的渲染测试在 CI 或无 GPU 环境中 panic。
- **原因**：直接使用了 `wgpu` 类型而非 `Texture` trait 抽象。
- **正确做法**：渲染逻辑测试使用 `NullTexture` / `NullTextureFactory`（见 `host/src/test_harness.rs`）。不在 `backend/` 之外 downcast 到 `GpuTexture`。

---

## 前端与 Vue

### 模板里使用的 `.vue` 组件误写成 `import type`

- **现象**：Vite / Vue 在运行时反复报警 `Failed to resolve component: Xxx`，但同名组件文件实际存在，TypeScript 类型检查也可能仍然通过。
- **原因**：在 `<script setup>` 中把模板要渲染的组件写成了 `import type Foo from "./Foo.vue"`。类型导入会在运行时被擦除，Vue 编译器无法把它注册为模板组件。
- **正确做法**：凡是会在模板里直接使用的 `.vue` 组件，一律使用普通导入：`import Foo from "./Foo.vue"`。只有纯类型位置才使用 `import type`，并避免对 `.vue` SFC 本体做类型导入。

---

## 存档兼容

### 新增 RuntimeState 字段破坏旧存档加载

- **现象**：添加字段后，旧版 JSON 存档反序列化失败。
- **原因**：新字段未设置 `#[serde(default)]`，旧 JSON 中缺少该字段。
- **正确做法**：`RuntimeState` / `SaveData` 新增字段必须加 `#[serde(default)]`。修改存档结构须在 `docs/engine/reference/save-format.md` 中记录迁移方案。

### slot save 不等于 full-frame restore

- **现象**：想把 `save/load/continue` 修成“恢复到当前屏幕的每一个细节”，于是直觉上准备扩 `vn-runtime::SaveData` 去存 `dialogue`、`choices`、`active_ui_mode` 等 host 字段。
- **原因**：混淆了两条能力边界。旧 `host` 的 slot/continue 本来就是粗粒度恢复；真正的完整帧恢复来自 host 侧 snapshot/backspace 机制，而不是 runtime 公共存档格式。
- **正确做法**：先确认问题到底属于“runtime 存档格式”还是“host 侧快照/恢复边界”。若目标只是宿主迁移等价，优先在 host 层收敛保存边界（例如退回最近 snapshot），不要贸然把 host 侧瞬时 UI 状态塞进 `vn-runtime` 公共模型。

---

## 测试

### 集成测试 boilerplate 过多导致 agent 出错

- **现象**：agent 每次写集成测试都要手动构建 Parser + Runtime + tick 循环，容易遗漏步骤。
- **原因**：缺少共享测试基础设施。
- **正确做法**：使用 `vn-runtime/tests/common/mod.rs` 中的 `ScriptTestHarness`。典型用法：

```rust
let mut h = ScriptTestHarness::new(r#"羽艾："你好""#);
let result = h.tick();
assert!(result.has_text_from("羽艾", "你好"));
result.assert_waiting_click();
```

### 并发测试共享固定临时路径会互相踩坏测试数据

- **现象**：本地单跑通过，但 `cargo test` 并发执行时，ZIP/文件系统相关测试随机失败，报错表现为“压缩包损坏”“文件不存在”或读到别的测试留下的数据。
- **原因**：多个测试共享固定的临时目录或文件名（如同一个 `temp_dir()/ring_test_zip/test_assets.zip`），并发执行时会相互覆盖、截断或提前删除文件。
- **正确做法**：每个测试都生成唯一的临时目录/文件名（可用时间戳、随机后缀或专用 temp helper），并在测试结束后单独清理自己的工作目录。

---

## 工具链与环境

### 会修改工作区的门禁不适合放 pre-commit

- **现象**：把 `cargo fmt` / `cargo clippy --fix` 这类会直接改文件的命令塞进 pre-commit 后，总会碰到 staged/unstaged 语义错位、partial commit 冲突，或者“检查通过但提交内容不一致”的问题。
- **原因**：Git 提交基于暂存区（index），而 Rust 工具链命令默认作用于工作区和整个 workspace，无法天然只约束“本次将提交的那一份快照”。
- **正确做法**：移除这类 pre-commit 自动门禁，把 `cargo check-all` 放到 CI 执行，并在后面追加 `git diff --exit-code`。本地开发者按需手动运行 `cargo check-all` 即可。

### PowerShell 中 `&&` 语法错误

- **现象**：在 PowerShell 中用 `&&` 连接命令报语法错误。
- **原因**：PowerShell（非 pwsh 7+）不支持 `&&` 作为命令连接符。
- **正确做法**：使用 `;` 分隔命令：`cd F:\Code\Ring-rs; cargo test`。

### wry WebView 不能使用 `file://` URL 加载本地页面

- **现象**：WebView2 的 IPC handler 收到消息时 panic：`http::Error(InvalidUri(InvalidFormat))`。panic 发生在 wry 内部（`webview2/mod.rs`），在用户 handler 被调用之前。
- **原因**：wry 用页面来源 URL 构造 `http::Request`，Windows `file://` 路径含盘符冒号（如 `F:`），`http::Uri` 无法解析。且 panic 发生在 COM 回调（`extern "system"`）中，无法 unwind，直接 abort。
- **正确做法**：使用 `with_custom_protocol()` 注册自定义协议（如 `game://`），通过闭包从文件系统读取资源。Windows 上 WebView2 会将其映射为 `http://game.localhost/...`，是合法的 HTTP URI。

### WebView 子窗口遮挡父窗口导致 RedrawRequested 不触发

- **现象**：WebView 小游戏完成后点击返回无响应，引擎卡死在小游戏界面。
- **原因**：`build_as_child` 创建的 WebView 子窗口覆盖了整个父窗口。Windows 认为父窗口被完全遮挡，优化掉 `WM_PAINT`，导致 winit 的 `RedrawRequested` 不再触发。放在 `RedrawRequested` 中的 channel 轮询代码永远不执行。
- **正确做法**：将小游戏完成轮询移到 `ApplicationHandler::about_to_wait()` 回调中。此回调在每次事件循环迭代都会执行，不依赖窗口重绘。同时在销毁 WebView 前先调用 `set_visible(false)`，然后 `request_redraw()` 恢复父窗口渲染。

### panic 时录制缓冲区不会自动导出

- **现象**：程序 panic 后，`recordings/` 目录下没有录制文件，尽管 panic hook 提示"检查 recordings 目录"。
- **原因**：录制导出仅在 F8 手动触发。panic hook 只打印信息，不实际导出。FFI abort 场景下析构函数也无法运行。
- **正确做法**：`AppState` 实现 `Drop`，在 `std::thread::panicking()` 时自动调用 `export_recording`。可覆盖正常 unwind 的 panic；FFI abort 无法覆盖，需从根源避免 FFI 边界 panic。

### host-tauri 未处理 WaitForSignal 导致游戏无法推进（host-tauri 时期，已归档）

- **现象**：New Game 后画面一直黑屏，无法显示对话或推进剧情。
- **原因**：`run_script_tick()` 使用 `CommandExecutor::execute_batch()` 的返回值（`ExecuteResult`）来设置 Host 等待状态，但 `ChangeScene` 的 `ExecuteResult` 为 `Ok`（仅修改 RenderState），忽略了 Runtime 返回的 `WaitingReason::WaitForSignal`。Runtime 永远等待信号但 Host 不知道需要发送信号，导致脚本在第一个 `changeScene with Fade` 处永久阻塞。
- **正确做法**：用 Runtime 的 `WaitingReason`（权威来源）映射 Host 等待状态，而非 `ExecuteResult`（派生值）。`process_tick` 中检测过渡/动画完成后，通过 `RuntimeInput::Signal` 解除 Runtime 等待。同理，`WaitForTime` 也需要 Host 在 `process_tick` 中递减并解除。

### Debug HTTP Server 与 Tauri WebView 双客户端竞争（host-tauri 时期，已归档）

- **现象**：MCP 浏览器调试时，外部浏览器画面异常——打字机效果碎裂、游戏速度翻倍、对话推进不同步。直接构造 HTTP 请求（curl）则正常。
- **原因**：`AppStateInner` 通过 `Arc<Mutex<>>` 被 Tauri IPC 和 Debug HTTP Server 共享。Tauri WebView 和外部浏览器各自运行独立的 `requestAnimationFrame` → `tick(dt)` 循环，导致：①两个 tick 交替推进打字机计时器 ②用户在一侧的点击对另一侧不可见 ③游戏以 2x 速度推进。
- **正确做法**：使用 `RING_HEADLESS=1` 环境变量启动 Tauri dev，隐藏 Tauri 窗口使 WebView 的 rAF 被抑制，外部浏览器成为唯一客户端。前端 `useEngine` 的游戏循环额外检查 `document.hidden` 作为安全网。

### UIResult 后续命令被 Host 吞掉

- **现象**：`showMap` / `callGame` 返回值后，地图或小游戏本身关闭了，但对话框仍停留在旧文本；下一次点击会直接跳过本应立刻出现的那句对话。
- **原因**：Host 在 `handle_ui_result()` 中调用 `runtime.tick(Some(RuntimeInput::UIResult))` 后，只清除了等待状态，却没有消费这次 tick 立即返回的 `Command` 和 `WaitingReason`。Runtime 已经前进到下一句，但 `RenderState` 仍停留在旧帧。
- **正确做法**：把“带输入的 runtime tick”与普通 `run_script_tick()` 统一走同一条 `Command`/等待态应用逻辑，确保 UIResult 解除等待时产出的首批命令立即写入 `RenderState`。

### `ring-asset` 下 iframe HTML 的相对资源会丢失目录（host-tauri 时期，已归档）

- **现象**：小游戏在 Debug HTTP Server 中正常，但在 Tauri WebView 中打开后无法交互；日志里出现 `ring-asset 协议资源未找到 path=game.js` 之类的报错。
- **原因**：小游戏 `index.html` 直接通过 `ring-asset` 自定义协议加载时，WebView 对相对资源的 base URL 解析不稳定，`./game.js` 之类的引用可能退化成根路径请求。
- **正确做法**：宿主先读取小游戏 HTML 文本，再注入显式的 `<base href=".../games/<id>/">` 后通过 iframe `srcdoc` 加载，确保脚本、样式和图片的相对路径在 Tauri 与 Debug Server 下表现一致。

### host-tauri 脚本自然结束后仍停在 InGame（host-tauri 时期，已归档）

- **现象**：脚本跑到最后一个节点后，不再有新对话，但界面仍停留在游戏画面，像是“卡住”。
- **原因**：Host 仅把 `script_finished` 设为 `true`，并未把“自然结束”收敛到标题态，会话仍保持 `host_screen = InGame`。
- **正确做法**：当 runtime 返回“无命令且无等待”时，直接执行 `return_to_title(false)`，统一走标题返回路径并清理 continue / 会话状态。

---

## Dioxus Desktop（host-dioxus）

### Windows 自定义协议 URL 格式

- **现象**：`ring-asset://localhost/path` 格式的 URL 在 WebView 中请求时，协议 handler 完全不被调用，图片/视频无法加载。
- **原因**：wry 在 Windows（WebView2）上将自定义协议注册为 `http://{name}.localhost/` 格式。`{name}://localhost/` 格式不会触发 handler。
- **正确做法**：所有 `ring-asset` 引用须使用 `http://ring-asset.localhost/path` 格式。

### WebGL eval 桥接的渲染模式

- **现象**：从 Rust 侧逐帧通过 `document::eval()` 调用 WebGL `drawArrays()` 会导致画面闪烁（只闪一帧然后变白/黑）。
- **原因**：`document::eval()` 的调度延迟不可预测，与 WebView 的渲染帧不同步。逐帧 eval 调用可能堆积或丢失。
- **正确做法**：JS 侧用 `requestAnimationFrame` 建立自主渲染循环，读取 Rust 通过 eval 设置的全局变量（如 `window.__ruleProgress`）。Rust 只负责写变量，不负责触发绘制。

### CSS 外部文件加载

- **现象**：`with_custom_head('<link rel="stylesheet" href="/assets/poc.css">')` 注入的外部 CSS 在 `cargo run` 模式下不加载，样式不生效。
- **原因**：Dioxus Desktop 的内部资源服务在 `cargo run`（非 `dx serve`）模式下可能不提供 `/assets/` 路径的静态文件。
- **正确做法**：使用 `with_custom_head('<style>...</style>')` 内联 CSS，或通过 `ring-asset` 自定义协议加载 CSS 文件。

### Dioxus event handler 不允许 `!` 返回类型

- **现象**：`onclick: move |_| { std::process::exit(0); }` 编译失败，报 `SpawnIfAsync is not implemented for !`。
- **原因**：Dioxus 的 event handler 闭包必须返回 `()` 或 `Result<(), CapturedError>`，`std::process::exit()` 返回 `!`（never type），导致闭包类型不匹配。
- **正确做法**：使用 `dioxus::desktop::window().close()` 关闭窗口（返回 `()`），不直接调用 `std::process::exit()`。

### Dioxus eval 双向通信的正确用法

- **现象**：想在 tick loop 中同步读取 JS 键盘事件，但 `document::eval().recv()` 是异步的，无法在同步 Mutex lock 区域使用。
- **原因**：`eval().recv()` 返回 `Future`，需要在 async context 中 await。同一 eval 实例的 send/recv 通道是长生命周期的。
- **正确做法**：用独立的 `spawn(async { ... })` 循环持续 `eval.recv()` 接收 JS 推送的事件。JS 侧用 `dioxus.send(data)` 推送，Rust 侧 `await eval.recv::<T>()` 接收。不要尝试在 tick loop 的同步区域处理。

### Dioxus Signal 与 Arc<Mutex> 的配合模式

- **现象**：需要在 Dioxus 组件间共享可变的 AppStateInner，但 Signal 要求 Clone。
- **原因**：`use_context` 要求类型实现 Clone。`AppStateInner` 包含 VNRuntime 等不可 Clone 的字段。
- **正确做法**：用 `AppState { inner: Arc<Mutex<AppStateInner>> }` 包装并 `#[derive(Clone)]`。tick loop 每帧 lock → process_tick → clone RenderState 到 Signal。用户交互直接 lock 调方法。Signal 只用于只读的 RenderState 快照。

### CSS border-image 等价 NinePatch 九宫格

- **现象**：egui host 大量使用 NinePatch 九宫格渲染 UI 组件（对话框、按钮、slot 等），Dioxus WebView 中需要等价实现。
- **原因**：WebView CSS 原生支持 `border-image` + `border-image-slice`，语义完全等价于 NinePatch。
- **正确做法**：`border-image-source: url("http://ring-asset.localhost/gui/textbox.png"); border-image-slice: 30 30 30 30 fill; border-image-width: 30px; border-style: solid; background: transparent;`。`layout.json` 中的 `*_borders` 值直接映射为 `border-image-slice` 参数。必须加 `fill` 关键字否则中间区域不渲染。

### Dioxus 中 1920×1080 基准分辨率缩放

- **现象**：需要所有 CSS 值使用 1920×1080 坐标系（与 egui host 的 ScaleContext 一致），同时适配不同窗口大小。
- **原因**：在 WebView 中没有 egui 的 ScaleContext，需要等价缩放机制。
- **正确做法**：`.game-container { width: 1920px; height: 1080px; transform-origin: top left; transform: scale(var(--scale-factor)); }`，JS resize handler 计算 `--scale-factor = min(innerWidth/1920, innerHeight/1080)`。所有 CSS 值直接写 1920 基准 px，无需手动换算。

### 数据驱动 screen_defs 模块可跨宿主复用

- **现象**：egui host 的 `ui/screen_defs/mod.rs`（ConditionDef/ActionDef/ButtonDef/ScreenDefinitions）复制到 host-dioxus 几乎不需改动。
- **原因**：该模块仅依赖 `vn_runtime::state::VarValue` 和 `PersistentStore`，与 UI 框架无关。
- **正确做法**：新宿主可直接复用 screen_defs 和 layout_config 的数据结构，只需适配导入路径。screens.json / layout.json 是两个宿主共享的配置源。

---

## 如何贡献新条目

1. 在对应分类下追加。如果不属于现有分类，新建一级标题。
2. 格式：`### 简短标题` + 现象/原因/正确做法 三段。
3. 如果对应领域有 domain rule（`.cursor/rules/domain-*.mdc`），在该 rule 的 Don't 列表中也添加引用。
