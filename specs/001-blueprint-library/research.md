# Phase 0 Research: Blueprint Library Persistence

**Input**: `specs/001-blueprint-library/spec.md` (post-clarification)

All items below were `NEEDS CLARIFICATION` candidates in the initial
Technical Context pass. Each is resolved by combining the spec's own
Assumptions section, the already-Accepted `docs/adr/0003-cad-documents-and-blueprints.md`,
and the concrete precedent already in the codebase from Commits 1-6 of this
same persistence phase.

## 1. Runtime toolchain and target

- **Decision**: Rust 2021 edition, current toolchain (`rustc 1.97.1`),
  built as part of the existing `factory-canvas` binary crate (`src/egui_main.rs`
  entry point) — no new crate, no new binary target.
- **Rationale**: matches every other module under `src/persistence/**`;
  `docs/adr/0001-editor-ui.md` fixes Rust + eframe/egui as the stack, and
  `docs/engineering-standards.md` forbids introducing parallel stacks.
- **Alternatives considered**: none — changing toolchain/target for one
  module inside an existing crate was never in scope.

## 2. Storage location resolution

- **Decision**: resolve the default root once, lazily, via
  `std::env::var("LOCALAPPDATA")` joined with `Factory Canvas/blueprints`,
  matching the default already fixed in the (pre-SDD) Phase 4 plan:
  `%LOCALAPPDATA%/Factory Canvas/blueprints/`. The resolver function takes
  the root as an explicit parameter everywhere internally; only one thin
  wrapper reads the environment variable, so every test injects a temporary
  directory instead and the domain/persistence code never depends on the
  real user profile.
