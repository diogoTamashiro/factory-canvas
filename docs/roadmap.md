# Implementation roadmap — Factory Canvas

> Operational document for continuing the project manually or in another conversation. Update it when each functional slice ends; do not use this file as a commit or PR log.

## Product goal

Factory Canvas is a native, offline Windows CAD tool for manually designing 2D factories in *Arknights: Endfield*. The first cycle covers spatial occupancy, CAD navigation, reusable modules, and local documents; it does not automatically validate production, connectivity, or throughput.

## Non-negotiable principles

- The domain in `src/domain/` does not depend on egui, the filesystem, SQLite, the network, or Python.
- The UI calls public `FactoryLayout` APIs; it never duplicates footprint, bounds, collision, or rotation calculations.
- Grid: origin `(0, 0)` at the top-left; X increases to the right; Y increases downward.
- Footprints and spatial rectangles are semi-open: edge contact is allowed and overlap is prohibited.
- `EntityId` is a stable identity, not a collection index. New IDs are monotonic and are not reused after removal.
- Placement, movement, and rotation errors preserve the layout.
- The default interface and tracked project documentation are in English; internal identifiers remain in English.
- KISS/YAGNI: do not introduce an ECS, event bus, plugins, DI, drag-and-drop, or speculative abstractions outside the current slice.
- Do not change `src/main.rs` (the `factory-canvas-legacy` binary) while evolving the egui editor without an explicitly approved need.

## Confirmed data

### Bases

| Template | Bounds |
|---|---:|
| Main PAC | 80×80 |
| Standard Sub-PAC | 30×30 |
| Sub-PAC Expansion I | 40×40 |
| Sub-PAC Expansion II | 50×50 |

### Initial catalog

| Block | Category | Footprint |
|---|---|---:|
| Xiranite Power Pole | Power | 2×2 |
| Refinery Unit | Production I | 3×3 |
| Crushing Unit | Production I | 3×3 |

Do not infer earlier Main PAC levels, ports, power range, recipes, throughput, or any other unconfirmed data.

## State already integrated into the egui editor

- Selection among the four confirmed bases; destructive changes require confirmation when instances exist.
- Palette with the three confirmed blocks.
- Click placement using a top-left origin, monotonic IDs, and domain-only validation.
- Grid with preserved aspect ratio and exclusive right/bottom edges in hit testing, including protection against `f32` rounding on the last tile.
- Semantic sidebar list with ID, name, origin, rotated footprint, and rotation.
- Instance selection through the canvas or text list; visual highlighting of the selection.
- Confirmed single-instance removal through the button, `Delete`, or `Backspace`; cancellation, Escape, or the backdrop preserve state.
- The domain already has `place`, `instance_at`, `remove_instance`, `move_instance`, and `rotate_instance`, all covered by tests.
- `factory-canvas-legacy` continues to compile as an independent binary.

## Visual movement and rotation — integrated

### UX interaction

With one instance selected:

- the sidebar provides text buttons to move one tile: up, left, right, and down;
- `ArrowUp`, `ArrowLeft`, `ArrowRight`, and `ArrowDown` perform the same movements;
- **Rotate 90°** and `R` rotate clockwise;
- no edit occurs without a selection or while a removal/base-change modal is pending;
- the controls do not anticipate bounds or collision: the attempt reaches the domain and receives English feedback;
- this slice has no dragging, click-to-move, pan/zoom, undo/redo, or persistence.

### Verified contracts

- a valid move changes only the selected instance's origin and preserves its ID, selection, and `next_entity_id`;
- an attempt outside the base preserves the layout, selection, and allocator and returns `InstanceEditError::OutOfBounds` mapped to a user-facing message;
- with one instance selected, clockwise rotation preserves its ID and origin and updates its orientation;
- buttons, arrow keys, `R`, and removal go through one app-state intent dispatcher;
- the semantic list continues to expose ID, origin, rotated footprint, and rotation after every edit.

## Placement footprint preview — integrated

With an active placement tool:

- the canvas derives a candidate from the active template and the tile under the cursor;
- the candidate is drawn over the canvas with semitransparent fill and an outline in the block color;
- the preview is visual only: it does not query, replicate, or anticipate bounds, collision, or acceptance;
- the final click remains in the existing flow and `FactoryLayout::place` remains the sole spatial authority;
- selecting an instance clears the placement tool and hides the preview; opening a removal/base-change modal suppresses it without discarding the saved block selection.

### Verified contracts

- a visual candidate exists only when a template is active and the cursor is inside the grid;
- the preview origin and footprint derive exclusively from hit testing and the catalog;
- the canvas uses `Option<BlockTemplate>` as the single source for the placement cursor, preview, and click intent;
- selection of an occupied tile still takes priority over placement;
- destructive modals suppress the preview and preserve the tool after cancellation.

## Phase 0 — CAD direction and data contract — integrated

This phase approved and versioned the architectural contract. Runtime types, catalog loading, and document persistence will still be implemented in Phases 3 through 5.

- Factory Canvas now has an explicit contract for `FactoryDocument`, `BlueprintDocument`, and a modular data package;
- machines, conveyors, power poles, and future components are constructible entities in the same spatial system;
- the selected product belongs to the positioned entity; the catalog declares capabilities without validating flow in this cycle;
- blueprints are independent copies with relative entities and exposed-port interfaces;
- JSON documents use `schema_version`; game data uses a SemVer `data_version`;
- private reference data is not part of the public repository.

## Phase 1 — Viewport and CAD navigation — integrated

The canvas has a persistent, pure `CanvasViewport`:

