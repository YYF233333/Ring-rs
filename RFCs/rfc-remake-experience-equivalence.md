# RFC: ref-project 重制体验等价计划

## 元信息

- 编号：RFC-002
- 状态：Active
- 作者：Ring-rs 开发组
- 日期：2026-03-07（最后更新：2026-03-15）
- 相关范围：`assets/scripts/remake/`、`vn-runtime`、`host`、`docs/script_syntax_spec.md`

---

## 1. 背景

本 RFC 定义 ref-project 重制的体验复刻标准与实施优先级。

目标是**复刻原作的玩家体验**，而不是复刻 Ren'Py 的实现细节。引擎以"人体工学优先、引擎吸收复杂度"为设计哲学，脚本侧描述叙事意图与演出意图，由 runtime/host 承担复杂视觉/音频/状态机实现。

---

## 2. 目标与非目标

### 2.1 目标

- **叙事等价**：剧情顺序、分支结果、章节解锁与结局触发一致。
- **演出等价**：核心镜头语言一致（转场节奏、角色入退场、黑白场切换、关键 CG 时机）。
- **交互等价**：玩家操作路径一致（主菜单/设置/历史/存读档/快进自动）。
- **听感等价**：BGM/SFX/语音的触发时机与层次一致。

### 2.2 明确非目标

- 不做 Ren'Py 全语法、全 API 兼容层。
- 不要求 Ren'Py transform/transition 内部算法完全一致。
- 不要求 GUI 像素级一致（保留美术风格与信息层级即可）。
- 不复刻开发者工具链行为（控制台、调试快捷键等）。
- 不因兼容旧实现破坏当前引擎分层与可维护性。

---

## 3. 仓库现状（2026-03-15）

### 3.1 脚本内容

| 内容 | 状态 |
|------|------|
| 夏篇 Ring 语义稿（`ring/summer/`） | 12 个 .md，已完成 |
| 冬篇 Ring 语义稿（`ring/winter/`） | 10 个 .md，已完成 |
| 主入口编排（`main.md`） | 已完成，summer → winter 全流程 |
| 节奏标签迁移（`{wait}`/`{speed}`/`-->`/`extend`） | 12 个文件已适配 |
| 原始 .rpy 保留 | 对照参考用，不参与运行 |

### 3.2 引擎能力——已实现

#### 脚本调度

- `callScript` / `returnFromScript` + 调用栈管理（跨文件脚本调度完整闭环）
- 非入口脚本 EOF 自动 return，入口文件 EOF 自动结束并返回主界面
- 禁用跨文件 `goto`（不做全局 label 命名空间索引）

#### 视觉演出

- 基础转场：dissolve、fade、fadewhite、rule_mask
- 角色调度：show/hide/move 效果
- sceneEffect 首批 capability：shake（shakeSmall/shakeVertical/bounceSmall）、blur（blurIn/blurOut）、dim（dimStep/dimReset）
- titleCard：全屏字卡 + 淡入淡出
- changeScene 多阶段过渡状态机（Fade/FadeWhite/Rule）
- 渲染后端：winit + wgpu + egui（RFC-007/008 已完成迁移）
- 渲染抽象层：Texture trait / TextureFactory / DrawCommand（RFC-008）

#### 文本与节奏

- `wait` 等待指令（定时/点击打断/Skip 跳过）
- `pause` 纯点击等待
- 节奏标签（`{wait}`/`{wait Ns}`/`{speed N}`/`{speed Nx}`/`{/speed}`/`-->`）
- `extend` 台词续接（打字机续接 + 历史追加）
- 窗口显隐控制（`textBoxHide/textBoxShow/textBoxClear`）

#### 音频

- BGM 播放/停止/交叉淡化
- SFX 一次性播放
- `bgmDuck` / `bgmUnduck`（duck_multiplier 独立叠加于 FadeState，平滑过渡）

#### 持久化与门控

- 存档/读档系统（槽位、元数据、版本兼容）
- 全局持久化域（`$persistent.key`，`saves/persistent.json`，严格双域隔离）
- `fullRestart`（持久化 → 清空会话 → 返回标题）
- `complete_summer` 首通门控闭环

#### 系统 UI 与播放控制

