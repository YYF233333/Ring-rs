# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划。

## 项目当前状态

### ✅ 已完成模块

1. **vn-runtime 核心运行时**
   - ✅ 脚本解析器（Parser）：覆盖当前已实现语法，50+ 测试用例
   - ✅ AST 定义：完整的脚本节点类型
   - ✅ Command 定义：Runtime → Host 通信协议
   - ✅ RuntimeInput 定义：Host → Runtime 输入模型
   - ✅ WaitingReason 定义：显式等待状态模型
   - ✅ RuntimeState 定义：可序列化的运行时状态
   - ✅ Engine（VNRuntime）：核心执行引擎
   - ✅ Executor：AST 节点到 Command 的转换
   - ✅ 错误处理：完整的错误类型和错误信息

2. **host 适配层（macroquad）**
   - ✅ 窗口与主循环
   - ✅ 资源管理系统（PNG/JPEG 支持）
   - ✅ 渲染系统（背景/角色/对话框/选择分支/章节标题）
   - ✅ 输入处理（键盘/鼠标，防抖）
   - ✅ Command 执行器
   - ✅ Runtime 集成
   - ✅ 过渡效果实现（dissolve/fade/fadewhite/rule(ImageDissolve)）
   - ✅ 音频系统（rodio，支持 MP3/WAV/FLAC/OGG）

---

## 开发历程总结（浓缩版）

> 目标：避免把 ROADMAP 写成"开发日志"。这里仅保留里程碑结论，细节进入对应阶段归档。

### 阶段 1-22：开发成果总结（约 50 行，可扫描）

#### 1) 架构与执行模型（Runtime/Host 分离 + 确定性）
- Runtime/Host 分离落地：`vn-runtime` 纯逻辑；`host` 做渲染/音频/输入/资源
- Runtime 只产出 `Command`，Host 只执行 `Command`（通信契约清晰）
- 显式状态与等待：`RuntimeState` 可序列化；`WaitingReason` 显式建模
- 执行推进：`tick(input) -> (commands, waiting)`，不依赖真实时间（时间由 Host 等待）

#### 2) 脚本语言与语义（面向编剧 + 可测试/可存档）
- 基础结构：章节标题、标签、对话/旁白、选择分支表格（Markdown 友好）
- 演出指令：`changeBG`/`changeScene`/`show`/`hide`，路径相对脚本目录解析
- 音频：BGM/SFX（HTML audio）+ `stopBGM`；BGM 切换交叉淡化
- 控制流：`goto`（标签跳转）
- 逻辑系统（阶段22）：变量（`set $var = <expr>`）+ 条件（`if/elseif/else/endif`）
  - 表达式：`==/!=`、`and/or/not`、括号；确定性求值；变量随存档持久化

#### 3) 演出/过渡/动画（观感与可扩展）
- `changeBG`（简单切换）vs `changeScene`（复合演出）职责分离，语义更可控
- 过渡效果统一表达式：支持命名参数；Fade/FadeWhite/Rule(ImageDissolve)/Dissolve
- 动画体系重构：Trait 驱动时间轴，类型安全属性键；覆盖角色/背景/场景过渡

#### 4) 资源系统与发布（规模化）
- 资源动态加载：按需加载 + LRU 驱逐 + 显存预算控制；Debug Overlay 可观测
- 资源来源抽象：文件系统 / ZIP 包统一口径；资源打包工具支持发布
- 路径治理：统一 `std::path` 规范化，修复 `../` 导致的资源键不一致

#### 5) 玩家体验与系统（可玩性闭环）
- UI 页面：Title/菜单/存读档/设置/历史；AppMode + 导航栈驱动
- 存档体系：Continue 专用存档 + 1-99 槽位与元信息；读档可恢复状态与历史
- 配置落地：`config.json` 驱动入口脚本与资源根目录等

#### 6) 工程化与质量（可维护、可回归）
- 本地门禁：`cargo check-all`（fmt/clippy/test 串行）稳定可复现
- host 结构治理：入口瘦身，`app/*` + `update/*` 模块化拆分降低耦合
- 覆盖率口径：`cargo cov-runtime`/`cargo cov-workspace`（主看 `vn-runtime`）

#### 关键指针（深入细节）
- 规范：`docs/script_syntax_spec.md`；导航：`docs/navigation_map.md`
- Runtime：`vn-runtime/src/{script/*,runtime/*,state.rs,save.rs,history.rs}`
- Host：`host/src/{app/*,resources/*,renderer/*,command_executor/*}`

### 阶段 23：开发者工作流与内容校验（lint/资源引用检查/诊断）✅ 已完成

