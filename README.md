# Factory Canvas

A native, offline Windows desktop application that helps **Arknights: Endfield** players plan factory layouts on a lightweight 2D canvas.

> **Current status:** the domain already provides geometry, a catalog, and validated editing for placing, listing, removing, moving, and rotating blocks. This includes atomic group movement and both in-place and orbital rotation, also applied atomically. The default binary uses `eframe/egui`; it lets users choose among the four confirmed bases and three confirmed blocks, navigate with the mouse wheel, middle mouse button, `Home`, and `F`, see a translucent preview during placement, and select multiple blocks by click, modifiers, or marquee. The next stages add versioned data, per-entity products, local blueprints, and persistence. The `iced` interface remains available only as a legacy binary during the migration.

## First MVP goal

The user chooses a base type and arranges blocks with known dimensions inside a fixed-bounds area.

The MVP must support:

- choosing a **Main Base** or **Secondary Base**;
- clearly seeing the available area bounds;
- searching for and selecting a block;
- placing a block by its fixed footprint (`width × height`);
- selecting one or more instances by stable identity;
- moving, rotating, and removing blocks;
- preventing overlap and out-of-bounds placement;
- panning and zooming;
- undoing and redoing actions;
- saving and reopening the layout locally.

## Base types

| Type | Description | Dimensions |
|---|---|---|
| Main PAC | Larger area for the player's main layout | 80×80 at the currently confirmed level |
| sub-PAC | Upgradeable secondary area | 30×30, 40×40, or 50×50 depending on the expansion |

Both bases are located in Wuling, are square, have no known internal obstacles, and can be upgraded. Earlier Main PAC levels have not yet been measured and will not be inferred. Detailed reference data remains local and is not tracked in the public repository.

## Outside the first MVP

- validation of physical connectivity between ports and belts;
- recipe, throughput, and bottleneck calculations;
- a CP-SAT solver and automatic optimization;
- automatic routing;
- screenshots or OCR;
- automatic game imports;
- network access, accounts, cloud services, or AI;
- 3D rendering or heavy sprites.

The existing Gallery, Planner, Python solver, and capture features remain frozen. Their code will not be deleted at this stage, but these features will not be part of the new product's primary navigation.

## Technical decisions

- **Product:** Factory Canvas
- **Platform:** Windows desktop
- **Language:** Rust
- **Target UI:** `eframe/egui`
- **Rendering:** one custom 2D canvas, never one widget per tile
- **Initial persistence:** versioned local JSON
- **Runtime:** fully offline

Rust and egui were chosen for low resource usage, canvas interaction, a native binary, and code that does not depend on external services.

## Target architecture

```text
egui UI ─────────┐
                 ├──> pure domain
persistence ─────┘
```

The domain knows nothing about egui, SQLite, the filesystem, or Python. See [`docs/architecture.md`](docs/architecture.md).

## Legacy code

The `factory-canvas-legacy` binary still contains:

- the `iced` UI;
- the gallery and screenshot capture;
- the Planner with its Python/OR-Tools bridge;
- the temporary per-tile `Cell` model.

These components remain only to preserve project history while the new editor is built through small, reversible tasks.

## Run the current application

The new shell requires stable Rust. Python 3.11 is required only by the legacy solver.

```powershell
cargo run
```

In the current editor:

1. choose a base;
2. select a block from the palette, check the translucent preview on the tile under the cursor, and click the tile that will become the footprint's top-left origin;
3. click a painted instance or its text row in the sidebar to replace the selection; use `Shift`+click to add an instance and `Ctrl`+click to toggle one;
4. with no placement tool active, drag the left mouse button from an empty area to marquee-select instances whose origins are inside the rectangle; `Shift` adds the matches and `Ctrl` toggles them;
5. use the directional controls or arrow keys to move the selection by one tile; use **Rotate 90°** or `R` to rotate one instance at its own origin or, when two or more are selected, rotate their positions and orientations around the selection center;
6. use **Frame selection** or `F` to focus the complete physical selection; `Home` still frames the entire base;
7. use **Remove block(s)**, `Delete`, or `Backspace` to request removal of the selection, then confirm the action;
8. use the sidebar to check the count, validation result, and text list of instances;
9. use the mouse wheel to zoom at the cursor and drag with the middle mouse button to pan the view.

Bounds and collisions are validated exclusively by the domain. The translucent preview is visual only: it does not indicate acceptance or prevalidate bounds or collisions. Only a click forwarded to `FactoryLayout::place` decides whether placement succeeds. Changing the base while blocks exist requires explicit confirmation and clears the layout only after **Change and clear**. Every selected instance is highlighted on the canvas. For multi-selection rotation, the domain computes the center of the union of the physical footprints and snaps it toward the top-left grid intersection. The selection retains this pivot while its members remain unchanged, and an accepted move translates it by the same delta. Group movement and rotation are atomic domain transactions: any bounds or collision failure preserves the entire group, selection, pivot, and allocator. Removal freezes the selected IDs in one confirmation request; **Cancel**, `Escape`, or the backdrop preserves the layout, selection, and IDs. Pan, zoom, and focus do not alter the layout. History and persistence are not yet part of the egui interface.

To open the frozen iced interface temporarily:

```powershell
cargo run --bin factory-canvas-legacy
```

## Verification

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
```

## Documentation

- [Product scope](docs/product-scope.md)
- [Implementation roadmap](docs/roadmap.md)
- [Architecture](docs/architecture.md)
- [Engineering standards](docs/engineering-standards.md)
- [Contributing](CONTRIBUTING.md)
- [ADR 0001 — Rust + egui](docs/adr/0001-editor-ui.md)
- [ADR 0002 — Factory Canvas product name](docs/adr/0002-product-name-factory-canvas.md)
- [ADR 0003 — CAD documents, blueprints, and versioned data](docs/adr/0003-cad-documents-and-blueprints.md)
- [Data model v1](docs/data-model.md)

## Maintenance principle

> The project must remain understandable, buildable, testable, and maintainable without relying on AI or on the conversation history that produced it.
