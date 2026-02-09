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
   - ✅ 资源管理系统（PNG/JPEG/WebP 支持）
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

### 阶段 23：开发者工作流与内容校验（lint/资源引用检查/诊断） 已完成

> **目标**：把脚本/资源/manifest 的常见错误前置到“运行前可发现”。

**交付**：
-  静态脚本诊断：未定义 `label`、`choice` 目标缺失、语法错误（复用 parser 报错）
-  资源引用检查：`<img src>` / `<audio src>` 统一逻辑路径解析 + 存在性校验（与 `ResourceManager` 同口径）
-  统一诊断体验：文件/脚本ID/行号/详情 + 分级（Error/Warn/Info）
-  工具链：`cargo script-check [path]`（xtask，支持只读扫描；Dev Mode 可自动诊断）

**关键入口**：`vn-runtime/src/diagnostic.rs`、`tools/xtask/src/main.rs`、`host/src/app/init.rs`

### 仓库瘦身与上下文治理（2w+ LOC） 已完成 

> **目标**：不改行为，降低“巨型文件 + 索引噪音”的协作成本。

**交付**：
-  `.cursorignore`：屏蔽 `target/`、`dist/`、资产与覆盖率产物等噪音
-  拆分大模块：`vn-runtime/script/parser/*`、`host/command_executor/*`（仅拆分/去重，不改语义）
-  日志/CLI 规范化：`tracing` + `clap`/`walkdir`

**验收**：`cargo test -p vn-runtime --lib`、`cargo test -p host --lib`、`cargo check-all`

### 阶段 24：演出与体验增强（基于现有动画系统渐进扩展） 已完成

> **目标**：不破坏“命令驱动 + 显式状态”，补齐最影响观感的演出能力。

**交付**：
-  TextBox 显式控制：`textBoxHide/show/clear` 全链路打通
-  `clearCharacters`：一键清立绘
-  `changeScene` 语义收敛：仅遮罩过渡 + 切背景；不再隐式隐藏 UI/清立绘
-  ChapterMark：非阻塞、固定节奏、覆盖策略
-  立绘位置：默认瞬移；需要动画时用 `with move/slide`（与 dissolve/fade 解耦）

**关键入口**：`vn-runtime/src/runtime/executor.rs`、`host/src/app/update/scene_transition.rs`、`docs/script_syntax_spec.md`

### 阶段 25：统一动画/过渡效果解析与执行（Effect Registry + AnimationSystem 统一入口） 已完成

> **目标**：将“效果的解析与执行”收敛为统一入口，避免多处重复维护。

**交付**：
-  统一解析：`Transition → ResolvedEffect`（`EffectKind/ResolvedEffect/resolve()/defaults`）
-  统一请求：`EffectRequest { target, effect }`（替代多套中间类型/字段）
-  统一应用：`EffectApplier` 分发到 AnimationSystem / TransitionManager / SceneTransitionManager
-  清理与收敛：移除旧 command/中间类型、删除冗余 timer/handlers、清理 `AnimationTarget`
-  测试与文档：效果矩阵 + resolver 单测；效果语义表/导航同步更新

**关键入口**：`host/src/renderer/effects/`、`host/src/app/command_handlers/effect_applier.rs`、`host/src/command_executor/*`

### 阶段 26：快进/自动/跳过体系（演出推进可控 + 无竞态） 已完成

> **目标**：推进模式可预测、可测试、无竞态；跳过时过渡/动画语义一致（该完成的完成、该切的只切一次）。

**交付**：
-  推进模式：`PlaybackMode::{Normal,Auto,Skip}`（Skip 为 Ctrl 按住的临时模式；Auto 为 A 键切换）
-  Auto：`WaitForClick + 对话已完成 + 无活跃效果` 时，计时到 `user_settings.auto_delay` 自动推进
-  Skip：统一入口 `skip_all_active_effects()`，一帧内收敛角色动画 / changeBG dissolve / changeScene / 打字机
-  changeScene 跳过不丢背景：`SceneTransitionManager::skip_to_end()` → `pending_background`；`Renderer::skip_scene_transition_to_end()` façade
-  测试：补齐 SceneTransition/Transition 的 skip 语义单测（含 midpoint / mid-dissolve 等关键路径）

