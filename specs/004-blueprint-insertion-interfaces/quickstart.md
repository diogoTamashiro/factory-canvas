# Quickstart: Validating Blueprint Insertion and Exposed Interfaces

This is a validation guide, not an implementation reference — it proves
the feature works end-to-end once built. It intentionally does not
duplicate data-model.md's contracts or contain full test/method bodies;
those live in `tasks.md` and the implementation itself.

## Prerequisites

- Repository built with `cargo build --bins` (debug is enough for manual
  validation; use `cargo build --release --bins` for the release gate).
- At least one blueprint already saved to the local library (any prior
  Commit 8 flow — select instances, **Save as blueprint**), to exercise
  User Story 1's insertion scenarios.
- Automated validation: `cargo test` runs the logical/deterministic
  domain and persistence tests under `tests/` — this covers every
  scenario below except the ones marked **(manual only)**, which depend
  on actually seeing rendered UI and canvas state, per this project's
  established preference for logical/deterministic validation over
  automated GUI capture.

## Scenario: Insert a blueprint into free space (US1, AC1)

1. Run `cargo run --bin factory-canvas`.
2. Choose the blueprint saved in Prerequisites from the **BLUEPRINT
   LIBRARY** sidebar section and start insertion (e.g. an **Insert**
   action on its row).
3. Click a grid location on the canvas with enough free space for the
   whole blueprint.

**Expected**: Every node of the blueprint appears as a new instance,
positioned at its recorded relative offset from the clicked point,
preserving each node's buildable, rotation, and configured product.
Automated: `Blueprint::insert_into` is unit-tested directly (no UI)
asserting the destination layout gains exactly `nodes().len()` new
instances with IDs `first_id..first_id + nodes().len()`, each at the
expected absolute origin/rotation/product. **(manual only** for the
actual visual confirmation on the canvas.)

## Scenario: Out-of-bounds insertion changes nothing (US1, AC2)

1. Select the same blueprint for insertion.
2. Click a grid location near an edge such that at least one node would
   fall outside the base's bounds.

**Expected**: The factory is left completely unchanged — same entity
count, same IDs, same allocator value — and a clear, non-blocking
message explains the insertion does not fit. Automated: assert
`Blueprint::insert_into` returns
`Err(BlueprintInsertionError::OutOfBounds { .. })` and the passed
`&mut FactoryLayout` is byte-for-byte unchanged (same `Debug`/`PartialEq`
snapshot before and after the call) — this is the same "clone that is
never committed on failure" contract already unit-tested for
`FactoryLayout::replace_instances_atomically`. **(manual only** for
seeing the notice text.)

## Scenario: Colliding insertion changes nothing (US1, AC3)

1. Place at least one instance on the canvas by hand.
2. Select the blueprint for insertion and click a point where a node
   would overlap that hand-placed instance.

**Expected**: Same as the out-of-bounds scenario — the factory is
completely unchanged, and the notice names a collision rather than an
out-of-bounds failure. Automated: assert
`Err(BlueprintInsertionError::Collision { .. })` and an unchanged layout.

## Scenario: Inserted instances behave like any other instance (US1, AC4)

1. After a successful insertion (first scenario above), select one of
   the newly inserted instances.
2. Move it, rotate it, and remove it, one action at a time.

**Expected**: Every action succeeds or fails using exactly the same
rules as a hand-placed instance at the same position would — no special
"this came from a blueprint" behavior, no group-move side effect on the
other inserted instances. Automated: after calling `insert_into` in a
test, assert each new `EntityId` independently accepts
`FactoryLayout::move_instance`/`rotate_instance`/`remove_instance` with
no reference to any other newly-inserted ID.

## Scenario: Catalog mismatch is re-validated at insertion time (US1, AC5)

1. Using a blueprint saved against a different catalog identity or data
   version than the currently active one (mirrors how a factory-document
   catalog mismatch is already exercised for `DocumentOpened`), attempt
   insertion.

