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

## 开发历程总结

### 阶段 1-2：基础框架 + 资源管理
- 窗口/主循环/调试信息
- `ResourceManager` 负责图片与音频资源加载、缓存与路径解析
- 支持 PNG 和 JPEG 图片格式

### 阶段 3：渲染系统
- 背景、角色、对话框、选择分支 UI 渲染
- 中文字体加载（simhei.ttf）
- 打字机效果

### 阶段 4：输入处理
- `InputManager` 统一采集鼠标/键盘输入
- WaitingReason 驱动的输入分发与防抖

### 阶段 5：Command 执行器
- 完整 Command 分发与 RenderState 更新
- 过渡效果与音频命令的执行管线

### 阶段 6：Runtime 集成
- Script 模式接入 `VNRuntime` 与 `Parser`
- Demo/Command/Script 三模式切换

### 阶段 7：过渡效果
- dissolve/fade/fadewhite 过渡支持
- 背景切换时自动应用过渡

### 阶段 8：音频系统
- `AudioManager` 采用 `rodio`，支持 MP3/WAV/FLAC/OGG
- BGM 循环/淡入淡出/切换，SFX 播放与音量控制

### 阶段 9：UI 完善
- 选择分支支持鼠标悬停高亮与点击选择
- 章节标题移至左上角，避免遮挡内容

### 阶段 10：测试与优化 ✅
- 创建完整功能测试脚本（test_comprehensive.md）
- 端到端功能验证：背景/角色/对话/选择分支/章节标题
- 修复 JPEG 图片加载问题（添加 image crate 支持）
- 修复选择分支后选项不消失的 bug
- 修复脚本资源路径：素材路径**相对于脚本文件**解析（便于 Typora 预览）
- 修复路径 edge case：统一使用 `std::path` 规范化路径，解决 `../` 导致的纹理缓存键不一致/背景不显示问题
- 文档更新

---

## 阶段 11：脚本语法补齐 + 立绘资源/布局元数据系统 ✅ 已完成

> 主题：**脚本语法补齐（音频/控制流）** + **立绘资源/布局元数据系统**

### 11.1 脚本语法补齐：音频/控制流 ✅

- **已完成**
  - `<audio src="..."></audio>`：SFX 播放一次
  - `<audio src="..."></audio> loop`：BGM 循环播放
  - `stopBGM`：停止当前 BGM（带淡出）
  - `goto **label**`：无条件跳转
  - 资源路径解析规则调整：素材路径相对于脚本文件，且路径已规范化（`std::path`）
  - BGM 切换自带交叉淡化效果（1秒）
  - 错误处理：文件不存在/格式不支持时打印错误但不崩溃
  - ~~音量/静音~~：由玩家在设置中控制，脚本层不实现
- **验收标准** ✅
  - 脚本可完整覆盖：播放 BGM → 切换 BGM（自动交叉淡化）→ 播放 SFX → stopBGM

### 11.2 资源管理（立绘）：anchor + pre_scale + preset 布局系统 ✅

- **背景**：立绘尺寸/构图不统一，无法用单一缩放规则保证"站位稳定/构图一致"。
- **核心想法**：为**每组立绘**提供可配置的 `anchor`（重心/对齐点）与 `pre_scale`（预处理缩放），再叠加**全局 preset**（点位 scale/偏移），使不同立绘在屏幕上呈现一致的相对效果。

- **已完成**
  - `assets/manifest.json` 资源清单：
    - `characters.groups`: 立绘组配置（anchor + pre_scale）
    - `characters.sprites`: 立绘路径到组的显式映射
    - `presets`: 九宫格站位预设（x, y, scale）
    - `defaults`: 默认配置（未配置立绘的兜底）
  - `host/src/manifest/mod.rs` 模块：
    - Manifest 数据结构定义（serde 序列化/反序列化）
    - `get_group_config()`: 查找顺序 = 显式映射 → 路径推导 → 默认配置
    - `infer_group_id()`: 兜底规则（目录名 / 文件名前缀）
    - 4 个单元测试覆盖核心场景
  - `Renderer::render_characters()` 重写：
    - 基于 anchor + pre_scale + preset 计算位置和缩放
    - 删除旧的硬编码 `position_to_screen_coords()` / `scale_character_size()`

