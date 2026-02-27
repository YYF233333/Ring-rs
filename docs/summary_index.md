# 摘要索引（Summary Index）

> 目标：在任务中让 agent 优先通过摘要完成定位与答复，减少源码扫描与上下文消耗。

## 适用范围（当前试点）

- 仅覆盖 `vn-runtime`。
- `host` 仍在结构调整中，暂不纳入摘要优先流程。

## 推荐阅读顺序

1. [vn-runtime 模块总览](module_summaries/vn-runtime.md)
2. 子模块摘要（按任务选读）
   - [script](module_summaries/vn-runtime/script.md)
   - [runtime](module_summaries/vn-runtime/runtime.md)
   - [command](module_summaries/vn-runtime/command.md)
   - [diagnostic](module_summaries/vn-runtime/diagnostic.md)
   - [parser](module_summaries/vn-runtime/parser.md)
3. [仓库导航地图](navigation_map.md)
4. 仅当需要实现细节时再读源码

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
- 新增或修改 `vn-runtime` 行为后，必须同步更新对应子模块摘要。
- 周期性验收可使用：[摘要抽样验收清单](summary_sampling_checklist.md)。
