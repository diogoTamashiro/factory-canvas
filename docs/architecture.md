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
  main.rs                  # legacy iced entry point
  lib.rs
  domain/
    base.rs                # base templates and confirmed levels
    geometry.rs            # point, dimension, and rotation
    catalog.rs             # block definitions
    layout.rs              # layout, occupancy, validation, and editing
  persistence/             # future versioned JSON and atomic save
  history.rs               # future undo/redo command
```

The canvas was extracted when it gained a real, separable responsibility: transforming coordinates and drawing. `egui_app.rs` continues to own state transitions; `egui_canvas.rs` does not modify the layout or replicate spatial validation. New components will be extracted only when another concrete boundary exists.

## Versioned direction: CAD, documents, and data

The current model remains valid for the minimal compiled catalog. The approved evolution separates three layers without introducing premature production validation:

```text
CatalogManifest + modular data ──> static definitions
                                      │
FactoryDocument ────────────────────> positioned entities
                                      │
BlueprintDocument ──────────────────> relative copies and exposed interfaces
```

- the game-data package has `schema_version` and a SemVer `data_version`, with constructible entities, PACs, products, port types, regions, and rules;
- machines, conveyors, power poles, and future components converge on constructible entities that use the same spatial mechanism;
- a port contains a relative physical anchor, side, flow direction, and type; rotation transforms only the physical anchor and side through a single domain rule, while flow and type remain static logical attributes;
- a positioned entity optionally stores the product selected by the user without calculating recipes, connectivity, or throughput;
- `FactoryDocument` and `BlueprintDocument` are separate, readable local JSON documents that can be migrated by `schema_version`;
- a blueprint saves a literal selection in relative coordinates and creates copies with new IDs when inserted;
- blueprint interfaces represent physical ports open at the selection boundary; they do not assert a conveyor connection or confirmed flow.

The complete contract is in [`docs/data-model.md`](data-model.md), and the decision is recorded in [ADR 0003](adr/0003-cad-documents-and-blueprints.md).

## Initial model

```rust
pub enum BaseKind {
    Main,
    Secondary,
}

pub enum SecondaryLevel {
    Standard,
    AreaExpansionI,
    AreaExpansionII,
}

pub enum BaseTemplate {
    MainCurrent,
    Secondary(SecondaryLevel),
}

pub enum GridSizeError {
    ZeroWidth,
    ZeroHeight,
}

pub struct GridSize {
    width: u16,
    height: u16,
}

impl GridSize {
    pub const fn new(width: u16, height: u16) -> Result<Self, GridSizeError> {
        if width == 0 {
            return Err(GridSizeError::ZeroWidth);
        }
        if height == 0 {
            return Err(GridSizeError::ZeroHeight);
        }
        Ok(Self { width, height })
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }
}

pub enum BlockCategory {
    Energy,
    ProductionI,
}

pub enum BlockTemplate {
    XiranitePowerPole,
    RefineryUnit,
    CrushingUnit,
}

pub struct BlockDefinition {
    id: &'static str,
    display_name: &'static str,
    category: BlockCategory,
    footprint: GridSize,
}

pub struct EntityId(u64);

pub struct BlockInstance {
    id: EntityId,
    template: BlockTemplate,
    origin: GridPoint,
    rotation: Rotation,
}

pub enum PlacementError {
    DuplicateEntityId { id: EntityId },
    OutOfBounds { id: EntityId },
    Collision { id: EntityId, conflicting_id: EntityId },
}

pub enum InstanceEditError {
    EntityNotFound { id: EntityId },
    OutOfBounds { id: EntityId },
    Collision { id: EntityId, conflicting_id: EntityId },
}