- 核心页面：主菜单、设置、历史、存档/读档、游内菜单
- Skip/Auto/Normal 三模式 + `skip_all_active_effects()` 收敛跳过

#### 视频播放

- `cutscene` 命令全链路（RFC-009）
- FFmpeg sidecar 解码 + rodio 音频播放
- FFmpeg 不可用时优雅降级（warn + 跳过）

### 3.3 引擎能力——未实现

#### sceneEffect 高级 capability（脚本中已使用，运行时降级为 warn + 跳过）

| capability | 使用位置 | 说明 |
|------------|---------|------|
| `focusPush` | 1-5.md | 镜头推近 |
| `pushIn` | 1-5.md | 镜头推入 |
| `panRight` | 1-5.md | 镜头右移 |
| `resetCamera` | 1-5.md | 镜头复位 |
| `skyPan` | 3-7.md | 天空全景平移 |
| `slowVerticalPan` | 3-7.md | 缓慢垂直平移 |
| `imageWipe` | 3-7.md | 图片遮罩擦除转场 |
| `flashbackIn` | ending.md | 闪回进入效果 |
| `flashbackOut` | ending.md | 闪回退出效果 |

#### 其他缺口

| 缺口 | 说明 |
|------|------|
| 三通道混音联动 | music/sound/voice 独立音量 + 设置页控制 |
| 角色演出语义层 | 站位预设（单人/双人/三人）、近远景预设 |
| 复刻检查器 | 演出段漏映射、节奏标签丢失、资源缺失等静态检查 |
| 端到端验收 | 关键段落演出观感、Skip/Auto 边界、存读档闭环 |

---

## 4. 分阶段计划

### 4.1 P0：完整可玩 ✅ 已完成

P0 的目标是"能从头到尾完整地玩通主线"。所有 P0 项目均已完成。

| 子项 | 内容 | 状态 |
|------|------|------|
| 跨文件脚本调度 | callScript/returnFromScript/调用栈 | ✅ 完成 |
| 演出基础闭环 | dissolve/fade/fadewhite/rule_mask/show/hide/move | ✅ 完成 |
| sceneEffect 首批 | shake/blur/dim 全链路 | ✅ 完成 |
| titleCard | 全屏字卡 + 淡入淡出 | ✅ 完成 |
| 文本节奏 | wait/pause/节奏标签/extend | ✅ 完成 |
| 窗口控制 | textBoxHide/textBoxShow/textBoxClear | ✅ 完成 |
| 脚本节奏适配 | 12 个 .md 节奏标签迁移 | ✅ 完成 |
| 持久化与门控 | $persistent/fullRestart/complete_summer | ✅ 完成 |
| 存读档 | 槽位/元数据/版本兼容 | ✅ 完成 |
| 核心 UI | 主菜单/设置/历史/存读档/游内菜单 | ✅ 完成 |
| 播放模式 | Skip/Auto/Normal + skip_all_active_effects() | ✅ 完成 |
| 音频基础 | BGM/SFX 播放与淡入淡出 | ✅ 完成 |
| bgmDuck | duck/unduck 音量压低与恢复 | ✅ 完成 |
| 视频播放 | cutscene 全链路（FFmpeg sidecar） | ✅ 完成 |

### 4.2 P1：增强"像原作"的主观感受

P1 的目标是拉齐主观观感与听感，使关键段落的演出接近原作体验。

#### P1-1 sceneEffect 高级 capability

- [ ] 镜头类：focusPush / pushIn / panRight / resetCamera（1-5.md）
- [ ] 全景类：skyPan / slowVerticalPan（3-7.md）
- [ ] 遮罩类：imageWipe（3-7.md）
- [ ] 视觉类：flashbackIn / flashbackOut（ending.md）
- [ ] 验收：1-5、3-7、ending 关键镜头段落观感抽检通过

#### P1-2 音频增强

- [ ] 三通道混音（music/sound/voice）与设置页音量联动
- [ ] BGM 平滑暂停/恢复与跨段衔接
- [ ] 验收：prologue BGM 情绪曲线一致

#### P1-3 端到端验收

