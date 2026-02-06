# Visual Novel Engine 开发路线图

> 本文档定义了项目的架构约束和具体执行计划。

## 总体架构原则（硬约束）

- Runtime 与 Host 分离
   - **`vn-runtime`**：纯逻辑核心（脚本解析/执行、状态管理、等待建模、产出 `Command`）
   - **`host`**：IO/渲染/音频/输入/资源宿主（执行 `Command` 产生画面/音频/UI）
   - Runtime **禁止**：引擎 API（macroquad）、IO、真实时间依赖
   - Host **禁止**：脚本逻辑；直接修改 Runtime 内部状态

- 显式状态、确定性执行
   - 所有运行状态必须**显式建模**且可序列化（支持存档/读档）
   - 不允许隐式全局状态
   - 不依赖真实时间推进逻辑（时间等待由 Host 负责）

- 命令驱动（Command-based）
   - Runtime **只产出** `Command`
   - Host **只执行** `Command`
   - Runtime 不直接渲染/播放音频/等待输入

---

## VN Runtime 核心模型（必须遵守）

- `RuntimeState`（唯一可变状态）
   - 脚本执行位置（`ScriptPosition`）
   - 脚本变量（variables）
   - 当前等待状态（`WaitingReason`）
   - 以及其他可恢复的显式状态（如已显示角色/背景等）

   要求：**可序列化**、可测试；禁止隐式状态。

- `WaitingReason`（显式等待模型）

   允许的等待原因（示例口径）：

   ```text
   None
   WaitForClick
   WaitForChoice { choice_count }
   WaitForTime(Duration)
   WaitForSignal(SignalId)
   ```

   禁止使用隐式 await/sleep 来推进脚本。

- 执行模型（tick）
   - Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动
   - 若处于等待：仅处理输入尝试解除等待
   - 若不等待：持续推进脚本直到再次阻塞或结束

- `RuntimeInput`（Host → Runtime）

   典型输入：

   ```text
   Click
   ChoiceSelected(index)
   Signal(signal_id)
   ```

   说明：`WaitForTime` 由 Host 处理（Host 等待指定时长再调用 tick）。

---

## Command 模型（Runtime → Host）

- `Command` 是 Runtime 与 Host 的**唯一通信方式**
- 要求：**声明式**、不包含引擎类型、不产生副作用


---

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

**已实现**：
- **脚本静态检查**（不运行游戏也能做）：
  - **未定义 label（Error）**：所有"显式跳转目标"（如 `goto`/choice 目标等）必须存在 ✅
  - choice 表格目标缺失检测 ✅
  - 语法错误（复用 parser 的错误信息）✅
- **资源引用检查**：
  - 脚本里的 `<img src>` / `<audio src>` 统一解析为逻辑路径后，检查资源是否存在 ✅
  - 路径规范化（与 `ResourceManager` 口径一致）✅
- **诊断输出体验**：
  - 报错格式统一：文件/脚本ID/消息/详情 ✅
  - 诊断分级：Error/Warn/Info ✅

**落地形态**：
- **核心能力放在 `vn-runtime`**：`vn-runtime/src/diagnostic.rs` 提供纯函数的脚本分析/诊断 API
- 以 `tools/xtask` 子命令形式提供：`cargo script-check`
- 允许"只读扫描"：不触碰 macroquad/音频设备，避免环境依赖

**使用方式**：
- `cargo script-check`：检查 `assets/scripts/` 下所有脚本
- `cargo script-check <path>`：检查指定文件或目录

**关键文件**：
- `vn-runtime/src/diagnostic.rs`（诊断 API）
- `vn-runtime/src/script/ast.rs`（Script.source_map 源码映射）
- `tools/xtask/src/main.rs`（script-check 命令）
- `host/src/app/init.rs`（Dev Mode 自动诊断）
- `host/src/config/mod.rs`（debug.script_check 配置）

**已实现的扩展功能**：
- ✅ Dev Mode 自动诊断：在 debug build 或 `debug.script_check=true` 时，Host 启动时自动检查脚本
- ✅ 行号精确定位：诊断输出包含准确的源码行号（如 `script.md:42: 错误信息`）