**DoD（摘要）**：
- 连点/按住 Skip：`changeScene Fade/FadeWhite/Rule` 必定落到目标背景，无闪现/卡遮罩
- `cargo check-all` 通过

**关键文件（预期入口）**：
- `host/src/app/update/{mod.rs,scene_transition.rs,script.rs}`
- `host/src/app/command_handlers/effect_applier.rs`
- `host/src/renderer/{mod.rs,scene_transition.rs,transition.rs,animation/system.rs}`

### 阶段 27：Host 结构治理（AppState 解耦 + 子系统边界） 已完成

> **目标**：抑制 `AppState` 上帝对象膨胀；明确子系统边界，让命令处理层只依赖必要能力。

**交付**：
-  子系统容器：`CoreSystems` / `UiSystems` / `GameSession`；`AppState` 顶层字段 ~30 → 12（3 子系统 + 基础设施）
-  `command_handlers` 脱离 `AppState`：
  - `apply_effect_requests` / `apply_*` / `ensure_character_registered` → `(&mut CoreSystems, &Manifest)`
  - `handle_audio_command` → `(&mut CoreSystems, &AppConfig)`
  - `skip_all_active_effects` / `cleanup_fading_characters` → `(&mut CoreSystems)`
-  调用点迁移：12 个文件字段路径更新（`app_state.X` → `app_state.{core,ui,session}.X`）
-  门禁通过：`cargo check-all`（fmt + clippy + tests）

**DoD（摘要）**：
- `command_handlers` 不再依赖 `&mut AppState`；`AppState` 顶层字段显著下降
- 注：`update` 层部分入口（如 `run_script_tick` / `handle_script_mode_input` / `modes.rs`）因跨子系统访问仍保留 `&mut AppState`

**关键文件**：
- `host/src/app/mod.rs`（`CoreSystems` / `UiSystems` / `GameSession` 定义 + `AppState` 重构）
- `host/src/app/command_handlers/*`（facade 签名迁移）
- `host/src/app/update/*`（字段路径迁移 + `skip_all_active_effects` facade）

### 阶段 28：对话语音体系（voice_id 标注 + 自动播放 + 工具链） 规划中

> **目标**：让对话语音“按 voice_id 对齐并自动播放”，编剧无需手动写 `playSFX/stopSFX`；缺失语音可被工具链提前发现。

**交付（方案 B）**：
- **脚本标注**：只标注 `voice_id`，不引入“播放语音”脚本命令  
  - 对话行尾：`角色："台词" [#v:foo_001]`  
- **解析/AST**：解析器把 `voice_id: Option<String>` 绑定到对话节点
- **通信契约（Command）**：扩展 `Command::ShowText` 为 `speaker/content + voice_id: Option<String>`，保持 Runtime 仍只产出声明式 Command
- **Host 播放语义**：Host 在执行 `ShowText` 时按约定路径自动查找并播放到 **Voice 专用通道**  
  - 默认路径：`voices/{voice_id}.ogg`（可按顺序尝试 ogg/mp3/wav/flac）  
  - 找不到则静默跳过（可选 `script-check` 预警，运行时不崩溃）  
  - 推进到下一句/Skip：stop 当前 voice；Auto：可选“等待 voice 播完再推进”（可配置）
- **工具链**：新增 `cargo voice-index`（或扩展 `cargo script-check --voices`）生成“台词→voice_id→资源路径→缺失项”清单

**DoD（摘要）**：
- voice 缺失不影响运行；voice 存在时能随对话稳定播放且不会叠音
- Skip/推进下 voice 行为可预测（默认 stop），Auto 可配置策略
- `voice-index`/`script-check --voices` 能产出清单并对缺失资源给出可定位诊断

