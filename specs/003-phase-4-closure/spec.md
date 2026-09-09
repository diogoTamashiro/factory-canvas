# Feature Specification: Phase 4 Closure Documentation

**Feature Branch**: `003-phase-4-closure` (directory identifier only — no git
branch is created for this feature; per the project constitution, this is
the final commit of Phase 4, which continues directly on `master`.
Branch-per-phase begins with the next new roadmap phase, Phase 5.)

**Created**: 2026-09-07

**Status**: Draft

**Input**: User description: "Commit 9 of the Phase 4 plan — reconcile all
tracked documentation (roadmap, README, continuity file, architecture, data
model, and the CAD documents/blueprints ADR) with the JSON documents and
blueprint library capability actually delivered across Commits 1-8, mark
Phase 4 as integrated, and identify Phase 5 as the next active slice. This
is a documentation-only commit: no non-documentation file changes. The
phase-wide aggregate review, gates, and native smoke check that follow
publication are process, not part of this specification."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Confirm Phase 4 is documented as complete (Priority: P1)

A maintainer returns to the project — with or without prior conversation
context — and needs to know, from `docs/roadmap.md` alone, that Phase 4
(JSON documents and blueprint library) is fully delivered, what it added,
and which phase to work on next.

**Why this priority**: `docs/roadmap.md` is this project's explicit
"operational document for continuing the project manually or in another
conversation," and the first file its own "Quick manual resumption" section
tells a maintainer to read. If it still describes Phase 4 as upcoming after
Phase 4 is actually done, every other document built on top of it inherits
the same confusion.

**Independent Test**: Read `docs/roadmap.md` top to bottom with no other
context, and correctly state which capabilities Phase 4 delivered and that
Phase 5 is the next active slice.

**Acceptance Scenarios**:

1. **Given** Commits 1-8 of Phase 4 are already published, **When** a
   maintainer reads `docs/roadmap.md`, **Then** it contains an integrated
   "Phase 4" section describing the JSON document and blueprint library
   capability actually delivered, in the same format as the existing
   integrated phase sections.
2. **Given** the new Phase 4 section exists, **When** the maintainer reads
   "Next active slice and remaining MVP phases," **Then** Phase 5
   (independent insertion and exposed interfaces) is clearly identified as
   next, and Phase 4 no longer appears there as upcoming.

---

### User Story 2 - Confirm the product-facing description matches shipped capability (Priority: P2)

A new contributor or resuming maintainer opens `README.md` and `CONTEXT.md`
and finds the current-status description accurate: the editor already
supports saving the current selection as a named local blueprint and
browsing the local blueprint library, not just an unspecified "next MVP
work."

**Why this priority**: These are among the first files a newcomer or
resuming maintainer opens; an inaccurate status line directly misrepresents
what the product can already do. This is lower-severity than the
operational roadmap being wrong (P1) but still user-facing.

**Independent Test**: Read `README.md`'s status line and `CONTEXT.md`'s
"Roadmap and next implementation" section and confirm both state Phase 4 is
integrated with blueprint save/browse available, and both point to Phase 5
as next.

**Acceptance Scenarios**:

1. **Given** Phase 4 is integrated, **When** a reader opens `README.md`,
   **Then** its status line states that the editor supports saving a named
   local blueprint from the current selection and browsing the local
   blueprint library.
2. **Given** Phase 4 is integrated, **When** a reader opens `CONTEXT.md`,
   **Then** its roadmap pointer states Phase 4 is integrated and identifies
   Phase 5 as next, rather than describing `FactoryDocument`/
   `BlueprintDocument` persistence as still upcoming.

---

### User Story 3 - Confirm architecture and data-contract documents have no drift (Priority: P3)

A maintainer checks `docs/architecture.md`, `docs/data-model.md`, and ADR
0003 against what was actually implemented, to confirm none of them still
describes `FactoryDocument`, `BlueprintDocument`, or the blueprint library
as future work, while capabilities genuinely deferred to Phase 5 (blueprint
insertion, physical-port interfaces) are still correctly marked as planned.

**Why this priority**: These are reference/contract documents consulted
less often than the roadmap or README, so their staleness is
lower-severity, but they are the documents a maintainer would trust most
when making a technical decision — leaving them wrong is a correctness risk
for whoever touches this code next.

**Independent Test**: Cross-check every passage in `docs/architecture.md`,
`docs/data-model.md`, and ADR 0003 that currently mentions
`FactoryDocument`, `BlueprintDocument`, or blueprint persistence as
"planned"/"next," and confirm each now correctly reflects
delivered-vs-deferred status.

**Acceptance Scenarios**:

1. **Given** Phase 4 is integrated, **When** a maintainer reads
   `docs/data-model.md`'s status banner and its "Planned factory
   document"/"Planned blueprint document"/"Planned persistence" sections,
   **Then** each now reflects that `FactoryDocument`, `BlueprintDocument`,
   and their persistence are implemented, while physical ports and
   blueprint insertion remain correctly described as planned.
2. **Given** Phase 4 is integrated, **When** a maintainer reads ADR 0003,
   **Then** it contains a dated "Phase 4 implementation note" (matching the
   format of the existing "Phase 3 implementation note") that records what
   was actually built and explicitly states blueprint insertion and
   physical-port interfaces remain deferred to Phase 5, with the ADR's
   original Status and Decision text unchanged.

### Edge Cases

- What happens if a currently-"planned" passage in `docs/architecture.md`
  or `docs/data-model.md` describes a capability that Commits 1-8 only
  partially implemented (e.g., physical ports remain unimplemented while
  `FactoryDocument` is done)? The passage must be split so the implemented
  part is described as delivered and the unimplemented part remains
  explicitly planned, rather than the whole passage flipping to "done" or
  staying "planned" as an all-or-nothing block.
