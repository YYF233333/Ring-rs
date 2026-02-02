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

### 里程碑摘要（阶段 1-10）
- **基础架构**：主循环/渲染/输入/资源管理跑通，Runtime/Host 分离落地
- **渲染与输入**：背景/立绘/对话/选择/章节标记 + 打字机效果 + 输入防抖
- **演出与音频**：dissolve/fade/fadewhite/rule(ImageDissolve) + rodio 音频（BGM/SFX/淡入淡出/切换）
- **质量与路径治理（阶段10）**：端到端脚本验证 + 统一 `std::path` 规范化解决 `../` 资源键不一致

---

## 阶段 11：脚本语法补齐 + 立绘资源/布局元数据系统 ✅ 已完成

> 主题：**脚本语法补齐（音频/控制流）** + **立绘资源/布局元数据系统**

### 11.1 脚本语法补齐：音频/控制流 ✅

- **完成摘要（保留可复用规则）**
  - 音频语法：`<audio src="..."></audio>`（SFX 一次） / `<audio src="..."></audio> loop`（BGM 循环） / `stopBGM`
  - 控制流语法：`goto **label**`（无条件跳转）
  - 路径规则：素材路径 **相对脚本文件目录**，并统一 `std::path` 规范化（解决 `../` cache key 不一致）
  - Host 音频策略：BGM 切换带交叉淡化（1s），脚本层不做音量/静音（交给玩家设置）
- **关键落点**
  - `vn-runtime/src/script/{ast.rs,parser.rs}`
  - `vn-runtime/src/runtime/executor.rs`
  - `host/src/audio/mod.rs`

### 11.2 资源管理（立绘）：anchor + pre_scale + preset 布局系统 ✅

- **完成摘要（保留可复用规则）**
  - `manifest.json` 提供：`anchor`（对齐点）、`pre_scale`（预缩放）、`presets`（站位预设）、`defaults`（兜底）
  - 查找顺序：显式映射 → 路径推导 → 默认配置（保证“缺元数据也可用”）
  - 渲染端按 `anchor + pre_scale + preset` 计算立绘位置与缩放，避免硬编码适配每张立绘
- **关键落点**
  - `assets/manifest.json`
  - `host/src/manifest/mod.rs`
  - `host/src/renderer/mod.rs`（角色渲染）

---

## 阶段 12：架构性改动 ✅ 已完成

> 主题：**先做会影响项目结构的部分**

### 12.x 完成摘要 ✅
- **存档/读档**：`SaveData/SaveVersion/SaveMetadata/AudioState/RenderSnapshot` + Host `SaveManager`（slot_XXX.json）
- **历史**：`HistoryEvent` + `History` 容器，随存档持久化，Runtime tick/input 自动记录
- **配置治理**：Host `AppConfig` + `config.json`，Manifest 校验（warnings/友好提示）
- **关键落点**
  - `vn-runtime/src/{save.rs,history.rs}`
  - `host/src/{save_manager/mod.rs,config/mod.rs,manifest/mod.rs}`

---

## 阶段 13：测试覆盖率 + 文档水平提升 ✅ 已完成

> 主题：**在大改动完成后集中提升质量**

### 13.x 完成摘要 ✅
- **测试**：补齐 parser/engine 的关键回归（goto/audio/stopBGM/相对路径/历史/恢复/分支），总测试数达标（以仓库为准）
- **文档**：README + manifest/save 格式说明同步更新
- **关键落点**
  - `vn-runtime/src/script/parser.rs`（测试）
  - `docs/{manifest_guide.md,save_format.md}`

---

## 阶段 14：结构性完善 ✅ 已完成

> 主题：**配置落地 + 存档/历史完善 + 资源治理**

### 14.x 完成摘要 ✅
- **Host 配置落地**：移除硬编码路径/窗口设置，改由 `config.json` 驱动；脚本目录动态扫描
- **存档可用性**：多 slot（后续 UI 替代快捷键），补齐 `play_time_secs` 等元信息
- **历史可用性**：输入/跳转等事件补齐记录（ChoiceMade/Jump/Goto 等）
- **资源治理**：路径规范化与 Manifest 校验覆盖
- **关键落点**
  - `host/src/main.rs`（配置/脚本扫描）
  - `host/src/resources/mod.rs`（路径/缓存键）

