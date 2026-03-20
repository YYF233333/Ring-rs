---
name: script-syntax-extension
description: >-
  Guide for extending the VN script language with new syntax, instructions, or
  inline tags. Covers the two-phase parser architecture, AST design, test
  strategy, diagnostic integration, and spec documentation. Use when adding new
  script keywords, block types, inline tags, or modifying parsing behavior.
---

# Script Syntax Extension

## When to Use

- Adding a new script instruction (e.g. `wait`, `sceneEffect`, `titleCard`).
- Adding a new inline tag (e.g. `{wait}`, `{speed}`).
- Adding a new line modifier (e.g. `-->` auto-advance).
- Modifying existing parsing behavior or error recovery.
- Debugging parser failures or source map misalignment.

## Architecture: Two-Phase Parser

The parser is hand-written (no regex, no parser generators) with two phases:

```
Raw text ──► phase1 ──► Vec<Block> ──► phase2 ──► Vec<ScriptNode> ──► Script
                                                        │
                                                   source_map
```

### Phase 1 — Block Recognition

File: `vn-runtime/src/script/parser/phase1.rs`

Scans raw lines and groups them into typed blocks:

| Block type | Pattern |
|-----------|---------|
| Chapter | Lines starting with `#` |
| Label | Lines matching `**name**` |
| Dialogue | `Name："text"` or `Name: "text"` |
| Narration | `> text` |
| Image/resource | `![](path)` |
| Directive | Single-line instructions (`goto`, `set`, `wait`, etc.) |
| Choice | `- [text](label)` groups |
| Conditional | `if`/`elseif`/`else`/`endif` blocks |

**To add a new block-level instruction**: add recognition logic in phase1 that produces a `Block::Directive` (or new variant if semantically distinct).

### Phase 2 — Semantic Parsing

File: `vn-runtime/src/script/parser/phase2.rs`

Converts each `Block` into a `ScriptNode`, validates parameters, and builds the source map.

**To add a new instruction**: add a match arm in phase2's directive dispatcher that parses parameters and constructs the `ScriptNode`.

### Inline Tags

File: `vn-runtime/src/script/parser/inline_tags.rs`

Processes `{tag}` / `{tag args}` / `{/tag}` within quoted dialogue text.

```
parse_inline_tags(raw_text) -> (clean_text, Vec<InlineEffect>)
```

Returns pure text (tags stripped) plus a list of effects with character-position indices. Called by phase2 when parsing dialogue and extend nodes.

## Step-by-Step: New Block-Level Instruction

### Step 1 — Design the syntax

Draft in `docs/script_syntax_spec.md` first. Follow design principles:

- **Human-friendly**: script authors use Typora/Markdown editors.
- **Tolerant**: ignore trailing spaces, support both `:` and `：`.
- **Markdown-compatible**: don't break preview rendering.
- **Unambiguous**: new syntax must not conflict with existing patterns.

Example spec entry:

```markdown
### N.N instruction_name

\`\`\`markdown
instruction_name arg1 (option: value)
\`\`\`

Description of semantics and parameters.
```

### Step 2 — AST node

File: `vn-runtime/src/script/ast/mod.rs`

Add a `ScriptNode` variant. Rules:

- Carry only **semantic** data (no rendering details).
- Use domain types from `command/mod.rs` where they exist (e.g. `Transition`, `Position`).
- Derive `Debug, Clone, PartialEq` for testability.

### Step 3 — Phase 1 recognition

File: `vn-runtime/src/script/parser/phase1.rs`

Most new instructions fit the existing `Block::Directive` path. You only need a new `Block` variant if the instruction spans multiple lines or has fundamentally different structure.

### Step 4 — Phase 2 parsing

File: `vn-runtime/src/script/parser/phase2.rs`

1. Add a match arm in the directive dispatcher.
2. Parse parameters using helpers from `parser/helpers.rs`.
3. Produce the `ScriptNode` variant.
4. Source map entry is typically automatic — the phase2 loop tracks line positions.

### Step 5 — Parser tests

Location: find the test module with `rg "#\[cfg\(test\)\]" vn-runtime/src/script/parser/`

