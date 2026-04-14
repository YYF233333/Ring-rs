# Subagent 选型路由

派发子任务时，根据任务特征选择正确的 agent 类型和模型。

## 模型分档

| 档位 | model 参数 | 适用场景 |
|------|-----------|----------|
| **fast** | `haiku` | 机械/简单任务：模式匹配即可完成，范围明确，结果可验证 |
| **standard** | `sonnet` | 中等复杂度：搜索+理解，需要一定推理但不需最强模型 |
| **strong** | 默认（继承 opus） | 复杂任务：跨上下文推理、设计决策、深度代码理解 |

## 任务→Agent 映射

### 只读任务

| 任务特征 | subagent_type | model | 示例 |
|----------|--------------|-------|------|
| 搜索文件/符号/定位代码 | `Explore` | — | "Command 在哪些文件中被 match" |
| 简单假设验证（单模块内） | `Explore` | — | "确认 X 字段是否参与序列化" |
| 风格/规范一致性检查 | `Explore` | — | "检查新文件是否遵循命名规范" |
| 深度根因分析/跨模块推理 | `general-purpose` | `opus` | "为什么切换场景后音频未停止" |
| 正确性/安全性审计 | `general-purpose` | `opus` | "审查新增的 parser 分支" |
| 架构设计/方案评估 | `Plan` | — | "设计 host-dioxus 的渲染架构" |

### 写入任务

| 任务特征 | subagent_type | model | 示例 |
|----------|--------------|-------|------|
| 机械批量操作（重命名/替换） | `general-purpose` | `sonnet` | "将所有 old_name 改为 new_name" |
| 简单代码变更（补测试/加字段） | `general-purpose` | `sonnet` | "为 parse_label 补边界测试" |
| 文档修正（链接/字段名更新） | `general-purpose` | `sonnet` | "更新文档中的链接" |
| 非简单实现（新特性/设计决策） | `general-purpose` | `opus` | "实现 @voice 指令的 executor" |
| 多步骤混合任务（搜索+修改+验证） | `general-purpose` | `opus` | "定位 bug → 修复 → 补测试" |

## 并行模式

跨模块改动（如新增 Command）时，可并行启动多个 subagent：

```
# 示例：并行探索 runtime + host-dioxus 的 Command 处理
Agent(Explore): "在 vn-runtime/src/command/ 中找到 Command 枚举所有变体"
Agent(Explore): "在 host-dioxus/src/command_executor.rs 中找到所有 Command match 分支"
```

对于独立子任务，在**同一条消息**中发起多个 Agent 调用实现并行。

## 决策流程

1. **是否需要深度推理/设计决策？** → 是：只读用 `general-purpose`+opus（investigator），需写入用 `general-purpose`+opus（coder）
2. **是否需要架构规划？** → 是：`Plan`
3. **是否纯代码探索？** → 是：`Explore`
4. **否 → `general-purpose`+sonnet**（worker）

## 反模式

- 用 opus 做批量替换 → 用 sonnet，不涉及代码逻辑理解
- 用 Explore 做修改 → Explore 是只读的，需要修改用 general-purpose
- 串行启动可并行的 subagent → 同一消息中并行发起
- 单个简单任务启动 subagent → 直接用 Grep/Glob/Read 即可