---

## 阶段 15：演出系统重构（changeBG/changeScene + Rule/ImageDissolve）✅ 已完成

> 主题：**收敛演出语义**，把“简单背景切换”和“复合场景切换（遮罩/清场/UI 时序）”彻底分离，确保脚本语义清晰且 Host 侧实现稳定可控。

### 15.x 完成摘要（浓缩）✅
- **职责分离**
  - `changeBG`：只做“简单背景切换”（立即 / dissolve）
  - `changeScene`：复合演出（UI 隐藏、清立绘、遮罩过渡、UI 恢复）
- **Rule/ImageDissolve**
  - Rule 为 **两段式**：旧背景→黑屏 →（黑屏停顿 0.2s）→ 黑屏→新背景；UI 在结束后 0.2s 淡入
  - Rule 无 fallback：缺 shader/缺纹理直接 `panic!`（避免静默降级）
- **路径规则**
  - `rule.mask` 按“相对脚本目录”解析后再送 Host，Host 侧资源加载也统一规范化

### 15.2 过渡命名参数（named args）✅

- **数据结构**
  - `Transition.args: Vec<(Option<String>, TransitionArg)>`（`None`=位置参数，`Some(key)`=命名参数）
- **语法规则（保留可复用信息）**
  - 位置参数：`Name(1.0, true, "x")`
  - 命名参数：`Name(duration: 1.0, reversed: true, mask: "rule.png")`
  - **禁止混用**：一次调用内只能全位置或全命名
  - key：标识符；value：Number/Bool/String；重复 key 报错；格式错误/括号错误报错（需可定位）
- **读取策略**
  - Host 读取时：命名参数优先，位置参数回退（确保兼容旧脚本）

### 15.3 稳定性修复与验收 ✅

- 修复 Rule phase 之间的闪帧（遮罩保持全覆盖 + 增加黑屏停顿）
- 修复 Rule phase 2 读取 `pending_background` 导致的崩溃（phase 2 使用 `current_background` 作为新背景来源）
- 端到端 演示脚本已更新：`assets/scripts/test_comprehensive.md`
- 测试通过：**101 个（host 24 + vn-runtime 77）**

### 15.4 `changeScene` 语法要点（归档）✅
- `changeBG`: 立即 / `with dissolve` / `with Dissolve(duration: N)`（不支持 fade/fadewhite，迁移到 changeScene）
- `changeScene`: 必须带 `with`，支持：
  - `with Dissolve(duration: N)`
  - `with Fade(duration: N)` / `with FadeWhite(duration: N)`
  - `with <img src="rule.png"/> (duration: N, reversed: bool)`
- Rule 说明：灰度遮罩按像素亮度控制溶解顺序（Ren'Py ImageDissolve 风格）

---

## 阶段 16：玩家 UI / 体验增强（主界面入口 + 存读档/设置/历史 UI + 测试入口重构）✅ 已完成

> 主题：把“可玩”从 **开发者快捷键驱动**升级为 **玩家 UI 驱动**，并收敛入口与状态机。

### 阶段 16 完成摘要（浓缩）
- **入口与状态机**
  - 新增 `AppMode` + `NavigationStack`：Title / InGame / InGameMenu / SaveLoad / Settings / History
  - 启动进入 **Title**（不再有 Demo/Command 模式）
- **UI 基建**
  - `host/src/ui/`：Theme / Button / Panel / List / Modal / Toast
  - `host/src/screens/`：Title / InGameMenu / SaveLoad / Settings / History
- **玩家路径（不靠快捷键也可闭环）**
  - Title：开始 / 继续（若有存档）/ 读档 / 设置 / 退出
  - InGame：ESC 打开系统菜单 → 存/读/设/史/返回标题/退出
  - Settings：保存 `user_settings.json`（玩家配置，不污染 `config.json`）
- **测试与回归**
  - Host 侧移除演示/命令入口分支，统一走 Title → Start/Load
  - 编译与测试通过（以当前仓库测试为准）

### 阶段 16 关键落点（文件）
- `host/src/app_mode.rs`
- `host/src/ui/*`
- `host/src/screens/*`
- `host/src/main.rs`

