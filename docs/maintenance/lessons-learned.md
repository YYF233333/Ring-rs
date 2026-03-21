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

### PowerShell 中 `&&` 语法错误

- **现象**：在 PowerShell 中用 `&&` 连接命令报语法错误。
- **原因**：PowerShell（非 pwsh 7+）不支持 `&&` 作为命令连接符。
- **正确做法**：使用 `;` 分隔命令：`cd F:\Code\Ring-rs; cargo test`。

---

## 如何贡献新条目

1. 在对应分类下追加。如果不属于现有分类，新建一级标题。
2. 格式：`### 简短标题` + 现象/原因/正确做法 三段。
3. 如果对应领域有 domain rule（`.cursor/rules/domain-*.mdc`），在该 rule 的 Don't 列表中也添加引用。
