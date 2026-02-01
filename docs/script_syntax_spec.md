# VN Script 语法规范

> 版本: 1.0.0  
> 本文档定义了 Visual Novel Engine 脚本语言的正式语法规范。
>
> **实现状态**: ✅ 解析器已完成（手写递归下降，50 个测试用例）

---

## 一、设计原则

### 1. 面向人类编剧

- 脚本主要由人类编剧在 Typora 等 Markdown 编辑器中编写
- 语法应尽量接近自然书写习惯
- 保持 Markdown 可读性，支持直接预览图片和格式（为了保证可预览，素材的路径都是相对于脚本文件的，解析时务必注意）

### 2. 容错解析

解析器应当容忍以下非语义性变化：
- 多余的空格和缩进
- 空行
- 行尾空白
- 中英文标点的合理混用（如 `：` 和 `:`）

### 3. 扩展性

语法设计应便于未来扩展新指令，而不破坏向后兼容性。

---

## 二、文件结构

VN 脚本文件使用 `.md` 扩展名，采用 UTF-8 编码。

## 三、基础语法元素

### 3.1 章节标记 (Chapter/Section)

使用 Markdown 标题语法：

```markdown
# Chapter 1
## Chapter 1-1
### Chapter 1.1
#### Chapter 1.1.1
```

章节标记用于组织脚本结构，有具体动画效果，应当被视为独立执行单元。

### 3.2 标签 (Label)

用于跳转目标，使用加粗语法：

```markdown
**label_name**
```

标签名可以为任意非空白字符，如 `**intro**`、`**标签a**`。

---

## 四、对话与旁白

### 4.1 角色对话

```markdown
角色名："对话内容"
角色名: "对话内容"
```

支持中文冒号 `：` 、英文冒号 `:`、中文引号`“”`和英文引号`""`，冒号分隔角色名称，引号标记对话内容，不可省略。

示例：
```markdown
羽艾："为什么会变成这样呢？"
路汐：“啊呀啊呀，这就受不了了吗？”
？？？："那么，世界回应了少女..."
```

### 4.2 旁白/内心独白

角色名为空时视为旁白：

```markdown
："这是旁白文本。"
: “这是旁白文本。”
```

---

## 五、演出指令

### 5.1 背景切换 (changeBG)

```markdown
changeBG <img src="path/to/image.jpg" /> with transition
changeBG <img src="path/to/image.jpg" />
```

参数说明：
- `<img src="...">`: 图片路径（支持 Typora 拖拽插入格式）
- `with transition`: 过渡效果（可选）

过渡效果支持：
- `dissolve` - 淡入淡出
- `fade` - 渐隐渐显
- `Dissolve(duration)` - 指定时长的淡入淡出，如 `Dissolve(1.5)`

### 5.2 场景切换 (changeScene)

```markdown
changeScene <img src="path/to/image.jpg" /> with transition
changeScene <img src="path/to/image.jpg" /> with <img src="rule.png" /> (duration: 1, reversed: true)
```

支持使用 rule 图片定义切换遮罩效果。

### 5.3 显示角色 (show)

```markdown
show <img src="path/to/sprite.png" /> as alias at position with transition
```

参数说明：
- `<img src="...">`: 立绘图片路径
- `as alias`: 角色别名，用于后续引用（如 `as royu`）
- `at position`: 位置（见下方位置定义）
- `with transition`: 过渡效果（可选）

**位置定义**：

| 位置名 | 说明 |
|--------|------|
| `left` | 左侧 |
| `right` | 右侧 |
| `center` / `middle` | 中央 |
| `nearleft` | 近左 |
| `nearright` | 近右 |
| `nearmiddle` | 近中 |
| `farleft` | 远左 |
| `farright` | 远右 |
| `farmiddle` | 远中 |

示例：
```markdown
show <img src="assets/立绘1-惊讶.png" /> as royu at nearmiddle with dissolve
```

### 5.4 隐藏角色 (hide)

```markdown
hide alias with transition
```

示例：
```markdown
hide royu with fade
```

### 5.5 UI 动画 (UIAnim)

