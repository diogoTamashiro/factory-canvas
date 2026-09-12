# Feature Specification: Command-Based Undo/Redo

**Feature Branch**: `005-command-undo-redo`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "fase 6" — Roadmap Phase 6 (Post-MVP phases §
"6. Command-based undo/redo"): "Model placement, removal, movement,
rotation, and base-change commands. Only then consider immediate removal
without confirmation; while no history exists, single or group removal
must remain confirmed." (`docs/roadmap.md`). `docs/product-scope.md`'s R6
adds blueprint insertion to the same command set: "Placement, movement,
rotation, removal, group editing, and blueprint insertion will support
undo and redo in a later phase." Per the project constitution's "Workflow
and Branching" section, this phase is developed on its own spec-kit
feature branch, same as every phase from Phase 5 onward.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Undo the most recent action (Priority: P1)

A player performs an edit — placing a block, removing one or more
instances, moving or rotating one or more instances, changing the active
base, or inserting a blueprint — and immediately realizes it was a
mistake. They trigger **Undo** and the factory returns to exactly how it
was right before that action, with no need to manually reconstruct the
previous layout by hand.

**Why this priority**: This is the single biggest source of daily
friction the roadmap names undo/redo to solve, and it delivers value on
its own regardless of whether redo or multi-step history exist yet — a
player who can reverse their very last mistake already has real
protection they do not have today.

**Independent Test**: Can be fully tested by performing any one of the
six commands (place, remove, move, rotate, change base, insert
blueprint), triggering Undo once, and confirming every entity,
identifier, origin, rotation, configured product, and the active base
exactly match what they were before that command executed.

**Acceptance Scenarios**:

1. **Given** a factory with an empty undo history, **When** the player
   places a new block and then triggers Undo, **Then** that block
   disappears and every other entity, the active base, and every
   identifier are exactly as they were before the placement.
2. **Given** one or more instances moved or rotated (singly or as a
   group), **When** the player triggers Undo, **Then** every affected
   instance returns to its exact prior origin and orientation, and no
   other instance is affected.
3. **Given** one or more instances removed through the existing
   confirmation flow, **When** the player triggers Undo, **Then** every
   removed instance reappears with its original identifier, buildable,
   origin, rotation, and configured product, exactly as before the
   removal.
4. **Given** a base change performed through the existing confirmation
   flow, **When** the player triggers Undo, **Then** the previously
   active base and every instance that existed on it reappear exactly as
   before the change.
5. **Given** a blueprint successfully inserted into the factory, **When**
   the player triggers Undo, **Then** every entity created by that
   insertion disappears as one unit and the rest of the factory is
   unaffected.

---

### User Story 2 - Redo an undone action (Priority: P2)

After undoing an action, a player decides they actually wanted to keep
it. They trigger **Redo** and the action is reapplied exactly as it
originally happened.

**Why this priority**: Completes the minimal undo/redo pair. Without it,
Undo is a one-way trapdoor a player might hesitate to use for fear of
losing work permanently, undermining the confidence User Story 1 is
meant to provide.

**Independent Test**: Can be fully tested by performing any one of the
six commands, undoing it, then redoing it, and confirming the resulting
layout is identical to the state immediately after the original command
executed.

**Acceptance Scenarios**:

1. **Given** a command was just undone, **When** the player triggers
   Redo, **Then** the layout returns to exactly the state it was in
   immediately after that command originally executed.
2. **Given** no command has been undone since the factory was opened or
   since the last new edit, **When** the player attempts Redo, **Then**
   nothing happens.
3. **Given** a command was undone and then a different new command was
   performed, **When** the player attempts Redo, **Then** nothing
   happens, because the undone command is no longer available to redo.

---

### User Story 3 - Undo and redo across multiple steps (Priority: P3)

A player wants to step back through several of their most recent edits
in sequence — not just the very last one — to return to an earlier point
in the current session, and can then step forward again through the same
sequence.

**Why this priority**: Generalizes User Story 1 and User Story 2 from a
single step to an arbitrary-length session history. Valuable, but
strictly additive on top of correct single-level undo/redo, so it can be
delivered and tested independently once the first two stories work.

**Independent Test**: Can be fully tested by performing a sequence of
several different commands, undoing three of them in a row, confirming
the layout matches the state before the third-from-last command, then
redoing twice and confirming the layout matches the state after the
second command in the original sequence.

**Acceptance Scenarios**:

1. **Given** a session with several consecutive commands already
   performed, **When** the player triggers Undo repeatedly, **Then**
   each trigger reverses exactly one more command in reverse
   chronological order, stopping cleanly once no earlier command
   remains.
2. **Given** the player has undone several steps, **When** they trigger
   Redo repeatedly, **Then** each trigger reapplies exactly one more
   command in original chronological order, stopping cleanly once no
   undone command remains.
3. **Given** the player has undone several steps and then performs one
   new command, **When** they check what is available to redo, **Then**
   every step that was available to redo before that new command is
   gone.

---

### Edge Cases

- What happens when the player triggers Undo or Redo while a destructive
  confirmation (a pending removal or a pending base change) is open?
  Blocked entirely, consistent with this project's existing convention
  of blocking every other editing shortcut during a pending destructive
  modal.
- What happens when the player triggers Undo with an empty undo history,
  or Redo with an empty redo history? Nothing happens — no layout
  change, no notice implying an action was taken.
