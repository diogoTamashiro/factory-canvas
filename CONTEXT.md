# Factory Canvas — current context

> Continuity file. In a new conversation, ask the agent to read this file before working on the project.

## Product

Factory Canvas is a native, offline Windows application for planning 2D Arknights: Endfield factory layouts.

The first goal is not to solve or optimize the factory. It is to provide a lightweight CAD tool in which the player arranges an entire factory or production modules on a 2D canvas, while game data and validation rules evolve separately.

## Known bases

There are two layout types in Wuling. Both are square, have no known internal obstacles, and can be upgraded:

- **Main PAC:** 80×80 at the currently confirmed level; earlier levels have not yet been measured;
- **sub-PAC:** 30×30 at the Standard level, 40×40 at Area Expansion I, and 50×50 at Area Expansion II.

The selected level determines the layout bounds. Do not infer the unknown Main PAC progression.

## Confirmed initial catalog

- **Xiranite Power Pole:** Power, 2×2 footprint;
- **Refinery Unit:** Production I, 3×3 footprint;
- **Crushing Unit:** Production I, 3×3 footprint.

All three support 0°, 90°, 180°, and 270° rotations and can be used on either base. Regional limits, power range, and ports are not validated by the current editor. Detailed sources are private and remain outside the public repository.

## First MVP

1. Choose a Main Base or Secondary Base.
2. Show the fixed layout bounds.
3. Place blocks with defined width and height.
4. Move, rotate, and remove blocks.
5. Prevent collisions and out-of-bounds placement.
6. Navigate the whole factory or subsets with pan, zoom, focus, and multi-selection.
7. Save the factory and production-module blueprints locally.

Belt connectivity, recipe validation, throughput, CP-SAT, capture, and OCR remain outside the first CAD increment. Buildable entities, physical ports, and product configuration are planned data contracts.

## Confirmed stack

- Windows desktop only;
- Rust;
- migration from `iced` to `eframe/egui`;
- a custom 2D canvas;
- `serde` with versioned JSON initially;
- offline, with no AI at runtime.

## Repository state

The local directory and GitHub repository are named `factory-canvas`, matching the product name.

The current implementation is legacy and contains the iced UI, gallery, capture, planner, and Python/OR-Tools bridge. It will be frozen rather than deleted all at once.

## Engineering rules

- KISS and YAGNI;
- moderate DRY and pragmatic SOLID;
- a domain with no UI or I/O dependency;
- ACID persistence operations;
- TDD for the domain and persistence;
- minimal dependencies;
- versioned documentation and ADRs;
- small, verifiable, narrated commits;
- code that remains maintainable without AI.

Details: `docs/engineering-standards.md`.

## New domain

- `src/domain/geometry.rs` contains `GridPoint`, `GridSize`, `Rotation`, and footprint transformations;
- `src/domain/base.rs` contains the four confirmed selectable templates: an 80×80 Main PAC and 30×30, 40×40, and 50×50 sub-PACs;
- `src/domain/catalog.rs` contains the three confirmed initial blocks, with stable IDs, names, categories, and footprints;
- `src/domain/layout.rs` contains `EntityId`, instances, per-tile occupancy queries, and atomic editing: place, enumerate, remove, move, and rotate without violating bounds or collisions; `selection_rotation_pivot` derives the physical center snapped to the grid and `rotate_instances_clockwise_about` validates the entire orbital batch before committing it;

The domain is independent of the UI and was developed with RED → GREEN tests.

## New interface

- `src/egui_main.rs` starts the default `factory-canvas` binary with `eframe/egui`;
- `src/selected_set.rs` keeps selected IDs in deterministic order, applies `Replace`, `Add`, and `Toggle` without duplicates, and retains the orbital pivot while selection membership remains unchanged;
- `src/egui_app.rs` owns the `FactoryLayout`, palette, selection, monotonic IDs, feedback, atomic group actions, and destructive confirmations for base changes and single or batch removal;
- `src/egui_canvas.rs` contains fitting, `CanvasState`, the pan/zoom viewport, hit testing, origin-based marquee selection, selection focus, placement preview, and grid/instance painting;
- the three confirmed blocks can be selected in the palette and placed by click with an initial rotation of zero; while a block is active, its translucent footprint appears on the tile under the cursor without prevalidating bounds or collisions;
- a plain click replaces the selection, `Shift` adds, `Ctrl` toggles, and a left-button drag that starts in empty space creates a marquee that considers only instance origins;
- every selected ID is highlighted; controls and arrow keys move the group by one tile, **Rotate 90°**/`R` preserves the origin of one instance or rotates two or more orbitally, and **Remove block(s)**, `Delete`, or `Backspace` opens one confirmation for the frozen IDs;
- `FactoryLayout::place` remains the placement authority; `move_instances_by`, `selection_rotation_pivot`, and `rotate_instances_clockwise_about` are the spatial authorities for group edits;
- the sidebar's text list mirrors painted instances semantically with ID, name, origin, footprint, and rotation, and supports the same selection modifiers;
- the mouse wheel zooms at the cursor, the middle mouse button pans the viewport, `Home` frames the entire base, and `F` or **Frame selection** frames the selection's physical bounds; navigation never changes the layout;
- `src/main.rs` remains frozen and is built separately as `factory-canvas-legacy` during the migration.

## Roadmap and next implementation

See `docs/roadmap.md` for the manual sequence, UX decisions, invariants, and gates. The versioned direction is CAD with factory documents, independent blueprints, and a modular data package. The viewport and multi-selection are already integrated; the next slice is the data package and per-entity product configuration.
