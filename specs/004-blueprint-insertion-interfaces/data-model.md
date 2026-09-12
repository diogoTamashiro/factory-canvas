# Phase 1 Data Model: Blueprint Insertion and Exposed Interfaces

This feature adds two new domain-level types (`Interface` on `Blueprint`,
and a batch-insertion operation on `Blueprint`) plus one additive DTO
field (`BlueprintDocumentV1Dto.interfaces`). It changes no existing
catalog, factory-document, or blueprint-library contract.

## `Interface`

A named boundary marker on a `Blueprint`, purely descriptive — see
research.md Decision 3 for its shape's rationale.

```text
Interface
  name: String              // non-blank after trim, unique within its blueprint
  anchor: GridPoint          // relative to the blueprint's own origin space,
                             // same coordinate space as BlueprintNode.relative_origin
  side: Side                 // North | East | South | West
```

- `Side` reuses the exact four-value compass enum
  `docs/data-model.md`'s "Planned physical ports" section already
  defines for the future `PortDefinition.anchor.side` — no new
  vocabulary is introduced.
- An `Interface` carries no `flow`, no `port_type`, and no connection
  state (FR-010) — it is a name plus a location, nothing else.
- Validation (`InterfaceError`):

  ```text
  InterfaceError
    BlankName { index: usize }
    DuplicateName { index: usize }
    NotOnBoundary { index: usize }
  ```

  - `BlankName`: the trimmed name is empty (FR-009).
  - `DuplicateName`: the trimmed name (case-sensitive, matching this
    project's existing name-comparison convention for
    `DocumentMetadata.name`) equals an earlier interface's trimmed name
    in the same list (FR-009).
  - `NotOnBoundary`: `anchor`/`side` does not satisfy the boundary rule
    in research.md Decision 3 (FR-009).
  - `index` identifies which element of the `interfaces` list failed,
    matching this codebase's existing `node_index`/`entity_index`
    convention for pinpointing one item in a validated collection.

## `Blueprint` — extended contract

`Blueprint::from_selection` gains one new parameter and validates it
alongside the existing selection validation, in the same call:

```text
Blueprint::from_selection(
  layout: &FactoryLayout,
  selected_ids: impl IntoIterator<Item = EntityId>,
  id: BlueprintId,
  metadata: DocumentMetadata,
  interfaces: Vec<Interface>,          // NEW — may be empty (FR-011)
) -> Result<Blueprint, BlueprintCreationError>
```

`BlueprintCreationError` gains one new variant that wraps `InterfaceError`
without discarding which interface failed:

```text
BlueprintCreationError
  EmptySelection                        // unchanged
  EntityNotFound { id: EntityId }       // unchanged
  InvalidInterface(InterfaceError)      // NEW
```

`Blueprint` gains a read accessor:

```text
Blueprint::interfaces(&self) -> &[Interface]
```

`Blueprint::from_nodes` (the persistence-facing reconstruction path,
`crate::persistence::blueprint_document`) gains the same new parameter,
already-validated `Vec<Interface>` reconstructed from the decoded
document — the persistence layer, not the domain layer, is responsible
for calling `BlueprintNode`/`Interface` reconstruction from DTOs, exactly
as it already is for nodes today (research.md Decision 4: no separate
"add interface later" path exists, so `from_nodes` only ever receives
interfaces the file itself already recorded at its original save time).

## Insertion

A single, atomic, all-or-nothing operation — see research.md Decision 2
for why it lives on `Blueprint`, not `FactoryLayout`.

```text
Blueprint::insert_into(
  &self,
  layout: &mut FactoryLayout,
  insertion_point: GridPoint,
  first_id: u64,
) -> Result<u64, BlueprintInsertionError>
```

