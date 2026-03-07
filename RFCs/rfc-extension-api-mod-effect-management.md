# RFC: 扩展 API 与 Mod 化效果管理

## 元信息

- 编号：RFC-004
- 状态：Active
- 作者：Ring-rs 开发组
- 日期：2026-03-07
- 影响范围：`vn-runtime`、`host`、`docs/script_syntax_spec.md`、`assets/scripts/remake/README.md`
- 相关 RFC：
  - `RFCs/rfc-show-unification-ergonomics.md`
  - `RFCs/rfc-remake-experience-equivalence.md`

---

## 0. 实施进度（2026-03-07）

> 本节用于同步 RFC 与当前代码状态，避免“文档计划”与“仓库实现”漂移。

### 0.1 已完成

- 已落地 `host/src/extensions/` 扩展 API 基础模块：
  - `manifest.rs`（扩展元信息）
  - `capability.rs`（扩展 trait 与错误模型）
  - `context.rs`（`EngineContext` + 诊断记录）
  - `registry.rs`（注册、冲突检测、版本兼容、调度）
  - `builtin_effects.rs`（内建 capability 实现）
- 已建立 `EffectRequest { capability_id, params, target, effect }` 的统一请求模型。
- 已将以下能力迁移为内建扩展并接入注册表：
  - `effect.dissolve`
  - `effect.fade`
  - `effect.rule_mask`
  - `effect.move`
- `effect_applier` 已改为“capability 路由优先 + capability 级回退”，不再维护直接调用 renderer 的 legacy 执行分支。
- 诊断日志已包含 `capability_id` 与扩展来源（`extension_name`）。

### 0.2 当前未完成

- `sceneEffect` 相关能力仍以规划为主，尚未形成首批稳定 capability（如 `effect.camera.blur_pulse`）。
- 第三方扩展加载机制（扫描/装载/生命周期治理）未启用。
- API 对外冻结策略与兼容窗口仍待最终确认。

### 0.3 阶段结论

- `Phase A`：完成
- `Phase B`：完成
- `Phase C`：首轮完成（高频基础效果已扩展化，复杂镜头类能力待增量推进）
- `Phase D`：未开始（按 RFC 约定本轮不启用）

---

## 1. 背景

重制迁移过程中，原作存在大量由 Ren'Py + Python + 第三方库组合出的复杂效果。  
Ring-rs 的目标不是把这类自由度直接暴露给脚本层，而是提供稳定高层语义接口，复杂实现交给程序员侧维护。

当前痛点：

1. 新增一个复杂效果时，常需要跨多个模块改动，成本高且容易引入耦合。
2. 效果实现入口分散，难以形成统一能力目录和版本约束。
3. 脚本语义与底层实现关系不够稳定，影响长期可维护性。

---

## 2. 目标与非目标

### 2.1 目标

- 提供稳定的扩展 API 界面，支持“像 mod 一样”注册能力。
- 让新增/迭代效果尽量在扩展模块内完成，避免到处改核心源码。
- 将“现有效果”逐步纳入同一注册与生命周期管理机制。
- 保持脚本层接口稳定，编剧侧不暴露实现复杂度。

### 2.2 非目标

- 本 RFC 不讨论安全沙箱或权限隔离。
- 本 RFC 不要求一次性替换所有历史实现。
- 本 RFC 不改变 Runtime/Host 的总体分层边界。

---

## 3. 提案总览

建立“三层模型”：

1. **Core（稳定内核）**
   - 负责脚本执行、命令分发、渲染调度主循环
   - 不感知具体效果实现细节

2. **Extension API（稳定扩展接口）**
   - 定义可注册能力、上下文访问、生命周期钩子、版本协商
   - 作为程序员扩展的唯一官方入口

3. **Extensions/Mods（可插拔实现）**
   - 承载复杂效果、镜头语言、音频策略等可演进能力
   - 通过注册表挂接到 Core

---

## 4. 建模细节

### 4.1 能力注册表（Capability Registry）

每个扩展声明其提供的能力（示例）：