- **验收标准** ✅
  - 元数据缺失时有合理默认值（不崩溃，可用）
  - 新立绘组只需编辑 manifest.json，无需改代码

---

## 阶段 12：架构性改动 ✅ 已完成

> 主题：**先做会影响项目结构的部分**

### 12.1 存档/读档系统 ✅

- **实现内容**
  - `vn-runtime/src/save.rs`：定义 `SaveData`、`SaveVersion`、`SaveMetadata`、`AudioState`、`RenderSnapshot`
  - 版本兼容策略：major 版本必须一致，minor 可不同
  - `host/src/save_manager/mod.rs`：存档文件布局 `saves/slot_XXX.json`，读写 API
  - 快捷键：F5 保存，F9 读取（后续将以 UI 替代，快捷键仅 dev-only，见阶段 16）
  - `VNRuntime::restore_state()` 支持状态恢复

### 12.2 历史记录数据模型 ✅

- **实现内容**
  - `vn-runtime/src/history.rs`：定义 `HistoryEvent`（Dialogue/ChapterMark/ChoiceMade/Jump/BackgroundChange/BgmChange）
  - `History` 容器，支持最大事件数限制、序列化
  - `VNRuntime` 在 tick 时自动记录历史
  - 历史数据随存档持久化

### 12.3 资源与配置治理 ✅

- **实现内容**
  - `host/src/config/mod.rs`：`AppConfig`（assets_root、saves_dir、window、debug、audio 配置）
  - `config.json` 配置文件，支持默认值和校验
  - `Manifest::validate()` 校验方法，检测无效锚点/预缩放/预设位置/引用不存在的组
  - `ManifestWarning` 警告类型，友好的错误提示

---

## 阶段 13：测试覆盖率 + 文档水平提升 ✅ 已完成

> 主题：**在大改动完成后集中提升质量**

### 13.1 测试覆盖率提升 ✅

- **完成内容**
  - parser：新增 goto/audio/stopBGM/相对路径测试（+14 个测试）
  - engine：新增历史记录、状态恢复、goto 跳转、选择分支测试（+5 个测试）
  - 总测试数：101 个（host 24 + vn-runtime 77）

### 13.2 文档质量提升 ✅

- **完成内容**
  - README：更新项目结构、快捷键说明、功能列表
  - `docs/manifest_guide.md`：manifest 配置完整指南
  - `docs/save_format.md`：存档格式与版本兼容说明

---

## 阶段 14：结构性完善 ✅ 已完成

> 主题：**配置落地 + 存档/历史完善 + 资源治理**

### 14.1 配置落地 ✅
- 移除 `main.rs` 中所有硬编码路径（assets_root、saves_dir、font_path、scripts）
- 窗口配置从 `config.json` 读取（width/height/title/fullscreen）
- 动态扫描 `assets/scripts/*.md` 作为脚本列表

### 14.2 存档系统完善 ✅
- 多 slot 管理：`[` / `]` 快捷键切换槽位（1-99）
- （后续将以 UI 替代，快捷键仅 dev-only，见阶段 16）
- 元信息完善：保存时记录游戏时长 `play_time_secs`
- UI 显示当前槽位

### 14.3 历史系统完善 ✅
- 在 `handle_input` 中记录 `ChoiceMade` 事件（包含所有选项文本和选择索引）
- 在 `tick` 中记录 `Goto` 跳转事件
- 选择分支跳转时同时记录 `ChoiceMade` + `Jump`

### 14.4 资源治理 ✅
- 资源 key 规范化（使用 `std::path` 归一化路径，已在阶段 10 完成）
- Manifest 校验已覆盖：anchor/pre_scale/preset 边界检测、sprite 引用检查

---

## 阶段 15：演出系统重构（changeBG/changeScene + Rule/ImageDissolve）✅ 已完成

> 主题：**收敛演出语义**，把“简单背景切换”和“复合场景切换（遮罩/清场/UI 时序）”彻底分离，确保脚本语义清晰且 Host 侧实现稳定可控。

### 15.1 `changeBG` / `changeScene` 职责分离 ✅

- **原则**
  - `changeBG`：只做简单背景切换（立即 / dissolve）
  - `changeScene`：统一承载复合演出（UI 隐藏、清立绘、遮罩过渡、UI 恢复）