**关键入口（预期）**：
- 语法/解析：`docs/script_syntax_spec.md`、`vn-runtime/src/script/parser/mod.rs`、`vn-runtime/src/script/ast.rs`
- Command/执行：`vn-runtime/src/command.rs`（`Command::ShowText`）、`vn-runtime/src/runtime/executor.rs`
- Host 执行与音频：`host/src/command_executor/ui.rs`、`host/src/app/command_handlers/audio.rs`、`host/src/audio/mod.rs`
- 工具链：`tools/xtask/src/main.rs`（`script-check` / `voice-index`）
 
### 阶段 29：UI 实现规范化（Theme 系统 + 组件库 + 个性化定制） 规划中

> **目标**：把当前“手绘式 UI”升级为**可主题化、可复用、可定制**的 UI 系统；统一视觉规范（颜色/字体/间距/圆角/阴影/动效），让 Title/Menu/SaveLoad/Settings/History 等页面在不改业务逻辑的前提下可整体换肤与局部定制。

**交付**：
- **Theme 规范化（Design Tokens）**：定义稳定的主题结构（如 `Palette/Typo/Spacing/Radii/Shadows`），提供默认主题（Dark/Light）与扩展点（游戏自定义主题）
- **UI 皮肤素材（Skin Assets）管线**：UI（对话框/按钮/图标/面板等）由开发者提供图片素材，我们按“皮肤配置 + 组件模板”自动搭建页面  
  - 约定资源目录：`ui/`（如 `ui/dialog/`、`ui/icons/`、`ui/buttons/`、`ui/panels/`）  
  - 皮肤配置：`ui_skin.json`（或并入 `ui_theme.json`），定义图标映射、对话框/面板九宫格（9-slice）切片、按钮状态图（normal/hover/pressed/disabled）、可选动画帧/时长  
  - 回退策略：缺失素材时使用默认皮肤/占位符并给出诊断；保证“没配全也能跑”，但发布门禁可要求齐全
- **样式 API 收敛**：组件不再散落硬编码颜色与尺寸，改为从 `UiContext.theme`/tokens 取值；统一状态样式（normal/hover/pressed/disabled/focus）
- **组件库升级**：梳理并补齐基础组件（Button/List/Panel/Modal/Toast/Scroll/Slider/Toggle/Tab），统一交互与布局约定，减少页面层重复绘制代码
- **布局与排版一致性**：统一字体加载/回退、字号层级、文本测量与裁剪省略；统一 DPI/分辨率适配策略（同一套布局在 720p/1080p 下可用）
- **个性化/定制入口**：提供 `ui_theme.json`（或 `config.json.ui`）加载主题覆盖；玩家侧提供 `UserSettings` 的 UI 偏好（如字体大小/UI 缩放/高对比度开关）
- **文档与示例**：新增 UI 规范文档（tokens/组件用法/页面模板），给内容作者一个“如何换肤”的最短路径

**DoD（摘要）**：
- 页面与组件的颜色/间距/圆角不再硬编码，全部来自 theme/tokens
- 主题切换不改业务逻辑（仅替换 theme 配置或加载覆盖）
- 能仅通过提供 `ui/` 素材 + `ui_skin.json` 完成对话框/按钮/图标等基础皮肤替换；缺失项能被静态检查/运行时诊断定位
- 至少完成 2 个页面（建议 Title + Settings）的组件化重写作为样板，并能复用到其余页面

**关键入口（预期）**：
- 主题与上下文：`host/src/ui/{theme.rs,mod.rs}`、`host/src/app/mod.rs`（`UiContext`）
- 组件库：`host/src/ui/{button.rs,list.rs,modal.rs,panel.rs,toast.rs}`（后续新增 slider/toggle/tab/scroll）
- 页面：`host/src/screens/{title.rs,settings.rs,save_load.rs,history.rs,ingame_menu.rs}`
- 资源与校验：`host/src/resources/*`、`tools/xtask/src/main.rs`（可扩展 `ui-check`）
 
---

## 开发原则

1. **遵循 ARCH.md 约束**
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
