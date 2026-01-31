
---

# Visual Novel Engine 项目规划与架构约束

> 本文档是本项目的**最高优先级设计约束**。
> 所有代码、测试、文档、重构、修复，**必须严格遵循本文档**。
> 若实现与本文档冲突，必须以本文档为准。

---

## 一、项目目标（Project Goal）

本项目旨在**在最小人类干预下**，使用大模型完成一个 **可运行、结构清晰、可扩展的 Visual Novel Engine**


---

## 二、总体架构原则（Architecture Principles）

### 1. Runtime 与 Host 分离

* VN Runtime 是**纯逻辑核心**
* Host（Bevy）仅作为 IO / 渲染 / 音频宿主
* Runtime **不得依赖 Bevy 或任何引擎 API**

### 2. 显式状态、确定性执行

* 所有运行状态必须显式建模
* 不允许隐式全局状态
* 不允许依赖真实时间推进逻辑

### 3. 命令驱动（Command-based）

* Runtime **只产出 Command**
* Host **只执行 Command**
* Runtime 不直接渲染、不播放音频、不等待输入

---

## 三、模块划分（High-Level Modules）

### 1. `vn-runtime`（核心引擎）

职责：

* 脚本执行
* 状态管理
* 等待 / 阻塞建模
* Command 生成

限制：

* 不允许 IO
* 不允许线程 / async runtime
* 不允许使用 Bevy 类型

---

### 2. `host`（宿主层）

职责：

* 窗口与渲染
* 资源加载
* 音频播放
* 输入采集
* 将 Runtime 的 Command 转换为实际效果

限制：

* 不允许包含脚本逻辑
* 不允许直接修改 Runtime 内部状态

---

## 四、VN Runtime 核心模型

### 1. RuntimeState（唯一可变状态）

```text
- Script 执行位置
- 脚本变量
- 当前等待状态
```

所有状态必须可序列化。

---

### 2. WaitingReason（显式等待模型）

```text
None
WaitForClick
WaitForChoice
WaitForTime(Duration)
WaitForSignal(SignalId)
```

禁止使用隐式 await / sleep。

---

### 3. Runtime 执行模型

* Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动
* 若处于等待状态，仅处理输入
* 若不等待，持续推进脚本直到再次阻塞
* 返回的 `WaitingReason` 告知 Host 当前等待状态

---

### 4. RuntimeInput（Host → Runtime 输入模型）

```text
Click                   - 用户点击（解除 WaitForClick）
ChoiceSelected(index)   - 用户选择了第 index 个选项（解除 WaitForChoice）
Signal(signal_id)       - 外部信号（解除 WaitForSignal）
```

说明：
* Host 负责采集用户输入并转换为 `RuntimeInput`
* `WaitForTime` 由 Host 处理：Host 获取该等待状态后，等待指定时长再调用 tick
* Runtime 不需要知道真实时间流逝，保持确定性执行

---

## 五、Command 模型（Runtime → Host）

Command 是 Runtime 与 Host 的**唯一通信方式**。

示例（非完整）：

```text
ShowBackground
ShowCharacter
HideCharacter
ShowText
PlayBgm
StopBgm
PresentChoices
```

要求：

* 声明式
* 不包含引擎相关类型
* 不产生副作用

---

## 六、关于 async / await 的特别约束

### 允许：

* 在 Runtime **内部实现中**使用 async 语法
* async 仅作为“脚本执行器”的语法糖
* async 状态必须可映射为显式 WaitingReason

### 禁止：

* Tokio / async-std 等运行时
* 多线程 / task spawn
* sleep / timer
* async 作为对外 API 语义

---

## 七、技术选型

### 语言

* rust 1.95.0-nightly

### 宿主引擎

* Bevy 0.18.0（仅作为 Host 层）

> bevy处于早期开发版，API变动较快，使用https://docs.rs/bevy/0.18.0/bevy/ 访问当前版本的文档。

### 开发准则

- 你（大模型）是这个项目的lead engineer，请按照既定开发计划工作，并向用户汇报设计理由、是否有备选方案，如果有为什么选择当前方案，解释你做的tradeoff。

- Test-Driven Development，开发时应当同步更新测试，确保代码可以被测试覆盖，使用test向我证明实现的正确性。在修复bug后应当补充回归测试。

- 不要遗漏文档，核心功能应当有详尽的文档和使用方法。

---

## 八、项目结构

分两个crate实现完整项目，vn-runtime crate为核心运行时，不依赖bevy，host crate使用bevy处理和外界的交互，并作为最终二进制入口。两个crate间使用command通信。

## 九、质量与可维护性要求

* 所有核心逻辑必须有单元测试
* 所有 public API 必须有文档注释
* 修复 bug 时必须补充回归测试
* 禁止“顺便重构无关代码”

---

## 十、实现态度要求（对模型）

* 优先清晰性而非抽象性
* 优先可读性而非技巧性
* 若存在多种方案，选择**最简单且符合约束的方案**
* 若不确定，必须在文档或注释中说明权衡

---

## 十一、最终目标（MVP）

一个可运行的程序，能够：

1. 显示背景
2. 显示角色立绘
3. 显示文本
4. 等待点击
5. 提供简单分支选择

## 十二、补充材料

- `docs/script_language_showcase.md` 展示了引擎脚本语言的实际使用示例（人工编写）
- `docs/script_syntax_spec.md` 定义了脚本语言的正式语法规范
- `ROADMAP.md` 定义了具体的开发执行计划

---

> **再次强调：**
> 若实现与本文档冲突，必须以本文档为准。

---
