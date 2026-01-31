# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划，遵循 PLAN.md 中的架构约束。

---

## 开发阶段总览

| 阶段 | 名称 | 目标 | 状态 |
|------|------|------|------|
| Phase 0 | 基础设施 | 类型定义、项目结构 | ✅ 已完成 |
| Phase 1 | 脚本解析器 | 完整的脚本解析能力 | ✅ 已完成 |
| Phase 2 | Runtime 核心 | 执行引擎与状态管理 | ✅ 已完成 |
| Phase 3 | Host 集成 | Bevy 渲染与交互 | ✅ 已完成 |
| Phase 4 | 整合测试 | MVP 功能验证 | ✅ 已完成 |

---

## Phase 0: 基础设施 ✅

### 0.1 核心类型定义

**目标**：定义 Runtime 与 Host 之间通信的所有类型。

**已实现**：

```
✅ RuntimeInput - Host 向 Runtime 传递的输入
  ├─ Click              - 用户点击
  ├─ ChoiceSelected(usize) - 选择结果
  └─ Signal(SignalId)   - 外部信号

✅ Command - Runtime 向 Host 发出的指令
  ├─ ShowBackground { path, transition }
  ├─ ShowCharacter { path, alias, position, transition }
  ├─ HideCharacter { alias, transition }
  ├─ ShowText { speaker, content }
  ├─ PresentChoices { style, choices }
  ├─ PlayBgm { path, looping }
  ├─ StopBgm { fade_out }
  ├─ PlaySfx { path }
  ├─ UIAnimation { effect }
  ├─ ChapterMark { title, level }
  └─ ChangeScene { path, transition }

✅ WaitingReason - Runtime 的等待状态
  ├─ None               - 不等待，继续执行
  ├─ WaitForClick       - 等待用户点击
  ├─ WaitForChoice { choice_count } - 等待用户选择
  ├─ WaitForTime(Duration) - 等待指定时长
  └─ WaitForSignal(SignalId) - 等待外部信号

✅ RuntimeState - 可序列化的运行时状态
  ├─ script_name        - 脚本名称
  ├─ script_position    - 脚本执行位置
  ├─ variables          - 脚本变量
  └─ waiting            - 当前等待状态
```

**验收标准**：
- [x] 所有类型实现 `Clone`, `Debug`
- [x] 可序列化类型实现 `Serialize`, `Deserialize`
- [x] 单元测试覆盖序列化/反序列化

---

### 0.2 项目结构搭建 ✅

**最终项目结构**：

```
vn-runtime/
├── Cargo.toml
└── src/
    ├── lib.rs           # 模块导出与公共 API
    ├── command.rs       # Command 枚举与相关类型
    ├── input.rs         # RuntimeInput 定义
    ├── state.rs         # RuntimeState 与 WaitingReason
    ├── error.rs         # 错误类型 (ParseError, RuntimeError)
    ├── script/          # 脚本相关
    │   ├── mod.rs
    │   ├── ast.rs       # 脚本 AST 定义
    │   └── parser.rs    # 手写递归下降解析器
    └── runtime/         # 执行引擎
        ├── mod.rs
        ├── engine.rs    # VNRuntime 核心
        └── executor.rs  # 指令执行器

host/
├── Cargo.toml
├── assets/
│   └── scripts/
│       └── demo.md      # 示例脚本
└── src/
    ├── main.rs          # 入口 + 脚本加载
    ├── lib.rs           # 库入口
    ├── plugin.rs        # Bevy VNPlugin
    ├── components.rs    # ECS 组件定义
    ├── resources.rs     # 资源与 Message 定义
    └── systems/         # Bevy 系统
        ├── mod.rs
        ├── setup.rs     # 初始化系统
        ├── input.rs     # 输入处理
        ├── runtime.rs   # Runtime 驱动
        ├── commands.rs  # Command 执行
        └── ui.rs        # UI 更新
```

**验收标准**：
- [x] `cargo build` 成功
- [x] `cargo test` 通过（50 个测试）
- [x] 模块结构清晰，符合 PLAN.md 约束

---

## Phase 1: 脚本解析器 ✅

### 1.1 AST 定义 ✅

**目标**：定义脚本的抽象语法树。

**已实现**（`vn-runtime/src/script/ast.rs`）：

