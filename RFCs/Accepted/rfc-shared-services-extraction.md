# RFC: 共享服务层提取

## 元信息

- 编号：RFC-025
- 状态：Accepted
- 作者：claude-4.6-opus
- 日期：2026-03-24
- 影响范围：`host`（app/audio/resources/save_manager/config）
- 前置：无（可立即开始）
- 父 RFC：RFC-024（VN+ Hub 架构愿景）

---

## 背景

当前 `host` 的各子系统（AudioManager、ResourceManager、SaveManager 等）在接口和实现上隐含了 VN-specific 的假设。这些假设在纯 VN 场景下不构成问题，但当 Hub 架构引入多模态后，非 VN mode 将难以复用这些服务。

本 RFC 的目标是**在日常 VN 开发过程中，以低成本增量方式**消除这些 VN 假设，使这些子系统成为任何 mode 都能使用的共享服务。这是 Hub 架构的地基，但即使不实施 Hub，这些解耦也提升了代码质量。

**实施时机：可立即开始，作为日常重构的一部分推进。**

---

## 目标与非目标

### 目标

- 识别并消除共享服务中的 VN-specific 耦合点
- 存档格式预留模态扩展点，使非 VN mode 的状态可序列化
- 确保改动对现有 VN 行为无可观测影响（纯内部重构）

### 非目标

- 不引入 mode 概念或 Mode trait（RFC-026 的职责）
- 不重构 VN 渲染管线（那是 vn_mode 的内部事务）
- 不新增功能——纯解耦重构
- 不过度抽象——只解耦已识别的具体耦合点

---

## 方案设计

### 待审查的耦合点

以下是需要逐项审查的候选耦合点。每个点在实施前应先确认是否真的构成阻碍，避免过度重构。

#### 1. SaveManager：存档格式扩展点

**现状**：`SaveData` 序列化 `RuntimeState`，紧密绑定 VN 状态模型。

**建议**：在存档格式中预留可选的扩展字段，使非 VN mode 的状态可以附着在存档上。

```rust
pub struct SaveData {
    pub runtime_state: RuntimeState,
    pub engine_version: String,
    // 新增：模态扩展数据（各 mode 可存入自己的状态）
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mode_data: BTreeMap<String, serde_json::Value>,
}
```

**成本**：极低（新增一个 Option 字段），不影响现有存档的反序列化兼容性。

#### 2. AudioManager：接口中立性

**现状**：AudioManager 已基本中立（BGM/SFX/voice 的 play/stop/fade），但需确认是否有方法签名隐含了 VN 时序假设（如"对话结束时停止 voice"）。

**建议**：审查 AudioManager 的 public API，确保所有方法可被非 VN 调用方使用。如有 VN-specific 的便利方法（如 `stop_voice_on_dialogue_advance`），将其上移到 VN 调用侧。

#### 3. ResourceManager：路径解析

**现状**：ResourceManager 使用 `LogicalPath` 统一路径解析，已较好解耦。

**建议**：确认资源目录结构是否允许 mode-specific 的资源命名空间（如 `modes/card_battle/cards/`），避免与 VN 资源路径冲突。这可能只需要文档约定而非代码改动。

#### 4. Config：模态无关的配置模型

**现状**：`config.json` 包含窗口、音频、调试等全局配置，以及 VN-specific 的配置（如 `text_speed`、`auto_delay`）。

**建议**：概念上区分"全局配置"和"VN 模态配置"，但不急于拆分文件。可以先在文档中标注哪些字段是全局的、哪些是 VN-specific 的，为将来 mode-specific 配置做准备。

#### 5. InputManager：事件模型

**现状**：InputManager 采集 winit 事件，转换为 `RuntimeInput`。`RuntimeInput` 是 VN-specific 的（Click/ChoiceSelected/Signal/UIResult）。

**建议**：InputManager 本身的低层事件采集（键鼠状态、防抖、长按）是模态无关的。VN-specific 的转换（按键 → RuntimeInput）应明确为 vn_mode 的职责，而非 InputManager 的固有行为。这个拆分可以在 RFC-026 实施时一并完成，当前仅标记。

---

## 实施策略

### 增量推进，不设专门周期

这些改动不需要集中实施。建议在日常 VN 开发中，当触及相关模块时顺手完成：

1. **改 SaveManager 时** → 加入 `mode_data` 字段
2. **改 AudioManager 时** → 审查并上移 VN-specific 便利方法
3. **改 Config 时** → 在文档中标注字段归属
4. **改 ResourceManager 时** → 确认目录命名空间策略

### 完成标准

每个耦合点独立判定，全部完成后本 RFC 标记为 Accepted：

- [x] SaveData 包含 `mode_data` 扩展字段，现有存档兼容性不受影响
- [x] AudioManager 的 public API 无 VN-specific 方法（审查确认已中立，无需改动）
- [x] ResourceManager 目录命名空间策略文档化
- [x] Config 字段归属（全局 vs VN-specific）文档化
- [x] InputManager 的 VN-specific 转换逻辑标记待拆分（RFC-026 时执行）
- [x] `cargo check-all` 通过（876 tests passed）
- [x] 相关模块摘要更新（save-manager、config）

---

## 风险

| 风险 | 缓解 |
|------|------|
| 过度重构：为假设的 mode 需求提前抽象 | 只处理已识别的具体耦合点，不预设 |
| 存档兼容性 | `mode_data` 使用 `serde(default)` + `skip_serializing_if`，不影响旧存档 |
| 改动分散难追踪 | 在本 RFC 的实施进度章节记录每个耦合点的完成状态 |
