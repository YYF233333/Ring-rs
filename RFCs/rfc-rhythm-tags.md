# RFC: 节奏标签与 extend 台词续接

## 元信息

- 编号：RFC-006
- 状态：Implemented
- 作者：Ring-rs 开发组
- 日期：2026-03-10（实施：2026-03-10）
- 相关范围：`vn-runtime`（AST / Parser / Command / Executor）、`host`（打字机 / 渲染 / 模式控制）
- 前置：P0-3 文本节奏与窗口控制

---

## 1. 背景

当前对话文本以纯 `String` 存储和传递（`ScriptNode::Dialogue { content: String }` → `Command::ShowText { content: String }` → `DialogueState { content: String }`），无法控制文字显示节奏。

`.rpy` 原始稿中约 80+ 处使用了 Ren'Py 节奏标签（`{w}`/`{nw}`/`{cps}`）和 `extend` 命令，集中在 prologue、2-3、3-1、3-5、3-7、ending 等关键章节，是营造阅读节奏的核心手段。Ring `.md` 语义稿目前尚未使用这些标签，需先实现引擎支持，再逐步迁移。

---

## 2. 目标与非目标

### 2.1 目标

- 内联等待标签：`{wait}` / `{wait Ns}`
- 行尾自动推进修饰符：`-->`
- 字速控制标签：`{speed N}` / `{speed Nx}` / `{/speed}`
- `extend` 台词续接命令

### 2.2 非目标

- `{p}` 段落清屏（原稿未使用，0 处）
- `{done}` 隐藏后续文本（用 extend 链式替代）
- `extend` 中的 sprite/表情变更（留后续迭代）
- 中文标签别名（如 `{等}`、`{快}`，留后续按需添加）

---

## 3. 语法设计

### 3.1 设计原则

- **语义化**：标签名自解释，新手无需查文档即可理解
- **行内 vs 行级分离**：文本内部的效果（等待、变速）用行内标签；行为修饰（自动推进）用行尾修饰符
- **Markdown 友好**：不与 Markdown 语法冲突，在编辑器中保持可读

### 3.2 标签语法

#### 3.2.1 `{wait}` / `{wait Ns}` — 内联等待

出现在引号内文本中，打字机显示到该位置时暂停。

```markdown
红叶："差不多吧。{wait 1s}"
红叶："子文，{wait 0.5s}你的作品{wait}怎么样了？"
```

- `{wait Ns}` — 暂停 N 秒后自动继续（N 为正浮点数，单位 `s` 可选）
- `{wait}` — 暂停直到玩家点击

#### 3.2.2 `-->` — 行尾自动推进

出现在整行末尾（引号外），表示"本句显示完后自动推进到下一条，不等待玩家点击"。

```markdown
红叶："差不多吧。{wait 1s}" -->
extend "哦我突然想起来，{wait 0.7s}" -->
extend "你的社团大作业还没提交吧？"
```

- `-->` 必须出现在行末（引号闭合之后，可有空白）
- 不带 `-->` 的行默认等待点击（当前行为）
- 语义等价于 Ren'Py 的 `{nw}`，但从文本内部提升到行级

#### 3.2.3 `{speed N}` / `{speed Nx}` / `{/speed}` — 字速控制

包裹文本片段，改变该片段的打字速度。

```markdown
红叶："{speed 2x}我没事！不要管我！{/speed}"
红叶："{speed 20}喂，妈，是我，红叶，{wait 1s}对是我。"
```

- `{speed N}` — 绝对字速，N 字符/秒
- `{speed Nx}` — 相对字速，当前基础速度的 N 倍（如 `2x` = 两倍速，`0.5x` = 半速）
- `{/speed}` — 重置到用户设置的默认字速
- 不闭合的 `{speed}` 效果持续到文本末尾（含 extend 追加）

#### 3.2.4 `extend` — 台词续接

行首命令，不清屏，将新文本追加到当前对话框。

```markdown
extend "追加文本"
extend "追加文本{wait 0.5s}" -->
```

- 继承上一行的 speaker
- 打字机从当前位置继续（`visible_chars` 不重置）
- 同样支持 `{wait}`、`{speed}` 标签和 `-->` 修饰符

### 3.3 与 Ren'Py 语法对照

| Ren'Py | Ring | 说明 |
|--------|------|------|
| `{w}` | `{wait}` | 点击等待 |
| `{w=1.0}` | `{wait 1s}` | 定时等待 |
| `{nw}` | `-->` | 自动推进（从行内提升到行尾） |
| `{cps=20}` | `{speed 20}` | 绝对字速 |
| `{cps=*2}` | `{speed 2x}` | 相对字速 |
| `{/cps}` | `{/speed}` | 重置字速 |
| `extend "text"` | `extend "text"` | 台词续接（语法一致） |

---

## 4. 数据模型

### 4.1 InlineEffect