> **主题**：仓库已经进入"功能迭代期"，下一阶段把编剧/制作过程里的问题前置：脚本/资源/manifest 的错误尽量在运行前就被发现。

**已实现（浓缩）**：
- ✅ **静态脚本诊断**：未定义 label / choice 目标缺失 / 语法错误（复用 parser 错误）  
- ✅ **资源引用检查**：`<img src>` / `<audio src>` 统一逻辑路径解析 + 存在性校验（与 `ResourceManager` 同口径）
- ✅ **诊断体验**：统一格式（文件/脚本ID/行号/详情）+ 分级（Error/Warn/Info）
- ✅ **工具链落地**：`cargo script-check [path]`（xtask），支持只读扫描；Dev Mode 可自动诊断

**关键入口**：`vn-runtime/src/diagnostic.rs`、`tools/xtask/src/main.rs`、`host/src/app/init.rs`

### 仓库瘦身与上下文治理（2w+ LOC）🟩 已完成 ✅

> **目标**：在不改行为前提下，降低"巨型文件 + 索引噪音"带来的协作/模型上下文成本。

**已完成（浓缩）**：
- ✅ `.cursorignore` 降低索引噪音（target/dist/assets/覆盖率产物等）
- ✅ 拆分大模块：`vn-runtime/script/parser/*`、`host/command_executor/*`（不改语义，只拆分/去重）
- ✅ 日志与 CLI 规范化：`tracing` + `clap`/`walkdir`

**验收**：`cargo test -p vn-runtime --lib`、`cargo test -p host --lib`、`cargo check-all`

### 阶段 24：演出与体验增强（基于现有动画系统渐进扩展）✅ 已完成

> **主题**：在不破坏"命令驱动 + 显式状态"的前提下，围绕现有动画系统与转场体系，补齐最影响观感的演出能力。

**已实现（浓缩）**：
- ✅ TextBox 显式控制：`textBoxHide/show/clear`（全链路打通）
- ✅ `clearCharacters` 一键清立绘
- ✅ `changeScene` 语义收敛：只做遮罩过渡 + 切背景；不再隐式隐藏 UI/清立绘
- ✅ ChapterMark：非阻塞、固定节奏、覆盖策略
- ✅ 立绘位置：默认瞬移；需要动画时用 `with move/slide`（与 dissolve/fade 解耦）

**关键入口**：`vn-runtime/src/runtime/executor.rs`、`host/src/app/update/scene_transition.rs`、`docs/script_syntax_spec.md`

### 阶段 25：统一动画/过渡效果解析与执行（Effect Registry + AnimationSystem 统一入口）✅ 已完成

> **主题**：把"过渡效果/动画效果"的**解析与执行**收敛到一个统一单元，背景/立绘/UI 共享同一套效果定义与时间轴驱动；命令执行层只负责把 `Transition` 翻译成"对动画系统的请求"，避免多处重复维护。

**已实现（浓缩）**：
- ✅ **统一解析**：`Transition → ResolvedEffect`（`EffectKind/ResolvedEffect/resolve()/defaults`）
- ✅ **统一请求**：`EffectRequest { target, effect }`（替代多套中间类型与字段）
- ✅ **统一应用入口**：`EffectApplier`（统一分发到 AnimationSystem / TransitionManager / SceneTransitionManager）
- ✅ **清理**：移除 `CharacterAnimationCommand/SceneTransitionCommand/TransitionInfo`、移除 `CommandExecutor` 冗余 timer、删除旧 handlers、清理 `AnimationTarget`
- ✅ **测试/文档**：效果矩阵测试 + resolver 单测；统一效果语义表与导航更新

**关键入口**：`host/src/renderer/effects/`、`host/src/app/command_handlers/effect_applier.rs`、`host/src/command_executor/*`

## 下一步开发方向

### 阶段 26：快进/自动/跳过体系（演出推进可控 + 无竞态）🟦 计划中

> **主题**：在不破坏“命令驱动 + 显式状态”的前提下，把**用户推进剧情的体验**（快进/自动/跳过）做成可预测、可测试、无竞态的系统；同时补齐“跳过时的过渡/动画收敛规则”，避免背景/遮罩/立绘进入不一致状态。

**建议优先级：高**（当前演出系统已统一入口，适合把“推进控制”也收敛起来）

**核心目标**：
- **统一推进模式**：Normal / Auto / Skip（或按键按住的临时 Skip）
- **统一跳过语义**：跳过时“该完成的效果必须完成、该切的背景必须切”，且只切一次
- **无竞态**：快点/连点/按住跳过不应导致背景闪现、遮罩卡住、立绘状态残留

