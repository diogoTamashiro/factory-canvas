<!--
Sync Impact Report
- Version change: 1.2.0 → 1.3.0
- Modified sections: Workflow and Branching (merging a feature branch back
  to `master` requires a hosted pull request again; the underlying
  `git merge --ff-only`-from-a-confirmed-`master` mechanics that keep
  history un-combinable are unchanged)
- Added sections: none
- Removed sections: none
- Rationale: Diogo directed the project to resume opening a hosted pull
  request per feature branch (2026-09-13), reversing v1.2.0's removal of
  that step. v1.2.0 dropped the PR specifically because GitHub's own
  squash-merge button silently combined Phase 5's PR #18 with an unrelated
  same-day commit once `master` and `origin/master` had diverged — the PR
  itself was never the problem, GitHub's default merge button was. Keeping
  the PR requirement while still merging locally with
  `git merge --ff-only` (never GitHub's merge-commit/squash button)
  restores the review/history benefits of a PR without reintroducing the
  specific failure v1.2.0 fixed.
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
`cargo clippy --all-targets --all-features -- -D warnings`, the automated tests
covering what that commit actually changes (see
`docs/engineering-standards.md` §Testing scope — never a blanket full-suite
run by default), `cargo build --release --bins`, `git diff --check`, and
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
- Merging a feature branch back to `master` MUST open a hosted pull
  request on GitHub before merging, even while Diogo remains this
  project's sole maintainer — effective 2026-09-13 (Constitution v1.3.0),
  reinstated by Diogo's explicit direction after Phases 6-8 and the
  large-file-split refactor operated without one. The PR keeps each
  feature's diff, CI status, and review discussion attached to the
  repository itself rather than only to a local Hermes session log. The
  frozen branch still needs a review before merging (Diogo's own review,
  an independent reviewer subagent per the `requesting-code-review`
  skill, or both), exactly as under the no-PR policy this replaces. The
  PR itself is still merged with `git merge --ff-only` from a `master`
  already confirmed equal to `origin/master` (`git fetch origin master`
  first) and then pushed directly — NOT GitHub's own merge-commit or
  squash-merge button, which can silently combine or reorder history
  (Phase 5's PR #18 squash-merged an unrelated local governance commit
  into the feature's own history this exact way, which is why the PR
  requirement was dropped in v1.2.0; reinstating the PR does not
  reintroduce that specific risk because the actual merge mechanics stay
  unchanged). If the fast-forward is refused because the two have
  diverged, sync or rebase the *feature* branch onto the confirmed-fresh
  `origin/master` and retry — never rebase onto a local `master` that has
  not itself been confirmed equal to the remote.
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

**Version**: 1.3.0 | **Ratified**: 2026-09-07 | **Last Amended**: 2026-09-13
