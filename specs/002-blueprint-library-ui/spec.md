# Feature Specification: Blueprint Library UI

**Feature Branch**: `002-blueprint-library-ui` (directory identifier only — no
git branch is created for this feature; per the project constitution, the
remaining commits of Phase 4 continue directly on `master`. Branch-per-phase
begins with the next new roadmap phase.)

**Created**: 2026-09-07

**Status**: Draft

**Input**: User description: "Commit 8 of the Phase 4 plan — a user
interface, built on top of the already-shipped blueprint capture (Commit 5)
and blueprint library persistence (Commit 7), that lets a player trigger
saving their current canvas selection as a named blueprint into the local
library, and lets them see the blueprints already saved in that library,
including any safe warnings about entries that could not be read. No
inserting a blueprint into a canvas, no editing, deleting, renaming,
importing, or exporting a blueprint — those remain separate, later
capabilities."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Save the current selection as a named blueprint (Priority: P1)

A player has arranged and configured a group of blocks on the canvas that
they want to reuse later. They select those blocks, choose to save them as
a blueprint, give it a name, and confirm. The blueprint is captured and
stored in their local library without altering the canvas they were working
on.

**Why this priority**: This is the entire reason a blueprint library needs
a user interface — the capture and persistence machinery already exists
(Commits 5 and 7), but a player has no way to actually use it without a way
to trigger a save from inside the editor. Without this, the underlying
capability delivers no value to the player.

**Independent Test**: Select one or more instances on the canvas, trigger
the save action, provide a name, confirm, and verify the selection remains
exactly as it was on the canvas (nothing is removed or altered) while a new
blueprint now exists in the local library with that name.

**Acceptance Scenarios**:

1. **Given** one or more instances are selected on the canvas, **When** the
   player triggers "save as blueprint" and confirms a non-blank name,
   **Then** a new blueprint is captured from the current selection and
   saved to the local library, and the canvas is left completely unchanged.
2. **Given** no instances are selected, **When** the player looks for the
   save-as-blueprint action, **Then** it is unavailable, so the player is
   never invited to attempt a save that cannot succeed.
3. **Given** the player has opened the save dialog, **When** they attempt
   to confirm with a blank or whitespace-only name, **Then** the save is
   not submitted and no blueprint is created.
4. **Given** the player has opened the save dialog, **When** they cancel
   instead of confirming, **Then** no blueprint is created and the canvas
   selection is unchanged.

---

### User Story 2 - See what is already saved in the library (Priority: P2)

A player wants to know what blueprints they have already saved before
deciding whether to save a new one or reuse an old design. They open the
blueprint library view and see each saved blueprint's name, how many
modules it contains, and when it was last saved.

**Why this priority**: Saving is only half the value; a library a player
cannot see is not meaningfully different from not having one. This is
naturally P2 rather than P1 because the very first blueprint a player ever
saves can only be viewed after User Story 1 has already produced one — but
the two are usually delivered together in the same commit.

**Independent Test**: With one or more blueprints already saved in the
library (via User Story 1 or a prior session), open the library view and
confirm each blueprint's name, module count, and last-saved time are
visible, matching what was actually saved.

**Acceptance Scenarios**:

1. **Given** at least one blueprint exists in the local library, **When**
   the player opens the library view, **Then** every valid blueprint
   appears with its name, module count, and last-saved time.
2. **Given** the library is completely empty, **When** the player opens the
   library view, **Then** they see a clear indication that no blueprints
   have been saved yet, rather than an empty or confusing screen.
3. **Given** the player just saved a new blueprint (User Story 1), **When**
   they next view the library, **Then** the newly saved blueprint appears
   without the player needing to perform any extra manual refresh step.

---

### User Story 3 - Unreadable library entries are visible as safe warnings, not silent gaps (Priority: P3)

Some of the player's stored blueprint files may have become unreadable
(edited by hand, corrupted, or referencing content no longer available) —
the underlying library already detects this and never hides other
blueprints because of it, but until now nothing on screen actually told the
player it happened. The player sees a safe, generic indication that
something needed attention, without being shown technical details.

**Why this priority**: The library's safety guarantees (Commit 7) are only
actually useful to the player once they are visible; otherwise a player
never learns that action might be needed and could wrongly assume
everything is fine. Lower priority than P1/P2 because it depends on an edge
case (a broken file) rather than the everyday path.

**Independent Test**: Place one or more valid blueprints and at least one
invalid or duplicate-identity file directly into library storage (bypassing
the UI), open the library view, and confirm the valid blueprints are listed
normally while a safe, generic notice reflects the presence of the
invalid/duplicate entry, without technical detail.

**Acceptance Scenarios**:

1. **Given** the library contains both valid blueprints and at least one
   file that cannot be read as a valid, unambiguous blueprint, **When** the
   player opens the library view, **Then** every valid blueprint is listed
   normally and a safe, generic notice is also shown reflecting the
   unreadable entry.
2. **Given** a safe, generic notice is shown for an unreadable entry,
   **When** the player reads it, **Then** it contains no file path, no raw
   file content, and no technical identifier — consistent with the
   library's existing privacy guarantee.

### Edge Cases

