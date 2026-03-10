# VN Script 语法规范

> 版本: 1.1.0  
> 本文档定义了 Visual Novel Engine 脚本语言的正式语法规范。
>
> **实现状态**: ✅ 解析器已完成（手写递归下降，176 个测试用例）
>
> **1.1.0 新增**: 变量系统（set）、条件分支（if/elseif/else/endif）、表达式求值

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

> **职责**：简单背景切换，不涉及遮罩或复合演出流程。

```markdown
changeBG <img src="path/to/image.jpg" />
changeBG <img src="path/to/image.jpg" /> with dissolve
changeBG <img src="path/to/image.jpg" /> with Dissolve(duration: 1.5)
```

参数说明：
- `<img src="...">`: 图片路径（支持 Typora 拖拽插入格式）
- `with transition`: 过渡效果（可选）

**支持的过渡效果**（仅限简单效果）：
- 无 `with` - 立即切换（无过渡）
- `dissolve` - 交叉溶解（默认时长）
- `Dissolve(duration: N)` 或 `Dissolve(N)` - 指定时长的交叉溶解

**不支持的效果**（请使用 `changeScene`）：
- ~~`fade`~~ / ~~`fadewhite`~~ → 迁移到 `changeScene with Fade(...)` / `FadeWhite(...)`
- 任何涉及遮罩的效果 → 使用 `changeScene`

> **迁移提示**：如果旧脚本使用 `changeBG ... with fade/fadewhite`，解析器会报错并提示迁移到 `changeScene with Fade(...)` / `FadeWhite(...)`。

### 5.2 场景切换 (changeScene)

> **职责**：复合场景切换，涉及 UI 隐藏/恢复、遮罩过渡、清除立绘等完整演出流程。

```markdown
changeScene <img src="bg.jpg" /> with Dissolve(duration: 1)
changeScene <img src="bg.jpg" /> with Fade(duration: 1)
changeScene <img src="bg.jpg" /> with FadeWhite(duration: 1)
changeScene <img src="bg.jpg" /> with <img src="rule.png" /> (duration: 1, reversed: true)
```

#### 5.2.1 设计意图

`changeScene` 是一个**复合场景切换**指令，用一行脚本表达完整的演出流程。与 `changeBG` 的区别：

| 特性 | changeBG | changeScene |
|------|----------|-------------|
| UI 隐藏/恢复 | ❌ | ✅ |
| 清除立绘 | ❌ | ✅ |
| 支持遮罩 | ❌ | ✅ |
| 复合流程 | ❌ | ✅ |

#### 5.2.2 目标语义（推荐实现）

1. 使用 `dissolve` 隐藏 UI（对话框/选择分支/章节标题等 UI 层）
2. 使用 `with` 指定的效果叠加遮罩（黑色/白色/rule 图片）
3. 清除所有立绘，替换背景为新指定背景
4. 使用同一效果隐去遮罩
5. 使用 `dissolve` 恢复 UI

> 说明：以上是**推荐的可观察语义**。为了保持 Runtime/Host 分离，这些步骤可以全部在 Host 内部完成。

#### 5.2.3 支持的效果

| 效果类型 | 语法 | 遮罩 | 说明 |
|---------|------|------|------|
| **Dissolve** | `with Dissolve(duration: N)` | 无遮罩 | UI隐藏 → 背景交叉溶解+清立绘 → UI恢复 |
| **Fade** | `with Fade(duration: N)` | 纯黑色 | UI隐藏 → 黑屏 → 换背景+清立绘 → 显现 → UI恢复 |
| **FadeWhite** | `with FadeWhite(duration: N)` | 纯白色 | UI隐藏 → 白屏 → 换背景+清立绘 → 显现 → UI恢复 |
| **Rule** | `with <img src="rule.png"/> (duration: N, reversed: bool)` | 图片遮罩 | UI隐藏 → rule过渡遮罩 → 换背景+清立绘 → rule反向 → UI恢复 |

#### 5.2.4 语法约束

- `changeScene` **必须**带 `with` 子句（没有 `with` 视为语法错误）
- 第一个 `<img src="..."/>` 是新背景路径（按"素材路径相对于脚本文件"的规则解析）

#### 5.2.5 参数说明

- `duration`: 过渡时长（秒），建议范围 \(0.1 \sim 3.0\)
- `reversed`: 是否反转遮罩方向（`true` 反向，`false` 正向，仅 Rule 效果）

