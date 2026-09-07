# Implementation Plan: Blueprint Library Persistence

**Branch**: `001-blueprint-library` (spec-directory identifier only; no git
branch — see constitution Workflow and Branching) | **Date**: 2026-09-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-blueprint-library/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Persist valid blueprints to a per-user, offline storage location and list
them back — including after a full application restart — so that a
blueprint captured once (Commit 5) is not lost. Implemented as one new
persistence module, `src/persistence/blueprint_library.rs`, that composes
three already-shipped building blocks without modifying any of them: the
`Blueprint`/`BlueprintId` domain types (Commit 5), the
`encode_blueprint_document`/`decode_blueprint_document` codec (Commit 6),
and the atomic same-directory write primitive (Commit 2). No new crate
dependency, no UI, no canvas insertion — those are explicitly out of scope
per spec.md §Assumptions and FR-012.

## Technical Context

**Language/Version**: Rust 2021 edition, `rustc`/`cargo` 1.97.1 (existing toolchain; unchanged)

**Primary Dependencies**: `std` only for this module's own logic (`std::fs`, `std::env`, `std::path`); reuses already-present crates transitively via the codec it calls (`serde`/`serde_json`, `time`, `semver`) and via `atomic_file` (`tempfile`). No new dependency is added to `Cargo.toml`.

**Storage**: Local filesystem — one JSON file per blueprint under `%LOCALAPPDATA%/Factory Canvas/blueprints/` (default), fully injectable to a temporary directory in tests (research.md §2)

**Testing**: `cargo test`, new integration tests in `tests/blueprint_library.rs` using `tempfile::tempdir()` — same pattern as `tests/blueprint_document_codec.rs` and the existing `atomic_file.rs` unit tests

**Target Platform**: Windows desktop only (existing product constraint, `docs/adr/0001-editor-ui.md`)

**Project Type**: Single Rust crate, desktop application — this feature adds one module to the existing `src/persistence/**` tree, no new crate or binary

**Performance Goals**: Not specified by spec.md (no SC-### references latency/throughput). A personal blueprint library is expected to hold on the order of tens to low hundreds of files; `list()` performs one `read_dir` pass plus one JSON decode per file, which is well within normal desktop I/O responsiveness at that scale — no explicit numeric target is introduced beyond "does not introduce a second filesystem pass or a network round trip."

**Constraints**: Offline-only (no network access — spec.md §Assumptions); every write MUST be atomic within the target directory (FR-011, reuses existing `atomic_file` primitive, same-volume same-directory rename); every user-visible warning MUST NOT leak a file path, raw file content, or technical identifier (FR-008, `docs/engineering-standards.md` privacy conventions already established in Commits 4 and 6)

**Scale/Scope**: One new source file (`src/persistence/blueprint_library.rs`), one new test file (`tests/blueprint_library.rs`), a one-line registration change to `src/persistence/mod.rs`. No UI, no new domain type (FR-012; this is a persistence-layer feature only).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|---|---|---|
| I. Explicit Engineering Standards | No new principle needed; this plan follows `docs/engineering-standards.md` §KISS (reuse `atomic_file`, no new abstraction layer), §YAGNI (no manifest/index file, no configurable root, no mock filesystem trait — research.md §2, §3, §8), §DRY (single atomic-write implementation, single JSON codec — research.md §4), §Dependencies (`std` only, no new crate — research.md §2) | PASS |
| II. Architectural Decisions Live in ADRs | Consistent with `docs/adr/0003-cad-documents-and-blueprints.md` (JSON documents, no SQLite for v1, blueprints as independent copies); introduces no new architectural decision that would require a new ADR | PASS |
| III. Spec-Driven From Commit 7 Onward | This plan *is* the SDD artifact for Commit 7 — spec.md → clarify → plan.md (this file) precedes any implementation; no code has been written yet | PASS |
| IV. Gates Are Still Mandatory | quickstart.md enumerates the same six gates (`fmt`, `clippy`, `test`, `build --release`, `diff --check`, `hermes verify`) required before this commit lands; nothing in this plan proposes skipping them | PASS |
| V. Privacy and Catalog Boundaries | No `data/**`/`reference/**`/`.hermes/**` content is read or required (research.md confirms no external/game data needed); `InvalidLibraryEntryReason` is a closed enum with no path/content/ID field, enforced structurally not by convention (data-model.md) | PASS |

No violations. Complexity Tracking table below is empty.

**Post-Phase-1 re-check**: after generating `research.md`, `data-model.md`,
and `quickstart.md` below, this table was re-evaluated against the final
design (six new types, all in `blueprint_library.rs`, all persistence-layer,
zero new dependencies, zero new domain types). No principle regressed;
result unchanged from the pre-research pass above.

## Project Structure

### Documentation (this feature)

```text
specs/001-blueprint-library/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no external API, CLI, or
service boundary — its only consumer is in-crate egui code planned for
Commit 8. Per the `/speckit-plan` Phase 1 instructions ("skip if project is
purely internal"), contracts are skipped.

### Source Code (repository root)

```text
src/
├── domain/
│   └── blueprint.rs                  # EXISTING, unmodified (Commit 5) — Blueprint, BlueprintId
├── persistence/
│   ├── mod.rs                        # MODIFY — register `pub mod blueprint_library;`
│   ├── atomic_file.rs                # EXISTING, unmodified (Commit 2) — write_atomically
│   ├── blueprint_document.rs         # MODIFY (see Deviations, tasks.md) — added crate-private encode_blueprint_document_as
│   ├── factory_document.rs           # EXISTING, unmodified (Commit 1) — CatalogCompatibility (reused)
│   └── blueprint_library.rs          # CREATE — BlueprintLibrary, save()/list(), types in data-model.md
└── (no other file touched)

tests/
└── blueprint_library.rs              # CREATE — integration tests per quickstart.md scenarios
```

**Structure Decision**: Single Rust crate (matches every prior commit in
this phase; no web/mobile split applies). `src/domain/**` was not modified
— the constitution's Principle I/`docs/engineering-standards.md`
§Pragmatic SOLID boundary ("the domain does not depend on UI or I/O") is
preserved because this feature is entirely a persistence-layer consumer of
the domain, never the reverse. One file originally scoped as
"EXISTING, unmodified" — `blueprint_document.rs` — required one small,
additive change during implementation (a crate-private
`encode_blueprint_document_as` helper); see tasks.md's "Deviations from
plan.md discovered during implementation" for the concrete correctness
reason (save-time ID collision retry must re-key the document's declared
identity, not just its filename, or FR-009's duplicate-resolution logic
would silently re-collapse the two blueprints back into one entry).

## Complexity Tracking

*No entries — Constitution Check above reports no violations to justify.*