```markdown
UIAnim fade
UIAnim Dissolve(1.5)
```

用于触发全局 UI 动画效果。

---

## 六、分支选择

使用 Markdown 表格语法：

```markdown
| 横排   |        |
| ------ | ------ |
| 选项1  | label1 |
| 选项2  | label2 |
| 选项3  | label3 |
```

表格结构：
- 第一行为表头（首个单元格指定分支界面的样式）
- 后续每行定义一个选项
- 第一列：选项显示文本
- 第二列：跳转目标 label

解析器应忽略表格中的额外空格和对齐字符。

---

## 音乐与音效

使用HTML audio语法播放音乐

```markdown
<audio src="../bgm/Signal.mp3"></audio>
<audio src="../bgm/Signal.mp3"></audio> loop
```

参数说明：
- `<audio src="...">`: 音频文件路径
- `loop`: 标识BGM，循环播放，没有该标识认为是SFX，play once

同一时间只能有一个BGM播放，播放下一个会自动停止前一个，BGM切换自带交叉淡化效果。SFX不做限制。

停止BGM：

```markdown
stopBGM
```

> 备注：音量/静音属于**玩家设置选项**，脚本层不提供音量/静音控制能力；制作时应尽量保证不同 BGM 的响度一致。

## 控制逻辑

无条件跳转：

```markdown
goto **label**
```

## 七、过渡效果语法

### 7.1 统一函数调用语法

所有过渡效果使用统一的**函数调用语法**，解析器不解释具体效果语义：

```
with <effect_expr>

effect_expr := identifier                      // 无参效果
             | identifier(arg, arg, ...)       // 带参效果
```

**示例**：
```markdown
with dissolve                    // 无参数
with fade
with Dissolve(1.5)               // 位置参数
with rule("mask.png", 1.0, true) // 多个位置参数
```

### 7.2 解析器产出结构

解析器将 `with` 子句解析为通用的 `Transition` 结构：

```rust
pub struct Transition {
    pub name: String,           // 效果名，如 "dissolve", "Dissolve", "rule"
    pub args: Vec<TransitionArg>,
}

pub enum TransitionArg {
    Number(f64),                // 数字，如 1.5
    String(String),             // 字符串，如 "mask.png"
    Bool(bool),                 // 布尔值，如 true/false
}
```

**设计理由**：
- 解析器只负责**结构提取**，不需要知道有哪些效果
- 新增效果时，只需在 Runtime/Host 层添加处理逻辑
- 避免"效果数 × 操作数"的规则爆炸

### 7.3 内置效果参考

以下是 Runtime 层支持的内置效果（解析器无需感知）：

| 效果名 | 语法 | 说明 |
|--------|------|------|
| dissolve | `dissolve` 或 `Dissolve(秒数)` | 淡入淡出 |
| fade | `fade` 或 `Fade(秒数)` | 渐隐渐显 |
| rule | `rule("mask.png", duration, reversed)` | 遮罩过渡 |

> **注意**：效果名大小写敏感，`dissolve` 和 `Dissolve(1.5)` 是不同的效果标识。

---

## 八、图片路径规范

### 8.1 HTML img 标签（推荐）

用于 Typora 等可视化编辑器的兼容：

```html
<img src="assets/images/bg.webp" alt="bg" style="zoom:10%;" />
```

解析器提取 `src` 属性作为实际路径。`alt`、`style` 等属性被忽略。

## 九、完整示例

```markdown
# Chapter 1

changeBG <img src="assets/images/bg/hospital.webp" /> with dissolve

？？？："唔…"
？？？："好冷…"

："中央空调正呼呼地吹出冷风。"
："刚刚还在梦中的我突然醒转过来。"

show <img src="assets/立绘1-惊讶.png" /> as protagonist at center with dissolve

**choice_point**

| 选择   |           |
| ------ | --------- |
| 继续睡 | sleep     |
| 起床   | wake_up   |

**sleep**

："我决定继续睡下去..."

**wake_up**

："我强撑着坐了起来。"

hide protagonist with fade
```

---

## 十、解析器实现指南

