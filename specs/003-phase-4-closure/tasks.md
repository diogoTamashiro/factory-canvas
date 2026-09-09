---

description: "Task list template for feature implementation"
---

# Tasks: Phase 4 Closure Documentation

**Input**: Design documents from `/specs/003-phase-4-closure/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md
(no `contracts/` — see plan.md "Project Structure": this feature exposes no
external interface; it edits static documentation only)

**Tests**: Not applicable in the traditional (compiled/executed) sense —
this feature ships no Rust source (plan.md Technical Context). The
project's established SDD verification discipline (Principle III) still
applies, expressed here as the mechanical and read-through checks
`quickstart.md` already defines: two `grep` commands for the exact stale
vocabulary this closure must eliminate (SC-002), a cross-reference
existence check (SC-003), a cross-file consistency read (SC-004), and a
`git status`/`git diff --stat` scope check (SC-005). These are included
below as verification tasks per story and in Polish, not skipped.

**Organization**: Tasks are grouped by user story (spec.md P1/P2/P3) to
enable independent verification of each story, per this project's
established sequential-commit workflow (one commit for this whole feature,
stories delivered in-order within it — matching
`specs/001-blueprint-library/tasks.md` and
`specs/002-blueprint-library-ui/tasks.md`'s own approach).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Every task names its exact file path

## Path Conventions

Documentation-only feature (see plan.md "Project Structure"): six existing
tracked files are edited in place — `docs/roadmap.md`, `README.md`,
`CONTEXT.md`, `docs/architecture.md`, `docs/data-model.md`,
`docs/adr/0003-cad-documents-and-blueprints.md`. No file under `src/`,
`tests/`, `catalog/`, `data/`, `.hermes/`, or the historical
`specs/001-blueprint-library/`/`specs/002-blueprint-library-ui/`
directories is touched.

---

## Phase 1: Setup

**Purpose**: Confirm the exact six-file scope and each file's current
stale passage still match `data-model.md`'s `DocumentTarget` table before
any edit begins, so this closure corrects exactly what was actually found,
not a stale memory of it.

- [X] T001 Re-read `docs/roadmap.md`, `README.md`, `CONTEXT.md`,
  `docs/architecture.md`, `docs/data-model.md`, and
  `docs/adr/0003-cad-documents-and-blueprints.md`, and confirm each file's
  `stale_passage` location listed in
  `specs/003-phase-4-closure/data-model.md`'s `DocumentTarget` table is
  still present verbatim (no other commit touched these files since
  research.md/data-model.md were written)

**Checkpoint**: The six-file worklist is confirmed current

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish a clean starting baseline so the final scope check
(SC-005) is meaningful, and confirm no other pending change would be
mistaken for part of this closure

**⚠️ CRITICAL**: No user story editing can begin until this phase is complete

- [X] T002 Run `git status --short` and `git diff --stat` at the repository
  root and confirm the working tree shows no pending changes outside this
  feature's own `specs/003-phase-4-closure/` SDD trail (spec.md, plan.md,
  research.md, data-model.md, quickstart.md, tasks.md, checklists/) —
  establishes the "before" state the Polish phase's scope check (T020)
  compares against

**Checkpoint**: Baseline confirmed clean — user story edits can now begin

---

## Phase 3: User Story 1 - Confirm Phase 4 is documented as complete (Priority: P1) 🎯 MVP

**Goal**: `docs/roadmap.md`, read alone with no other context, correctly
states Phase 4 is fully delivered and identifies Phase 5 as next.

**Independent Test**: Read `docs/roadmap.md` top to bottom and correctly
state which capabilities Phase 4 delivered and that Phase 5 is next
(spec.md US1 Independent Test).

### Implementation for User Story 1

- [X] T003 [US1] Add an integrated "## Phase 4 — JSON documents and
  blueprint library — integrated" section to `docs/roadmap.md`, positioned
  immediately after the existing "## Phase 3 — data package and per-entity
  product — integrated" section, in the same format and level of detail,
  summarizing the capability actually delivered by Commits 1-8: versioned
  `FactoryDocument` save/open through native file dialogs, a transactional
  document session, `Blueprint` capture from a canvas selection, the
  `BlueprintDocument` v1 codec, atomic same-directory writes, the
  persistent per-user `BlueprintLibrary`, and the save-as-blueprint /
  browse-the-library editor UI (satisfies spec.md FR-001's first clause)
- [X] T004 [US1] Update `docs/roadmap.md`'s "## Next active slice and
  remaining MVP phases" section to remove Phase 4 as upcoming and name
  Phase 5 (independent insertion and exposed interfaces) as the sole next
  active slice (satisfies spec.md FR-001's second clause; depends on T003
  — same file, sequential edit)
- [X] T005 [US1] Verify: read `docs/roadmap.md` top to bottom per
  `specs/003-phase-4-closure/quickstart.md`'s "Scenario: `docs/roadmap.md`
  reads as complete on its own" and confirm Phase 4 reads as integrated and
  Phase 5 as the only remaining next slice (depends on T003, T004)

**Checkpoint**: User Story 1 is independently verifiable — a maintainer
reading only `docs/roadmap.md` now gets an accurate picture of Phase 4 and
Phase 5.

---

## Phase 4: User Story 2 - Confirm the product-facing description matches shipped capability (Priority: P2)

**Goal**: `README.md` and `CONTEXT.md` both accurately describe that the
editor already supports saving a named local blueprint and browsing the
local blueprint library, and both point to Phase 5 as next.

**Independent Test**: Read `README.md`'s status line and `CONTEXT.md`'s
"Roadmap and next implementation" section and confirm both state Phase 4
is integrated with blueprint save/browse available, and both point to
Phase 5 as next (spec.md US2 Independent Test).

### Implementation for User Story 2

- [X] T006 [P] [US2] Update `README.md`'s "Current status" line (top of
  file, current text ending "...The next MVP work is versioned factory
  documents and local blueprints...") to state Phase 4 is integrated: the
  editor supports saving the current canvas selection as a named local
  blueprint and browsing the local blueprint library; next MVP work is
  Phase 5 (spec.md FR-002)
- [X] T007 [P] [US2] Update `CONTEXT.md`'s "## Roadmap and next
  implementation" section (current text: "Phase 4 is next: versioned
  FactoryDocument and BlueprintDocument persistence with atomic local
  saves.") to state Phase 4 is integrated and identify Phase 5 as next
  (spec.md FR-003)
- [X] T008 [US2] Verify: read `README.md` and `CONTEXT.md` per
  `specs/003-phase-4-closure/quickstart.md`'s "Scenario: `README.md` and
  `CONTEXT.md` describe current capability accurately" and confirm neither
  describes `FactoryDocument`/`BlueprintDocument` persistence as still
  upcoming (depends on T006, T007)

**Checkpoint**: User Stories 1 AND 2 are both independently verifiable —
a newcomer's first two reads (README, CONTEXT) now match shipped capability.

---

## Phase 5: User Story 3 - Confirm architecture and data-contract documents have no drift (Priority: P3)

**Goal**: `docs/architecture.md`, `docs/data-model.md`, and ADR 0003 no
longer describe `FactoryDocument`, `BlueprintDocument`, or the blueprint
library as future work, while blueprint insertion and physical-port
interfaces remain correctly marked as planned for Phase 5.

**Independent Test**: Cross-check every passage in these three documents
that currently mentions `FactoryDocument`, `BlueprintDocument`, or
blueprint persistence as "planned"/"next," and confirm each now correctly
reflects delivered-vs-deferred status (spec.md US3 Independent Test).

### Implementation for User Story 3

- [X] T009 [P] [US3] Update `docs/architecture.md`'s "## CAD, documents,
  and runtime data" intro paragraph (current text: "Factory and blueprint
  documents remain the next layers") to state that `FactoryDocument`,
  `BlueprintDocument`, blueprint capture, and their persistence are
  implemented, while physical ports remain a planned extension (spec.md
  FR-004, first clause)
- [X] T010 [US3] Update `docs/architecture.md`'s "## Canvas" → "Next
  increments" bullet list (current text: "Phase 4 adds versioned factory
  documents, blueprint documents, migrations, and atomic local saves") to
  remove that bullet and instead name Phase 5's remaining scope (blueprint
  insertion, physical-port interfaces) (spec.md FR-004, second clause;
  depends on T009 — same file, sequential edit)
- [X] T011 [P] [US3] Update `docs/data-model.md`'s status banner (top of
  file, line 3: "...Physical ports, FactoryDocument, BlueprintDocument,
  migrations, saves, and blueprint insertion remain contracts for later
  phases.") to state `FactoryDocument`, `BlueprintDocument`, and their
  persistence are implemented, while physical ports and blueprint
  insertion remain planned (spec.md FR-005, first clause)
- [X] T012 [US3] Update `docs/data-model.md`'s "## Planned factory
  document" section heading and body to reflect that `FactoryDocument` is
  implemented (spec.md FR-005, second clause; depends on T011 — same file,
  sequential edit)
- [X] T013 [US3] Update `docs/data-model.md`'s "## Planned blueprint
  document" section heading and body to reflect that `BlueprintDocument`
  is implemented (spec.md FR-005, second clause; depends on T012 — same
  file, sequential edit)
- [X] T014 [US3] Update `docs/data-model.md`'s "## Planned persistence"
  section heading and body to reflect that factory/blueprint persistence
  is implemented, while noting migration beyond schema v1 remains future
  work (spec.md FR-005, second clause; depends on T013 — same file,
  sequential edit)
- [X] T015 [P] [US3] Append a new dated "## Phase 4 implementation note —
  2026-09-07" section to
  `docs/adr/0003-cad-documents-and-blueprints.md`, positioned immediately
  after the existing "## Phase 3 implementation note — 2026-08-29" section
  and before "## Consequences," recording what Commits 1-8 actually
  implemented (versioned document codecs, atomic saves, blueprint capture,
  the persistent local library, and its editor UI) and explicitly
  confirming that blueprint insertion and physical-port interfaces — both
  named in the original Decision — remain deferred to Phase 5; the ADR's
  "Status" and original "Decision" text are left unchanged (spec.md FR-006;
  spec.md Edge Cases — ADRs are historical decision records)
- [X] T016 [US3] Verify: read `docs/architecture.md`, `docs/data-model.md`,
  and `docs/adr/0003-cad-documents-and-blueprints.md` per
  `specs/003-phase-4-closure/quickstart.md`'s "Scenario: Architecture and
  data-contract documents show no drift" and confirm physical ports,
  blueprint insertion, and undo/redo remain correctly described as still
  planned (depends on T009, T010, T011, T012, T013, T014, T015)

**Checkpoint**: All three user stories are independently verifiable —
every tracked document a maintainer might consult now agrees Phase 4 is
integrated and Phase 5 is next.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verify the six files are mutually consistent as a whole, that
this closure touched nothing outside its declared scope, and that all six
required gates still pass, before this commit is considered done.

- [X] T017 Run `specs/003-phase-4-closure/quickstart.md`'s two mechanical
  `grep` commands for the exact stale vocabulary
  (`factorydocument`/`blueprintdocument`/`blueprint library` co-occurring
  with `planned`/`next`/`remain`) across all six files and confirm zero
  matches (spec.md SC-002; depends on T005, T008, T016)

  **Result**: Second grep (status word before term) — zero matches. First
  grep (term before status word) surfaced two hits, both confirmed
  false positives from textual proximity, not actual staleness — the
  exact case `quickstart.md` itself anticipated:
  - `README.md`: "...blueprint library from the sidebar. The **next** MVP
    work is independent blueprint **insertion**..." — "next" describes
    Phase 5 (blueprint insertion), not the blueprint library, which the
    same sentence already calls available.
  - `docs/data-model.md`: "...blueprint library) **are implemented**.
    Physical ports and blueprint insertion **remain** contracts for Phase
    5." — the subject of "remain" is "blueprint insertion" (correctly
    still planned), not "blueprint library" (correctly called
    implemented one clause earlier).
  No genuine SC-002 violation in either file.
- [X] T018 Run `specs/003-phase-4-closure/quickstart.md`'s cross-reference
  check and confirm every path or document link touched while editing the
  six files still resolves to a file that exists (spec.md SC-003; depends
  on T005, T008, T016)

  **Result**: Every relative link inside the six edited files resolves —
  `docs/data-model.md` (incl. `#updating-a-package` anchor),
  `docs/architecture.md`, `docs/adr/0003-cad-documents-and-blueprints.md`,
  `docs/product-scope.md`, `docs/engineering-standards.md`,
  `CONTRIBUTING.md`, `docs/adr/0001-editor-ui.md`,
  `docs/adr/0002-product-name-factory-canvas.md` all confirmed present.
  No new cross-reference was added; none was broken.