- What happens to the undo/redo history when the player starts a new
  factory or opens a different factory document? It is cleared; history
  never spans two different factory sessions.
- What happens when a command affecting multiple entities at once (a
  multi-instance move, a multi-instance rotation, or a multi-node
  blueprint insertion) is undone? It is reversed as a single step, never
  as one step per affected entity.
- What happens when the player performs a new edit after undoing one or
  more steps? Every step that was available to redo from that point on
  is discarded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST let a player undo the single
  most-recently executed command, where a command is one of: placing an
  entity, removing one or more entities, moving one or more entities,
  rotating one or more entities, changing the active base, or inserting
  a blueprint.
- **FR-002**: Undoing a command MUST restore the factory to its exact
  state immediately before that command executed: the same entities
  with the same identifiers, origins, rotations, and configured
  products, and — for a base-change command — the same previously
  active base together with every instance that existed on it.
- **FR-003**: The system MUST let a player redo the single
  most-recently undone command, re-applying it and producing the exact
  same resulting state the original command produced when it first
  executed, provided no new command has been performed since that undo.
- **FR-004**: Performing any new command after one or more undos MUST
  discard every previously undone command that redo would otherwise
  still be able to reapply.
- **FR-005**: The system MUST let a player undo or redo more than one
  step in a row, stepping backward or forward through the current
  factory session's full sequence of executed commands rather than only
  ever the single most recent one.
- **FR-006**: A command that affects multiple entities at once (a
  multi-instance move, a multi-instance rotation, or a blueprint
  insertion creating several entities) MUST be undone or redone as a
  single atomic step, never as separate steps per affected entity.
- **FR-007**: Undo and redo MUST have no effect and MUST NOT change the
  layout, the notice shown to the player, or either history, whenever
  the relevant history (undo or redo) is empty.
- **FR-008**: Undo and redo MUST be blocked while a destructive
  confirmation (a pending removal or a pending base change) is open,
  consistent with this project's existing convention of blocking other
  editing shortcuts during a pending destructive modal.
- **FR-009**: An identifier freed by a command that is later undone (a
  removed entity's identifier, or a placed or inserted entity's
  identifier) MUST NOT be assigned to any different, unrelated entity;
  the identifier allocator MUST continue to move forward only, exactly
  as it does today, regardless of any undo or redo.
- **FR-010**: Undo/redo history MUST be scoped to the currently open
  factory only; starting a new factory or opening a different factory
  document MUST clear both the undo and redo history.
- **FR-011**: This phase MUST NOT change single- or group-instance
  removal's existing confirmation requirement — every removal continues
  to require confirmation exactly as before; removing that confirmation
  once undo/redo exists remains a distinct, not-yet-scheduled decision
  per the roadmap.
- **FR-012**: This phase MUST NOT make any document-lifecycle action
  (New, Open, Save, Save As) or saving a selection as a blueprint part
  of the undo/redo history; these remain outside the set of undoable
  commands, and New/Open's own unsaved-changes confirmation remains
  their relevant safeguard.
- **FR-013**: This phase MUST NOT add per-instance production-target
  configuration to the set of undoable commands.

### Key Entities *(include if feature involves data)*

- **Undoable command**: A single, atomic, reversible record of one
  layout-mutating action (placing, removing, moving, or rotating one or
  more entities; changing the active base; or inserting a blueprint),
  carrying enough information to both reproduce its effect (redo) and
  exactly reverse it (undo).
- **Undo/redo history**: The current factory session's ordered sequence
  of executed undoable commands together with the current position
  within that sequence, separating commands available to undo from
  commands available to redo. Scoped to one open factory; cleared
  whenever a different factory is opened or a new one is started.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can reverse any one of their last several actions
  (placement, removal, movement, rotation, base change, or blueprint
  insertion) without manually reconstructing the prior layout by hand.
- **SC-002**: 100% of undo operations restore a layout identical — same
  entities, identifiers, positions, rotations, product configuration,
  and active base — to a snapshot taken immediately before the original
  command executed.
- **SC-003**: 100% of redo operations reproduce a layout identical to a
  snapshot taken immediately after the original command first executed.
- **SC-004**: A player can undo and redo through a sequence of several
  consecutive edits within one working session without any step
  silently failing, being skipped, or corrupting the layout.
- **SC-005**: After performing a new edit following one or more undos,
  attempting to redo never reproduces a layout inconsistent with what is
  currently on the canvas.

## Assumptions

- Undo/redo history is in-memory editor-session state, not part of any
  persisted document: it is not added to the `FactoryDocument` or
  `BlueprintDocument` schema, and reopening a saved factory always
  starts with an empty history.
- History depth is unbounded within a session, consistent with this
  project's documented scale (tens to low hundreds of entities) and a
  modest number of discrete edits per working session.
- Undo and redo are exposed through both header buttons and keyboard
  shortcuts (`Ctrl+Z` / `Ctrl+Y`), consistent with this project's
  existing pattern of pairing a button with a shortcut for every other
  editing action (movement arrows/buttons, `R`/**Rotate 90°**,
  `Ctrl+O`/`Ctrl+S`/`Ctrl+Shift+S`).
- Selection is not restored by undo or redo; only the factory layout
  (entities and active base) is. Selection is ephemeral editor state,
  not part of the persisted or historical layout, consistent with how
  existing atomic group edits already treat it separately.
- A command attempt that is rejected (for example, a placement rejected
  for collision) never changes the layout, so it is never recorded in
  the undo history.
