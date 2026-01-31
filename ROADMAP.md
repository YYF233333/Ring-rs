# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划，遵循 PLAN.md 中的架构约束。

---

## 开发阶段总览

| 阶段 | 名称 | 目标 | 预计工作量 |
|------|------|------|-----------|
| Phase 0 | 基础设施 | 类型定义、项目结构 | 1 天 |
| Phase 1 | 脚本解析器 | 完整的脚本解析能力 | 2-3 天 |
| Phase 2 | Runtime 核心 | 执行引擎与状态管理 | 2-3 天 |
| Phase 3 | Host 集成 | Bevy 渲染与交互 | 3-4 天 |
| Phase 4 | 整合测试 | MVP 功能验证 | 1-2 天 |

---

## Phase 0: 基础设施

### 0.1 核心类型定义 (`vn-runtime/src/types.rs`)

**目标**：定义 Runtime 与 Host 之间通信的所有类型。

```
□ RuntimeInput - Host 向 Runtime 传递的输入
  ├─ Click              - 用户点击
  ├─ ChoiceSelected(usize) - 选择结果
  └─ Signal(SignalId)   - 外部信号

□ Command - Runtime 向 Host 发出的指令
  ├─ ShowBackground { path, transition }
  ├─ ShowCharacter { path, alias, position, transition }
  ├─ HideCharacter { alias, transition }
  ├─ ShowText { speaker, content }
  ├─ PresentChoices { choices: Vec<Choice> }
  ├─ PlayBgm { path, loop }
  ├─ StopBgm { fade_out }
  └─ ...

□ WaitingReason - Runtime 的等待状态
  ├─ None               - 不等待，继续执行
  ├─ WaitForClick       - 等待用户点击
  ├─ WaitForChoice      - 等待用户选择
  ├─ WaitForTime(Duration) - 等待指定时长
  └─ WaitForSignal(SignalId) - 等待外部信号

□ RuntimeState - 可序列化的运行时状态
  ├─ script_position    - 脚本执行位置
  ├─ variables          - 脚本变量
  └─ waiting            - 当前等待状态
```

**验收标准**：
- [ ] 所有类型实现 `Clone`, `Debug`
- [ ] 可序列化类型实现 `Serialize`, `Deserialize`
- [ ] 单元测试覆盖序列化/反序列化

---

### 0.2 项目结构搭建

```
vn-runtime/
├── Cargo.toml
└── src/
    ├── lib.rs           # 模块导出
    ├── types.rs         # 核心类型定义
    ├── command.rs       # Command 枚举
    ├── input.rs         # RuntimeInput 定义
    ├── state.rs         # RuntimeState 定义
    ├── script/          # 脚本相关
    │   ├── mod.rs
    │   ├── ast.rs       # 脚本 AST 定义
    │   ├── parser.rs    # 脚本解析器
    │   └── lexer.rs     # 词法分析（可选）
    └── runtime/         # 执行引擎
        ├── mod.rs
        ├── engine.rs    # 核心执行逻辑
        └── executor.rs  # 指令执行器

host/
├── Cargo.toml
└── src/
    ├── main.rs          # 入口
    ├── plugin.rs        # Bevy VN Plugin
    ├── render/          # 渲染相关
    │   ├── mod.rs
    │   ├── background.rs
    │   ├── character.rs
    │   └── text.rs
    ├── audio/           # 音频相关
    │   └── mod.rs
    └── input/           # 输入处理
        └── mod.rs
```

**验收标准**：
- [ ] `cargo build` 成功
- [ ] `cargo test` 通过（空测试）
- [ ] 模块结构清晰，符合 PLAN.md 约束

---

## Phase 1: 脚本解析器

### 1.1 AST 定义 (`vn-runtime/src/script/ast.rs`)

**目标**：定义脚本的抽象语法树。

```rust
// 脚本节点类型
enum ScriptNode {
    Chapter(String),                    // 章节
    Label(String),                      // 标签
    Dialogue { speaker: Option<String>, content: String },
    ChangeBG { path: String, transition: Option<Transition> },
    ChangeScene { path: String, transition: Option<Transition>, rule: Option<RuleConfig> },
    ShowCharacter { path: String, alias: String, position: Position, transition: Option<Transition> },
    HideCharacter { alias: String, transition: Option<Transition> },
    Choice { options: Vec<ChoiceOption> },
    UIAnim { effect: String },
}
```

