# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划，遵循 PLAN.md 中的架构约束。

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

> 目标：避免把 ROADMAP 写成“开发日志”。这里仅保留里程碑结论，细节进入对应阶段归档。

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

## 下一步开发方向

### 仓库瘦身与上下文治理（2w+ LOC）🟩 已完成（短期拆分）

> **主题**：在不改变引擎行为的前提下，优先降低“单文件巨无霸 + 索引噪音”带来的协作/模型上下文成本；其次再评估用成熟库替换易碎的手写解析逻辑。

**现状（数据）**：
- Rust 代码约 **21k LOC**
  - `host` ~13k（59 files）
  - `vn-runtime` ~7k（15 files）
- 最大文件：`vn-runtime/src/script/parser.rs` ~2.7k 行（上下文消耗主因）

**短期（零行为改动 / 立刻见效）**：
- **IDE/模型索引忽略**：新增 `.cursorignore`，忽略 `target/`、`dist/`、`assets/`、覆盖率产物、zip/exe/pdb 等（减少噪音与误读大文件）✅
- **拆分大文件（结构瘦身）**：
  - `vn-runtime/src/script/parser.rs` → `vn-runtime/src/script/parser/*`（按“辅助函数/表达式/阶段1/阶段2/测试”分组），保持 `Parser` API 与测试不变 ✅
    - 入口：`vn-runtime/src/script/parser/mod.rs`
    - 模块：`helpers.rs` / `expr_parser.rs` / `phase1.rs` / `phase2.rs` / `tests.rs`
  - `host/src/renderer/text_renderer.rs` 抽内部共享渲染逻辑，消除 `render_dialogue_box`/`_with_alpha` 以及 choices 渲染的重复路径 ✅
  - `host/src/command_executor/mod.rs` 按 command/feature 拆分文件（只搬代码，不改语义）✅
    - `host/src/command_executor/{background.rs,character.rs,ui.rs,audio.rs,types.rs,mod.rs}`

**中期（可选：真正减少自研代码量）**：
- **脚本解析**：
  - Markdown：评估 `pulldown-cmark` / `comrak` 作为“块识别/表格/强调”等基础解析层，只保留“AST -> ScriptNode”的映射
  - HTML 标签：评估 `tl` / `scraper` 来提取 `img/audio` 的 `src`（减少边界 case 与手写字符串解析代码）
- **开发工具**：`tools/xtask` 评估 `clap`（子命令/帮助/参数校验）与 `walkdir`（文件遍历）
- **日志治理**：`host` 逐步将 `println!/eprintln!` 收敛到 `tracing`/`log`（可控等级，减少输出污染）

**建议优先级（从低风险/高收益开始）**：
- P0：日志治理（Host）→ `tracing`/`log`
- P1：xtask CLI 规范化 → `clap` + `walkdir`
- P2：HTML 标签提取替换（仅 `img/audio src`，可回滚）→ `tl` / `scraper`
- P3：Markdown 解析层替换（优先只做 Phase1 POC 与差异报告）→ `pulldown-cmark` / `comrak`

**建议里程碑（最小可执行集合）**：
- M1（1-2 天）：Host 日志统一（不改行为），关键路径输出字段化，可控等级
- M2（0.5-1 天）：xtask 迁移 `clap`/`walkdir`，`cargo script-check --help` 清晰、参数校验一致
- M3（可选，1-2 天 POC）：HTML 提取库替换试验（保留 fallback），用边界测试护航
- M4（可选，3-5 天 POC）：Markdown 库仅用于“块识别”对比，输出差异报告后再决策是否接入主线

**验收标准（DoD）**：
- “拆分大文件”不改变行为：`cargo test -p vn-runtime --lib` 通过 ✅
- Host 编译与相关测试通过：`cargo check -p host` / `cargo test -p host --lib` ✅
- 一键门禁通过：`cargo check-all` ✅
- 新增/调整的忽略规则不影响打包与运行，但显著降低本地索引/上下文开销



### 阶段 24：演出与体验增强（基于现有动画系统渐进扩展）🟦 计划中

> **主题**：在不破坏“命令驱动 + 显式状态”的前提下，围绕现有动画系统与转场体系，补齐最影响观感的演出能力。

**目标（建议从小到大）**：
- **立绘动效**：在已有 alpha 动画基础上扩展到移动/缩放/缓动（不追求全能，先做最常用）
- **重构 changeScene 职责（给编剧操作空间）**：
  - `changeScene` **只负责**：拉遮罩/蒙版过渡 + 切换背景（不再隐式隐藏 UI / 不再隐式清理立绘）
  - 立绘由编剧显式控制：`hide alias ...` 或（可选新增）`clearCharacters` 一键清空
  - UI 操作由**专门命令**承担：把“对话框显示/隐藏/清理”的语义做成显式命令（避免塞进 `changeScene`）
- **新增 TextBox 命令（对话框显式控制）**：
  - `textBoxHide`：隐藏对话框（不影响背景/立绘）
  - `textBoxShow`：显示对话框
  - `textBoxClear`：清理对话框内容（对话/choices 等按设计明确范围）
- **修复与重定义 ChapterMark 语义（当前实现有问题，实际看不到）**：
  - 语义明确：章节切换时，在**左上角异步显示**一个 mark（非阻塞），“固定持续时间”后自动消失
  - 时间推进**不受用户快进/连续点击影响**（避免被瞬间跳过）
  - 处理章节很短的情况：两个 chapter mark 不能乱叠，需要明确策略（覆盖/队列/延迟/合并）
  - `chapter mark` 与章节强绑定：不需要专用脚本指令（由章节标题节点触发即可）
- **更多过渡效果（可选）**：在现有 `Transition` 表达式之上新增 1-2 个简单效果（如 wipe/slide），并保持参数约定一致

**验收标准（DoD）**：
- 新增效果有最小文档说明（脚本写法 + 默认值）
- `changeScene`/UI/立绘职责分离后，脚本语义更可控：同一段演出可以通过脚本组合出不同“先清 UI/先清立绘/先换背景”的流程
- ChapterMark 可见且稳定：持续时间固定、不受快进影响、不会出现重叠导致的闪烁/不可读
- 与资源系统/缓存系统兼容，不引入额外的全局隐式状态

**关键文件**：
- `host/src/renderer/animation/*`
- `host/src/renderer/{transition.rs,scene_transition.rs,character_animation.rs}`
- `host/src/app/command_handlers/*`

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
