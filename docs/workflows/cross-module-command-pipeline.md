# Cross-Module Command Pipeline

## When to Use

- Adding a new script instruction that produces a Command for the host.
- Extending an existing Command variant with new fields.
- Tracing end-to-end behavior of a Command across crate boundaries.
- Debugging "command executed but nothing happened" issues.

## Pipeline Overview

```
Script text
  |  +-------------------------------------+
  v  | vn-runtime (pure logic, no IO)      |
phase1 --> Block                            |
  |                                         |
phase2 --> ScriptNode (AST)                 |
  |                                         |
Executor --> Command (+ WaitingReason)      |
  +-----------------------------------------+
  |  +-------------------------------------+
  v  | host-dioxus (rendering, audio, IO)  |
CommandExecutor --> RenderState + Output    |
  +-----------------------------------------+
```

## Step-by-Step: Adding a New Command

### Step 0 -- Orientation

Search for a recent similar Command (e.g. `SceneEffect`, `TitleCard`, `ExtendText`) to see the pattern.

### Step 1 -- Define the syntax (if new instruction)

1. Draft the syntax in `docs/authoring/script-syntax.md`.
2. Decide: block-level instruction or inline tag?
   - Block-level: phase1 needs a new `Block` variant, phase2 parses it.
   - Inline tag: `parser/inline_tags.rs`, `InlineEffectKind` in `command/mod.rs`.

### Step 2 -- AST node

File: `vn-runtime/src/script/ast/mod.rs`

Add or extend a `ScriptNode` variant. Carry only semantic data, never host-specific details.

```rust
// Example: ScriptNode::TitleCard { text: String, duration: f32 }
```

### Step 3 -- Parser

Files: `vn-runtime/src/script/parser/phase1.rs`, `phase2.rs`

1. **phase1**: recognize the raw line/block pattern -> produce a `Block` variant.
2. **phase2**: convert `Block` -> `ScriptNode`, validate parameters.
3. Source map: ensure the new node is covered by `source_map` (usually automatic if you follow existing patterns).
4. **Tests**: add parser round-trip tests in the parser test module.

### Step 4 -- Command enum

File: `vn-runtime/src/command/mod.rs`

1. Add or extend a `Command` variant.
2. If the command needs host-side waiting, add a `SIGNAL_*` constant.
3. Keep the variant data minimal -- only what the host needs to act.

### Step 5 -- Executor

File: `vn-runtime/src/runtime/executor/mod.rs`

1. Add a match arm for the new `ScriptNode` -> produce the `Command`.
2. If the command should block script progression, set `ExecuteResult::waiting`.
3. **Tests**: add executor unit tests verifying correct Command output.

### Step 6 -- CommandExecutor (host-dioxus)

File: `host-dioxus/src/command_executor.rs`

1. Add a match arm in `execute()` dispatching to a handler function.
2. The handler updates `RenderState`.

### Step 7 -- Diagnostics (if applicable)

File: `vn-runtime/src/diagnostic/mod.rs`

If the new syntax references resources or labels, update `extract_resource_references` or `analyze_script`.

### Step 8 -- Verify

```bash
cargo check-all
# if failed, see detailed log
cargo test -p vn-runtime --lib
```

### Step 9 -- Update docs

1. `docs/authoring/script-syntax.md` -- add syntax documentation.
2. `docs/engine/architecture/navigation-map.md` -- add to common patterns if new.

## Checklist Template

Copy this and track progress:

```
New Command: [name]
- [ ] Syntax drafted in script-syntax.md
- [ ] ScriptNode variant added in script/ast/mod.rs
- [ ] phase1 block recognition
- [ ] phase2 parsing + tests
- [ ] Command variant added in command/mod.rs
- [ ] Executor mapping + tests
- [ ] CommandExecutor handler (host-dioxus)
- [ ] Diagnostic rules (if needed)
- [ ] cargo check-all passes
```

## Extending an Existing Command

When adding fields to an existing variant (e.g. adding `inline_effects` to `ShowText`):

1. Update `Command` variant in `command/mod.rs`.
2. Update `Executor` to populate the new field.
3. Update `CommandExecutor` to consume it.
4. Check `save.rs` -- if the field affects saveable state, update save/restore logic.
5. Update tests at each layer.

## Common Pitfalls

- Repository-wide recurring pitfalls (`SIGNAL_*` mismatches, source map drift) are tracked in `docs/maintenance/lessons-learned.md`.
- **Transition args**: Use structured `Transition`/`TransitionArg` types, not raw strings.

## Reference: Recent Command Additions

Search the codebase for these patterns to see complete examples:

| Command | RFC/Feature | Key files |
|---------|-------------|-----------|
| `SceneEffect` | Effects system | `ast/mod.rs`, `executor/mod.rs`, `command/mod.rs`, `command_executor.rs` |
| `TitleCard` | Title card display | Same pipeline |
| `ExtendText` | RFC-006 rhythm tags | `inline_tags.rs`, `ast/mod.rs`, `executor/mod.rs`, `command/mod.rs`, `command_executor.rs` |
| `BgmDuck/Unduck` | Audio ducking | `executor/mod.rs`, `command/mod.rs`, `command_executor.rs` |