**Expected**: If every referenced buildable/product still resolves in
the active catalog, insertion succeeds normally; if not, the whole
insertion is rejected with a clear explanation and the factory is
unchanged. Automated: two focused tests — one where a mismatched-version
blueprint still inserts successfully (every reference still resolves),
one where a blueprint referencing a buildable absent from the active
catalog is rejected with
`Err(BlueprintInsertionError::BuildableNotFound { .. })`.

## Scenario: Mark and name an interface while saving a blueprint (US2, AC1)

1. Select one or more instances whose combined footprint has at least
   one edge tile.
2. Open the save-as-blueprint dialog and mark one boundary point,
   naming it (e.g. `"Input"`).
3. Confirm the save.

**Expected**: The save succeeds; reopening or re-listing the blueprint
later still shows the interface name. Automated:
`Blueprint::from_selection` with one valid `Interface` succeeds, and the
resulting `Blueprint::interfaces()` contains exactly that interface;
round-tripped through `encode_blueprint_document`/`decode_blueprint_document`,
the decoded blueprint's `interfaces()` matches the original. **(manual
only** for the actual dialog interaction.)

## Scenario: Zero interfaces is a fully valid blueprint (US2, AC2)

1. Save a blueprint without marking any interface (the normal Commit 8
   flow, unchanged).

**Expected**: The save succeeds exactly as it always has; the blueprint
is fully insertable. Automated: existing Commit 5/7/8 tests that save a
blueprint with no interfaces continue to pass unmodified — this scenario
is a non-regression check, not new behavior (FR-011).

## Scenario: Duplicate or blank interface names are rejected (US2, AC3)

1. Attempt to mark two interfaces on the same blueprint with the same
   name (or a blank/whitespace-only name).

**Expected**: The attempt is rejected with a clear explanation; the
blueprint's existing interfaces (if any were already validly marked) are
unaffected. Automated:
`Blueprint::from_selection` with two interfaces sharing a trimmed name
returns `Err(BlueprintCreationError::InvalidInterface(InterfaceError::DuplicateName { .. }))`;
a blank/whitespace-only name returns
`Err(BlueprintCreationError::InvalidInterface(InterfaceError::BlankName { .. }))`.

## Scenario: Named interfaces are visible without implying a connection (US2, AC4)

1. View the library entry (or an insertion-selection view) for a
   blueprint that has one or more named interfaces.

**Expected**: Each interface's name is visible; nothing in the UI claims
or implies the interface is connected to anything (FR-010). **(manual
only** — this is a rendering/wording check.)

## Mechanical check: an off-boundary or non-boundary anchor/side is rejected

Automated only (no realistic manual reproduction): construct an
`Interface` whose `anchor`/`side` does not satisfy research.md Decision
3's boundary rule (e.g. an interior tile, or a `side` that does not point
outward from that tile's position in the blueprint's bounding
rectangle) and assert
`Err(BlueprintCreationError::InvalidInterface(InterfaceError::NotOnBoundary { .. }))`.

## Mechanical check: pre-existing blueprint files without `interfaces` still decode (Decision 1)

Automated only: decode a blueprint document JSON byte string that has no
`"interfaces"` key at all (representing every file saved before this
feature) and assert it decodes successfully with an empty
`interfaces()` list — proving `#[serde(default)]` backward compatibility
without needing an on-disk fixture from an actual pre-feature build.

## Scope check: `catalog/`, `data/`, `.hermes/`, and every historical `specs/*` directory are unaffected

```bash
git status --short
git diff --stat
```

**Expected**: Only files under `src/`, `tests/`, and
`specs/004-blueprint-insertion-interfaces/` appear across this feature's
commits. Nothing under `catalog/`, `data/`, `.hermes/`, or the historical
`specs/001-blueprint-library/`, `specs/002-blueprint-library-ui/`,
`specs/003-phase-4-closure/` directories is listed.

## Running the required gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

All six must pass before any commit on this feature branch is considered
done, per Constitution Principle IV — unchanged from every prior phase.
Per the constitution's "Workflow and Branching" section, this feature's
commits also merge back to `master` only once every commit on the branch
is complete and independently reviewed, since this is the first phase
developed on its own spec-kit feature branch.
