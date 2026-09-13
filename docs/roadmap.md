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

## Tracked public compatibility fallback

### Bases

| Base | Bounds |
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

The embedded public package intentionally contains no products or production targets. A complete valid local catalog can supply additional data without changing the editor's spatial rules.

## State already integrated into the egui editor

- One validated runtime `Catalog` supplies the base list, default and bounds, buildable palette, preview, painter, layout resolution, and semantic labels.
- The tracked public fallback supplies the four confirmed bases and three confirmed buildables; destructive base changes require confirmation when instances exist.
- Click placement using a top-left origin, monotonic IDs, and domain-only validation.
- Grid with preserved aspect ratio and exclusive right/bottom edges in hit testing, including protection against `f32` rounding on the last tile.
- Semantic sidebar list with ID, name, origin, rotated footprint, rotation, and configured product or `no product`.
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

- the canvas derives a candidate from the active buildable and the tile under the cursor;
- the candidate is drawn over the canvas with semitransparent fill and an outline in the block color;
- the preview is visual only: it does not query, replicate, or anticipate bounds, collision, or acceptance;
- the final click remains in the existing flow and `FactoryLayout::place` remains the sole spatial authority;
- selecting an instance clears the placement tool and hides the preview; opening a removal/base-change modal suppresses it without discarding the saved block selection.

### Verified contracts

- a visual candidate exists only when a buildable is active and the cursor is inside the grid;
- the preview origin and footprint derive exclusively from hit testing and the catalog;
- the canvas uses `Option<BuildableId>` as the single source for the placement cursor, preview, and click intent;
- selection of an occupied tile still takes priority over placement;
- destructive modals suppress the preview and preserve the tool after cancellation.

## Phase 0 — CAD direction and data contract — integrated

This phase approved and versioned the architectural contract. Phase 3 implemented runtime catalog types, loading, and product configuration; Phase 4 implemented document persistence. Schema migration beyond v1 remains future work, introduced only when a second schema version exists.

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

- The default egui interface, tracked catalog display names, notices, dialogs, controls, and painted abbreviations use English.
- All 12 tracked public Markdown documents use natural technical English.
- Stable IDs, dimensions, geometry, shortcuts, state transitions, and persistence contracts are unchanged.
- The frozen legacy application retains its existing language and behavior; no runtime i18n framework or language selector was added.

## Approved product milestone — complete the CAD MVP

The milestone covers Phases 3, 4, and 5, in that order. All three are integrated. Phase 6 (post-MVP, command undo/redo), Phase 7 (post-MVP, sidebar row accessibility), and Phase 8 (post-MVP, visual rotation animation) are also integrated. Each phase has its own plan, approval before execution, atomic commits, and end-of-phase review.

## Phase 3 — data package and per-entity product — integrated

- `CatalogId`, `RegionId`, `BaseId`, `BuildableId`, `ProductId`, and `CategoryId` are validated runtime IDs. `Catalog` stores ordered definitions plus deterministic indexes in an immutable snapshot.
- Schema v1 is one strict manifest plus required regions, bases, buildables, and products modules. All fields are required, unknown fields are rejected, and the complete candidate must pass schema, SemVer, dimensions, identifiers, metadata, symbols, uniqueness, paths, and cross-reference validation.
- `catalog/public/` is tracked, minimal, and embedded. A complete ignored `data/catalog/` package wins only when fully valid. A missing private manifest silently selects public; any other failure selects the complete public package and leaves a persistent sanitized warning. Sources are never mixed, and user-facing diagnostics expose no raw private JSON, full private paths, private identifiers, or private values.
- Compiled base and block enums have been replaced by catalog-backed `BaseId` and `BuildableId` references. Existing placement, bounds, collision, movement, rotation, selection, and ID behavior is preserved.
- Each `BlockInstance` stores `Option<ProductId>`. The domain accepts only products present in the active catalog and declared by that buildable, preserves state on rejection and spatial edits, and revalidates the target during placement.
- Exactly one selected capable instance exposes the **PRODUCT** chooser in its declared target order. **No product** clears the target, and semantic labels show the configured product or `no product`.
- This phase validates configuration and references only. It does not validate recipes, rates, ports, connectivity, throughput, regional mechanics, or game-data accuracy, and it does not change frozen `src/main.rs`.

