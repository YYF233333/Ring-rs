# 摘要索引（Summary Index）

> 目标：在任务中让 agent 优先通过摘要完成定位与答复，减少源码扫描与上下文消耗。

## 适用范围（当前）

- 覆盖 `vn-runtime`、`host`、`host-dioxus`。`host-tauri` 已删除（RFC-033 迁移完成）。
- 任务默认遵循“摘要优先、源码兜底”。

## 摘要结构

- [vn-runtime 模块总览](../engine/architecture/module-summaries/vn-runtime.md)
- `vn-runtime` 子模块摘要（按任务选读）
   - [script](../engine/architecture/module-summaries/vn-runtime/script.md)
   - [runtime](../engine/architecture/module-summaries/vn-runtime/runtime.md)
   - [command](../engine/architecture/module-summaries/vn-runtime/command.md)
   - [diagnostic](../engine/architecture/module-summaries/vn-runtime/diagnostic.md)
   - [parser](../engine/architecture/module-summaries/vn-runtime/parser.md)
- [host 模块总览](../engine/architecture/module-summaries/host.md)
- `host` 子模块摘要（按任务选读）
   - [app](../engine/architecture/module-summaries/host/app.md)
   - [app_update](../engine/architecture/module-summaries/host/app-update.md)
   - [app_command_handlers](../engine/architecture/module-summaries/host/app-command-handlers.md)
   - [extensions](../engine/architecture/module-summaries/host/extensions.md)
   - [command_executor](../engine/architecture/module-summaries/host/command-executor.md)
   - [rendering_types](../engine/architecture/module-summaries/host/rendering-types.md)
   - [renderer](../engine/architecture/module-summaries/host/renderer.md)
   - [renderer_render_state](../engine/architecture/module-summaries/host/renderer-render-state.md)
   - [renderer_animation](../engine/architecture/module-summaries/host/renderer-animation.md)
   - [renderer_effects](../engine/architecture/module-summaries/host/renderer-effects.md)
   - [renderer_scene_transition](../engine/architecture/module-summaries/host/renderer-scene-transition.md)
   - [resources](../engine/architecture/module-summaries/host/resources.md)
   - [audio](../engine/architecture/module-summaries/host/audio.md)
   - [video](../engine/architecture/module-summaries/host/video.md)
   - [game_mode](../engine/architecture/module-summaries/host/game-mode.md)
   - [input](../engine/architecture/module-summaries/host/input.md)
   - [ui](../engine/architecture/module-summaries/host/ui.md)
   - [backend](../engine/architecture/module-summaries/host/backend.md)
   - [config](../engine/architecture/module-summaries/host/config.md)
   - [manifest](../engine/architecture/module-summaries/host/manifest.md)
   - [save_manager](../engine/architecture/module-summaries/host/save-manager.md)
   - [host_app](../engine/architecture/module-summaries/host/host-app.md)
   - [egui_actions](../engine/architecture/module-summaries/host/egui-actions.md)
   - [egui_screens](../engine/architecture/module-summaries/host/egui-screens.md)
   - [ui_modes](../engine/architecture/module-summaries/host/ui-modes.md)
- `host-dioxus`：后端模块与旧 host 共享逻辑（state/command_executor/render_state 等），前端为 RSX 组件重写。导航见 [navigation-map.md](../engine/architecture/navigation-map.md) 的 `host-dioxus/` 章节。
- [符号索引（Symbol Index）](../engine/symbol-index.md)（当前完整覆盖 `vn-runtime` / `host`）
- [仓库导航地图](../engine/architecture/navigation-map.md)
- [经验沉淀（Lessons Learned）](lessons-learned.md)
- 仅当需要实现细节时再读源码

## 升级到源码阅读的判定

满足任一条件才升级：

- 已完成摘要与规范阅读，且需要落到实现细节。
- 任务包含代码修改、测试补充或行为验证（此类任务也应先摘要后源码）。
- 摘要信息不足以回答边界条件。
- 多份摘要存在冲突。
- 需要确认最新实现细节（分支逻辑、错误处理、字段语义）。

## 使用约定

- 回答中优先引用摘要文档结论，再补充“是否需要看源码”判断。
- 发现摘要过期时，先在对应文档标记 `stale`，再补充修订。
- 新增或修改 `vn-runtime` / `host` 行为后，必须同步更新对应子模块摘要。
- 周期性验收可使用：[摘要抽样验收清单](summary-sampling-checklist.md)。
- 更新摘要时须同步维护 `LastVerified` 与 `## Owner`：`Owner` 为校验者所使用的底层模型名称，如无法获知则填写**自认的模型名称**。无法确定时写 `未记录`（见 [摘要维护协议](summary-maintenance.md)）。
