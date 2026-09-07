# Tasks: Blueprint Library Persistence

**Input**: Design documents from `/specs/001-blueprint-library/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included. Each user story's `spec.md` "Independent Test" field and `quickstart.md`'s scenario walkthroughs are an explicit request for integration-test coverage of every acceptance scenario. Per the project constitution (Principle III), this feature is spec-driven rather than test-first: the spec/plan/tasks below were written, reviewed, and approved before any implementation, so tests are written alongside their behavior rather than as a mandatory pre-implementation RED step — but `cargo test` and the other five gates (Principle IV) must still pass before this commit lands.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project (per plan.md's Structure Decision): `src/`, `tests/` at repository root. This feature touches exactly three files: `src/persistence/blueprint_library.rs` (new), `src/persistence/mod.rs` (one-line registration), `tests/blueprint_library.rs` (new).

## Phase 1: Setup

**Purpose**: Project initialization and basic structure

- [X] T001 Create `src/persistence/blueprint_library.rs` with a module-level doc comment describing its purpose (per plan.md Summary), and register it with `pub mod blueprint_library;` in `src/persistence/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and the shared `list()` behavior that every user story's tests exercise

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Define `BlueprintLibrary` (with the no-I/O `at(root: PathBuf)` constructor), `BlueprintLibraryEntry`, `InvalidLibraryEntryReason`, `InvalidLibraryEntry`, `BlueprintLibraryListing`, and `BlueprintLibrarySaveError` types exactly per `specs/001-blueprint-library/data-model.md` in `src/persistence/blueprint_library.rs` (depends on T001)
- [X] T003 Implement the `<blueprint_id>.factory-blueprint.json` filename convention parse/format helper (data-model.md "Filename convention" section: filenames matching `blueprint_<32-lowercase-hex>.factory-blueprint.json`, round-tripping through `BlueprintId::parse`) in `src/persistence/blueprint_library.rs` (depends on T002)
- [X] T004 Implement the core `list(&self, active_catalog: &Catalog) -> BlueprintLibraryListing` loop: `fs::read_dir` the root once, skip entries whose `file_type()` reports a symlink (FR-006), skip filenames that do not match T003's convention (FR-005), decode every remaining candidate via `decode_blueprint_document` (Commit 6), route decode failures to `invalid_entries` with `InvalidLibraryEntryReason::UnreadableOrMalformed` (FR-007), and sort successfully decoded entries by `name` ascending then `id` ascending (FR-004) in `src/persistence/blueprint_library.rs` (depends on T002, T003)

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Blueprints survive a restart (Priority: P1) 🎯 MVP

**Goal**: A blueprint saved in one run is found, with correct name/node-count/updated-at, by a fresh `BlueprintLibrary` instance in a later run — including when the storage location does not exist yet.

**Independent Test**: Save one valid blueprint to a temp root, drop the `BlueprintLibrary` value, construct a new one at the same root, call `list()`, and confirm the blueprint is discovered.

### Tests for User Story 1

- [X] T005 [US1] Integration test `saved_blueprint_is_discovered_by_a_new_library_instance` (spec.md US1/AC1) in `tests/blueprint_library.rs`
- [X] T006 [US1] Integration test `first_save_creates_the_storage_root_automatically` (spec.md US1/AC2) in `tests/blueprint_library.rs`

### Implementation for User Story 1

- [X] T007 [US1] Implement `BlueprintLibrary::default_for_user()`, resolving `%LOCALAPPDATA%/Factory Canvas/blueprints` via `std::env::var("LOCALAPPDATA")` and delegating to `at()` (research.md §2) in `src/persistence/blueprint_library.rs` (depends on T002)
- [X] T008 [US1] Implement `save(&self, blueprint: &Blueprint) -> Result<(), BlueprintLibrarySaveError>` happy path: encode via `encode_blueprint_document`, create the root directory if missing (`fs::create_dir_all`, satisfies FR-002), write via `atomic_file::write_atomically` at the T003-convention filename (FR-001) in `src/persistence/blueprint_library.rs` (depends on T002, T003, T007)
- [X] T009 [US1] Integration test `interrupted_or_failing_save_leaves_previously_stored_blueprint_untouched` (spec.md Edge Cases, FR-011, SC-005 — Windows: `icacls /deny` on the target directory for the current user, restored via an RAII guard even on panic; a plain `attrib +R` was tried first and confirmed, empirically, not to actually block writes on this OS) in `tests/blueprint_library.rs` (depends on T008)

**Checkpoint**: User Story 1 is fully functional and testable independently — T005, T006, T009 pass using only T001-T004, T007-T008.

---

## Phase 4: User Story 2 - One broken file never hides the rest of the library (Priority: P2)

**Goal**: One corrupted or unreadable file in storage never blocks, crashes, or hides the rest of an otherwise-valid library, and its warning never leaks anything private.

