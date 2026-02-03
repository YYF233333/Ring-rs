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

### 里程碑摘要（阶段 1-14）
- **基础架构**：主循环/渲染/输入/资源管理跑通，Runtime/Host 分离落地
- **渲染与输入**：背景/立绘/对话/选择/章节标记 + 打字机效果 + 输入防抖
- **演出与音频**：dissolve/fade/fadewhite/rule(ImageDissolve) + rodio 音频（BGM/SFX/淡入淡出/切换）
- **质量与路径治理**：端到端脚本验证 + 统一 `std::path` 规范化解决 `../` 资源键不一致
- **脚本语法补齐**：音频/控制流（goto） + 立绘布局元数据系统
- **架构性改动**：存档/读档 + 历史记录 + 配置治理
- **结构性完善**：配置落地 + 存档/历史完善 + 资源治理

---

## 阶段 11-14：脚本语法扩展 + 架构完善 + 质量提升 ✅ 已完成

> **成果总结**：补齐脚本语法（音频/控制流）、建立立绘布局元数据系统、实现存档/历史/配置架构、完善测试和文档。

**核心成果**：
- ✅ **脚本语法**：音频指令（SFX/BGM/stopBGM）、控制流（goto）、路径规范化
- ✅ **立绘布局系统**：manifest.json 元数据（anchor/pre_scale/presets），避免硬编码适配
- ✅ **架构性改动**：存档/读档系统、历史记录、配置治理（AppConfig + config.json）
- ✅ **质量提升**：测试覆盖率提升、文档完善（manifest/save 格式说明）

**关键文件**：
- `vn-runtime/src/script/{ast.rs,parser.rs}`、`vn-runtime/src/runtime/executor.rs`
- `host/src/manifest/mod.rs`、`host/src/renderer/mod.rs`
- `vn-runtime/src/{save.rs,history.rs}`、`host/src/{save_manager/mod.rs,config/mod.rs}`
- `docs/{manifest_guide.md,save_format.md}`

---

## 阶段 15-17：演出系统 + 玩家 UI + 体验打磨 ✅ 已完成

> **成果总结**：完成了演出系统重构（changeBG/changeScene 职责分离、过渡命名参数、Rule/ImageDissolve）、玩家 UI 系统（Title/菜单/存读档/设置/历史）、以及体验打磨（Continue 存档、入口配置化、文档整理）。

**核心成果**：
- ✅ **演出系统**：`changeBG`（简单切换）vs `changeScene`（复合演出）职责分离；过渡支持命名参数；Rule/ImageDissolve 两段式实现
- ✅ **玩家 UI**：`AppMode` + `NavigationStack` 状态机；完整的 UI 组件库（Theme/Button/Panel/List/Modal/Toast）；Title/菜单/存读档/设置/历史界面
- ✅ **体验打磨**：Continue 专用存档；SaveLoad 1-99 槽位 + 完整元信息；入口脚本配置化；文档同步更新

**关键文件**：
- `host/src/renderer/transition.rs`、`host/src/renderer/image_dissolve.rs`
- `host/src/app_mode.rs`、`host/src/ui/*`、`host/src/screens/*`
- `host/src/save_manager/mod.rs`、`host/src/config/mod.rs`

---

## 阶段 18：Scale-up 资源动态加载 + 资源打包/发布 ✅ 已完成

> **成果总结**：实现了完整的资源动态加载系统，支持按需加载、LRU 缓存驱逐、显存预算控制；支持从文件系统或 ZIP 包加载资源；提供资源打包工具用于发布。详见 `docs/resource_management.md`。

**核心功能**：
- ✅ **ResourceSource 抽象层**：统一文件系统和 ZIP 包的资源访问接口
- ✅ **TextureCache + LRU 驱逐**：默认 256MB 显存预算，自动驱逐最久未使用的纹理
- ✅ **按需加载**：启动不再预加载所有资源，运行时按需加载并缓存
- ✅ **ZipSource + 打包工具**：支持将资源打包为 ZIP 文件，发布时无需散落资源目录
- ✅ **Debug Overlay**：按 F1 显示缓存统计（命中率、占用、驱逐次数等）

**关键文件**：
- `host/src/resources/`：ResourceManager、ResourceSource、TextureCache
- `tools/asset-packer/`：资源打包工具
- `docs/resource_management.md`：用户使用指南

---

### 阶段 19：动画体系重构 ✅ 已完成

> **主题**：统一动画系统架构，基于 Trait 的类型安全设计。

**要点**：
- 动画系统只做时间轴：驱动对象 `f32` 属性从 A→B（无中心化值缓存）
- 对象实现 `Animatable` 暴露属性；系统分配 `ObjectId`，用 `AnimPropertyKey(TypeId + ObjectId + property_id)` 唯一定位