- **Parser**
  - `changeBG` 限制只支持 `dissolve`（其他效果报错并提示迁移到 `changeScene`）
  - `changeScene` 强制要求 `with` 子句
  - 支持 `Fade(duration: N)` / `FadeWhite(duration: N)`
  - 支持 `with <img src="rule.png"/> (duration: N, reversed: bool)`（Rule 语法糖）

- **Runtime**
  - `Command::ChangeScene { path, transition }` 仍保持声明式
  - `rule` 的 `mask` 路径在发送到 Host 前会按“相对脚本目录”自动解析

- **Host**
  - Fade/FadeWhite：三阶段（遮罩淡入 → 遮罩淡出 → UI 淡入 0.2s）
  - Rule：四阶段（旧背景→黑屏 → 黑屏停顿 0.2s → 黑屏→新背景 → UI 淡入 0.2s）
  - Rule 使用 `ImageDissolve`（灰度遮罩像素亮度控制溶解顺序），并移除 fallback：缺资源/缺 shader 直接 `panic!`

### 15.2 过渡命名参数（named args）✅

- `Transition.args` 升级为 `Vec<(Option<String>, TransitionArg)>`
- 支持位置参数与命名参数两种写法，**禁止混用**
- Host 端读取参数时：命名参数优先，位置参数回退

### 15.3 稳定性修复与验收 ✅

- 修复 Rule phase 之间的闪帧（遮罩保持全覆盖 + 增加黑屏停顿）
- 修复 Rule phase 2 读取 `pending_background` 导致的崩溃（phase 2 使用 `current_background` 作为新背景来源）
- 端到端 演示脚本已更新：`assets/scripts/test_comprehensive.md`
- 测试通过：**101 个（host 24 + vn-runtime 77）**

### 15.4 `changeScene` 脚本语法与效果定义（归档）✅

#### 设计原则

- **changeBG**：简单背景切换（无复合流程）
  - 支持：无过渡（立即切换）、`dissolve`（交叉溶解）
  - **不再支持** `fade`/`fadewhite`（迁移到 `changeScene with Fade(...)`）
  
- **changeScene**：复合场景切换（涉及遮罩/清场/UI 时序）
  - 语义：UI 隐藏 → 遮罩覆盖 → 清立绘+换背景 → 遮罩消失 → UI 恢复
  - 支持效果：`Dissolve` / `Fade` / `FadeWhite` / `Rule`（图片遮罩）

#### 目标脚本语法

```markdown
# changeBG（简单背景切换）
changeBG <img src="bg.jpg"/>                      # 无过渡，立即切换
changeBG <img src="bg.jpg"/> with dissolve        # 交叉溶解
changeBG <img src="bg.jpg"/> with Dissolve(duration: 1.5)

# changeScene（复合场景切换）- 必须带 with
changeScene <img src="bg.jpg"/> with Dissolve(duration: 1)
changeScene <img src="bg.jpg"/> with Fade(duration: 1)
changeScene <img src="bg.jpg"/> with FadeWhite(duration: 1)
changeScene <img src="bg.jpg"/> with <img src="rule.png"/> (duration: 1, reversed: true)
```

#### changeScene 效果详解

| 效果 | 语法 | 遮罩颜色/类型 | 说明 |
|------|------|--------------|------|
| Dissolve | `with Dissolve(duration: N)` | 无遮罩 | UI隐藏 → 背景交叉溶解 → UI恢复 |
| Fade | `with Fade(duration: N)` | 纯黑 | UI隐藏 → 黑屏 → 换背景+清立绘 → 显现 → UI恢复 |
| FadeWhite | `with FadeWhite(duration: N)` | 纯白 | UI隐藏 → 白屏 → 换背景+清立绘 → 显现 → UI恢复 |
| Rule | `with <img src="rule.png"/> (...)` | 图片遮罩（ImageDissolve） | UI隐藏 → 旧背景→黑屏（按灰度顺序溶解） → 黑屏停顿 0.2s → 黑屏→新背景（反向溶解） → UI 淡入 0.2s |

---

## 阶段 16：玩家 UI / 体验增强（主界面入口 + 存读档/设置/历史 UI + 测试入口重构）✅ 已完成

> 主题：把"可玩"从 **开发者快捷键驱动**升级为 **玩家 UI 驱动**。该阶段只做 UI/流程与测试入口重构，避免引入新的脚本语法与演出系统变更。

### 16.1 应用流程（App Flow）重构：引入"主界面入口" ✅

