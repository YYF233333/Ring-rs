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


## 下一步开发方向

### 阶段 21：测试覆盖率攻坚（vn-runtime 近 100%）+ 技术债继续偿还 🟦 计划中

> **主题**：在不引入 CI 的前提下，把“质量反馈”从**能跑**提升到**可度量**；同时继续梳理不合理结构/边界，降低后续功能迭代风险。

**背景**：
- `cargo tarpaulin` 显示 workspace 总体覆盖率 < 20%（host 受环境/设备/渲染驱动影响很难拉满）
- `vn-runtime` 基本不依赖外部环境，理论上可做到接近 100% 覆盖率

### 21.1 覆盖率度量与口径统一（优先级：高）
- 统一覆盖率口径：
  - **主目标**：`vn-runtime` 覆盖率（以 line 为主，必要时加 branch）
  - **次目标**：workspace 覆盖率（允许合理 exclude：macroquad 入口/平台适配等）
- 产出本地可复现报告（HTML/JSON 均可），并写清“如何跑”和“怎么看”。
- 可选：评估 `cargo llvm-cov`（Windows 友好）与 `cargo tarpaulin`（Linux/WSL 常用）的取舍，最终选一个作为主口径。

**验收**：开发者本地一条命令可以生成报告；文档写清统计口径与 exclude 列表。

### 21.2 vn-runtime 覆盖率冲刺（优先级：最高）
**目标**：`vn-runtime` 覆盖率达到 **95%+**（目标值可根据实际分支复杂度再收敛到 98%）。

建议按模块补齐用例：
- **脚本解析**：更多错误用例、边界输入（空行/缩进/非法 token/转义字符串/参数列表）
- **执行引擎**：WaitingReason/输入驱动推进、goto/label、命令序列边界、错误路径
- **状态/序列化**：save/load roundtrip、版本兼容（如有）、history 记录边界
- **命令结构**：Transition 参数解析（位置/命名参数）、不允许混用的约束测试

**验收**：覆盖率达到目标；关键 bug 修复都有回归测试；无“只为覆盖率而写”的脆弱测试（断言语义而非实现细节）。

### 21.3 host 结构与可测试边界第二轮（优先级：中）
> 目标不是让 host 覆盖率“接近 100%”，而是把“能测的纯逻辑”抽出来，并建立稳定边界。

- 继续盘点不合理结构（文件位置、模块耦合、跨层读写内部字段）
- 将可测试纯逻辑下沉到：
  - `vn-runtime`（优先）或 host 的纯逻辑模块（不触碰 macroquad/音频设备）
- 对外部依赖（时间/文件系统/音频/渲染）逐步引入 trait 注入或 feature gate（仅对新增/重构模块做，不强行全盘重写）

**验收**：新增/重构模块自带测试；host 的 headless 集成测试覆盖更多关键链路（按需增加 3-5 个场景）。



### 阶段 22：脚本语法扩展（变量系统 + 条件分支）🟦 计划中

> **主题**：扩展脚本语言，支持变量、条件分支、循环等编程特性，使脚本更灵活。

**目标**：
- **变量系统**：支持数字、字符串、布尔类型；变量作用域（全局/局部）；变量持久化到存档
- **条件分支**：`if/elseif/else` 语法，支持变量比较和逻辑运算
- **循环**：`while` 循环，支持条件控制
- **表达式求值**：支持算术、比较、逻辑运算

**核心设计**：
- `RuntimeState` 扩展：添加 `variables: HashMap<String, Value>` 字段
- AST 扩展：新增 `If`、`While`、`SetVariable` 节点
- 表达式解析器：支持变量引用和运算（`$var_name` 语法）
- 向后兼容：现有脚本无需修改即可运行

**关键文件**：
- `vn-runtime/src/script/ast.rs`：扩展 AST 节点
- `vn-runtime/src/script/parser.rs`：表达式解析
- `vn-runtime/src/runtime/engine.rs`：变量作用域管理
- `vn-runtime/src/state.rs`：RuntimeState 扩展

**验收标准**：
- 支持变量声明、赋值、引用
- 支持 `if/elseif/else` 条件分支
- 支持 `while` 循环
- 变量随存档持久化
- 现有脚本无需修改即可运行

### 阶段 23：演出效果增强 🟦 计划中

> **主题**：增强演出效果，支持立绘动画、对话框动画、更丰富的过渡效果。

**目标**：
- **立绘动画**：淡入/淡出、移动、缩放动画
- **对话框动画**：显示/隐藏动画、样式切换
- **过渡效果扩展**：更多内置过渡效果（wipe、slide 等）
- **动画系统**：统一的时间轴和缓动函数（ease-in/out 等）

**关键文件**：
- `host/src/renderer/animation.rs`：动画系统
- `host/src/renderer/character.rs`：立绘动画
- `host/src/renderer/dialogue.rs`：对话框动画

### 阶段 24：编辑器工具 🟦 计划中

> **主题**：开发可视化脚本编辑器，提升开发效率。

**目标**：
- **脚本编辑器**：语法高亮、自动补全、实时预览
- **资源管理器**：可视化资源浏览和管理
- **调试工具**：断点、变量监视、执行流程可视化

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