**已完成**：
- ✅ 角色动画：`AnimatableCharacter`
- ✅ 背景过渡：`AnimatableBackgroundTransition` + `TransitionManager`
- ✅ 场景切换：`AnimatableSceneTransition` + `SceneTransitionManager`（动画系统驱动 shader 变量）
- ✅ 清理旧架构：移除 `PropertyKey` 字符串 key + 值缓存模式

**changeScene 流程（简版）**：
- `CommandExecutor` 产出 `SceneTransitionCommand`（Fade/FadeWhite/Rule）
- `main.rs` 启动/更新过渡（中间点切背景；结束后恢复 UI）
- `SceneTransitionManager` 管理多阶段（FadeIn/FadeOut/UIFadeIn）
- `Renderer` 读取进度/alpha 渲染遮罩或 dissolve

**关键文件**：
- `host/src/renderer/animation/`
- `host/src/renderer/character_animation.rs`
- `host/src/renderer/transition.rs`
- `host/src/renderer/scene_transition.rs`

---

## 阶段 20：仓库可维护性提升（技术债偿还）✅ 已完成

> **主题**：降低耦合、减少“巨型文件/巨型模块”、补齐本地质量门禁与测试层，让后续功能迭代更快更稳。

**核心成果（浓缩）**：
- ✅ **本地质量门禁**：新增 `cargo` alias + `tools/xtask`，一键执行 `fmt --check → clippy → test`（`cargo check-all`）
- ✅ **入口瘦身**：`host/src/main.rs` **1821 → 169 行**，仅保留 macroquad 入口与胶水；业务逻辑下沉到 `host/src/app/*`
- ✅ **结构与死代码治理**：
  - `command_executor` 拆分：类型定义下沉到 `host/src/command_executor/types.rs`（主模块明显变短）
  - 清理遗留/无用实现：旧脚本加载器、无过渡渲染函数、无用字段/unused imports 等
- ✅ **host 测试补强**：
  - 单元测试：**~92 → 111**（补齐 `command_executor` / `render_state` 关键逻辑）
  - 集成测试：`host/tests/command_execution.rs` 新增 **7** 个 headless 场景测试

**关键文件**：
- `.cargo/config.toml`、`tools/xtask/`
- `host/src/main.rs`、`host/src/app/`
- `host/src/command_executor/{mod.rs,types.rs}`
- `host/src/renderer/render_state.rs`
- `host/tests/command_execution.rs`

---

## 阶段 21：覆盖率度量落地 + host 结构第二轮 ✅ 已完成

> **主题**：在不引入 CI 的前提下，把“质量反馈”从**能跑**提升到**可度量**；同时继续梳理 host 不合理结构/边界，让后续功能迭代更顺手。

**核心成果（浓缩）**：
- ✅ **覆盖率口径与工具链**：以 `cargo llvm-cov` 作为 Windows 友好的主口径，本地一条命令生成 HTML 报告（`cargo cov-runtime` / `cargo cov-workspace`）
- ✅ **vn-runtime 覆盖率冲刺达标**（`--all-features`）：
  - **行覆盖率**：**96.99%**
  - **`script/parser.rs` 行覆盖率**：**94.10%**（补齐大量错误/边界分支与 warning 路径）
- ✅ **host 结构与边界第二轮**（测试非主目标，重构优先）：
  - 入口进一步去业务化：资源引导/按需加载从 `host/src/main.rs` 下沉到 `host::app`（`app/bootstrap.rs`）
  - 初始化拆分：`AppState::new` 的资源/音频/manifest/脚本扫描/用户设置等初始化收口到 `app/init.rs`
  - 更新逻辑模块化：将“巨型 `app/update.rs`”拆分为 `app/update/`（modes/script/scene_transition），降低耦合、便于继续演进
- ✅ **本地质量门禁稳定**：`cargo fmt-all` + `cargo check-all` 重构后持续通过

**关键文件**：
- `docs/coverage.md`、`.cargo/config.toml`、`tools/xtask/`
- `vn-runtime/src/{script/parser.rs,runtime/*,save.rs,history.rs,state.rs,command.rs}`
- `host/src/main.rs`、`host/src/app/{bootstrap.rs,init.rs,update/*}`


## 下一步开发方向

### 阶段 22：脚本逻辑系统（变量/表达式/条件）🟦 计划中

> **主题**：在保持 Markdown 可读性的前提下，引入“可控、可测试、可存档”的脚本逻辑能力；严格遵循 `PLAN.md` 的确定性与显式状态约束。