### 仓库瘦身与上下文治理（2w+ LOC）🟩 已完成 ✅

> **目标**：在不改行为前提下，降低"巨型文件 + 索引噪音"带来的协作/模型上下文成本。

**已完成（关键改动）**：
- ✅ `.cursorignore`：忽略 `target/`、`dist/`、`assets/`、覆盖率产物、zip/exe/pdb 等，降低索引噪音
- ✅ `vn-runtime`：`script/parser.rs` 拆分为 `script/parser/*`（保持 `Parser` API 与测试不变）
- ✅ `host`：拆分 `command_executor`；`text_renderer` 去重复逻辑（只搬/抽公共逻辑，不改语义）
- ✅ `host` 日志治理：`println!/eprintln!` → `tracing`（可控等级、字段化），并支持 `config.json debug.log_level`
- ✅ `xtask`：CLI 规范化为 `clap` + `walkdir`（`cargo script-check --help` 清晰、参数校验一致）

**验收（DoD）**：
- ✅ `cargo test -p vn-runtime --lib`
- ✅ `cargo check -p host` / `cargo test -p host --lib`
- ✅ `cargo check-all`

## 下一步开发方向

### 阶段 24：演出与体验增强（基于现有动画系统渐进扩展）✅ 已完成

> **主题**：在不破坏"命令驱动 + 显式状态"的前提下，围绕现有动画系统与转场体系，补齐最影响观感的演出能力。

**已实现**：

- ✅ **新增 TextBox 命令（对话框显式控制）**：`textBoxHide` / `textBoxShow` / `textBoxClear`
  - 全链路：AST → Parser → Executor → Command → Host CommandExecutor
- ✅ **新增 clearCharacters 命令**：一键清除所有角色立绘
- ✅ **重构 changeScene 职责**：只负责遮罩过渡 + 切换背景，不再隐式隐藏 UI / 清除立绘
- ✅ **修复 ChapterMark 语义**：非阻塞、固定持续时间（FadeIn 0.4s → Visible 3.0s → FadeOut 0.6s）、覆盖策略
- ✅ **立绘动效（位置移动）**：`show alias at newPosition` 默认瞬移；需要动画时用 `show alias at newPosition with effect`

**测试覆盖**：vn-runtime 195 tests / host 114 tests 全部通过

**待完成（可选扩展）**：更多过渡效果（wipe/slide）、立绘缩放动画

**关键文件**：
- `vn-runtime/src/script/ast.rs`、`command.rs`、`runtime/executor.rs`、`script/parser/phase2.rs`
- `host/src/command_executor/{ui,background,types}.rs`、`host/src/app/command_handlers/character.rs`
- `host/src/renderer/render_state.rs`、`host/src/app/update/{mod,scene_transition}.rs`
- `docs/script_syntax_spec.md`

### 阶段 25：统一动画/过渡效果解析与执行（Effect Registry + AnimationSystem 统一入口）🟦 计划中

> **主题**：把“过渡效果/动画效果”的**解析与执行**收敛到一个统一单元，背景/立绘/UI 共享同一套效果定义与时间轴驱动；命令执行层只负责把 `Transition` 翻译成“对动画系统的请求”，避免多处重复维护（例如 `dissolve` 同时存在于背景与立绘的实现）。

#### 背景与动机（为什么要做）
- 当前 `Transition`（如 `dissolve/fade`）在不同目标（背景/场景/立绘）存在**各自的解释与实现**，导致：
  - 语义漂移：同名效果在不同对象上表现不一致
  - 维护成本高：改一个效果要改多处
  - 扩展困难：新增 wipe/slide 等效果需要复制粘贴多份逻辑

#### 设计目标

- **统一效果源**：同一个效果名（如 `dissolve`）在所有目标上共享同一份“解析/默认值/校验/时间轴”

#### 核心方案（统一解析单元）
- 在 Host 引入一个“效果注册表/解析器”模块（建议命名 `host/src/renderer/effects/`）：
  - `EffectRegistry`：维护效果名 → 规格（支持哪些参数、默认值、适用目标、输出哪些属性动画）
  - `EffectResolver`：把 `vn_runtime::command::Transition` 解析成 `ResolvedEffect`（已填默认值、已校验）
  - `EffectApplier`：把 `ResolvedEffect + Target` 转成对 `AnimationSystem` 的**一组动画请求**（可多属性、多阶段）