**Independent Test**: Place several valid blueprint files and one invalid file in a temp root, call `list()`, and confirm every valid blueprint is listed while the invalid one produces exactly one safe warning.

### Tests for User Story 2

- [X] T010 [US2] Integration test `invalid_file_is_isolated_while_valid_blueprints_remain_listed` (spec.md US2/AC1, SC-002) — save N (N >= 2) valid blueprints, write one additional file that fails `decode_blueprint_document`, assert `entries.len() == N` and `invalid_entries.len() == 1` with `reason == UnreadableOrMalformed`, in `tests/blueprint_library.rs`
- [X] T011 [US2] Test `invalid_entry_warning_never_leaks_path_content_or_identifier` (spec.md US2/AC2, FR-008, SC-003) — asserts the `Debug` output of every `InvalidLibraryEntry` produced in T010's scenario contains no substring of the temp directory's absolute path and no substring of the malformed file's raw bytes, in `tests/blueprint_library.rs` (depends on T010)

### Implementation for User Story 2

No new implementation task: `InvalidLibraryEntry` (T002) structurally carries no path/content/identifier field, and the classification loop (T004) already routes every decode failure through it. These two tests are pure verification that the Foundational design decision holds in practice — see `specs/001-blueprint-library/data-model.md` ("FR-008 holds by construction rather than by discipline").

**Checkpoint**: User Stories 1 AND 2 both work independently.

---

## Phase 5: User Story 3 - Unusual storage contents behave predictably (Priority: P3)

**Goal**: An unrelated file, a symlink, or two files claiming the same blueprint identity each produce a safe, predictable, non-crashing, non-duplicating result; a save-time ID collision is detected and avoided rather than silently overwriting an existing blueprint.

**Independent Test**: Add a non-blueprint file, a symbolic link, and two files that decode to the same blueprint identity into storage; list and confirm each is handled per spec.md US3.

### Implementation for User Story 3