- [X] T019 Cross-check every `PhaseStatusClaim` row in
  `specs/003-phase-4-closure/data-model.md` against the six files' final
  content and confirm no two files contradict each other about what Phase
  4 delivered or what remains deferred to Phase 5 (spec.md SC-004; depends
  on T005, T008, T016)

  **Result**: All six files' final content cross-checked directly against
  each other. `docs/roadmap.md` has an integrated "Phase 4" section and
  names only Phase 5 as next; `README.md` and `CONTEXT.md` both state
  "Phase 4 is integrated" and identify Phase 5 as next; `docs/architecture.md`
  describes documents/blueprints as implemented and its "Next increments"
  names only Phase 5; `docs/data-model.md` renamed all three sections from
  "Planned" to "Implemented factory document"/"Implemented blueprint
  document"/"Implemented persistence" while "Planned physical ports"
  correctly remains planned; ADR 0003's new "Phase 4 implementation note"
  confirms blueprint insertion and physical ports are deferred to Phase 5,
  consistent with every other file. Zero contradictions found.
- [X] T020 Run `git status --short` and `git diff --stat` and confirm
  exactly the six `DocumentTarget` files changed relative to T002's
  baseline, with nothing under `src/`, `tests/`, `catalog/`, `data/`,
  `.hermes/`, `specs/001-blueprint-library/`, or
  `specs/002-blueprint-library-ui/` listed (spec.md SC-005, FR-008;
  depends on T002, T005, T008, T016)

  **Result**: `git status --short` shows exactly the six modified files
  (`CONTEXT.md`, `README.md`, `docs/adr/0003-cad-documents-and-blueprints.md`,
  `docs/architecture.md`, `docs/data-model.md`, `docs/roadmap.md`) plus the
  untracked `specs/003-phase-4-closure/` SDD trail itself. `git diff --stat`
  confirms 6 files changed, 43 insertions(+), 35 deletions(-). Nothing
  under `src/`, `tests/`, `catalog/`, `data/`, `.hermes/`, or either
  historical spec directory appears.
