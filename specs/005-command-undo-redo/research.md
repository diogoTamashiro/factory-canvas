# Phase 0 Research: Command-Based Undo/Redo

No item in Technical Context was marked `NEEDS CLARIFICATION`; every
technology and process choice already follows this project's established
Rust/eframe/egui toolchain and SDD workflow unchanged. This document
records the concrete design decisions this plan makes to turn spec.md's
functional requirements into an implementable contract.

## Decision 1: Whole-layout snapshot history, not a per-command-type reversible-action model

**Decision**: `EditHistory` stores a `Vec<EditorSnapshot>` for undo and a
second `Vec<EditorSnapshot>` for redo, where an `EditorSnapshot` is a
plain value holding a cloned `FactoryLayout` plus the `next_entity_id`
allocator value at that point in time. Before any of the six commands
(place, remove, move, rotate, base change, insert blueprint) actually
mutates `self.layout`, `egui_app.rs` pushes the *current* `EditorSnapshot`
(the state right before the mutation) onto the undo stack and clears the
redo stack. Undo pops the top undo entry, pushes the layout's state
*right before restoring it* onto the redo stack, then restores the popped
snapshot. Redo is the mirror operation.

**Rationale**: `FactoryLayout` and its `BlockInstance`s are already
`#[derive(Clone, PartialEq, Eq)]` — nothing here requires adding a new
derive or trait to the domain. `FactoryLayout::replace_instances_atomically`
already clones the whole layout, mutates the clone, validates it, and
only then commits by replacing `self` — this project has already decided
a whole-layout clone-and-replace is an acceptable cost for "editing a
group of instances," which is the same order of cost as "cloning a
snapshot for history." At this project's documented scale (tens to low
hundreds of entities — `docs/roadmap.md`, `specs/*/plan.md`'s recurring
"Performance Goals" language), a `BTreeMap<EntityId, BlockInstance>` clone
is not a real cost concern, and a snapshot model needs no per-command
"how do I reverse a rotation/how do I reverse a multi-node insertion"
logic to design, test, or get wrong — undo of *any* command is exactly
"restore the previous snapshot," and redo of *any* command is exactly
"restore the next snapshot," uniformly.

**Alternatives considered**:
- *Per-command-type reversible actions* (an enum with one variant per
  command, each carrying enough data to compute its own inverse — e.g. a
  `Move` variant storing the delta to reverse, a `Remove` variant storing
  the removed instances to reinsert): rejected. This is the traditional
  "command pattern" approach and would use less memory per entry, but it
  requires writing and testing six separate inverse operations (one per
  command in spec FR-001), each a fresh place to introduce a subtle bug
  (e.g. reversing a multi-instance rotation about a pivot correctly, or
  reconstructing a blueprint insertion's exact set of newly-created
  entities to remove on undo). KISS favors the simpler, uniform
  mechanism when nothing in this project's actual scale needs the memory
  savings.
- *Storing only a diff between states* (e.g. which entities were added/
  removed/changed) rather than a full snapshot: rejected for the same
  reason — more moving parts to compute and verify correctly, for a
  saving this project's scale does not need. A whole-`FactoryLayout`
  clone is already this project's accepted idiom for "safely mutate,
  discard on failure."

**Consequence**: `EditorSnapshot`'s `PartialEq`-based tests can assert
byte-for-byte layout equality before/after an undo/redo round trip using
the exact same `assert_eq!(destination, before)` idiom
`tests/domain_blueprint.rs` already uses for `insert_into`'s atomicity
tests (Phase 5) — no new assertion helper is needed.

## Decision 2: History lives in `src/history.rs`, one level above `domain/`, not inside `FactoryLayout`

**Decision**: `EditHistory` and `EditorSnapshot` are defined in a new
`src/history.rs` module, `pub(crate)`, imported and owned by
`FactoryCanvasApp` in `src/egui_app.rs` exactly the way `SelectedSet`
(`src/selected_set.rs`) already is. `src/domain/layout.rs` gains no new
method, field, or trait.

