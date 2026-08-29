# Product scope — Factory Canvas

## Problem

Building a factory directly in the game renderer consumes resources, makes alternatives harder to try, and provides no simple schematic view of the available space. Factory Canvas provides a lightweight 2D representation so the player can test base occupancy before or during construction in the game.

## Target user

An Arknights: Endfield player who wants to organize a factory manually, understand space usage, and try layouts without depending on an online tool or 3D renderer.

## Value proposition

- lighter than opening and rendering the base in the game;
- a clear view focused on space;
- fully offline;
- no account or external service;
- the player keeps control of the layout.

## First CAD cycle scope

### R1 — Base type

When creating a layout, the user chooses:

- `Main` — Main PAC, 80×80 at the currently confirmed level;
- `Secondary` — sub-PAC, 30×30, 40×40, or 50×50 according to the expansion level.

`BaseTemplate` represents the selected option, and its type and confirmed level determine the layout's `GridSize`. Both bases are in Wuling, are square, have no known internal obstacles, and can evolve. Unknown Main PAC levels will not be inferred.

The current `egui` shell enumerates `BaseTemplate::ALL` and redraws the grid from the bounds derived from the selected option. A change is immediate when the layout is empty; when instances exist, a modal either cancels safely or requires explicit confirmation to change the base and clear the layout.

### R2 — Constructible entity catalog

The current editor still has a compiled block catalog. The versioned direction is a unified catalog of constructible entities: machines, conveyors, power poles, and future components share placement, footprint, rotation, bounds, and collision behavior.

Each entity definition will contain, at minimum:

- stable identifier;
- display name;
- unrotated width and height;
- visual category;
- relative physical ports, flow direction, and type;
- static capabilities, such as the products a machine can produce.

Through `BlockTemplate::ALL`, the domain exposes the initial catalog with Xiranite Power Pole (2×2), Refinery Unit (3×3), and Crushing Unit (3×3). All support four rotations and both base types. The future migration to data-loaded catalog IDs will preserve the spatial domain and will not anticipate port, recipe, or flow validation.

The current egui palette derives names and footprints from these definitions, preserves the selected template for repeated placements, and keeps no parallel catalog in the UI. The public package describes schemas; detailed game data may remain local and ignored by Git.

### R3 — Placement

The user can place, select, move, rotate, and remove blocks. Every operation snaps to the grid.

`FactoryLayout::place` attaches an instance only after validating its ID, rotated footprint, bounds, and collision. Footprints use semi-open rectangles: overlap is rejected, but edge contact is allowed.

The domain also enumerates instances immutably and deterministically by ID. Removal returns the removed instance or `None` for a missing ID. Single-instance movement and rotation take absolute values and revalidate bounds and collision; single-instance rotation preserves its origin. For sets, `move_instances_by` and `rotate_instances_clockwise_about` remove the old positions from a copy of the layout, validate every final destination, and commit only if the complete batch is accepted. The orbital pivot comes from the center of the physical footprints and snaps toward the top-left grid corner.

The egui interface converts a click into a grid coordinate, treats the empty tile as the top-left origin, creates monotonic IDs, and calls `FactoryLayout::place`. With an active block, the canvas draws its semitransparent footprint; the preview is visual only. A normal click replaces the selection, `Shift` adds, and `Ctrl` toggles in both the canvas and the text list. With no active tool, dragging the left mouse button from empty space creates a marquee and includes only instances whose origin is inside the rectangle. Every selected instance is highlighted. Controls and arrow keys move the set; **Rotate 90°**/`R` rotates one instance at its own origin or moves and orients two or more around the shared pivot. The pivot persists until selection membership changes and follows valid moves; failures preserve both the batch and pivot. `Remove block(s)`, `Delete`, and `Backspace` freeze the IDs in a confirmation request; cancellation, Escape, or the backdrop preserve the layout, selection, and allocation.

The next CAD phases add a modular data package, product configuration per entity, JSON documents, and local blueprints. A selected product does not imply recipe, input, output, or throughput validation in this phase.

### R4 — Constraints

The domain rejects:

- duplicate instance ID;
- edits to a missing ID;
- blocks outside the bounds;
- overlapping footprints;
- zero dimensions;
- references to a missing definition.

### R5 — Navigation

The CAD cycle provides pan, zoom, framing of the base, and focus on the selected set.

A persistent viewport applies pan and zoom to painting and hit testing through one transform; the right and bottom edges remain exclusive. The mouse wheel zooms at the cursor, the middle button pans the view, and `Home` frames the full base. `F` and **Frame selection** calculate the union of the selected physical footprints, apply visual padding, and do not change `FactoryLayout`.

### R6 — History

Placement, movement, rotation, removal, group editing, and blueprint insertion will support undo and redo in a later phase.

### R7 — Persistence

Users will be able to save and open factories and blueprints locally in readable, versioned formats.

### R8 — CAD documents and blueprints

The complete factory will be a `FactoryDocument`. Users will be able to save a selected set as a `BlueprintDocument` in a persistent local library. Inserting a blueprint creates an independent copy with new IDs; it does not update the original definition automatically.

Blueprints preserve entities in relative coordinates and expose nameable interfaces for physical ports open at the selection boundary. They do not represent confirmed connectivity or validated flow.

## First MVP non-goals

- validation of connectivity between ports and conveyors;
- recipes, automatic production, and input/output balance;
- throughput and bottlenecks;
- solver or auto-layout;
- automatic routing;
- online synchronization;
- capture/OCR;
- 3D rendering.

## Non-functional requirements

- Windows desktop;
- offline;
- canvas without one widget per tile;
- idle CPU usage near zero;
- scalable, high-contrast interface;
- keyboard shortcuts for main actions;
- no AI dependency;
- user files are never overwritten after a validation error.

## MVP acceptance criterion

At the end of the CAD cycle, a player must be able to create a Main or Secondary layout, navigate between the overview and subsets, place constructible entities with different footprints, rotate them, reorganize them without collisions, configure the product of capable entities, save the document, and reuse local blueprints without depending on a network connection.

## Data still pending

1. Detailed PAC, entity, port, product, region, and rule data maintained in the versioned local package.
2. Confirmed validation rules for conveyor and port connectivity.
3. Recipes, rates, throughput, and other production mechanics.
