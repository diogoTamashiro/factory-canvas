# Factory Canvas architecture

## Status

The default binary already uses `eframe/egui`; the iced implementation remains frozen in a legacy binary while the migration advances in small slices.

## Goal

Keep layout rules independent of the UI and persistence so that all geometry can be tested without opening a window.

## Allowed dependencies

```text
┌──────────────┐      ┌────────────────┐
│ UI (egui)    │ ───> │                │
└──────────────┘      │  pure domain   │
┌──────────────┐      │                │
│ persistence  │ ───> │                │
└──────────────┘      └────────────────┘
```

- the domain does not import egui, the filesystem, a database, or Python;
- the UI calls public domain operations;
- persistence converts between a versioned file and the domain;
- the UI and persistence do not depend on each other;
- no circular dependencies.

## Binaries during migration

- `factory-canvas` is the default binary and starts `src/egui_main.rs` with eframe 0.36, the `glow` backend, and AccessKit;
- `factory-canvas-legacy` compiles `src/main.rs` without adapting iced to the new shell;
- both can be verified during the transition, but only the egui binary receives new editor features;
- the egui state owns a `FactoryLayout`, `SelectedSet`, and `CanvasState`; palette, placement, selection, removal, enumeration, and drawing consult only public domain APIs.

## Current structure and incremental target

```text
src/
  egui_main.rs             # default eframe entry point
  egui_app.rs              # shell, state, palette, feedback, and modals
  egui_app_tests.rs        # editor transition tests
  egui_canvas.rs           # fit, viewport, marquee, focus, and painter
  selected_set.rs          # ordered set and selection modes
  catalog_loader.rs        # strict embedded/filesystem catalog adapter
  main.rs                  # legacy iced entry point
  lib.rs
  domain/
    geometry.rs            # point, dimension, and rotation
    catalog.rs             # typed IDs, definitions, indexes, and validation
    layout.rs              # Catalog/BaseId-backed layout and atomic editing
  persistence/             # future versioned JSON and atomic save
  history.rs               # future undo/redo command
```

The canvas was extracted when it gained a real, separable responsibility: transforming coordinates and drawing. `egui_app.rs` continues to own state transitions; `egui_canvas.rs` does not modify the layout or replicate spatial validation. New components will be extracted only when another concrete boundary exists.

## CAD, documents, and runtime data

The runtime catalog is implemented. Factory and blueprint documents remain the next layers:

```text
CatalogManifest + modular data ──> static definitions
                                      │
FactoryDocument ────────────────────> positioned entities
                                      │
BlueprintDocument ──────────────────> relative copies and exposed interfaces
```

- catalog schema v1 has `schema_version` and a SemVer `data_version`, with regions, PACs, constructible entities, and products;
- machines, conveyors, power poles, and future components converge on constructible entities that use the same spatial mechanism;
- a positioned entity stores an optional product selected by the user without calculating recipes, connectivity, or throughput;
- port types, physical ports, rules, and their rotation behavior remain planned extensions and are rejected as unknown schema-v1 fields;
- `FactoryDocument` and `BlueprintDocument` are separate, readable local JSON documents that can be migrated by `schema_version`;
- a blueprint saves a literal selection in relative coordinates and creates copies with new IDs when inserted;
- blueprint interfaces represent physical ports open at the selection boundary; they do not assert a conveyor connection or confirmed flow.

The complete contract is in [`docs/data-model.md`](data-model.md), and the decision is recorded in [ADR 0003](adr/0003-cad-documents-and-blueprints.md).

## Runtime catalog boundary

```text
catalog/public/** ── embedded ──┐
                                ├──> strict decoder and validation ──> Catalog
data/catalog/** ─── optional ───┘
```

Both sources pass through the same schema-v1 decoder. A complete valid private package is preferred. A missing private manifest silently selects the public catalog; any other private failure selects the public catalog and keeps a persistent sanitized warning. Modules from the two sources are never combined.

Every DTO boundary rejects unknown fields. The adapter validates required fields, schema and SemVer, typed IDs, dimensions, display metadata, symbols, unique paths, canonical path containment, per-kind uniqueness, and all references before it constructs the immutable `Catalog` snapshot. It returns a typed `CatalogLoadError` instead of a partial catalog. User-facing diagnostics omit raw JSON, full private paths, private identifiers, and private values.

`src/catalog_loader.rs` owns filesystem and embedding concerns. The domain receives a validated `Catalog` and continues to import neither the filesystem nor egui. The chosen snapshot is the source for base enumeration, default and bounds, buildable palette, symbols and footprints, preview and painter output, instance resolution, product lookup, and semantic labels.

Catalogs are immutable for the process lifetime. There is no hot reload. Package authors should close the app before changing the manifest or modules and restart it afterward because the separate file reads are not one filesystem-atomic snapshot. Changes under `catalog/public/` require a rebuild and re-embedding.

## Runtime model

```text
Catalog
  metadata: CatalogId + data_version + display_name
  default_base_id: BaseId
  regions[]: RegionDefinition
  bases[]: BaseDefinition
  buildables[]: BuildableDefinition
  products[]: ProductDefinition
  deterministic indexes

FactoryLayout
  catalog: Catalog
  base_id: BaseId
  instances: BTreeMap<EntityId, BlockInstance>

BlockInstance
  id: EntityId
  buildable_id: BuildableId
  production_target: Option<ProductId>
  origin: GridPoint
  rotation: Rotation
```

`FactoryLayout` derives bounds from the selected `BaseDefinition`. `place` checks the duplicate ID first, then resolves the `BuildableDefinition`, validates any configured product against the destination catalog and buildable, and checks the rotated footprint, bounds, and collision before insertion. Every error preserves the previous state.