**Rationale**: `docs/architecture.md`'s own "Current structure and
incremental target" source tree already names and reserves this exact
module — `history.rs # future undo/redo command` — one level above
`domain/`, alongside `selected_set.rs`. That existing documentation
already answers *where* this code goes; this decision simply confirms it
against the actual requirements instead of introducing a competing
location. Placing history above `domain/` (rather than as a
`FactoryLayout` method) keeps `docs/roadmap.md`'s "Non-negotiable
principles" invariant intact — "the domain in `src/domain/` does not
depend on egui, the filesystem, SQLite, the network, or Python" — since
history is inherently an *editor-session* concept (spec FR-010: cleared
on New/Open, never persisted), not a property of a factory layout's own
spatial validity. `FactoryLayout` itself has no notion of "this is a
step in some session's history" and should not gain one.

**Alternatives considered**:
- *A `FactoryLayout::with_history()` wrapper or a history-aware layout
  type*: rejected — would make the domain aware of editor-session
  concerns it has no reason to know about, contradicting the
  domain-independence invariant above and this project's pragmatic-SOLID
  rule ("the domain does not depend on UI or I/O" — session history is
  UI/editor state, not domain state).
- *History embedded directly as fields on `FactoryCanvasApp` without a
  separate type*: rejected — `SelectedSet` already established the
  precedent of extracting session-scoped, UI-adjacent-but-UI-independent
  state into its own small, independently testable type one level above
  `domain/`, with its own `#[cfg(test)] mod tests` requiring no egui
  context. `EditHistory` follows the same shape for the same reason.

## Decision 3: `egui_app.rs` decides *when* a command is history-worthy; `EditHistory` only stores and replays

