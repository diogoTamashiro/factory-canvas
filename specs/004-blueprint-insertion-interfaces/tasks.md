# Tasks: Blueprint Insertion and Exposed Interfaces

**Input**: Design documents from `/specs/004-blueprint-insertion-interfaces/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included. Each user story's `spec.md` "Independent Test" field and `quickstart.md`'s scenario walkthroughs are an explicit request for integration-test coverage of every acceptance scenario. Per the project constitution (Principle III), this feature is spec-driven rather than test-first: the spec/plan/tasks below were written, reviewed, and approved before any implementation, so tests are written alongside their behavior rather than as a mandatory pre-implementation RED step — but `cargo test` and the other five gates (Principle IV) must still pass before each commit lands.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

Single project (per plan.md's Structure Decision): `src/`, `tests/` at repository root. This feature concentrates in `src/domain/blueprint.rs`, `src/persistence/blueprint_document.rs`, `src/blueprint_library_view.rs`, `src/egui_app.rs`, and (only if confirmed necessary during implementation) `src/egui_canvas.rs`, plus `tests/domain_blueprint.rs` and `tests/blueprint_document_codec.rs`.

## Phase 1: Setup

**Purpose**: Confirm the exact starting state this feature builds on

- [X] T001 Re-read `src/domain/blueprint.rs`, `src/domain/layout.rs` (`FactoryLayout::place`, `move_instances_by`'s checked-arithmetic pattern), and `src/persistence/blueprint_document.rs` in full and confirm every signature this plan/data-model.md assumes (`Blueprint::from_selection`, `Blueprint::from_nodes`, `BlueprintCreationError`, `PlacementError`, `BlueprintDocumentV1Dto`) still matches what data-model.md documents — no code change, a pre-flight confirmation only

  **Result**: Confirmed. All signatures match data-model.md exactly. Notable confirmed details: `FactoryLayout::place` is the single validation authority (duplicate ID → buildable → product → spatial); `OccupiedRect` (private to `layout.rs`) already implements the union math research.md Decision 3 reuses conceptually (not directly callable from `blueprint.rs`, so T003's boundary helper reimplements the same union-of-rects logic locally over `BlueprintNode`s instead of instances); `BlueprintNodeValidationError`/`BlueprintCreationError` both currently `#[derive(..., Copy)]` (`BlueprintCreationError`) — adding a `Vec`-carrying variant to `BlueprintCreationError` in T032 will require dropping its `Copy` derive, noted for that task.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `Interface`/`Side` types both user stories' tests reference, and the DTO field User Story 2 persists

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Define `Side` (`North | East | South | West`, `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, matching `docs/data-model.md`'s "Planned physical ports" vocabulary per research.md Decision 3) and `Interface` (`name: String`, `anchor: GridPoint`, `side: Side`) in `src/domain/blueprint.rs` (depends on T001)
- [X] T003 Define `InterfaceError` (`BlankName { index: usize }`, `DuplicateName { index: usize }`, `NotOnBoundary { index: usize }`) and the boundary-validation helper (bounding-rectangle-of-nodes union plus outward-side check, per data-model.md/research.md Decision 3) in `src/domain/blueprint.rs` (depends on T002)
- [X] T004 Add `interfaces: Vec<InterfaceDto>` to `BlueprintDocumentV1Dto` with `#[serde(default)]` (research.md Decision 1), define `InterfaceDto` (`name: String`, `anchor: { x: i32, y: i32 }`, `side: "north" | "east" | "south" | "west"`), and wire encode/decode conversion to/from `domain::blueprint::Interface`/`Side` in `src/persistence/blueprint_document.rs` (depends on T002)
- [X] T005 In `tests/blueprint_document_codec.rs`, change `decoding_rejects_a_document_with_unknown_top_level_field`'s tamper key from `"interfaces"` to a still-genuinely-unknown field name (e.g. `"unknown_top_level_field"`) — required by research.md Decision 1's "Consequence for existing tests" note, since `interfaces` becomes a real field in T004 (depends on T004)

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Insert a saved blueprint into the current factory (Priority: P1) 🎯 MVP

**Goal**: A player picks a blueprint from the library and inserts it into the currently open factory as a single atomic batch of new, independently-selectable entities at a chosen point.

**Independent Test**: Save a blueprint from one factory, open or create a different factory, insert that blueprint, and confirm the inserted instances appear as new, independently selectable entities with fresh IDs, matching the blueprint's captured buildables, relative layout, rotations, and per-instance product configuration.

### Tests for User Story 1

- [X] T006 [P] [US1] Integration test `insert_into_places_every_node_at_its_relative_offset_with_fresh_sequential_ids` (spec.md US1/AC1) in `tests/domain_blueprint.rs`
- [X] T007 [P] [US1] Integration test `insert_into_preserves_buildable_rotation_and_configured_product_per_node` (spec.md US1/AC1, FR-003) in `tests/domain_blueprint.rs`
- [X] T008 [P] [US1] Integration test `insert_into_out_of_bounds_leaves_the_layout_completely_unchanged` (spec.md US1/AC2, FR-004) — asserts `Err(BlueprintInsertionError::OutOfBounds { .. })` and a byte-for-byte unchanged `FactoryLayout` in `tests/domain_blueprint.rs`
- [X] T009 [P] [US1] Integration test `insert_into_colliding_with_an_existing_instance_leaves_the_layout_completely_unchanged` (spec.md US1/AC3, FR-005) — asserts `Err(BlueprintInsertionError::Collision { .. })` and an unchanged layout in `tests/domain_blueprint.rs`
- [X] T010 [P] [US1] Integration test `insert_into_colliding_between_two_of_its_own_nodes_is_rejected` (spec.md FR-005's "or another entity being inserted in the same operation") in `tests/domain_blueprint.rs`
- [X] T011 [P] [US1] Integration test `insert_into_rejects_a_node_whose_buildable_is_absent_from_the_active_catalog` (spec.md US1/AC5, FR-006) — asserts `Err(BlueprintInsertionError::BuildableNotFound { .. })` and an unchanged layout in `tests/domain_blueprint.rs`
- [X] T012 [P] [US1] Integration test `insert_into_succeeds_when_every_reference_still_resolves_despite_a_different_catalog_data_version` (spec.md US1/AC5's success branch) in `tests/domain_blueprint.rs`
- [X] T013 [P] [US1] Integration test `insert_into_rejects_entity_id_allocator_exhaustion_before_placing_anything` (research.md Decision 6) — `first_id` close to `u64::MAX` such that the batch would overflow, asserts `Err(BlueprintInsertionError::EntityIdsExhausted)` and an unchanged layout, in `tests/domain_blueprint.rs`
- [X] T014 [P] [US1] Integration test `insert_into_rejects_a_node_whose_translated_coordinate_would_overflow` (research.md Decision 6) — an insertion point near `i32::MAX`/`i32::MIN` such that `insertion_point + node.relative_origin` overflows, asserts `Err(BlueprintInsertionError::CoordinateOverflow { .. })` and an unchanged layout, in `tests/domain_blueprint.rs`
- [X] T015 [US1] Integration test `inserted_instances_are_individually_movable_rotatable_and_removable_with_no_group_link` (spec.md US1/AC4, FR-007) — after a successful `insert_into`, each new `EntityId` independently accepts `move_instance`/`rotate_instance`/`remove_instance` with no reference to any other newly-inserted ID, in `tests/domain_blueprint.rs` (depends on T006)

### Implementation for User Story 1

- [X] T016 [US1] Define `BlueprintInsertionError` (`CoordinateOverflow { node_index: usize }`, `EntityIdsExhausted`, `BuildableNotFound { node_index: usize, buildable_id: BuildableId }`, `ProductNotFound { node_index: usize, product_id: ProductId }`, `UnsupportedProduct { node_index: usize, buildable_id: BuildableId, product_id: ProductId }`, `OutOfBounds { node_index: usize }`, `Collision { node_index: usize, conflicting_id: EntityId }`) in `src/domain/blueprint.rs` (depends on T016)
- [X] T017 [US1] Implement `Blueprint::insert_into(&self, layout: &mut FactoryLayout, insertion_point: GridPoint, first_id: u64) -> Result<u64, BlueprintInsertionError>` per data-model.md/research.md Decision 2 and Decision 6 (data-model.md's "Insertion point" note documents a deviation from the original plan here — `insertion_point` is an explicit parameter, not implicitly pre-applied by the caller): pre-check allocator capacity and per-node coordinate arithmetic with `checked_add` before any mutation, clone `layout`, call the clone's `place()` once per node in ascending node order translating `PlacementError` variants to their `BlueprintInsertionError` counterparts with `node_index`, commit the clone into `*layout` only on full success, in `src/domain/blueprint.rs` (depends on T016)
- [X] T018 [US1] Run T006-T014 and confirm they pass against T017's implementation (fixes any mismatch found, does not add new scenarios)

  **Result**: All 10 pass. One test-authoring bug found and fixed (not a code bug):
  `inserted_instances_are_individually_movable_rotatable_and_removable_with_no_group_link`
  asserted the second node's post-move origin as `(38, 30)` instead of the
  correct `(40, 30)` (second source instance was placed at relative
  x-offset 10 from the first, not 8) — corrected the expected value.

**Checkpoint**: User Story 1 is fully functional and testable independently at the domain layer — T006-T015 pass using only T002-T005, T016-T017. UI wiring for User Story 1 continues below since spec.md's Acceptance Scenarios describe player-visible behavior.

### UI for User Story 1

- [X] T019 [US1] Add an "Insert" action to each library row rendered by `BlueprintLibraryView` (mirroring the existing per-row rendering already in `src/blueprint_library_view.rs`/`src/egui_app.rs` sidebar code) that arms the chosen blueprint for canvas placement, in `src/blueprint_library_view.rs` and `src/egui_app.rs`
- [X] T020 [US1] Generalize the existing "armed candidate" canvas interaction (`FactoryCanvasApp::selected_block: Option<BuildableId>`, `placement_buildable_for_canvas`, `CanvasInteraction::Place`, `place_selected_at` in `src/egui_app.rs`) to also carry an armed `Blueprint` (research.md Decision 5) — confirm during implementation whether `egui_canvas.rs`'s painter/hit-testing needs a distinct code path for a multi-node preview versus the existing single-buildable preview, and update plan.md's Project Structure note on `egui_canvas.rs` if it does, in `src/egui_app.rs` (and `src/egui_canvas.rs` only if confirmed necessary) (depends on T017, T019)
- [X] T021 [US1] Add `EditorNotice` variants for insertion outcomes (success, and one per `BlueprintInsertionError` category, following the existing `PlacementRejected(PlacementError)`/`EntityIdsExhausted` pattern) and a safe, non-blocking message mapping for each — no internal error detail, node index, or path leaks into the rendered text (matching `safe_blueprint_save_error_detail`'s existing discipline) — in `src/egui_app.rs` (depends on T020)
- [X] T022 [US1] Wire `next_entity_id` consumption for a successful insertion the same way `place_selected_at` already advances it (`self.next_entity_id = ...`, `self.session.mark_dirty()`), using `in...[truncated]

**Checkpoint**: User Story 1 is fully functional and testable independently, end to end.

---

## Phase 4: User Story 2 - Expose and name physical-port interfaces on a saved blueprint (Priority: P2)

**Goal**: While saving a selection as a blueprint, a player marks specific boundary locations as named interfaces, purely descriptive metadata with no connection/flow implication.

**Independent Test**: Save a selection as a blueprint, mark one or more boundary locations as interfaces with a distinct name each, reload the blueprint library, and confirm each interface's name and boundary location are preserved and visible without implying any connection state.

### Tests for User Story 2

- [X] T023 [P] [US2] Integration test `from_selection_with_one_valid_interface_persists_its_name_and_location` (spec.md US2/AC1) in `tests/domain_blueprint.rs`
- [X] T024 [P] [US2] Integration test `from_selection_with_zero_interfaces_remains_a_fully_valid_blueprint` (spec.md US2/AC2, FR-011) — non-regression check that existing no-interface saves still succeed unmodified, in `tests/domain_blueprint.rs`
- [X] T025 [P] [US2] Integration test `from_selection_rejects_two_interfaces_sharing_a_trimmed_name` (spec.md US2/AC3, FR-009) — asserts `Err(BlueprintCreationError::InvalidInterface(InterfaceError::DuplicateName { .. }))` in `tests/domain_blueprint.rs`
- [X] T026 [P] [US2] Integration test `from_selection_rejects_a_blank_or_whitespace_only_interface_name` (FR-009) — asserts `Err(BlueprintCreationError::InvalidInterface(InterfaceError::BlankName { .. }))` in `tests/domain_blueprint.rs`
- [X] T027 [P] [US2] Integration test `from_selection_rejects_an_interface_anchor_side_not_on_the_bounding_rectangle_boundary` (FR-009, research.md Decision 3) — an interior anchor and a side that does not point outward at that tile, each asserting `Err(BlueprintCreationError::InvalidInterface(InterfaceError::NotOnBoundary { .. }))`, in `tests/domain_blueprint.rs`
- [X] T028 [P] [US2] Integration test `from_selection_accepts_a_corner_anchor_with_either_of_its_two_outward_sides` (research.md Decision 3's corner-tile allowance) in `tests/domain_blueprint.rs`
- [X] T029 [P] [US2] Integration test `encode_then_decode_round_trips_interfaces_exactly` (spec.md US2's Independent Test — "reloading the blueprint library... preserved") in `tests/blueprint_document_codec.rs`
- [X] T030 [P] [US2] Integration test `decoding_a_pre_existing_document_with_no_interfaces_key_yields_an_empty_list` (research.md Decision 1 — backward compatibility for files saved before this feature) in `tests/blueprint_document_codec.rs`
- [X] T031 [P] [US2] Integration test `decoding_rejects_a_document_whose_interfaces_entry_has_a_blank_or_duplicate_name_or_an_off_boundary_anchor` (mirrors `Blueprint::from_nodes`'s existing "re-validate everything, all-or-nothing" contract, extended to interfaces) in `tests/blueprint_document_codec.rs`

### Implementation for User Story 2

- [X] T032 [US2] Extend `Blueprint::from_selection` with the new `interfaces: Vec<Interface>` parameter (validated via T003's helper alongside the existing selection validation, returning `BlueprintCreationError::InvalidInterface` on failure) and add `Blueprint::interfaces(&self) -> &[Interface]` in `src/domain/blueprint.rs` (depends on T003; every existing call site of `from_selection` — `src/blueprint_library_view.rs`, and every pre-existing test in `tests/domain_blueprint.rs`/`src/blueprint_library_view.rs` tests — updated to pass `Vec::new()` unless the call site is itself a new US2 test)
- [X] T033 [US2] Extend `Blueprint::from_nodes` (`src/persistence/blueprint_document.rs`'s reconstruction path) with the same `Vec<Interface>` parameter, re-validating exactly as T032 does, and wire T004's `InterfaceDto`s through the existing encode/decode calls in `src/persistence/blueprint_document.rs` (depends on T004, T032)
- [X] T034 [US2] Run T023-T031 and confirm they pass against T032-T033's implementation (fixes any mismatch found, does not add new scenarios)

  **Result**: All 9 pass with no code changes needed — every test passed on
  first run.

**Checkpoint**: User Story 2 is fully functional and testable independently at the domain/persistence layer — T023-T031 pass using only T002-T005, T032-T033. UI wiring continues below.

### UI for User Story 2

- [X] T035 [US2] Extend the save-as-blueprint dialog's `PendingBlueprintSave` state (`src/blueprint_library_view.rs`) with an in-progress interface list (name input plus a way to pick a boundary anchor/side on the selection preview), and thread it through `begin_save`/`cancel_save`/`confirm_save` into the new `Blueprint::from_selection` parameter (depends on T032, T035's own UI surface confirmed during implementation)
- [X] T036 [US2] Render each blueprint's interface names (no connection-state claim, per FR-010) in the **BLUEPRINT LIBRARY** sidebar row / insertion-selection view established by T019, in `src/bluepr...[truncated]

**Checkpoint**: All user stories are independently functional, end to end.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final verification before this feature's commit(s) land

- [X] T037 Confirm every scenario walkthrough in `quickstart.md` is covered by one of T006-T036, with no manual-only step left unautomated beyond what quickstart.md itself marks `(manual only)`

  **Result**: All 5 US1 ACs and all 4 US2 ACs, plus both mechanical
  checks, map to an existing test with a matching name (verified by name,
  not just count — every test name quickstart.md's "Automated:" prose
  references was found verbatim in `tests/domain_blueprint.rs` or
  `tests/blueprint_document_codec.rs`). The 3 steps quickstart.md itself
  marks `(manual only)` (visual canvas confirmation, notice-text wording,
  dialog interaction) remain manual, per this project's established
  preference for logical/deterministic validation over automated GUI
  capture — consistent with every prior phase.
- [X] T038 Confirm scope: `git status --short` and `git diff --stat` show changes only under `src/`, `tests/`, and `specs/004-blueprint-insertion-interfaces/` — nothing under `catalog/`, `data/`, `.hermes/`, or the historical `specs/001-blueprint-library/`, `specs/002-blueprint-library-ui/`, `specs/003-phase-4-closure/` directories

  **Result**: Confirmed. `git status --short` lists 9 modified files
  (`src/blueprint_library_view.rs`, `src/domain/blueprint.rs`,
  `src/egui_app.rs`, `src/egui_app_tests.rs`, `src/egui_canvas.rs`,
  `src/persistence/blueprint_document.rs`,
  `src/persistence/blueprint_library.rs`,
  `tests/blueprint_document_codec.rs`, `tests/blueprint_library.rs`,
  `tests/domain_blueprint.rs`) plus one untracked directory
  (`specs/004-blueprint-insertion-interfaces/`). Nothing under
  `catalog/`, `data/`, `.hermes/`, or any historical `specs/00N-*/`
  directory appears in either command's output.
- [X] T039 Run `cargo test` (confirm only new tests were added, nothing regressed), then all six mandatory gates in order — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300` (constitution Principle IV)

**Result**: 369 tests total (350 pre-existing + 19 new: 10 in
`tests/domain_blueprint.rs` for US1 insertion, 6 in
`tests/domain_blueprint.rs` for US2 interface validation, 3 in
`tests/blueprint_document_codec.rs` for US2 persistence), 0 failed, 0
regressed. All six gates passed: `cargo fmt --check` (exit 0),
`cargo clippy --all-targets --all-features -- -D warnings` (exit 0, one
`clippy::int_plus_one` lint found and fixed in
`FootprintBounds::contains_boundary_point` during implementation — see
plan.md deviation note), `cargo test` (exit 0, 369 passed), `cargo build
--release --bins` (exit 0, `Finished release profile [optimized]` in 1m
32s), `git diff --check` (exit 0, no whitespace errors), `hermes verify
--skip-start --json --timeout 300` (`"ok": true`, both `build` and `test`
phases green).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T005) completion
- **User Story 2 (Phase 4)**: Depends on Foundational (T002-T005) completion; does not depend on User Story 1's domain/persistence work (T016-T018), but its UI phase (T036) reuses the library-row rendering User Story 1's UI phase (T019) establishes
- **Polish (Phase 5)**: Depends on every task above

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on US2's domain/persistence layer; its UI phase does not depend on US2 either
- **User Story 2 (P2)**: No dependency on US1's domain/persistence layer (T016-T018); its UI phase (T036) depends on US1's T019 (the library row it renders interface names into)

### Within Each User Story

- Tests and implementation for a story are interleaved above in the order that keeps every test compilable and meaningful the moment it is written (spec-driven, not strict red-green-refactor — see the Tests note at the top of this file)
- Domain/persistence work before UI work within each story, since the UI phase calls the domain/persistence functions the earlier phase defines
- Story complete before moving to the next priority, per this project's established sequential-commit workflow

### Parallel Opportunities

- T006-T014 (User Story 1's domain tests) are marked `[P]` — independent test functions in the same file (`tests/domain_blueprint.rs`) with no dependency on each other, differing only in which scenario they construct; writing them is parallelizable even though this project's single-agent sequential-commit workflow (constitution "Workflow and Branching") means they are still executed one after another in practice, as `specs/001-blueprint-library/tasks.md` already established for this project
- T023-T031 (User Story 2's domain/codec tests) are marked `[P]` for the same reason, split across `tests/domain_blueprint.rs` (T023-T028) and `tests/blueprint_document_codec.rs` (T029-T031)
- T015 is not marked `[P]` — it depends on T006 existing as a template for constructing a successful insertion first
- No implementation task (T016-T022, T032-T036) is marked `[P]`: each modifies a file another task in the same story also modifies, or depends on the immediately preceding task's new signature

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T005) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T006-T022)
4. **STOP and VALIDATE**: run T006-T015 and confirm they pass in isolation; manually validate the UI scenarios in quickstart.md
5. This alone already delivers spec.md US1's "Why this priority": the blueprint library's entire reason to exist — a saved blueprint can go back into a factory

### Incremental Delivery

1. Setup + Foundational → foundation ready (T001-T005)
2. Add User Story 1 → validate independently → this is the MVP (T006-T022)
3. Add User Story 2 → validate independently → adds the named-interface capability on top of the same `Blueprint`/`BlueprintDocument` (T023-T036)
4. Polish (T037-T039) → all six gates green, commit(s)

---

## Notes

- **[Story] label** maps every task to its owning user story for traceability back to `spec.md`
- Tasks with no `[Story]` label (T001-T005, T037-T039) are Setup, Foundational, or Polish — not story-specific
- This tasks.md, together with `plan.md`, `research.md`, `data-model.md`, and `quickstart.md`, is the complete spec-driven design record for this feature; implementation should not need to re-derive any decision already made in those files
- Per this project's established workflow (`docs/adr/`, constitution "Workflow and Branching"), this is the first feature developed on its own branch rather than directly on `master` — commit narration, the six-gate verification, and the two-independent-reviewer gate happen at commit-preparation time, per story or per logical group, same process as every prior phase, just against a feature branch merged back to `master` once complete rather than committed directly to `master`
- Any deviation from this plan discovered during implementation must be narrated explicitly in the final report, per this project's established workflow (see `specs/001-blueprint-library/tasks.md`'s "Deviations from plan.md" section for the precedent this project follows)