#### 目标分类（Target Model）
- **以 `Animatable` + `ObjectId` 为唯一动画对象模型**：
  - `Animatable` 只描述“对象暴露哪些可动画属性”（能力接口），不承担路由/唯一标识
  - `ObjectId`（由 `AnimationSystem::register` 分配）是动画系统内部唯一标识，用于索引动画、去注册、以及属性键 `AnimPropertyKey(TypeId + ObjectId + property)`
- 在 Effect 层引入 `EffectRequest`（或 `EffectContext`）作为统一输入：**“动画对象是谁（ObjectId） + 这次效果需要哪些上下文 + 要改哪些属性”**：
  - `EffectRequest { object_id: background_id, kind: Background { old_bg, new_bg }, effect: ResolvedEffect }`
  - `EffectRequest { object_id: character_id(alias), kind: Character { old_pos, new_pos, texture_change }, effect: ResolvedEffect }`
  - `EffectRequest { object_id: scene_mask_id, kind: SceneMask { ... }, effect: ResolvedEffect }`
  - （可选）UI 元素同理：`object_id` 由 UI 管理层维护映射
- （清理项）`AnimationTarget` 当前无引用：在阶段 25 推进统一入口后，可删除该模块与导出，避免“概念存在但无实现落点”的噪音。

#### 效果语义规范（先收敛，再扩展）
- **第一批统一效果**（把当前重复处收敛掉）：
  - `dissolve(duration=0.3)`：统一为“alpha 交叉淡化”的通用时间轴（背景/立绘/遮罩复用）
  - `fade(duration=0.3)`：同上（与 dissolve 在实现层可共享，只是名称别名/参数差异）
  - `rule(src=..., duration=...)`：保持现有场景/背景用法，统一参数解析与默认值
- **位置动画**：
  - `move(duration=0.3, easing=linear)` / `slide(...)`：仅对 `Character` 的 position 偏移生效
  - **明确约定**：`show alias at pos` 默认瞬移；只有 `with move/slide` 才平滑移动（`with dissolve/fade` 不触发移动）

#### 分阶段落地计划（可并行但建议顺序）
- **Step A：抽离解析与默认值**（不改表现）
  - 把 `command_executor/*` 中对 `Transition.name/duration` 的手写解析迁移到 `EffectResolver`
  - 为每个效果补齐参数校验与默认值（单元测试覆盖）
- **Step B：统一执行入口**（减少重复代码）
  - 背景过渡、场景遮罩过渡、立绘淡入淡出：改为统一走 `EffectApplier → AnimationSystem`
  - `CommandExecutor` 只负责选择 target + 触发 apply（不再自己算 duration/分支）
- **Step C：补齐效果矩阵测试**
  - 同一 `dissolve` 在 Background/Character/SceneMask 上：解析一致、默认值一致、不会产生语义分叉
  - 回归测试：现有脚本演出行为不变（除明确修正的语义）
- **Step D：文档与脚本规范更新**
  - 在 `docs/script_syntax_spec.md` 增加“效果名的统一语义表”
  - 明确哪些效果适用哪些目标，哪些参数可用

#### 关键文件（预期改动入口）
- 新增：`host/src/renderer/effects/{mod.rs,registry.rs,resolver.rs,applier.rs}`（命名可调整）
- 调整：`host/src/command_executor/{background.rs,character.rs,ui.rs,mod.rs}`
- 调整：`host/src/app/command_handlers/*`（把动画请求统一交给 AnimationSystem）
- 既有动画系统：`host/src/renderer/animation/*`

#### 验收标准（DoD）
- `dissolve/fade/rule` 的参数解析与默认值只存在**一处**（registry/resolver），并有单测
- 背景/立绘/场景遮罩的过渡执行路径统一走 `EffectApplier → AnimationSystem`
- 同名效果在不同目标上行为一致（除 target 本身差异）
- `cargo test -p host --lib` 通过，并新增覆盖“效果解析一致性”的测试

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