#### 5.2.6 错误处理规则

- 缺少背景 `<img src="...">`：报错并带行号
- 缺少 `with`：报错并带行号
- `Fade` 的 `color` 不是 `black`/`white`：报错并带行号
- `duration` 不是数字 / `reversed` 不是布尔：报错并带行号
- rule 图片无法加载：Host 打印错误但不崩溃（与资源系统一致）

### 5.3 显示角色 (show)

```markdown
show <img src="path/to/sprite.png" /> as alias at position with transition
```

```markdown
show alias at position with transition
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

#### 5.3.1 运行时隐藏状态（引擎内部）

`show` 的公共入口只有一条，但引擎内部会维护最小隐藏状态以保证行为可预测：

1. alias 绑定表
   - `alias -> current_sprite_path`
   - 用于支持 `show alias at ...`（无 `<img>`）时复用当前差分
2. 槽位状态表
   - `alias -> { position, visible/fading_out }`
   - 用于判定“首次入场 / 差分切换 / 移动 / 复合变化”

硬约束：
- 同脚本输入 + 同初始状态 => 同输出行为（确定性）
- 隐藏状态只作为引擎实现细节，不改变脚本层单入口语义

#### 5.3.2 `show` 状态机决策

对同一 `alias` 执行 `show` 时，按如下状态机决策：

| 当前状态 | 输入变化 | 默认行为 |
|---|---|---|
| Absent | 首次 `show` | 创建角色并入场（可选 alpha 过渡） |
| Present | 仅差分变化（path 变化，position 不变） | 原地切换差分 |
| Present | 仅位置变化（path 不变，position 变化） | 默认瞬移；`with move/slide` 才做移动动画 |
| Present | 差分+位置同帧变化 | 默认 `diffThenMove`（先换差分，再做移动） |

#### 5.3.3 复合行为与 fallback（`diffThenMove`）

当同一条 `show` 同时改变差分和位置时：

1. 先应用新差分（更新 `current_sprite_path`）
2. 再应用移动动画（旧位置 -> 新位置）

fallback 规则：
- 若 `with` 不是 `move/slide`（例如 `dissolve`），仍执行移动阶段，按默认 move 时长降级
- 若缺省 `with`，仍执行移动阶段，使用默认 move 参数

该规则保证“复合变化”观感稳定，不因脚本未精确指定移动效果而退化为跳闪。

#### 5.3.4 `show` 的错误处理

- 使用 `show alias at ...` 时，若 alias 尚未绑定差分，解析/执行报错（需要先通过带 `<img>` 的 `show` 完成绑定）
- 位置参数非法时报错并带行号
- 过渡参数非法时报错并带行号

#### 5.3.5 可观测性要求（调试）

调试日志至少应能观测：
- 本次 `show` 命中的策略分支（spawn/diff_only/move_only/diff_then_move）
- 关键前后状态（old/new path、old/new position）
- fallback 触发原因（例如“非 move 效果降级为默认 move”）

### 5.4 隐藏角色 (hide)

```markdown
hide alias with transition
```

示例：
```markdown
hide royu with fade
```

#### 5.4.1 `hide` 状态语义

- `hide alias`：立即移除该角色槽位
- `hide alias with dissolve/fade`：进入 `fading_out`，淡出完成后移除
- `hide` 后该 alias 的可见状态被清理；后续若使用 `show alias at ...`，需先重新绑定差分

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
<audio src="../bgm/Signal.mp3"></audio> ♾️
```

参数说明：
- `<audio src="...">`: 音频文件路径
- `loop` 或 `♾️`：标识BGM，循环播放，没有该标识认为是SFX，play once

同一时间只能有一个BGM播放，播放下一个会自动停止前一个，BGM切换自带交叉淡化效果。SFX不做限制。

停止BGM：

```markdown
stopBGM
```

> 备注：音量/静音属于**玩家设置选项**，脚本层不提供音量/静音控制能力；制作时应尽量保证不同 BGM 的响度一致。

## UI 与立绘显式控制（阶段 24 新增）

### 对话框控制

```markdown
textBoxHide
textBoxShow
textBoxClear
```

| 指令 | 说明 |
|------|------|
| `textBoxHide` | 隐藏对话框（不影响背景/立绘） |
| `textBoxShow` | 显示对话框 |
| `textBoxClear` | 清理对话框内容（对话/选择分支） |

