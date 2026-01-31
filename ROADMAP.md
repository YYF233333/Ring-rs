# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划，遵循 PLAN.md 中的架构约束。

---

## 当前状态（Reality Check）

目前工程已经：

- ✅ `vn-runtime`：脚本解析 + Runtime tick 循环 + Command 生成完成，并有较完整单元测试
- ✅ `host`：可以启动 Bevy 窗口、加载脚本并创建 Runtime
- ⚠️ 但 **Host 层的“可视化 + 可交互”链路尚未真正打通**：
  - 画面未稳定显示（背景/人物资源缺失会导致看不到效果）
  - 输入未稳定驱动 Runtime（点击/空格/选择按钮到 `RuntimeInput` 的端到端闭环需要补齐验证）
  - 缺少“每条脚本指令都能看见效果”的逐条验收

因此：Phase 0-4 在“代码层”已完成，但在“产品可用性层面”需要进入下一阶段。

---

## 开发阶段总览

| 阶段 | 名称 | 目标 | 状态 |
|------|------|------|------|
| Phase 0 | 基础设施 | 类型定义、项目结构 | ✅ 已完成 |
| Phase 1 | 脚本解析器 | 完整的脚本解析能力 | ✅ 已完成 |
| Phase 2 | Runtime 核心 | 执行引擎与状态管理 | ✅ 已完成 |
| Phase 3 | Host 集成 | Bevy 渲染与交互骨架 | ✅ 已完成（骨架） |
| Phase 4 | 整合测试 | 能加载脚本并启动 | ✅ 已完成（最小跑通） |
| Phase 5 | 可交互 MVP+ | 打通显示/输入/资源/逐指令验收 | 🚧 进行中 |

---

## Phase 5: 可交互 MVP+（重点阶段）🚧

### 5.0 总目标

把“窗口能打开”提升为“可玩 demo”：

- 运行后能看到背景
- 能看到人物立绘（含位置）
- 能看到文本框/角色名
- 点击/空格可以推进
- 选择分支可点击并跳转

并且对每一种脚本指令：

- Runtime 产出 Command ✅
- Host 能执行 Command ✅
- 有 demo 脚本可见效果 ✅
- 有对应测试/回归用例 ✅
- 文档明确说明语法与限制 ✅

---

### 5.1 资源与可运行 Demo（必须先打底）

**目标**：让任何人 clone 后（至少在本机）可以直接跑出可见效果。

**工作项**：

- [ ] 在 `host/assets/` 提供最小占位资源（背景与立绘）
- [ ] 约定资源路径规则：脚本中的 `src="backgrounds/xxx.png"` 对应 `host/assets/backgrounds/xxx.png`
- [ ] Demo 脚本更新为“每条指令至少出现一次”，并且资源都存在
- [ ] Host 输出更清晰的资源加载错误（指明缺少哪个文件、期望路径是什么）

**验收标准**：

- [ ] `cargo run -p host` 后，必定能看到背景与文本（不依赖用户自行添加图片）

---

### 5.2 显示链路：对话（ShowText）

**目标**：把对话框显示做到稳定、可扩展。

**工作项**：

- [ ] 文本 UI：背景面板、角色名、正文
- [ ] 处理 `Command::ShowText { speaker, content }`
- [ ] 文本换行、长文本布局（至少不溢出屏幕）
- [ ] 支持旁白（`speaker=None`）

**验收标准**：

- [ ] Demo 能显示多行对话与旁白

**测试**：

- [ ] `vn-runtime`：对话节点产出 `ShowText + WaitForClick` 的回归测试
- [ ] `host`：UI 系统在接到 `VNCommand::ShowText` 时能更新 `DialogueState`（纯逻辑测试/最小集成测试）

---

### 5.3 输入链路：点击/空格推进（WaitForClick）

**目标**：点击/空格能稳定推进 runtime。

**工作项**：

- [ ] 收集输入并转换为 `RuntimeInput::Click`
- [ ] Runtime tick：从 `WaitForClick` 解除并继续推进
- [ ] 防抖/一次点击只推进一次（避免一帧多次触发）

**验收标准**：

- [ ] 连续点击可逐句推进对话

**测试**：

- [ ] `vn-runtime`：等待/解除等待的回归测试

---

### 5.4 显示链路：背景（ShowBackground / ChangeBG）

