# vn-runtime/parser 摘要

## Purpose

解析脚本文本到 `Script` AST，采用两阶段手写解析器，保证可控错误信息与行号追踪。

## PublicSurface

- 入口：`vn-runtime/src/script/parser/mod.rs`
- 核心接口：`Parser::parse`、`Parser::parse_with_base_path`、`Parser::warnings`
- 子模块：`phase1`、`phase2/`（目录：`mod.rs` 分发 + `display` / `control` / `dialogue` / `misc` 按域拆分）、`helpers`、`expr_parser`；`inline_tags` 为 crate 内（`pub(crate)`）

## KeyFlow

1. `phase1` 将原始文本识别为块结构（`Vec<Block>`）。
2. `phase2` 子目录将块解析为 `ScriptNode`：`mod.rs` 聚合入口，具体语法按域落在 `display` / `control` / `dialogue` / `misc`。
3. 解析过程中同步构建 source map（节点索引 -> 源文件行号）。
4. 使用 `Script::with_source_map` 产出最终脚本对象。
5. 阶段 0 新增 `callScript` / `returnFromScript` 单行语法解析。
6. `wait <duration>` 解析为 `ScriptNode::Wait`。
7. phase2 解析 `pause` -> `ScriptNode::Pause`。
8. phase2 解析 `sceneEffect name(args...)` -> `ScriptNode::SceneEffect { effect: Transition }`。
9. phase2 解析 `titleCard "text" (duration: N)` -> `ScriptNode::TitleCard { text, duration }`。
10. `inline_tags` 子模块提供 `parse_inline_tags(raw) -> (String, Vec<InlineEffect>)`，提取 `{wait}`/`{speed}`/`{/speed}` 标签为位置索引效果列表，返回纯文本。
11. phase2 解析对话行时调用 `parse_inline_tags` 处理内联标签，并检测 `-->` 行尾修饰符设置 `no_wait`。
12. phase2 解析 `extend "text"` -> `ScriptNode::Extend { content, inline_effects, no_wait }`。

## Dependencies

- 依赖 `script/ast` 承载语义节点。
- 依赖 `command` 类型承载过渡参数等结构。
- 输出结果被 `runtime` 与 `diagnostic` 复用。

## Invariants

- 解析器不依赖 regex，主要使用手写字符串处理。
- `parse_with_base_path` 负责相对路径上下文，影响资源引用解析。
- `warnings` 与硬错误分离：尽量容错，不阻塞有效节点产出。

## FailureModes

- 语法结构错误导致 `ParseError`。
- base_path 设置错误导致资源逻辑路径不正确。
- phase2 规则变更可能影响 source map 准确性。

## WhenToReadSource

- 增加语法关键字或新块类型时。
- 排查脚本报错行号、告警行为或路径解析问题时。

## RelatedDocs

- [script 子模块摘要](script.md)
- [脚本语法规范](../../script_syntax_spec.md)
- [模块总览](../vn-runtime.md)

## LastVerified

2026-03-20

## Owner

Composer