```rust
pub enum ScriptNode {
    Chapter { title: String, level: u8 },
    Label(String),
    Dialogue { speaker: Option<String>, content: String },
    ChangeBG { path: String, transition: Option<Transition> },
    ChangeScene { path: String, transition: Option<Transition> },
    ShowCharacter { path: String, alias: String, position: Position, transition: Option<Transition> },
    HideCharacter { alias: String, transition: Option<Transition> },
    Choice { style: Option<String>, options: Vec<ChoiceOption> },
    UIAnim { effect: String, args: Option<String> },
    PlayBgm { path: String, looping: bool },
    StopBgm { fade_out: bool },
    PlaySfx { path: String },
}
```

**验收标准**：
- [x] AST 能表达 showcase 中的所有语法元素
- [x] 类型实现 `Debug`, `Clone`, `PartialEq`

---

### 1.2 解析器实现 ✅

**目标**：将 `.md` 脚本文件解析为 AST。

**实现方案**：手写递归下降解析器（无 regex 依赖）

**技术决策**：
- ❌ 最初使用 regex 实现
- ✅ 重构为手写解析器，提高可维护性和健壮性

**解析器特性**：
- 两阶段解析：块识别 → 块内容解析
- 支持中英文标点（冒号 `:` `：`，引号 `"` `""`）
- 容错处理空格、缩进变化
- 正确解析 HTML `<img>` 标签
- 正确解析 Markdown 表格

**验收标准**：
- [x] 能正确解析示例脚本
- [x] 单元测试覆盖所有语法元素
- [x] 错误信息包含行号

---

### 1.3 解析器测试 ✅

**测试覆盖**（50 个测试用例）：

```rust
✅ test_parse_dialogue            // 基本对话
✅ test_parse_narration           // 旁白
✅ test_parse_show_character      // 角色显示
✅ test_parse_hide_character      // 角色隐藏
✅ test_parse_change_bg           // 背景切换
✅ test_parse_choice_table        // 选择分支
✅ test_parse_chapter             // 章节标记
✅ test_parse_label               // 标签
✅ test_parse_uianim              // UI 动画
✅ test_parse_transition_effects  // 过渡效果
✅ test_whitespace_tolerance      // 空格容错
✅ test_chinese_punctuation       // 中文标点
✅ test_realistic_script          // 完整脚本
// ... 更多测试
```

---

## Phase 2: Runtime 核心 ✅

### 2.1 执行引擎 ✅

**目标**：实现核心 tick 循环。

**已实现**（`vn-runtime/src/runtime/engine.rs`）：

```rust
impl VNRuntime {
    pub fn new(script: Script) -> Self;
    pub fn tick(&mut self, input: Option<RuntimeInput>) -> VnResult<(Vec<Command>, WaitingReason)>;
    pub fn state(&self) -> &RuntimeState;
    pub fn restore(script: Script, state: RuntimeState) -> Self;
    pub fn jump_to_label(&mut self, label: &str) -> VnResult<()>;
}
```

**执行逻辑**：
1. 检查当前 `WaitingReason`
2. 根据 `input` 决定是否解除等待
3. 若不再等待，继续执行脚本直到下一个阻塞点
4. 收集执行过程中产生的 `Command`
5. 返回 `(commands, new_waiting_reason)`

**验收标准**：
- [x] 能执行简单脚本并产出正确 Command
- [x] WaitForClick 正确阻塞和恢复
- [x] WaitForChoice 正确处理选择
- [x] 状态可序列化/反序列化

---

### 2.2 指令执行器 ✅

**目标**：将 AST 节点转换为 Command。

**已实现**（`vn-runtime/src/runtime/executor.rs`）：

- 对话/旁白 → `ShowText` + `WaitForClick`
- 背景切换 → `ShowBackground`
- 角色显示 → `ShowCharacter`
- 角色隐藏 → `HideCharacter`
- 选择分支 → `PresentChoices` + `WaitForChoice`
- 标签跳转 → 内部位置更新

---

### 2.3 Runtime 测试 ✅

**测试覆盖**：
- 简单对话流程
- 选择分支跳转
- 状态序列化/反序列化
- 标签跳转
- 错误处理

---

## Phase 3: Host 集成 ✅

### 3.1 Bevy Plugin 架构 ✅

**已实现**（`host/src/plugin.rs`）：

```rust
pub struct VNPlugin;

impl Plugin for VNPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<VNState>()
            .init_resource::<DialogueState>()
            .add_message::<PlayerInput>()
            .add_message::<VNCommand>()
            .add_systems(Startup, setup_system)
            .add_systems(Update, (
                input_system,
                tick_runtime_system,
                execute_commands_system,
                update_dialogue_system,
                update_characters_system,
            ));
    }
}
```

