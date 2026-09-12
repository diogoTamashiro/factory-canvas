# Implementation Plan: Command-Based Undo/Redo

**Branch**: `005-command-undo-redo` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-command-undo-redo/spec.md`

## Summary

Deliver Phase 6 of the roadmap: let a player undo and redo the six
layout-mutating editor commands (place, remove, move, rotate, change base,
insert blueprint) across an unbounded, session-scoped history, with any
new command after an undo discarding the redo history. Implemented as a
new, UI-independent `EditHistory` type in `src/history.rs` — the exact
module `docs/architecture.md`'s "Current structure and incremental
target" tree already reserves for this — that stores whole-layout
snapshots around each successful command, following the same
clone-validate-commit discipline `FactoryLayout`'s own atomic batch
operations already use — not a per-command-type reversible-action model,
since a full snapshot is already cheap at this project's documented
scale (tens to low hundreds of entities) and requires no new per-command
undo/redo logic to get wrong.

## Technical Context

**Language/Version**: Rust, stable toolchain, edition 2021 (unchanged from
every prior phase).

**Primary Dependencies**: None new. No new crate is added; this feature
is pure in-memory editor state built entirely from types `FactoryLayout`
and `egui_app.rs` already expose.

**Storage**: N/A — undo/redo history is in-memory editor-session state
only (spec.md Assumptions), never persisted to `FactoryDocument`,
`BlueprintDocument`, or any file. Reopening a saved factory always starts
with empty history (FR-010).

**Testing**: `cargo test`, scoped to the files this feature actually
touches per `docs/engineering-standards.md` §Testing scope (not a
blanket full-suite run) — new `src/history.rs` gets its own
`#[cfg(test)] mod tests` (pure logic, no UI), and `src/egui_app_tests.rs`
gets new editor-level integration tests for each of the six commands'
undo/redo behavior, following this project's existing "logical,
deterministic tests over UI automation" discipline.

**Target Platform**: Windows desktop (unchanged); no new platform
surface.

**Project Type**: Desktop application (single Rust crate + two
binaries), unchanged.

**Performance Goals**: N/A beyond existing invariants — undo/redo is a
one-shot, user-triggered operation at the same documented scale (tens to
low hundreds of entities) every other atomic layout operation already
targets. Cloning a whole `FactoryLayout` per history entry is the same
clone this project's `replace_instances_atomically` already performs on
every group move/rotate today, just retained instead of discarded.

**Constraints**: Every one of the six commands MUST be recorded as
exactly one history entry regardless of how many entities it touches
(spec FR-006). Undo/redo MUST be blocked while any destructive
confirmation is open (FR-008). The identifier allocator (`next_entity_id`)
MUST only ever move forward — undoing a placement or insertion must not
roll it back, and undoing a removal must not consume a new identifier
for the restored entity (FR-009). No file under `catalog/`, `data/`,
`.hermes/`, or any historical `specs/*` directory may be touched.

**Scale/Scope**: One new module (`src/history.rs`, `pub(crate)`,
no `domain/` placement since this is editor-session state, not layout
domain logic — mirrors `src/selected_set.rs`'s existing placement
one level up from `domain/`), plus instrumentation of the six existing
mutation call sites in `src/egui_app.rs`, two new header actions
(Undo/Redo) with matching keyboard shortcuts, and their respective test
files. Comparable in size to Commit 9 (multi-selection group edits).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No new dependency,
  process, or abstraction beyond what `docs/engineering-standards.md`
  already sanctions; the snapshot-based history follows the same
  clone-then-commit pattern `FactoryLayout::replace_instances_atomically`
  already uses, just retained across calls instead of discarded after
  one.
