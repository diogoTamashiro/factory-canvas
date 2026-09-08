---

description: "Task list template for feature implementation"
---

# Tasks: Blueprint Library UI

**Input**: Design documents from `/specs/002-blueprint-library-ui/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md
(no `contracts/` — see plan.md "Project Structure": this feature exposes no
external interface)

**Tests**: Not explicitly requested in spec.md as a distinct ask, but this
project's own established, memorized convention (research.md Decision 11;
`docs/roadmap.md` "Required gates" for UI changes; every prior commit's
`tasks.md`) is logical/deterministic tests for every mutating method and
pure helper, plus a non-blocking manual script for real visual/interaction
confirmation. Test tasks are included below on that basis, not as a
deviation from how this project already works.

**Organization**: Tasks are grouped by user story (spec.md P1/P2/P3) to
enable independent implementation and testing of each story, per this
project's established sequential-commit workflow (one commit for this
whole feature, stories delivered in-order within it — see Notes at the
bottom, matching `specs/001-blueprint-library/tasks.md`'s own approach).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Every task names its exact file path

## Path Conventions

Single Rust crate, binary-only feature (see plan.md "Project Structure"):
new module `src/blueprint_library_view.rs`, modified `src/egui_app.rs`,
`src/egui_app_tests.rs`, `src/egui_main.rs`. No `tests/*.rs` integration
test file — this feature's code is not reachable from the library target's
public surface (same reasoning `document_session.rs`/`selected_set.rs`
already establish; see data-model.md).

---

## Phase 1: Setup

**Purpose**: Register the new module so it compiles before anything is put in it

- [X] T001 Create `src/blueprint_library_view.rs` with a module-level doc
  comment describing its purpose (per plan.md Summary and data-model.md),
  and register it with `mod blueprint_library_view;` in `src/egui_main.rs`
  (alongside the existing `mod document_session;`, `mod egui_app;`,
  `mod egui_canvas;`, `mod selected_set;` — binary-only, not `pub`, matching
  every existing sibling module)

**Checkpoint**: `cargo check --bins` passes with an empty module

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `BlueprintLibraryView`/`PendingBlueprintSave` types and
their behavioral contract (data-model.md) — every user story's UI code
calls into these

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Define `PendingBlueprintSave` (`selected_ids: Vec<EntityId>`,
  `name_input: String`) and `BlueprintLibraryView` (`library: Option<BlueprintLibrary>`,
  `listing: BlueprintLibraryListing`, `pending_save: Option<PendingBlueprintSave>`)
  in `src/blueprint_library_view.rs`, importing `BlueprintLibrary`,
  `BlueprintLibraryListing`, `BlueprintLibrarySaveError` from
  `factory_canvas::persistence::blueprint_library` and `EntityId` from
  `factory_canvas::domain::layout` (data-model.md's two new types)
- [X] T003 Implement `BlueprintLibraryView::new() -> Self` (no I/O;
  disconnected: `library: None`, empty `listing`, `pending_save: None`) and
  `BlueprintLibraryView::with_library(library: BlueprintLibrary) -> Self`
  (no I/O; empty `listing`, no `pending_save`) in
  `src/blueprint_library_view.rs`, implementing `Default` for
  `BlueprintLibraryView` in terms of `new()` (data-model.md "Design
  correction" — these two must never call `default_for_user()` or touch the
  filesystem, since `from_startup_catalog` uses one of them and is shared by
  every test)
- [X] T004 Implement `BlueprintLibraryView::refresh(&mut self, catalog: &Catalog)`
  (no-op if `library` is `None`; otherwise overwrites `listing` with
  `library.list(catalog)`) in `src/blueprint_library_view.rs`
- [X] T005 Implement `BlueprintLibraryView::connect_to_default_storage(&mut self, catalog: &Catalog)`
  (calls `BlueprintLibrary::default_for_user()`; on `Some`, replaces `self`
  with `Self::with_library(library)` then calls `self.refresh(catalog)`; on
  `None`, leaves `self` unchanged) in `src/blueprint_library_view.rs` —
  the **only** call site of `default_for_user()` in this feature
  (data-model.md "Design correction")
- [X] T006 [P] Unit test `disconnected_view_has_no_library_and_an_empty_listing`
  (`BlueprintLibraryView::new()` and `Default::default()` both yield
  `library: None`-equivalent behavior and an empty `listing`, asserted via a
  public-enough accessor or `#[cfg(test)]`-visible field access from within
  the same module) in `src/blueprint_library_view.rs`
- [X] T007 [P] Unit test `with_library_starts_with_an_empty_cached_listing_until_refreshed`
  (construct with `BlueprintLibrary::at(tempfile::tempdir())`, assert
  `listing` is empty before any `refresh` call even if a blueprint already
  exists on disk at that path) in `src/blueprint_library_view.rs`
- [X] T008 [P] Unit test `refresh_populates_listing_from_an_already_saved_blueprint`
  (save one blueprint directly via `BlueprintLibrary::save` against a temp
  dir, construct a view with `with_library` over that same path, call
  `refresh`, assert the entry appears) in `src/blueprint_library_view.rs`

**Checkpoint**: `BlueprintLibraryView` compiles, is unit-tested in
isolation, and is provably never I/O-triggering except through
`connect_to_default_storage` — user story UI work can now begin

---

## Phase 3: User Story 1 - Save the current selection as a named blueprint (Priority: P1) 🎯 MVP

**Goal**: A player can select canvas instances, name them, and have a new
blueprint appear in local storage — without altering the canvas.

**Independent Test**: Select instances, trigger save, confirm a name;
verify the canvas is byte-for-byte unchanged and a new blueprint now exists
via `BlueprintLibraryView`'s own state (per spec.md US1's Independent Test).

### Tests for User Story 1

- [X] T009 [P] [US1] Unit test `confirm_save_rejects_a_blank_trimmed_name_without_calling_library_save`
  (build a `PendingBlueprintSave` with `name_input: "   "`, call
  `BlueprintLibraryView::confirm_save`, assert it returns `None` and the
  view's `listing`/underlying storage are untouched — the defence-in-depth
  check data-model.md requires independent of the UI's disabled button) in
  `src/blueprint_library_view.rs`
- [X] T010 [P] [US1] Unit test `confirm_save_with_no_pending_save_or_no_library_is_a_no_op`
  (two cases: `pending_save: None`; and `pending_save: Some(..)` with
  `library: None` — both return `None` and mutate nothing) in
  `src/blueprint_library_view.rs`
- [X] T011 [US1] Unit test `confirm_save_captures_the_selection_and_appears_in_the_next_refresh`
  (real temp-dir-backed `BlueprintLibrary`, a `FactoryLayout` with one
  placed instance, `begin_save` with that instance's ID, `confirm_save`
  with a valid trimmed name, assert `Some(Ok(()))`, assert the view's
  `listing` already reflects the new entry per Decision 2's automatic
  post-save refresh — no separate manual `refresh` call) in
  `src/blueprint_library_view.rs` (depends on T002-T005)
- [X] T012 [US1] Unit test `cancel_save_creates_nothing_and_clears_pending_state`
  (`begin_save`, then `cancel_save`, assert `pending_save` is `None` and the
  library's `listing`/underlying storage are untouched) in
  `src/blueprint_library_view.rs`
- [X] T013 [US1] Unit test `save_action_button_is_only_enabled_with_a_non_empty_selection`
  (drive `sidebar_ui` or the narrower helper it calls through
  `egui::Context::run_ui`, once with `self.selected` empty and once
  non-empty, assert the "Save as blueprint" control's presence/enabled
  state matches — mirrors the existing `document_shortcut_frame` test
  harness pattern) in `src/egui_app_tests.rs`
- [X] T014 [US1] Unit test `successful_blueprint_save_leaves_canvas_selection_and_next_entity_id_unchanged`
  (place instances, select them, drive the full save flow through
  `FactoryCanvasApp`'s new method from T017, assert `self.layout`,
  `self.selected`, and `self.next_entity_id` are all identical before and
  after — the FR-004/SC-004 guarantee) in `src/egui_app_tests.rs`

### Implementation for User Story 1

- [X] T015 [US1] Implement `BlueprintLibraryView::begin_save(&mut self, selected_ids: Vec<EntityId>)`
  (sets `pending_save` to a fresh `PendingBlueprintSave` with an empty
  `name_input` and the given frozen selection) in
  `src/blueprint_library_view.rs`
- [X] T016 [US1] Implement `BlueprintLibraryView::cancel_save(&mut self)`
  (clears `pending_save`, no other side effect) and
  `BlueprintLibraryView::confirm_save(&mut self, layout: &FactoryLayout, catalog: &Catalog, now: OffsetDateTime) -> Option<Result<(), BlueprintLibrarySaveError>>`
  (per data-model.md's full behavioral contract: `None` if nothing pending
  or no library; blank-name re-check; `Blueprint::from_selection` with a
  fresh `BlueprintId::generate()` and a `DocumentMetadata` built from the
  trimmed name and `now`; `library.save`; on `Ok` clear `pending_save` and
  `refresh`; on `Err` clear `pending_save` only) in
  `src/blueprint_library_view.rs` (depends on T003-T005, T015)
- [X] T017 [US1] Add `blueprint_library: BlueprintLibraryView` field to
  `FactoryCanvasApp` (`src/egui_app.rs`, alongside the existing `session:
  DocumentSession` field), initialize it via `BlueprintLibraryView::new()`
  in `from_startup_catalog`, and call
  `self.blueprint_library.connect_to_default_storage(self.layout.catalog())`
  exactly once at the end of the real `FactoryCanvasApp::new(creation_context)`
  constructor only (never in `from_startup_catalog` itself, per data-model.md's
  "Design correction") (depends on T003, T005)
- [X] T018 [US1] Add two new `EditorNotice` variants, `BlueprintSaved` and
  `BlueprintSaveFailed(BlueprintLibrarySaveError)`, to the `EditorNotice`
  enum in `src/egui_app.rs` (siblings of the existing
  `DocumentSaved`/`DocumentSaveFailed(FactoryDocumentError)` pair, per
  research.md Decision 7), and a private
  `fn safe_blueprint_save_error_detail(error: &BlueprintLibrarySaveError) -> &'static str`
  free function mapping every variant (`Io`, `Encoding`, `IdCollisionExhausted`)
  to a fixed, generic message with no interpolation of `error`'s internals
  (research.md Decision 8) — wire both into `notice_text`/`notice_color`'s
  existing `match` arms in `src/egui_app.rs`
- [X] T019 [US1] Add a `FactoryCanvasApp` method (e.g.
  `request_save_as_blueprint(&mut self)`) that calls
  `self.blueprint_library.begin_save(self.selected.iter().collect())` only
  when `!self.selected.is_empty()` in `src/egui_app.rs` (FR-003's
  enforcement point, independent of whether the button itself is also
  disabled)
- [X] T020 [US1] Render the "Save as blueprint" button inside
  `editor_state_ui`'s existing `if selection_count > 0 { ... }` block in
  `src/egui_app.rs`, positioned after "Rotate 90° (R)" and before "Remove
  block(s)" (research.md Decision 3), wired to call the T019 method
- [X] T021 [US1] Implement `save_as_blueprint_modal(&mut self, context: &egui::Context, now: time::OffsetDateTime)`
  in `src/egui_app.rs`, reusing the exact `egui::Modal` frame/button styling
  of `instance_removal_modal`/`base_change_modal` (research.md Decision 5):
  a heading, one `ui.text_edit_singleline(&mut pending.name_input)`, and a
  Cancel/Confirm button row where Confirm is
  `ui.add_enabled(!pending.name_input.trim().is_empty(), ...)` — Cancel
  calls `self.blueprint_library.cancel_save()`, Confirm calls
  `self.blueprint_library.confirm_save(&self.layout, self.layout.catalog(), now)`
  and sets `self.notice` to `BlueprintSaved`/`BlueprintSaveFailed` based on
  the returned `Option<Result<...>>` (`None` is treated as no-op, matching
  how the modal would simply not have been open); also update
  `destructive_modal_open` if this modal should block other canvas edits
  while open (spec Assumption #3) — check whether existing non-destructive
  modals already gate this, otherwise add
  `self.blueprint_library.pending_save.is_some()` to that predicate
- [X] T022 [US1] Call `self.save_as_blueprint_modal(ui.ctx(), time::OffsetDateTime::now_utc())`
  from `ui_with_dialogs` in `src/egui_app.rs` (alongside the existing
  `self.base_change_modal(...)`, `self.instance_removal_modal(...)`,
  `self.unsaved_changes_modal(...)` calls)

**Checkpoint**: User Story 1 is fully functional and independently
testable — a player can save a named blueprint from a selection, and the
canvas is provably unchanged by it

---

## Phase 4: User Story 2 - See what is already saved in the library (Priority: P2)

**Goal**: The sidebar shows every valid saved blueprint's name, module
count, and last-saved time, including the one User Story 1 just created,
with no manual refresh step.

**Independent Test**: With one or more blueprints already saved (via US1
or a prior session), open the library view and confirm name/module
count/last-saved time are visible and correct (per spec.md US2's
Independent Test).

### Tests for User Story 2

- [X] T023 [P] [US2] Unit test `empty_library_listing_renders_an_explicit_no_blueprints_indication`
  (a `BlueprintLibraryView` with an empty `listing`, drive the new sidebar
  section, assert the rendered text matches a fixed "no blueprints yet"
  string — FR-007/SC-006) in `src/egui_app_tests.rs`
- [X] T024 [US2] Unit test `populated_library_listing_renders_name_module_count_and_last_saved_time_per_entry`
  (a `BlueprintLibraryView::with_library` over a temp dir with two saved
  blueprints of different node counts, `refresh`, drive the sidebar
  section, assert both entries' name/count/time are present in the
  rendered text) in `src/egui_app_tests.rs` (depends on T004)

### Implementation for User Story 2

- [X] T025 [US2] Add a `blueprint_library_section_ui(&mut self, ui: &mut Ui)`
  method in `src/egui_app.rs` rendering a "BLUEPRINT LIBRARY" header
  (matching the existing section-header style used by "CONSTRUCTION BASE"/
  "BLOCKS"/"EDITOR STATUS"), then either the fixed "No blueprints saved
  yet." label (if `self.blueprint_library.listing.entries` is empty) or one
  row per `BlueprintLibraryEntry` showing `name()`, a
  `"{node_count} modules"`-style label, and a formatted `updated_at()` —
  reuse the existing `time`-formatting convention already used elsewhere in
  this file for document metadata, if one exists, otherwise a minimal
  `OffsetDateTime::format` call with a fixed, already-imported
  `time::format_description`
- [X] T026 [US2] Call `self.blueprint_library_section_ui(ui)` from
  `sidebar_ui` in `src/egui_app.rs`, after the existing `self.editor_state_ui(ui)`
  call (research.md Decision 4) — no new `ScrollArea` needed, this section
  lives inside the sidebar's existing single shared scroll area

**Checkpoint**: User Stories 1 AND 2 both work independently — saving and
viewing the library are both fully functional together

---

## Phase 5: User Story 3 - Unreadable library entries are visible as safe warnings, not silent gaps (Priority: P3)

**Goal**: When the library reports invalid/duplicate entries or a
compatibility mismatch, the player sees a safe, generic, non-leaking
indication — never silence, never a technical detail.

**Independent Test**: Place valid blueprints and at least one invalid file
directly into storage (bypassing the UI), open the library view, confirm
valid entries list normally and a safe generic notice reflects the
unreadable one (per spec.md US3's Independent Test).

### Tests for User Story 3

- [X] T027 [P] [US3] Unit test `invalid_library_entries_render_a_safe_generic_notice_with_no_leaked_detail`
  (a `BlueprintLibraryListing` with a non-empty `invalid_entries` built
  directly — no need to go through real file corruption, `InvalidLibraryEntry`
  values can be constructed directly for this UI-layer test — drive the
  sidebar section, assert the rendered text contains a generic count-based
  notice and does not contain any path-like or hex-identifier-like
  substring) in `src/egui_app_tests.rs`
- [X] T028 [US3] Unit test `catalog_compatibility_mismatch_is_shown_but_the_entry_still_lists`
  (a `BlueprintLibraryEntry` with `compatibility()` returning
  `CatalogCompatibility::DataVersionMismatch` — construct via a real saved
  blueprint decoded against a catalog with a different `data_version`,
  mirroring how `DocumentOpened(CatalogCompatibility::...)` is already
  exercised for factory documents — assert the entry still renders and a
  visible, non-blocking mismatch indication is present) in
  `src/egui_app_tests.rs`

### Implementation for User Story 3

- [X] T029 [US3] Extend `blueprint_library_section_ui` (T025) in
  `src/egui_app.rs` to render one safe, generic line reflecting
  `self.blueprint_library.listing.invalid_entries.len()` when non-zero
  (e.g. `"{n} entries could not be read."`), using only the count — never
  iterating `InvalidLibraryEntryReason` into anything more specific than
  that count, per FR-008/FR-009/SC-005
- [X] T030 [US3] Extend each entry row in `blueprint_library_section_ui`
  (T025) in `src/egui_app.rs` to show a short, visible, non-blocking
  indication when `entry.compatibility()` is not
  `CatalogCompatibility::Exact` — reuse the same three-variant match
  already established by `notice_text`'s
  `DocumentOpened(CatalogCompatibility::...)` arms (`src/egui_app.rs:285-294`)
  for message wording, applied per-entry instead of as a one-shot notice

**Checkpoint**: All three user stories are independently functional — this
feature is complete

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification before this commit lands

- [X] T031 Run `cargo test` (confirm only new tests were added, nothing
  regressed from the 331 tests established through Commit 7), then all six
  gates in order: `cargo fmt --check`, `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo test`, `cargo build --release
  --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout
  300` (Constitution Principle IV)
- [X] T032 Walk through every scenario in `specs/002-blueprint-library-ui/quickstart.md`
  manually against a real debug build (`cargo run --bin factory-canvas`),
  per this project's established manual-test-script practice for UI changes
  (`docs/roadmap.md`; user's stated preference for logical validation over
  automated GUI capture, with manual confirmation for real visual/interaction
  behavior) — does not block gates or publication, produces a short report
  of what was actually seen for Diogo
- [X] T033 Close the Decision 9 traceability gap found during final
  validation: expose `BlueprintLibraryView::is_connected`, disable and relabel
  the selection-scoped save control when storage is unavailable, render the
  fixed persistent `"Blueprint library unavailable."` sidebar notice, guard
  `request_save_as_blueprint`, and add deterministic regression coverage in
  `src/blueprint_library_view.rs`, `src/egui_app.rs`, and
  `src/egui_app_tests.rs`
- [X] T034 Add an exact privacy-regression test for every
  `BlueprintLibrarySaveError` variant rendered through
  `EditorNotice::BlueprintSaveFailed`, and correct the mapping's requirement
  reference from FR-008 to FR-011 in `src/egui_app.rs` and
  `src/egui_app_tests.rs`
- [X] T035 Make the timestamp formatter enforce its own `UTC` suffix
  invariant by normalizing any `OffsetDateTime` input, even though current
  `DocumentMetadata` already normalizes stored values, and add a direct
  non-UTC regression case in `src/egui_app.rs` and
  `src/egui_app_tests.rs`
- [X] T036 Replace the ineffective failed-save test with a deterministic
  real I/O failure (a library root nested below a regular file), asserting
  `confirm_save` returns `BlueprintLibrarySaveError::Io`, closes pending
  state, and preserves the previously cached listing in
  `src/blueprint_library_view.rs`
- [X] T037 Correct the quickstart and UI-test references to Commit 7's
  actual persistence-test path, `tests/blueprint_library.rs`, in
  `specs/002-blueprint-library-ui/quickstart.md` and
  `src/egui_app_tests.rs`
- [X] T038 Extend the legacy "any destructive modal" regression matrix with
  the real blueprint save modal, proving `destructive_modal_open` recognizes
  it and document commands neither call native dialogs nor mutate pending
  blueprint state in `src/egui_app_tests.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T005) completion
- **User Story 2 (Phase 4)**: Depends on Foundational (T002-T005); reads the
  same `BlueprintLibraryView.listing` User Story 1 populates, but is
  independently testable by constructing a view with pre-existing saved
  blueprints (T023-T024 do not require US1's UI code to exist first)
- **User Story 3 (Phase 5)**: Depends on Foundational (T002-T005) and on
  T025 (the sidebar section US2 creates, which US3 extends rather than
  duplicates) — genuinely not independent of US2's implementation task,
  though it is independent of US2's *tests*
- **Polish (Phase 6)**: Depends on every task above

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on US2 or US3
- **User Story 2 (P2)**: No dependency on US1's UI tasks (T017-T022), only
  on Foundational — but delivering it without US1 already merged would mean
  nothing has ever saved a blueprint through the UI yet, so it is only
  demonstrable end-to-end after US1
- **User Story 3 (P3)**: T027 is independent (builds its own
  `BlueprintLibraryListing` directly); T029-T030 structurally extend T025
  (US2's implementation), so US3's *implementation* tasks depend on US2's
  implementation task, not just Foundational

### Within Each User Story

- Tests before implementation within each phase (written first, expected to
  fail, per this project's TDD-derived convention — see spec-kit template
  Notes; this project's Constitution Principle III means the *spec* comes
  before implementation at the feature level, it does not remove
  test-before-code discipline at the task level, per Constitution Principle
  III's own text: "it does not remove testing")
- Within Phase 3: T009-T014 (tests) before T015-T022 (implementation);
  T015-T016 (view-layer methods) before T017 (wiring into `FactoryCanvasApp`)
  before T018-T022 (UI rendering/wiring, which needs both)

### Parallel Opportunities

- T006, T007, T008 (Foundational tests) can run in parallel — different
  test functions in the same new file, no shared mutable state
- T009, T010 (US1 tests) can run in parallel with each other; T011-T014 are
  sequential relative to each other only because they build on the same
  growing understanding of the flow, not because of a real file conflict —
  mark them non-`[P]` anyway per this project's established practice
  (`specs/001-blueprint-library/tasks.md` also declined `[P]` throughout,
  reasoning: "This project is also implemented by a single agent in
  sequential, narrated, atomic commits ... not a multi-person team that
  would exploit file-level parallelism")
- T023 is the only genuinely parallel US2 task (US3's T027 is also
  parallel-marked for the same single-function-independence reason)

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T008) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T009-T022)
4. **STOP and VALIDATE**: run the Phase 3 tests and confirm they pass in
   isolation; walk through quickstart.md's US1 scenarios manually
5. This alone already delivers spec.md US1's "Why this priority": a player
   can actually use the already-shipped Commit 5/7 machinery for the first
   time

### Incremental Delivery

1. Setup + Foundational → foundation ready (T001-T008)
2. Add User Story 1 → validate independently → this is the MVP (T009-T022)
3. Add User Story 2 → validate independently → the library becomes visible,
   not just writable (T023-T026)
4. Add User Story 3 → validate independently → the library's existing
   safety guarantees (Commit 7) become visible, not just internally true
   (T027-T030)
5. Polish (T031-T038) → all six gates green, manual script run, final
   design-record reconciliation, commit

---

## Notes

- **[Story] label** maps every task to its owning user story for
  traceability back to spec.md
- Tasks with no `[Story]` label (T001-T008, T031-T038) are Setup,
  Foundational, or Polish — not story-specific
- Almost no task is marked `[P]` — see "Parallel Opportunities" above for
  why, consistent with `specs/001-blueprint-library/tasks.md`'s own
  reasoning for the same choice
- This `tasks.md`, together with `plan.md`, `research.md`, `data-model.md`,
  and `quickstart.md`, is the complete spec-driven design record for this
  commit; implementation should not need to re-derive any decision already
  made in those files
- Commit narration, the six-gate verification, and this project's
  two-independent-reviewer gate for domain/persistence-adjacent code
  (established across Commits 4-7, `requesting-code-review` skill) happen
  after T038, at commit-preparation time — they are process, not feature
  tasks, and are intentionally not numbered above
- One design correction was already made and folded directly into
  data-model.md and this file during task generation itself (the
  `BlueprintLibraryView` constructor shape — see data-model.md's "Design
  correction" callout) rather than deferred to a "Deviations" section
  after the fact, since it was caught before any implementation code
  existed to deviate from

## Implementation Record

- T032 was completed manually by Diogo against the real debug build; all
  quickstart scenarios were approved before commit preparation.
- Final plan-to-code reconciliation found that research.md Decision 9's
  explicit unavailable-storage UI had not been represented by the original
  T001-T032 breakdown. T033 was added and completed before review: the final
  code shows a persistent safe notice, disables/relabels the save control,
  and prevents programmatic dialog opening while disconnected.
- An intermediate implementation cleared `pending_save` when a disconnected
  view received `confirm_save`. That was superseded before review because
  T010/data-model.md require this branch to be a true no-op; the final UI
  prevents the attempt at its actual entry point instead.
- T024's first implementation draft exercised one populated entry. It was
  strengthened before review to the task's exact contract: two saved
  blueprints with different module counts and timestamps, both asserted in
  the rendered sidebar.
- T027 uses a real malformed file in an isolated temp directory rather than
  directly constructing `InvalidLibraryEntry`: that type intentionally has
  private fields and no public test constructor. This keeps Commit 7's
  persistence API unchanged and gives the UI test stronger end-to-end
  evidence without touching real user storage.
- T034 was added during privacy review because T018 implemented fixed error
  mapping but the original task list did not require per-variant evidence.
  The new test asserts exact safe text for all three save-error variants and
  rejects their internal enum/debug names.
- T035 was added during pre-review self-audit as local invariant hardening.
  `DocumentMetadata::new` already normalizes current stored timestamps to
  UTC, but the formatter itself previously relied on that upstream fact
  while appending a literal `UTC` suffix. It now enforces the suffix locally,
  and a direct non-UTC regression case prevents future misuse.
- T036 replaces an ineffective first-draft test that compared an unchanged
  cached listing without actually entering `confirm_save`'s error branch.
  The final test forces a real, portable I/O failure using a regular file as
  an impossible parent directory and proves the full FR-011 state contract.
- T038 was added because the first independent spec-compliance reviewer
  correctly flagged that the pre-existing
  `document_commands_are_blocked_while_any_destructive_modal_is_open`
  regression test (predates this feature) enumerated the base-change,
  removal, and unsaved-action modals but never the new blueprint save
  modal. The test now also opens a real `BlueprintLibraryView` save dialog,
  attempts `DocumentCommand::Open`, and asserts the native dialog is never
  invoked and the pending save is byte-for-byte unchanged.
