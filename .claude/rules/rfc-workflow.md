---
paths:
  - "RFCs/**"
---

# RFC 流程

## 何时需要 RFC

- 跨模块架构变更
- 脚本语法/语义变更
- Runtime ↔ Host 协议 (Command) 变更
- 新增子系统
- 存档格式不兼容变更

单模块 bug 修复、加测试、不改外部行为的重构不需要 RFC。

## 生命周期

```
Proposed → Active → Accepted（移至 RFCs/Archived/）
    ↓
 Withdrawn
```

## 创建 RFC

1. **编号**：读 `RFCs/README.md` 取最大编号 +1，格式 `RFC-XXX`（三位数）。
2. **文件**：`RFCs/rfc-<kebab-case>.md`。
3. **模板必要章节**：

| 章节 | 内容 |
|------|------|
| 元信息 | 编号、状态 Proposed、作者、日期、范围、前置 |
| 背景 | 问题描述，含具体数据 |
| 目标与非目标 | 明确划定范围边界 |
| 设计 | 技术方案，含子章节 |
| 影响 | 模块 × 改动 × 风险 表格 |
| 迁移计划 | 向后兼容考量 |
| 验收标准 | 具体、可测试的 checklist |

4. **更新索引**：在 `RFCs/README.md` 表格末尾追加行。

## 实施 RFC

1. 更新状态为 `Active`（RFC 文件 + `RFCs/README.md`）。
2. 按设计实施；若需偏离设计，**先更新 RFC 再改代码**。
3. 逐项验证验收标准。

## 完成 RFC

1. 确认所有验收标准满足。
2. 更新状态为 `Accepted`。
3. `git mv RFCs/<file> RFCs/Archived/<file>`。
4. 更新 `RFCs/README.md` 状态列。
5. 同步受影响文档（navigation-map、script-syntax 等）。

## 常见错误

- 实施偏离设计但未更新 RFC → 必须同步回写。
- 验收标准写"能用" → 必须具体（"parser 测试覆盖所有语法变体"）。
- 忘记更新 `RFCs/README.md` → 每次状态变更必须同步。
- 非目标被顺手实现 → 尊重范围边界，需要时单开新 RFC。