- [X] T021 Run all six required gates in order — `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo
  test`, `cargo build --release --bins`, `git diff --check`, `hermes
  verify --skip-start --json --timeout 300` — and confirm every one passes
  unchanged from the pre-closure baseline, since no Rust source was
  touched (Constitution Principle IV; research.md Decision 1; depends on
  T020)

  **Result**: All six gates green.
  - `cargo fmt --check` → exit 0
  - `cargo clippy --all-targets --all-features -- -D warnings` → exit 0,
    zero warnings
  - `cargo test` → 350 tests, 0 failed (identical to Commit 8's baseline;
    no source touched)
  - `cargo build --release --bins` → `Finished release profile [optimized]`
  - `git diff --check` → exit 0 (only benign LF/CRLF normalization
    warnings, no blocking whitespace error)
  - `hermes verify --skip-start --json --timeout 300` → `"ok": true`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user
  stories
- **User Story 1 (Phase 3)**: Depends on Foundational (T002) completion
- **User Story 2 (Phase 4)**: Depends on Foundational (T002) completion;
  does not depend on User Story 1
- **User Story 3 (Phase 5)**: Depends on Foundational (T002) completion;
  does not depend on User Story 1 or User Story 2
- **Polish (Phase 6)**: Depends on every user story's verification task
  (T005, T008, T016) and T002's baseline

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on US2 or US3 — touches only
  `docs/roadmap.md`
