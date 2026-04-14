---
name: parallel-subagent-orchestration
description: Breaks large, mostly independent work into parallel subagent batches to reduce context sharing and improve throughput. Use when tasks span many files or modules with weak coupling, such as full-repo code review, repo-wide test updates, broad audits, or parallel investigations.
---

# Parallel Subagent Orchestration

## Use This Skill When

- The task spans many files, modules, or subsystems.
- The work can be split into mostly independent chunks.
- Typical cases: full-repo code review, repo-wide test updates, broad audits, migration prep, or parallel investigations.
- Do not use this skill for small tasks or tightly coupled changes that need constant shared context.

## Workflow

1. Route the task to the right topology. Not everything benefits from parallelism.

   | Task shape | Topology | Rationale |
   |------------|----------|-----------|
   | Repo-wide review, audit, or broad test updates | **Parallel** (this skill) | Chunks are independent; no shared mutation. |
   | Scoped refactor or rename with cross-file dependencies | **Sequential or single-agent** | Shared invariants require ordering. |
   | Debugging / root cause analysis | **Single investigator** | Hypothesis chains are inherently serial. |
   | Feature implementation spanning multiple domains | **Planner → parallel workers** | Parent designs the contract, workers implement per-domain. |

   If the task does not clearly fit "parallel," stop here and handle it without this skill.
2. Do a minimal top-level scan only. Gather the module map, constraints, and final deliverable.
3. Partition the work into independent chunks. Prefer module, directory, feature area, or test bucket boundaries.
4. Choose subagent count.
   - Start from the number of independent chunks.
   - Launch up to 4 subagents concurrently per wave.
   - If there are more than 4 chunks, run multiple waves.
   - Keep one chunk per subagent; do not mix unrelated work.
5. Choose the subagent role and model per chunk. Pre-defined subagent files live in `.cursor/agents/`. Pick from the table below or use the Task tool with an inline model choice.

   | Role | Subagent | Model | When to use |
   |------|----------|-------|-------------|
   | Simple/mechanical | `worker` | composer-2-fast | Renames, grep-replace, file inventory, simple code changes, doc fixes, style checks, test additions |
   | Complex implementation | `coder` | inherit | Non-trivial features, design decisions, cross-module refactors, semantic doc rewrites |
   | Deep analysis (readonly) | `investigator` | inherit | Cross-module debugging, root cause analysis, correctness/safety audits |
   | Search/locate | `explore` | composer-1.5（固定，不可覆盖） | 仅当需要 Cursor 内置语义搜索时使用；其余场景优先 `worker` |
   | Run commands | `shell`+fast | composer-2 | Command execution only |

   Selection guidelines:
   - **See `.cursor/rules/subagent-routing.mdc` for the full routing decision guide.**
   - Two tiers only: **fast** (worker) for tasks completable by pattern matching, **strong** (coder/investigator with inherit) for tasks requiring reasoning.
   - Use `readonly: true` for investigator and explore. Worker and coder can write files.
   - Prefer `worker` over `explore` when possible — `explore` 固定使用 composer-1.5（不可覆盖），`worker` 使用 composer-2-fast 更快更强。
6. Launch subagents in parallel.
   - Use one message with multiple Task tool calls to get true parallelism.
   - Give each subagent only the context it needs: relevant file paths, constraints, and expected output format. Do not dump the full repo map.
   - Tell each subagent exactly what to return.
7. Validate subagent results.
   - Check that each subagent actually inspected its assigned scope (files listed, behaviors checked).
   - For code changes: run `cargo check` or the equivalent to verify compilation. Run tests if feasible.
   - For review/audit: spot-check high-severity findings against the actual source.
   - Flag suspiciously thin reports ("no findings" with few files inspected) for parent re-inspection.
8. Aggregate and synthesize.
   - Merge duplicate findings.
   - Resolve cross-chunk conflicts.
   - Produce one concise final summary with overall status, per-chunk outcomes, and next actions.
   - If the first 1–2 waves reveal heavy conflicts or most chunks needed the same shared context, fall back to single-agent execution for the remaining work.

## Domain Partitions

This project has 6 pre-defined domain partitions (in `.cursor/rules/domain-*.mdc`). Use these as the default split boundaries for any multi-domain task.

| Domain ID | Scope |
|-----------|-------|
| `script-lang` | `vn-runtime/src/script/**`, `command/**`, `diagnostic.rs` |
| `runtime-engine` | `vn-runtime/src/runtime/**`, `state.rs`, `input.rs`, `save.rs`, `history.rs` |
| `host-state` | `host-dioxus/src/state/**`, `command_executor.rs`, `render_state.rs` |
| `host-ui` | `host-dioxus/src/vn/**`, `screens/**`, `components/**`, `main.rs` |
| `resources` | `host-dioxus/src/resources.rs`, `manifest.rs`, `config.rs`, `save_manager.rs`, `init.rs` |
| `host-infra` | `host-dioxus/src/audio.rs`, `debug_server.rs`, `error.rs` |

When partitioning:

