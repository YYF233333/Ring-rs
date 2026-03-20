# 摘要抽样验收清单（vn-runtime）

> 目标：验证“仅靠摘要是否足以回答常见只读问题”。

## 使用方式

- 每次抽样 10 题，题目只允许来自 `vn-runtime`。
- 先按 `docs/summary_index.md` 的顺序阅读摘要，不直接读源码。
- 仅当触发升级条件时，才允许回源码，并记录触发原因。

## 评分规则

- `Pass`：仅靠摘要可给出可执行结论（含边界条件/下一步建议）。
- `Partial`：结论基本可用，但缺少关键细节，需要小范围源码核对。
- `Fail`：摘要无法支持结论，必须回源码才能回答。

建议目标：10 题中 `Pass >= 8`。

## 题目模板（10 题）

1. `script`：某条脚本语句最终会映射到哪类 `ScriptNode`？
2. `parser`：相对路径解析依赖哪个入口参数？
3. `runtime`：`tick` 在等待状态下的返回行为是什么？
4. `runtime`：`ChoiceSelected` 非法索引会发生什么？
5. `command`：新增渲染行为时为何通常要扩展 `Command`？
6. `diagnostic`：未定义 label 如何被检查并定位行号？
7. `diagnostic`：资源引用提取覆盖哪些资源类型？
8. `runtime`：等待状态有哪些，Host 如何配合推进？
9. `runtime+history`：哪些命令会被记录到历史事件？
10. `cross-module`：一次脚本执行从解析到命令输出的最短链路是什么？

## 记录模板

```
Date:
Evaluator:

Q1: Pass/Partial/Fail - note
Q2: Pass/Partial/Fail - note
Q3: Pass/Partial/Fail - note
Q4: Pass/Partial/Fail - note
Q5: Pass/Partial/Fail - note
Q6: Pass/Partial/Fail - note
Q7: Pass/Partial/Fail - note
Q8: Pass/Partial/Fail - note
Q9: Pass/Partial/Fail - note
Q10: Pass/Partial/Fail - note

Summary:
- Pass:
- Partial:
- Fail:
- NeedSourceEscalations:
- Actions:
```

## 升级触发统计建议

- 分类记录：`冲突` / `细节不足` / `行为分支` / `需改代码`
- 若同类触发在一轮中 >= 3 次，应优先补该子模块摘要。
