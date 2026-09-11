# Feature Specification: Blueprint Insertion and Exposed Interfaces

**Feature Branch**: `004-blueprint-insertion-interfaces`

**Created**: 2026-09-09

**Status**: Draft

**Input**: User description: "fase 5" — Roadmap Phase 5: "Independent
insertion and exposed interfaces. Insert a blueprint as a batch, with new
IDs and atomic failure on bounds/collision. Expose and name physical ports
open at the boundary without assuming a connection." (`docs/roadmap.md`
§"Next active slice and remaining MVP phases"). This is the first
roadmap phase developed on its own spec-kit feature branch, per the
project constitution's "Workflow and Branching" section.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Insert a saved blueprint into the current factory (Priority: P1)

A player has saved one or more blueprints to their local library while
working on earlier factories. While editing the current factory, the
player picks a blueprint from the **BLUEPRINT LIBRARY** sidebar and
inserts it as a new group of instances at a chosen location, so they can
reuse a previously designed production module instead of rebuilding it
tile by tile.

**Why this priority**: This is the entire reason the blueprint library
exists (per ADR 0003's original Decision) and the single capability this
phase's roadmap entry names first. Without it, "save a blueprint" is a
dead end with no way back into a factory.

**Independent Test**: Can be fully tested by saving a blueprint from one
factory, opening or creating a different factory, inserting that
blueprint, and confirming the inserted instances appear as new,
independently selectable entities with fresh IDs, matching the
blueprint's captured buildables, relative layout, rotations, and
per-instance product configuration.

**Acceptance Scenarios**:

1. **Given** a saved blueprint and a factory with enough free, in-bounds
   space at the chosen insertion point, **When** the player inserts the
   blueprint there, **Then** every node in the blueprint becomes a new
   positioned entity in the factory, each with a fresh factory-local ID
   that never collides with an existing entity or with another node in
   the same insertion, positioned at the node's relative offset from the
   chosen insertion point, preserving each node's buildable, rotation,
   and configured product.
2. **Given** a saved blueprint and a chosen insertion point where at
   least one node's resulting position would fall outside the base's
   bounds, **When** the player attempts the insertion, **Then** the
   factory is left completely unchanged (no entity is added, no ID is
   consumed) and the player sees a clear, non-blocking explanation that
   the insertion does not fit.
3. **Given** a saved blueprint and a chosen insertion point where at
   least one resulting position would overlap an existing instance or
   another node in the same blueprint, **When** the player attempts the
   insertion, **Then** the factory is left completely unchanged and the
   player sees a clear, non-blocking explanation that the insertion
   collides with existing layout.
4. **Given** an inserted blueprint's instances, **When** the player
   inspects or selects them, **Then** they behave exactly like any other
   positioned entity — selectable, movable, rotatable, removable, and
   configurable — with no residual link back to the source blueprint or
   to each other as a group after insertion completes.
5. **Given** a blueprint whose recorded catalog identity or data version
   does not exactly match the active catalog, **When** the player
   attempts insertion, **Then** the system re-validates every node's
   buildable and configured product against the active catalog exactly
   as it does for a freshly opened factory document, accepting the
   insertion only if every reference still resolves, and otherwise
   leaving the factory unchanged with a clear explanation.

---

### User Story 2 - Expose and name physical-port interfaces on a saved blueprint (Priority: P2)

While saving a selection as a blueprint, or while reviewing a blueprint
already in the library, a player marks specific physical ports open at
the selection's boundary and gives each one a short, human-readable name,
so that later — when the blueprint is inserted or when the player is
simply deciding which blueprint to reuse — they can tell at a glance
which of a module's edges are meant to receive or send something,
without the system asserting that any actual connection or flow exists.

**Why this priority**: This is the second capability this phase's
roadmap entry names, and the ADR's original Decision already commits to
it ("treat every physical port exposed at a selection boundary as a
nameable blueprint interface without asserting a confirmed connection or
flow"). It depends on nothing from User Story 1 and can be built,
tested, and demonstrated on its own, but delivers materially less value
without insertion, since today nothing else in the product reads or acts
on a named interface.

**Why not a run-of-project prerequisite**: physical ports themselves
(`PortDefinition`, `PortTypeId`, `anchor`/`side`/`flow` — per
`docs/data-model.md`'s "Planned physical ports" section) are a
buildable-level catalog concept that does not exist in the runtime
catalog schema yet. This story defines only the blueprint-level
interface — a named boundary marker on a saved selection — and does not
introduce buildable-level port definitions, connection validation, or
flow semantics; those remain explicitly out of scope for every phase to
date.

**Independent Test**: Can be fully tested by saving a selection as a
blueprint, marking one or more boundary locations as interfaces with a
distinct name each, reloading the blueprint library, and confirming each
interface's name and boundary location are preserved and visible without
implying any connection state.

**Acceptance Scenarios**:

1. **Given** a canvas selection about to be saved as a blueprint,
   **When** the player marks a point on the selection's outer boundary
   and assigns it a non-blank name, **Then** the saved `BlueprintDocument`
   records that interface's name and its boundary location relative to
   the blueprint's own footprint.
2. **Given** a blueprint with zero marked interfaces, **When** it is
   saved and later reloaded, **Then** it is a completely valid blueprint
   with an empty interface list — marking an interface is optional, never
   required.
3. **Given** two interfaces on the same blueprint, **When** the player
   assigns them the same name, **Then** the system rejects the duplicate
   name with a clear explanation and keeps both interfaces at their
   prior, distinct names.
4. **Given** a blueprint with one or more named interfaces, **When** the
   player views that blueprint's entry in the **BLUEPRINT LIBRARY**
   sidebar or a insertion-selection view, **Then** each interface's name
   is visible without any claim that it is connected to anything.

---

### Edge Cases

- What happens when a blueprint is inserted into a factory using a
  different catalog than the one recorded on the blueprint, and the
  blueprint references a buildable or product the active catalog does
  not have at all (not just a version mismatch)? → Same answer as
  Acceptance Scenario 5 of User Story 1: the insertion is rejected as a
  whole, the factory is left unchanged, and the explanation names the
  mismatch category without leaking raw identifiers beyond what the
  existing `notice_text` redaction conventions already allow.
- What happens if the player attempts to insert a blueprint into an
  empty factory (no base chosen yet, or a factory with zero existing
  entities)? → Insertion behaves the same as with a partially filled
  factory: the only constraints are the base's own bounds and the
  blueprint's own internal (self-)collisions, since there are no other
  entities to collide with.
- What happens if two or more nodes inside the same blueprint would
  themselves overlap after being translated to the insertion point? → An
  internally colliding blueprint cannot have been saved in the first
  place (existing `Blueprint::from_selection`/`from_nodes` validation
  already prevents overlapping nodes at capture and load time), so this
  case cannot arise from a validly loaded blueprint; insertion does not
  need to re-check node-to-node overlap within the same blueprint, only
  against the destination factory's existing instances.
- What happens if the player marks more boundary interfaces on a
  blueprint than the number of tiles on its perimeter allows, or picks a
  location that is not actually on the selection's outer boundary? →
  Rejected at the moment of marking, with a clear explanation; the
  blueprint's interface list is left unchanged.
- What happens to a blueprint's named interfaces after it is inserted
  into a factory? → Per this phase's explicit scope boundary (see
  FR-010), interfaces are not yet connected to anything or reconciled
  against a destination factory's own layout; they are informational
  metadata on the source blueprint only, until a later phase gives them
  operational meaning.
- What happens when inserting a blueprint would need an entity ID beyond
  what the factory's current allocator can produce? → Same non-blocking,
  atomic-rejection behavior as any other insertion failure: the whole
  insertion is declined and the factory is left completely unchanged
  (the destination factory keeps whatever allocator ceiling it already
  has today; this phase does not change allocator capacity).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST let a player insert a previously saved
  blueprint into the currently open factory as a single, atomic
  operation, given a chosen insertion point.
- **FR-002**: Inserting a blueprint MUST create one new, independent
  positioned entity per blueprint node, each assigned a fresh
  factory-local identifier that never reuses an identifier already
  present in the destination factory, and never collides with another
  identifier newly assigned within the same insertion.
- **FR-003**: Each inserted entity MUST preserve the source node's
  buildable, rotation, and configured product exactly, positioned at the
  node's relative offset from the chosen insertion point.
- **FR-004**: An insertion MUST be rejected as a whole, leaving the
  factory completely unchanged (no entity added, no identifier
  consumed), if any resulting entity's position would fall outside the
  active base's bounds.
- **FR-005**: An insertion MUST be rejected as a whole, leaving the
  factory completely unchanged, if any resulting entity's position would
  overlap an existing entity in the destination factory or another
  entity being inserted in the same operation.
- **FR-006**: An insertion MUST re-validate every node's buildable and
  configured product against the destination factory's active catalog
  before creating any entity, using the same referential-validity rules
  `FactoryLayout::place` and `Blueprint::from_nodes` already apply
  elsewhere, and MUST reject the whole insertion (leaving the factory
  unchanged) if any reference does not resolve.
- **FR-007**: After a successful insertion, the newly created entities
  MUST be ordinary positioned entities indistinguishable from
  hand-placed ones — independently selectable, movable, rotatable,
  reconfigurable, and removable — with no persisted grouping, batch
  identifier, or back-reference to the source blueprint or to each
  other.
- **FR-008**: The system MUST let a player mark a location on a
  to-be-saved selection's outer boundary as a named interface and
  persist that name and boundary location as part of the resulting
  `BlueprintDocument`.
- **FR-009**: The system MUST reject an attempt to assign a blank name
  or a name already used by another interface on the same blueprint,
  and MUST reject a boundary location that does not lie on the
  selection's own outer boundary.
- **FR-010**: A blueprint interface MUST NOT assert, validate, or imply
  any confirmed connection, adjacency, or flow to anything else — marking
  and naming an interface is purely descriptive metadata on the
  blueprint itself. This phase does not reconcile a blueprint's
  interfaces against a destination factory's layout in any way at
  insertion time or afterward.
- **FR-011**: A blueprint with zero marked interfaces MUST remain a
  completely valid, insertable blueprint; marking an interface is
  optional.
- **FR-012**: The system MUST continue to reject, as unsupported schema,
  any buildable-level physical-port definition
  (`PortDefinition`/`PortTypeId`/anchor/side/flow) — this phase
  introduces only the blueprint-level named-interface marker described
  in FR-008 through FR-011, not the catalog-level port system described
  in `docs/data-model.md`'s "Planned physical ports" section.
- **FR-013**: This phase MUST NOT introduce blueprint editing, deletion,
  renaming, import, or export — the local blueprint library remains
  limited to save, list, and (as of this phase) insert.

### Key Entities *(include if feature involves data)*

- **Blueprint interface**: A named boundary marker on a saved
  `BlueprintDocument`, identifying one location on the blueprint's own
  outer boundary the player considers a meaningful point of contact
  (e.g., "Input", "Power in", "Output A"). Carries a non-blank, unique
  (within its blueprint) name and a boundary location. Does not carry a
  port type, flow direction, or connection state — those remain
  catalog-level physical-port concepts out of scope for this phase.
- **Insertion**: A single, atomic, all-or-nothing operation that
  translates every node of one blueprint into new, independent,
  freshly-identified positioned entities inside a target factory at a
  chosen point, or changes nothing at all if any part of that translation
  would be invalid.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can reuse a previously saved production module by
  inserting it into a new factory in a single action, without manually
  re-placing any of its individual buildables.
- **SC-002**: 100% of insertion attempts that would place any entity out
  of bounds or in collision with existing layout are rejected without
  altering the destination factory in any way — verified by the
  factory's complete entity set, identifiers, and allocator state being
  byte-for-byte unchanged after a rejected attempt.
- **SC-003**: 100% of successfully inserted entities carry an
  identifier that did not exist in the destination factory before the
  insertion and does not collide with any other entity created in the
  same insertion.
- **SC-004**: A player can mark and name at least one interface on a
  blueprint and see that name preserved after the application is closed
  and reopened, with zero implied connection state attached to it.
- **SC-005**: Attempting to assign a duplicate or blank interface name
  is rejected 100% of the time without altering any of the blueprint's
  existing interfaces.

## Assumptions

- "Insertion point" means a single grid location the player chooses on
  the canvas (analogous to how placement already anchors a buildable's
  footprint today); this phase does not require a preview-drag or
  multi-step wizard beyond what the existing placement/preview
  interaction pattern already establishes for single buildables.
- Fresh entity identifiers for an insertion are drawn from the
  destination factory's own existing `next_entity_id` allocator, the
  same allocator every other placement already uses — this phase does
  not introduce a second identifier scheme.
- "Boundary location" for an interface is expressed in the same
  selection-relative coordinate space `Blueprint::from_selection` already
  establishes for nodes, so no new coordinate system is introduced.
- Interfaces are stored per-blueprint (on the `BlueprintDocument`), not
  per-node; a single interface names one boundary point of the whole
  module, not a specific buildable's own port.
- This phase does not implement the catalog-level physical-port system
  (`PortDefinition`, `PortTypeId`, flow/adjacency validation) described
  in `docs/data-model.md`'s "Planned physical ports" section; that
  remains a distinct, not-yet-scheduled increment, consistent with ADR
  0003 treating ports and blueprint interfaces as related but separate
  concerns.
- Undo/redo (roadmap Phase 6) remains out of scope; a rejected insertion
  is prevented outright rather than being applied and then undone.