The app has no hot reload. Close it before editing the private five-file package and restart it afterward because the reads are not one filesystem-atomic snapshot. Changes under `catalog/public/` require a rebuild so the fallback is re-embedded.

## Phase 4 — JSON documents and blueprint library — integrated

- `FactoryDocument` is a versioned local JSON document (`schema_version` 1) capturing the active base, every positioned entity, the allocator's `next_entity_id`, and document metadata (name, optional description, `created_at`/`updated_at`). Encoding is deterministic (entities sorted by ID); decoding is strict and all-or-nothing, rejecting unknown fields, malformed IDs, invalid rotations, and layout-validation failures without ever partially applying a document.
- Every save is atomic: encode fully in memory, create a temporary file in the target's directory, write all bytes, flush and synchronize, then atomically replace the target. A validation, encoding, or write failure leaves the previously saved file completely untouched; a successful save is reported only after replacement.
- `DocumentSession` tracks the open document's path, metadata, dirty flag, and catalog compatibility across New/Open/Save/Save As, driven by native file dialogs (`rfd`) and the `Ctrl+O`/`Ctrl+S`/`Ctrl+Shift+S` shortcuts (New is button-only, with no bound shortcut). An unsaved-changes prompt guards New, Open, and window-close whenever the session is dirty; cancelling any destructive modal preserves the layout, selection, and session exactly.
- A recorded `catalog_id`/`catalog_data_version` mismatch alone does not block factory loading: an otherwise-valid document opens with a compatibility result, but full validation against the active catalog still applies and can fail. Opening does not rewrite the file. The session retains the mismatch, while later actions such as selecting an instance can replace the visible open-result notice. A successful factory save records active-catalog provenance and resets session compatibility to `Exact`.
- `Blueprint::from_selection` captures one or more selected canvas instances as an independent, self-contained module: entities are re-expressed as `BlueprintNode`s in coordinates relative to the selection's own footprint, with fresh local `BlueprintEntityId`s that share no identity with the source factory. `BlueprintDocument` (schema v1) persists a blueprint the same way `FactoryDocument` does — versioned JSON, strict all-or-nothing decoding, deterministic encoding.
- `BlueprintLibrary` is a local, offline, per-user library at `%LOCALAPPDATA%/Factory Canvas/blueprints`: `save()` creates the storage root automatically on first use, resolves a blueprint-ID collision by regenerating a fresh ID and retrying (bounded), and every write goes through the same atomic temp-file-plus-rename primitive `FactoryDocument` uses. `list()` silently skips symlinks, directories, and files that do not match the blueprint naming convention; it isolates any unreadable-or-malformed content or duplicate-identity file into a safe, path-free warning instead of failing the whole listing, and returns every valid blueprint sorted alphabetically by name with a stable ID tiebreak.
- The editor exposes this library directly: a **Save as blueprint** action appears only while at least one instance is selected, opens a short confirmation dialog for a non-blank name, and leaves the canvas, selection, and every entity identifier completely unchanged on success. A persistent **BLUEPRINT LIBRARY** sidebar section lists every saved blueprint's name, module count, and last-saved time, shows an explicit empty-library indication with none saved yet, and refreshes automatically after a successful save. Unreadable entries appear as a safe, generic warning count; catalog mismatches have indicators on their cached library rows, separate from the replaceable factory open-result notice. These indicators are non-blocking and expose no file path, raw content, or technical identifier.
- This phase does not insert a blueprint into the canvas or provide physical-port interfaces; those were delivered in Phase 5. Blueprint editing, deletion, renaming, import, and export are also not delivered, with no specific phase assigned. This phase introduces no migration beyond schema v1 and does not change frozen `src/main.rs`.

