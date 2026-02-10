---
name: repo-maintainability
overview: 在不破坏 ARCH.md 硬约束（Runtime/Host 分离、显式状态、Command 驱动、确定性）的前提下，通过“文档/索引 + 结构治理 + 测试回归 + 工具链”四条线并行推进，显著降低 Cursor/新人理解成本与 token 浪费，并为下一阶段特性（语音、UI Theme/组件库）铺路。
todos:
  - id: doc-ai-onboarding
    content: 新增 docs/dev_index.md、docs/ai_context.md、docs/change_guide.md、docs/command_contract.md，并更新 docs/navigation_map.md 与各 crate README（vn-runtime/host/tools）。
    status: pending
  - id: runtime-refactor-tests
    content: vn-runtime：修正文档示例；消除 Choice 的 node_index-1 约定；收敛 history 记录映射；拆分 phase2.rs；补齐端到端/存档/诊断等回归测试。
    status: pending
  - id: host-refactor-tests
    content: host：收敛淡出清理逻辑；拆分 modes.rs；参数化 screen_width 依赖；解耦 save/restore；补齐 headless 集成测试；同时为 UI Theme/组件库提供 tokens 与样板页面迁移路径。
    status: pending
  - id: tooling-gates
    content: 工具链：新增 check-fast/check-core；改进 script-check 缺失目录提示；完善 .cursorignore（本地数据/大文件）与可选 pre-commit 指南。
    status: pending
isProject: false
---

## 目标与硬约束

- **目标**：让 Cursor/新人以最少阅读快速定位改动点；降低耦合与重复；关键链路可回归；为阶段28（语音）与阶段29（UI Theme/组件库）提供更清晰的扩展点。
- **硬约束（必须保持）**：遵守 [C:\code\Ring-rs\ARCH.md](ARCH.md)
  - Runtime 禁止引擎/IO/真实时间依赖；Host 禁止脚本语义与直接篡改 Runtime 内部。
  - 关键逻辑必须有单测/回归测试；Public API 必须有文档注释；禁止无关顺便重构。

## 现状关键信号（用于确定治理切入点）

- **vn-runtime 复杂度热点**：Choice 通过 `node_index - 1` 推断当前节点（隐式约定，易脆）以及历史记录 `record_history` 在 Engine 内集中匹配；`phase2.rs` 超大文件；路径解析逻辑分散；错误/诊断体系分层但缺少统一说明与转换。
- **host 复杂度热点**：淡出角色清理逻辑重复（`host/src/app/update/mod.rs` 里自行实现一份，且 `script.rs` 也有清理函数）；`modes.rs` 聚合多模式更新导致文件膨胀；`effect_applier` 依赖 `screen_width()` 影响可测性；存档恢复对 `AppState` 深耦合。
- **工具/文档**：`docs/navigation_map.md` 已很强，但缺少“变更流程 checklist / Command 契约 / 各 crate README / 决策记录 / AI 上下文包”。

## 交付物与分阶段推进（均衡：每阶段=结构治理+特性前置）

### 阶段A：AI/新协作者上手与索引体系（高收益、低风险，优先）

- **新增开发者索引**：新增 `docs/dev_index.md`
  - 读者分流：内容作者 vs 引擎开发者。
  - 5分钟入口：架构约束、导航地图、常用命令、最常见改动路径。
- **Command 契约文档**：新增 `docs/command_contract.md`
  - 从 `vn-runtime/src/command.rs` 映射到 `host/src/command_executor/*` 与 `host/src/app/command_handlers/*`，说明扩展新 Command 的步骤与回归点。
- **各 crate README**：新增 `vn-runtime/README.md`、`host/README.md`、`tools/xtask/README.md`、`tools/asset-packer/README.md`
  - 写清职责、入口文件、模块地图、测试命令与常见改动入口。
- **AI 上下文包（减少 token 浪费）**：新增 `docs/ai_context.md`
  - 固化：硬约束、关键入口、常见配方、禁止事项、测试门禁、命名约定。
  - 目标：让模型优先读这一份再动手，避免全仓库漫游。
- **更新导航地图**：补齐 `docs/navigation_map.md` 中的常见改动场景（效果/动画、表达式语法、输入映射、asset-packer/xtask 扩展等）。

### 阶段B：vn-runtime 结构治理 + 语音特性前置（中风险，带回归测试）

- **修正文档示例偏差**：`vn-runtime/src/runtime/engine.rs` 示例仍写 `Script::parse`（与实际 `Parser::parse` 不一致），统一修正并在 `vn-runtime/src/lib.rs` 等示例同步。
- **消除 Choice 的隐式约定**：移除 `node_index - 1` 推断（见 `vn-runtime/src/runtime/engine.rs`）
  - 方案：在进入 `WaitForChoice` 时把必要的 Choice 上下文（选项文本/目标 label/或节点索引）显式存入等待状态或 RuntimeState 的某个字段，使 `ChoiceSelected` 处理不依赖位置魔法。
  - **测试**：补 Choice + jump + history 的回归用例（含连续 choice、choice 后立即 goto 等）。