### 10.1 两阶段解析架构

解析器采用**两阶段架构**，将块识别与块内解析分离：

```
┌─────────────────────────────────────────────────────────┐
│                    原始脚本文本                           │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│              阶段 1: 块识别 (Block Recognition)          │
│                                                         │
│  规则：                                                  │
│  - 以 `|` 开头的连续行 → Table 块                        │
│  - 其他非空行 → SingleLine 块                            │
│  - 空行 → 块分隔符（不产生块）                            │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
                    Vec<Block>
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│              阶段 2: 块内解析 (Block Parsing)            │
│                                                         │
│  SingleLine Parser:                                     │
│    - 章节标记、标签、对话、指令等                          │
│                                                         │
│  Table Parser:                                          │
│    - 选择分支表格                                        │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
                  Vec<ScriptNode>
```

**设计理由**：
- **关注点分离**：块边界识别与块内语法解析独立
- **易于扩展**：新增多行结构只需添加新的块类型
- **无二义性**：块边界规则简单明确，不会随语法增多而产生冲突

### 10.2 块类型定义

```rust
enum Block {
    /// 单行内容（对话、指令、标签等）
    SingleLine { line: String, line_number: usize },
    
    /// 表格块（选择分支）
    Table { lines: Vec<String>, start_line: usize },
}
```

### 10.3 阶段 1：块识别规则

```
输入文本逐行处理：

1. 空行 → 不产生块，仅作为块分隔
2. 以 `|` 开头 → 
   - 如果前一个块也是 Table，合并到该块
   - 否则，开始新的 Table 块
3. 其他行 → SingleLine 块
```

**示例**：

```
输入:
  changeBG <img src="bg.png" />
  ："对话内容"
  | 表头 |        |
  | ---- | ------ |
  | 选项1 | label1 |
  | 选项2 | label2 |
  ："继续对话"

输出:
  Block::SingleLine("changeBG <img src=\"bg.png\" />")
  Block::SingleLine("：\"对话内容\"")
  Block::Table([
    "| 表头 |        |",
    "| ---- | ------ |",
    "| 选项1 | label1 |",
    "| 选项2 | label2 |"
  ])
  Block::SingleLine("：\"继续对话\"")
```

### 10.4 阶段 2：SingleLine 解析优先级

对于 `SingleLine` 块，按以下优先级识别行类型：

1. `#` 开头 → 章节标记
2. `**...**` 格式 → 标签定义
3. 指令关键字开头（大小写不敏感）→ 演出指令
   - `changeBG`, `changeScene`, `show`, `hide`, `UIAnim`
4. 包含 `：` 或 `:` → 对话/旁白
5. 其他 → 未知行，记录警告但不中断解析

### 10.5 阶段 2：Table 解析规则

对于 `Table` 块：

1. 第一行：表头（提取首个单元格作为样式标识）
2. 第二行：分隔符行（`| --- | --- |` 格式，跳过）
3. 后续行：选项行
   - 第一列：选项显示文本
   - 第二列：跳转目标 label

### 10.6 容错规则

- 指令关键字大小写不敏感（`changeBG` = `changebg` = `ChangeBG`）
- 多余空格自动忽略
- 支持 Windows (CRLF) 和 Unix (LF) 换行符
- 表格分隔符 `|` 两侧的空格自动 trim
- 解析错误时记录行号，便于调试

---

## 附录：与 Ren'Py 的对比

| 功能 | Ren'Py | 本引擎 |
|------|--------|--------|
| 对话 | `e "Hello"` | `角色："对话"` |
| 显示 | `show eileen happy at left` | `show <img> as alias at position` |
| 隐藏 | `hide eileen` | `hide alias` |
| 标签 | `label start:` | `**start**` |
| 跳转 | `jump label` | 通过选择分支跳转 |
| 背景 | `scene bg room` | `changeBG <img>` |

本引擎语法受 Ren'Py 启发，但针对 Markdown 可读性和 Typora 编辑体验进行了优化。

---

> **文档维护说明**：
> 本规范应随引擎功能演进而更新。任何语法变更需要在此文档中记录。

