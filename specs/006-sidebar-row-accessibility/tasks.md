# Tasks: Sidebar Instance Row Accessibility

**Input**: Design documents from `/specs/006-sidebar-row-accessibility/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included. Each user story's `spec.md` "Independent Test" field and `quickstart.md`'s scenario walkthroughs are an explicit request for integration-test coverage of every acceptance scenario. Per the project constitution (Principle III), this feature is spec-driven rather than test-first: the spec/plan/tasks below were written, reviewed, and approved before any implementation, so tests are written alongside their behavior rather than as a mandatory pre-implementation RED step — but the tests covering this feature's own changed files (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope) and the other five gates must still pass before each commit lands.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (per plan.md's Structure Decision): `src/` at repository
root, no `tests/` directory involvement. This feature concentrates
entirely in one existing method in `src/egui_app.rs` and its existing
test coverage in `src/egui_app_tests.rs`. No new module, no
`domain/`/`persistence/`/`egui_canvas.rs` file changes (research.md
Decisions 1-4; plan.md's Project Structure). No `history.rs` or any
other Phase 6 file is touched — this feature is independent of Phase 6.

## Phase 1: Setup

**Purpose**: Confirm the exact starting state this feature builds on

- [X] T001 Re-read `src/egui_app.rs`'s `sidebar_ui` "INSTANCES ON CANVAS" loop (the `egui::Label`-based row, its `ui.add_sized([ui.available_width(), 0.0], ...)` call, and the `response.clicked_by(egui::PointerButton::Primary)` + modifier-reading dispatch immediately below it), `instance_semantic_label`, `block_palette_ui`'s existing `Button::new(label).selected(selected)` call (the pattern this feature reuses), and `src/egui_app_tests.rs`'s `right_sidebar_frame`, `accesskit_node_text`, `accesskit_node_center`, `primary_click`, and `production_test_app` helpers, and confirm every signature this plan/data-model.md assumes still matches — no code change, a pre-flight confirmation only

  **Result**: Confirmed — every signature matches exactly. One real
  correction to this task file's own T005 assumption: there is no
  existing "sidebar-row click coverage pattern" for `Shift`/`Ctrl`
  exercised via a simulated mouse event anywhere in
  `src/egui_app_tests.rs` today — every existing `SelectionMode::Add`/
  `Toggle` test calls `apply_canvas_interaction`/`selected.apply`
  directly, never a simulated click carrying `egui::Modifiers`. T005
  will construct that event from scratch using the `modifiers` field
  `egui::Event::PointerButton` already exposes (currently only ever set
  to `Modifiers::NONE` by `primary_click`) rather than reuse a
  pre-existing helper.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None required

**⚠️ CRITICAL**: This feature introduces no new type, module, or shared
infrastructure (plan.md's Summary: "no new type at all"). There is
nothing here for a User Story to depend on beyond Phase 1's
confirmation — User Story 1 can begin immediately after T001.

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Assistive technology recognizes a sidebar row as a real selectable control (Priority: P1) 🎯 MVP

**Goal**: Every row in the sidebar's "INSTANCES ON CANVAS" list is
exposed to assistive technology as an interactive, selectable control
that reports its own accurate selected/unselected state, with its
complete existing information (identifier, name, origin, footprint,
rotation, product) fully preserved.

**Independent Test**: Render the sidebar with several instances, some
selected and some not, inspect the AccessKit tree, and confirm every
row's node has `role() == Role::Button`, an accurate individual
`toggled()` value, and a `label()` matching `instance_semantic_label`'s
full, untruncated output — independently of any visual/color-only
inspection.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T002 [P] [US1] Integration test `instance_row_is_exposed_as_a_selectable_button` in `src/egui_app_tests.rs` (spec.md US1/AC1, data-model.md's AccessKit contract): render the sidebar with one placed instance via `right_sidebar_frame`, find that instance's row node by matching its `label()` against `instance_semantic_label`'s expected output, and assert `role() == egui::accesskit::Role::Button` (not `Role::Label`)
- [X] T003 [P] [US1] Integration test `each_selected_row_reports_its_own_toggled_state_independently` in `src/egui_app_tests.rs` (spec.md US1/AC2-AC3, data-model.md): place three instances, select two of them via `SelectionMode::Add` (leaving the third unselected), render the sidebar, and assert the two selected rows' nodes each have `toggled() == Some(egui::accesskit::Toggled::True)` and the unselected row's node has `Some(egui::accesskit::Toggled::False)` — checked independently per row, not only for the most recently selected one
- [X] T004 [P] [US1] Integration test `instance_row_label_stays_complete_for_a_long_realistic_value` in `src/egui_app_tests.rs` (spec.md US1/AC4, FR-003): place an instance configured with `production_test_app`'s longest available product display name, render the sidebar, and assert the row's node `label()` equals `instance_semantic_label`'s full output string byte-for-byte — nothing shortened, cut off, or replaced with an elision marker

  **Result**: Rewritten from the task's original plan during writing —
  comparing `label()` text to itself after finding the node BY that same
  text is tautological and proves nothing about clipping (confirmed by
  inspecting the `/speckit-plan` spike's own data: AccessKit's `label()`
  carried the full string identically across `Wrap`/`Truncate`/`Extend`
  modes even when rendered height stayed single-line). Rewritten to
  compare rendered row **height** between a short-label and a long-label
  instance in the same catalog/render pass — a row whose full text
  actually wrapped renders measurably taller; one that was clipped to one
  line would not. Also needed a much longer buildable/product name than
  first planned: `right_sidebar_frame`'s test screen is 420px wide (not
  this project's real 264px sidebar), so the original moderately-long
  string still fit on one line at that width and had to be lengthened
  until it demonstrably wrapped. Passes today (GREEN on the first
  correct version) because `.wrap()` already exists on the current
  `Label` — this task is a regression guard for T006's widget swap, not
  proof of a current bug.
- [X] T005 [P] [US1] Integration test `sidebar_row_click_selection_modifiers_are_unaffected` in `src/egui_app_tests.rs` (spec.md FR-004): reuse this project's existing sidebar-row click coverage pattern (plain click via `primary_click`/`accesskit_node_center` on a row found by its `Role::Button` node, `Shift`, and `Ctrl`) and assert `SelectionMode::Replace`/`Add`/`Toggle` still produce the exact same `self.selected` outcome as before this feature — a regression check, not new behavior

  **Result**: As T001 flagged, no `Shift`/`Ctrl` simulated-click helper
  existed; built the `egui::Event::PointerButton { modifiers: ..., ... }`
  sequence directly using `Modifiers::SHIFT`/`Modifiers::CTRL`.

### Implementation for User Story 1

- [X] T006 [US1] In `src/egui_app.rs`'s `sidebar_ui` "INSTANCES ON CANVAS" loop, replace the `egui::Label::new(...).wrap().sense(egui::Sense::click())` row widget with `Button::new(RichText::new(instance_semantic_label(...)).size(11.0)).selected(self.selected.contains(id)).wrap()`, per research.md Decision 1 — the `response.clicked_by(...)` + modifier dispatch immediately below stays unchanged (depends on T001)

  **Result**: Done — `egui::Label::new(...).sense(Sense::click())` replaced
  with `egui::Button::new(RichText::new(label).size(11.0)).selected(is_selected).wrap()`.
  The `response.clicked_by(...)` dispatch below was untouched, as planned.
- [X] T007 [US1] In the same loop, delete the row's now-redundant manual `.color(if self.selected.contains(id) { ACCENT } else { TEXT_PRIMARY })` branch on the `RichText`, per research.md Decision 3 — `.selected(...)` on the `Button` is the row's sole source of selected-state appearance (depends on T006)

  **Result**: Done — the manual `.color(...)` branch is gone; `.selected(...)`
  is now the row's only source of selected-state appearance, matching
  `block_palette_ui`'s existing pattern exactly.
- [X] T008 [US1] Run T002-T005 and confirm they pass against T006-T007's implementation (fixes any mismatch found, does not add new scenarios) (depends on T007)

  **Result**: All 4 pass, but getting there required fixing real bugs in
  the TESTS themselves (not production code) found while running this
  step — recorded here since they surfaced during T008, not earlier:
  1. T004's first draft picked the wrong row (matched the block palette's
     `block_option_label` row by content collision, not the instance row)
     — fixed by asserting on a fresh two-buildable catalog (one short
     display name, one deliberately very long) instead of reusing one
     buildable for both a "short" and "long" case.
  2. T005's `Shift`/`Ctrl` frames originally appended a trailing
     `ModifiersChanged(NONE)` to "reset after" — but egui resolves a whole
     frame's `input.modifiers` from the LAST `ModifiersChanged` event in
     that frame's own event list, so the trailing reset made the click
     itself see `shift=false`/`ctrl=false`. Fixed by leaving the reset to
     the next frame's own leading `ModifiersChanged`, never appending one
     after a click in the same frame.
  3. T005's Ctrl-click step reused `center_1` computed before the
     shift-click frame — but selecting a second instance changes the
     "EDITOR STATUS" notice text above the instance list (`"1 selected"` →
     `"2 selected"`), which changes that notice's wrapped height and shifts
     every row below it. Fixed by re-querying row 1's position immediately
     before the Ctrl-click, exactly like production code (and every other
     row lookup in this file) already does — never caching a row's
     screen position across a frame that could have changed layout above it.
  None of these three were production-code bugs; all were test-construction
  mistakes caught and fixed before this task was marked done.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently — this is also the entire scope of this feature's implementation; User Story 2 below adds no further production code.

---

## Phase 4: User Story 2 - Sidebar rows look and behave like the app's other selectable controls (Priority: P2)

**Goal**: A sidebar instance row visually conveys the same
selectable-control affordance the block palette's options already show,
and a row holding keyboard focus is visually distinguishable from one
that does not.

**Independent Test**: Compare a rendered instance row's appearance to a
block palette option's appearance, and confirm a row can receive
keyboard focus with a distinguishable visual state.

### Tests for User Story 2

- [X] T009 [P] [US2] Integration test `instance_row_supports_keyboard_focus_like_a_button` in `src/egui_app_tests.rs` (spec.md US2/AC2, FR-007): render the sidebar with one instance, find its row node, and assert it `supports_action(egui::accesskit::Action::Focus)` — the same focusability contract already true of the block palette's `Button`-based options and now true of this row (research.md Decision 2: `Sense::click()` already implies focusability, so no new code should be needed for this to already pass once T006 lands)

  **Result**: Passes on first run — confirms research.md Decision 2.

### Implementation for User Story 2

- [X] T010 [US2] Run T009 and confirm it already passes against User Story 1's T006-T007 implementation — per research.md Decision 2, no new production code is expected here; if it does not already pass, investigate why `Button`'s default focusability was not inherited and fix the minimal cause (depends on T006, T009)

  **Result**: Already passed — no production code change needed for US2.

**Checkpoint**: All user stories should now be independently functional. The remaining acceptance criteria for User Story 2 (FR-006's visual affordance match, FR-007's visible focus distinction) are inherently visual and are covered by quickstart.md's two **(manual only)** scenarios rather than by an automated test, per this project's established preference for logical/deterministic validation over automated GUI capture (`docs/engineering-standards.md` §TDD: "use a manual checklist for visual interaction").

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Close out the feature exactly like every prior phase

- [X] T011 Confirm every scenario walkthrough in `quickstart.md` is covered by one of T002-T010, with no manual-only step left unautomated beyond what quickstart.md itself marks `(manual only)`

  **Result**: Confirmed — 4 automatable scenarios map 1:1 to T002/T003/
  T004/T005; the 2 `(manual only)` scenarios (focused-row visual
  distinction, row-vs-palette appearance match) are correctly left for
  T013's manual script; the FR-005 fallback-not-triggered check needs no
  test of its own (already resolved by the `/speckit-plan` spike, cited
  in research.md); the scope-check scenario is T012. No gap found.
- [X] T012 Confirm scope: `git status --short` and `git diff --stat` show changes only under `src/egui_app.rs`, `src/egui_app_tests.rs`, and `specs/006-sidebar-row-accessibility/` — nothing under `catalog/`, `data/`, `.hermes/`, `src/domain/`, `src/persistence/`, `src/history.rs`, or any historical `specs/00N-*/` directory

  **Result**: Confirmed — `git status --short` shows exactly
  `M src/egui_app.rs`, `M src/egui_app_tests.rs`, and the untracked
  `specs/006-sidebar-row-accessibility/` directory. `git diff --stat`:
  13 lines changed in `egui_app.rs` (net -9, the widget swap plus deleted
  color branch), 303 lines added in `egui_app_tests.rs` (5 new tests).
  Nothing else touched.
- [X] T013 Produce the manual test script for quickstart.md's two `(manual only)` scenarios (a focused row's visual distinction; a row's appearance matching the block palette's affordance) for Diogo to run, per `docs/roadmap.md`'s "Engineering workflow per slice" step 5 — does not block gates or publication

  **Result**: Written to
  `.hermes/reports/2026-09-12-phase-7-manual-test.md`, following the
  Phase 5 manual-test script's established format (Preparação/Caso N/
  Resultado reportado/Conclusão). Status `PENDING` — does not block
  this feature's gates or commit, per established project convention.
- [X] T014 Run the tests covering this feature's own changed files (`cargo test --test <file>` / `cargo test <name>` for whichever test binary contains `src/egui_app_tests.rs`'s tests — confirm only new tests were added, nothing regressed), then the other five mandatory gates in order — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300` (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope)

