# Tasks: Visual Rotation Animation

**Input**: Design documents from `/specs/007-visual-rotation-animation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included. Each user story's `spec.md` "Independent Test" field and `quickstart.md`'s scenario walkthroughs are an explicit request for integration-test coverage of every acceptance scenario. Per the project constitution (Principle III), this feature is spec-driven rather than test-first: the spec/plan/tasks below were written, reviewed, and approved before any implementation, so tests are written alongside their behavior rather than as a mandatory pre-implementation RED step — but the tests covering this feature's own changed files (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope) and the other five gates must still pass before each commit lands.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (per plan.md's Structure Decision): `src/` at repository
root, no `tests/` directory involvement. Unlike Phase 6 and Phase 7 (which
concentrated entirely in `src/egui_app.rs`/`src/egui_app_tests.rs`), this
feature's implementation lives primarily in `src/egui_canvas.rs` (the
module `docs/architecture.md` already assigns "painter" work to), with a
small amount of wiring in `src/egui_app.rs` at the four call sites that
replace the whole layout. `src/egui_canvas.rs` already has its own
`#[cfg(test)] mod tests`, used for this feature's deterministic
time-driven tests. No `src/domain/`, `catalog/`, `src/history.rs`, or any
historical `specs/00N-*/` file changes (research.md; plan.md's Project
Structure).

## Phase 1: Setup

**Purpose**: Confirm the exact starting state this feature builds on

- [X] T001 Re-read `src/egui_canvas.rs`'s `show()` signature and `paint_instances` (the per-instance fill/stroke/symbol painting this feature adds an arrow alongside), `src/egui_app.rs`'s `FactoryCanvasApp` struct fields, `rotate_selected_clockwise` (both its single-instance and orbital-group branches), `new_document_at`, `open_document_from`, `replace_base`, and `apply_restored_snapshot` (the four call sites that replace `self.layout` wholesale outside a normal rotation), and `src/domain/layout.rs`'s `EntityId::value()`, `Rotation::clockwise()`, `BlockInstance::rotation()`/`origin()`, and confirm every signature this plan/data-model.md assumes still matches — no code change, a pre-flight confirmation only

  **Result**: Confirmed — every signature matches exactly. One
  consequence not spelled out in plan.md: `show()`'s existing
  `selected: &SelectedSet` parameter is `&self.selected` (an immutable
  borrow of one `FactoryCanvasApp` field) at its only call site
  (`canvas_ui`), so the new `rotation_visuals: &mut RotationVisuals`
  parameter this feature adds must borrow the DIFFERENT field
  `&mut self.rotation_visuals` — safe under Rust's disjoint-field-borrow
  rule since both are direct field accesses on the same struct, not
  borrows through an intervening `&self`/`&mut self` method call.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Introduce the one new shared type both user stories need before either can be implemented or tested

**⚠️ CRITICAL**: No user story work can begin until this phase is complete — both stories read and update the same `RotationVisuals` bookkeeping and rely on the same four resync call sites already discarding stale state correctly.