- [X] T012 [US3] Implement duplicate-`BlueprintId` resolution in `list()`: after T004's decode-and-sort pass, group successfully decoded entries by `BlueprintId`, keep the first per the existing sort order as the valid `BlueprintLibraryEntry`, and route every other entry sharing that ID to `invalid_entries` with `InvalidLibraryEntryReason::DuplicateBlueprintId` (FR-009; research.md §6, data-model.md §6) in `src/persistence/blueprint_library.rs` (depends on T004) — implemented by sorting candidates on `id().as_str()` (a plain `&str`) rather than a `HashMap` keyed by `BlueprintId`, since `BlueprintId` derives neither `Hash` nor `Ord` and plan.md scopes `src/domain/blueprint.rs` as unmodified; see tasks.md Notes "Deviations from plan.md" below
- [X] T013 [US3] Implement save-time ID collision detection in `save()`: before writing, scan existing filenames in the target directory for the candidate `BlueprintId` (cheap filename check via T003's helper, not a full decode); on collision, regenerate via `BlueprintId::generate()` and retry up to a small fixed bound, returning `BlueprintLibrarySaveError::IdCollisionExhausted` if every attempt collides (FR-010; research.md §7) in `src/persistence/blueprint_library.rs` (depends on T008) — the retry re-encodes the document under the new ID via a new crate-private `encode_blueprint_document_as` in `blueprint_document.rs`, not just a new filename; see Deviations note below for why

### Tests for User Story 3

- [X] T014 [US3] Integration test `unrelated_file_in_storage_is_silently_ignored` (spec.md US3/AC1) — a file not matching the T003 filename convention produces neither an entry nor a warning, in `tests/blueprint_library.rs`
- [X] T015 [US3] Integration test `symlinked_blueprint_file_is_not_followed_or_listed` (spec.md US3/AC2, `#[cfg(windows)]` using `std::os::windows::fs::symlink_file`) — a symlink inside the root pointing at a valid blueprint elsewhere is skipped rather than followed, in `tests/blueprint_library.rs` — this test self-skips with a printed diagnostic (rather than failing the suite) when the executing Windows session lacks `SeCreateSymbolicLinkPrivilege` (no Developer Mode, not elevated); confirmed empirically absent in the implementation environment, so this scenario's actual pass/fail was not observed here — `list()`'s `file_type().is_symlink()` check (T004) is otherwise exercised by every other test's non-symlink files
- [X] T016 [US3] Integration test `duplicate_blueprint_identity_resolves_to_one_entry_and_one_warning` (spec.md US3/AC3) — two files decoding to the same `BlueprintId` yield exactly one `entries` item and exactly one `invalid_entries` item with `DuplicateBlueprintId`, in `tests/blueprint_library.rs` (depends on T012)
- [X] T017 [US3] Integration test `saving_a_colliding_blueprint_id_retries_with_a_new_identifier` (spec.md US3/AC4) — saving a second blueprint forced to collide with an already-stored ID still succeeds and yields two distinct entries on the next `list()`, in `tests/blueprint_library.rs` (depends on T013)

- [X] T019 [US1] Integration test `listing_orders_entries_alphabetically_by_name_with_id_tiebreak_and_is_stable_across_repeated_calls` (spec.md FR-004/SC-004) — added beyond the original T001-T018 breakdown in response to `/speckit-analyze` finding C1 (FR-004/SC-004 had implementation coverage via T004 but no verifying test); saves blueprints out of alphabetical order plus two sharing one name, calls `list()` twice, and asserts both the exact name+ID order and that repeated calls agree, in `tests/blueprint_library.rs`

**Checkpoint**: All three user stories are independently functional. Every acceptance scenario in `spec.md` has a corresponding passing test.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification before this commit lands

- [X] T018 Run `cargo test` (confirm only new tests were added, nothing regressed), then all six mandatory gates in order — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300` (constitution Principle IV; `specs/001-blueprint-library/quickstart.md` "Automated validation") — and confirm every scenario walkthrough in `quickstart.md` is covered by one of T005-T017, with no manual-only step left unautomated

  **Result**: 331 tests total (321 pre-existing + 10 new in `tests/blueprint_library.rs`), 0 failed, 0 regressed. `cargo fmt --check`, `cargo clippy -D warnings`, `cargo build --release --bins`, `git diff --check`, and `hermes verify --skip-start --json --timeout 300` (`"ok": true`) all passed.

## Deviations from plan.md discovered during implementation

Two small, additive corrections were needed once real compilation and the
US3 collision scenario were exercised — both narrated here rather than
silently folded in, per this project's workflow:

1. **T012 (duplicate-ID grouping)**: plan.md's data-model.md described
   grouping decoded entries "by `BlueprintId`" without specifying a
   mechanism. `BlueprintId` derives neither `Hash` nor `Ord` (confirmed by
   `cargo check` failing on `HashMap<BlueprintId, _>::entry`), and adding
   either derive would modify `src/domain/blueprint.rs`, which plan.md's
   Project Structure explicitly scopes as unmodified for this feature. The
   grouping is instead done by sorting candidates on `id().as_str()` (a
   plain `&str`, which is `Ord`) — same observable behavior (FR-009), no
   domain-layer change.
2. **T013 (save-time collision retry)**: writing the design walkthrough in
   quickstart.md's US3/AC4 surfaced a correctness gap the plan had not
   worked through: retrying a collision by choosing a new *filename* while
   leaving the encoded document's own `blueprint_id` field unchanged would
   make `list()` decode two files to the *same* content identity, which
   T012's own duplicate-resolution logic would then collapse back down to
   one entry — directly contradicting AC4's "two distinct entries"
   requirement. The fix is a small, additive, crate-private
   `encode_blueprint_document_as(blueprint, id)` in
   `src/persistence/blueprint_document.rs` (the existing
   `encode_blueprint_document` now delegates to it with `blueprint.id()`),
   so a collision retry re-persists the document under its actual new
   identity, not just a new filename. This is the one file plan.md listed
   as "EXISTING, unmodified" that was, in fact, touched — verified with
   `cargo test --test blueprint_document_codec` (unaffected, still 12/12)
   and the full suite (331/331) after the change.
3. **BlueprintLibrarySaveError no longer derives `Copy`**: plan.md/
   data-model.md did not specify derives for this type. `Copy` was
   attempted first (matching the project's general preference for small
   `Copy` error enums) but rejected by the compiler because `Encoding`
   wraps `BlueprintDocumentError`, which is not `Copy` (it owns a
   `DocumentMetadataError`, itself carrying no non-`Copy` field today but
   not marked `Copy` either). Downgraded to `Clone` only — no behavior
   change, purely a derive-list correction caught at compile time before
   any test ran.

### Independent Review (post-implementation, pre-commit)

Two independent reviewer subagents (no shared context with the implementer
or each other) reviewed the final staged diff against the exact commit
identity (`parent 7af8a926...`, `staged tree 7641336e...`), per this
project's two-independent-reviewer gate:

1. **Spec-compliance reviewer**: verified all 12 FRs and all 5 SCs
   individually against the actual code logic and test assertions (not
   just comment text), specifically re-tracing the save-time ID-collision
   retry, the content-declared-identity duplicate grouping, and the
   privacy guarantee's structural (not just behavioral) enforcement.
   Result: **12/12 FR met, 5/5 SC met, zero blocking findings.** Verified
   identity independently via `git rev-parse HEAD` and `git write-tree`,
   and ran the full test suite itself (331/331 passing) rather than
   trusting the implementer's self-report.
2. **Security/quality reviewer**: checked for hardcoded secrets, `unsafe`
   blocks, path traversal through crafted filenames, shell injection in
   the test-only `icacls` helper, TOCTOU in the collision-retry loop, and
   whether `save()`'s doc comment matches its implementation. Result:
   **zero security concerns, zero logic errors.** Independently confirmed
   zero `unsafe` blocks, traced `MAX_ID_COLLISION_RETRIES` termination
   (max 9 attempts, no underflow, every branch reachable), and assessed
   the abstract TOCTOU pattern in the collision-retry loop as real but not
   practically exploitable for this product's single-user, synchronous-UI
   desktop usage (documented as a non-blocking suggestion, not a defect).
   Also ran `cargo check`, the full test suite, and
   `clippy --all-targets -- -W clippy::all -W clippy::pedantic` itself on
   this machine.

Both reviews' non-blocking suggestions that were free to apply before
commit were applied: `for loser in ... { let _ = loser; ... }` simplified
to `for _ in ...`, one `{root:?}` in a test assertion message switched to
`root.display()`, and one `.map(|entry| entry.name())` switched to the
method-reference form `.map(BlueprintLibraryEntry::name)`. All three are
style-only with no behavior change, verified by re-running the full six
gates after applying them (fmt, clippy -D warnings, full test suite,
release build, `git diff --check`, `hermes verify`) — all green.

Suggestions deliberately left as documented, non-blocking notes rather
than acted on before commit: the TOCTOU pattern (see above), `save()` not
returning the caller-visible final `BlueprintId` after a collision rename
(a re-list already recovers it; a richer return type is a reasonable
future API improvement, not a correctness gap), and the disclosed,
already-documented (research.md) test-coverage gap on the collision
loop's multi-retry-then-succeed and exhaustion branches, infeasible to
test without making `BlueprintId::generate()` seedable.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T004) completion
- **User Story 2 (Phase 4)**: Depends on Foundational (T002-T004) completion; does not depend on User Story 1
- **User Story 3 (Phase 5)**: Depends on Foundational (T002-T004) for T012/T014/T015; depends on User Story 1's T008 for T013/T017
- **Polish (Phase 6)**: Depends on every task above

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on US2 or US3
- **User Story 2 (P2)**: No dependency on US1 or US3 beyond the shared Foundational phase
- **User Story 3 (P3)**: T012/T014/T015 depend only on Foundational; T013/T017 depend on US1's `save()` (T008) existing, since collision-avoidance is an addition to `save()`'s happy path rather than a standalone function

### Within Each User Story

- Tests and implementation for a story are interleaved above in the order that keeps every test compilable and meaningful the moment it is written (spec-driven, not strict red-green-refactor — see the Tests note at the top of this file)
- Story complete before moving to the next priority, per this project's established sequential-commit workflow

### Parallel Opportunities

None of the 18 tasks above are marked `[P]`. This feature is concentrated in exactly two files (`src/persistence/blueprint_library.rs`, `tests/blueprint_library.rs`) with a linear dependency chain inside each — genuinely independent, conflict-free file pairs are the exception here, not the rule (T014/T015 do not strictly need T012/T013 to exist first, for example, but marking a single task `[P]` inside an otherwise sequential file is misleading rather than useful). This project is also implemented by a single agent in sequential, narrated, atomic commits (constitution "Workflow and Branching"), not a multi-person team that would exploit file-level parallelism. Tasks are therefore listed in the exact order they should be executed.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T004) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T005-T009)
4. **STOP and VALIDATE**: run T005, T006, T009 and confirm they pass in isolation
5. This alone already delivers the feature's entire reason to exist (spec.md US1's "Why this priority"): a blueprint survives a restart

### Incremental Delivery

1. Setup + Foundational → foundation ready (T001-T004)
2. Add User Story 1 → validate independently → this is the MVP (T005-T009)
3. Add User Story 2 → validate independently → adds robustness guarantees on top of the same `list()` (T010-T011)
4. Add User Story 3 → validate independently → adds the remaining edge-case guarantees (T012-T017)
5. Polish (T018) → all six gates green, commit

---

## Notes

- **[Story] label** maps every task to its owning user story for traceability back to `spec.md`
- Tasks with no `[Story]` label (T001-T004, T018) are Setup, Foundational, or Polish — not story-specific
- No task is marked `[P]` — see "Parallel Opportunities" above for why
- This tasks.md, together with `plan.md`, `research.md`, `data-model.md`, and `quickstart.md`, is the complete spec-driven design record for this commit; implementation should not need to re-derive any decision already made in those files
- Commit narration, the six-gate verification, and this project's two-independent-reviewer gate for domain/persistence code (established across Commits 4-6, `requesting-code-review` skill) happen after T018, at commit-preparation time — they are process, not feature tasks, and are intentionally not numbered above