**Result**: All six gates green.
- Test (`cargo test --bin factory-canvas`, this feature's scope): **208
  passed**, 0 failed (203 pre-existing + 5 new: T002, T003, T004, T005,
  T009). No regressions.
- `cargo fmt --check`: EXIT_0.
- `cargo clippy --all-targets --all-features -- -D warnings`: EXIT_0, no
  warnings.
- `cargo build --release --bins`: EXIT_0.
- `git diff --check`: EXIT_0 (only a pre-existing CRLF/LF line-ending
  notice, not an error).
- `hermes verify --skip-start --json --timeout 300`: `"ok": true`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Empty — nothing blocks User Story 1
- **User Story 1 (Phase 3)**: Depends on Phase 1 (T001) only. This phase alone delivers 100% of this feature's production code (T006-T007)
- **User Story 2 (Phase 4)**: Depends on User Story 1's T006 (the widget swap `Button`'s inherited focusability comes from) — adds a test only, no new production code expected
- **Polish (Phase 5)**: Depends on both user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Setup (Phase 1) — no dependency on any other story
- **User Story 2 (P2)**: Depends on User Story 1's `Button` widget swap (T006) already being in place to verify against — not independently implementable before US1, though it is independently *testable* as its own acceptance criterion once US1 lands

### Within Each User Story