- the mouse wheel applies cursor-anchored zoom, limited to the safe range from 25% to 400%;
- the middle button applies screen-space pan;
- `Home` frames the full base and is disabled during destructive modals;
- painting, preview, and hit testing use the same transformed grid rectangle;
- navigation does not change the domain, placement, selection, or IDs.

## Phase 2 — Multi-selection and CAD group — integrated

- `SelectedSet` keeps unique, ordered IDs; a normal click replaces, `Shift` adds, and `Ctrl` toggles in the canvas and sidebar;
- a marquee starts only from a primary-button drag on an empty tile with no placement tool; membership uses only the instance origin;
- every selected instance is highlighted and exposed through a semantic count;
- arrow keys/buttons move the set; `R`/button preserves the origin of a single selection or rotates the positions and orientations of two or more instances around the physical center snapped to the grid;
- the orbital pivot stays stable while the selected IDs do not change, follows valid moves, and is preserved with the batch when an edit is rejected;
- the domain validates movement and rotation as atomic batches without colliding with the members' own old positions;
- `Delete`, `Backspace`, and **Remove block(s)** freeze the IDs in one modal; cancellation preserves the layout/selection and confirmation removes the snapshot once;
- `F` and **Frame selection** focus the union of the physical footprints with padding; `Home` continues to frame the complete base;
- placement, monotonic IDs, pan/zoom, and earlier modals retain their contracts.

## Phase 2.5 — English project surface — integrated

- The default egui interface, compiled display names, notices, dialogs, controls, and painted abbreviations use English.
- All 12 tracked public Markdown documents use natural technical English.
- Stable IDs, dimensions, geometry, shortcuts, state transitions, and persistence contracts are unchanged.
- The frozen legacy application retains its existing language and behavior; no runtime i18n framework or language selector was added.

## Approved product milestone — complete the CAD MVP

The next milestone covers Phases 3, 4, and 5, in that order. Each phase will have its own plan, approval before execution, and atomic commits; scope will be reviewed again at the end of each one.

## Next active slice — Phase 3: data package and per-entity product

When resuming:

1. define the minimum contract for `CatalogManifest` and versioned modules;
2. track only a minimal catalog with already-public, confirmed data; the complete package and private reference data remain local and ignored by Git;
3. load and validate the catalog in a testable, fully offline way;
4. migrate the three compiled templates to the modular source while preserving stable IDs and current spatial behavior;
5. add optional `production_target` only to capable entities, without validating recipes or throughput;
6. reject incompatible schemas, duplicate IDs, and invalid references without changing open state;
7. update tests, documentation, gates, and independent review.

## Remaining MVP phases, in order

### 4. JSON documents and blueprint library

Persist the factory and modules as local JSON with `schema_version`, explicit migration, and atomic saves. Convert a literal selection into an independent blueprint.

### 5. Independent insertion and exposed interfaces

Insert a blueprint as a batch, with new IDs and atomic failure on bounds/collision. Expose and name physical ports open at the boundary without assuming a connection.

## Post-MVP phases

### 6. Command-based undo/redo

Model placement, removal, movement, rotation, and base-change commands. Only then consider immediate removal without confirmation; while no history exists, single or group removal must remain confirmed.

### 7. Accessibility and polish

Review the selectable sidebar row and, if the egui version allows it without clipping, use a control with more explicit button/focus semantics. Always keep the complete label with ID, name, origin, footprint, and rotation.

### Deliberately later items

Connectivity validation, recipes, throughput, solver/CP-SAT, auto-layout, OCR, game import, login, cloud, AI, heavy sprites, and 3D rendering.

## Engineering workflow per slice

1. Create or review a plan under `.hermes/plans/` (not tracked) with scope, decisions, and gates.
2. Synchronize `master` with `origin/master`, confirm a clean worktree, and start only after the phase plan is approved.
3. Work in RED → GREEN tracer bullets: one logical behavior, failing test, minimal implementation, green test.
4. For UI work, cover transitions, rejections, and the semantic representation with logical, deterministic tests; do not use visual automation as a substitute for these contracts.
5. For changes whose acceptance depends on real appearance or interaction, produce a manual test script; Diogo runs it and reports the result without blocking gates or publication.
6. Run the complete gates before freezing the stage.
7. Stage explicitly, run `git diff --cached --check`, scan added lines for security issues, and independently review the frozen snapshot.
8. Only then create atomic `[verified]` commits directly on `master` and publish to `origin/master`.
9. After the push, compare the local SHA with `refs/heads/master` on the remote before declaring the phase published.

## Required gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

For UI changes:

1. cover the success path, at least one rejection/safety path, and changed state transitions with logical, deterministic tests;
2. check that semantic labels expose the same relevant state as the painter;
3. compile the main binary and release bins as part of the gates, without treating an automated capture as proof of interaction or appearance;
4. produce a manual test script for Diogo, without treating an automated capture as proof of interaction or appearance;
5. stop every test process and confirm it does not block the next build.

## Quick manual resumption

If this work resumes without conversation context:

```bash
git fetch origin
git status --short --branch
git log --oneline -5 origin/master
```

Then read, in order:

1. `docs/roadmap.md` (this file);
2. `CONTEXT.md`;
3. `docs/architecture.md`;
4. `src/egui_app.rs` and `src/egui_app_tests.rs`;
5. `src/domain/layout.rs` and `tests/domain_layout_editing.rs`.

Synchronize `master` with `origin/master`, start with the first behavior not yet covered by a test, and keep the slice small. Branches or PRs are used only if Diogo explicitly changes the workflow for a specific task.