**目标**：背景图片可见，切换可用。

**工作项**：

- [ ] 处理 `Command::ShowBackground { path, transition }`
- [ ] 背景 entity 的生成/替换策略（保留一个背景实体）
- [ ] 图片缩放策略（铺满/等比）先确定一种默认

**验收标准**：

- [ ] Demo 能至少切换 2 张背景

**测试**：

- [ ] `vn-runtime`：ChangeBG 节点产出 `ShowBackground` 的回归测试

---

### 5.5 显示链路：人物（ShowCharacter / HideCharacter）

**目标**：人物立绘可见，位置正确，显示/隐藏可用。

**工作项**：

- [ ] 处理 `Command::ShowCharacter { path, alias, position, transition }`
- [ ] 处理 `Command::HideCharacter { alias, transition }`
- [ ] alias → entity 映射与重复 show 的更新逻辑
- [ ] position → 屏幕坐标映射（先实现 Left/Center/Right 的可靠版本，其他位置作为扩展）

**验收标准**：

- [ ] Demo 能显示两个角色并切换立绘

**测试**：

- [ ] `vn-runtime`：show/hide 节点产出命令的回归测试

---

### 5.6 选择分支（PresentChoices + WaitForChoice）

**目标**：分支选择可点击，能跳转到正确 label。

**工作项**：

- [ ] 处理 `Command::PresentChoices { choices }`
- [ ] UI 生成按钮列表
- [ ] 点击按钮 → `RuntimeInput::ChoiceSelected(index)`
- [ ] Runtime：根据 index 跳转并继续

**验收标准**：

- [ ] Demo 中选择不同选项会走不同对话

**测试**：

- [ ] `vn-runtime`：选择分支跳转的回归测试
- [ ] `host`：选择按钮点击到输入消息的最小集成测试

---

### 5.7 “逐指令打通”清单（按难度从低到高）

> 原则：每实现一条指令，就在 demo 脚本里出现它，并增加对应回归测试。

- [ ] `Dialogue / Narration` → `ShowText`
- [ ] `changeBG` → `ShowBackground`
- [ ] `show ... as ... at ...` → `ShowCharacter`
- [ ] `hide ...` → `HideCharacter`
- [ ] `choice table` → `PresentChoices` + `WaitForChoice`
- [ ] `Chapter` → `ChapterMark`（至少能影响 UI 或日志）
- [ ] `UIAnim` → `UIAnimation`（先日志/占位实现）
- [ ] `PlayBgm/StopBgm/PlaySfx`（可选：先日志，后接 Bevy Audio）

---

### 5.8 文档与使用说明补齐

**目标**：让使用者知道如何写脚本、放资源、运行 demo、排错。

**工作项**：

- [ ] 更新 `docs/script_syntax_spec.md`：逐条标记“已实现/未实现/限制”
- [ ] 新增/更新（在已有文档中补充）“如何运行 demo + assets 目录约定”
- [ ] 在 `ROADMAP.md` 记录每个子任务的完成日期与验收结果

---

## 进度追踪（Phase 5）

| 任务 | 状态 | 完成日期 |
|------|------|----------|
| 5.1 资源与可运行 Demo | ⏳ 待开始 | - |
| 5.2 对话显示（ShowText） | ⏳ 待开始 | - |
| 5.3 输入推进（WaitForClick） | ⏳ 待开始 | - |
| 5.4 背景显示（ShowBackground） | ⏳ 待开始 | - |
| 5.5 人物显示/隐藏（Show/HideCharacter） | ⏳ 待开始 | - |
| 5.6 选择分支（PresentChoices） | ⏳ 待开始 | - |
| 5.7 逐指令打通与回归测试 | ⏳ 待开始 | - |
| 5.8 文档完善 | ⏳ 待开始 | - |

---

## 后续迭代计划（Phase 6+）

### 优先级 1（内容生产必需）

- [ ] 存档/读档（RuntimeState 已具备序列化基础）
- [ ] 过渡动画（fade/dissolve）

### 优先级 2（体验优化）

- [ ] 文本打字机效果
- [ ] 历史记录回看
- [ ] UI 美术化

### 优先级 3（脚本语言增强）

- [ ] 条件分支（if/else）
- [ ] 变量系统（赋值/表达式）
- [ ] 脚本热重载