- **Rationale**: `std::env::var` needs no new dependency, satisfies
  `docs/engineering-standards.md` §Dependencies ("prefer std"; "every crate
  needs a documented current benefit"). A `dirs`/`directories` crate would
  only save one `env::var` call and is not justified. Windows-only is
  already an established constraint (`docs/adr/0001-editor-ui.md`), so no
  cross-platform path resolution abstraction is needed.
- **Alternatives considered**: `directories` crate (rejected — unjustified
  new dependency for one env lookup); a path under the executable's own
  directory like the legacy `screenshot.rs` capture folder (rejected — that
  pattern is specific to the frozen legacy binary in `src/main.rs` and does
  not survive reinstalling/moving the application, unlike a proper per-user
  profile directory).

## 3. File naming and identity

- **Decision**: one file per blueprint, named `<blueprint_id>.factory-blueprint.json`
  where `<blueprint_id>` is the full `blueprint_<uuid-simple>` string already
  produced by `BlueprintId::generate()` (Commit 5) and already round-tripped
  by `encode_blueprint_document`/`decode_blueprint_document` (Commit 6). The
  library never derives or stores identity separately from the blueprint
  document's own `blueprint_id` field — the filename is a cache of that
  field for fast collision/lookup checks, not a second source of truth.
- **Rationale**: reuses the existing default from the pre-SDD Phase 4 plan
  and the ID scheme already implemented and reviewed in Commit 5; avoids
  inventing a second identity or index format.
- **Alternatives considered**: an index/manifest file listing all blueprints
  (rejected — the spec's Assumptions explicitly keep this phase filesystem-
  discovery based, no manifest to keep in sync, matching FR-002/FR-003's
  "no manual setup" requirement and avoiding a second state to corrupt).

## 4. Atomic save

- **Decision**: reuse `src/persistence/atomic_file.rs::write_atomically`
  unchanged (temp file in the same directory, flush, sync, atomic rename).
  No second save algorithm is implemented.
- **Rationale**: FR-011 ("a save that fails or is interrupted MUST leave any
  previously stored blueprint file... unchanged") is exactly what
  `write_atomically` already guarantees and already has a passing regression
  test (`failed_atomic_replace_preserves_previous_target_bytes`).
  `docs/engineering-standards.md` §ACID requires "temporary file plus
  rename" for atomicity; a second implementation would violate DRY.
- **Alternatives considered**: direct `fs::write` (rejected — not atomic,
  directly violates FR-011); a database (SQLite) (rejected in
  `docs/adr/0003-cad-documents-and-blueprints.md` "Alternatives considered"
  for the first version of these documents).

## 5. Listing: discovery, filtering, ordering

- **Decision**: `fs::read_dir` the root once per `list()` call; for each
  entry, use `DirEntry::file_type()` (not `fs::metadata`, which follows
  links) to skip anything that is a symlink; skip anything whose file name
  does not end in `.factory-blueprint.json`; attempt to read and
  `decode_blueprint_document` every remaining candidate; sort successfully
  decoded entries by `metadata().name()` ascending, tiebreaking on
  `id().as_str()` ascending (per the spec Clarifications answer, FR-004).
- **Rationale**: `std::fs::FileType::is_symlink()` on Windows is derived
  from the `FILE_ATTRIBUTE_REPARSE_POINT` flag returned directly by
  `FindFirstFile`/`FindNextFile` (what `read_dir` uses internally) without
  following the reparse point, so FR-006 ("ignore symlinks... rather than
  following") is satisfiable with `std::fs` alone, no extra crate. Filename
  suffix filtering satisfies FR-005 ("ignore any file that is not a
  blueprint file") without opening every unrelated file in the directory.
  Sorting by name then ID gives an order that is a pure function of file
  contents, satisfying FR-004/SC-004 determinism regardless of filesystem
  iteration order (which `fs::read_dir` does not guarantee is stable).
- **Alternatives considered**: sort by filename directly (rejected — the
  filename is the ID, and the spec's clarified answer requires sorting by
  the human-readable display name, not the ID); sort by
  `updated_at`/mtime (rejected by the clarification answer, kept only as
  the non-recommended Option B).

## 6. Invalid-entry and duplicate-identity handling

- **Decision**: a file that fails to decode (`decode_blueprint_document`
  returns `Err`) becomes exactly one `InvalidLibraryEntry` value carrying no
  path, no raw bytes, and no technical identifier — only a fixed, safe
  reason enum, mirroring the `BlueprintDocumentError`/`FactoryDocumentError`
  privacy pattern already reviewed and shipped in Commits 4 and 6. When two
  or more successfully-decoded documents report the same `blueprint_id`
  (FR-009), the deterministic sort from item 5 is applied first (by name
  then by the shared ID, which is now itself a further tiebreak — for two
  same-ID entries, the *filename* is the final, purely mechanical tiebreak
  since it is guaranteed distinct on a real filesystem), the first result
  is kept as the valid library entry, and every other file sharing that ID
  becomes one more safe invalid-entry warning of its own — never a second,
  ambiguous list entry.
- **Rationale**: directly satisfies FR-007/FR-008/FR-009 and reuses the
  privacy-safe error pattern that already passed two independent reviews in
  this same phase, rather than inventing a new error-formatting convention.
- **Alternatives considered**: silently dropping duplicate-ID files with no
  warning (rejected — spec's User Story 3 requires the player to see *a*
  safe warning, not silence, for a conflicting file); keeping both as
  separate list entries with a suffix (rejected — spec explicitly forbids
  "two separate, ambiguous entries").

## 7. Save-time ID collision

- **Decision**: before persisting a new blueprint, the save function lists
  existing blueprint IDs in the target directory (a cheap filename scan, not
  a full decode) and — in the astronomically unlikely event of a UUIDv4
  collision — regenerates a new `BlueprintId::generate()` and retries, up to
  a small fixed bound, before giving up with a safe, generic failure.
- **Rationale**: directly satisfies FR-010. `BlueprintId::generate()` already
  exists (Commit 5) and is cheap to call again; no new ID scheme is
  introduced.
- **Alternatives considered**: ignoring the (extremely rare) collision case
  entirely (rejected — FR-010 explicitly requires detection, and the spec's
  User Story 3 / Acceptance Scenario 4 requires it be tested).

## 8. Testing strategy

- **Decision**: `tests/blueprint_library.rs` integration tests using
  `tempfile::tempdir()` (already a dependency) as the injected root for
  every scenario — first-run auto-creation, discovery-after-restart
  (simulated by dropping and recreating the library value against the same
  temp path), invalid-file isolation, symlink handling (created via
  `std::os::windows::fs::symlink_file`, gated `#[cfg(windows)]` consistent
  with the Windows-only product), duplicate-ID handling, save-failure
  preservation (by making the target directory read-only mid-test, then
  restoring permissions for cleanup), and ID-collision retry (by
  pre-populating a file at the ID a mocked/seeded UUID would produce is not
  feasible without changing `BlueprintId::generate()`'s signature — see
  Complexity Tracking in plan.md for the scope decision here).
- **Rationale**: matches the established pattern from
  `atomic_file.rs`/`blueprint_document.rs` tests (`tempfile`, no mocking
  framework, real filesystem I/O in a throwaway directory) and
  `docs/engineering-standards.md` §TDD ("test... IDs, and round trips").
- **Alternatives considered**: an injectable trait/mock filesystem
  (rejected — YAGNI; the project has exactly one real filesystem target,
  Windows, and `docs/engineering-standards.md` §KISS forbids abstractions
  without more than one real need).

## Outcome

No `NEEDS CLARIFICATION` markers remain in Technical Context. Phase 1
(data-model.md, quickstart.md) can proceed. No `contracts/` directory is
produced — see plan.md's Project Structure section for the justification
(this module's only consumer is Commit 8's in-crate egui UI code, not an
external API/service boundary).