- **Insertion point**: an explicit `GridPoint` parameter. Each node's
  absolute target origin is `insertion_point + node.relative_origin`,
  computed inside `insert_into` itself — not pre-translated by the
  caller. (**Deviation from the original plan, discovered during
  implementation**: an earlier draft of this section described the
  insertion point as passed "implicitly" via origins the caller had
  already translated before calling, with `insert_into` reading
  `self.nodes()` directly with no location parameter of its own. That
  design cannot work: `BlueprintNode::relative_origin` is a private
  field with only a read accessor, and `self.nodes()` is owned by the
  `Blueprint` being inserted, not something the caller can rewrite in
  place before the call. `insert_into` must take the insertion point as
  its own parameter to compute absolute origins at all — this is what
  T017 actually implements, and what every quickstart.md scenario
  already described in practice ("Click a grid location on the
  canvas").)
- **On success**: every node became a new `BlockInstance` inside
  `layout`, each keeping its buildable, rotation, and configured product
  exactly (FR-003), with a fresh sequential ID starting at `first_id`
  (FR-002); the return value is `first_id + self.nodes().len() as u64`,
  the caller's new `next_entity_id`.
- **On failure**: `layout` is completely unchanged (FR-004, FR-005,
  FR-006) — not even partially mutated — and the specific node and
  reason are identified by the returned error:

  ```text
  BlueprintInsertionError
    CoordinateOverflow { node_index: usize }
    EntityIdsExhausted
    BuildableNotFound { node_index: usize, buildable_id: BuildableId }
    ProductNotFound { node_index: usize, product_id: ProductId }
    UnsupportedProduct {
      node_index: usize,
      buildable_id: BuildableId,
      product_id: ProductId,
    }
    OutOfBounds { node_index: usize }
    Collision { node_index: usize, conflicting_id: EntityId }
  ```

  This mirrors `PlacementError`'s existing variant shape one-for-one
  (`BuildableNotFound`/`ProductNotFound`/`UnsupportedProduct`/
  `OutOfBounds`/`Collision`), plus two variants specific to a *batch*
  operation that a single `place()` call never needs:
  `CoordinateOverflow` (translating a node's relative origin by the
  insertion point would not fit in `i32`) and `EntityIdsExhausted` (the
  allocator cannot supply `self.nodes().len()` fresh IDs starting at
  `first_id` without overflowing `u64`).
- **Post-insertion independence** (FR-007): the function returns no
  batch identifier, group handle, or back-reference of any kind — its
  only output is the advanced allocator value. Every created
  `BlockInstance` is stored in `layout` exactly like a hand-placed one,
  because it *is* one: `insert_into` calls the same `FactoryLayout::place`
  every hand-placement already goes through, which stores only
  `BlockInstance`'s existing fields (`id`, `buildable_id`, `origin`,
  `rotation`, `production_target`) with no new grouping field added to
  `BlockInstance` itself.

## `BlueprintDocument` — extended schema (still v1)

```text
BlueprintDocument
  schema_version        // unchanged: 1
  catalog_id             // unchanged
  catalog_data_version   // unchanged
  blueprint_id           // unchanged
  metadata               // unchanged
    name
    description (required, nullable)
    created_at
    updated_at
  nodes[]                // unchanged
  interfaces[]           // NEW, additive, defaults to [] when absent (research.md Decision 1)
    name
    anchor: { x, y }
    side: "north" | "east" | "south" | "west"
```

- Decoding: a document with no `interfaces` key at all (every file saved
  before this feature) decodes with an empty `interfaces` list —
  `serde`'s field-level default, not a schema-version branch. A document
  whose `interfaces` array contains an invalid entry (blank/duplicate
  name, off-boundary anchor/side) is rejected exactly like an invalid
  node is today — the whole document fails to decode, all-or-nothing,
  via `Blueprint::from_nodes`'s existing "nothing is trusted from the
  input beyond re-validating it" contract, extended to also re-validate
  interfaces the same way.
- Encoding: every save always writes the `interfaces` array explicitly
  (possibly empty), matching how every other DTO field is always written
  today — no field is ever conditionally omitted based on emptiness.

## Existing types this feature does not change

- `FactoryDocument` and its schema — untouched; insertion only mutates
  an already-open `FactoryLayout` in memory through the same `place()`
  path a hand placement already uses, so the *save* path for the
  destination factory is unaffected.
- `BlueprintLibrary`'s `save()`/`list()` contracts — untouched; insertion
  reads an already-loaded `Blueprint` (from a library entry the player
  already has selected), it does not add a new library-level operation.
- `BlockInstance` — untouched; inserted entities are ordinary
  `BlockInstance` values with no new field.
- The planned catalog-level `PortDefinition`/`PortTypeId` system in
  `docs/data-model.md`'s "Planned physical ports" section — untouched
  and still rejected as unsupported schema (FR-012); `Interface`'s
  `Side` enum is a new, independent type in `domain/blueprint.rs`, not a
  reuse of any not-yet-existing catalog type.
