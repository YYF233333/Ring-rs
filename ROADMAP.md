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


## 下一步开发方向

### 阶段 20：仓库可维护性提升 🟦 计划中 

> **主题**：偿还技术债、降低耦合、提升可测试性与工程化质量门禁，让后续功能迭代更快更稳。

**现状（基于仓库扫描）**：
- **`host/src/main.rs` 约 1515 行**：承载了脚本扫描/加载、UI 状态机、存档、运行循环、绘制、命令处理等多种职责，维护成本高。
- `host` 存在不少单元测试，但 **`host/tests` 目前为空**（缺集成测试层），且部分测试对外部环境/设备依赖不清晰。
- **无 CI**（仓库无 `.github`）。作为单人项目当前可以不引入 CI，但仍需要**本地一键质量门禁**（`fmt/clippy/test`）来降低回归风险。
- `host` 里还有多处“超长模块”（如 `command_executor/mod.rs`、`renderer/scene_transition.rs`、`renderer/animation/system.rs` 等），需要继续拆分边界。

---

## 目标与验收标准（Definition of Done）

**工程化门禁**：
- 建立**本地一键质量门禁**：用脚本/alias 把 `cargo fmt --check`、`cargo clippy`、`cargo test` 串起来，保证每次改动都能快速得到红/绿反馈（可按 crate/feature 分层）。
- 可选：基础质量配置（例如 `rustfmt.toml`、Clippy 约束策略）仍建议逐步补齐。
- 可选：引入依赖/许可审计（`cargo-deny` / `cargo-audit`）；等未来开源/多人协作时再接入 CI。

**代码结构**：
- `host/src/main.rs` 目标缩减到 **< 200 行**：仅保留 macroquad 入口、window 配置与启动胶水代码。
- 主要业务逻辑迁移到 `host` library 内的清晰模块边界（App/脚本加载/命令处理/存档/渲染桥接）。
- `command_executor` 与 `renderer` 中的超长文件按职责拆分为子模块，并建立稳定 API（避免跨模块直接读写内部字段）。

**测试体系**：
- `vn-runtime` 维持并继续加强（已有较完整单测基础）。
- `host` 新增 **集成测试**（`host/tests/*.rs` 至少 5 个），覆盖“配置/清单/资源路径/存档回放/命令执行桥接”等关键路径。
- 为 host 引入“可测试边界”：将渲染/音频/时间/文件系统等外部依赖通过 trait/feature 注入，支持 headless 测试。

---

## 具体改进计划（建议按 3 个里程碑落地）

### 20.1 工程化与质量门禁（优先级：最高）
- **本地一键命令**：添加一条“单人开发门禁命令”（推荐其一）
  - `cargo` alias：例如 `cargo q` / `cargo check-all`（在 `.cargo/config.toml` 里维护）
  - 或 `justfile` / `Makefile` / `scripts/`：例如 `just check` 一键执行 `fmt + clippy + test`
- **建议分层执行**（快 → 慢）：
  - `vn-runtime`：全量测试（纯逻辑，最快、最稳定）
  - `host`：优先跑 `cargo test -p host --lib`（避免启动窗口），再逐步引入 headless 集成测试
  - `quality`：`cargo fmt --check` + `cargo clippy`（建议逐步收敛到 `-D warnings`）
- **统一工具链（可选）**：未来如需可复现构建，可再添加 `rust-toolchain.toml`（锁定 Rust 版本 + components）
- **质量基线**：
  - Clippy：以 “新代码零 warning” 为目标（可先从 `vn-runtime` 开始，再逐步收紧到 `host`）
  - 文档：补充 `CONTRIBUTING.md`（开发/测试/打包命令、提交规范、目录约定）

**验收**：本地执行 1-2 条命令即可得到红/绿反馈；主分支始终可构建可测试。

### 20.2 `host/src/main.rs` 拆分（优先级：最高）
把 `main.rs` 当前的顶层职责拆成可维护模块（建议结构）：
- `host/src/bin/host.rs`（或保留 `main.rs` 但只做入口）：macroquad `main` + `window_conf`
- `host/src/app/`
  - `mod.rs`：`App`/`AppState`（只保留高层状态与调度）
  - `update.rs`：`update()` 顶层分发（`Title/InGame/Menu/SaveLoad/Settings/History`）
  - `draw.rs`：`draw()` / `draw_debug_info()`
- `host/src/app/script_loader.rs`
  - `scan_scripts*`、`load_script*`、`collect_prefetch_paths`、脚本来源（FS/ZIP）统一抽象
- `host/src/app/command_handlers/`
  - `audio.rs`：`handle_audio_command`
  - `transition.rs`：`handle_scene_transition`、`apply_transition_effect`
  - `character.rs`：`handle_character_animation`
- `host/src/app/save.rs`
  - `build_save_data`、`quick_save/quick_load`、`restore_from_save_data`、continue 存档逻辑

**验收**：
- `main.rs` 仅保留入口与组装，业务逻辑都有归属模块
- 关键函数拥有清晰的输入/输出（减少 “到处传 `&mut AppState`” 的隐式耦合）

### 20.3 模块边界再梳理（优先级：中）
- **`command_executor` 拆分**：从单文件巨模块拆为 `executor.rs + audio.rs + ui.rs + transition.rs + character.rs` 等
  - 明确：输入（Runtime commands）、输出（渲染/音频/状态变化）与错误模型
- **渲染层瘦身**：对 `renderer/scene_transition.rs`、`animation/system.rs` 进行“类型与职责”拆分
  - 数学/时间轴/缓动：纯逻辑模块（易测）
  - macroquad 绘制：薄壳模块（少逻辑）
- **资源/清单/配置**：整理公共错误类型与路径处理（减少重复）

**验收**：Top-N 超长文件显著减少；模块间依赖更单向（App → executor → renderer/audio/resources）。

### 20.4 host 测试补强（优先级：中-高）
建议把 host 测试分成三层：
- **纯逻辑单测**（现有基础上补齐）：
  - 过渡/动画系统：时间轴推进、状态机边界、插值正确性
  - 配置/manifest：反序列化、默认值、错误提示
- **Headless 集成测试（新增 `host/tests/`）**：
  - 用 stub/trait 注入替代真实渲染/音频设备，跑“命令执行链路 + 状态演进”
  - 典型用例：加载脚本 → tick → 产出 commands → host 执行 → 状态变化断言
- **少量手工/冒烟**：
  - 仅用于验证窗口/音频设备相关（不进入 CI 关键路径）

**验收**：新增的 `host/tests` 能在本地稳定运行（未来接入 CI 时也应可稳定运行）；关键 bug 修复都有回归测试。

---

## 建议的落地顺序（最小风险）
1. 先上本地一键质量门禁 + 工具链（不改业务逻辑，收益最大）
2. 再拆 `main.rs`（只做“搬家重排”，保持行为不变；每次拆分后跑测试）
3. 最后做 headless 注入与集成测试（把外设依赖隔离掉）



### 阶段 20：脚本语法扩展（变量系统 + 条件分支）🟦 计划中

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

### 阶段 21：演出效果增强 🟦 计划中

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

### 阶段 22：编辑器工具 🟦 计划中

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
