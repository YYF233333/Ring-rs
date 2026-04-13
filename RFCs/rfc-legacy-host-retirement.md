# RFC: Legacy Host (winit/wgpu/egui) Retirement

## Meta

- Number: RFC-034
- Status: Active
- Author: Yufeng Ying
- Date: 2026-04-13
- Scope: host/, workspace config, CI, docs
- Prerequisites: RFC-033 (Dioxus Host Migration) — Accepted

---

## Background

RFC-033 completed the Dioxus Desktop migration. `host-dioxus` is now the default runtime (`cargo run`), and `host/` is already commented out of workspace members. The old host crate (149 Rust files, winit/wgpu/egui stack) is dead code that:

1. Inflates repository size and cognitive overhead.
2. Creates confusion about which host is canonical.
3. Will accumulate `vn-runtime` API drift if left unfixed.

This RFC tracks the structured retirement of `host/` and the remaining bug-fix work needed to achieve full feature parity in `host-dioxus`.

---

## Goals & Non-Goals

### Goals

- **Parity bugs resolved**: Fix all known behavioral differences between `host-dioxus` and old host (tracked below).
- **Clean removal**: Delete `host/` crate and all references once parity is confirmed.
- **Documentation updated**: Navigation map, module summaries, and CLAUDE.md reflect single-host architecture.

### Non-Goals

- New features beyond what the old host supported — those belong in separate RFCs.
- Preserving old host code in a separate branch (git history is sufficient).

---

## Parity Bug Tracker

| # | Bug / Feature Gap | Severity | Status |
|---|-------------------|----------|--------|
| 1 | Quick menu positioned above dialogue box; should be inside textbox area at bottom | Visual | Fixed |
| 2 | NVL mode: ADV dialogue box and quick menu still render | Functional | Fixed |
| 3 | Dialogue/NVL text is mouse-selectable (blue highlight), breaks immersion | Visual | Fixed |
| 4 | showMap UI mode not implemented (map overlay missing) | Functional | Fixed |
| 5 | callGame UI mode not implemented (minigame iframe missing) | Functional | Fixed |
| 6 | Scene transitions (Dissolve/Fade) flash-cut instead of animating — CSS transition has no prior state on newly created elements | Visual | Fixed |
| 7 | Character name (speaker) white text invisible on light textbox — namebox.png is fully transparent, no contrast | Visual | Fixed |
| 8 | Narrator lines (`旁白："..."`) display "旁白" as speaker name — should be hidden | Functional | Fixed |
| 9 | Character sprite scaling incorrect — CSS `max-height: 100%` clamps image before `scale()` transform | Visual | Fixed |
| 10 | Skip mode shows only 1 character per line — `complete_typewriter` + `clear_click_wait` in same frame, need two-frame strategy | Functional | Fixed |
| 11 | Game container not centered — menu bar reduces viewport height, `transform-origin: top left` leaves black bar on right | Visual | Fixed |
| 12 | BGM/SFX volume nearly inaudible — JS `volume / 100` but Rust sends 0.0–1.0 range (0.8 → 0.008) | Functional | Fixed |

*New bugs discovered during testing should be appended here.*

---

## Design

### Phase 1: Parity Bug Fixes (Current)

Fix all items in the Parity Bug Tracker above. Each fix is a self-contained change to `host-dioxus/`.

### Phase 2: Removal

Once all bugs are resolved and manual play-through confirms parity:

1. Remove `host/` directory entirely.
2. Remove `host` from root `Cargo.toml` workspace members (already commented out).
3. Remove host-specific references in:
   - `CLAUDE.md` (cross-module sync table, workspace members list)
   - `ARCH.md`
   - `docs/engine/architecture/navigation-map.md`
   - `docs/engine/architecture/module-summaries/`
   - `.cursor/rules/domain-host-app.mdc`
4. Update `docs/workflows/cross-module-command-pipeline.md` to reflect single-host model.

### Phase 3: Cleanup

1. Remove any `host`-only compatibility shims in `vn-runtime` (if any exist).
2. Simplify any dual-host abstractions that are no longer needed.

---

## Impact

| Module | Change | Risk |
|--------|--------|------|
| `host/` | Deleted entirely | None (already inactive) |
| `host-dioxus/` | Bug fixes only | Low — behavioral corrections |
| `docs/` | Reference updates | Low — documentation only |
| `CLAUDE.md` | Remove old-host references | Low |

---

## Migration Plan

No migration needed — `host/` is already inactive. Users and CI already use `host-dioxus` exclusively.

---

## Acceptance Criteria

- [ ] All Parity Bug Tracker items resolved
- [ ] Manual play-through of reference project confirms equivalent experience
- [ ] `host/` directory removed
- [ ] All documentation references to old host updated or removed
- [ ] `cargo check-all` passes
- [ ] No remaining imports or references to `host` crate in workspace