- [X] T002 In `src/egui_canvas.rs`, add a new `pub(crate) struct RotationVisuals` (research.md Decision 2-3, data-model.md): a per-`EntityId` record of last-known `Rotation`, last-known `GridPoint` origin, and an accumulated target angle in degrees (`f32`, monotonically increasing, never normalized until paint time), plus a one-shot instant-snap flag. Implement `new()` (empty) and `resync(&mut self, layout: &FactoryLayout)` (research.md Decision 3: rebuild every per-entity record from `layout`'s actual current state and arm the instant-snap flag) — no painting logic yet, no `egui::Context` dependency yet

  **Result**: Done, with one real correction to data-model.md's own
  informal "keyed by `EntityId`" description: `EntityId` does not derive
  `Hash` (confirmed by `cargo check` failing with
  `EntityId: Hash` not satisfied), and this feature's scope explicitly
  excludes any `src/domain/` change (plan.md, FR-010) — adding `Hash`
  there would be exactly that. Keyed the `HashMap` by `EntityId::value()`
  (the existing public `u64` accessor) instead, which is already the
  same pattern `egui::Id::new((entity_id.value(), ...))`-style code in
  this project would use for a similar purpose. No semantic change:
  `u64` values are just as unique as their `EntityId` wrappers.
- [X] T003 In `src/egui_app.rs`, add a `rotation_visuals: RotationVisuals` field to `FactoryCanvasApp` (initialized via `RotationVisuals::new()` in `Default`/`from_startup_catalog`, mirroring the existing `canvas: CanvasState` field's placement) (depends on T002)

  **Result**: Done exactly as planned.
- [X] T004 In `src/egui_app.rs`, call `self.rotation_visuals.resync(&self.layout)` immediately after each of the four places that replace `self.layout` wholesale — `new_document_at`, `open_document_from`, `replace_base`, and `apply_restored_snapshot` (used by both `undo` and `redo`) — per research.md Decision 3 and FR-008/FR-009 (depends on T003)

  **Result**: Done exactly as planned, all four call sites.

**Checkpoint**: Foundation ready — user story implementation can now begin. `RotationVisuals` exists and stays correctly synced at every non-rotation layout replacement, but nothing is painted yet (no visible behavior change until User Story 1).

---

## Phase 3: User Story 1 - A rotated block visibly turns in place (Priority: P1) 🎯 MVP

**Goal**: Every placed instance shows a small orientation indicator that
matches its current rotation at rest, and a single selected instance's
indicator turns smoothly from its old orientation to its new one whenever
the domain accepts a rotation — never when one is rejected, and never as
a side effect of undo/redo, New, Open, or a base change.

**Independent Test**: Place one instance, confirm its orientation
indicator is visible and points correctly at rest; trigger a rotation and
confirm the indicator's angle interpolates smoothly between the old and
new orientation over simulated time, ending exactly at the new value;
confirm a rejected rotation attempt produces no interpolation at all; and
confirm undo, redo, New, Open, and a base change each leave the indicator
at its correct resting angle with zero interpolation frames.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T005 [P] [US1] Deterministic test `orientation_indicator_matches_resting_rotation_for_every_instance` in `src/egui_canvas.rs`'s test module (spec.md US1/AC1, FR-001, FR-002): place two or more instances at different `Rotation` values with no transition ever triggered, render one frame, and assert each instance's computed/painted arrow angle equals its `Rotation`'s degree value exactly — proves the indicator is present and correct even before any rotation ever happens
- [X] T006 [P] [US1] Deterministic test `single_instance_rotation_animates_smoothly_between_old_and_new_angle` in `src/egui_canvas.rs`'s test module (spec.md US1/AC2-AC3, FR-003): drive two frames with explicit, closely-spaced `egui::RawInput.time` values around a single accepted rotation, and assert the angle read partway through the configured transition window is strictly between the old and new resting angles (not equal to either) — proving genuine interpolation, not an instant jump
- [X] T007 [P] [US1] Deterministic test `rotation_transition_ends_exactly_at_the_domain_accepted_value` in `src/egui_canvas.rs`'s test module (spec.md US1/AC3, FR-007): drive a frame with `RawInput.time` set well past the transition window's end after a single accepted rotation, and assert the angle exactly equals the new `Rotation`'s degree value — zero drift between the final visual state and the actual stored state
- [X] T008 [P] [US1] Deterministic test `a_second_rotation_mid_transition_continues_from_the_current_angle` in `src/egui_canvas.rs`'s test module (spec.md Edge Cases, FR-006): trigger a rotation, advance time to partway through its transition, trigger a second rotation, and assert the second transition's starting value matches the first transition's in-flight interpolated value at that instant (research.md Decision 1's confirmed native `egui` retargeting behavior) — no jump back to the pre-first-rotation resting angle
- [X] T009 [P] [US1] Deterministic test `rejected_rotation_starts_no_transition_and_changes_nothing` in `src/egui_canvas.rs`'s test module (spec.md US1/AC4, FR-005): force a rotation attempt the domain rejects (for example, out of bounds), and assert the instance's `RotationVisuals` bookkeeping and painted angle are byte-for-byte unchanged from immediately before the attempt
- [X] T010 [P] [US1] Deterministic test `layout_replacing_operations_keep_rotation_instant_with_no_transition` in `src/egui_app_tests.rs` (spec.md FR-008, FR-009): for each of New, Open, a base change, and undo/redo (`apply_restored_snapshot`), start a rotation transition, then trigger the operation, and assert the very next frame's painted angle already equals the final resting value with no interpolated value observable — covers all four call sites T004 wires

  **Real bug found and fixed here**: `egui`'s own vendored
  `AnimationManager::animate_value` only ARMS a `0.0`-duration instant
  snap for the CALL AFTER the one that passes it — the call that passes
  `duration=0.0` itself still returns the old, pre-reset interpolated
  value (confirmed by reading `egui-0.36.1/src/animation_manager.rs`
  directly, and by this exact test failing first: `undo` produced angle
  `90.0` instead of the expected `0.0`). `resync()` alone was not
  sufficient — the fix adds one throwaway `animate_value_with_time(...,
  0.0)` read inside `visual_state_for` before the real read, consuming
  the stale value so the frame right after a resync reads the
  already-settled angle with zero observable interpolation, satisfying
  FR-008/FR-009.

### Implementation for User Story 1

- [X] T011 [US1] In `src/egui_canvas.rs`'s `paint_instances`, for each instance, compute its displayed angle by calling `egui::Context::animate_value_with_time` (or the instant-snap path when `RotationVisuals`' one-shot flag is armed, per research.md Decision 1/3) with `RotationVisuals`' accumulated target for that instance, updating the per-entity record when a real `Rotation` change is detected (research.md Decision 4), then paint a small arrow at that angle alongside the existing fill/stroke/symbol drawing (depends on T002-T004)

  **Result**: Done, with two implementation details beyond the task's
  literal description: (1) `paint_instances` now also reads the
  interpolated fractional origin (`visual_state_for`'s `x`/`y`) instead
  of the domain's integer `GridPoint`, via a new
  `footprint_screen_rect_fractional` helper — required for the group
  rotation's position to visibly SLIDE (spec.md FR-004) rather than jump
  tile-to-tile once rounded; (2) the arrow is a simple filled triangle
  (`egui::Shape::convex_polygon`) computed from the angle via basic
  trigonometry, matching the "temporary placeholder until real
  icons/sprites exist" decision from `/speckit-specify`'s clarify round.
  `paint_instances` now takes `&mut RotationVisuals` and calls
  `consume_instant_sync()` once after its per-instance loop, per
  data-model.md's documented one-shot lifecycle.
- [X] T012 [US1] Run T005-T010 and confirm they pass against T011's implementation (fixes any mismatch found, does not add new scenarios) (depends on T011)

**Checkpoint**: At this point, User Story 1 is fully functional and testable independently — single-instance rotation is now visibly animated, every instance has a correct resting orientation indicator, and the four layout-replacing operations remain perfectly instant.

---

## Phase 4: User Story 2 - A rotated group visibly turns and resettles together (Priority: P2)

**Goal**: When two or more selected instances rotate together around
their shared pivot, every member visibly slides and turns from its old
position and orientation to the new one the domain's orbital rotation
already computed, all resettling together.

**Independent Test**: Select two or more instances, trigger rotation, and
confirm every member's position and angle interpolate together toward
their domain-accepted final values over simulated time, ending exactly
at those values with the shared pivot behavior unaffected.

### Tests for User Story 2

- [X] T013 [P] [US2] Deterministic test `group_rotation_animates_position_and_angle_together_for_every_member` in `src/egui_canvas.rs`'s test module (spec.md US2/AC1, FR-004): select two or more instances, drive two frames with explicit `RawInput.time` values around an accepted orbital rotation, and assert every member's painted origin AND angle are both strictly between their old and new domain-accepted values partway through the transition, ending exactly at the final values well past the transition window (spec.md US2/AC2) — reusing T007's "ends exactly at the accepted value" style of assertion, extended to origin
- [X] T014 [P] [US2] Deterministic test `plain_move_without_rotation_never_animates_position` in `src/egui_canvas.rs`'s test module (research.md Decision 4): move a selected instance via the existing arrow-key/button move action (no rotation involved), and assert its painted origin snaps directly to the new position with no interpolation — confirms position animation is gated on a real per-instance `Rotation` change, never triggered by an ordinary move alone

### Implementation for User Story 2

- [X] T015 [US2] Extend `paint_instances`' per-instance computation from T011 so that whenever a real `Rotation` change is detected for an instance (research.md Decision 4), its origin is ALSO animated (via the same `animate_value_with_time` mechanism, one call per axis) from its previous stored origin to its current one, synchronized with the angle animation already in place; an instance whose origin changes without its own `Rotation` changing continues to render at its current origin directly, with no interpolation (depends on T011)

  **Two real production bugs found and fixed here** (both confirmed by
  T013 failing first, following this project's established TDD
  pattern): (1) the x/y animation entries were originally only touched
  when `rotated` was true, so the very first rotation of a group member
  was itself unanimated — `egui`'s `animate_value` returns its target
  directly, with zero transition, the first time it ever sees a given
  `Id`. Fixed by reading x/y on every frame with a duration of `0.0`
  whenever not `rotated`. (2) `rotated` is only ever true on the SINGLE
  frame a rotation is first detected — using it to gate the origin's
  animation duration meant every subsequent frame of that same,
  still-in-flight transition passed `duration=0.0`, which the vendored
  `AnimationManager::animate_value` treats as "snap here now",
  prematurely freezing the position mid-slide. Fixed by introducing a
  separate `plain_move` signal (origin changed this frame with NO
  rotation detected this same frame) that is the ONLY thing allowed to
  force an instant `0.0` duration — every other frame, including every
  later frame of an in-flight rotation, now consistently reuses the
  angle's own `duration`. A third, related bug in the shared
  `animate_instant_or_transition` helper (T011) was generalized while
  fixing these: ANY call that changes an animation's target — not only
  ones during a `resync()` — returns the pre-change value on that exact
  call, so the helper now always performs its one throwaway
  zero-duration read whenever `duration == 0.0`, not only when
  `instant_sync_pending`.
- [X] T016 [US2] Run T013-T014 and confirm they pass against T015's implementation (fixes any mismatch found, does not add new scenarios) (depends on T015)

**Checkpoint**: All user stories are now independently functional. The remaining acceptance scenarios (watching the smooth motion actually render) are inherently visual and are covered by quickstart.md's **(manual only)** scenarios rather than by an automated test, per this project's established preference for logical/deterministic validation over automated GUI capture.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Close out the feature exactly like every prior phase

- [X] T017 Confirm every scenario walkthrough in `quickstart.md` is covered by one of T005-T016, with no manual-only step left unautomated beyond what quickstart.md itself marks `(manual only)`

  **Result**: Confirmed by direct cross-reference of every `##
  Scenario:` heading. "A rotated block visibly turns" → T005 (resting
  angle)/T006-T007 (interpolation + exact end value), manual-only part
  deferred to T019. "A rejected rotation never animates" → T009. "A
  rotated group visibly turns and resettles together" → T013, manual-
  only part deferred to T019. "A new rotation mid-transition continues
  smoothly" → T008. "Undo/redo, New, Open, and a base change stay
  perfectly instant" → T010. The mechanical scope check is T018 below.
- [X] T018 Confirm scope: `git status --short` and `git diff --stat` show changes only under `src/egui_canvas.rs`, `src/egui_app.rs`, `src/egui_app_tests.rs`, and `specs/007-visual-rotation-animation/` — nothing under `catalog/`, `data/`, `.hermes/`, `src/domain/`, `src/persistence/`, `src/history.rs`, or any historical `specs/00N-*/` directory

  **Result**: Confirmed exactly as expected — `git status --short`
  shows `M src/egui_app.rs`, `M src/egui_app_tests.rs`,
  `M src/egui_canvas.rs`, `?? specs/007-visual-rotation-animation/` and
  nothing else. `git diff --stat`: 3 files changed, 710 insertions(+), 6
  deletions(-) (7 in `egui_app.rs`, 176 in `egui_app_tests.rs`, 533 in
  `egui_canvas.rs`) — confirms FR-010 (zero domain/catalog/persistence
  change) held throughout implementation, not just at planning time.
- [X] T019 Produce the manual test script for quickstart.md's `(manual only)` scenarios (watching a single-instance rotation turn smoothly; watching a group rotation slide and turn together) for Diogo to run, per `docs/roadmap.md`'s "Engineering workflow per slice" step 5 — does not block gates or publication

  **Result**: Written to
  `.hermes/reports/2026-09-12-phase-8-manual-test.md` (gitignored, same
  as the Phase 5/7 precedents), 5 cases: orientation indicator always
  visible + smooth single-instance turn (US1), rejected rotation
  animates nothing (US1 AC4), synchronized group slide+turn (US2), a
  plain move never animates (research.md Decision 4 — the negative case
  quickstart.md itself doesn't spell out but this feature's scope
  explicitly requires), and undo/redo/New/Open/base-change staying
  instant (FR-008/FR-009). Status `PENDING` for all 5 cases, per this
  project's established convention that manual testing does not block
  gates or publication.
- [X] T020 Run the tests covering this feature's own changed files (`cargo test --bin factory-canvas` or scoped equivalent — confirm only new tests were added, nothing regressed), then the other five mandatory gates in order — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300` (Constitution Principle IV, `docs/engineering-standards.md` §Testing scope)

  **A real design bug was found and fixed while running the clippy
  gate**: `egui_canvas::show()` had grown to 8 parameters (adding
  `rotation_visuals` in T011 pushed it over clippy's
  `too_many_arguments` limit of 7). Rather than silence the lint with
  `#[allow(...)]` (no precedent for that anywhere in this codebase),
  moved `rotation_visuals` to live INSIDE `CanvasState` alongside
  `viewport`/`interaction`/`focus_selection_requested` — genuine domain
  modeling, not an artificial parameter object, since `CanvasState`
  already exists specifically to group this exact kind of per-canvas
  mutable state. `FactoryCanvasApp`'s own now-redundant
  `rotation_visuals` field was removed; all four resync call sites and
  every test now go through `self.canvas.rotation_visuals` /
  `app.canvas.rotation_visuals`. This also surfaced one genuinely dead
  method, `RotationVisuals::new()` (only ever called by this feature's
  own tests, never by production code once `CanvasState`'s own
  `#[derive(Default)]` took over construction) — removed per this
  project's established YAGNI convention, tests switched to
  `RotationVisuals::default()`.

**Result**: All 6 gates green. `cargo test --bin factory-canvas`: 219
passed, 0 failed (208 pre-existing + 11 new: 9 from US1's T005-T010, 2
from US2's T013-T014). `cargo fmt --check`: clean. `cargo clippy
--all-targets --all-features -- -D warnings`: clean (after the
`too_many_arguments` fix above). `cargo build --release --bins`: clean,
1m45s. `git diff --check`: clean (only pre-existing CRLF warnings, not
errors). `hermes verify --skip-start --json --timeout 300`: `"ok":
true`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS both user stories, since both read/update the same `RotationVisuals` bookkeeping and rely on the same four resync call sites
- **User Story 1 (Phase 3)**: Depends on Phase 2 (T002-T004). This phase delivers the arrow indicator and single-instance rotation animation — the entire visible MVP of this feature
- **User Story 2 (Phase 4)**: Depends on User Story 1's T011 (the per-instance angle-computation path it extends to also cover origin) — not independently implementable before US1, though it is independently *testable* as its own acceptance criterion once US1 lands
- **Polish (Phase 5)**: Depends on both user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependency on any other story
- **User Story 2 (P2)**: Depends on User Story 1's `paint_instances` angle-animation path (T011) already being in place to extend — not independently implementable before US1

### Within Each User Story

- Tests written and run to fail before implementation (T005-T010 before T011; T013-T014 before T015)
- Implementation before the confirmation/fix step
- Story complete before moving to the next priority

### Parallel Opportunities

- T005, T006, T007, T008, T009 (all US1 tests in `src/egui_canvas.rs`'s test module, different test functions, no shared mutable state) can be written in parallel; T010 (in `src/egui_app_tests.rs`) is also independent of them
- T013, T014 (both US2 tests, different test functions in the same module) can be written in parallel
- T002 and T003 are sequential (T003's field declaration needs T002's type to exist)

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Deterministic test orientation_indicator_matches_resting_rotation_for_every_instance in src/egui_canvas.rs"
Task: "Deterministic test single_instance_rotation_animates_smoothly_between_old_and_new_angle in src/egui_canvas.rs"
Task: "Deterministic test rotation_transition_ends_exactly_at_the_domain_accepted_value in src/egui_canvas.rs"
Task: "Deterministic test a_second_rotation_mid_transition_continues_from_the_current_angle in src/egui_canvas.rs"
Task: "Deterministic test rejected_rotation_starts_no_transition_and_changes_nothing in src/egui_canvas.rs"
Task: "Deterministic test layout_replacing_operations_keep_rotation_instant_with_no_transition in src/egui_app_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (the shared `RotationVisuals` type and its four resync call sites — no visible behavior change yet)
3. Complete Phase 3: User Story 1 — every instance gets a correct orientation indicator, and single-instance rotation animates smoothly
4. **STOP and VALIDATE**: Test User Story 1 independently
5. User Story 2 (Phase 4) extends the same painting path to also animate position for the group-rotation case

### Incremental Delivery

1. Complete Setup + Foundational → the shared bookkeeping exists but nothing paints differently yet
2. Add User Story 1 → test independently → this alone satisfies spec.md's core visibility requirement (MVP): every block shows its orientation, and rotating one visibly turns it
3. Add User Story 2 → test independently → group rotation now also slides and turns together → close out with Polish

### Parallel Team Strategy

Not applicable at this feature's scale — one implementer completes
T001-T020 sequentially within the phase/story order above.

---

## Notes

- [P] tasks = different files, or different test functions in the same file with no shared mutable state
- [Story] label maps task to specific user story for traceability
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- This feature's one real technical risk (spec.md's implicit question of whether `egui` can animate values at all without a new dependency, and whether mid-flight retargeting works correctly for FR-006) was already resolved during `/speckit-plan` via a disposable spike (research.md Decision 1) — no task here re-litigates it; T006-T008 simply assert the now-known-safe outcome
