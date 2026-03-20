# vn-runtime 模块摘要

## Purpose

`vn-runtime` 是视觉小说引擎的纯逻辑核心：解析脚本、执行节点、维护运行时状态，并以 `Command` 向宿主层输出“做什么”。

## PublicSurface

- 对外入口：`vn-runtime/src/lib.rs`
- 关键公开类型：`VNRuntime`、`Command`、`RuntimeInput`、`RuntimeState`、`WaitingReason`
- 关键公开能力：`Parser`（脚本解析）、`analyze_script`（静态诊断）、`SaveData`（存档模型）

## KeyFlow

1. 文本脚本通过 `Parser` 转为 `Script`（含 source map 与 base_path）。
2. `VNRuntime::tick(input)` 驱动执行循环。
3. `Executor` 将 `ScriptNode` 转为 `Command`/等待原因/跳转信息。
4. `RuntimeState` 记录位置、变量、等待、可恢复渲染状态片段。
5. Host 消费 `Command` 并在下一帧继续喂入 `RuntimeInput`。

## Submodules

- [script](vn-runtime/script.md)：AST/表达式/解析器总入口
- [runtime](vn-runtime/runtime.md)：引擎循环与执行器
- [command](vn-runtime/command.md)：Runtime-Host 通信契约
- [diagnostic](vn-runtime/diagnostic.md)：脚本静态分析与资源引用提取
- [parser](vn-runtime/parser.md)：两阶段解析细节专题

## Invariants

- 不依赖渲染引擎、IO、真实时钟。
- 状态显式可序列化（用于存档恢复）。
- Runtime 与 Host 仅通过 `Command`/`RuntimeInput` 交互。

## FailureModes

- 标签不存在：`goto`/`choice` 目标无法解析。
- 输入状态不匹配：等待态与输入类型不一致。
- 选择索引越界：`ChoiceSelected` 超出选项数量。
- 表达式求值失败：变量缺失或类型不匹配。

## WhenToReadSource

- 需要确认具体分支行为（例如 choice 历史记录、条件分支短路）。
- 需要补回归测试或新增语法节点。
- 需要确认错误类型与消息格式。

## RelatedDocs

- [摘要索引](../summary_index.md)
- [仓库导航地图](../navigation_map.md)
- [脚本语法规范](../script_syntax_spec.md)
- [存档格式](../save_format.md)

## LastVerified

2026-03-18

## Owner

Composer