```rust
/// 内联效果（标记在纯文本的字符位置上）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineEffect {
    /// 触发位置（纯文本中的字符索引，0-based）
    pub position: usize,
    /// 效果类型
    pub kind: InlineEffectKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineEffectKind {
    /// {wait} 或 {wait Ns}
    Wait(Option<f64>),
    /// {speed N} — 绝对字速（字符/秒）
    SetCpsAbsolute(f64),
    /// {speed Nx} — 相对字速倍率
    SetCpsRelative(f64),
    /// {/speed} — 重置字速
    ResetCps,
}
```

### 4.2 解析示例

输入：`"差不多吧。{wait 1s}"`，行尾有 `-->`

输出：
- `content`: `"差不多吧。"`（纯文本，标签已剥离）
- `inline_effects`: `[InlineEffect { position: 5, kind: Wait(Some(1.0)) }]`
- `no_wait`: `true`（因为行尾有 `-->`）

输入：`"{speed 2x}我没事！不要管我！{/speed}"`

输出：
- `content`: `"我没事！不要管我！"`
- `inline_effects`: `[{ position: 0, kind: SetCpsRelative(2.0) }, { position: 9, kind: ResetCps }]`
- `no_wait`: `false`

### 4.3 设计理由：分离纯文本与效果列表

- **历史回看**：直接使用 `content`（纯净文本），无需剥离标签
- **存档**：`DialogueState` 不直接序列化存档，读档后重建，影响最小
- **打字机**：按 `visible_chars` 逐字推进时，查表 `inline_effects` 触发效果
- **职责清晰**：vn-runtime 负责解析标签语义；Host 负责渲染行为

---

## 5. 各层变更

### 5.1 vn-runtime：新增 inline_tags 解析模块

- 新文件：`vn-runtime/src/script/parser/inline_tags.rs`
- 核心函数：`fn parse_inline_tags(raw: &str) -> (String, Vec<InlineEffect>)`
  - 输入：引号内原始文本
  - 输出：(纯文本, 效果列表)
  - `-->` 的检测在 Phase2 层面处理（行级，不在引号内）
- 解析策略：扫描文本，遇到 `{` 开始匹配标签名，提取参数，记录当前纯文本偏移

### 5.2 AST 变更

`vn-runtime/src/script/ast/mod.rs`：

```rust
Dialogue {
    speaker: Option<String>,
    content: String,                   // 纯文本（标签已剥离）
    inline_effects: Vec<InlineEffect>, // 新增
    no_wait: bool,                     // 新增（来自 --> 行修饰符）
},

/// 台词续接（不清屏追加文本）
Extend {
    content: String,
    inline_effects: Vec<InlineEffect>,
    no_wait: bool,
},
```

### 5.3 Command 变更

`vn-runtime/src/command/mod.rs`：

```rust
ShowText {
    speaker: Option<String>,
    content: String,
    inline_effects: Vec<InlineEffect>, // 新增
    no_wait: bool,                     // 新增
},

/// 台词续接
ExtendText {
    content: String,
    inline_effects: Vec<InlineEffect>,
    no_wait: bool,
},
```

### 5.4 Executor 变更

`vn-runtime/src/runtime/executor/mod.rs`：

- `ScriptNode::Dialogue` → `Command::ShowText`：传递 `inline_effects` 和 `no_wait`
- `ScriptNode::Extend` → `Command::ExtendText` + `WaitForClick`
- 等待策略：**始终产生 `WaitForClick`**，`no_wait` 由 Host 侧处理
  - 理由：与 Auto 模式统一机制——Host 在打字机完成后自动发送 Click

### 5.5 历史记录变更

`vn-runtime/src/history.rs` + `vn-runtime/src/runtime/engine/mod.rs`：

- `record_history` 对 `ShowText`：只记录 `content`（纯文本），行为不变
- `record_history` 对 `ExtendText`：追加到最近一条 `HistoryEvent::Dialogue` 的 `content`，而非新建事件

### 5.6 Host：DialogueState 扩展

`host/src/renderer/render_state/mod.rs`：

```rust
pub struct DialogueState {
    pub speaker: Option<String>,
    pub content: String,
    pub visible_chars: usize,
    pub is_complete: bool,
    // --- 新增 ---
    pub inline_effects: Vec<InlineEffect>,
    pub no_wait: bool,
    pub inline_wait: Option<InlineWait>,
    pub effective_cps: Option<EffectiveCps>,
}

/// 打字机内联等待状态
#[derive(Debug, Clone)]
pub struct InlineWait {
    /// 剩余等待时间（None = 等待点击）
    pub remaining: Option<f64>,
}

/// 当前有效字速覆盖
#[derive(Debug, Clone)]
pub enum EffectiveCps {
    Absolute(f64),
    Relative(f64),
}
```

### 5.7 Host：打字机行为变更

`host/src/app/update/modes.rs` 通用打字机推进逻辑：