**验收标准**：
- [ ] AST 能表达 showcase 中的所有语法元素
- [ ] 类型实现 `Debug`, `Clone`, `PartialEq`

---

### 1.2 解析器实现 (`vn-runtime/src/script/parser.rs`)

**目标**：将 `.md` 脚本文件解析为 AST。

**实现要点**：
1. 逐行解析，识别行类型
2. 提取 HTML img 标签中的 src 属性
3. 解析 Markdown 表格为选择分支
4. 容错处理（空格、缩进、标点）

**验收标准**：
- [ ] 能正确解析 `docs/script_language_showcase.md`
- [ ] 单元测试覆盖所有语法元素
- [ ] 错误信息包含行号

---

### 1.3 解析器测试

```rust
#[test]
fn test_parse_dialogue() { ... }

#[test]
fn test_parse_show_character() { ... }

#[test]
fn test_parse_choice_table() { ... }

#[test]
fn test_parse_full_showcase() { ... }
```

---

## Phase 2: Runtime 核心

### 2.1 执行引擎 (`vn-runtime/src/runtime/engine.rs`)

**目标**：实现核心 tick 循环。

```rust
impl VNRuntime {
    /// 创建新的 Runtime 实例
    pub fn new(script: Script) -> Self;
    
    /// 核心驱动函数
    /// 返回 (commands, waiting_reason)
    pub fn tick(&mut self, input: RuntimeInput) -> (Vec<Command>, WaitingReason);
    
    /// 获取当前状态（用于序列化）
    pub fn state(&self) -> &RuntimeState;
    
    /// 从状态恢复（用于读档）
    pub fn restore(script: Script, state: RuntimeState) -> Self;
}
```

**执行逻辑**：
1. 检查当前 `WaitingReason`
2. 根据 `input` 决定是否解除等待
3. 若不再等待，继续执行脚本直到下一个阻塞点
4. 收集执行过程中产生的 `Command`
5. 返回 `(commands, new_waiting_reason)`

**验收标准**：
- [ ] 能执行简单脚本并产出正确 Command
- [ ] WaitForClick 正确阻塞和恢复
- [ ] WaitForChoice 正确处理选择
- [ ] 状态可序列化/反序列化

---

### 2.2 指令执行器 (`vn-runtime/src/runtime/executor.rs`)

**目标**：将 AST 节点转换为 Command。

```rust
impl Executor {
    fn execute_node(&mut self, node: &ScriptNode) -> ExecuteResult {
        match node {
            ScriptNode::Dialogue { speaker, content } => {
                self.commands.push(Command::ShowText { speaker, content });
                ExecuteResult::Wait(WaitingReason::WaitForClick)
            }
            ScriptNode::ChangeBG { path, transition } => {
                self.commands.push(Command::ShowBackground { path, transition });
                ExecuteResult::Continue
            }
            // ...
        }
    }
}
```

---

### 2.3 Runtime 测试

```rust
#[test]
fn test_simple_dialogue_flow() { ... }

#[test]
fn test_choice_branching() { ... }

#[test]
fn test_state_serialization() { ... }
```

---

## Phase 3: Host 集成

### 3.1 Bevy Plugin 架构 (`host/src/plugin.rs`)

**目标**：创建 Bevy 插件封装 VN 功能。

```rust
pub struct VNPlugin;

impl Plugin for VNPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            process_commands,
            handle_input,
            update_text_display,
        ));
    }
}
```

---

### 3.2 背景渲染 (`host/src/render/background.rs`)

**目标**：处理 `ShowBackground` Command。

**实现要点**：
- 加载图片资源
- 应用过渡效果
- 管理背景 Entity

---

### 3.3 角色立绘渲染 (`host/src/render/character.rs`)

**目标**：处理 `ShowCharacter` / `HideCharacter` Command。

**实现要点**：
- 根据 position 计算屏幕坐标
- 管理多个角色 Entity（通过 alias）
- 过渡动画

---

### 3.4 文本显示 (`host/src/render/text.rs`)

**目标**：处理 `ShowText` / `PresentChoices` Command。

