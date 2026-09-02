# Factory Canvas

A native, offline Windows desktop application that helps **Arknights: Endfield** players plan factory layouts on a lightweight 2D canvas.

> **Current status:** Phase 3 is integrated. The default `eframe/egui` editor chooses one validated runtime `Catalog` at startup. That catalog drives base choices and bounds, the buildable palette, placement preview, painter, layout resolution, and semantic labels. The tracked public fallback supplies four confirmed bases and three confirmed buildables. A catalog can also declare products that capable buildables expose as a per-instance choice. The next MVP work is versioned factory documents and local blueprints. The `iced` interface remains available only as a frozen legacy binary.

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

## Runtime catalog

The tracked package under `catalog/public/` is the minimal compatibility catalog and is embedded in the binary at build time. A complete package under the ignored `data/catalog/` directory can replace it at startup:

| Private package | Startup result |
|---|---|
| complete and valid | use the private catalog |
| `manifest.json` missing | use the public catalog without a warning |
| any other read or validation failure | use the public catalog and keep a persistent, sanitized warning |

The loader never mixes public and private modules. Schema v1 requires one manifest plus `regions`, `bases`, `buildables`, and `products` modules. Every field is required, unknown fields are rejected, module paths must be safe and unique, and the complete package must pass cross-reference validation before it becomes a `Catalog`. A failed candidate cannot replace any part of the fallback. User-facing diagnostics omit raw JSON, full private paths, private identifiers, and private values. See [`docs/data-model.md`](docs/data-model.md) for the complete contract.

The app reads one catalog at startup and does not hot-reload it. Close the app before changing a package and restart it afterward. Validation is all-or-nothing, but reading the manifest and four modules is not one filesystem-atomic operation. Changes under `catalog/public/` require a rebuild because those files are embedded in the executable.

To add a region, base, buildable, or product, edit the corresponding module while the app is closed, keep existing IDs stable, update `data_version`, and make every new reference resolve before restarting. The package-maintenance checklist and field contract are in [`docs/data-model.md`](docs/data-model.md#updating-a-package).

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
5. when exactly one selected buildable declares products, use the **PRODUCT** chooser to configure one or choose **No product** to clear it; the semantic instance row reports the current choice;
6. use the directional controls or arrow keys to move the selection by one tile; use **Rotate 90°** or `R` to rotate one instance at its own origin or, when two or more are selected, rotate their positions and orientations around the selection center;
7. use **Frame selection** or `F` to focus the complete physical selection; `Home` still frames the entire base;
8. use **Remove block(s)**, `Delete`, or `Backspace` to request removal of the selection, then confirm the action;
9. use the sidebar to check the count, validation result, and text list of instances;
10. use the mouse wheel to zoom at the cursor and drag with the middle mouse button to pan the view.

The chosen runtime catalog is the single source for bases, buildables, footprints, symbols, and product capabilities. Bounds and collisions are validated exclusively by the domain. The translucent preview is visual only: it does not indicate acceptance or prevalidate bounds or collisions. Only a click forwarded to `FactoryLayout::place` decides whether placement succeeds.

Changing the base while blocks exist requires explicit confirmation and clears the layout only after **Change and clear**. Every selected instance is highlighted on the canvas. For multi-selection rotation, the domain computes the center of the union of the physical footprints and snaps it toward the top-left grid intersection. The selection retains this pivot while its members remain unchanged, and an accepted move translates it by the same delta. Group movement and rotation are atomic domain transactions: any bounds or collision failure preserves the entire group, selection, pivot, and allocator. Removal freezes the selected IDs in one confirmation request; **Cancel**, `Escape`, or the backdrop preserves the layout, selection, and IDs. Pan, zoom, and focus do not alter the layout.

A configured `production_target` must exist in the active catalog and be listed by that buildable. Rejected changes preserve the instance, and placement revalidates a configured target against its destination catalog. This is configuration and referential validation only: the editor does not validate recipes, rates, ports, connectivity, throughput, regional mechanics, or the accuracy of game data. The minimal public fallback declares no products or production targets, so its **PRODUCT** chooser is absent. History and persistence are not yet part of the egui interface.

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
