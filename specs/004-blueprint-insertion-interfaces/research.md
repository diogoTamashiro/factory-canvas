# Phase 0 Research: Blueprint Insertion and Exposed Interfaces

No item in Technical Context was marked `NEEDS CLARIFICATION`; every
technology and process choice already follows this project's established
Rust/eframe/egui toolchain and SDD workflow unchanged. This document
instead records the concrete design decisions this plan makes to turn
spec.md's functional requirements into an implementable contract, since
this feature (unlike Commit 9's documentation closure) does introduce new
domain types and behavior.

## Decision 1: `interfaces[]` is an additive optional field within schema v1, not a new schema version

**Decision**: Add `interfaces: Vec<InterfaceDto>` to `BlueprintDocumentV1Dto`
as a field with `#[serde(default)]`, so every blueprint file saved before
this feature (which has no `interfaces` key at all) still decodes
successfully with an empty interface list, while every blueprint saved
after this feature explicitly writes the field on every encode.
`BLUEPRINT_DOCUMENT_SCHEMA_VERSION` stays `1`.

**Rationale**: `BlueprintDocumentV1Dto` uses `#[serde(deny_unknown_fields)]`,
and every blueprint file the player already has on disk was encoded
without an `interfaces` key. If the new field were required (no
`#[serde(default)]`), every pre-existing saved blueprint would fail to
decode after this feature ships — a real backward-compatibility break for
data the player already has, which `docs/engineering-standards.md`'s ACID
"Consistency" rule and this project's existing migration
convention ("Files have a `schema_version`; migrations preserve the
original until the result has been validated") both argue against
introducing without a genuine reason. Nothing about this feature changes
what makes a v1 document valid in a way that would break round-tripping
an old file — it only adds new, optional information — so bumping to v2
and writing a migration would be process overhead with no corresponding
compatibility need, which YAGNI already rules out.

**Alternatives considered**:
- *Bump to schema_version 2 with a v1→v2 migration*: rejected — solves a
  compatibility problem that does not exist here (v1 is not being
  redefined, only extended), and would require inventing the "migration
  in memory when supported" step `docs/data-model.md`'s "Implemented
  persistence" section already documents as designed-into-the-sequence
  but not yet exercised, purely to satisfy process rather than a real
  incompatibility.
- *Require `interfaces` unconditionally, forcing existing files to be
  hand-edited or lost*: rejected outright — violates ACID Consistency and
  this project's repeated commitment (Commits 7-9) to never silently
  discard or corrupt a player's existing local data.

**Consequence for existing tests**: `tests/blueprint_document_codec.rs`'s
`decoding_rejects_a_document_with_unknown_top_level_field` test currently
proves its point by tampering an encoded document to add an `"interfaces"`
key and expecting rejection (line 129). Once `interfaces` is a real,
known field, that specific tamper stops being a valid "unknown field"
probe and the test must switch to a different, still-genuinely-unknown
field name (e.g., `"unknown_top_level_field"`) to keep testing the same
invariant (`deny_unknown_fields` still rejects anything that is not one
of the document's real fields).

## Decision 2: Insertion lives as `Blueprint::insert_into`, not as a new `FactoryLayout` method

**Decision**: Implement batch insertion as
`Blueprint::insert_into(&self, layout: &mut FactoryLayout, first_id: u64) -> Result<u64, BlueprintInsertionError>`
in `src/domain/blueprint.rs`. It clones `layout`, translates each node to
an absolute `BlockInstance` at the chosen insertion point with a fresh
sequential ID, and calls the clone's already-`pub` `place()` once per
node in ascending node order. The first `place()` failure aborts the
whole attempt (the clone is discarded, `layout` is never touched); if
every node places successfully, `*layout` is replaced by the validated
clone and the function returns `first_id + node_count` as the caller's
new `next_entity_id`.

**Rationale**: `FactoryLayout::place` already performs every check
FR-002 through FR-006 require per entity — duplicate-ID rejection
(structurally impossible here since IDs are freshly and sequentially
allocated), buildable/product referential validity against the
*destination's* active catalog, and bounds/collision — exactly the
"same referential-validity rules" FR-006 asks insertion to reuse. Adding
a second, parallel validation path inside `FactoryLayout` itself would
duplicate logic `place()` already owns, which this project's DRY-in-
moderation and pragmatic-SOLID rules both discourage ("keep one source
of truth for rotation, footprint, and collision rules"; "no layer exists
only to follow SOLID"). Placing the orchestration in `blueprint.rs`
instead of `layout.rs` also preserves the existing one-directional module
dependency (`blueprint.rs` already imports from `layout.rs` for
`EntityId`/`FactoryLayout`) rather than introducing a reverse import of
blueprint types into `layout.rs`, which would create a domain module
depending both ways on a sibling for no structural reason.
`FactoryLayout` already implements `Clone`, and the existing
`replace_instances_atomically` private helper already establishes the
same "clone, mutate/validate, commit the clone only on total success"
pattern this feature reuses at a coarser (whole-blueprint) grain.

**Alternatives considered**:
- *A new `FactoryLayout::insert_blueprint_nodes` method taking
  `&[BlueprintNode]` directly*: rejected — would make `layout.rs` depend
  on `blueprint.rs`'s types, reversing the existing dependency direction
  for no benefit, since every check it would need is already reachable
  through the existing `pub place()` from the `blueprint.rs` side.
- *Orchestrate insertion entirely in `egui_app.rs` (the UI layer),
  calling `layout.clone()`/`place()` directly without a new domain
  function*: rejected — this project's pragmatic-SOLID rule keeps the
  domain, not the UI shell, as the authority for a transactional
  multi-entity domain operation; `egui_app.rs` already delegates single-
  instance placement logic to `FactoryLayout::place` rather than
  reimplementing bounds/collision itself, and batch insertion is the
  same category of operation at a larger grain.

**New error type**: `BlueprintInsertionError` mirrors the shape
`PlacementError` already uses (index-qualified variants, same as
`FactoryDocumentError`'s `entity_index` and `BlueprintDocumentError`'s
`node_index` conventions elsewhere in this codebase) — see data-model.md
for its exact variants.

## Decision 3: Interface boundary location reuses the planned physical-port's `anchor`/`side` shape

**Decision**: An `Interface`'s boundary location is expressed as
`{ anchor: GridPoint, side: Side }`, where `Side` is the same
`North | East | South | West` compass enum `docs/data-model.md`'s
"Planned physical ports" section already defines for the future
catalog-level `PortDefinition`. "On the selection's outer boundary"
means: `anchor` lies within the blueprint's own footprint bounding
rectangle (the union of every node's rotated footprint, computed with
the same union-of-occupied-rects math `FactoryLayout`'s private
`OccupiedRect::union` already implements for multi-selection rotation
pivots, applied here to a blueprint's nodes instead of a factory's placed
instances), and `side` points outward at that tile — `West` only valid
when `anchor.x` sits on the rectangle's left edge, `East` only on the
right edge, `North` only on the top edge, `South` only on the bottom
edge, with a corner tile allowing either of its two adjacent outward
sides.

**Rationale**: Reusing the exact shape already documented for the
future physical-port system keeps a single boundary-location vocabulary
across the product instead of inventing a second, incompatible one this
early — `docs/data-model.md` already establishes `anchor`/`side` as this
project's chosen representation for "a location on a footprint's edge,"
and FR-010/FR-012 already require blueprint interfaces to stay a
*distinct, simpler* concept (no `flow`, no `port_type`, no catalog
schema change) rather than a smaller reinvention of the same idea with
different field names. The bounding-rectangle definition of "boundary"
is the simplest rule that satisfies FR-009 ("MUST reject a boundary
location that does not lie on the selection's own outer boundary")
without requiring a full occupancy-grid computation for footprints that
are not solid rectangles — consistent with YAGNI, since nothing in
spec.md asks interfaces to be validated against actual buildable-level
adjacency (that remains explicitly out of scope, FR-012).

**Alternatives considered**:
- *A single relative `GridPoint` with no side*: rejected — cannot express
  which of a corner tile's two plausible outward faces the player means,
  and diverges from the vocabulary `docs/data-model.md` already commits
  the eventual physical-port system to, which this feature explicitly
  says it complements (spec.md Assumptions).
- *Full per-tile occupancy boundary tracing (true polygon boundary of
  possibly non-rectangular footprints)*: rejected as unnecessary
  complexity for a purely descriptive marker with zero validated
  adjacency or flow; the bounding-rectangle simplification cannot be
  observably wrong to a player, since nothing downstream currently reads
  or enforces true occupancy-level adjacency.

## Decision 4: Interfaces can only be marked at initial blueprint save, never added to an already-saved blueprint

**Decision**: `Blueprint::from_selection` gains one new parameter,
`interfaces: Vec<Interface>`, validated internally (blank/duplicate
name, boundary location) as part of the same call that already
validates the selection itself. There is no separate "add an interface
to an existing blueprint" operation.

**Rationale**: Spec FR-013 already states this phase "MUST NOT introduce
blueprint editing" — and adding an interface to a blueprint already
persisted in the library is, by definition, editing that blueprint's
document. Spec FR-008's own wording ("mark a location on a **to-be-saved**
selection's outer boundary") already scopes marking to the save moment,
and every one of User Story 2's four Acceptance Scenarios describes
either marking during a save or reading an already-saved blueprint's
interface names — none describes mutating an already-saved blueprint's
interface list. This closes an otherwise-underspecified point (User
Story 2's narrative mentions "or while reviewing a blueprint already in
the library," which this decision resolves as "the player can *see* an
interface's name while reviewing," not "the player can add one there")
without needing a `[NEEDS CLARIFICATION]` marker, since FR-008 and FR-013
already jointly force this reading.

**Alternatives considered**:
- *Allow adding/removing interfaces on an already-saved blueprint via a
  new library operation*: rejected — this is blueprint editing, which
  FR-013 explicitly excludes from this phase, and would require a new
  `BlueprintLibrary` write operation beyond `save()`/`list()`, expanding
  scope beyond what spec.md approved.

## Decision 5: Insertion point selection reuses the existing single-buildable placement/preview interaction

**Decision**: The player chooses a blueprint's insertion point the same
way they already choose where to place a single buildable — click a grid
location while a candidate (here, a whole blueprint instead of a
`BuildableId`) is "armed," reusing the same hit-testing and preview
machinery `egui_canvas.rs`/`egui_app.rs` already implement for
`Option<BuildableId>`-driven placement, generalized to also carry an
"armed blueprint" state.

**Rationale**: Spec Assumptions already establish this ("this phase does
not require a preview-drag or multi-step wizard beyond what the existing
placement/preview interaction pattern already establishes for single
buildables"). Reusing the existing interaction avoids introducing a
second, parallel placement-cursor concept for no product benefit,
consistent with KISS ("choose the solution with the fewest concepts") and
DRY-in-moderation.

**Alternatives considered**: A dedicated multi-step insertion wizard
(pick blueprint → preview overlay → drag to reposition → confirm) was
considered and rejected as unnecessary for this phase's scope; nothing in
spec.md's acceptance scenarios requires drag-repositioning before commit,
only a single chosen point.

## Decision 6: ID-exhaustion and coordinate-overflow are insertion failures, not partial insertions

**Decision**: Before committing any change, `Blueprint::insert_into`
must confirm the destination's `next_entity_id` allocator can cover the
*entire* batch (`first_id` through `first_id + node_count - 1`, without
`u64` overflow) and that every node's absolute origin
(`insertion_point + node.relative_origin`) fits in `i32` without overflow
(mirroring the existing `checked_add` pattern `move_instances_by` already
uses). Either failure aborts the whole insertion with a dedicated error
variant, before any `place()` call runs.

**Rationale**: Spec's Edge Cases section already answers this
("Same non-blocking, atomic-rejection behavior as any other insertion
failure: the whole insertion is declined and the factory is left
completely unchanged"). Checking allocator capacity and coordinate
arithmetic up front, rather than discovering an overflow mid-loop, keeps
the atomic-or-nothing contract simple to verify by inspection rather than
by reasoning about partial loop state.

**Alternatives considered**: Silently clamping an out-of-range
coordinate or wrapping an overflowed ID was considered and rejected —
both would silently misplace or misidentify an entity relative to what
the player actually asked for, which contradicts FR-004/FR-005's
"rejected as a whole, leaving the factory completely unchanged" contract.