**实现要点**：
- 文本框 UI
- 角色名显示
- 选择按钮生成

---

### 3.5 输入处理 (`host/src/input/mod.rs`)

**目标**：收集用户输入并转换为 `RuntimeInput`。

**实现要点**：
- 点击检测
- 选择按钮点击
- 将输入传递给 Runtime

---

## Phase 4: 整合测试

### 4.1 端到端测试脚本

创建 `assets/test_script.md`：

```markdown
# Test Script

changeBG <img src="assets/test_bg.png" /> with dissolve

测试角色："这是一段测试对话。"

show <img src="assets/test_sprite.png" /> as test at center with dissolve

："这是旁白文本。"

**choice_test**

| 选择 |        |
| ---- | ------ |
| 选项A | opt_a |
| 选项B | opt_b |

**opt_a**
测试角色："你选择了 A。"

**opt_b**
测试角色："你选择了 B。"

hide test with fade
```

---

### 4.2 验收清单

**MVP 功能验收**：
- [ ] 显示背景图片
- [ ] 显示角色立绘（支持位置）
- [ ] 显示文本（角色名 + 对话）
- [ ] 点击继续
- [ ] 分支选择（跳转正确）

**技术验收**：
- [ ] Runtime 无 Bevy 依赖
- [ ] 所有状态可序列化
- [ ] 单元测试覆盖率 > 80%
- [ ] 无 clippy 警告

---

## 依赖管理

### vn-runtime/Cargo.toml

```toml
[package]
name = "vn-runtime"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
regex = "1"           # 用于解析 HTML img 标签
thiserror = "2"       # 错误处理

[dev-dependencies]
serde_json = "1"      # 测试序列化
```

### host/Cargo.toml

```toml
[package]
name = "host"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "vn-game"
path = "src/main.rs"

[dependencies]
bevy = "0.18"
vn-runtime = { path = "../vn-runtime" }
```

---

## 风险与备选方案

### 风险 1: Bevy 0.18 API 变动

**缓解措施**：参考 https://docs.rs/bevy/0.18.0/bevy/ 官方文档

### 风险 2: 脚本解析复杂度

**缓解措施**：
- 采用逐行解析而非完整 parser generator
- 对无法解析的行记录警告但不中断

### 风险 3: 过渡动画实现复杂

**缓解措施**：
- MVP 阶段仅实现简单 fade 效果
- 复杂过渡效果作为后续迭代

---

## 执行顺序

```
Week 1:
├── Day 1-2: Phase 0 (类型定义 + 项目结构)
├── Day 3-5: Phase 1 (脚本解析器)
└── Day 6-7: Phase 1 测试 + 文档

Week 2:
├── Day 1-3: Phase 2 (Runtime 核心)
├── Day 4-6: Phase 3 (Host 集成)
└── Day 7: Phase 4 (整合测试)
```

---

## 进度追踪

| 任务 | 状态 | 完成日期 |
|------|------|----------|
| Phase 0.1 核心类型定义 | ⏳ 待开始 | - |
| Phase 0.2 项目结构搭建 | ⏳ 待开始 | - |
| Phase 1.1 AST 定义 | ⏳ 待开始 | - |
| Phase 1.2 解析器实现 | ⏳ 待开始 | - |
| Phase 1.3 解析器测试 | ⏳ 待开始 | - |
| Phase 2.1 执行引擎 | ⏳ 待开始 | - |
| Phase 2.2 指令执行器 | ⏳ 待开始 | - |
| Phase 2.3 Runtime 测试 | ⏳ 待开始 | - |
| Phase 3.1 Bevy Plugin | ⏳ 待开始 | - |
| Phase 3.2 背景渲染 | ⏳ 待开始 | - |
| Phase 3.3 角色立绘渲染 | ⏳ 待开始 | - |
| Phase 3.4 文本显示 | ⏳ 待开始 | - |
| Phase 3.5 输入处理 | ⏳ 待开始 | - |
| Phase 4.1 端到端测试 | ⏳ 待开始 | - |
| Phase 4.2 MVP 验收 | ⏳ 待开始 | - |

---

> **文档维护说明**：
> 完成每个任务后，更新对应状态和完成日期。
> 如遇设计变更，需同步更新 PLAN.md。