> **设计意图**：`changeScene` 不再隐式隐藏/恢复 UI，编剧通过这些命令显式控制对话框的可见性和内容。

### 清除所有角色立绘

```markdown
clearCharacters
```

一键清除场景中所有角色立绘。等效于对每个角色逐一执行 `hide`（但不带过渡动画）。

> **设计意图**：`changeScene` 不再隐式清除立绘，编剧可以选择在换场前用 `clearCharacters` 或逐个 `hide` 来控制立绘。

### 典型场景切换脚本示例

```markdown
textBoxHide
clearCharacters
changeScene <img src="new_bg.jpg" /> with Fade(duration: 1)
textBoxShow
："新的场景开始了。"
```

## 节奏控制

### 等待 (wait)

```markdown
wait 1.0
wait 0.5
```

参数说明：
- 第一个参数为等待时长（秒），必须为正数

语义约定：
- 等待结束后自动执行下一条指令（不需要玩家点击）
- 等待期间可被**点击打断**，打断后立即执行下一条
- **Skip 模式**下直接跳过（不实际等待），避免拖慢快进节奏
- **Auto 模式**下正常等待，到期自动推进

---

## 控制逻辑

### 跨文件脚本调度（阶段 0 新增）

```markdown
callScript [prologue](scripts/remake/ring/summer/prologue.md)
callScript [chapter1](scripts/remake/ring/summer/1-1.md)
returnFromScript
```

语义约定：

- `callScript [label](path)`：调用目标脚本，并将“当前脚本下一条指令”压入调用栈；`label` 仅作展示，不参与入口选择，调用统一从目标文件开头执行。
- `returnFromScript`：从当前脚本返回最近一次调用点；若调用栈为空则报运行时错误。
- 非入口脚本执行到文件末尾时，自动等价于 `returnFromScript`。
- 入口脚本执行到文件末尾时，运行结束并返回主界面。
- 路径按“相对当前脚本目录”解析，行为与 `changeBG/show` 的资源相对路径解析一致。

### fullRestart

```markdown
fullRestart
```

执行 `fullRestart` 后，Host 将：

1. 把 runtime 当前的 `persistent_variables` 合并写入 `saves/persistent.json`
2. 清空当前游戏会话（会话变量、调用栈、等待状态等）
3. 返回标题画面

**常见用法**：首通章节后持久化进度标记，再重启让玩家从另一分支继续：

```markdown
if $persistent.complete_summer != true
  set $persistent.complete_summer = true
  fullRestart
else
  goto **Winter**
endif
```

### 注释说明行

```markdown
> 说明：这是一行脚本说明注释
```

- 以 `>` 开头的行被视为注释说明文本，解析阶段会直接跳过，不产生 warning。

### 无条件跳转

```markdown
goto **label**
```

约束：

- `goto` 仅允许跳转到**当前脚本文件内**的标签。
- 暂不支持跨文件 `goto`（如 `goto **summer::start**`），请使用 `callScript` + `returnFromScript`。

### 变量设置

使用 `set` 指令设置脚本变量：

```markdown
set $变量名 = 值
```

**支持的值类型**：

| 类型 | 语法示例 | 说明 |
|------|----------|------|
| 字符串 | `"文本"` 或 `'文本'` | 用引号包围的文本 |
| 布尔值 | `true` / `false` | 逻辑真/假 |
| 整数 | `42` / `-1` | 整数数值 |
| 变量引用 | `$other_var` | 引用另一个变量的值 |

**示例**：

```markdown
set $player_name = "Alice"
set $has_key = true
set $score = 100
set $is_ready = $completed
```

**变量命名规则**：

- 变量名必须以 `$` 开头
- 普通变量名只能包含字母、数字和下划线，例如 `$my_var`
- 变量名区分大小写

**注意**：普通变量属于会话变量，随存档保存；游戏重启（`fullRestart`）后会被清空。

### 持久化变量（$persistent.key）

持久化变量通过 `$persistent.key` 命名空间访问，跨游戏会话保留（即使执行 `fullRestart` 也不清空）：

```markdown
set $persistent.complete_summer = true

if $persistent.complete_summer != true
  ...
endif
```

**规则**：

- 命名空间严格隔离：`$persistent.key` 只查持久变量，`$key` 只查会话变量，互不可见
- 持久变量以 bare key（去掉 `persistent.` 前缀）存储于 `saves/persistent.json`
- 启动游戏时自动加载，执行 `fullRestart` 时自动写入
- 读档恢复时，`persistent.json` 中的值覆盖存档中可能携带的旧值（以磁盘为权威）

