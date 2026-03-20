# Examples

> All examples below use the project's 6 pre-defined domain partitions. See `SKILL.md § Domain Partitions` for the full table.

## Example 1: Full-Repo Code Review

Task:

- Review a large repository for bugs, regressions, and missing tests.

Good partition (aligned with domain partitions):

- Subagent 1: domain `script-lang` — script parsing, AST, command definitions
- Subagent 2: domain `runtime-engine` — state transitions and command execution
- Subagent 3: domain `renderer` — rendering pipeline, effects, GPU backend
- Subagent 4: domain `host-app` — command executor, app lifecycle, update loop
- Subagent 5: domain `resources` — resource manager, config, manifest, save manager
- Subagent 6: domain `media-ui` — audio, video, UI, extensions
- Subagent 7: CI, test harness, and workflow files (no domain — standalone slice)

Run as 2 waves (4 + 3).

Subagent roles:

- Subagents 1–6: `/investigator` (inherit, readonly) for standard review slices. Each prompt starts with `Read .cursor/rules/domain-{id}.mdc for invariants.`
- Subagent 7: `/worker` (composer-2-fast) for CI/workflow file review — mostly config, low reasoning needed.
- Upgrade any chunk to `/investigator` (inherit, readonly) if it involves cross-module invariants or subtle behavior.

Expected parent-agent synthesis:

- Deduplicate findings that appear in multiple chunks.
- Group the final report by severity.
- Highlight repo-wide themes such as missing integration tests or repeated error-handling issues.
- Spot-check at least one high-severity finding per chunk against actual source.

## Example 2: Repo-Wide Test Updates

Task:

- Add or refresh tests after a broad behavior change across many subsystems.

Good partition (aligned with domain partitions):

- Subagent 1: domain `script-lang` — parser and command tests
- Subagent 2: domain `runtime-engine` — engine, executor, state tests
- Subagent 3: domain `renderer` — renderer, animation, effects, backend tests
- Subagent 4: domain `host-app` — command executor, app update tests
- Subagent 5: domains `resources` + `media-ui` (merged — both small test surface) — resource, audio, UI tests
- Subagent 6: docs and test inventory updates

Subagent roles:

- Subagents 1–5: `/worker` (composer-2-fast) for simple test additions; upgrade to `/coder` (inherit) if the chunk requires reasoning about state machines or cross-module contracts.
- Subagent 6: `/worker` (composer-2-fast) for docs and test inventory.

Expected parent-agent synthesis:

- Run `cargo check` / `cargo test` after merging all subagent changes to catch cross-chunk breakage.
- Merge all touched files into one coherent change summary.
- Note which test commands passed and which were not run.
- Surface any remaining gaps that need integration coverage.

## Example 3: Multi-Wave Execution

If there are 9 independent chunks:

1. Launch 4 subagents in wave 1.
2. Collect results. Check compilation/test status. If wave 1 revealed new shared constraints (e.g., a public API signature changed), propagate them into wave 2 prompts. If two chunks turned out to be coupled, merge them into one subagent for wave 2.
3. Launch up to 4 subagents in wave 2.
4. Repeat: collect, validate, propagate.
5. Launch remaining subagents in wave 3.
6. Final synthesis after all waves complete.

If the first 1–2 waves produce heavy conflicts or require the same large shared context, stop launching new waves and fall back to single-agent execution for the remaining work.

## Example 4: Module Summary / Doc-Code Audit

Task:

- Update module summaries (`docs/module_summaries/`) to match current source code across all modules.

Good partition (aligned with domain partitions):

- Subagent 1: domain `script-lang` summaries (script, command, diagnostic, parser)
- Subagent 2: domain `runtime-engine` summaries (runtime, vn-runtime overview)
- Subagent 3: domain `renderer` summaries (renderer, render_state, animation, effects, scene_transition, rendering_types, backend)
- Subagent 4: domain `host-app` summaries (app, app_update, app_command_handlers, command_executor, host_app, egui_actions)
- Subagent 5: domains `resources` + `media-ui` summaries (resources, manifest, config, save_manager, audio, video, input, ui, egui_screens, extensions)
- Subagent 6: cross-cutting docs (`navigation_map.md`, `summary_index.md`, test inventory)

Subagent roles:

- Simple summary updates: `/worker` (composer-2-fast). Mechanical diffing and text alignment.
- Semantic rewrites after architecture changes: `/coder` (inherit). Needs to understand code intent to produce accurate summaries.

Expected parent-agent synthesis:

- Verify no conflicting edits to shared index files.
- Run a quick scan for stale cross-references between summaries.
- Produce a changelog of what was updated.

Why this fits well:

- Each module's summary is independent.
- The work is repetitive and well-scoped.
- Serial execution is painfully slow because the agent re-reads the same boilerplate context for every module.

## Example 5: Parallel Hypothesis Debugging

Task:

- A bug manifests but the root cause is unclear. There are 3 independent hypotheses.

Good partition:

- Subagent 1: Investigate hypothesis A — state machine transition drops an event under concurrent input.
- Subagent 2: Investigate hypothesis B — resource path resolution returns a stale cache entry.
- Subagent 3: Investigate hypothesis C — command executor skips a side effect when a flag combination is unexpected.

Subagent roles:

- All chunks: `/investigator` (inherit parent, readonly). Debugging requires careful code reading and cross-module reasoning; cheaper models are more likely to miss subtle issues or hallucinate evidence.

Expected parent-agent synthesis:

- Collect evidence for/against each hypothesis.
- If one subagent found the root cause, confirm it by cross-checking with the other subagents' observations.
- If no subagent found a definitive answer, synthesize partial clues into a narrowed investigation plan.

Why this fits well:

- This is not splitting one task into pieces — it is running independent investigations in parallel. A single agent would test hypotheses sequentially, losing earlier context as it digs into each one.
- Each hypothesis touches different code paths with minimal overlap.

## Example 6: Cross-Repo Rename / Migration

Task:

- Rename a widely-used type or concept across the entire repository (e.g., `ScriptCommand` → `Directive`).

Good partition (aligned with domain partitions):

- Subagent 1: domain `script-lang` — update type definitions, parser logic, and tests (likely owns the renamed type)
- Subagent 2: domain `runtime-engine` — update consumers, state transitions, and tests
- Subagent 3: domain `host-app` — update command executor and app lifecycle consumers and tests
- Subagent 4: domain `renderer` — update rendering consumers and tests
- Subagent 5: domains `resources` + `media-ui` — update remaining host modules
- Subagent 6: docs, config files, and CI references

Subagent roles:

- Most chunks: `/worker` (composer-2-fast). Mechanical find-and-replace with compilation check.
- The chunk owning the type definition (usually `script-lang`): `/coder` (inherit) if the rename involves structural changes beyond simple renaming.
- The docs/CI chunk: `/worker` (composer-2-fast). Pure text replacement.

Expected parent-agent synthesis:

- Run `cargo check` across the whole workspace to catch cross-crate breakage.
- Run `cargo test` to verify behavior is unchanged.
- Check that no stale references remain via a repo-wide grep.

## Example 7: When Not To Use This Skill

Do not use this skill when:

- The task only touches 2 to 3 tightly coupled files.
- A single design decision must be made before any chunk can proceed.
- Every chunk would need the same large shared context anyway.

In those cases, a single agent is usually faster and safer.
