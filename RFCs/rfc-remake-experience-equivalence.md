# RFC: ref-project 重制体验等价计划

## 元信息

- 编号：RFC-002
- 状态：Active
- 作者：Ring-rs 开发组
- 日期：2026-03-07（最后更新：2026-03-10）
- 相关范围：`assets/scripts/remake/`、`vn-runtime`、`host`、`docs/script_syntax_spec.md`

---

## 0. 阶段对齐记录

### 0.1 阶段 0 对齐结论（2026-03-07）

> 目标：先保证"文档状态 = 仓库现状"，避免实现计划漂移。

- `assets/scripts/remake/ring/` 主线章节语义稿已齐备（summer 12、winter 10）。
- `show/hide` 收敛与 Effect capability 注册表已落地（见 `rfc-show-unification-ergonomics` 与 `rfc-extension-api-mod-effect-management`）。
- `assets/scripts/remake/main.md` 已切到 Ring 章节编排，但仍依赖未实现的 `callScript` / `returnFromScript`，当前属于"目标调度稿"。
- `sceneEffect` 在脚本侧已有使用，但运行时 capability 尚未形成首批稳定契约。

### 0.2 阶段 1 对齐结论（2026-03-10）

> 目标：核查各 P0 子项的实际落地情况，重新规划剩余工作。

**主要发现**：

- `callScript` / `returnFromScript` 及调用栈已完整实现，跨文件调度闭环。
- 基础演出能力（dissolve/fade/fadewhite/rule_mask + show/hide/move）已实现，Host capability 注册表已接入。
- `wait` 等待指令已实现（定时/点击打断/Skip 跳过）。
- 窗口显隐控制（`textBoxHide/textBoxShow/textBoxClear`）已实现。
- 存档/读档系统已完整实现（含版本兼容、元数据、槽位管理）。
- 主菜单、设置、历史、存档/读档 UI 页面均已实现。
- **核心缺口**：`sceneEffect` 命令尚未解析执行；节奏标签 `{w}/{nw}/{cps}`、`pause/extend` 未支持。持久化域（`persistent.*`）与 `fullRestart` 已于 2026-03-10 完成。

### 0.3 阶段 2 对齐结论（2026-03-10）

> 目标：P0 剩余核心缺口首批实施。

**本轮完成**：

- `sceneEffect` 命令全链路（AST/Parser/Command/Executor/Host/Renderer）已完成。
  - 首批 capability：`effect.scene.shake`（shakeSmall/shakeVertical/bounceSmall）、`effect.scene.blur`（blurIn/blurOut）、`effect.scene.dim`（dimStep/dimReset）。
  - 渲染层：shake 偏移、blur 近似、dim 遮罩已实现。
  - 有 duration 的效果自动进入 `WaitForSignal("scene_effect")`，动画完成后自动解除。
  - Skip 模式下场景效果自动跳过。
- `pause` 指令全链路已完成（纯点击等待，复用 `WaitForClick`）。
- `titleCard` 命令全链路已完成（全屏字卡 + 淡入淡出 + `WaitForSignal("title_card")`）。
- 新增 13 个单元测试覆盖解析与执行。
- **剩余缺口**：节奏标签与 `extend` 台词续接未支持；镜头类高级效果（focusPush/panRight/pushIn 等）留待 P1-1。

### 0.4 阶段 3 对齐结论（2026-03-10）

> 目标：节奏标签与 extend 台词续接。

**本轮完成**（详见 `RFCs/rfc-rhythm-tags.md`）：

- 节奏标签全链路（AST/Parser/Command/Executor/Host 打字机）已完成。
  - 语义化语法：`{wait}`/`{wait Ns}` 内联等待、`-->` 行尾自动推进、`{speed N}`/`{speed Nx}`/`{/speed}` 字速控制。
  - 内联标签解析器 `inline_tags.rs`（11 个单元测试），输出纯文本 + `Vec<InlineEffect>` 位置索引。
  - Host 打字机扩展：inline_wait 状态、effective_cps 覆盖、no_wait 自动推进（Normal/Auto/Skip 三模式兼容）。
  - 点击行为分级：inline 点击等待时跳过当前等待点（不完成全部文本）。
- `extend` 台词续接全链路已完成（AST + Parser + Command + Executor + Host 打字机续接 + 历史追加）。
- **剩余缺口**：镜头类高级效果留 P1-1；脚本内容迁移（.rpy -> .md 节奏标签适配）为独立任务。

---

## 1. 背景

本 RFC 定义 ref-project 重制的体验复刻标准与实施优先级。
目标是复刻原作的玩家体验，而不是复刻 Ren'Py 的实现细节。

---

## 2. 目标与非目标

### 2.1 目标

- 叙事等价：剧情顺序、分支结果、章节解锁与结局触发一致。
- 演出等价：核心镜头语言一致（转场节奏、角色入退场、黑白场切换、关键 CG 时机）。
- 交互等价：玩家操作路径一致（主菜单/设置/历史/存读档/快进自动）。
- 听感等价：BGM/SFX/语音（如保留）的触发时机与层次一致。