**落地建议（按模块）**：
- **输入与模式状态（Host）**：在 `AppState` 建立 `PlaybackMode`（含 auto 的节拍/策略、skip 的触发条件），将点击/按住/自动统一转成更新循环的控制信号
- **过渡/动画的跳过收敛**：`EffectApplier`/`Renderer` 提供“跳过当前演出”的统一入口（内部调用 AnimationSystem / TransitionManager / SceneTransitionManager 的 skip），并明确：
  - Background dissolve：跳过即直接完成到新背景（alpha=1）
  - changeScene：跳过需保证遮罩到达中点并切背景，再完成淡出/收尾（或直接完成到 Completed）

**验收标准（DoD）**：
- 连点/按住 Skip 时：`changeScene Fade/FadeWhite/Rule` 必定切到目标背景，且无闪现/卡遮罩
- 新增单测：SceneTransition/Transition 的 skip 语义覆盖（至少 Fade 与 Rule 两条路径）
- 新增集成测试：脚本层模拟快速输入，验证背景最终状态与过渡完成状态一致
- `cargo check-all` 通过

**关键文件（预期入口）**：
- `host/src/app/update/{mod.rs,scene_transition.rs,script.rs}`
- `host/src/app/command_handlers/effect_applier.rs`
- `host/src/renderer/{mod.rs,scene_transition.rs,transition.rs,animation/system.rs}`

### 阶段 27：Host 结构治理（AppState 解耦 + 子系统边界）🟦 计划中

> **主题**：控制 `AppState` 的“上帝对象”膨胀，把 Host 的状态与能力按职责拆分为若干子系统接口；让 command_handlers/update/screen 只依赖**必要能力**，减少改动波及面，提升可测试性与可读性。

**动机（可维护性问题）**：
- `host/src/app/mod.rs::AppState` 聚合了资源/渲染/输入/执行器/UI/动画/存档/脚本等多类状态，导致：
  - 任意功能改动都容易触及 `AppState` 与大量调用点（高耦合）
  - handler/screen 容易“顺手”拿到不该依赖的能力（边界被侵蚀）
  - 单测/集测构造成本上升（需要填充更多无关字段）

**设计目标**：
- **按能力分层**：将 `AppState` 拆为“子系统 struct + façade 接口”，减少直接字段暴露
- **依赖最小化**：handler/screen 通过参数传入的 façade/trait 获取能力，而不是随处 `&mut AppState`
- **不改语义**：本阶段原则上只做结构治理与迁移，不引入新玩法/新演出表现

**建议落地方式（渐进迁移）**：
- **Step A：定义子系统容器**（先搬字段，不改行为）
  - `CoreSystems`：`ResourceManager` / `Renderer` / `RenderState` / `AnimationSystem` / `CommandExecutor` / `AudioManager`
  - `UiSystems`：`NavigationStack` / `UiContext` / `ToastManager` / screens（Title/Menu/SaveLoad/Settings/History）
  - `GameSession`：`VNRuntime` / `WaitingReason` / `typewriter_timer` / `script_finished` / `manifest` / `character_object_ids`
- **Step B：建立 façade**（控制可见性）
  - 为 `EffectApplier`/update 侧提供 `EffectContext`/`GameContext`（只暴露本模块需要的方法）
  - 将 `pub` 字段逐步收敛为私有，通过 getter/方法访问（减少跨模块写入）
- **Step C：迁移调用点**（以模块为单位）
  - 优先迁移：`host/src/app/command_handlers/*`、`host/src/app/update/*`
  - 再迁移：screens/UI 相关模块

**验收标准（DoD）**：
- `AppState` 字段数量显著下降（或至少不再对外 `pub` 暴露大部分字段）
- `command_handlers` 与 `update` 层不再直接依赖整颗 `&mut AppState`（改为依赖 façade）
- `cargo check-all` 通过，且新增/调整的测试构造成本下降（文档或注释说明）

**关键文件（预期入口）**：
- `host/src/app/mod.rs`（AppState 拆分入口）
- `host/src/app/command_handlers/*`
- `host/src/app/update/*`

---

## 开发原则

1. **遵循 PLAN.md 约束**
   - Runtime 与 Host 严格分离
   - Command 驱动模式
   - 显式状态管理

2. **测试驱动开发**
   - 每个模块都要有单元测试
   - 关键功能要有集成测试
   - 修复 bug 后补充回归测试

3. **渐进式开发**
   - 先实现核心功能，再完善细节
   - 每个阶段都要有可运行的版本
   - 及时集成和测试

4. **代码质量**
   - 清晰的模块划分
   - 完善的文档注释
   - 遵循 Rust 最佳实践

---

> **注意**：本路线图是动态文档，会根据实际开发进度和需求变化进行调整。