**目标（按优先级）**：
- **变量模型**：`Number/String/Bool` 三类即可；支持读写；**随存档持久化**
- **表达式**：算术/比较/逻辑；无副作用；错误信息带行号/上下文
- **条件分支**：最小可用 `if/elseif/else`
- （可选）**循环**：如果确实需要，再做 `while`（必须有防死循环策略与错误提示）（不需要，我们已经有goto，配合ifelse可以实现循环）

**约束与取舍**：
- Runtime 只做**结构解析 + 确定性求值**，不做 IO，不依赖真实时间
- Host 不参与脚本逻辑（仍然只执行 Command）
- 语法优先“写给编剧”，宁可少而稳，不做全语言化

**关键设计点（建议落地方向）**：
- `RuntimeState` 增加 `variables: HashMap<String, Value>`
- AST 增加最小集合：`SetVar`、`If`（以及表达式节点/字面量/变量引用）
- Parser 增量扩展：保持两阶段块解析架构（见 `docs/script_syntax_spec.md`）
- Executor/Engine：表达式求值与分支跳转必须可测试（拒绝隐式状态）

**验收标准（DoD）**：
- 新语法有**正式规范**（同步更新 `docs/script_syntax_spec.md`）
- `vn-runtime` 有覆盖关键语义的测试（含错误路径与存档 roundtrip）

**关键文件**：
- `vn-runtime/src/{state.rs,save.rs,script/{ast.rs,parser.rs},runtime/{executor.rs,engine.rs}}`
- `docs/script_syntax_spec.md`

### 阶段 23：开发者工作流与内容校验（lint/资源引用检查/诊断）🟦 计划中

> **主题**：仓库已经进入“功能迭代期”，下一阶段把编剧/制作过程里的问题前置：脚本/资源/manifest 的错误尽量在运行前就被发现。

**目标**：
- **脚本静态检查**（不运行游戏也能做）：
  - 未定义 label / 不可达 label
  - choice 表格格式错误与目标缺失
  - `changeScene` 缺 `with`、参数类型不匹配等（复用 parser 的错误信息）
- **资源引用检查**：
  - 脚本里的 `<img src>` / `<audio src>` 统一解析为逻辑路径后，检查资源是否存在
  - Rule 遮罩/背景/立绘等引用路径统一规范化（与 `ResourceManager` 口径一致）
- **诊断输出体验**：
  - 报错格式统一：文件/行号/原始行/解释
  - 可选生成一份汇总报告（便于长脚本批量修）

**落地形态（建议）**：
- 以 `tools/xtask` 子命令形式提供：例如 `cargo script-check`（或 `cargo run -p xtask -- script-check`）
- 允许“只读扫描”：不触碰 macroquad/音频设备，避免环境依赖
- **Dev Mode 自动诊断（强烈建议）**：在 **debug build**（`cfg(debug_assertions)`）或配置开启的 dev mode 下，**游戏启动时自动执行一次 script-check**，输出诊断摘要与可定位明细（避免“经常忘记跑”）

**验收标准（DoD）**：
- 对 `assets/scripts/` 可以一条命令跑完检查，失败时输出可定位的诊断
- Dev Mode 启动时自动跑检查：默认 **只告警不阻塞启动**（除非显式配置为 hard-fail）
- 文档补齐：`CONTRIBUTING.md`/`docs/*` 写清怎么用

**关键文件**：
- `tools/xtask/src/main.rs`
- `vn-runtime/src/script/*`
- `host/src/resources/{path.rs,mod.rs}`（路径规范化口径）

### 阶段 24：演出与体验增强（基于现有动画系统渐进扩展）🟦 计划中

> **主题**：在不破坏“命令驱动 + 显式状态”的前提下，围绕现有动画系统与转场体系，补齐最影响观感的演出能力。

**目标（建议从小到大）**：
- **立绘动效**：在已有 alpha 动画基础上扩展到移动/缩放/缓动（不追求全能，先做最常用）
- **重构 changeScene 职责（给编剧操作空间）**：
  - `changeScene` **只负责**：拉遮罩/蒙版过渡 + 切换背景（不再隐式隐藏 UI / 不再隐式清理立绘）
  - 立绘由编剧显式控制：`hide alias ...` 或（可选新增）`clearCharacters` 一键清空
  - UI 操作由**专门命令**承担：把“对话框显示/隐藏/清理”的语义做成显式命令（避免塞进 `changeScene`）
- **移除 `UIAnim`**：
  - `UIAnim` 命令本身要去掉（不再作为脚本语法/Runtime Command）（已经去掉了）
  - 如需要 UI 动画，改为更明确的命令集合（先做最小集合，后续再扩展“动画”概念）
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
