# 摘要索引（Summary Index）

> 目标：在任务中让 agent 优先通过摘要完成定位与答复，减少源码扫描与上下文消耗。

## 适用范围（当前）

- 覆盖 `vn-runtime` 与 `host`。
- 任务默认遵循“摘要优先、源码兜底”。

## 摘要结构

- [vn-runtime 模块总览](module_summaries/vn-runtime.md)
- `vn-runtime` 子模块摘要（按任务选读）
   - [script](module_summaries/vn-runtime/script.md)
   - [runtime](module_summaries/vn-runtime/runtime.md)
   - [command](module_summaries/vn-runtime/command.md)
   - [diagnostic](module_summaries/vn-runtime/diagnostic.md)
   - [parser](module_summaries/vn-runtime/parser.md)
- [host 模块总览](module_summaries/host.md)
- `host` 子模块摘要（按任务选读）
   - [app](module_summaries/host/app.md)
   - [app_update](module_summaries/host/app_update.md)
   - [app_command_handlers](module_summaries/host/app_command_handlers.md)
   - [extensions](module_summaries/host/extensions.md)
   - [command_executor](module_summaries/host/command_executor.md)
   - [rendering_types](module_summaries/host/rendering_types.md)
   - [renderer](module_summaries/host/renderer.md)
   - [renderer_render_state](module_summaries/host/renderer_render_state.md)
   - [renderer_animation](module_summaries/host/renderer_animation.md)
   - [renderer_effects](module_summaries/host/renderer_effects.md)
   - [renderer_scene_transition](module_summaries/host/renderer_scene_transition.md)
   - [resources](module_summaries/host/resources.md)
   - [audio](module_summaries/host/audio.md)
   - [video](module_summaries/host/video.md)
   - [input](module_summaries/host/input.md)
   - [ui](module_summaries/host/ui.md)
   - [backend](module_summaries/host/backend.md)
   - [config](module_summaries/host/config.md)
   - [manifest](module_summaries/host/manifest.md)
   - [save_manager](module_summaries/host/save_manager.md)
   - [host_app](module_summaries/host/host_app.md)
   - [egui_actions](module_summaries/host/egui_actions.md)
   - [egui_screens](module_summaries/host/egui_screens.md)
- [仓库导航地图](navigation_map.md)
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
- 周期性验收可使用：[摘要抽样验收清单](summary_sampling_checklist.md)。
- 更新摘要时须同步维护 `LastVerified` 与 `## Owner`：`Owner` 为校验者所使用的底层模型名称，如无法获知则填写**自认的模型名称**。无法确定时写 `未记录`（见 [摘要维护协议](summary_maintenance.md)）。