- [ ] 关键段落演出观感抽检（prologue/1-5/3-5/3-7/ending）
- [ ] Skip/Auto/Normal 模式端到端稳定性（含 wait/pause/inline_wait 跳过行为、模式切换边界）
- [ ] 存读档端到端闭环（从标题进入任意章节，完成存读档循环）

### 4.3 P2：长期可维护与生产效率

#### P2-1 角色演出语义层

- [ ] 抽象角色入场/退场/换表情/镜头位置语义
- [ ] 常用站位预设（单人/双人/三人站位与近远景）
- [ ] 验收：抽查 4 个章节，角色镜头主观一致

#### P2-2 复刻检查器

- [ ] 高风险演出段漏映射检查
- [ ] 节奏标签丢失检查
- [ ] 资源缺失检查
- [ ] 门控状态异常检查

#### P2-3 内容生产规范

- [ ] 形成重制脚本规范（命名、章节切分、演出标注）
- [ ] 建立"转换后脚本 + 演出回放样例"的回归样本库

#### P2-4 视觉一致性迭代

- [ ] UI 风格持续贴近原作（按钮、字体、层次、动效节奏）
- [ ] 保持现有引擎架构清晰，不回退到 Ren'Py 风格耦合实现

---

## 5. 里程碑

| 里程碑 | 定义 | 状态 |
|--------|------|------|
| M1 可玩主线 | 从 main.md 单入口跑通 summer → winter 全流程 | ✅ 已达成 |
| M2 关键观感 | prologue/1-5/3-7/ending 关键镜头段落抽检通过 | 待 P1-1/P1-3 |
| M3 门控闭环 | 首通后重启，章节入口和菜单状态正确，可存读档回归 | ✅ 已达成 |
| M4 主观一致 | 完整通关体验与原作主观感受接近 | 待 P1 全部完成 |

---

## 6. 下一步：P1-1 sceneEffect 高级 capability

### 6.1 现状

首批 sceneEffect capability（shake/blur/dim）已实现，走通了完整的全链路：AST → Parser → Command → Executor → Host capability 注册表 → Renderer。新增 capability 只需在 Host 侧注册效果处理器，不涉及 vn-runtime 侧改动。

### 6.2 待实现 capability 清单

按脚本使用频率和实现复杂度排序：

| capability | 类型 | 涉及脚本 | 实现思路 |
|------------|------|---------|---------|
| focusPush / pushIn | 镜头推近 | 1-5 | 背景/角色层 scale 动画 |
| panRight | 镜头平移 | 1-5 | 背景/角色层 translate 动画 |
| resetCamera | 镜头复位 | 1-5 | scale/translate 回归默认值 |
| skyPan | 全景平移 | 3-7 | 背景层水平/垂直 translate |
| slowVerticalPan | 缓慢垂直平移 | 3-7 | 背景层垂直 translate（长时间） |
| imageWipe | 遮罩擦除 | 3-7 | 灰度遮罩图 + 进度驱动显隐 |
| flashbackIn | 闪回进入 | ending | 色调偏移 + blur + vignette |
| flashbackOut | 闪回退出 | ending | 上述效果反向消退 |

### 6.3 实施路径

1. 在 `host/src/renderer/effects/` 注册新的 `EffectKind` 变体与参数解析。
2. 在 `host/src/app/command_handlers/effect_applier` 注册 capability 处理器。
3. 在 `host/src/renderer/scene_effects.rs` 实现渲染逻辑（偏移/缩放/遮罩等）。
4. 逐个 capability 验证对应脚本段落。

---

## 7. 相关 RFC

| RFC | 标题 | 状态 | 与本 RFC 关系 |
|-----|------|------|-------------|
| RFC-003 | show 语义收敛与人体工学优先 | Accepted | 定义了 show/hide 命令体系 |
| RFC-004 | 扩展 API 与 Mod 化效果管理 | Active | 定义了 capability 注册表框架 |
| RFC-006 | 节奏标签与 extend | Accepted | P0-3 文本节奏的实现方案 |
| RFC-007 | 渲染后端迁移 | Accepted | 基础设施升级 |
| RFC-008 | 渲染后端 Trait 抽象 | Accepted | 基础设施升级 |
| RFC-009 | Cutscene 视频播放 | Accepted | cutscene 实现方案 |
