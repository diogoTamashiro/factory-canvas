# Phase 0 Research: Phase 4 Closure Documentation

No item in Technical Context was marked `NEEDS CLARIFICATION` — every
unknown was resolvable directly from the repository's own tracked
documents (already read in full) and this project's established Phase 3
closure precedent. This document instead records the concrete decisions
this plan makes and the exact stale passages each decision targets.

## Decision 1: All six gates still run, despite this being a documentation-only commit

**Decision**: This commit runs the full six-gate sequence
(`cargo fmt --check`, `cargo clippy --all-targets --all-features --
-D warnings`, `cargo test`, `cargo build --release --bins`,
`git diff --check`, `hermes verify --skip-start --json --timeout 300`)
before being considered done, exactly like Commits 7 and 8.

**Rationale**: `CONTRIBUTING.md` §Verification says "Documentation-only
changes may skip builds and tests if they do not alter commands,
configuration, or behavior" — but that clause predates this project's
constitution (`CONTRIBUTING.md` was last touched 2026-08-29;
`.specify/memory/constitution.md` was ratified 2026-09-07) and is
superseded by it for SDD-governed work. Constitution Principle IV states
"Every commit MUST still pass" the six gates with no documentation
carve-out, and Governance requires any conflicting instruction to be
resolved rather than silently followed. Since Rust source is completely
unchanged, all six gates are expected to pass trivially and at negligible
cost — there is no real tension between "always run them" and "keep this
commit cheap."

**Alternatives considered**: Skip gates per `CONTRIBUTING.md`'s older
clause. Rejected: contradicts the newer, explicitly non-negotiable
Constitution Principle IV, which this project's own Governance section
says wins over an older, unreconciled document for exactly this kind of
conflict.

## Decision 2: The six target files and their exact stale passages

**Decision**: Update precisely these six files, at precisely these
locations — no other passage in them is touched:

| File | Stale passage(s) found | Required change |
|---|---|---|
| `docs/roadmap.md` | §"Next active slice and remaining MVP phases" lists "4. JSON documents and blueprint library" as upcoming (no phase 4 section exists among the integrated phases above it) | Add an integrated "## Phase 4 — JSON documents and blueprint library — integrated" section (same format as the existing "## Phase 3 — ... — integrated" section immediately above it); reduce "Next active slice" to just Phase 5 |
| `README.md` | Line 5 status paragraph: "...The next MVP work is versioned factory documents and local blueprints..." | State Phase 4 is integrated: save-as-blueprint and library browsing are available; next MVP work is Phase 5 |
| `CONTEXT.md` | §"Roadmap and next implementation": "Phase 3's runtime catalog ... are integrated. Phase 4 is next: versioned FactoryDocument and BlueprintDocument persistence with atomic local saves." | State Phase 4 is integrated; Phase 5 is next |
| `docs/architecture.md` | §"CAD, documents, and runtime data" intro: "Factory and blueprint documents remain the next layers"; §"Canvas" → "Next increments" bullet: "Phase 4 adds versioned factory documents, blueprint documents, migrations, and atomic local saves" | Reconcile both passages: documents/persistence are implemented; "Next increments" instead names Phase 5 (blueprint insertion, physical ports) |
| `docs/data-model.md` | Status banner (line 3): "...Physical ports, FactoryDocument, BlueprintDocument, migrations, saves, and blueprint insertion remain contracts for later phases."; §"Planned factory document"; §"Planned blueprint document"; §"Planned persistence" | Banner and section headings/bodies updated: `FactoryDocument`, `BlueprintDocument`, and their persistence are implemented; physical ports and blueprint insertion remain correctly planned |
| `docs/adr/0003-cad-documents-and-blueprints.md` | No implementation note for Phase 4 (only a "Phase 3 implementation note" exists) | Append a new, dated "Phase 4 implementation note" after the existing Phase 3 note, before "## Consequences"; Status and Decision text unchanged |

**Rationale**: Each row above was located by reading every one of the six
files in full during specification and planning, not guessed from the file
names alone — this table is the actual, complete worklist for `/speckit-tasks`,
not a placeholder.

