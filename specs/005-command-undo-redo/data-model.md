# Phase 1 Data Model: Command-Based Undo/Redo

This feature adds two new editor-session types (`EditorSnapshot`,
`EditHistory`) in a new `src/history.rs` module. It changes no existing
domain, catalog, factory-document, or blueprint-document contract — see
research.md Decision 2 for why history is deliberately kept out of
`src/domain/`.

## `EditorSnapshot`

A point-in-time copy of everything undo/redo needs to restore exactly —
see research.md Decision 1 for why a whole-layout snapshot was chosen
over a per-command reversible-action model.

```text
EditorSnapshot
  layout: FactoryLayout        // full clone — entities, base, catalog reference
```

- Derives `Clone`, `PartialEq`, `Eq` — `FactoryLayout` already does, so
  no new trait implementation is needed on any existing type.
- Carries no `next_entity_id` — corrected during implementation (plan.md
  Implementation Deviations #1, research.md Decision 6): the allocator is
  already monotonic by construction and is simply never touched by
  undo/redo at all, rather than being captured and restored per
  snapshot.
- Carries no selection state (research.md Decision 5) and no viewport/
  camera state, consistent with `docs/data-model.md`'s existing "No
  viewport, camera, or other editor-only metadata is persisted" rule —
  extended here to session history, not just saved documents.

## `EditHistory`

The current factory session's undo/redo stacks.

```text
EditHistory
  undo_stack: Vec<EditorSnapshot>
  redo_stack: Vec<EditorSnapshot>
```

```text
EditHistory::new() -> EditHistory
  // Empty history — both stacks empty. Used whenever a session starts:
  // FactoryCanvasApp::default(), new_document_at, open_document_from
  // (FR-010).

EditHistory::record(&mut self, snapshot: EditorSnapshot)
  // Pushes `snapshot` (the state immediately BEFORE the command that is
  // about to execute) onto undo_stack, and clears redo_stack entirely
  // (FR-004). Called by egui_app.rs only on the success branch of one of
  // the six commands, never on a rejected attempt (spec Assumptions).

EditHistory::undo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot>
  // Pops the top of undo_stack. If empty, returns None and changes
  // nothing (FR-007). Otherwise pushes `current` (the state right before
  // this undo) onto redo_stack and returns the popped snapshot for the
  // caller to restore.

EditHistory::redo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot>
  // Symmetric to undo(): pops the top of redo_stack, pushes `current`
  // onto undo_stack, returns the popped snapshot. Returns None and
  // changes nothing if redo_stack is empty (FR-007).

EditHistory::clear(&mut self)
  // Empties both stacks (FR-010). Called by new_document_at and
  // open_document_from.

EditHistory::can_undo(&self) -> bool
EditHistory::can_redo(&self) -> bool
  // Used to enable/disable the Undo/Redo header buttons; read-only,
  // never mutate either stack.
```

- `pub(crate)`, defined in `src/history.rs`, imported by
  `src/egui_app.rs` exactly as `SelectedSet` already is
  (research.md Decision 2).
- No maximum depth: `docs/roadmap.md`/every prior phase's plan.md already
  documents this project's scale as "tens to low hundreds of entities"
  per factory and a modest number of discrete edits per working session
  — unbounded `Vec` growth within one session is not a real concern at
  that scale (spec.md Assumptions: "History depth is unbounded within a
  session").

## `FactoryCanvasApp` — extended contract

`FactoryCanvasApp` gains one new field:

```text
FactoryCanvasApp
  ...                    // unchanged existing fields
  history: EditHistory   // NEW
```

Six existing methods each gain one `self.history.record(...)` call,
placed immediately before the mutation on the branch that is about to
succeed (never on a rejection branch):

```text
place_selected_at              // before self.layout.place(...)
confirm_instance_removal       // before removing any instance
move_selected_by               // before self.layout.move_instances_by(...)
rotate_selected_clockwise      // before rotating (single or group)
replace_base                   // before constructing the new FactoryLayout
insert_armed_blueprint_at      // before blueprint.insert_into(...)
```

Two existing methods each gain one `self.history.clear()` call:

```text
new_document_at        // starting a new factory clears history (FR-010)
open_document_from     // opening a different factory clears history (FR-010)
```

Two new methods:

```text
FactoryCanvasApp::undo(&mut self)
  // No-op if destructive_modal_open() (FR-008). Builds an EditorSnapshot
  // from self.layout, calls self.history.undo(current_snapshot); on
  // Some(restored), assigns self.layout from `restored` (next_entity_id
  // is never touched — research.md Decision 6), reconciles self.selected
  // via the existing refresh_selection_notice() pruning step (research.md
  // Decision 5), marks the session dirty, and sets a new
  // EditorNotice::Undone notice. On None, does nothing (FR-007).

FactoryCanvasApp::redo(&mut self)
  // Symmetric to undo(): no-op if destructive_modal_open(); on
  // Some(restored), applies it the same way and sets EditorNotice::Redone.
```

- No `EditorNotice` variant leaks any snapshot content (entity count,
  identifiers, or internal detail) beyond what a normal command's own
  existing notices already show — `Undone`/`Redone` are plain,
  content-free notices, matching this project's existing safe-notice
  discipline (`safe_blueprint_save_error_detail` and similar).

## Two new header actions and shortcuts

```text
DocumentCommand           // UNCHANGED enum — Undo/Redo are NOT added here,
                           // since they are not document-lifecycle actions
                           // (FR-012) and must remain independently gated
                           // by destructive_modal_open() the same way
                           // every other non-document editing action is,
                           // not folded into execute_document_command's
                           // dialog-driven New/Open/Save/SaveAs flow

HistoryCommand             // NEW, mirrors DocumentCommand's shape
  Undo
  Redo
```

```text
history_shortcut_for_frame(context: &egui::Context, blocked: bool) -> Option<HistoryCommand>
  // Mirrors document_shortcut_for_frame: Ctrl+Z -> Undo, Ctrl+Y -> Redo,
  // returns None if `blocked` (destructive_modal_open()) or the context
  // has text-edit focus, same guard document_shortcut_for_frame already
  // applies (research.md Decision 4).
```

## What this feature does NOT change

- `src/domain/layout.rs` — `FactoryLayout` gains no new method, field, or
  derive. Undo/redo only ever calls its already-`pub` `Clone`.
- `BlueprintDocument`/`FactoryDocument` schemas — history is never
  persisted (FR-010, spec.md Assumptions).
- `SelectedSet` — no new method; its existing `retain` already prunes
  invalid IDs after a restore, and its existing rotation-pivot
  invalidation already runs on the next selection-membership change.
- Production-target configuration (`set_production_target`) — explicitly
  excluded from the undoable command set (FR-013).