- **User Story 2 (P2)**: No dependency on US1 or US3 beyond the shared
  Foundational phase — touches only `README.md` and `CONTEXT.md`
- **User Story 3 (P3)**: No dependency on US1 or US2 beyond the shared
  Foundational phase — touches only `docs/architecture.md`,
  `docs/data-model.md`, and the ADR

### Within Each User Story

- Edits to the same file are sequential (T003→T004; T009→T010;
  T011→T012→T013→T014) to avoid conflicting simultaneous edits; edits to
  different files within a story have no ordering requirement
- Each story's own verification task (T005, T008, T016) runs only after
  every edit task in that story
- Story complete before moving to the next priority, per this project's
  established sequential-commit workflow

### Parallel Opportunities

Five tasks are marked `[P]`: T006/T007 (README.md vs. CONTEXT.md — US2),
and T009/T011/T015 (docs/architecture.md vs. docs/data-model.md vs. the
ADR — US3, each the first task touching its file). These are genuinely
different files with no dependency between them. As in
`specs/001-blueprint-library/tasks.md` and
`specs/002-blueprint-library-ui/tasks.md`, this project is implemented by
a single agent in sequential, narrated, atomic commits (constitution
"Workflow and Branching"), not a multi-person team that would exploit
file-level parallelism — the `[P]` markers record which edits are
order-independent for traceability, not an instruction to actually
multi-thread this closure.