**Alternatives considered**: A broader sweep of every Markdown file in the
repository for any stale phrase. Rejected: `docs/engineering-standards.md`,
`CONTRIBUTING.md`, and `docs/product-scope.md` were checked and contain no
Phase-4-specific "planned"/"next" language about
`FactoryDocument`/`BlueprintDocument`/the blueprint library — widening scope
to touch them would violate spec.md's own SC-005 (exactly six files) for no
benefit.

## Decision 3: What counts as "no longer planned" is exact and mechanically checkable

**Decision**: SC-002 ("zero passages ... describe `FactoryDocument`,
`BlueprintDocument`, or the blueprint library as 'planned,' 'next,' or
'remaining'") is verified by a targeted case-insensitive search across the
six files for `factorydocument`, `blueprintdocument`, and `blueprint
library` co-occurring with `planned`, `next`, or `remain` in the same
sentence/bullet, after the edits — not by unaided re-reading alone. See
quickstart.md for the literal check.

**Rationale**: A purely manual re-read risks missing a passage exactly like
how the original staleness was introduced in the first place (each of
these files was correct when written, then Phase 4 shipped around it).
Making the check mechanical and specific to this closure's own vocabulary
avoids both false confidence and an over-broad grep that would also flag
correctly-still-planned items like physical ports.

**Alternatives considered**: Trust a manual final read-through alone.
Rejected: this is exactly the failure mode that produced the staleness
being fixed right now — a mechanical check costs one command and closes
that gap.

## Decision 4: Data model for this closure tracks documentation claims and cross-references, not application types

**Decision**: `data-model.md` for this feature does not describe a Rust
struct, persistence format, or domain type (there is none — this closure
ships no code). It instead models the two kinds of thing this closure must
get right: **phase-status claims** (does a passage correctly say Phase 4 is
integrated and Phase 5 is next?) and **cross-references** (does every path
or document link this closure touches or adds still resolve?).

**Rationale**: `/speckit-plan`'s Phase 1 step still asks to "extract
entities from feature spec... if data involved." This feature's "data" is
the documentation content itself; modeling it explicitly (which file, which
section, current vs. target status) turns FR-001–FR-006 and SC-002/SC-003/
SC-004 into a concrete, checkable worklist for `/speckit-tasks`, rather than
skipping data-model.md as inapplicable.

**Alternatives considered**: Omit `data-model.md` entirely as N/A.
Rejected: Decision 2's table already shows this closure has real,
enumerable structure worth tracking explicitly; a documentation-content
model is more useful here than an empty file.

## Decision 5: No new ADR

**Decision**: This closure does not create a new ADR. It appends a dated
"Phase 4 implementation note" to the existing ADR 0003, in the same format
already used for the "Phase 3 implementation note" (`docs/adr/0003-cad-documents-and-blueprints.md`,
current lines 28–32).

**Rationale**: ADR 0003 already is the binding decision record for
`FactoryDocument`/`BlueprintDocument`/blueprint-library persistence
(Constitution Principle II). Commits 1–8 implemented that decision; they
did not change it or introduce a new one. Appending a dated implementation
note — exactly the mechanism this project already used once for Phase 3 —
is reconciliation, not a new architectural decision.

**Alternatives considered**: Rewrite ADR 0003's original Decision text to
read as already-implemented. Rejected: ADRs are historical decision
records (spec.md Edge Cases); rewriting the original dated Decision would
destroy that history. A new dated note preserves it while adding the
implementation outcome.

## Decision 6: Editing order does not matter; only final cross-file consistency does

**Decision**: The six files may be edited in any order during
implementation. What is verified before commit is the **final** state of
all six together (SC-004: zero contradictions across them), not any
intermediate state.

**Rationale**: This is a single atomic commit (spec.md FR-007); no
intermediate state is ever published or reviewed on its own. Requiring a
specific edit order would add process ceremony with no corresponding
requirement.

**Alternatives considered**: Mandate roadmap.md first, then README/CONTEXT,
then architecture/data-model, then the ADR, as a strict sequence. Rejected:
adds a constraint spec.md never asked for for zero benefit, since only the
final committed state is ever checked.