---

## 阶段 17：玩家体验打磨 + 存档信息完善 + 文档/发布整理 ✅ 已完成

> 主题：把阶段16的“能用”打磨成“可交付”：信息完整、流程稳定、文档一致、发布可复现。

### 17.1 存档信息与"继续"逻辑完善 ✅
- **目标**
  - SaveLoad 列表显示真实元信息：时间戳 / 游玩时长 / 脚本ID / 章节标题（已有字段则直接展示）
  - Title “继续”不使用通用存档槽位；改为读取一个**专用 Continue 存档**：
    - 在玩家退出/返回标题时维护（记录当前位置/脚本进度）
    - 下次点击“继续”从该位置恢复
  - SaveLoad 支持 1-99（滚动/分页均可），避免硬编码 20 个槽位
- **验收标准**
  - 存档列表与 Title “继续”在多存档情况下行为可预期、无歧义
  - 坏档/缺档/旧档：UI 提示清晰、不崩溃
  - Continue 存档不存在时：“继续”置灰或提示（不回退到任意 slot）

### 17.2 游戏开始入口配置化 ✅
- **目标**
  - `config.json` 直接指定默认入口脚本 **path**（例如 `start_script_path`）
  - 未指定入口脚本：**直接 panic**（强制工程显式配置）
- **验收标准**
  - “开始游戏”不依赖源码常量，可通过配置切换入口脚本

### 17.3 UI/输入一致性与边界修复 ✅
- **目标**
  - 菜单/历史/存读档打开时：屏蔽剧情推进与选择输入（避免双重消费）（已完成，可作为回归项）
  - 返回标题：清理 UI 状态、停止/淡出音频、重置必要状态
  - Toast/Modal 统一风格与行为（ESC/Enter 的语义一致）
- **验收标准**
  - 快速连续操作（打开菜单→读档→返回→继续）不出现“错状态/卡死/误触发”

### 17.4 文档与发布整理 ✅
- **目标**
  - README：移除已废弃的 Demo/Command/旧快捷键说明，改为 UI 入口说明
  - `docs/save_format.md`：补齐与 UI 展示相关的字段说明（timestamp/play_time/chapter）
  - 给出最小发布/运行说明（Windows 路径/配置/资源目录）
- **验收标准**
  - 新用户仅看 README + config.json 示例即可跑起来并完成“开始→存档→读档”

---

## 阶段 17 完成总结

**实现内容**

1. **Continue 专用存档**
   - 新增 `saves/continue.json`，在返回标题/退出时自动保存
   - Title "继续"按钮读取专用存档（不存在则置灰）
   - SaveManager 新增 `save_continue`, `load_continue`, `has_continue` 方法

2. **SaveLoad 列表改进**
   - 显示真实元信息：章节标题、格式化时间戳、游玩时长
   - 支持 1-99 槽位（不再硬编码 20）
   - 时间戳格式化为可读格式（`YYYY-MM-DD HH:MM`）

3. **配置化入口脚本**
   - 新增 `config.json` 的 `start_script_path` 字段（必填）
   - 未配置时启动 panic（强制显式配置）
   - 移除基于脚本扫描的隐式入口逻辑

4. **状态清理与音频管理**
   - 返回标题时：停止音频（0.5s 淡出）、清理 RenderState、保存 Continue
   - 开始新游戏时：删除旧 Continue 存档
   - 确保资源状态一致性

5. **文档更新**
   - README：移除 Demo/Command 模式，更新操作说明为 UI 驱动
   - save_format.md：补充 Continue 存档说明、元数据字段详解
   - 明确配置要求和运行流程

**验收情况**

- ✅ Continue 存档逻辑正常（不存在时"继续"置灰）
- ✅ SaveLoad 显示详细信息（时间戳/时长/章节）
- ✅ 1-99 槽位全部可用（滚动列表）
- ✅ 未配置 start_script_path 时 panic（已验证）
- ✅ 返回标题/退出时音频正确停止并保存 Continue
- ✅ 新用户可根据 README + config.json 成功启动并使用存档系统
- ✅ 所有测试通过（29 passed; 0 failed）

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