---

## Parallel Example: User Story 2

```bash
# These two edits touch different files and can be done in either order:
Task: "Update README.md's Current status line to state Phase 4 is integrated"
Task: "Update CONTEXT.md's Roadmap and next implementation section"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T003-T005)
4. **STOP and VALIDATE**: read `docs/roadmap.md` alone and confirm it
   correctly states Phase 4 is complete and Phase 5 is next
5. This alone already delivers this closure's highest-value outcome
   (spec.md US1's "Why this priority"): the project's own designated
   resumption document is no longer stale

### Incremental Delivery

1. Setup + Foundational → baseline ready (T001-T002)
2. Add User Story 1 → validate independently → the roadmap alone is now
   accurate (T003-T005)
3. Add User Story 2 → validate independently → the two most-read
   newcomer-facing files now agree (T006-T008)
4. Add User Story 3 → validate independently → the architecture and
   data-contract documents have zero drift (T009-T016)
5. Polish (T017-T021) → cross-file consistency, scope, and all six gates
   green, commit

---

## Implementation Record — findings from independent review

The original plan contained T001-T021. Independent review of parent
`a498ef0c9530c818e2f6a9c9d01d70ae0a99ed84` and candidate tree
`2946129601b039f4318d11b15316fd3da0f1f206` failed both specification
compliance and factual accuracy. The initial T019 read-through therefore
missed real contradictions; its result above records that initial check,
not approval of the final candidate. T020's diff statistics and T021's
gate results likewise describe the pre-review snapshot.

The following three corrective tasks extend the original plan without
changing its six-document scope. They are not three additional features
or a count of individual edits.

- [X] T022 Correct residual status language in `docs/roadmap.md` outside
  T003's original hunk. The milestone paragraph now says Phases 3 and 4
  are integrated and Phase 5 is next. The Phase 0 summary attributes
  document persistence to Phase 4 and reserves future migration work for
  a second schema version. The review flagged the milestone contradiction;
  the subsequent broader scan also corrected the Phase 0 summary.
- [X] T023 Correct the `persistence/` source-tree comment in
  `docs/architecture.md`: versioned JSON and atomic save are implemented,
  not future work. This resolves the contradiction with the same file's
  later "CAD, documents, and runtime data" section. No application
  module or future-feature scope was changed.
- [X] T024 Correct two capability claims in `docs/roadmap.md` against
  `src/egui_app.rs:615-640` and
  `src/persistence/blueprint_library.rs:129-225`. New has no bound keyboard
  shortcut; only Ctrl+O, Ctrl+S, and Ctrl+Shift+S are documented as
  document shortcuts. Library enumeration silently skips symlinks,
  directories, and non-matching filenames; unreadable/malformed candidate
  contents and duplicate identities produce invalid-entry warnings.

The narrower SC-002 patterns still yield the two false positives explained
under T017. They are discovery aids, not proof of whole-file consistency;
review also inspected the bare status words and phase-number references
throughout the six documents. ADR 0003's original Status, Decision, and
dated Phase 3 note remain historical records; its new Phase 4 note supplies
the current implementation status.

A subsequent review of corrected tree
`eef0143c1ed3fe8ef79c6eb4e9f8e2170a787d5b` approved specification
compliance, but its factual reviewer ended with HTTP 429 and a failed
response-schema check. That incomplete review is not an approval.

A fresh factual review of re-verified tree `1a9de4484f223c42ff7946adfee7d3f81c9bb191`
(same six-document/SDD-file scope, only `specs/003-phase-4-closure/tasks.md`
itself changed since `eef0143` to record T022-T024 truthfully) completed
and returned `passed: false` with seven blocking findings (B1-B7), each
citing exact source lines. None required a Rust source change; every
finding was documentation prose that overclaimed, underclaimed, or
duplicated a stale claim the earlier rounds had already fixed in one
file but not its sibling. The following corrective tasks fix each finding
against the cited source, still inside this closure's six-document scope:

- [X] T025 (fixes B1) Correct `docs/data-model.md`'s "Implemented
  blueprint document" diagram: `BlueprintDocument`'s DTO nests `name`,
  `description` (required, nullable), `created_at`, and `updated_at`
  inside a `metadata` object — confirmed against
  `src/persistence/blueprint_document.rs:146-164`
  (`BlueprintDocumentV1Dto`/`DocumentMetadataDto`) — not as root-level
  fields as the diagram previously showed.
- [X] T026 (fixes B2) Correct `docs/data-model.md` and `docs/roadmap.md`'s
  catalog-mismatch prose: a `catalog_id`/`catalog_data_version` mismatch
  alone is non-blocking, but loading still performs full validation
  against the *active* catalog and can fail; opening never rewrites the
  on-disk file; a reconstructed blueprint's provenance is the active
  catalog, not the original mismatching one — confirmed against
  `src/persistence/factory_document.rs:353-419`,
  `src/persistence/blueprint_document.rs:273-335`, and
  `src/domain/blueprint.rs:207-258` (`from_nodes`). The prior wording
  implied an unconditional, validation-free "loads unchanged" guarantee.
- [X] T027 (fixes B3) Correct `docs/data-model.md` and `docs/roadmap.md`
  to distinguish three separate things previously conflated as one
  "persistent" warning: (a) the factory `DocumentSession`'s retained
  in-memory compatibility state; (b) the single current, *replaceable*
  `EditorNotice` shown after opening — replaced by later actions such as
  selecting an instance (`src/egui_app.rs:958-980`) — and reset to
  `Exact` after a successful save (`src/document_session.rs:69-90`); and
  (c) the blueprint library's separate, per-cached-row mismatch
  indicators (`src/egui_app.rs:1679-1686`), which are the only ones
  that are actually persistent/visible across actions.
- [X] T028 (fixes B4) Correct the ADR's Phase 4 implementation note
  (`docs/adr/0003-cad-documents-and-blueprints.md`) to state that only
  Open, Save, and Save As have keyboard shortcuts (`Ctrl+O`, `Ctrl+S`,
  `Ctrl+Shift+S`) and New is button-only — confirmed against
  `src/egui_app.rs:615-640` — replacing a duplicate of the Ctrl+N
  overclaim T024 had already fixed in `docs/roadmap.md` but not here.
- [X] T029 (fixes B5) Correct the same ADR note's enumeration-warning
  claim: `BlueprintLibrary::list()` silently ignores symlinks,
  directories, and non-blueprint filenames before ever reading them;
  only read/decode failures on matching regular-file candidates and
  duplicate-identity losers become safe, path-free warnings — confirmed
  against `src/persistence/blueprint_library.rs:129-171,201-210` —
  replacing a duplicate of the symlink-as-warning overclaim T024 had
  already fixed in `docs/roadmap.md` but not here.
- [X] T030 (fixes B6) Correct `docs/roadmap.md`'s Phase 4 closing bullet:
  only blueprint insertion and physical-port interfaces are committed to
  Phase 5 (per this closure's own `spec.md` and the unchanged
  `specs/002-blueprint-library-ui/spec.md`); blueprint editing, deletion,
  renaming, import, and export are correctly stated as not delivered and
  not assigned to any specific phase, rather than bundled into Phase 5
  without approval.
- [X] T031 (fixes B7) Correct `docs/data-model.md`'s "Implemented
  persistence" section: save inputs come from domain APIs and
  `DocumentMetadata` construction, not one shared complete-domain-
  validation pass at save time. Factory encoding checks `next_entity_id`,
  formats metadata timestamps, then rejects zero entity IDs while
  encoding entities (`src/persistence/factory_document.rs:270-305`);
  blueprint encoding formats metadata and serializes already-built nodes
  without a further domain/canonical-ID revalidation pass
  (`src/persistence/blueprint_document.rs:217-255`,
  `src/domain/blueprint.rs:201-206`). The atomic same-directory
  temp-file/flush/sync/replace sequence itself was already accurate and
  is unchanged.

T025-T031 are documentation-only corrections inside the same six-file
`DocumentTarget` scope; no Rust source, test, or historical spec file
was touched. Two of the seven findings (B4, B5) were pure duplicates:
T024 had already corrected the equivalent claim in `docs/roadmap.md`,
but the ADR's own copy of that same claim, added by T015/T028's shared
"Phase 4 implementation note," had not been checked against the same
source and still carried the earlier, incorrect wording — a reminder
that a multi-document closure must re-verify a corrected claim
everywhere it is duplicated, not only where a reviewer happened to
point.

A fresh, from-scratch final review of re-verified tree
`967d1a8bd8cf41c3d3ca05b8c9636c4d54e09d32` confirmed all seven B1-B7
fixes correct against the cited source, but surfaced one further
finding outside B1-B7 and outside every prior round's scope:

- [X] T032 `README.md`'s walkthrough section (line 120, untouched by
  every prior hunk in this closure) still read "History and persistence
  are not yet part of the egui interface" — directly contradicting this
  closure's own corrected status banner three lines above (`README.md`
  line 5, from T006), and the numbered walkthrough (lines 103-114) never
  mentioned any document command or the blueprint library at all. Fixed:
  narrowed the sentence to the part still true ("History (undo/redo) is
  not yet part of the egui interface") and added one paragraph naming
  the actual `Ctrl+O`/`Ctrl+S`/`Ctrl+Shift+S` shortcuts, button-only
  New, **Save as blueprint**, and the **BLUEPRINT LIBRARY** sidebar
  section — confirmed against `src/egui_app.rs:615-640`
  (`document_shortcut_for_frame`) and the same Save-as-blueprint/library
  UI already verified for T003. This finding matters beyond its own
  fix: it escaped every mechanical SC-002 grep and every prior review
  round because it uses none of the target vocabulary
  (`FactoryDocument`/`BlueprintDocument`/`blueprint library`) and sits
  outside the specific stale-passage locations `data-model.md`'s
  `DocumentTarget` table named — a reminder that a documentation
  closure's grep-based checks bound recall to the vocabulary anticipated
  in advance, not to the full space of possible staleness.

Final commit preparation requires: re-running the mechanical SC-002/
SC-003/SC-004 checks and all six gates on this corrected tree's exact
new identity, a native startup smoke check, and a fresh independent
review of that exact tree with no blocking findings. Earlier gate,
smoke, and review results are provenance for their own snapshot only
and do not carry forward automatically.

---

## Notes

- **[Story] label** maps every task to its owning user story for
  traceability back to `spec.md`
- Tasks with no `[Story]` label (T001-T002, T017-T021) are Setup,
  Foundational, or Polish — not story-specific
- This `tasks.md`, together with `plan.md`, `research.md`, `data-model.md`,
  and `quickstart.md`, is the complete spec-driven design record for this
  commit; implementation should not need to re-derive any decision already
  made in those files
- Commit narration, the six-gate verification, and this project's
  established independent-review step at commit-preparation time (per
  `docs/roadmap.md`'s "Engineering workflow per slice," step 7, which
  applies to every slice without a documentation-only carve-out) happen
  after T021 — they are process, not feature tasks, and are intentionally
  not numbered above. Since this closure introduces no domain, persistence,
  or UI code, that review's focus is documentation accuracy against
  spec.md's functional requirements and `data-model.md`'s
  `PhaseStatusClaim`s (SC-002, SC-004), not code security — unlike the
  domain/persistence-adjacent-code review scope Commits 4-8 used
  (`requesting-code-review` skill).