- **II. Architectural Decisions Live in ADRs**: This is the first
  substantive architecture decision this project makes about undo/redo's
  actual mechanism (whole-layout snapshot vs. a per-command-type
  reversible-action model) — research.md Decision 1 records the choice
  and its rationale. It does not contradict any Accepted ADR (0001-0003
  cover UI stack, product naming, and document/blueprint schema — none
  address editing history). Recording a new ADR is recommended but not
  strictly required by Principle II's own trigger condition
  ("MUST NOT contradict an Accepted ADR without... a new ADR"); flagged
  in this plan's Completion Report for the user's explicit decision
  rather than created unprompted.
- **III. Spec-Driven From Commit 7 Onward**: PASS. `spec.md` is written,
  self-validated against the 16-item quality checklist (16/16 pass), and
  approved before this plan.
- **IV. Gates Are Still Mandatory**: PASS. All six gates (`cargo fmt
  --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  the tests covering this feature's own changed files per
  `docs/engineering-standards.md` §Testing scope, `cargo build --release
  --bins`, `git diff --check`, `hermes verify --skip-start --json
  --timeout 300`) apply to every commit on this branch, same as every
  prior phase.
- **V. Privacy and Catalog Boundaries**: PASS. No `data/**`,
  `reference/**`, or `.hermes/**` content is read, touched, or
  introduced; no game entity, base, product, region, dimension,
  capacity, or port is invented — undo/redo only ever replays already-
  validated layout states that already existed in this session.

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — Phase 1 introduces exactly one new type
(`EditHistory` in `src/history.rs`, holding `Vec<EditorSnapshot>`
undo/redo stacks), directly required by spec FR-001–FR-010, no
speculative field or trait beyond what those FRs need. No dependency
change. No decision contradicts ADR 0001, ADR 0002, or ADR 0003. The
Constitution Check above still holds unchanged post-design.

## Project Structure

### Documentation (this feature)

```text
specs/005-command-undo-redo/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no network API, CLI, or
cross-process service boundary — the same no-`contracts/` justification
`specs/001-blueprint-library/plan.md` through
`specs/004-blueprint-insertion-interfaces/plan.md` already used.

### Source Code (repository root)

```text
src/
├── history.rs                  # NEW — EditorSnapshot, EditHistory
│                               # (undo/redo stacks over whole-layout
│                               # snapshots), pub(crate), no domain/ or
│                               # egui dependency, own #[cfg(test)]
│                               # mod tests — mirrors selected_set.rs's
│                               # existing placement one level above
│                               # domain/; this is the exact module
│                               # docs/architecture.md already reserves
│                               # under "history.rs # future undo/redo
│                               # command"
├── domain/
│   └── layout.rs               # UNCHANGED — undo/redo only clones and
│                                # restores whole FactoryLayout values
│                                # through its already-public Clone impl;
│                                # no new method needed
├── egui_app.rs                 # MODIFY — new EditHistory field on
│                                # FactoryCanvasApp, wraps the six
│                                # existing mutation call sites
│                                # (place_selected_at,
│                                # confirm_instance_removal,
│                                # move_selected_by,
│                                # rotate_selected_clockwise,
│                                # replace_base,
│                                # insert_armed_blueprint_at) to record a
│                                # snapshot before each successful
│                                # mutation, new Undo/Redo header actions
│                                # and Ctrl+Z/Ctrl+Y shortcuts, history
│                                # cleared on New/Open
├── egui_app_tests.rs            # MODIFY — new editor-level undo/redo
│                                # integration tests, one per command
│                                # plus the cross-cutting FR-004/FR-007/
│                                # FR-008 behaviors
└── egui_canvas.rs               # UNCHANGED — no new canvas interaction;
                                  # Undo/Redo are header/keyboard actions
                                  # only, like Save/Open

catalog/                        # UNCHANGED — no catalog schema change
data/                            # UNCHANGED (ignored, private)
.hermes/                         # UNCHANGED (ignored, private)
specs/001-blueprint-library/     # UNCHANGED — frozen historical SDD trail
specs/002-blueprint-library-ui/  # UNCHANGED — frozen historical SDD trail
specs/003-phase-4-closure/       # UNCHANGED — frozen historical SDD trail
specs/004-blueprint-insertion-interfaces/  # UNCHANGED — frozen historical SDD trail
```

**Structure Decision**: Single-crate desktop application, unchanged from
every prior phase. `EditHistory` is placed one level above `domain/`,
alongside `selected_set.rs`, because it is editor-session state scoped
to one open factory (spec FR-010) rather than a layout-domain concept —
the domain's own `FactoryLayout` gains no new method and stays exactly
as unaware of history as it is today. `egui_app.rs` remains the single
place that decides *when* a command is significant enough to record,
matching its existing role as the sole owner of state transitions
(`docs/architecture.md` §"Current structure and incremental target").

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*

## Implementation Deviations

*(Filled in after `/speckit-implement` — every material difference between
this plan and what was actually built, per this project's standing rule
that any plan deviation discovered during implementation is narrated
explicitly.)*

1. **`EditorSnapshot` does not carry `next_entity_id` — research.md
   Decision 6 was wrong.** The original design (this plan/research.md/
   data-model.md, before implementation) called for storing and
   restoring `next_entity_id` verbatim per snapshot. Writing T014's test
   surfaced a direct contradiction with spec FR-009 ("the identifier
   allocator MUST continue to move forward only... regardless of any
   undo or redo"): restoring an older, smaller `next_entity_id` after
   undoing a placement would let a later, unrelated placement reuse the
   identifier the undone command had already consumed. Corrected by
   removing `next_entity_id` from `EditorSnapshot` entirely —
   `FactoryCanvasApp::next_entity_id` is simply never touched by
   `undo`/`redo` at all, since the allocator is already monotonic by
   construction (only `place`/`insert_into` ever advance it; nothing
   anywhere ever decreases it) and every entity's own identifier is
   already self-contained inside the restored `FactoryLayout`.

2. **`header_ui`'s return type changed from `Option<DocumentCommand>` to
   `(Option<DocumentCommand>, Option<HistoryCommand>)`.** Not anticipated
   by data-model.md, which only described a new sibling
   `history_shortcut_for_frame` function and new header buttons without
   specifying how the header's existing single-command return value
   would carry a second, independent command. Since Undo/Redo buttons
   live in the same `header_ui` closure as the document-command buttons,
   the simplest correct change was returning both command options from
   the one function call already made once per frame, rather than
   introducing a second UI pass. Three pre-existing tests that called
   `header_ui`/its `header_frame` test helper directly needed updating to
   destructure the new tuple shape; no existing document-command behavior
   changed.

3. **T034's test does not introduce new
   `history_shortcut_frame`/`dispatch_history_shortcut_frame` free
   functions**, unlike the task's own suggested template
   (`document_shortcut_frame`/`dispatch_document_shortcut_frame`).
   `history_shortcut_for_frame` and `dispatch_history_command_for_frame`
   (production functions, T031/T032) already have signatures simple
   enough that an inline `egui::Context::run_ui` call in the test body was
   sufficient, avoiding two more test-only wrapper functions each used at
   a single call site.

4. **T037 found and fixed a real coverage gap, not just confirmed
   coverage**: the "History is cleared on New/Open" quickstart.md
   scenario had no dedicated test. `self.history.clear()` (T017) was only
   ever exercised incidentally by other tests that happened to call
   `new_document_at`/`open_document_from` for unrelated reasons, never
   asserting the history-clearing behavior itself. Added
   `history_is_cleared_on_new_document` and
   `history_is_cleared_on_open_document` in `src/egui_app_tests.rs`
   during Polish, both passing on first run against the already-complete
   T017 implementation — this was a missing-test gap, not a missing-code
   gap.

No deviation altered any FR, AC, or the three explicit exclusions
(FR-011 removal confirmation, FR-012 document-lifecycle/blueprint-save
actions, FR-013 production-target configuration); all are
implementation-detail corrections discovered while building against
real code and real test coverage, consistent with this project's
established pattern from every prior phase.