### 2.2 明确非目标

- 不做 Ren'Py 全语法、全 API 兼容层。
- 不要求 Ren'Py transform/transition 内部算法完全一致。
- 不要求 GUI 像素级一致（保留美术风格与信息层级即可）。
- 不复刻开发者工具链行为（控制台、调试快捷键等）。
- 不因兼容旧实现破坏当前引擎分层与可维护性。

---

## 3. 当前基线（2026-03-10 实测）

### 3.1 已完成

- [x] 剧情相关 `rpy` 已搬迁至 `assets/scripts/remake/{summer,winter}`。
- [x] 主流程已在 `assets/scripts/remake/main.md` 编排，优先保证剧情与事件时序正确。
- [x] 未实现能力与实现计划已并入 `assets/scripts/remake/README.md`。
- [x] `callScript` / `returnFromScript` + 调用栈管理（跨文件脚本调度完整闭环）。
- [x] 基础演出能力闭环：dissolve、fade、fadewhite、rule_mask；show/hide/move 效果。
- [x] `wait` 等待指令（定时/点击打断/Skip 跳过）。
- [x] 窗口显隐控制（`textBoxHide/textBoxShow/textBoxClear`）。
- [x] 存档/读档系统（含版本兼容、元数据、槽位 UI）。
- [x] 核心 UI 页面：主菜单、设置、历史、存档/读档、游内菜单。
- [x] Host 侧 capability 注册表与效果扩展 API 框架。
- [x] 全局持久化域（`$persistent.key` / `saves/persistent.json` / `fullRestart`）与 `complete_summer` 章节门控。

### 3.2 运行稿说明

- 当前 `assets/scripts/remake/ring/` 为重制语义稿。
- 命令体系按"人体工学优先、引擎吸收复杂度"推进（详见 `RFCs/rfc-show-unification-ergonomics.md`）。
- 脚本中使用了尚未实现的 `cutscene` 等命令，这些命令在运行时会被忽略（不中断流程）。`sceneEffect` 与 `titleCard` 已于 2026-03-10 实现。

---

## 4. 分阶段计划（P0/P1/P2）

### 4.1 P0：先保证"能完整玩且感觉对"

#### P0-1 跨文件脚本调度

- 状态：**✅ 已完成**
- [x] `callScript` / `returnFromScript`（含 call stack）
- [x] 禁用跨文件 `goto`（不做全局 label 命名空间索引）
- [x] 非入口脚本 EOF 自动 return，入口文件 EOF 自动结束

#### P0-2 演出能力最小闭环

- 状态：**进行中**（首批 sceneEffect/titleCard 已落地，高级镜头类与组合编排待补）
- [x] 提供统一转场描述能力（替代 `tran_*` 家族）并接入 capability 路由。
- [x] 覆盖基础高频视觉类型：dissolve、黑场过渡、rule/mask。
- [x] **`sceneEffect` 首批 capability**（shake/blur/dim 解析执行渲染全链路，2026-03-10）。
- [x] **`titleCard` 命令支持**（全屏字卡 + 淡入淡出，2026-03-10）。
- [ ] 高级镜头效果（focusPush/panRight/pushIn/skyPan/imageWipe 等，留 P1-1）。
- [ ] 支持组合演出编排（如：黑场 + Pause + 反向开场）。
- [ ] 验收：`prologue`、`3-5`、`3-7`、`ending` 关键段落观感一致。

#### P0-3 文本节奏与窗口控制

- 状态：**进行中**（`wait`/`pause` 与窗口显隐已落地，节奏标签与 `extend` 待补）
- [x] 支持 `wait` 等待指令（到期自动推进、点击打断、Skip 模式跳过）。
- [x] 支持脚本驱动窗口显隐基础策略（`textBoxHide/textBoxShow/textBoxClear`）。
- [x] **`pause` 指令支持**（纯点击等待，复用 WaitForClick，2026-03-10）。
- [ ] **节奏标签支持**（`{w}`/`{nw}`/`{cps}` 等高频标签的语义化替代或兼容）。
- [ ] `extend`（续接台词，不清屏追加）语义支持。
- [ ] 验收：关键台词段落阅读节奏与原作接近。

#### P0-4 持久化与章节门控

- 状态：**✅ 已完成**（2026-03-10）
- [x] 存档/读档系统（槽位、元数据、版本兼容）。
- [x] **建立全局持久化域**（`$persistent.key` 命名空间，`saves/persistent.json`，严格双域隔离）。
- [x] **`fullRestart` 等价流程**（持久化 persistent_variables → 清空会话 → 返回标题）。
- [x] **对齐 `complete_summer` 行为**（`main.md` 已切换至 `$persistent.complete_summer`，首通门控逻辑完整）。
- [x] 验收：首通后重启，菜单与章节状态正确（persistent.json 权威，读档时覆盖）。