## Phase 5 — blueprint insertion and exposed interfaces — integrated

- `Blueprint::insert_into` places every node of a saved blueprint into the currently open factory as one atomic, all-or-nothing batch at a chosen insertion point: each node becomes a new, independent `BlockInstance` with a fresh sequential ID starting at the caller's `next_entity_id`, keeping its buildable, rotation, and configured product exactly. Any single failure — out of bounds, collision (with an existing instance or another node in the same batch), a buildable/product no longer present in the active catalog, ID-allocator exhaustion, or coordinate overflow — rejects the whole batch and leaves the destination factory byte-for-byte unchanged; nothing is partially applied. `FactoryLayout` itself is unchanged: insertion reuses the already-public `FactoryLayout::place()` as-is.
- Inserted instances are ordinary instances from the moment they land: moving, rotating, or removing one has no special "came from a blueprint" behavior and no group-move side effect on the other instances from the same insertion.
- A blueprint may also carry named `Interface`s — boundary markers placed only on the outer edge of its own bounding rectangle, each with a unique non-blank name. An interface is purely descriptive metadata (FR-010): it asserts no physical port, connection, or flow state, and reuses the exact `anchor`/`side` vocabulary already documented for a possible future catalog-level physical-port system without introducing that system itself. Interfaces are markable only while saving a blueprint for the first time; a saved blueprint cannot later be edited to add, remove, or rename one (blueprint editing itself remains unassigned to any phase, per Phase 4).
- `interfaces[]` is an additive optional field within `BlueprintDocument` schema v1 (no version bump); every blueprint file saved before this phase still decodes with an empty interface list.
- The **BLUEPRINT LIBRARY** sidebar gained an **Insert** action per row, arming that blueprint for canvas placement the same way a single buildable is armed, with a matching multi-node placement preview; each row also shows its blueprint's interface names. The save-as-blueprint dialog gained an interface-marking section (name plus a boundary-location choice drawn from that exact selection's own valid boundary points).
- This phase does not add a catalog-level physical-port system (`PortDefinition`/`PortTypeId`/flow) — a distinct, later increment with no assigned date — and does not add blueprint editing, deletion, renaming, import, or export.
- First phase developed on its own spec-kit feature branch and merged via reviewed pull request (`004-blueprint-insertion-interfaces`, PR #18), per the Constitution's "Workflow and Branching" section taking effect from this phase onward.

## Phase 6 — command undo/redo — integrated

- `EditHistory` (a new `src/history.rs` module) keeps an `undo_stack`/`redo_stack` of whole-layout `EditorSnapshot`s. Each of the six mutating commands — place, remove, move, rotate, base change, blueprint insert — records a pre-mutation snapshot on its own success branch only, never on a rejected attempt, and pushing a new snapshot always clears the redo stack.
- `undo`/`redo` are a no-op at either stack's empty end and a no-op while any destructive confirmation modal is open; selection is pruned via the existing post-restore reconciliation path rather than itself being restored, and starting a new factory or opening a different one clears both stacks.
- The identifier allocator (`next_entity_id`) is never captured or restored by a snapshot: it is already monotonic by construction (only placement/insertion ever advance it, nothing ever decreases it), so undoing or redoing a command can never make it move backward, satisfying the specific guarantee this phase's spec required.
- New **Undo**/**Redo** header buttons (disabled at an empty stack or while a modal is open) and `Ctrl+Z`/`Ctrl+Y` shortcuts reuse the same shared-dispatcher pattern the existing document commands already use.
- `src/domain/`, `catalog/`, and `persistence/` are untouched; history is session-only and is never part of any saved document.
- First phase merged with a local `git merge --ff-only` rather than a hosted pull request, per Constitution v1.2.0 — see "Engineering workflow per slice" above for why.

## Phase 7 — sidebar row accessibility — integrated

- Each row in the sidebar's **INSTANCES ON CANVAS** list now renders through `egui::Button::new(...).selected(...)` instead of `egui::Label` with a manual `.sense(Sense::click())` and a hand-rolled selected-color branch — the exact pattern the block palette's own options already used. Every row reports `Role::Button`, an accurate per-row `toggled()` state, and standard keyboard focusability to AccessKit and any assistive technology reading it; previously the row was exposed as plain static text despite already being clickable.
- The row's complete label (identifier, name, origin, footprint, rotation, product) is unaffected: `.wrap()` already preserved it in full before this phase, and still does — confirmed by a disposable spike during planning (deleted before implementation) showing `Button::wrap()` and `Label::wrap()` share the same underlying layout and text-wrap behavior at this project's real sidebar width.
- Click dispatch (plain/`Shift`/`Ctrl` → Replace/Add/Toggle) is untouched; a manual `.color(if selected { ACCENT } else { TEXT_PRIMARY })` branch is deleted, since `Button`'s own `.selected()` styling is now the row's sole source of selected-state appearance.
- Scope is deliberately narrow: only the sidebar's instance list changed. The block palette, blueprint library, and status bar were already using semantically-correct controls and are untouched.
- Second phase merged with a local `git merge --ff-only` rather than a hosted pull request, per Constitution v1.2.0.

## Phase 8 — visual rotation animation — integrated

- Every placed instance now paints a small orientation-indicator arrow — a documented placeholder shape until real per-block icons/sprites exist. Rotating a selected instance turns its arrow smoothly over ~200ms via `egui::Context::animate_value_with_time`, instead of snapping instantly; a group's orbital rotation animates position AND angle together for every member, synchronized by the same shared pivot the domain already computes. A plain move (no rotation) never animates position, and a rejected rotation animates nothing.
- A new `RotationVisuals` (living inside `egui_canvas::CanvasState`, keyed by `EntityId::value()` since `EntityId` itself doesn't derive `Hash`) tracks each instance's last-known `Rotation`/origin and an accumulated target angle; `resync()` rebuilds it from a fresh `FactoryLayout` and forces the very next frame to snap instantly instead of interpolating. It is wired into all four call sites that replace the whole layout — `new_document_at`, `open_document_from`, `replace_base`, and `apply_restored_snapshot` (shared by undo and redo) — so undo/redo, New, Open, and a base change all remain perfectly instant even if a rotation was mid-transition the moment they fired.
- Two real `egui` behavior quirks were found and fixed via TDD, both in how `animate_value_with_time` resolves its own return value on the exact call that changes an animation's target: a `duration=0.0` call only arms the instant snap for the *next* read, and an animation entry only touched when a rotation is detected skips its own first-ever transition. Confirmed against the vendored `egui` source, not just observed behavior.
- `src/domain/`, `src/persistence/`, `src/history.rs`, and every catalog/data file are untouched — this phase is 100% presentational.
- Third phase merged with a local `git merge --ff-only` rather than a hosted pull request, per Constitution v1.2.0.

## Phase 9 — custom buildable icons — integrated

- Any buildable's existing JSON catalog object gains one optional `icon` field: a path relative to a new dedicated, versioned `assets/icons/` directory (parallel to `catalog/public/`, never embedded, always read from disk). A placed instance, both placement-preview kinds (single-buildable and each blueprint member independently), and the buildable palette all show that buildable's icon in place of its abbreviated text when the reference resolves to a real, safely-contained, decodable static PNG file; an omitted, `null`, blank, or unusable reference falls back to the existing text with no application-wide effect, and an explicit-but-unusable reference (not merely an omitted one) surfaces one sanitized, non-blocking warning alongside the existing catalog warning.
- The former placeholder orientation arrow (Phase 8) is removed entirely. On an accepted rotation, the icon itself now turns smoothly in place of the arrow; a buildable with no usable icon instead turns its own fallback text label the same way, via a new `epaint::TextShape` with an explicit angle (`painter.text(...)` has no rotation parameter). `RotationVisuals` — the animation state and its four whole-layout-resync call sites Phase 8 established — is completely unmodified; only which shape consumes its per-frame angle changed.
- Path safety for `icon` reuses `catalog_loader`'s own canonicalize-then-`starts_with` algorithm against the new `assets/icons/` root: empty/NUL/rooted/Windows-prefixed/parent-directory references and symlink/junction escapes are rejected before any byte is read, and a 4 MiB raw-file cap runs before any PNG decode is attempted. PNG content is decoded via `image` 0.25.10 (already compiled into the dependency tree via `eframe`, promoted from transitive to direct — zero new transitive dependencies) using `load_from_memory_with_format(..., ImageFormat::Png)`, which rejects by actual file content rather than by filename extension.
- Icons are loaded into real GPU textures exactly once, in `FactoryCanvasApp::new` — the only construction path with a live `egui::Context` — so every test-only constructor (`from_startup_catalog`, `Default`) stays fully filesystem/GPU-free, unchanged from every prior phase's testing discipline.
- `README.md` documents the full customization workflow (icon placement, the JSON field, supported format, restart vs. rebuild, troubleshooting) as an extension of its existing runtime-catalog section, using the same confirmed Xiranite Power Pole example already in the README; `docs/data-model.md` records `icon` as `buildables`' one optional field.
- `src/domain/layout.rs`, `src/persistence/`, and every existing placement/rotation/collision/production rule are untouched — this phase, like Phase 8, is entirely presentational plus one new optional, non-validated string field on the existing buildable definition.
- This is the second phase in this project's history merged via a hosted, independently reviewed pull request — the first was Phase 5 (PR #18); Phases 6-8 and the intervening large-file-split refactor merged with a local `git merge --ff-only` alone, per Constitution v1.2.0. Constitution v1.3.0 (2026-09-13) reinstated the hosted-PR requirement for every feature branch going forward, while keeping that same `git merge --ff-only`-from-a-confirmed-`master` mechanic as the actual merge step (never GitHub's own merge-commit or squash-merge button).

## Post-MVP phases

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
8. Only then create atomic `[verified]` commits. Through Phase 4, these committed directly on `master` and published straight to `origin/master`. Starting with Phase 5, per the Constitution's "Workflow and Branching" section, each phase instead develops on its own spec-kit feature branch (`NNN-slug`). Phase 5 itself merged via a hosted, independently reviewed pull request (`004-blueprint-insertion-interfaces`, PR #18); starting with Phase 6, per Constitution v1.2.0, merging back to `master` used a local `git merge --ff-only` after review instead, without a hosted PR — Diogo is this project's sole maintainer, so a hosted PR added ceremony with no second human ever acting on it, and Phase 5's squash-merge silently combined the feature with an unrelated same-day commit once local `master` had drifted ahead of `origin/master` (see Phase 5 below). Phases 6, 7, 8, and the large-file-split refactor that followed Phase 8 all merged this way, with no PR. Effective 2026-09-13, per Constitution v1.3.0, Diogo directed the project to reinstate a hosted pull request for every feature branch going forward — the merge mechanics that prevented Phase 5's squash problem are unchanged (still `git merge --ff-only` from a `master` confirmed equal to `origin/master`, never GitHub's own merge-commit or squash-merge button), only the "open a PR first" step returns. Do not read the absence of a PR on Phases 6-8 or the refactor as a mistake; they were correct under the policy active at the time.
9. After the push (direct, or after a fast-forward merge from a reviewed and approved PR), compare the local SHA with `refs/heads/master` on the remote before declaring the phase published.

## Required gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test <file>   # only tests covering this stage's changes — see docs/engineering-standards.md §Testing scope
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
4. `src/catalog_loader.rs` and `src/domain/catalog.rs`;
5. `src/domain/layout.rs` and its domain tests;
6. `src/egui_app.rs`, `src/egui_app_tests.rs`, and `src/egui_canvas.rs`.

Synchronize `master` with `origin/master`, start with the first behavior not yet covered by a test, and keep the slice small. Branches or PRs are used only if Diogo explicitly changes the workflow for a specific task.