Write tests covering:

- **Happy path**: valid syntax produces expected `ScriptNode`.
- **Tolerance**: extra spaces, alternate punctuation still parse correctly.
- **Error cases**: missing required parameters produce `ParseError`.
- **Edge cases**: empty arguments, unicode characters, adjacent instructions.

Pattern:

```rust
#[test]
fn parse_instruction_name_basic() {
    let script = parse("instruction_name arg1");
    assert_eq!(script.nodes[0], ScriptNode::InstructionName { ... });
}
```

### Step 6 — Connect to executor

File: `vn-runtime/src/runtime/executor/mod.rs`

Add a match arm: `ScriptNode::InstructionName { .. } => { ... }`.

If this instruction produces a new Command, see the [cross-module-command-pipeline](../cross-module-command-pipeline/SKILL.md) skill for the full host-side flow.

### Step 7 — Diagnostics

File: `vn-runtime/src/diagnostic/mod.rs`

If the new syntax:

- References **resources** (images, audio) → update `extract_resource_references`.
- References **labels** → update jump target analysis in `analyze_script`.
- Has **common mistakes** → add a warning-level diagnostic rule.

### Step 8 — Verify and document

```bash
cargo check-all
```

Update:

1. `docs/script_syntax_spec.md` — formal syntax entry.
2. `docs/module_summaries/vn-runtime/parser.md` — add to KeyFlow list.
3. `docs/module_summaries/vn-runtime/script.md` — mention new node.

## Step-by-Step: New Inline Tag

### Step 1 — Design in spec

Add to the inline tags section of `docs/script_syntax_spec.md`.

### Step 2 — InlineEffectKind

File: `vn-runtime/src/command/mod.rs`

Add a variant to `InlineEffectKind`:

```rust
pub enum InlineEffectKind {
    Wait(Option<f32>),
    SetCpsAbsolute(f32),
    SetCpsRelative(f32),
    ResetCps,
    NewTagName { /* fields */ },  // ← add here
}
```

### Step 3 — Parse the tag

File: `vn-runtime/src/script/parser/inline_tags.rs`

Add recognition in the tag dispatcher. The function scans for `{tag_name ...}` patterns and converts them to `InlineEffect` entries with character positions.

### Step 4 — Host consumption

The host processes inline effects in the typewriter system:

- `host/src/renderer/render_state/mod.rs` — `advance_typewriter` checks `InlineEffectKind`.
- `host/src/app/update/modes.rs` — frame-level effect timers.

Add handling for the new `InlineEffectKind` variant in both locations.

### Step 5 — Tests

- Parser: `parse_inline_tags` unit test with the new tag.
- Executor: verify the tag survives the `ScriptNode` → `Command` mapping.
- Host: if the tag has visible behavior, verify in typewriter state tests.

## Checklist Template

```
New Syntax: [instruction/tag name]
- [ ] Syntax designed in script_syntax_spec.md
- [ ] ScriptNode / InlineEffectKind variant added
- [ ] phase1 recognition (if needed)
- [ ] phase2 parsing / inline_tags parsing
- [ ] Parser tests (happy + error + edge)
- [ ] Executor mapping
- [ ] Diagnostic rules (if references resources/labels)
- [ ] cargo check-all passes
- [ ] Spec and summaries updated
```

## Common Pitfalls

- **Breaking Markdown preview**: New syntax that looks like Markdown formatting (e.g. using `*` or `_`) will confuse Typora. Test in a Markdown editor.
- **Phase 1/2 mismatch**: If phase1 doesn't recognize the line, phase2 never sees it. Unrecognized lines are silently skipped or treated as narration.
- **Source map drift**: Multiline constructs (like `choice` blocks) need careful line tracking. Verify with `diagnostic` tests.
- **base_path sensitivity**: Resource paths in new syntax are relative to the script file. Use `parse_with_base_path` in tests, or verify `extract_resource_references` handles the new node.
- **Tolerance gaps**: Forgetting to handle `：` (Chinese colon), leading/trailing spaces, or missing optional parameters.
