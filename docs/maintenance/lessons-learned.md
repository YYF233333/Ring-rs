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

### host-tauri 未处理 WaitForSignal 导致游戏无法推进

- **现象**：New Game 后画面一直黑屏，无法显示对话或推进剧情。
- **原因**：`run_script_tick()` 使用 `CommandExecutor::execute_batch()` 的返回值（`ExecuteResult`）来设置 Host 等待状态，但 `ChangeScene` 的 `ExecuteResult` 为 `Ok`（仅修改 RenderState），忽略了 Runtime 返回的 `WaitingReason::WaitForSignal`。Runtime 永远等待信号但 Host 不知道需要发送信号，导致脚本在第一个 `changeScene with Fade` 处永久阻塞。
- **正确做法**：用 Runtime 的 `WaitingReason`（权威来源）映射 Host 等待状态，而非 `ExecuteResult`（派生值）。`process_tick` 中检测过渡/动画完成后，通过 `RuntimeInput::Signal` 解除 Runtime 等待。同理，`WaitForTime` 也需要 Host 在 `process_tick` 中递减并解除。

### Debug HTTP Server 与 Tauri WebView 双客户端竞争

- **现象**：MCP 浏览器调试时，外部浏览器画面异常——打字机效果碎裂、游戏速度翻倍、对话推进不同步。直接构造 HTTP 请求（curl）则正常。
- **原因**：`AppStateInner` 通过 `Arc<Mutex<>>` 被 Tauri IPC 和 Debug HTTP Server 共享。Tauri WebView 和外部浏览器各自运行独立的 `requestAnimationFrame` → `tick(dt)` 循环，导致：①两个 tick 交替推进打字机计时器 ②用户在一侧的点击对另一侧不可见 ③游戏以 2x 速度推进。
- **正确做法**：使用 `RING_HEADLESS=1` 环境变量启动 Tauri dev，隐藏 Tauri 窗口使 WebView 的 rAF 被抑制，外部浏览器成为唯一客户端。前端 `useEngine` 的游戏循环额外检查 `document.hidden` 作为安全网。

---

## 如何贡献新条目

1. 在对应分类下追加。如果不属于现有分类，新建一级标题。
2. 格式：`### 简短标题` + 现象/原因/正确做法 三段。
3. 如果对应领域有 domain rule（`.cursor/rules/domain-*.mdc`），在该 rule 的 Don't 列表中也添加引用。