### 条件分支

使用 `if/elseif/else/endif` 实现条件分支：

```markdown
if 条件表达式
  内容...
elseif 条件表达式
  内容...
else
  内容...
endif
```

**条件表达式语法**：

| 语法 | 说明 | 示例 |
|------|------|------|
| `$var == 值` | 相等比较 | `$name == "Alice"` |
| `$var != 值` | 不等比较 | `$role != "guest"` |
| `表达式 and 表达式` | 逻辑与 | `$a == true and $b == true` |
| `表达式 or 表达式` | 逻辑或 | `$x == 1 or $y == 2` |
| `not 表达式` | 逻辑非 | `not $is_locked` |
| `(表达式)` | 括号分组 | `($a == true) and ($b == false)` |

**完整示例**：

```markdown
set $player_role = "user"

if $player_role == "admin"
  ："欢迎回来，管理员。"
  show <img src="admin_badge.png" /> as badge at right
elseif $player_role == "user"
  ："欢迎回来，用户。"
else
  ："欢迎，访客。请先登录。"
endif

if $has_key == true and $door_unlocked == false
  ："你用钥匙打开了门。"
  set $door_unlocked = true
endif
```

**设计约束**：

- 条件分支必须以 `endif` 结束
- `elseif` 和 `else` 是可选的
- 条件表达式必须返回布尔值
- 不支持嵌套条件（如需复杂逻辑，请使用标签和 goto）

## 七、过渡效果语法

### 7.1 统一效果表达式（支持命名参数）

所有过渡效果使用统一的效果表达式语法，解析器不解释具体效果语义：

```
with <effect_expr>

effect_expr := identifier                                  // 无参效果
             | identifier(positional_args)                  // 位置参数
             | identifier(named_args)                       // 命名参数（不允许与位置参数混用）
             | <img src="mask.png" ... /> (named_args)      // rule-based effect（仅此形式）

positional_args := arg ("," arg)*
named_args      := named_arg ("," named_arg)*
named_arg       := identifier ":" arg
```

**示例**：
```markdown
with dissolve                    // 无参数
with Dissolve(1.5)               // 位置参数
with Dissolve(duration: 1.5)     // 命名参数（不允许与位置参数混用）
with Fade(duration: 1)           // Fade 黑屏过渡（仅 changeScene）
with FadeWhite(duration: 1)      // FadeWhite 白屏过渡（仅 changeScene）
with <img src="assets/rule_10.png" /> (duration: 1, reversed: true) // rule-based effect（仅 changeScene）
```

### 7.2 解析器产出结构

解析器将 `with` 子句解析为通用的 `Transition` 结构：

```rust
pub struct Transition {
    pub name: String,           // 效果名，如 "dissolve", "Dissolve", "Fade", "rule"
    pub args: Vec<(Option<String>, TransitionArg)>, // None=位置参数，Some(key)=命名参数
}

pub enum TransitionArg {
    Number(f64),                // 数字，如 1.5
    String(String),             // 字符串，如 "mask.png", "black", "white"
    Bool(bool),                 // 布尔值，如 true/false
}
```

**设计理由**：
- 解析器只负责**结构提取**，不需要知道有哪些效果
- 新增效果时，只需在 Runtime/Host 层添加处理逻辑
- 避免"效果数 × 操作数"的规则爆炸
- 命名参数的意义在于：允许乱序、允许只填写部分参数，其余使用默认值（由 Host/具体 effect 解释层决定）
- 同一次调用中**不允许混用**位置参数与命名参数（语法层保证，避免歧义）

### 7.3 内置效果参考（统一语义表）

以下是内置效果参考（解析器无需感知；具体支持情况由 Host/指令决定）。

**阶段 25 统一说明**：所有效果的解析逻辑由 `host/src/renderer/effects/` 模块统一处理，同名效果在不同目标上共享同一份解析与默认值。