- What happens if the player triggers "save as blueprint" and the
  underlying save fails (e.g. storage cannot be written to)? The player
  sees a safe, generic failure notice; the canvas and its selection remain
  unchanged, and no partial blueprint is left behind (inherited from the
  library's existing atomic-save guarantee; see FR-011).
- What happens if a blueprint's stored catalog no longer matches the
  currently active catalog exactly? The blueprint still appears in the
  listing — this was already decided by the blueprint document schema and
  library capabilities delivered earlier — with a visible but non-blocking
  indication that it does not match exactly (see FR-010).
- What happens if the player changes the canvas selection while the save
  dialog is still open? The save dialog is a short, focused interaction
  that blocks other canvas edits until it is confirmed or canceled,
  consistent with this editor's existing confirmation-dialog behavior, so
  this cannot happen (see Assumptions).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST let the player capture their current canvas
  selection as a new blueprint and provide it a name before it is saved to
  the local blueprint library.
- **FR-002**: System MUST require a non-blank name, after trimming leading
  and trailing whitespace, before a save-as-blueprint action can be
  confirmed.
- **FR-003**: The save-as-blueprint action MUST be unavailable when no
  canvas instance is currently selected, so the player is never invited to
  attempt a save that cannot succeed.
- **FR-004**: A successful save-as-blueprint action MUST leave the canvas,
  its selection, and all entity identifiers completely unchanged —
  capturing a blueprint is never destructive to the source factory.
- **FR-005**: Canceling the save-as-blueprint action at any point before
  confirmation MUST result in no blueprint being created and MUST leave the
  canvas selection unchanged.
- **FR-006**: System MUST display every valid blueprint currently in the
  local library, showing at minimum its name, its module count, and when
  it was last saved.
- **FR-007**: When the local library contains no blueprints at all, system
  MUST show a clear, explicit indication of that, rather than an empty or
  ambiguous display.
- **FR-008**: A blueprint that was just saved successfully MUST appear in
  the library display the next time the player views it, without requiring
  any manual refresh action beyond viewing it.
- **FR-009**: When the library reports one or more entries that could not
  be read as a valid, unambiguous blueprint, system MUST display a safe,
  generic notice reflecting their presence, using only the safe reason
  categories the library already provides — never a file path, raw
  content, or technical identifier.
- **FR-010**: When a listed blueprint's stored catalog does not exactly
  match the currently active catalog, system MUST show a visible,
  non-blocking indication of that mismatch using the compatibility
  information the library already provides for that entry.
- **FR-011**: If a save-as-blueprint action fails, system MUST show a safe,
  generic failure notice and MUST leave the canvas and any previously saved
  blueprints completely unchanged.
- **FR-012**: System MUST NOT insert a listed blueprint into the canvas,
  and MUST NOT edit, delete, rename, import, or export a blueprint from
  this interface — this capability is limited to triggering a save and
  viewing the listing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can save their current selection as a named
  blueprint in under 15 seconds from deciding to do so (select, trigger,
  name, confirm), without needing to leave the editor.
- **SC-002**: 100% of blueprints successfully saved through this interface
  are visible the next time the player views the library, with no manual
  recovery step.
- **SC-003**: 100% of attempts to confirm a save with a blank name are
  prevented before any blueprint is created.
- **SC-004**: Zero canvas edits — to instance count, position, rotation, or
  identifiers — occur as a side effect of saving a blueprint, verified
  across every acceptance scenario.
- **SC-005**: Every displayed warning about an unreadable or conflicting
  library entry contains zero file paths, zero raw file contents, and zero
  technical identifiers, across every tested scenario.
- **SC-006**: A player with zero saved blueprints sees an explicit
  "no blueprints yet" indication rather than an empty or ambiguous screen,
  100% of the time.

## Assumptions

- The blueprint library listing is presented as its own persistent,
  scrollable section of the existing editor sidebar, alongside the
  construction-base and blocks sections already there, rather than as a
  separate on-demand window or overlay. This mirrors the sidebar's existing
  pattern of always-visible sections and avoids introducing a new
  interaction paradigm (window/panel management) for an MVP commit; an
  internal scroll area keeps an arbitrarily long list from crowding the
  fixed-width sidebar. This is the one design choice in this spec with a
  genuinely competitive alternative (an on-demand view opened by a button)
  — worth revisiting with `/speckit-clarify` if this default is unwanted.
- The save-as-blueprint action is presented as a contextual action
  available only while at least one canvas instance is selected —
  mirroring how other selection-scoped actions (move, rotate, remove) are
  already presented in this editor — rather than as an always-visible
  header command.
- Confirming a save opens a short, focused interaction (consistent with
  this editor's existing confirmation-modal pattern) that blocks other
  canvas edits until the player confirms or cancels, rather than allowing
  concurrent canvas editing while naming a blueprint.
- The blueprint library always reads from and writes to the single, fixed,
  per-user storage location already defined by the library capability
  (Commit 7); this interface introduces no new storage location,
  configuration, or user-facing setting.
- This capability does not add a dedicated keyboard shortcut for saving a
  blueprint in this commit; the existing document shortcuts (new, open,
  save, save-as) are unaffected. A dedicated shortcut may be added later
  without changing this specification.
- Catalog-compatibility indication for a listed blueprint reuses the same
  compatibility concept and safe presentation already established for
  opened factory documents; this capability does not introduce a new
  compatibility model.
- No blueprint is inserted into the canvas, edited, deleted, renamed,
  imported, or exported by this interface; those remain separate, later
  capabilities per the project roadmap (Phase 5 and beyond).