#### P0-5 核心系统 UI 可用

- 状态：**进行中**（基础页面已实现，快进/自动/跳过稳定性待收尾）
- [x] 主菜单、设置、历史、存档/读档页面基本可用。
- [ ] **快进/自动/跳过模式全流程稳定性验证**（含 wait/pause 的正确跳过行为）。
- [ ] 优先保障信息架构与流程一致，不逐条复刻 Ren'Py screen。
- [ ] 验收：从标题进入任意章节并完成一次存读档闭环。

### 4.2 P1：增强"像原作"的主观感受

#### P1-1 镜头语言高级能力

- [ ] `sceneEffect` 高级 capability 扩展（camera zoom/blur/shake/parallel 等）。
- [ ] 构建常用镜头预设（冲击镜头、模糊进出、推拉等）。
- [ ] 验收：抽查 4 个章节，关键镜头主观一致。

#### P1-2 角色演出语义层

- [ ] 抽象角色入场/退场/换表情/镜头位置语义（不照搬 Ren'Py transform）。
- [ ] 构建常用镜头预设（单人/双人/三人站位与近远景）。
- [ ] 验收：抽查 4 个章节，角色镜头主观一致。

#### P1-3 音频高级体验

- [ ] `bgmDuck` / `bgmUnduck` 正式实现（BGM 临时压低与恢复）。
- [ ] 支持 BGM 平滑暂停/恢复与跨段衔接。
- [ ] 三通道混音（music/sound/voice）与设置页联动。
- [ ] 验收：关键段（如 `prologue`）BGM 情绪曲线一致。

#### P1-4 视频与终章表现

- [ ] `cutscene` 命令：支持过场视频播放、跳过与播放后流程回归。
- [ ] 验收：`ending` 视频段完整可用，跳过不破坏剧情状态。

### 4.3 P2：长期可维护与生产效率

#### P2-1 复刻检查器（体验视角）

- [ ] 新增专项静态检查：高风险演出段漏映射、节奏标签丢失、资源缺失、门控状态异常。

#### P2-2 内容生产规范

- [ ] 形成重制脚本规范（命名、章节切分、演出标注）。
- [ ] 建立"转换后脚本 + 演出回放样例"的回归样本库。

#### P2-3 视觉一致性迭代

- [ ] UI 风格持续贴近原作（按钮、字体、层次、动效节奏）。
- [ ] 保持现有引擎架构清晰，不回退到 Ren'Py 风格耦合实现。

---

## 5. 建议实施顺序

1. 先打通可玩版本：P0
2. 再拉齐观感与听感：P1
3. 最后做规模化与长期维护：P2

---

## 6. P0 执行看板（2026-03-10 更新）

### 6.1 当前优先级排序

优先级基于"阻塞性 > 可玩闭环 > 观感补齐"原则：

| 优先级 | 子项 | 关键任务 | 状态 |
|--------|------|---------|------|
| 1 | P0-2 演出闭环 | `sceneEffect` 首批 capability（解析 + 执行） | 未开始 |
| 2 | P0-2 演出闭环 | `titleCard` 命令支持 | 未开始 |
| 3 | P0-3 节奏控制 | `pause` 指令 + `{w}`/`{nw}` 节奏标签 | 未开始 |
| 4 | P0-3 节奏控制 | `extend` 台词续接语义 | 未开始 |
| 5 | P0-5 核心 UI | 快进/自动/跳过稳定性收尾验证 | 进行中 |

### 6.2 已完成项（本轮确认）

| 子项 | 完成情况 |
|------|---------|
| P0-1 跨文件调度 | `callScript`/`returnFromScript`/调用栈 完整落地 |
| P0-2 基础效果 | dissolve/fade/fadewhite/rule_mask/move 已实现 |
| P0-3 wait | `wait` 指令完整实现（含 Skip 跳过） |
| P0-3 窗口显隐 | `textBoxHide/textBoxShow/textBoxClear` 已实现 |
| P0-4 存读档基础 | 单次会话存档/读档闭环完整 |
| P0-4 持久化域 | `$persistent.key` 双域严格隔离、`saves/persistent.json`、`fullRestart` 完整实现（2026-03-10） |
| P0-4 章节门控 | `main.md` 切换至 `$persistent.complete_summer`，首通后门控逻辑完整（2026-03-10） |
| P0-5 基础 UI | 主菜单/设置/历史/存档/读档/游内菜单已实现 |

### 6.3 里程碑 DoD（Definition of Done）

- **M1（可玩主线）**：从 `main.md` 单入口跑通 summer → winter 主流程，不手动切脚本 ← *P0-1/P0-4 已完成，主线调度闭环*
- **M2（关键观感）**：`prologue`、`3-5`、`3-7`、`ending` 的关键镜头通过抽样回放验收 ← *依赖 P0-2 sceneEffect*
- **M3（门控闭环）**：首通后重启，章节入口和菜单状态正确，且可存读档回归 ← **✅ P0-4 已完成，可手动验收**