| 效果名 | 语法 | 适用指令 | 默认时长 | 说明 |
|--------|------|---------|---------|------|
| dissolve | `dissolve` 或 `Dissolve(duration: N)` | changeBG, changeScene, show, hide | 0.3s | Alpha 交叉溶解 |
| fade | `fade` 或 `Fade(duration: N)` | changeScene: 黑屏遮罩；show/hide: 等价 dissolve | 0.5s（场景）/ 0.3s（立绘） | 上下文相关 |
| fadewhite | `FadeWhite(duration: N)` | **仅 changeScene** | 0.5s | 白屏遮罩过渡 |
| rule | `<img src="mask.png" /> (duration: N, reversed: bool)` | **仅 changeScene** | 0.5s | 图片遮罩过渡 |
| move | `move(duration: N)` 或 `slide(duration: N)` | **仅 show**（立绘位置变更） | 0.3s | 平滑位置移动 |
| none | `none` | 所有 | 0s | 无效果（瞬间切换） |

**语义约定**：
- `show alias at pos` 默认瞬移；只有 `with move/slide` 才产生平滑移动动画
- `with dissolve/fade` 不触发位置移动（仅影响 alpha）
- 未知效果名降级为 `dissolve`

### 7.4 Capability 映射附录（引擎内部）

从 2026-03 起，Host 将效果请求统一路由到扩展 capability 注册表（内建扩展优先，缺失/失败时走 capability 级回退路径）。

| 脚本语义（示例） | Transition 归一化 | capability_id | 默认来源 |
|------------------|-------------------|---------------|----------|
| `show ... with dissolve` | `EffectKind::Dissolve` | `effect.dissolve` | `builtin.effect.dissolve` |
| `hide ... with fade`（立绘上下文） | `EffectKind::Fade`（alpha 语义） | `effect.dissolve` | `builtin.effect.dissolve` |
| `changeScene ... with Fade(...)` | `EffectKind::Fade` | `effect.fade` | `builtin.effect.fade` |
| `changeScene ... with FadeWhite(...)` | `EffectKind::FadeWhite` | `effect.fade` | `builtin.effect.fade` |
| `changeScene ... with <img src="mask.png"/> (...)` | `EffectKind::Rule` | `effect.rule_mask` | `builtin.effect.rule_mask` |
| `show ... with move/slide` | `EffectKind::Move` | `effect.move` | `builtin.effect.move` |

诊断约定：
- 诊断日志必须至少包含 `capability_id` 与扩展来源（`extension_name`）。
- capability 缺失或执行失败时，不阻断主线，按统一回退映射降级到可执行 capability。

> **注意**：旧语法 `changeBG with fade/fadewhite` 已废弃，请使用 `changeScene with Fade(...)`/`FadeWhite(...)`。

#### 7.3.1 `Fade` / `FadeWhite` 效果（纯色遮罩）

`Fade` 和 `FadeWhite` 使用纯色遮罩实现场景切换，**仅在 `changeScene` 中可用**：

```markdown
changeScene <img src="bg.jpg" /> with Fade(duration: 1)       // 黑屏过渡
changeScene <img src="bg.jpg" /> with FadeWhite(duration: 1)  // 白屏过渡
```

解析器归一化为：
```
Transition { name: "Fade", args: [(Some("duration"), Number(1.0))] }
Transition { name: "FadeWhite", args: [(Some("duration"), Number(1.0))] }
```

#### 7.3.2 `rule` 效果（图片遮罩 / ImageDissolve）

```markdown
changeScene <img src="assets/bg2.jpg" /> with <img src="assets/rule_10.png" /> (duration: 1, reversed: true)
```

解析器应将该写法归一化为 `Transition { name: "rule", args: [(Some("mask"), String(mask)), (Some("duration"), Number(duration)), (Some("reversed"), Bool(reversed))] }`。

**ImageDissolve 原理**（参考 Ren'Py）：
- 遮罩图片必须是**灰度图**（或使用红色通道作为亮度值）
- 过渡过程中，根据像素亮度值控制溶解顺序：
  - 亮度 ≤ progress 的像素显示新内容
  - 亮度 > progress 的像素仍显示旧内容/遮罩
- `reversed: true` 时反转亮度判断（暗的先溶解）

**参数说明**：
| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `mask` | String | 必填 | 遮罩图片路径（相对于脚本文件目录） |
| `duration` | Number | 0.5 | 过渡时长（秒） |
| `reversed` | Bool | false | 是否反转溶解顺序 |

**路径解析**：`mask` 路径支持相对路径，自动相对于脚本文件所在目录解析。例如脚本在 `assets/scripts/main.md`，遮罩路径为 `../backgrounds/rule_10.png`，则解析为 `assets/backgrounds/rule_10.png`。

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
   - `changeBG`, `changeScene`, `show`, `hide`, `goto`, `callScript`, `returnFromScript`, `wait`
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