- **目标**
  - 启动后进入 **主菜单（Title/MainMenu）**，提供明确入口：开始游戏 / 读档 / 设置
  - 游戏内提供 **系统菜单（InGameMenu）**：存档 / 读档 / 设置 / 历史 / 返回标题 / 退出
  - **不再依赖快捷键**进行存读档与设置入口（保留 debug hotkey 仅限 dev feature 或 debug build）

- **需要新增/明确的状态**
  - `AppMode`：`Title | InGame | InGameMenu | SaveLoad | Settings | History`
  - `Navigation`：统一返回栈（例如从 InGameMenu 打开 SaveLoad，返回仍回 InGameMenu）
  - `InputCapture`：菜单打开时屏蔽推进剧情/选择等输入，避免“双重消费”

- **验收标准**
  - 进入游戏、读档、打开设置均可通过 UI 完成
  - 不使用任何快捷键也能完成：开始→存档→退出到标题→读档→继续

### 16.2 主菜单（Title/MainMenu）✅

- **功能**
  - **开始游戏**：从配置指定的默认脚本/入口 label 启动（例如 `config.json` 增加 `start_script_id` / `start_label`，或在 Host 侧约定默认脚本）
  - **继续**：自动加载“最新存档”（按 `timestamp/play_time` 选择），若无则灰掉
  - **读档**：进入 Save/Load 界面（默认在 “Load” tab）
  - **设置**：进入 Settings
  - **退出**：优雅退出（保存设置）

- **验收标准**
  - “继续”在无存档时不会崩溃、不会误导（状态清晰）
  - 主菜单与游戏内菜单的视觉/交互一致（同一套 UI 组件）

### 16.3 游戏内系统菜单（InGameMenu）✅

- **功能**
  - 打开方式：UI按钮（例如右上角“菜单”）与可选键位（如 `Esc`）——但**存读档/设置功能不再只靠键位**
  - 菜单项：继续 / 存档 / 读档 / 设置 / 历史 / 返回标题 / 退出
  - “返回标题”默认不自动存档，但可给二次确认（避免误操作）

- **验收标准**
  - 菜单打开时剧情停止推进、选择不被误触发
  - 菜单关闭后输入恢复正常

### 16.4 存档 / 读档 UI（SaveLoad）✅

- **信息架构**
  - Slot 列表（1-99，可分页/滚动）
  - 每个 slot 显示：章节/脚本 id、时间戳、游玩时长、（可选）缩略图预览
  - 操作：保存（覆盖提示）、读取（确认）、删除、重命名（可延后）

- **与现有系统的对接原则**
  - 继续沿用 `SaveManager`、`SaveMetadata`、`play_time_secs`
  - 快速存读档快捷键（F5/F9）降级为 dev-only（计划：feature gate），正式流程只走 UI

- **验收标准**
  - 无存档/损坏存档/旧版本存档能给出可理解错误提示，不崩溃
  - 读档后脚本定位与渲染/音频/历史均正确恢复（回归现有验收）

### 16.5 设置 UI（Settings）✅

- **范围（玩家设置，不进脚本）**
  - 音量：BGM/SFX（含静音）
  - 显示：全屏/窗口、分辨率（可选）、UI 缩放/字体大小（可选）
  - 其他：文字速度/自动播放（可后置）

- **配置治理**
  - `config.json` 作为“默认配置/工程配置”
  - 新增 `user_settings.json`（或同类文件）保存玩家设置（覆盖默认但不污染工程配置）

- **验收标准**
  - 设置修改即时生效，并在重启后保留
  - 设置界面可从 Title 与 InGameMenu 双入口打开

### 16.6 历史回看 UI（History）✅

- **功能**
  - 面板显示 `HistoryEvent`（至少：对白/旁白/章节标记/选择项）
  - 支持滚动、按章节分段（可选）、按角色过滤（可选）
  - 不要求“点击回跳/回放”

- **验收标准**
  - 历史 UI 不影响当前游戏状态，不会导致推进/选择被触发
  - 与存档的历史数据一致（读档后历史可继续累积）

### 16.7 视觉与交互一致性（UI 基建）✅

- **组件化**
  - Button / Panel / List / Tabs / Modal(Confirm) / Toast
  - 统一布局与字体（与现有 TextRenderer 兼容）