- `effect.dissolve`
- `effect.fade`
- `effect.rule_mask`
- `effect.camera.blur_pulse`
- `audio.bgm.duck`

Core 仅根据 `capability_id` 路由，不包含具体效果分支逻辑。

### 4.2 统一请求协议

脚本语义经 Runtime 解析后，进入统一请求对象（示例概念）：

- `EffectRequest { capability_id, target, params }`

Core 将请求交给注册表匹配的扩展处理；缺失时走 fallback。

### 4.3 稳定上下文接口（EngineContext）

扩展通过受控上下文获取能力：

- 读取必要运行态（只读）
- 提交渲染/音频动作（受约束）
- 查询时间轴与帧信息
- 输出诊断信息

禁止扩展直接依赖 Core 私有结构体，避免“内部字段耦合”。

### 4.4 生命周期钩子

扩展可实现标准钩子（示意）：

- `on_load`
- `on_scene_enter`
- `on_request`
- `on_update`
- `on_unload`

通过统一生命周期，避免每加一个效果就改主循环流程。

### 4.5 Manifest 与版本协商

每个扩展声明元信息（示例）：

- `name`
- `version`
- `engine_api_version`
- `capabilities`
- `dependencies`

启动时做 API 版本协商，不兼容则禁用并给出诊断。

---

## 5. 与脚本语义的关系

脚本层保持高层稳定接口（如 `show`、`sceneEffect`、`titleCard`），不直接引用扩展实现细节。  
脚本语义到扩展能力的映射由引擎内部完成：

- `sceneEffect blurPulse(...)` -> `effect.camera.blur_pulse`
- `show ... with dissolve` -> `effect.dissolve`

这样可以在不改脚本的前提下替换实现策略。

---

## 6. 现有效果纳入同机制的迁移计划

## 6.1 Phase A：注册表落地（不改行为）

- 建立扩展 API 抽象与能力注册中心
- 将现有内建效果先“包装为内建扩展”挂入注册表
- 目标：行为不变，仅重构接入路径

## 6.2 Phase B：请求统一（收敛入口）

- 将现有多处效果分发入口统一收敛到请求协议
- 清理散落的特效 if-else 路由逻辑

## 6.3 Phase C：增量扩展化

- 将复杂/非通用效果逐步迁移到独立扩展模块
- 优先处理重制高频场景（`prologue`、`3-5`、`3-7`、`ending`）

## 6.4 Phase D：第三方扩展支持（可选）

- 在内建扩展稳定后，评估对外开放扩展装载机制
- 对外开放前先冻结 API 版本策略与诊断规范

---

## 7. 约束与防“史山”规则

1. Core 不引入按效果名硬编码分支（必须经注册表路由）。
2. 新效果默认以扩展方式接入，禁止直接侵入主循环。
3. 扩展 API 变更必须走 RFC，并声明兼容策略。
4. 每个能力至少有一个验收样例与回归测试。
5. 诊断必须可定位到 `capability_id` 与扩展来源。

---

## 8. DoD（验收标准）

- 至少 3 个现有效果成功迁移为“内建扩展”实现，行为一致。
- Core 中效果分发代码显著收敛，新增效果不需要跨模块改主流程。
- 缺失能力时有明确 fallback 与诊断，不阻断主线运行。
- 脚本语义层不新增复杂实现细节暴露。

---

## 9. 风险与缓解

1. **风险：抽象过早导致接口僵化**  
   **缓解：先内建扩展化，再冻结对外 API。**

2. **风险：迁移期双路径并存复杂度上升**  
   **缓解：分阶段设置收敛里程碑，阶段完成后删除旧路径。**

3. **风险：性能回归（额外路由开销）**  
   **缓解：注册表查找缓存 + 关键路径基准测试。**

---

## 10. 待确认事项

1. `sceneEffect` 首批 capability 清单与参数契约（建议先 `blur_pulse` / `camera_shake`）。
2. 扩展 API 首个稳定版本的兼容窗口（建议按主版本兼容，1-2 里程碑冻结）。
3. 第三方扩展开放前的装载策略与诊断规范（目录约定、冲突优先级、禁用策略）。
