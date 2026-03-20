---
name: cross-module-command-pipeline
description: >-
  Step-by-step guide for adding or modifying a Command that spans the full
  vn-runtime → host pipeline (parser → AST → executor → Command → CommandExecutor
  → command_handlers). Use when adding a new script instruction, extending an
  existing Command variant, or tracing a Command across module boundaries.
---

# Cross-Module Command Pipeline

## When to Use

- Adding a new script instruction that produces a Command for the host.
- Extending an existing Command variant with new fields.
- Tracing end-to-end behavior of a Command across crate boundaries.
- Debugging "command executed but nothing happened" issues.

## Pipeline Overview

```
Script text
  │  ┌─────────────────────────────────────┐
  ▼  │ vn-runtime (pure logic, no IO)      │
phase1 ──► Block                            │
  │                                         │
phase2 ──► ScriptNode (AST)                 │
  │                                         │
Executor ──► Command (+ WaitingReason)      │
  └─────────────────────────────────────────┘
  │  ┌─────────────────────────────────────┐
  ▼  │ host (rendering, audio, IO)         │
CommandExecutor ──► RenderState Δ + Output  │
  │                                         │
command_handlers ──► Audio / Effects / UI    │
  └─────────────────────────────────────────┘
```

## Step-by-Step: Adding a New Command

### Step 0 — Orientation

Read summaries before touching source:

| What | Summary |
|------|---------|
| Command contract | `docs/engine/architecture/module-summaries/vn-runtime/command.md` |
| Parser | `docs/engine/architecture/module-summaries/vn-runtime/parser.md` |
| Executor | `docs/engine/architecture/module-summaries/vn-runtime/runtime.md` |
| CommandExecutor | `docs/engine/architecture/module-summaries/host/command-executor.md` |
| Command handlers | `docs/engine/architecture/module-summaries/host/app-command-handlers.md` |

Then `rg` for a recent similar Command (e.g. `SceneEffect`, `TitleCard`, `ExtendText`) to see the pattern.

### Step 1 — Define the syntax (if new instruction)

1. Draft the syntax in `docs/authoring/script-syntax.md`.
2. Decide: block-level instruction or inline tag?
   - Block-level → phase1 needs a new `Block` variant, phase2 parses it.
   - Inline tag → `parser/inline_tags.rs`, `InlineEffectKind` in `command/mod.rs`.

### Step 2 — AST node

File: `vn-runtime/src/script/ast/mod.rs`

Add or extend a `ScriptNode` variant. Carry only semantic data, never host-specific details.

```rust
// Example: ScriptNode::TitleCard { text: String, duration: f32 }
```

### Step 3 — Parser

Files: `vn-runtime/src/script/parser/phase1.rs`, `phase2.rs`

1. **phase1**: recognize the raw line/block pattern → produce a `Block` variant.
2. **phase2**: convert `Block` → `ScriptNode`, validate parameters.
3. Source map: ensure the new node is covered by `source_map` (usually automatic if you follow existing patterns).
4. **Tests**: add parser round-trip tests in the parser test module.

### Step 4 — Command enum

File: `vn-runtime/src/command/mod.rs`

1. Add or extend a `Command` variant.
2. If the command needs host-side waiting, add a `SIGNAL_*` constant.
3. Keep the variant data minimal — only what the host needs to act.

### Step 5 — Executor

File: `vn-runtime/src/runtime/executor/mod.rs`

1. Add a match arm for the new `ScriptNode` → produce the `Command`.
2. If the command should block script progression, set `ExecuteResult::waiting`.
3. **Tests**: add executor unit tests verifying correct Command output.

### Step 6 — CommandExecutor (host side)

Files: `host/src/command_executor/mod.rs` + relevant sub-module

1. Add a match arm in `execute()` dispatching to a handler function.
2. The handler updates `RenderState` and writes to `self.last_output`.
3. If the command needs a new sub-module (e.g. `host/src/command_executor/effects.rs`), create it and re-export.
4. If a wait signal is needed, set `ExecuteOutput::wait_signal`.

### Step 7 — Command handlers (if side-effects needed)

Files: `host/src/app/command_handlers/`

1. If the command produces audio, effect, or animation outputs, add handling in the appropriate sub-module (`audio.rs`, `effect_applier.rs`).
2. Consume the output from `CommandExecutor::last_output`.

### Step 8 — Diagnostics (if applicable)

File: `vn-runtime/src/diagnostic/mod.rs`

If the new syntax references resources or labels, update `extract_resource_references` or `analyze_script`.

### Step 9 — Verify

```bash
cargo check-all
# if failed, see detailed log
cargo test -p vn-runtime --lib
```

### Step 10 — Update docs

1. `docs/authoring/script-syntax.md` — add syntax documentation.
2. Module summaries — update affected summaries (command, parser, command_executor, etc.).
3. `docs/engine/architecture/navigation-map.md` — add to "常见改动" if it establishes a new pattern.

## Checklist Template

Copy this and track progress:

```
New Command: [name]
- [ ] Syntax drafted in script_syntax_spec.md
- [ ] ScriptNode variant added in script/ast/mod.rs
- [ ] phase1 block recognition
- [ ] phase2 parsing + tests
- [ ] Command variant added in command/mod.rs
- [ ] Executor mapping + tests
- [ ] CommandExecutor handler (host)
- [ ] command_handlers side-effects (if needed)
- [ ] Diagnostic rules (if needed)
- [ ] cargo check-all passes
- [ ] Module summaries updated
```

## Extending an Existing Command

When adding fields to an existing variant (e.g. adding `inline_effects` to `ShowText`):

1. Update `Command` variant in `command/mod.rs`.
2. Update `Executor` to populate the new field.
3. Update `CommandExecutor` to consume it.
4. Update downstream handlers if the new field produces side-effects.
5. Check `save.rs` — if the field affects saveable state, update save/restore logic.
6. Update tests at each layer.

## Common Pitfalls

- **Forgetting the host side**: Adding a Command variant without a `CommandExecutor` match arm → runtime panic on `unreachable!`.
- **Signal mismatch**: `SIGNAL_*` constant string must match between `runtime/executor/mod.rs` (wait setup) and `command_executor` (signal emission).
- **Transition args**: Use structured `Transition`/`TransitionArg` types, not raw strings.
- **Source map drift**: If phase1/phase2 change line grouping, source map can misalign — verify with diagnostic tests.

## Reference: Recent Command Additions

Search the codebase for these patterns to see complete examples:

| Command | RFC/Feature | Key files |
|---------|-------------|-----------|
| `SceneEffect` | Effects system | `ast/mod.rs`, `executor/mod.rs`, `command/mod.rs`, `command_executor/effects.rs` |
| `TitleCard` | Title card display | Same pipeline |
| `ExtendText` | RFC-006 rhythm tags | `inline_tags.rs`, `ast/mod.rs`, `executor/mod.rs`, `command/mod.rs`, `command_executor/ui.rs` |
| `BgmDuck/Unduck` | Audio ducking | `executor/mod.rs`, `command/mod.rs`, `command_executor/audio.rs` |
