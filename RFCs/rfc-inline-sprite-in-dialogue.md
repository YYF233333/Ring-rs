# RFC: 对话内联差分语法

## 元信息

- 编号：RFC-005
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-10
- 相关范围：`docs/authoring/script-syntax.md`、`vn-runtime/src/script/parser/`、`vn-runtime/src/command/mod.rs`、`host/src/command_executor/`

---

## 1. 背景

对照 ref-project（Ren'Py），差分切换与对话几乎总是在同一行发生：

```renpy
红叶 summer_normal1 "我倒是觉得有几个写的不错的。"
红叶 summer_backhand3 "再说一遍？"
```

当前语法中，切换差分需要独立的 `show` 行，导致同等内容行数约为 Ren'Py 的 1.5–2 倍：

```markdown
show <img src="assets/红叶-normal1.png" /> as royu at s11
红叶："我倒是觉得有几个写的不错的。"
show <img src="assets/红叶-backhand3.png" /> as royu at s11
红叶："再说一遍？"
```

同时，项目存在"可视化需求"：编剧在 Typora 中编写时应能预览差分图片，以便在写作时直观判断当前表情是否符合情境。

---

## 2. 目标与非目标

### 2.1 目标

- 支持在对话行中同时内联切换差分图片
- 内联的 `<img>` 标签在 Typora 中**渲染为缩略图**，满足可视化预览需求
- 与已有的无差分对话语法（`角色名："..."`) 保持向后兼容

### 2.2 非目标

- 不支持在对话行中变更角色的**位置**（位置变更仍需独立 `show` 行）
- 不支持在对话行中指定**过渡效果**（差分切换默认为原地瞬切）
- 不替代 `show`：首次入场、移动、显式过渡效果仍使用 `show`

---

## 3. 提案

### 3.1 语法形式

在角色名与冒号之间插入 `<img>` 标签：

```markdown
角色名 <img src="path/to/sprite.png" />："对话文本"
角色名 <img src="path/to/sprite.png" style="zoom:8%;" />："对话文本"
```

`style` 等额外属性由 Typora 使用，解析器只提取 `src`，与现有 `show` 的图片解析规则一致。

**示例**：

```markdown
红叶 <img src="assets/立绘/红叶-normal1.png" style="zoom:8%;" />："我倒是觉得有几个写的不错的。"
红叶 <img src="assets/立绘/红叶-backhand3.png" style="zoom:8%;" />："再说一遍？"
```

Typora 渲染效果：`红叶 [缩略图] ："对话文本"` — 编剧可在写作时直观看到差分图片。

### 3.2 引擎语义

内联差分对话行等价于以下两条独立指令的合并执行：

1. `show <img> as <当前绑定该角色的 alias>` at `<当前位置>`（原地差分切换，无过渡）
2. 显示对话

**别名绑定规则**：

- 引擎维护 `角色显示名 → alias` 的映射表。该映射在执行 `show <img> as alias` 时自动建立（`alias` 作为角色名的绑定 key）。
- 内联差分时，按角色名查找映射表，找到对应 alias 后更新差分。
- 若角色名在映射表中**不存在**（从未 `show` 过），运行时报错并提示需要先用 `show` 建立绑定。

> **注意**：别名映射基于**角色显示名**（如 `红叶`）。若同一显示名绑定了多个 alias（非常规用法），引擎取最近一次绑定的 alias。

### 3.3 与现有语法的关系

| 写法 | 语义 |
|------|------|
| `角色名："对话"` | 对话，不改变差分 |
| `角色名 <img>："对话"` | 原地切换差分 + 对话（本 RFC 新增） |
| `show <img> as alias at pos with trans` | 完整入场/移动/过渡（保持不变） |
| `show alias at pos with move` | 移动（保持不变） |

---

## 4. 解析器变更

### 4.1 AST 节点

对话节点新增可选的差分字段：

```rust
pub struct DialogueNode {
    pub character: Option<String>,     // None = 旁白
    pub sprite: Option<ImagePath>,     // 新增：内联差分（None = 不切换）
    pub text: String,
}
```

### 4.2 解析规则

对话行识别逻辑（现有：`角色名：`）扩展为：

```
dialogue_line := character_name [SPACE <img_tag>] COLON QUOTE text QUOTE
```

即在字符名与冒号之间，可选出现一个 `<img>` 标签。

解析器提取 `src` 属性，存入 `DialogueNode::sprite`。其他属性（`alt`、`style`）忽略，与 `show` 指令的图片解析保持一致。

**旁白**（`:"文本"`）不支持内联差分（旁白无角色实体，不需要差分）。

---

## 5. Command 层变更

`Command::ShowDialogue`（或等价命令）新增可选的 `sprite_update` 字段：

```rust
pub struct ShowDialogue {
    pub character: Option<String>,
    pub sprite_update: Option<String>,  // 新增：差分图片的绝对路径（None = 不切换）
    pub text: String,
}
```

Host 的 command_executor 在显示对话前，若 `sprite_update` 有值，先执行差分更新（等效于无过渡的原地 `show`）。

---

## 6. 风险

- **解析歧义**：对话行中出现 `<img>` 但不符合预期格式时，应产生明确错误信息，而非静默退化为旁白或丢弃图片。
- **首次使用未绑定 alias**：运行时报错行为需与 `show alias at ...`（alias 未绑定时）的错误提示风格一致。
- **多 alias 同名字符**：属于非常规用法，文档说明行为（取最近绑定），不需要额外防护。

---

## 7. 迁移计划

1. 更新 `docs/authoring/script-syntax.md`：§4.1 对话语法增加内联差分形式，§5.3 `show` 说明中注明与内联差分的分工
2. 更新解析器，增加对应测试用例（含：有差分/无差分/旁白/alias 未绑定错误）
3. 更新 Command 定义与 Host executor
4. 更新 `docs/engine/architecture/module-summaries/` 对应摘要

---

## 8. 验收标准（DoD）

- [ ] `角色名 <img src="..."/>："对话"` 成功解析，`sprite` 字段有值
- [ ] `角色名："对话"` 解析不受影响，`sprite` 为 None
- [ ] 对话行中的 `<img>` 路径按"相对脚本文件目录"解析，与 `show` 行为一致
- [ ] alias 未绑定时运行时报错，信息明确
- [ ] 旁白行（`:"..."`) 不受影响
- [ ] Typora 中打开含内联差分的脚本，图片正常渲染为缩略图（人工验证）
- [ ] 解析器新增对应测试用例
- [ ] `docs/authoring/script-syntax.md` 完成更新