**Decision**: `EditHistory` exposes a minimal interface — `record(snapshot)`,
`undo(current) -> Option<EditorSnapshot>`, `redo(current) -> Option<EditorSnapshot>`,
`clear()` — with no knowledge of *which* of the six commands triggered a
`record` call. Each of the six existing mutation call sites in
`egui_app.rs` (`place_selected_at`, `confirm_instance_removal`,
`move_selected_by`, `rotate_selected_clockwise`, `replace_base`,
`insert_armed_blueprint_at`) calls `self.history.record(...)` with a
snapshot of `self.layout`/`self.next_entity_id` taken immediately before
mutating, but only on the branch that actually proceeds to mutate — never
on a branch that returns early due to a rejection (e.g. `PlacementError`,
`InstanceEditError`, `BlueprintInsertionError`), consistent with spec's
Assumptions ("A command attempt that is rejected... is never recorded in
the undo history").

**Rationale**: This mirrors the existing separation between `FactoryLayout`
(validates and mutates) and `egui_app.rs` (decides when a validated
mutation happened and what notice to show) that every one of the six call
sites already implements today — each already has an `Ok(...) => { ...
} Err(error) => { self.notice = ... }` shape, so adding "and also record a
snapshot" to the `Ok` arm is a minimal, uniform change to an already-
established pattern rather than a new control-flow shape.

**Alternatives considered**:
- *`EditHistory` itself intercepts and re-validates each command*:
  rejected — would duplicate validation `FactoryLayout` already owns
  (the same DRY-in-moderation argument Phase 5's research.md Decision 2
  already made for insertion), and would need to know about six
  different command shapes instead of one uniform snapshot shape.

## Decision 4: Undo/redo are header actions plus `Ctrl+Z`/`Ctrl+Y`, gated by the same `destructive_modal_open` check every other shortcut already uses

**Decision**: Two new header buttons ("Undo", "Redo") next to the
existing New/Open/Save/Save As buttons, plus two new keyboard shortcuts
(`Ctrl+Z` for Undo, `Ctrl+Y` for Redo) recognized by a new
`history_shortcut_for_frame` function mirroring the existing
`document_shortcut_for_frame`. Both the buttons and the shortcuts are
disabled/ignored whenever `self.destructive_modal_open()` is true (spec
FR-008), exactly like every existing editing action already is.

**Rationale**: `Ctrl+Z`/`Ctrl+Y` are the platform-conventional Windows
shortcuts for undo/redo (this project targets Windows desktop only per
`docs/roadmap.md`), and neither combination is bound to anything today
(confirmed by inspecting every `egui::Key`/`egui::KeyboardShortcut` use
in `src/egui_app.rs` — `Ctrl+O`/`Ctrl+S`/`Ctrl+Shift+S` for documents,
plain `Delete`/`Backspace`/`F`/`R`/arrow keys for selected-instance
actions, `Home` for framing — no collision). `destructive_modal_open()`
already exists as the single source of truth for "is a pending
confirmation blocking other editing actions right now," reused as-is
rather than duplicated.

**Alternatives considered**:
- *`Ctrl+Shift+Z` for redo* (the macOS/some-editors convention): rejected
  — `Ctrl+Y` is the long-standing Windows convention (Windows Notepad,
  Office, most Windows-native editors) and this project is Windows-only;
  no reason to deviate from the platform users actually expect.

## Decision 5: Selection is not restored by undo/redo

**Decision**: Undoing or redoing a command restores `self.layout` and
`self.next_entity_id` only. `self.selected` (the `SelectedSet`) is left
as-is, then reconciled the same way every other layout-changing action
already reconciles it — `refresh_selection_notice()`'s existing
`self.selected.retain(|id| layout.instance(id).is_some())` step, called
after the restore, already drops any now-invalid ID from a restored
layout without any new code.

**Rationale**: Selection is explicitly documented as ephemeral editor
state distinct from the persisted/historical layout — `docs/data-model.md`
already states "No viewport, camera, or other editor-only metadata is
persisted," and every existing atomic group edit (move, rotate) already
treats the *rotation pivot* memory as separate, invalidatable state tied
to selection membership rather than to the layout itself
(`SelectedSet::retain`, `remember_rotation_pivot`/`translate_rotation_pivot`
in `src/selected_set.rs`). Restoring selection would require storing a
seventh piece of history-adjacent state per snapshot and defining what
"restore a selection across an undone base change" even means when the
selected entities may not exist in the restored layout — complexity spec.md
does not ask for and this decision avoids.

**Alternatives considered**:
- *Store and restore selection alongside each snapshot*: rejected —
  spec.md's Assumptions already settle this ("Selection is not restored
  by undo or redo... consistent with how existing atomic group edits
  already treat it separately"); revisiting it here would only
  contradict the approved spec.

## Decision 6: `next_entity_id` is never touched by undo/redo at all

**Original decision (WRONG — corrected during implementation, see
plan.md's Implementation Deviations #1 for the full account)**:
`EditorSnapshot` was originally designed to store `next_entity_id:
Option<u64>` as a plain field captured at snapshot time, restoring it
verbatim alongside the layout.

**Why the original decision was wrong**: Spec FR-009 requires the
allocator to "only ever move forward" even across undo/redo. Restoring
an older, smaller `next_entity_id` value after undoing a placement would
let a *later*, unrelated placement reuse the identifier the undone
command had already consumed once redo-then-diverge happened — exactly
the case FR-009 forbids. This was caught while writing T014's test
(`undo_never_lets_the_entity_id_allocator_move_backward`), not during
design review.

**Corrected decision**: `EditorSnapshot` carries only the `FactoryLayout`
— no allocator field at all. `undo`/`redo` restore `self.layout` and
leave `self.next_entity_id` completely untouched. This works because the
allocator is already monotonic by construction: `place`/`insert_into`
are the only two call sites that ever advance it, and nothing anywhere
in the codebase ever decreases it. Every entity's own identifier is
already self-contained inside the layout being restored, so no separate
allocator bookkeeping is needed for undo/redo to be correct.

**Consequence**: `EditorSnapshot::new` takes one argument (`layout:
FactoryLayout`) instead of two, and exposes `into_layout(self) ->
FactoryLayout` rather than a pair of accessors.