`instances` exposes only immutable references in ascending `EntityId` order. `instance_at` resolves the instance occupying a `GridPoint` with the same rotated footprint and semi-open bounds used by validation; the UI uses it for hit testing without repeating spatial arithmetic. `remove_instance` returns the removed instance or `None` when the ID does not exist; removal needs no spatial revalidation because it only reduces occupied area and does not change the remaining instances.

`move_instance` and `rotate_instance` take absolute values. Both build a candidate, validate existence, bounds, and collision while ignoring only the current version with the same ID, and replace the map only after success. Single-instance rotation preserves the origin. Failures return `InstanceEditError` without changing the layout. Spatial edits preserve `production_target`.

Occupancy uses semi-open rectangles `[left, right) × [top, bottom)`. Overlap is therefore a collision, while edge contact is allowed without inventing additional clearance. The tracked public buildables are square; the integrated rotation path reuses spatial validation, while the 90°/270° axis swap remains covered by generic geometry tests without inventing a catalog entry.

For two or more instances, `selection_rotation_pivot` calculates the union of the effective footprints, takes its center, and snaps half-tile coordinates toward the top-left grid corner. `rotate_instances_clockwise_about` rotates each rectangle 90° in the coordinate system where Y increases downward, advances its orientation, and validates every destination in a copy without the batch members' old positions. `SelectedSet` stores the accepted pivot while the IDs remain unchanged; a valid move translates it by the same delta, while a changed selection invalidates it and a rejected edit preserves it.

## Invariants

- footprint width and height are in `1..=65535`;
- typed catalog IDs are stable and unique within their namespaces;
- `bounds` derives from the selected `BaseDefinition`;
- an instance references an existing `BuildableDefinition` through `BuildableId`;
- a configured `ProductId` exists in the catalog and is supported by that buildable; placement rechecks both references;
- only instances accepted by `place` enter the collection;
- public enumeration is read-only and deterministic by ID;
- removal changes no remaining instance;
- movement and rotation preserve ID, buildable, and product target; single-instance rotation also preserves the origin, while orbital rotation changes origin and orientation;
- a failed edit preserves the complete previous state;
- the rotated footprint remains inside `bounds`;
- two instances do not occupy the same tile;
- IDs are unique and stable for the life of the layout;
- occupancy derives from instances and is not persisted as a copy.

## Canvas

Current state:

- one painter draws the background, grid, base outline, and instances;
- the active catalog and selected `BaseId` determine the displayed lines and dimensions;
- a tested transform fits the complete grid into the available area, centered and with its aspect ratio preserved;
- `CanvasState` groups the persistent viewport, transient marquee state, and focus request; all painting, preview, and screen-to-grid conversion use the same transformed rectangle;
- major lines every ten tiles keep larger bases readable;
- hit testing converts screen coordinates to `GridPoint`, with exclusive right and bottom edges;
- a normal click on an occupied tile produces `Replace`; `Shift` produces `Add`; `Ctrl` produces `Toggle`; an empty click deselects only without modifiers;
- a marquee starts only with a primary-button drag on an empty tile and no placement tool, normalizes any direction, and includes only entities whose origin belongs to the continuous grid rectangle;
- the clicked empty tile is the top-left origin of a zero-rotation candidate identified by the active `BuildableId`;
- `FactoryLayout::place` validates duplicate ID, buildable and configured product, bounds, and collision before any increment to the UI allocator;
- accepted instances appear in the painter and in a parallel text list for AccessKit, with ID, name, origin, footprint, and rotation; all IDs in `SelectedSet` receive a visual outline, and the list accepts the same selection modifiers;
- exactly one selected capable instance exposes a product chooser in its declared target order; `None` clears it, and the semantic list reports the resolved product name or `no product`;
- single or group removal goes exclusively through `FactoryLayout::remove_instance` after a modal freezes the IDs; `Delete` and `Backspace` open the same request, and cancellation/backdrop/Escape preserve state and IDs;
- text controls and arrow keys move the set one tile, and **Rotate 90°**/`R` rotates one instance at its own origin or two or more around the persistent shared pivot; `move_instances_by` and `rotate_instances_clockwise_about` remove old positions from a copy, validate the final layout, and only then commit atomically;
- with an active placement tool, the canvas derives a visual candidate for the active buildable from the tile under the cursor and draws it semitransparently; it does not query or replicate bounds, collision, or domain validation, and the final click continues through `FactoryLayout::place`;
- the mouse wheel applies cursor-anchored zoom, the middle button applies pan, `Home` restores the full base, and `F`/button frames the union of selected physical footprints with padding; navigation does not change `FactoryLayout`;
- changing an empty base is immediate; with instances, a modal requires confirmation before creating another empty layout.

Next increments:

- Phase 4 adds versioned factory documents, blueprint documents, migrations, and atomic local saves;
- only the visible region will be drawn when a later viewport optimization exists;
- continuous repaint occurs only during interaction or animation.

## Persistence

First format: readable JSON with `schema_version`.

Safe save:

1. validate state;
2. serialize;
3. write a temporary file on the same volume;
4. synchronize when applicable;
5. rename atomically;
6. only then report success.

SQLite may return later for a layout/recent-items library without replacing the user's portable file.

## Frozen components

- `src/main.rs`, compiled as `factory-canvas-legacy`;
- `src/db.rs`;
- `src/screenshot.rs`;
- `src/solver_bridge.rs`;
- `solver/`;
- one `Cell` editor per tile in `src/blueprint.rs`.

They will not be connected to the new domain during the first MVP.