- Tests written and run to fail before implementation (T002-T005 before T006-T007; T009 before confirming T010)
- Implementation before the confirmation/fix step
- Story complete before moving to the next priority

### Parallel Opportunities

- T002, T003, T004, T005 (all US1 tests, different test functions in the same file, no shared mutable state) can be written in parallel
- T006 and T007 are sequential (both edit the same loop in the same method)
- T009 (US2's single test) has no parallel sibling within its own phase

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Integration test instance_row_is_exposed_as_a_selectable_button in src/egui_app_tests.rs"
Task: "Integration test each_selected_row_reports_its_own_toggled_state_independently in src/egui_app_tests.rs"
Task: "Integration test instance_row_label_stays_complete_for_a_long_realistic_value in src/egui_app_tests.rs"
Task: "Integration test sidebar_row_click_selection_modifiers_are_unaffected in src/egui_app_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (empty — no work)
3. Complete Phase 3: User Story 1 — this is the entire feature's production code
4. **STOP and VALIDATE**: Test User Story 1 independently
5. User Story 2 (Phase 4) is a thin confirmation pass on top of the same code, not a second increment of production work

### Incremental Delivery

1. Complete Setup (Foundational is empty) → immediately ready for User Story 1
2. Add User Story 1 → test independently → this alone satisfies spec.md's core accessibility requirement (MVP)
3. Add User Story 2 → confirm the same widget swap already satisfies the visual/focus polish criteria → close out with Polish

### Parallel Team Strategy

Not applicable at this feature's scale — a single-widget substitution in
one method is smaller than any prior phase's smallest unit of parallel
work; one implementer completes T001-T014 sequentially within the story
order above.

---

## Notes

- [P] tasks = different files, no dependencies. Within this feature, "different files" narrows to "different test functions in the same file with no shared mutable state," since almost everything here lives in `src/egui_app.rs`/`src/egui_app_tests.rs`.
- [Story] label maps task to specific user story for traceability
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- This feature's real technical risk (spec.md FR-005's clipping question) was already resolved during `/speckit-plan` via a disposable spike (research.md Decision 1) — no task here re-litigates it; T002-T005 simply assert the now-known-safe outcome