- What happens if updating one document reveals that an already-published
  document's historical content (e.g., an ADR's original dated Decision
  text) turns out to differ from what was actually built? The closure
  corrects only forward-looking/status language (banners, "planned"/"next"
  framing, and an added implementation note); it does not rewrite an ADR's
  original dated Decision or Status, consistent with ADRs being historical
  decision records.
- What happens if this closure's edits to Phase 4's roadmap entry would
  otherwise collide with Phase 5's existing description? Only Phase 4's
  entry moves from "next" to "integrated"; Phase 5's existing description
  in `docs/roadmap.md` is preserved as-is unless it is already inconsistent
  with the newly integrated Phase 4 entry.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `docs/roadmap.md` MUST gain an integrated "Phase 4 — JSON
  documents and blueprint library" section, in the same format and level of
  detail as the existing integrated phase sections, summarizing the
  capability actually delivered by Commits 1-8, and MUST update "Next
  active slice and remaining MVP phases" so Phase 5 is the identified next
  slice.
- **FR-002**: `README.md`'s "Current status" line MUST state that Phase 4
  is integrated and that the editor supports saving the current canvas
  selection as a named local blueprint and browsing the local blueprint
  library.
- **FR-003**: `CONTEXT.md`'s "Roadmap and next implementation" section MUST
  state that Phase 4 is integrated and identify Phase 5 as next, replacing
  its current claim that `FactoryDocument`/`BlueprintDocument` persistence
  is still upcoming.
- **FR-004**: `docs/architecture.md` MUST update every passage that
  currently describes `FactoryDocument`, `BlueprintDocument`, atomic saves,
  or the blueprint library as future, a "next layer," or otherwise
  "planned" so each instead reflects what Commits 1-8 actually implemented,
  while leaving genuinely future items (physical ports, undo/redo, and any
  migration beyond schema v1) marked as still planned.
- **FR-005**: `docs/data-model.md`'s status banner and its "Planned factory
  document" / "Planned blueprint document" / "Planned persistence" section
  headings and bodies MUST be updated to reflect that `FactoryDocument`,
  `BlueprintDocument`, and their persistence are now implemented, while
  physical ports and blueprint insertion remain correctly described as
  still planned.
- **FR-006**: `docs/adr/0003-cad-documents-and-blueprints.md` MUST gain a
  dated "Phase 4 implementation note," in the same format as the existing
  "Phase 3 implementation note," recording what was actually implemented
  and explicitly confirming that blueprint insertion and physical-port
  interfaces — both named in the original Decision — remain deferred to
  Phase 5, without altering the ADR's Status or original Decision text.
- **FR-007**: Every document updated under FR-001 through FR-006 MUST
  remain mutually consistent with the others, MUST NOT describe any
  capability beyond what Commits 1-8 actually shipped, and MUST NOT alter
  any confirmed game-data fact (bases, footprints, dimensions) already
  recorded in those files.
- **FR-008**: This closure MUST NOT modify any file under `src/`, `tests/`,
  `catalog/`, `data/`, `.hermes/`, or the historical
  `specs/001-blueprint-library/` and `specs/002-blueprint-library-ui/`
  directories — Commit 9 is documentation-only, and those SDD trails remain
  frozen records of already-published commits.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A maintainer with zero conversation history can read
  `docs/roadmap.md` alone and correctly state, within 2 minutes, that Phase
  4 is complete and Phase 5 is next, matching the file's own "Quick manual
  resumption" reading order.
- **SC-002**: Zero passages across the six updated documents
  (`docs/roadmap.md`, `README.md`, `CONTEXT.md`, `docs/architecture.md`,
  `docs/data-model.md`, `docs/adr/0003-cad-documents-and-blueprints.md`)
  describe `FactoryDocument`, `BlueprintDocument`, or the blueprint library
  as "planned," "next," or "remaining" once this closure is complete.
- **SC-003**: 100% of documentation cross-references touched by this
  closure resolve to files that exist in the repository.
- **SC-004**: Zero statements across the six updated documents contradict
  each other about what Phase 4 delivered or what remains deferred to
  Phase 5.
- **SC-005**: The commit that closes Phase 4 contains zero changes outside
  the six documentation files named in FR-001 through FR-006.

## Assumptions

- "Documentation closure" for this commit is scoped to the six tracked
  English documents that describe product status, architecture, or data
  contracts and already contain now-stale "planned"/"next" language about
  Phase 4's capabilities: `docs/roadmap.md`, `README.md`, `CONTEXT.md`,
  `docs/architecture.md`, `docs/data-model.md`, and
  `docs/adr/0003-cad-documents-and-blueprints.md`. This mirrors the
  project's own precedent set when Phase 3 closed, where
  `docs/data-model.md`'s status banner and ADR 0003's "Phase 3
  implementation note" were both updated alongside the roadmap, rather than
  only touching `docs/roadmap.md`.
- `docs/engineering-standards.md` is not expected to need changes: it
  documents process/quality rules, not phase-specific product status, and
  Phase 4 did not change any of those rules.
- No new ADR is created for this closure: ADR 0003 already covers the
  document/blueprint decision this phase implements, and appending a dated
  implementation note (as already done once for Phase 3) is this project's
  established way of reconciling an ADR with what was actually shipped.
- This closure documents that blueprint insertion and physical-port
  interfaces remain deferred to Phase 5; it does not implement, stub, or
  further design either capability.
- The final aggregate review, the six required gates, and a native smoke
  check are performed when this commit is prepared, per
  `docs/roadmap.md`'s existing "Engineering workflow per slice" and the
  project constitution's Principle IV; they are not restated here as
  functional requirements since they already bind every commit
  unconditionally.