pub struct FactoryLayout {
    base_template: BaseTemplate,
    instances: BTreeMap<EntityId, BlockInstance>,
}
```

Base dimensions come from a confirmed data source. `BaseTemplate` identifies the exact selected option: Main PAC in its currently known state, or Standard Sub-PAC, Sub-PAC Expansion I, or Sub-PAC Expansion II. The selected level therefore determines the bounds without inventing the still-unknown Main PAC progression.

`BlockTemplate::ALL` lists exactly the three confirmed initial blocks and resolves each option to an immutable `BlockDefinition`. Power behavior, ports, regional constraints, and recipes are not part of this initial definition.

`FactoryLayout` derives its bounds from `BaseTemplate` instead of keeping a copy that could diverge. `place` validates duplicate IDs, the rotated footprint, bounds, and collision before insertion; every error preserves the previous state.

`instances` exposes only immutable references in ascending `EntityId` order. `instance_at` resolves the instance occupying a `GridPoint` with the same rotated footprint and semi-open bounds used by validation; the UI uses it for hit testing without repeating spatial arithmetic. `remove_instance` returns the removed instance or `None` when the ID does not exist; removal needs no spatial revalidation because it only reduces occupied area and does not change the remaining instances.

`move_instance` and `rotate_instance` take absolute values. Both build a candidate, validate existence, bounds, and collision while ignoring only the current version with the same ID, and replace the map only after success. Single-instance rotation preserves the origin. Failures return `InstanceEditError` without changing the layout.

Occupancy uses semi-open rectangles `[left, right) × [top, bottom)`. Overlap is therefore a collision, while edge contact is allowed without inventing additional clearance. The current confirmed templates are square; the integrated rotation path reuses spatial validation, while the 90°/270° axis swap remains covered by generic geometry tests without inventing a catalog block.

For two or more instances, `selection_rotation_pivot` calculates the union of the effective footprints, takes its center, and snaps half-tile coordinates toward the top-left grid corner. `rotate_instances_clockwise_about` rotates each rectangle 90° in the coordinate system where Y increases downward, advances its orientation, and validates every destination in a copy without the batch members' old positions. `SelectedSet` stores the accepted pivot while the IDs remain unchanged; a valid move translates it by the same delta, while a changed selection invalidates it and a rejected edit preserves it.

## Invariants

- footprint is always greater than zero;
- catalog IDs are stable and unique among the confirmed options;
- `bounds` derives from the selected `BaseTemplate`;
- an instance references an existing `BlockTemplate`;
- only instances accepted by `place` enter the collection;
- public enumeration is read-only and deterministic by ID;
- removal changes no remaining instance;
- movement and rotation preserve ID and template; single-instance rotation also preserves the origin, while orbital rotation changes origin and orientation;
- a failed edit preserves the complete previous state;
- the rotated footprint remains inside `bounds`;
- two instances do not occupy the same tile;
- IDs are unique and stable for the life of the layout;
- occupancy derives from instances and is not persisted as a copy.

## Canvas

Current state:

- one painter draws the background, grid, base outline, and instances;
- `BaseTemplate::bounds()` determines the displayed lines and dimensions;
- a tested transform fits the complete grid into the available area, centered and with its aspect ratio preserved;
- `CanvasState` groups the persistent viewport, transient marquee state, and focus request; all painting, preview, and screen-to-grid conversion use the same transformed rectangle;
- major lines every ten tiles keep larger bases readable;
- hit testing converts screen coordinates to `GridPoint`, with exclusive right and bottom edges;
- a normal click on an occupied tile produces `Replace`; `Shift` produces `Add`; `Ctrl` produces `Toggle`; an empty click deselects only without modifiers;
- a marquee starts only with a primary-button drag on an empty tile and no placement tool, normalizes any direction, and includes only entities whose origin belongs to the continuous grid rectangle;
- the clicked empty tile is the top-left origin of a candidate with zero rotation;
- `FactoryLayout::place` decides duplicate ID, bounds, and collision before any increment to the UI allocator;
- accepted instances appear in the painter and in a parallel text list for AccessKit, with ID, name, origin, footprint, and rotation; all IDs in `SelectedSet` receive a visual outline, and the list accepts the same selection modifiers;
- single or group removal goes exclusively through `FactoryLayout::remove_instance` after a modal freezes the IDs; `Delete` and `Backspace` open the same request, and cancellation/backdrop/Escape preserve state and IDs;
- text controls and arrow keys move the set one tile, and **Rotate 90°**/`R` rotates one instance at its own origin or two or more around the persistent shared pivot; `move_instances_by` and `rotate_instances_clockwise_about` remove old positions from a copy, validate the final layout, and only then commit atomically;
- with an active placement tool, the canvas derives a visual candidate for the active template from the tile under the cursor and draws it semitransparently; it does not query or replicate bounds, collision, or domain validation, and the final click continues through `FactoryLayout::place`;
- the mouse wheel applies cursor-anchored zoom, the middle button applies pan, `Home` restores the full base, and `F`/button frames the union of selected physical footprints with padding; navigation does not change `FactoryLayout`;
- changing an empty base is immediate; with instances, a modal requires confirmation before creating another empty layout.

Next increments:

- the modular data package and product configuration per entity enter the next slice;
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