1. Map the task to affected domains.
2. One subagent per domain (unless a domain is trivially affected — merge small slices).
3. Each subagent prompt should include: `Read .cursor/rules/domain-{id}.mdc for domain invariants before starting.`
4. For cross-domain tasks, the parent agent owns the boundary contract (e.g., Command enum changes span `script-lang` + `host-app`).

## Partitioning Rules

Each chunk should have:

- A clear scope.
- Minimal file overlap with other chunks.
- A concrete expected output.
- A total source size the subagent can realistically process. As a rule of thumb, keep each chunk under ~5K LOC of relevant source. A 20K-LOC directory like `host/` must be split into subsystem-level chunks, not assigned to one subagent.

Prefer these split strategies:

- By top-level crate or module for full-repo reviews.
- By subsystem for test updates.
- By file family when boundaries are already clear.
- By investigation question when the questions are largely independent.

Avoid splits that:

- Require constant back-and-forth between subagents.
- Touch the same files heavily across multiple chunks.
- Depend on one unresolved global design decision.

## Subagent Prompt Template

Use this structure:

```markdown
Domain: [domain ID from partition table]
Task: [one chunk only]

Context:
- Read `.cursor/rules/domain-{id}.mdc` for domain invariants.
- Read the summary files listed in the domain rule for orientation.

Scope:
- [directory/module/file set — align with domain boundaries]

Goal:
- [expected outcome]

Constraints:
- [repo rules, test requirements, no unrelated refactors]

Return:
- Summary
- Findings or completed changes
- Files touched
- Tests run / not run
- Confidence (high / medium / low)
- Open risks or blockers
```

## Output Rules

For review or audit tasks, ask each subagent to return:

- Severity-ordered findings.
- Exact affected files or symbols.
- Missing tests or validation gaps.
- Explicit `no findings` when the chunk looks clean.
- **Confidence (high / medium / low):** self-assessed certainty that the chunk was thoroughly covered. A `no findings` + `low confidence` combination signals the parent to re-inspect.

For test update tasks, ask each subagent to return:

- Tests added or updated.
- Behavior covered.
- Commands executed.
- Failures or blockers.
- Whether more integration coverage is still needed.
- **Confidence (high / medium / low):** self-assessed certainty that the chunk was thoroughly covered.

## Parent Agent Responsibilities

- Own the global plan and final synthesis.
- Keep shared context minimal and only distribute chunk-local context.
- If a chunk reveals cross-cutting design impact, stop parallel edits for that area and pull the decision back to the parent agent.

## Why This Helps

A single agent processing N subtasks sequentially carries the full history of all prior subtasks in every API call. Total tokens grow roughly as O(N²). Splitting into independent subagents reduces this to O(N): each subagent runs with only its own chunk context, and the parent receives N compact summaries instead of N complete execution traces.

Concrete benefits:

- **Shorter parent context.** The parent never ingests raw subagent working logs, only structured summaries. This keeps the parent well within its context window even for large tasks.
- **Lower total token cost.** Low-information-density output (file reads, search results, intermediate reasoning) stays inside the subagent and is never forwarded. Subsequent parent turns do not pay for this bulk.
- **Better attention.** LLM attention quality degrades as context grows. Keeping each agent's context small and focused directly reduces the chance of forgetting earlier constraints or instructions.
- **Parallelism.** Independent chunks proceed concurrently, reducing wall-clock time.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| **Weak-model hallucination.** Subagents on a fast model may fabricate APIs, files, or test results. | Match model to cognitive complexity (see `.cursor/rules/subagent-routing.mdc`). For reasoning-heavy chunks, use `coder` or `investigator` (inherit). Parent should spot-check high-severity findings. |
| **Hidden cross-chunk coupling.** Partition looks independent but chunks share an invariant. | Parent reviews wave results for conflicts before launching the next wave. If coupling is discovered, merge the affected chunks into one subagent. |
| **Silent subagent failure.** A subagent reports "no findings" because it failed to understand the task, not because the chunk is clean. | Require every subagent to report the files it actually inspected and the behaviors it checked. A suspiciously thin report triggers parent re-inspection. |
| **Accumulated drift across waves.** Later waves operate on stale assumptions if earlier waves changed shared code. | Between waves, parent checks compilation and test status, then propagates new constraints into the next wave's prompts. |
| **Diminishing returns.** The task is not actually parallelizable and subagent overhead exceeds savings. | If the first wave produces heavy conflicts or most chunks need the same shared context, fall back to single-agent execution. Do not force more waves. |

## Additional Resources

- See [examples.md](examples.md) for concrete partitioning patterns.
- **Subagent routing guide:** `.cursor/rules/subagent-routing.mdc` — lookup table, `model:fast` criteria, anti-patterns.
- Pre-defined subagent files: `.cursor/agents/` (`worker`, `coder`, `investigator`).
- System agents (no config needed): `explore` (search/locate, 固定 composer-1.5 不可覆盖，优先用 `worker` 替代), `shell` (command execution, must pass `model: fast`), `browser-use` (browser testing).
