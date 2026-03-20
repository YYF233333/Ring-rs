---
name: rfc-workflow
description: >-
  Guides the RFC process for cross-module changes, syntax/semantic alterations,
  and protocol modifications. Covers when to write an RFC, the template, status
  lifecycle, and post-implementation sync. Use when proposing or implementing
  architectural changes, new features spanning multiple modules, or when the user
  mentions RFC.
---

# RFC Workflow

## When an RFC is Required

Per project rules (`CLAUDE.md`), the following changes require an RFC:

- Cross-module architectural changes.
- Script syntax or semantic changes.
- Runtime ↔ Host protocol (Command) changes.
- New subsystem introduction.
- Changes that affect save format compatibility.

## When an RFC is NOT Required

- Bug fixes within a single module.
- Adding tests or documentation.
- Refactors that don't change external behavior or module boundaries.
- Small feature additions contained within one module.

If uncertain whether an RFC is needed, ask the user.

## RFC Lifecycle

```
Proposed ──► Active ──► Accepted (move to Accepted/)
    │                       │
    └──► Withdrawn          └──► update RFCs/README.md
```

| Status | Meaning |
|--------|---------|
| Proposed | Draft, open for discussion |
| Active | Approved for implementation, work in progress |
| Accepted | Fully implemented, moved to `RFCs/Accepted/` |
| Withdrawn | Abandoned (keep file for history) |

## Step-by-Step: Writing an RFC

### Step 1 — Assign a number

Check `RFCs/README.md` for the highest existing RFC number. New RFC gets the next sequential number (three-digit format: `RFC-001`, `RFC-014`, `RFC-015`).

### Step 2 — Create the file

File: `RFCs/rfc-<topic>.md`

Naming convention: lowercase, hyphens, descriptive slug. Examples:

- `rfc-dialogue-voice-pipeline.md`
- `rfc-cutscene-video-playback.md`
- `rfc-config-externalization.md`

### Step 3 — Write the RFC

Use this template:

```markdown
# RFC: [Title]

## 元信息

- 编号：RFC-XXX
- 状态：Proposed
- 作者：[model name]
- 日期：YYYY-MM-DD
- 相关范围：[affected crates/modules]
- 前置：[dependencies, if any]

---

## 背景

[Why is this change needed? What problem does it solve?
Include concrete data: how many places are affected, what user-facing
behavior is broken or missing.]

---

## 目标与非目标

### 目标

- [Concrete deliverable 1]
- [Concrete deliverable 2]

### 非目标

- [Explicitly out of scope item 1]
- [Item 2 — with reason why it's deferred]

---

## 方案设计

[Technical design. Include:]

### [Design aspect 1]

[Details, code examples, data structures]

### [Design aspect 2]

[...]

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| [module] | [what changes] | [risk level] |

---

## 迁移计划

[How to get from current state to proposed state.
Include backward compatibility considerations.]

---

## 验收标准

- [ ] [Criterion 1: specific, testable]
- [ ] [Criterion 2]
- [ ] [Tests pass: `cargo check-all`]
- [ ] [Documentation updated]
```

### Step 4 — Update the index

File: `RFCs/README.md`

Add a row to the table:

```markdown
| RFC-XXX | [Name] | `rfc-<topic>.md` | Proposed |
```

### Step 5 — Align before implementation

Before writing code, ensure the RFC content is agreed upon. If working with a user, present the key design decisions for confirmation.

## Step-by-Step: Implementing an RFC

### Step 1 — Mark as Active

Update status in both the RFC file and `RFCs/README.md`:

```
- 状态：Active
```

### Step 2 — Implement

Follow the RFC's design. If implementation reveals that the design needs changes:

1. **Update the RFC first** — document the deviation and rationale.
2. Then update the code.

Never let code drift from the RFC without updating the RFC.

### Step 3 — Verify acceptance criteria

Check every item in the RFC's 验收标准 section. Each criterion should be demonstrably met.

### Step 4 — Mark as Accepted

1. Update status to `Accepted` in the RFC file.
2. Move the file: `RFCs/rfc-<topic>.md` → `RFCs/Accepted/rfc-<topic>.md`.
3. Update `RFCs/README.md` — change status, no need to modify file name.

### Step 5 — Sync documentation

Update all affected docs:

- Module summaries in `docs/engine/architecture/module-summaries/`.
- `docs/engine/architecture/navigation-map.md` if new patterns or modules were introduced.
- `docs/authoring/script-syntax.md` if syntax changed.
- Any other relevant docs in `docs/`.

## Quality Checklist

```
RFC: [title]
- [ ] Number assigned, not conflicting
- [ ] File created at RFCs/rfc-<topic>.md
- [ ] All template sections filled
- [ ] 目标/非目标 clearly separated
- [ ] 影响范围 table covers all affected modules
- [ ] 验收标准 are specific and testable
- [ ] RFCs/README.md index updated
- [ ] Status transitions tracked
```

## Common Pitfalls

- **Scope creep in implementation**: The RFC says "non-goal" but the implementer adds it anyway. Respect non-goals; create a follow-up RFC if needed.
- **Stale RFC**: Code was implemented differently but RFC was never updated. Always sync back.
- **Missing migration plan**: Large changes without a migration path break existing scripts or saves.
- **Vague acceptance criteria**: "It works" is not a criterion. "Parser test covers all syntax variants" is.
- **Forgetting the index**: `RFCs/README.md` must stay current — it's the discovery entry point.

## Reference: Existing RFC Patterns

Browse `RFCs/Accepted/` for examples of well-structured RFCs:

| RFC | Good example of |
|-----|----------------|
| RFC-006 (rhythm tags) | Syntax design with Ren'Py comparison, phased implementation |
| RFC-008 (render backend trait) | Trait abstraction design, migration from concrete to trait-based |
| RFC-009 (cutscene video) | New subsystem introduction with external dependency |
| RFC-013 (config externalization) | Config schema design, backward compatibility |
| RFC-014 (test tiering) | Test strategy and quality gates |
