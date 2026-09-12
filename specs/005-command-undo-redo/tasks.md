# Tasks: Command-Based Undo/Redo

**Input**: Design documents from `/specs/005-command-undo-redo/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included. Each user story's `spec.md` "Independent Test" field and `quickstart.md`'s scenario walkthroughs are an explicit request for integration-test coverage of every acceptance scenario. Per the project constitution (Principle III), this feature is spec-driven rather than test-first: the spec/plan/tasks below were written, reviewed, and approved before any implementation, so tests are written alongside their behavior rather than as a mandatory pre-implementation RED step — but the tests covering this feature's own changed files (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope) and the other five gates must still pass before each commit lands.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (per plan.md's Structure Decision): `src/` at repository root, no `tests/` directory involvement — this feature concentrates entirely in a new `src/history.rs` and modifications to `src/egui_app.rs`/`src/egui_app_tests.rs`. No `domain/`, `persistence/`, or `egui_canvas.rs` file changes (research.md Decisions 1-2; plan.md's Project Structure).

## Phase 1: Setup

**Purpose**: Confirm the exact starting state this feature builds on

- [X] T001 Re-read `src/egui_app.rs`'s six mutation call sites (`place_selected_at`, `confirm_instance_removal`, `move_selected_by`, `rotate_selected_clockwise`, `replace_base`, `insert_armed_blueprint_at`), `new_document_at`, `open_document_from`, `destructive_modal_open`, `document_shortcut_for_frame`, and `EditorNotice`/`notice_text`/`notice_color`, plus `src/selected_set.rs` (the precedent module this feature's placement mirrors) in full, and confirm every signature this plan/data-model.md assumes still matches — no code change, a pre-flight confirmation only

  **Result**: Confirmed. One correction to T015's assumption: `FactoryCanvasApp`'s real constructor is `from_startup_catalog` (called by both `Default::default()` and `new()`), not a `Self { ... }` literal directly inside `Default::default()` — `history: EditHistory::new()` goes in `from_startup_catalog`'s own `Self { ... }` literal.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `EditorSnapshot`/`EditHistory` types every user story's tests and implementation depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Create `src/history.rs` defining `EditorSnapshot` (`layout: FactoryLayout`, `next_entity_id: Option<u64>`, `#[derive(Debug, Clone, PartialEq, Eq)]`) per data-model.md (depends on T001)