- **收敛历史记录映射**：把 `record_history` 从 Engine 迁移到更合适的位置（例如 `Command` 上的转换函数或独立模块），Engine 仅调用。
  - **测试**：覆盖 `ShowText/ChapterMark/ShowBackground/PlayBgm/StopBgm` 等映射稳定性。
- **拆分超大 parser 文件**：将 `vn-runtime/src/script/parser/phase2.rs` 按语法域拆模块（例如 `phase2/dialogue.rs`、`phase2/scene.rs`、`phase2/choice.rs`、`phase2/control_flow.rs`…），`phase2/mod.rs` 做分发。
  - 目标：定位某语法改动不再在 700+ 行文件里搜索。
- **路径解析去重**：将 `script.resolve_path()` 的重复调用抽成统一 helper（executor 内集中入口），为后续“语音资源路径策略”预留扩展点。
- **错误/诊断体系“先文档后统一”**：先在 `docs/` 补一页说明 ParseError/Diagnostic/RuntimeError 的职责与展示口径；再引入最小转换接口（如把 runtime 错误转成可展示诊断）而不强行合并类型。
- **端到端回归测试补齐**（建议新增到 `vn-runtime` 的 tests 或模块内）
  - parse -> tick -> commands 的串联测试
  - save/restore 往返测试（含变量/历史/等待）
  - diagnostic 行号与 source map 相关用例
  - base_path + 资源引用解析用例

### 阶段C：host 结构治理 + UI Theme/组件库前置（中高风险，分批提交）

- **去重淡出角色清理**：将 `host/src/app/update/mod.rs` 中淡出完成检测/注销逻辑收敛到单一函数（复用 `script.rs` 中已有的清理入口或抽公共 helper）。
- **拆分 `modes.rs**`：把 `host/src/app/update/modes.rs` 按 AppMode 拆到 `host/src/app/update/modes/` 子模块（title/ingame/menu/save_load/settings/history），保留顶层分发。
- **参数化屏幕尺寸依赖**：把 `effect_applier` 内的 `screen_width()` 依赖改为从调用处传入（例如来自 `UiContext`/配置的 design_width），提升可测性并减少隐式全局。
- **存档读写解耦**：为 `build_save_data`/`restore_from_save_data` 引入更窄的上下文结构（或 trait facade），减少对 `AppState` 深层字段的直接耦合，降低未来存档字段扩展的改动面。
- **补充 host 数据流说明**：新增 `docs/host_command_flow.md`（或并入 `command_contract.md`）
  - 明确 `CommandExecutor` vs `command_handlers` 的职责分界与数据流（Command -> Output -> Handlers）。
- **host 集成测试补齐（保持 headless）**
  - runtime tick -> executor -> effect_requests/audio_command 的链路测试
  - effect_applier 的可测试化后单测
  - save/restore 往返测试（host侧渲染状态/会话状态恢复）
  - input 映射与防抖逻辑测试（若可 headless）
  - navigation/mode 切换逻辑测试
- **UI Theme/组件库前置**
  - 在 `host/src/ui/theme.rs` 与 `docs/` 中先确定 tokens 结构与扩展点（不立刻重写全部页面）。
  - 选 1-2 个页面做样板（Title/Settings），其余页面按规划渐进迁移。

### 阶段D：工具链与本地门禁体验（低风险，贯穿）

- **script-check 的体验改进**：当默认 `assets/scripts` 不存在时，输出更明确的修复指引（并在文档说明仓库可能不带 assets，需要自行提供）。
- **可选 pre-commit（不强制）**：提供最小脚本与安装说明，默认跑`cargo check-all`。

## 推荐的提交与验收策略（避免巨大 PR，利于回归）

- 每个阶段拆成 3-8 个小 PR：每个 PR 都有明确 DoD（文档更新/测试新增/重构不改行为）。
- 每个结构改动必须附带：
  - 更新 `docs/navigation_map.md`（若涉及入口变更）
  - 更新相应 crate README
  - 新增或调整回归测试
- 门禁统一用：`cargo check-all`。

## 风险点与回避策略

- 大规模重命名/移动文件：采用“先加新模块/新 facade，再迁移调用点，再删旧代码”的三步法，保证每步可编译可测试。
- 行为不变承诺：所有重构 PR 必须以测试护栏为先（先补测试再动结构）。

## 关键参考入口（规划执行时优先读）

- 架构硬约束：[C:\code\Ring-rs\ARCH.md](ARCH.md)
- 导航地图：[C:\code\Ring-rs\docs\navigation_map.md](docs/navigation_map.md)
- Runtime 引擎与 Choice 隐式约定位置：`vn-runtime/src/runtime/engine.rs`
- Host 更新入口与淡出清理重复位置：`host/src/app/update/mod.rs`
- 工具链与 script-check：`tools/xtask/src/main.rs`、`.cargo/config.toml`

