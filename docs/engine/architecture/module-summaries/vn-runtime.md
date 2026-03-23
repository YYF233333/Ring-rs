# vn-runtime 模块摘要

## Purpose

`vn-runtime` 是视觉小说引擎的纯逻辑核心：把脚本文本变成可执行语义，再把执行结果收敛为 Host 可消费的 `Command`。

## PublicSurface

- 对外入口：`vn-runtime/src/lib.rs`
- 核心公开类型：`VNRuntime`、`Command`、`RuntimeInput`、`RuntimeState`、`WaitingReason`
- 核心公开能力：`Parser`、`analyze_script`、`SaveData`

## KeyFlow

1. `Parser` 将脚本文本解析为 `Script`，同时保留 `base_path` 与 source map。
2. `VNRuntime::tick(input)` 驱动执行循环，先消费输入解除等待，再继续执行节点。
3. `Executor` 将 `ScriptNode` 降为 `Command`、等待原因、跳转或跨脚本控制流。
4. `RuntimeState` 保存位置、变量域、等待状态、调用栈和可序列化快照。
5. Host 消费 `Command`，并通过 `RuntimeInput` 将点击、选择、信号或 UI 结果回传给 Runtime。

## Submodules

- [script](vn-runtime/script.md)：脚本 AST、表达式与解析入口
- [runtime](vn-runtime/runtime.md)：执行循环、等待状态与历史记录
- [command](vn-runtime/command.md)：Runtime 到 Host 的通信契约
- [diagnostic](vn-runtime/diagnostic.md)：静态诊断与资源引用提取
- [parser](vn-runtime/parser.md)：两阶段解析实现细节

## Invariants

- 不依赖渲染引擎、文件系统、真实时钟或平台 IO。
- 运行状态显式建模且可序列化，便于存档与恢复。
- Runtime 与 Host 的交互边界收敛在 `Command` / `RuntimeInput` 上。

## FailureModes

- `goto` / `choice` 指向不存在的 label。
- 输入类型与当前等待状态不匹配。
- 选择索引越界。
- 表达式求值失败，或跨脚本目标未注册。

## WhenToReadSource

- 需要确认具体等待解除规则、跳转分支或历史记录行为时。
- 需要新增语法、命令、测试或排查错误类型时。
- 摘要不足以回答字段语义或恢复语义时。

## RelatedDocs

- [摘要索引](../../../maintenance/summary-index.md)
- [仓库导航地图](../navigation-map.md)
- [脚本语法规范](../../../authoring/script-syntax.md)
- [存档格式](../../reference/save-format.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4