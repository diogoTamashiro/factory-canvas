<!--
Sync Impact Report
- Version change: (unratified template) → 1.0.0
- Modified principles: none (first real ratification)
- Added sections: Core Principles I-V; Workflow and Branching; Governance
- Removed sections: none
- Rationale: this constitution intentionally defers all substantive engineering
  rules to docs/engineering-standards.md and all architectural decisions to
  docs/adr/*, which already existed and are Accepted before this file was
  filled in. It does not restate their content — see Governance.
- Follow-up TODOs: none
-->

# Factory Canvas Constitution

## Core Principles

### I. Explicit Engineering Standards

This project's engineering principles — KISS, YAGNI, DRY in moderation,
pragmatic SOLID, ACID for persistence, dependency policy, Git/commit
conventions, pre-commit review, and Definition of Done — are governed by
`docs/engineering-standards.md`. That file is the single source of truth;
this constitution does not restate its rules and MUST NOT drift from it. If a
`/speckit-*` workflow needs a principle not yet covered there, add it to
`docs/engineering-standards.md` first, then reference it from here.

### II. Architectural Decisions Live in ADRs

Product and architecture decisions — UI stack, product naming, document and
blueprint schema, catalog/runtime-data boundaries, and similar choices — are
recorded in `docs/adr/*`. Existing and future ADRs are binding. A
`/speckit-plan` or `/speckit-implement` run MUST NOT contradict an Accepted
ADR without first recording a new ADR that supersedes it.

### III. Spec-Driven From Commit 7 Onward (NON-NEGOTIABLE)

Effective 2026-09-07, work on Factory Canvas follows spec-driven development
(SDD) at the spec-anchored rigor level: a written, reviewed, and approved
spec (and, where warranted, a plan/tasks breakdown) precedes implementation.
Commits 1-6 of the in-flight persistence phase were built under strict TDD
(red-green-refactor) before this change and are not retroactively
re-specified — they stand as-is. This principle changes what is authored
*first*; it does not remove testing (see Principle IV).

### IV. Gates Are Still Mandatory

Every commit MUST still pass, in this order: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
`cargo build --release --bins`, `git diff --check`, and
`hermes verify --skip-start --json --timeout 300`. SDD changes *how*
behavior is defined before implementation; it does not relax verification
after implementation.

### V. Privacy and Catalog Boundaries

Private game/reference data under `data/**`, `reference/**`, and
`.hermes/**` MUST NOT be committed, echoed in error messages, or included in
test fixtures. The public catalog stays minimal and uses the real production
schema. No game entity, base, product, region, dimension, capacity, port, or
terminology is invented — see
`docs/adr/0003-cad-documents-and-blueprints.md` and `docs/data-model.md`.

## Workflow and Branching

- Commits 1-6 of Phase 4 were merged directly to `master` before this
  constitution existed; they are not moved to a feature branch
  retroactively. The remaining commits of that phase also continue direct on
  `master`, since the phase itself predates this change of method.
- Starting with the next new roadmap phase (Phase 5 onward), each phase is
  developed on its own spec-kit feature branch (`specs/NNN-slug/`) and
  merged to `master` once that phase's commits are complete and verified.
  One spec-kit feature/branch corresponds to one roadmap phase, which may
  contain several atomic commits — not one branch per commit.
- Within a phase/feature branch, commits remain small, atomic, narrated, and
  independently reversible, per `docs/engineering-standards.md` §Git.
- `git commit`/`git push` happen only after the user has approved the plan
  for that unit of work; work is narrated, never silent.

## Governance

This constitution defers substantive engineering rules to
`docs/engineering-standards.md` and architectural rules to `docs/adr/*`;
amending those files is how this constitution's substance actually changes.
Only the SDD-specific and branching rules in Principles III-IV and the
Workflow and Branching section above are owned directly by this file. Any
`/speckit-*` command whose output would conflict with an Accepted ADR or with
`docs/engineering-standards.md` MUST stop and ask the user rather than
silently overriding it.

**Version**: 1.0.0 | **Ratified**: 2026-09-07 | **Last Amended**: 2026-09-07