**Bevy 0.18 适配**：
- 使用 `Message` 替代 `Event`
- 使用 `MessageReader`/`MessageWriter` 替代 `EventReader`/`EventWriter`
- 适配新的查询 API

---

### 3.2 背景渲染 ✅

**已实现**：
- `ShowBackground` Command 处理
- 图片资源加载
- 背景 Entity 管理

---

### 3.3 角色立绘渲染 ✅

**已实现**：
- `ShowCharacter` / `HideCharacter` Command 处理
- 支持 9 个位置：Left, Right, Center, NearLeft, NearRight, NearMiddle, FarLeft, FarRight, FarMiddle
- 通过 alias 管理多个角色

---

### 3.4 文本显示 ✅

**已实现**：
- 文本框 UI
- 角色名显示
- 对话内容显示
- 选择按钮生成

---

### 3.5 输入处理 ✅

**已实现**：
- 鼠标点击检测
- 空格键检测
- 选择按钮点击
- 输入转换为 `RuntimeInput`

---

## Phase 4: 整合测试 ✅

### 4.1 端到端测试脚本 ✅

**创建示例脚本**（`host/assets/scripts/demo.md`）：

```markdown
# 第一章：相遇

changeBG <img src="backgrounds/classroom.png">

羽艾："你好，我是羽艾。"

show <img src="characters/yuai_normal.png"> as yuai at center

羽艾："很高兴认识你！"

："（这是一段旁白文字）"

羽艾："你想去哪里呢？"

| 选择 |  |
| --- | --- |
| 去图书馆 | library |
| 去操场 | playground |
```

---

### 4.2 验收清单 ✅

**MVP 功能验收**：
- [x] 显示背景图片
- [x] 显示角色立绘（支持位置）
- [x] 显示文本（角色名 + 对话）
- [x] 点击继续
- [x] 分支选择（跳转正确）

**技术验收**：
- [x] Runtime 无 Bevy 依赖
- [x] 所有状态可序列化
- [x] 单元测试覆盖率 > 80%（50 个测试）
- [x] cargo check 通过（3 个未使用字段警告，为预留功能）

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
thiserror = "2"

[dev-dependencies]
serde_json = "1"
```

> **注意**：解析器为手写实现，不使用 regex 依赖

### host/Cargo.toml

```toml
[package]
name = "host"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.18"
vn-runtime = { path = "../vn-runtime" }
```

---

## 进度追踪

| 任务 | 状态 | 完成日期 |
|------|------|----------|
| Phase 0.1 核心类型定义 | ✅ 已完成 | 2026-01-31 |
| Phase 0.2 项目结构搭建 | ✅ 已完成 | 2026-01-31 |
| Phase 1.1 AST 定义 | ✅ 已完成 | 2026-01-31 |
| Phase 1.2 解析器实现 | ✅ 已完成 | 2026-01-31 |
| Phase 1.3 解析器测试 | ✅ 已完成 | 2026-01-31 |
| Phase 2.1 执行引擎 | ✅ 已完成 | 2026-01-31 |
| Phase 2.2 指令执行器 | ✅ 已完成 | 2026-01-31 |
| Phase 2.3 Runtime 测试 | ✅ 已完成 | 2026-01-31 |
| Phase 3.1 Bevy Plugin | ✅ 已完成 | 2026-01-31 |
| Phase 3.2 背景渲染 | ✅ 已完成 | 2026-01-31 |
| Phase 3.3 角色立绘渲染 | ✅ 已完成 | 2026-01-31 |
| Phase 3.4 文本显示 | ✅ 已完成 | 2026-01-31 |
| Phase 3.5 输入处理 | ✅ 已完成 | 2026-01-31 |
| Phase 4.1 端到端测试 | ✅ 已完成 | 2026-01-31 |
| Phase 4.2 MVP 验收 | ✅ 已完成 | 2026-01-31 |

---

## 后续迭代计划

### 优先级 1（核心功能增强）

- [ ] 音频系统实现（BGM / SFX）
- [ ] 存档/读档功能
- [ ] 过渡动画效果（dissolve, fade 等）

### 优先级 2（体验优化）

- [ ] 文本打字机效果
- [ ] UI 样式美化
- [ ] 历史记录回看

### 优先级 3（扩展功能）

- [ ] 条件分支（if/else）
- [ ] 变量系统
- [ ] 脚本热重载

---

> **MVP 完成总结**：
>
> 项目于 2026-01-31 完成所有 MVP 阶段。
> - 50 个单元测试全部通过
> - 窗口可正常启动并加载脚本
> - 架构符合 PLAN.md 约束（Runtime/Host 分离、命令驱动）

