# Implementation Plan: Blueprint Insertion and Exposed Interfaces

**Branch**: `004-blueprint-insertion-interfaces` | **Date**: 2026-09-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-blueprint-insertion-interfaces/spec.md`

## Summary

Deliver Phase 5 of the roadmap: (1) insert a saved `Blueprint` into the
currently open factory as a single atomic batch of new, independently
identified `BlockInstance`s, rejecting the whole insertion and leaving the
factory completely unchanged on any out-of-bounds, collision, or
catalog-referential failure; and (2) let a player mark and name physical
interfaces on the outer boundary of a blueprint-in-progress, persisted on
`BlueprintDocument` as purely descriptive metadata with no connection or
flow semantics. Both extend already-implemented Phase 3/4 primitives
(`FactoryLayout`, `Blueprint`, `BlueprintDocument`, `next_entity_id`) rather
than introducing a new domain layer.

## Technical Context

**Language/Version**: Rust, stable toolchain, edition 2021 (unchanged from
every prior phase).

**Primary Dependencies**: None new. Reuses `serde`/`serde_json` (schema
codec), `time` (timestamps — unaffected by this feature), the existing
`eframe`/`egui` UI shell. No new crate is added.

**Storage**: Local JSON files, unchanged mechanism
(`crate::persistence::atomic_file::write_atomically`). This feature adds
one new optional field to the existing `BlueprintDocument` schema-v1 DTO
(see research.md Decision 1) and reads/writes it through the same codec;
it does not introduce a new file format, a new schema version, or a new
storage location.

**Testing**: `cargo test` (unit/integration tests under `tests/` and
inline `#[cfg(test)]` modules), following this project's established
RED→GREEN→REFACTOR discipline for domain and persistence code
(`docs/engineering-standards.md` §TDD). Geometry, ID-allocation,
atomicity, and codec round-trip behavior are unit/integration-tested;
manual validation covers only the canvas insertion interaction itself,
per this project's existing native-UI validation precedent (Commits 7-8).

**Target Platform**: Windows desktop (unchanged); no new platform
surface.

**Project Type**: Desktop application (single Rust crate + two binaries),
unchanged.

**Performance Goals**: N/A beyond existing invariants — insertion and
interface marking are one-shot, user-triggered operations on data already
sized for interactive, single-user editing (tens to low hundreds of
entities per factory, per existing product scope). No new performance
target is introduced.

**Constraints**: Insertion MUST be atomic (spec FR-001, FR-004, FR-005,
FR-006): every entity is validated before any entity is written into the
layout, and any single failure discards the whole candidate batch. Fresh
entity IDs MUST come from the destination factory's existing
`next_entity_id: Option<u64>` allocator (spec Assumptions) — no second ID
scheme. Interfaces MUST NOT imply connection/flow state (spec FR-010) and
MUST NOT introduce the catalog-level physical-port system (spec FR-012).
No file under `catalog/`, `data/`, `.hermes/`, or the historical
`specs/001-blueprint-library/`, `specs/002-blueprint-library-ui/`,
`specs/003-phase-4-closure/` directories may be touched.

**Scale/Scope**: Two independently deliverable user stories (insertion
P1, interfaces P2), touching `src/domain/blueprint.rs` (new
`Blueprint::insert_into`/`BlueprintInsertionError` plus new `Interface`
type and accessor — `src/domain/layout.rs` itself is unchanged, per
research.md Decision 2), `src/persistence/blueprint_document.rs` (one
new optional DTO field plus its own encode/decode tests),
`src/blueprint_library_view.rs` and `src/egui_app.rs` (insertion trigger,
notices, and interface-marking UI), plus their respective test files
under `tests/`. Comparable in size to Commit 5 (blueprint capture) plus
Commit 8 (blueprint library UI) combined.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No new dependency,
  process, or abstraction beyond what `docs/engineering-standards.md`
  already sanctions; the batch-insertion primitive follows the same
  clone-validate-commit pattern `FactoryLayout::replace_instances_atomically`
  already uses for group move/rotate.