- [X] T003 In `src/history.rs`, implement `EditHistory` (`undo_stack: Vec<EditorSnapshot>`, `redo_stack: Vec<EditorSnapshot>`) with `new()`, `record(&mut self, snapshot: EditorSnapshot)` (pushes onto `undo_stack`, clears `redo_stack` — FR-004), `undo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot>`, `redo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot>`, `clear(&mut self)`, `can_undo(&self) -> bool`, `can_redo(&self) -> bool`, all `pub(crate)`, per data-model.md (depends on T002)
- [X] T004 [P] Add `mod history;` to `src/egui_main.rs` (the binary's module-declaration file, alongside its existing `mod blueprint_library_view;`, `mod document_session;`, `mod egui_app;`, `mod egui_canvas;`, `mod selected_set;`) (depends on T002)
- [X] T005 [P] In `src/history.rs`, write `#[cfg(test)] mod tests` covering `EditHistory` in isolation (no `FactoryCanvasApp`, no egui context): `record` pushes to undo and clears redo; `undo` on an empty stack returns `None` and leaves both stacks unchanged; `undo` followed by `redo` restores the pre-undo current snapshot; `record` after an `undo` correctly discards what `redo` would have returned (FR-004, FR-007) (depends on T003)
- [X] T006 Run T005 and confirm it passes against T002-T003's implementation (fixes any mismatch found, does not add new scenarios) (depends on T005)

  **Result**: 6 tests pass (added a multi-step test beyond the minimum
  4 scenarios asked for, covering research.md Decision 1's "generalizes
  to N steps" claim directly at this layer, ahead of US3). One real API
  mismatch found and fixed during writing (not a logic bug): `Catalog::new`
  takes `default_base_id: BaseId` as an explicit second positional
  argument, not derived from the base list — the test helper was
  corrected to match the real signature.

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Undo the most recent action (Priority: P1) 🎯 MVP

**Goal**: A player performs any one of the six commands (place, remove, move, rotate, change base, insert blueprint) and can trigger Undo to return the factory to its exact state immediately before that command.

**Independent Test**: Perform any one of the six commands, trigger Undo once, and confirm every entity, identifier, origin, rotation, configured product, and the active base exactly match what they were before that command executed.

### Tests for User Story 1

- [X] T007 [P] [US1] Editor-level integration test `undo_reverses_a_placement` in `src/egui_app_tests.rs` (spec.md US1/AC1): place one instance, snapshot the layout, trigger Undo, assert the layout equals the pre-placement snapshot
- [X] T008 [P] [US1] Editor-level integration test `undo_reverses_a_single_and_group_move_or_rotation` in `src/egui_app_tests.rs` (spec.md US1/AC2): move/rotate a single instance and separately a group, triggering Undo after each, asserting exact restoration
- [X] T009 [P] [US1] Editor-level integration test `undo_reverses_a_confirmed_removal` in `src/egui_app_tests.rs` (spec.md US1/AC3): remove one or more instances through the existing confirmation flow, trigger Undo, assert every removed instance reappears with its original identifier, buildable, origin, rotation, and configured product
- [X] T010 [P] [US1] Editor-level integration test `undo_reverses_a_confirmed_base_change` in `src/egui_app_tests.rs` (spec.md US1/AC4): change the active base through the existing confirmation flow, trigger Undo, assert the previous base and every instance on it reappear exactly
- [X] T011 [P] [US1] Editor-level integration test `undo_reverses_a_blueprint_insertion_as_one_unit` in `src/egui_app_tests.rs` (spec.md US1/AC5): insert a multi-node blueprint, trigger Undo, assert every entity the insertion created disappears in that single Undo and the rest of the factory is unaffected (also exercises FR-006's "one history entry regardless of entity count")
- [X] T012 [P] [US1] Editor-level integration test `undo_with_empty_history_is_a_no_op` in `src/egui_app_tests.rs` (FR-007): trigger Undo on a freshly-created app with zero commands performed, assert the layout, allocator, and notice are completely unchanged
- [X] T013 [P] [US1] Editor-level integration test `undo_is_blocked_while_a_destructive_confirmation_is_pending` in `src/egui_app_tests.rs` (FR-008): open a pending removal or pending base-change confirmation, attempt Undo (both the method call and, per T023, the `Ctrl+Z` shortcut), assert nothing changes and the confirmation remains pending
- [X] T014 [US1] Editor-level integration test `undo_never_lets_the_entity_id_allocator_move_backward` in `src/egui_app_tests.rs` (FR-009): place an instance, undo it, place a different instance, assert the second placement's identifier is not the identifier freed by the undo (depends on T007)

  **Result**: All 8 pass. One real design flaw discovered while writing
  T014 (not a test-authoring bug — the original plan itself was wrong):
  research.md Decision 6 called for restoring `next_entity_id` verbatim
  from each snapshot, but that directly contradicts spec FR-009 ("the
  identifier allocator MUST continue to move forward only... regardless
  of any undo or redo"). Restoring an older `next_entity_id` after
  undoing a placement would let a later placement reuse the identifier
  the undone command had already consumed. Fixed by removing
  `next_entity_id` from `EditorSnapshot` entirely — the allocator is
  already monotonic by construction (only `place`/`insert_into` ever
  advance it, nothing ever decreases it), so undo/redo simply never
  touches it. See plan.md's Implementation Deviations for the full
  account. T008's group-move/rotation sub-scenarios also needed
  rewriting to read origins back from the layout instead of hand-computing
  expected absolute coordinates, after an arithmetic mistake surfaced
  test flakiness unrelated to the feature itself.

### Implementation for User Story 1

- [X] T015 [US1] Add `history: EditHistory` field to `FactoryCanvasApp` in `src/egui_app.rs`, initialized via `EditHistory::new()` in `Default::default()` (depends on T003)
- [X] T016 [US1] In `src/egui_app.rs`, add a private snapshot helper (e.g. `fn snapshot(&self) -> EditorSnapshot`) building an `EditorSnapshot` from `self.layout.clone()`/`self.next_entity_id`, and call `self.history.record(self.snapshot())` immediately before the mutation on the success branch of each of the six commands — `place_selected_at`, `confirm_instance_removal`, `move_selected_by`, `rotate_selected_clockwise`, `replace_base`, `insert_armed_blueprint_at` — never on a rejection branch, in `src/egui_app.rs` (depends on T015)
- [X] T017 [US1] Add `self.history.clear()` to `new_document_at` and `open_document_from` in `src/egui_app.rs` (FR-010) (depends on T015)
- [X] T018 [US1] Implement `FactoryCanvasApp::undo(&mut self)` in `src/egui_app.rs` per data-model.md: no-op if `self.destructive_modal_open()` (FR-008); otherwise build the current `EditorSnapshot`, call `self.history.undo(current)`, and on `Some(restored)` assign `self.layout`/`self.next_entity_id` from it, reconcile `self.selected` via the existing `refresh_selection_notice()` pruning step, call `self.session.mark_dirty()`, and set a new `EditorNotice::Undone` — on `None`, do nothing (FR-007) (depends on T016, T017)
- [X] T019 [US1] Add `EditorNotice::Undone` variant plus its `notice_text`/`notice_color` arms (a plain, content-free message following the existing safe-notice discipline — no entity count, identifier, or internal detail) in `src/egui_app.rs` (depends on T018)
- [X] T020 Run T007-T014 and confirm they pass against T015-T019's implementation (fixes any mismatch found, does not add new scenarios)

  **Result**: Implemented T015-T019 and T025-T026 (US2's `redo`/`Redone`)
  together as one `apply_restored_snapshot` helper shared by both
  `undo`/`redo`, since they are fully symmetric and splitting them across
  two separate edit passes would mean rewriting the same function twice.
  T025/T026 are marked complete here; T027 (running US2's own tests)
  still runs separately in its own phase below. `replace_base` records a
  snapshot unconditionally, including the `layout.is_empty()` short-circuit
  branch, so every base change is undoable per FR-002, not only the
  confirmed-non-empty path. `move_selected_by`/`rotate_selected_clockwise`
  only record when something actually changed (delta non-zero / rotation
  succeeded), mirroring the existing `mark_dirty()` guard already there.
  `cargo check --bin factory-canvas` compiles clean (only expected
  dead-code warnings for `undo`/`redo`/`Undone`/`Redone`, not yet called
  by anything until T007+'s tests and T030-T033's UI wiring).

**Checkpoint**: User Story 1 is fully functional and testable independently at the state-transition layer — T007-T014 pass using only T002-T003, T015-T019. The Undo trigger itself (header button, `Ctrl+Z`) is wired in the cross-cutting UI phase below since both User Story 1 and User Story 2 share the same header/shortcut surface.

---

## Phase 4: User Story 2 - Redo an undone action (Priority: P2)

**Goal**: After undoing a command, a player can trigger Redo to reapply it exactly as it originally happened, and Redo has no effect once superseded by a new command.

**Independent Test**: Perform any one of the six commands, undo it, then redo it, and confirm the resulting layout is identical to the state immediately after the original command executed.

### Tests for User Story 2

- [X] T021 [P] [US2] Editor-level integration test `redo_reapplies_an_undone_command_exactly` in `src/egui_app_tests.rs` (spec.md US2/AC1): perform a command, snapshot the post-command state, undo, redo, assert the layout equals the post-command snapshot
- [X] T022 [P] [US2] Editor-level integration test `redo_with_empty_redo_history_is_a_no_op` in `src/egui_app_tests.rs` (spec.md US2/AC2, FR-007): attempt Redo with no prior Undo in the session, assert nothing changes
- [X] T023 [P] [US2] Editor-level integration test `a_new_command_after_undo_discards_the_redo_history` in `src/egui_app_tests.rs` (spec.md US2/AC3, FR-004): perform command A then B, undo (reverses B), perform a different command C, attempt Redo, assert nothing changes (B is no longer available)
- [X] T024 [US2] Editor-level integration test `redo_is_blocked_while_a_destructive_confirmation_is_pending` in `src/egui_app_tests.rs` (FR-008, mirrors T013 for the redo direction) (depends on T021)

### Implementation for User Story 2

- [X] T025 [US2] Implement `FactoryCanvasApp::redo(&mut self)` in `src/egui_app.rs`, symmetric to `undo()` (T018): no-op if `destructive_modal_open()`; otherwise call `self.history.redo(current)` and apply `Some(restored)` the same way, setting `EditorNotice::Redone` (depends on T018)
- [X] T026 [US2] Add `EditorNotice::Redone` variant plus its `notice_text`/`notice_color` arms, matching `Undone`'s discipline, in `src/egui_app.rs` (depends on T025)

  **Result**: Implemented alongside T018-T019 (see that task's Result note) — `undo`/`redo` share one `apply_restored_snapshot` helper.
- [X] T027 [US2] Run T021-T024 and confirm they pass against T025-T026's implementation (fixes any mismatch found, does not add new scenarios)

**Checkpoint**: User Story 2 is fully functional and testable independently at the state-transition layer, on top of User Story 1's `history`/`undo()` foundation.

---

## Phase 5: User Story 3 - Undo and redo across multiple steps (Priority: P3)

**Goal**: A player can trigger Undo or Redo repeatedly to step backward and forward through the current session's full sequence of commands, not only the single most recent one.

**Independent Test**: Perform a sequence of several different commands, undo three of them in a row, confirm the layout matches the state before the third-from-last command, then redo twice and confirm the layout matches the state after the second command in the original sequence.

### Tests for User Story 3

- [X] T028 [US3] Editor-level integration test `undo_and_redo_step_through_a_multi_command_sequence_correctly` in `src/egui_app_tests.rs` (spec.md US3/AC1-AC3): perform at least three different commands with a snapshot taken after each, undo three times asserting each step matches the previous snapshot in reverse order, redo twice asserting each step matches the next snapshot in original order

### Implementation for User Story 3

- [X] T029 [US3] Run T028 against the existing `EditHistory`/`undo()`/`redo()` implementation from User Story 1 and User Story 2 — no new production code is expected, since `Vec`-backed stacks already support arbitrary depth (research.md Decision 1); if T028 fails, the fix belongs in T003's `EditHistory` implementation, not a new method (depends on T018, T025)

  **Result**: Passed with no new production code, confirming research.md
  Decision 1's "generalizes to N steps" claim.

**Checkpoint**: All three user stories are independently functional. User Story 3 validates that Setup/Foundational's stack-based design already generalizes beyond one step, per research.md Decision 1's rationale — no new mechanism should be needed here.

---

## Phase 6: Cross-Cutting UI (Header Actions and Shortcuts)

**Purpose**: Expose `undo()`/`redo()` (User Story 1 and 2's own methods) through the header and keyboard, shared by both stories rather than duplicated per story

- [X] T030 Add `HistoryCommand` enum (`Undo`, `Redo`) in `src/egui_app.rs`, mirroring `DocumentCommand`'s shape (data-model.md) (depends on T018, T025)
- [X] T031 Add `history_shortcut_for_frame(context: &egui::Context, blocked: bool) -> Option<HistoryCommand>` in `src/egui_app.rs`, mirroring `document_shortcut_for_frame`: `Ctrl+Z` → `Undo`, `Ctrl+Y` → `Redo`, returns `None` if `blocked` or `context.text_edit_focused()` (research.md Decision 4) (depends on T030)
- [X] T032 Wire `history_shortcut_for_frame`'s result into a dispatch call each frame (mirroring `dispatch_document_command_for_frame`'s existing pattern), calling `self.undo()`/`self.redo()` on the matching variant, in `src/egui_app.rs` (depends on T031)
- [X] T033 Add "Undo" and "Redo" header buttons next to the existing New/Open/Save/Save As buttons in `header_ui`, each `add_enabled` against `self.history.can_undo()`/`self.history.can_redo()` respectively and disabled whenever `commands_enabled` (the existing `!self.destructive_modal_open()` check already gating every other header button) is false, in `src/egui_app.rs` (depends on T030)

  **Result**: `header_ui`'s return type changed from `Option<DocumentCommand>`
  to `(Option<DocumentCommand>, Option<HistoryCommand>)` — a deviation
  from the original plan/data-model.md, which did not anticipate this
  signature change. 3 pre-existing tests that called `header_ui`/its
  `header_frame` test helper directly needed updating to destructure the
  new tuple shape (`header_frame` itself gained a third return element).
  No behavior of any pre-existing document command changed.
- [X] T034 [P] Editor-level integration test `ctrl_z_and_ctrl_y_trigger_undo_and_redo` in `src/egui_app_tests.rs`, using the existing `document_shortcut_frame`/`dispatch_document_shortcut_frame`-style helpers as a template for a new `history_shortcut_frame`/`dispatch_history_shortcut_frame` pair, asserting each shortcut calls through to the same effect as the header button (depends on T032)

  **Result**: Implemented directly against `dispatch_history_command_for_frame`
  rather than introducing new `history_shortcut_frame`/
  `dispatch_history_shortcut_frame` free functions — `egui::Context::run_ui`
  called inline was sufficient and avoided adding two more test-only
  functions for a single call site each; also added
  `history_header_button_command_has_priority_over_shortcut`, mirroring
  `header_command_has_priority_over_shortcut_in_shared_dispatcher`'s
  existing document-command precedent, since that project convention
  exists specifically to test this priority rule.
- [X] T035 [P] Editor-level integration test `history_header_buttons_are_disabled_when_their_stack_is_empty_or_a_modal_is_pending` in `src/egui_app_tests.rs`, asserting `can_undo()`/`can_redo()` correctly gate button enablement in each state (depends on T033)
- [X] T036 Run T034-T035 and confirm they pass against T030-T033's implementation (fixes any mismatch found, does not add new scenarios)

  **Result**: All 3 new tests (T034's two plus T035's one) pass on first
  run once T030-T033 were in place. `cargo clippy --bin factory-canvas
  --all-features -- -D warnings` (production, no `--tests`) and the same
  command with `--tests` both pass clean — the dead-code warnings that
  were present after T007-T029 (since `undo`/`redo`/`Undone`/`Redone`
  were only reachable from tests until this phase) are gone now that
  production code (the header buttons and shortcut dispatcher) calls
  them too.

**Checkpoint**: Undo and Redo are reachable end-to-end through both the header and keyboard, gated identically to every other editing action.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final verification before this feature's commit(s) land

- [X] T037 Confirm every scenario walkthrough in `quickstart.md` is covered by one of T005-T036, with no manual-only step left unautomated beyond what quickstart.md itself marks `(manual only)`

  **Result**: 1 gap found and fixed (not just confirmed) — the "History
  is cleared on New/Open" scenario had no dedicated test; T017's
  `self.history.clear()` calls were only ever exercised incidentally
  through other tests that happened to call `new_document_at`/
  `open_document_from`, never asserting the history-clearing behavior
  itself. Added `history_is_cleared_on_new_document` and
  `history_is_cleared_on_open_document` in `src/egui_app_tests.rs` (both
  pass on first run). Every other scenario in quickstart.md now maps to
  an existing test with a matching name: the 6-commands undo/redo
  scenarios (T007-T011), redo scenarios (T021-T023), multi-step (T028),
  blocked-during-modal (T013/T024), both mechanical checks
  (T008/T011 for multi-entity atomicity, T014 for the allocator), and
  now history-clearing. The steps quickstart.md itself marks
  `(manual only)` (visual canvas confirmation, notice-text wording)
  remain manual, per this project's established preference for
  logical/deterministic validation over automated GUI capture.
- [X] T038 Confirm scope: `git status --short` and `git diff --stat` show changes only under `src/` and `specs/005-command-undo-redo/` — nothing under `catalog/`, `data/`, `.hermes/`, or any historical `specs/00N-*/` directory

  **Result**: Confirmed. `git status --short` lists 3 modified files
  (`src/egui_app.rs`, `src/egui_app_tests.rs`, `src/egui_main.rs`) plus 2
  untracked paths (`src/history.rs`, `specs/005-command-undo-redo/`).
  Nothing under `catalog/`, `data/`, `.hermes/`, or any historical
  `specs/00N-*/` directory appears in either command's output.
- [X] T039 Run the tests covering this feature's own changed files (`cargo test --lib`, plus whichever test binary contains `src/history.rs`'s and `src/egui_app_tests.rs`'s tests — confirm only new tests were added, nothing regressed), then the other five mandatory gates in order — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300` (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope)

**Result**: 203 tests in the `factory-canvas` binary (179 pre-existing +
24 new: 6 in `src/history.rs`'s own `#[cfg(test)] mod tests`, 18 in
`src/egui_app_tests.rs` — 8 for US1, 3 for US2, 1 for US3, 3 for the
UI cross-cutting phase, 2 for the history-is-cleared gap T037 found),
0 failed, 0 regressed. All six gates passed: `cargo fmt --check` (exit
0), `cargo clippy --bin factory-canvas --all-features -- -D warnings`
(exit 0, both with and without `--tests`), `cargo test --bin
factory-canvas` (exit 0, 203 passed), `cargo build --release --bins`
(exit 0), `git diff --check` (exit 0, no whitespace errors), `hermes
verify --skip-start --json --timeout 300` (`"ok": true` — its own
internal `cargo test` phase runs the full repository suite, which is
this gate's fixed, Constitution-named command rather than a choice made
here, and it independently confirms nothing regressed anywhere else in
the project either).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T006) completion
- **User Story 2 (Phase 4)**: Depends on User Story 1's `history`/`undo()` (T015-T019) — redo is defined symmetrically to undo and cannot be tested without it existing first
- **User Story 3 (Phase 5)**: Depends on both User Story 1 (T018) and User Story 2 (T025) — it validates their combination, not a new mechanism
- **Cross-Cutting UI (Phase 6)**: Depends on User Story 1 (T018) and User Story 2 (T025) — exposes both through one shared header/shortcut surface rather than wiring each story's trigger separately
- **Polish (Phase 7)**: Depends on every task above

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on User Story 2 or 3 — independently testable via `undo()` called directly, without any header/shortcut wiring
- **User Story 2 (P2)**: Depends on User Story 1's `EditHistory`/`undo()` existing (redo is `EditHistory`'s own symmetric stack operation, not a new independent mechanism) — independently testable via `redo()` called directly
- **User Story 3 (P3)**: Depends on both prior stories; its own independent test is a pure combination of `undo()`/`redo()` already delivered

Unlike Phase 5 (`004-blueprint-insertion-interfaces`), where US1 and US2 were genuinely independent of each other's domain/persistence layer, this feature's three user stories are *not* mutually independent: they are sequential depth increments over the same one `EditHistory` mechanism (single-step undo → single-step redo → multi-step undo/redo), each fully working before the next begins — an incremental-delivery shape spec.md's own "Why this priority" text for User Story 2 and 3 already states explicitly ("Completes the minimal undo/redo pair"; "Generalizes User Story 1 and User Story 2... strictly additive").

### Within Each User Story

- Tests and implementation for a story are interleaved above in the order that keeps every test compilable and meaningful the moment it is written (spec-driven, not strict red-green-refactor — see the Tests note at the top of this file)
- Story complete before moving to the next priority, per this project's established sequential-commit workflow

### Parallel Opportunities

- T007-T013 (User Story 1's editor-level tests) are marked `[P]` — independent test functions in the same file (`src/egui_app_tests.rs`) with no dependency on each other, differing only in which command scenario they construct; writing them is parallelizable even though this project's single-agent sequential-commit workflow (constitution "Workflow and Branching") means they are still executed one after another in practice, as `specs/004-blueprint-insertion-interfaces/tasks.md` already established for this project
- T021-T023 (User Story 2's tests) are marked `[P]` for the same reason
- T034-T035 (Cross-Cutting UI tests) are marked `[P]` for the same reason
- T004 is marked `[P]` — a module-declaration change independent of T003's own file content
- T014 is not marked `[P]` — it depends on T007 existing as a template for constructing a successful placement first
- T024, T028 are not marked `[P]` — each depends on an earlier test in its own phase existing as a template
- No implementation task (T015-T019, T025-T026, T030-T033) is marked `[P]`: each modifies a file another task in the same phase also modifies, or depends on the immediately preceding task's new signature

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T006) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T007-T020)
4. **STOP and VALIDATE**: run T007-T014 and confirm they pass in isolation
5. This alone already delivers spec.md US1's "Why this priority": a player who can reverse their very last mistake already has real protection they do not have today — even before Redo, multi-step history, or any header/keyboard trigger exist, since `undo()` is directly callable and testable as a `FactoryCanvasApp` method

### Incremental Delivery

1. Setup + Foundational → foundation ready (T001-T006)
2. Add User Story 1 → validate independently → this is the MVP (T007-T020)
3. Add User Story 2 → validate independently → completes the minimal undo/redo pair (T021-T027)
4. Add User Story 3 → validate independently → generalizes to multi-step history, expected to require no new production code (T028-T029)
5. Add Cross-Cutting UI → Undo/Redo become reachable by a real player through the header and `Ctrl+Z`/`Ctrl+Y` (T030-T036)
6. Polish (T037-T039) → all six gates green, commit(s)

---

## Notes

- **[Story] label** maps every task to its owning user story for traceability back to `spec.md`
- Tasks with no `[Story]` label (T001-T006, T030-T039) are Setup, Foundational, Cross-Cutting UI, or Polish — not story-specific
- This tasks.md, together with `plan.md`, `research.md`, `data-model.md`, and `quickstart.md`, is the complete spec-driven design record for this feature; implementation should not need to re-derive any decision already made in those files
- Per this project's established workflow (constitution "Workflow and Branching"), this is the second feature developed on its own branch rather than directly on `master` (the first was `004-blueprint-insertion-interfaces`) — commit narration, the six-gate verification (test gate scoped per Constitution v1.1.0/`docs/engineering-standards.md` §Testing scope), and independent review happen at commit-preparation time, per story or per logical group, against a feature branch merged back to `master` once complete
- Any deviation from this plan discovered during implementation must be narrated explicitly in the final report, per this project's established workflow (see `specs/001-blueprint-library/tasks.md`'s "Deviations from plan.md" section for the precedent this project follows)