- **交互**
  - 操作提示（例如保存成功 toast）
  - 危险操作二次确认（覆盖存档、返回标题、退出）

### 16.8 测试入口重构：移除"演示模式/命令模式" ✅

- **目标**
  - 测试不再依赖运行时的“演示模式/命令模式”分支（删除这些分支，逻辑回归到唯一入口：Title/MainMenu→Start/Load）
  - 建立可扩展的集成测试入口：以“场景驱动”模拟 UI 操作与脚本推进

- **建议方向（不强绑定实现）**
  - Host 侧抽象 `AppDriver` / `TestDriver`：可注入脚本、注入输入、读取 RenderState/RuntimeState 快照
  - 以“流程用例”替代“模式切换”：Start→Save→BackToTitle→Load→Continue
  - 回归覆盖：存档/读档、UI 可见性、转场、音频状态、历史累积

- **验收标准**
  - 不需要 demo/command 模式也能跑完端到端集成测试
  - CI 里测试稳定（无依赖交互式键盘输入）

---

### 附：脚本语法增强（已完成的实现要点归档）

1. **效果支持命名参数**（例：`Dissolve(duration: 1)`）
   - **目标**：在不破坏现有 `Dissolve(1.0)` / `Fade(0.5)` 位置参数写法的前提下，支持更可读、可扩展的命名参数写法。
   - **语法范围**：仅作用于 `with <effect_expr>` 的 `effect_expr`（即 `Transition` 解析），不改变其他指令结构。
   - **语法定义**
     - **位置参数（现有）**：`Name(1.0, 0.5, true, "x")`
     - **命名参数（新增）**：`Name(duration: 1.0, reversed: true, mask: "rule.png")`
     - 命名参数是为了支持用户乱序填写参数或者只填充部分参数，其余使用默认值的qol优化
     - **不允许混用**：同一次调用中**只能**全是位置参数或全是命名参数：
       - ✅ `Dissolve(1.0)`
       - ✅ `Dissolve(duration: 1.0)`
       - ❌ `Dissolve(1.0, duration: 1.0)`
   - **解析产物（兼容策略）**
     - `Transition` 的参数建议升级为“带可选名称的参数项列表”，避免在 `Vec<TransitionArg>` 里做隐式编码：
       - `args: Vec<(Option<String>, TransitionArg)>`
       - `None` 表示位置参数，`Some(key)` 表示命名参数
     - 示例：
       - `Dissolve(1.0)` → `[(None, Number(1.0))]`
       - `Dissolve(duration: 1.0)` → `[(Some("duration"), Number(1.0))]`
     - 位置参数保持原样：`(None, Number(1.0))`
   - **类型规则**
     - 命名参数 key 必须是标识符（建议：[a-zA-Z_][a-zA-Z0-9_]*）（同意）
     - value 支持 `Number` / `Bool` / `String` 
     - 同名 key 不允许重复（重复 → 报错并带行号）
   - **错误规则（必须可定位）**
     - 混用位置/命名参数 → ParseError（指出 effect 名与行号）
     - 命名参数缺少 `:` 或 value → ParseError
     - 命名参数 key 非法 / 重复 key → ParseError
     - 括号不匹配 / 逗号格式错误 → ParseError
   - **与现有语法的关系（落地建议）**
     - `Dissolve(duration: 1)` 视为 `Dissolve(1)` 的等价写法（Host 端读取 duration 时优先命名参数，再回退位置参数）
     - `changeScene ... with <img .../> (duration: 1, reversed: true)` 的括号参数也复用同一套“命名参数列表”解析器（避免两套语法/两套 bug）
   - **测试计划**
     - 单测覆盖：纯位置参数、纯命名参数、不允许混用、重复 key、非法 key、缺少冒号/值、字符串/布尔/数字三类值
     - 回归：现有脚本（含 `Dissolve(1.5)` / `with dissolve`）全部应保持通过
     - 说明：`rule("mask.png", 1.0, true)` 这种函数写法**不要求支持**；rule-based effect 统一走 `<img src="..."/>(args)` 语法糖
   - **验收标准**
     - `docs/script_syntax_spec.md` 增补命名参数语法与“不混用”规则
     - Parser 新增测试并通过
     - Host 对至少 `Dissolve/Fade/rule` 能正确读取 `duration/reversed`（命名优先，位置回退）

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