```
每帧更新:
  if dialogue.inline_wait 存在:
    match inline_wait.remaining:
      Some(t) => 递减 t by dt；到 0 → 清除 inline_wait
      None    => 不推进（等玩家点击跳过此等待）
  else:
    计算有效字速:
      match dialogue.effective_cps:
        Some(Absolute(n)) → n
        Some(Relative(m)) → user_settings.text_speed * m
        None              → user_settings.text_speed
    timer += dt * effective_speed
    while timer >= 1.0:
      timer -= 1.0
      advance visible_chars by 1
      检查 inline_effects 中 position == visible_chars 的效果:
        Wait(duration)     → 设置 inline_wait { remaining: duration }; break
        SetCpsAbsolute(n)  → effective_cps = Some(Absolute(n))
        SetCpsRelative(m)  → effective_cps = Some(Relative(m))
        ResetCps           → effective_cps = None
```

### 5.8 Host：点击行为变更

`host/src/app/update/script.rs` `handle_script_mode_input`：

```
用户点击时:
  if 打字机处于 inline_wait（点击等待，remaining = None）:
    清除 inline_wait，恢复打字机推进    ← 跳过当前等待点，不完成全部文本
  elif 打字机未完成（正在打字 / 定时等待中）:
    complete_typewriter()               ← 跳过所有效果和等待，直接显示全部
  elif 打字机已完成:
    run_script_tick()                   ← 推进脚本
```

与当前行为的差异：引入了 `inline_wait` 点击等待时"跳过等待但不完成全部"的分支。

### 5.9 Host：no_wait 自动推进

打字机完成后（`is_complete && no_wait`）：

- Normal 模式：立即触发 `run_script_tick(Some(RuntimeInput::Click))`，无需等待
- Auto 模式：同 Normal（`no_wait` 覆盖 `auto_delay`）
- Skip 模式：行为不变（已经是自动推进）

实现位置：在 `modes.rs` 通用打字机更新之后、各模式分支之前，统一检查。

### 5.10 Host：ExtendText 处理

`host/src/command_executor/ui.rs`：

```rust
fn execute_extend_text(
    &mut self,
    content: &str,
    inline_effects: Vec<InlineEffect>,
    no_wait: bool,
    render_state: &mut RenderState,
) -> ExecuteResult {
    if let Some(ref mut dialogue) = render_state.dialogue {
        let offset = dialogue.content.chars().count();
        dialogue.content.push_str(content);
        for mut effect in inline_effects {
            effect.position += offset;
            dialogue.inline_effects.push(effect);
        }
        dialogue.no_wait = no_wait;
        dialogue.is_complete = false;
        // visible_chars 不重置，打字机从当前位置继续
    }
    ExecuteResult::WaitForClick
}
```

### 5.11 Host：Skip 模式

- `complete_typewriter()` 清除所有 `inline_wait`、重置 `effective_cps`，直接 `visible_chars = total`
- `no_wait` 在 Skip 模式下同样立即推进（当前行为已兼容）

### 5.12 Phase2：解析 `-->` 和 `extend`

`vn-runtime/src/script/parser/phase2.rs`：

- **`-->` 检测**：在 `parse_line()` 入口处，先检查行尾是否有 `-->`，若有则去除并设置 `no_wait = true`，再进入正常解析流程
- **`extend` 解析**：识别行首 `extend` 关键字 → 提取引号内容 → `parse_inline_tags()` → `ScriptNode::Extend { ... }`

---

## 6. 风险

- **打字机复杂度增加**：`inline_wait` + `effective_cps` 引入新的时序状态，需充分测试 Normal/Auto/Skip 三种模式组合
- **`{wait}` 中途等待的 UX**：点击行为从"一击完成全部"变为"一击跳过当前等待点"，不过这与 Ren'Py `{w}` 行为一致，玩家预期合理
- **存档兼容性**：`Command::ShowText` 签名变更（新增 `inline_effects` 和 `no_wait`），但旧存档不直接序列化 `DialogueState`，读档后重建对话状态，影响可控

---

## 7. 迁移计划

1. 实现引擎支持（本 RFC 范围）
2. 无标签的对话行行为完全不变（`inline_effects: vec![]`, `no_wait: false`），向后兼容
3. `.rpy` 中的节奏标签逐步迁移到 `.md` 语义稿（独立任务，不在本 RFC 范围内）

---

## 8. 验收标准（DoD）

- [ ] `parse_inline_tags` 单元测试覆盖：`{wait}`、`{wait Ns}`、`{speed N}`、`{speed Nx}`、`{/speed}`、组合标签、嵌套/相邻、无标签文本
- [ ] Phase2 测试：`-->` 行修饰符正确提取 `no_wait`；`extend` 解析正确
- [ ] Executor 测试：Dialogue/Extend 正确生成 Command（含 inline_effects 和 no_wait）
- [ ] Host 打字机测试：`{wait}` 暂停、`{wait Ns}` 定时暂停、`{speed}` 变速、`-->` 自动推进
- [ ] extend 链式追加正确：文本拼接 + 效果位置偏移 + 打字机续接
- [ ] Skip/Auto/Normal 三种模式下行为正确
- [ ] 历史记录显示纯净文本（无标签残留）；extend 追加到同一条历史