- **II. Architectural Decisions Live in ADRs**: PASS. This feature
  implements exactly what ADR 0003's original Decision already commits
  to ("save blueprints as independent copies... treat every physical
  port exposed at a selection boundary as a nameable blueprint interface
  without asserting a confirmed connection or flow") — it does not
  contradict or need to supersede that ADR. Whether the new
  `interfaces[]` field is additive-within-v1 or a new schema version is
  a genuine open design point resolved in research.md Decision 1, not an
  ADR-level architectural reversal.
- **III. Spec-Driven From Commit 7 Onward**: PASS. `spec.md` is written,
  self-validated against the 16-item quality checklist, and approved
  before this plan.
- **IV. Gates Are Still Mandatory**: PASS. All six gates (`cargo fmt
  --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test`, `cargo build --release --bins`, `git diff --check`,
  `hermes verify --skip-start --json --timeout 300`) apply to every
  commit on this branch, same as every prior phase.
- **V. Privacy and Catalog Boundaries**: PASS. No `data/**`,
  `reference/**`, or `.hermes/**` content is read, touched, or
  introduced; no game entity, base, product, region, dimension,
  capacity, or port is invented — insertion only recombines
  already-validated catalog references, and interfaces carry no game
  data at all (just a player-chosen name and a coordinate).

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — Phase 1 introduces exactly two new domain-level
additions (`Interface`/`Side`/`InterfaceError` on `Blueprint`, and
`Blueprint::insert_into`/`BlueprintInsertionError`, both in
`blueprint.rs` per research.md Decision 2 — `FactoryLayout` itself gains
no new method), both directly required by spec FR-001–FR-011, no
speculative field or trait. No dependency change. No decision contradicts
ADR 0001, ADR 0002, or ADR 0003. The Constitution Check above still holds
unchanged post-design.

## Project Structure

### Documentation (this feature)

```text
specs/004-blueprint-insertion-interfaces/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no network API, CLI, or
cross-process service boundary. Its only "contract" is the
`BlueprintDocument` JSON schema's new optional field, which is fully
specified in data-model.md and enforced by codec tests under `tests/` —
the same no-`contracts/` justification `specs/001-blueprint-library/plan.md`
and `specs/002-blueprint-library-ui/plan.md` already used.

### Source Code (repository root)

```text
src/
├── domain/
│   ├── layout.rs              # UNCHANGED — insertion reuses the already-
│   │                          # `pub` FactoryLayout::place() as-is
│   │                          # (research.md Decision 2); no new method
│   ├── blueprint.rs           # MODIFY — new Interface/Side/InterfaceError
│   │                          # types and interface storage on Blueprint
│   │                          # (User Story 2); new BlueprintInsertionError
│   │                          # and Blueprint::insert_into (User Story 1) —
│   │                          # both live here per research.md Decision 2
│   └── document.rs            # UNCHANGED
├── persistence/
│   ├── blueprint_document.rs  # MODIFY — new optional interfaces[] DTO
│   │                          # field, encode/decode support
│   ├── factory_document.rs    # UNCHANGED
│   ├── blueprint_library.rs   # UNCHANGED (list()/save() contracts do
│   │                          # not change; insertion reads an already
│   │                          # -loaded Blueprint, it does not add a
│   │                          # new library operation)
│   └── atomic_file.rs         # UNCHANGED
├── blueprint_library_view.rs  # MODIFY — expose an "Insert" action per
│                              # library row (User Story 1); expose
│                              # interface-marking during the existing
│                              # save-as-blueprint dialog (User Story 2)
├── egui_app.rs                # MODIFY — wire the insertion trigger into
│                              # FactoryCanvasApp (consuming
│                              # next_entity_id the same way
│                              # place_selected_at already does), new
│                              # EditorNotice variants for insertion
│                              # outcomes, canvas interaction for
│                              # choosing an insertion point
├── egui_canvas.rs             # MODIFY only if an insertion preview needs
│                              # a painter hook distinct from the existing
│                              # single-buildable preview (confirmed
│                              # during implementation, not assumed here)
├── egui_main.rs                # UNCHANGED
└── main.rs                     # UNCHANGED (frozen legacy binary)

tests/
├── domain_blueprint.rs         # MODIFY — Blueprint::insert_into tests
│                                # (User Story 1) alongside Interface
│                                # validation tests (User Story 2); both
│                                # live here, not domain_layout_editing.rs,
│                                # since insert_into is a Blueprint method
│                                # that reuses layout.rs unchanged
│                                # (research.md Decision 2)
├── blueprint_document_codec.rs # MODIFY — interfaces[] encode/decode
│                                # round-trip tests; the existing
│                                # "unknown top-level field" test must
│                                # switch its probe field from
│                                # "interfaces" (line 129) to a different
│                                # still-unknown name, since interfaces
│                                # becomes a real field
└── blueprint_library.rs        # UNCHANGED (library save/list contract
                                 # does not change)

catalog/                        # UNCHANGED — no catalog schema change
data/                            # UNCHANGED (ignored, private)
.hermes/                         # UNCHANGED (ignored, private)
specs/001-blueprint-library/     # UNCHANGED — frozen historical SDD trail
specs/002-blueprint-library-ui/  # UNCHANGED — frozen historical SDD trail
specs/003-phase-4-closure/       # UNCHANGED — frozen historical SDD trail
```

**Structure Decision**: Single-crate desktop application, unchanged from
every prior phase. No new module boundary is introduced — insertion
extends `FactoryLayout` (the existing spatial-authority module) and
interfaces extend `Blueprint`/`BlueprintDocument` (the existing
blueprint-authority modules), matching this project's established
pragmatic-SOLID rule that a layer exists only when a real, cohesive
responsibility already owns that data.

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*

## Implementation Deviations

*(Filled in after `/speckit-implement` — every material difference between
this plan and what was actually built, per this project's standing rule
that any plan deviation discovered during implementation is narrated
explicitly.)*

1. **`insert_into`'s `insertion_point` is an explicit parameter, not
   implicit.** data-model.md originally described the insertion point as
   "passed implicitly via how the caller has already translated the
   nodes" — this turned out to be logically impossible:
   `BlockInstance`/`BlueprintNode`'s coordinates are read-only from
   outside `blueprint.rs`, so there is no way for a caller to
   pre-translate nodes before calling a method on the very `Blueprint`
   that owns them. Corrected to
   `insert_into(&self, layout: &mut FactoryLayout, insertion_point: GridPoint, first_id: u64)`
   — matching spec.md's own "chosen insertion point" language and every
   quickstart.md scenario. `data-model.md` was corrected to match before
   `tasks.md` was generated.

2. **`BlueprintLibrary::load(id, catalog) -> Result<Blueprint, ...>` is a
   new library operation.** This plan's Project Structure table said
   `blueprint_library.rs` was UNCHANGED because "insertion reads an
   already-loaded `Blueprint`... it does not add a new library
   operation" — but `BlueprintLibraryListing::entries` only ever holds
   `BlueprintLibraryEntry` (id, name, node count, timestamp,
   compatibility — deliberately not the full node list, by that type's
   own existing doc comment). Actually inserting a chosen library entry
   requires reading its complete `Blueprint` from disk first. Added
   `BlueprintLibrary::load`, reusing the exact same
   `decode_blueprint_document` call `list()` already makes per file, for
   one already-known ID instead of every file in the root — no new codec
   path, no new file format.

3. **`egui_canvas.rs` needed real changes, not just a possible painter
   hook.** This plan flagged `egui_canvas.rs` as "MODIFY only if an
   insertion preview needs a painter hook distinct from the existing
   single-buildable preview." It did: `resolve_grid_interaction`,
   `marquee_start_at`, `update_marquee_frame`, and `show()` all gained a
   new `armed_blueprint: Option<&Blueprint>` parameter alongside the
   existing `selected_block: Option<&BuildableId>`, a new
   `CanvasInteraction::PlaceBlueprint(GridPoint)` variant, and a
   multi-rectangle preview in `placement_preview_for_hover` covering
   every node's footprint (not just one buildable's).

4. **Interface marking is a save-dialog list-and-dropdown UI, not
   canvas-boundary clicking.** research.md's UI-scope note left "a way
   to pick a boundary anchor/side" deliberately open. Implemented as: a
   new `Blueprint::boundary_points(&self, catalog) -> Vec<(GridPoint,
   Side)>` domain method (not in the original data-model.md) computing
   every valid `(anchor, side)` once from the existing `FootprintBounds`
   geometry, then a save-as-blueprint dialog section listing in-progress
   interfaces (name field + a `ComboBox` populated from
   `boundary_points`) rather than new canvas hit-testing. This keeps the
   validated set finite and always valid by construction, and avoids
   introducing graphical boundary-click detection this feature's scope
   did not otherwise require.

5. **One new `EditorNotice` variant beyond the insertion-outcome set
   research.md anticipated**: `BlueprintInsertionUnavailable`, shown when
   a library entry's `load()` fails at insert-request time (e.g. the
   file was deleted from disk since the cached listing was built) —
   distinct from every `BlueprintInsertionError` case, which all assume
   a `Blueprint` was already loaded successfully.

6. **One clippy fix during T039's gate run**:
   `FootprintBounds::contains_boundary_point`'s original
   `x <= self.right - 1` / `y <= self.bottom - 1` bounds check triggered
   `clippy::int_plus_one`; rewritten as the equivalent `x < self.right` /
   `y < self.bottom` (same semantics, `cargo test` unchanged before and
   after).

No deviation altered any FR, AC, or the two explicit out-of-scope
boundaries (FR-012, FR-013); all are implementation-detail corrections
discovered while building against real code, consistent with this
project's established pattern from every prior phase.
