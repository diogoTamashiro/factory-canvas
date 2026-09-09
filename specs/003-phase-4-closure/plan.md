# Implementation Plan: Phase 4 Closure Documentation

**Branch**: `003-phase-4-closure` (directory identifier only — see spec.md
header; no git branch, per constitution Workflow and Branching; this is
still the last commit of Phase 4, which predates branch-per-phase)
**Date**: 2026-09-07
**Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-phase-4-closure/spec.md`

## Summary

Reconcile six already-tracked documentation files
(`docs/roadmap.md`, `README.md`, `CONTEXT.md`, `docs/architecture.md`,
`docs/data-model.md`, `docs/adr/0003-cad-documents-and-blueprints.md`) with
the JSON-document and blueprint-library capability actually delivered by
Commits 1–8 of Phase 4, converting every stale "planned"/"next" passage
about `FactoryDocument`, `BlueprintDocument`, or the blueprint library into
an accurate "integrated" statement, while leaving genuinely-still-future
items (physical ports, blueprint insertion, undo/redo) correctly marked as
planned for Phase 5+. This is a pure documentation closure: no source,
test, catalog, or historical-spec file changes (spec.md FR-008, SC-005).

## Technical Context

**Language/Version**: N/A for this feature's own diff — no Rust source is
touched. The repository's existing toolchain (Rust, stable, edition 2021)
is unchanged and its six verification gates still run against it
unmodified (see Constitution Check below).

**Primary Dependencies**: None. No `Cargo.toml`/`Cargo.lock` change.

**Storage**: N/A — no runtime storage behavior is touched; this closure
only edits static Markdown files already tracked in the repository.

**Testing**: The project's six required gates (`cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
`cargo build --release --bins`, `git diff --check`,
`hermes verify --skip-start --json --timeout 300`) still run and must pass
— see research.md's resolution of the `CONTRIBUTING.md`-vs-Constitution
tension on whether a docs-only commit may skip them. This feature's own
"testing" is a mechanical content check (quickstart.md): grep-verify that
no targeted stale phrase remains (SC-002) and that every touched
cross-reference resolves to a real path (SC-003).

**Target Platform**: N/A (documentation).

**Project Type**: Documentation-only change to an existing Windows desktop
Rust application (`eframe/egui`); no application code is added or altered.

**Performance Goals**: N/A.

**Constraints**: Exactly the six files named in FR-001 through FR-006 may
change (SC-005); nothing under `src/`, `tests/`, `catalog/`, `data/`,
`.hermes/`, or the historical `specs/001-blueprint-library/` and
`specs/002-blueprint-library-ui/` directories may be touched (FR-008). No
confirmed game-data fact (bases, footprints, dimensions) may be altered
(FR-007). ADR 0003's original Status and Decision text must not be
rewritten — only a new, dated implementation note may be appended (FR-006,
mirroring the existing Phase 3 implementation note already in that file).

**Scale/Scope**: Six files edited, zero files created or deleted (the
`specs/003-phase-4-closure/` SDD trail itself is process scaffolding, not
part of the six-file closure diff). Comparable in effort to the documented
Phase 3 closure that already touched `docs/data-model.md`'s status banner
and appended ADR 0003's "Phase 3 implementation note."

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No dependency, abstraction,
  or process rule changes; this closure only corrects factual staleness in
  already-existing documents.
- **II. Architectural Decisions Live in ADRs**: PASS. No new architectural
  decision is introduced. ADR 0003 remains the binding decision record;
  this closure appends a dated implementation note (the same mechanism
  already used for Phase 3) rather than altering its Status or Decision.
- **III. Spec-Driven From Commit 7 Onward**: PASS. This plan follows an
  already-authored, checklist-validated `spec.md` (16/16) for this feature.
- **IV. Gates Are Still Mandatory**: PASS, with an explicit resolution
  recorded in research.md — `CONTRIBUTING.md`'s older "documentation-only
  changes may skip builds and tests" clause (predates this constitution)
  is superseded by Principle IV's unconditional "every commit MUST still
  pass" wording for this SDD-governed phase. All six gates still run.
- **V. Privacy and Catalog Boundaries**: PASS. No `data/**`, `reference/**`,
  or `.hermes/**` content is touched, read, or introduced; no game entity,
  base, product, region, dimension, capacity, or port is invented — this
  closure only describes already-confirmed, already-public capability.

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — Phase 1 introduces no new domain, persistence, or UI
type (the "entities" in data-model.md are documentation-content tracking
constructs for this closure itself, not application types), no dependency
change, and no decision contradicts ADR 0001, ADR 0002, or ADR 0003. The
Constitution Check above still holds unchanged post-design.

## Project Structure

### Documentation (this feature)

```text
specs/003-phase-4-closure/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no API, CLI, or external
service boundary — it edits static documentation only (same justification
`specs/001-blueprint-library/plan.md` and
`specs/002-blueprint-library-ui/plan.md` already used for their own
no-`contracts/` decision).

### Source Code (repository root)

This feature's actual "source" is the six documentation files themselves,
not `src/`:

```text
docs/
├── roadmap.md              # MODIFY — new integrated "Phase 4" section;
│                             # "Next active slice" updated to Phase 5
├── architecture.md         # MODIFY — "CAD, documents, and runtime data"
│                             # and "Next increments" reconciled with what
│                             # Commits 1-8 actually shipped
├── data-model.md           # MODIFY — status banner and "Planned factory
│                             # document"/"Planned blueprint document"/
│                             # "Planned persistence" sections updated
└── adr/
    └── 0003-cad-documents-and-blueprints.md
                             # MODIFY — new dated "Phase 4 implementation
                             # note" appended after the existing "Phase 3
                             # implementation note"; Status/Decision
                             # unchanged

README.md                   # MODIFY — "Current status" line
CONTEXT.md                  # MODIFY — "Roadmap and next implementation"

src/                         # UNCHANGED — no application code in this feature
tests/                       # UNCHANGED
catalog/                     # UNCHANGED
data/                        # UNCHANGED (ignored, private)
.hermes/                     # UNCHANGED (ignored, private)
specs/001-blueprint-library/  # UNCHANGED — frozen historical SDD trail
specs/002-blueprint-library-ui/  # UNCHANGED — frozen historical SDD trail
```

**Structure Decision**: Edit exactly the six tracked documentation files
named above in place; introduce no new document, no new directory, and no
change anywhere in `src/` or `tests/`. This mirrors the project's own
Phase 3 closure precedent (a roadmap update plus a `data-model.md` status
banner plus an ADR implementation note), scaled up to the full six-file set
this larger phase's own spec (User Stories 1–3) requires.

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*
