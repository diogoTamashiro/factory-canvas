# Feature Specification: Blueprint Library Persistence

**Feature Branch**: `001-blueprint-library` (directory identifier only — no git
branch is created for this feature; per the project constitution, the
remaining commits of Phase 4 continue directly on `master`. Branch-per-phase
begins with the next new roadmap phase.)

**Created**: 2026-09-07

**Status**: Draft

**Input**: User description: "Commit 7 of the Phase 4 plan — a local,
offline, persistent library that saves valid blueprints to per-user storage
and lists them, so a blueprint captured in one session is still there the
next time the application runs. No canvas insertion, no edit/delete/rename,
no import/export, and no user interface in this capability — those are
separate, later capabilities."

## Clarifications

### Session 2026-09-07

- Q: When the library lists the saved blueprints, what should be the
  primary ordering criterion shown to the player? (FR-004) → A: Alphabetical
  by name (A→Z), with the blueprint's underlying identifier as a tiebreak
  when two or more blueprints share the same display name.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Blueprints survive a restart (Priority: P1)

A player has a blueprint that was captured and saved earlier. They close the
application and reopen it later (possibly after restarting their computer).
The blueprint they saved is still available — nothing was lost just because
the application was not running.

**Why this priority**: This is the entire point of the capability. Without
durable storage across restarts, a "blueprint library" provides no value
over transient, in-memory state — a player would have to recreate their
production modules every session.

**Independent Test**: Save one valid blueprint to storage, end the process
entirely, start a fresh instance of the library component, and confirm the
blueprint is discovered and appears in the list with correct name, module
count, and last-updated time.

**Acceptance Scenarios**:

1. **Given** a blueprint was saved to storage in a previous run, **When** a
   new run lists the library, **Then** that blueprint appears with its name,
   module count, and last-updated time.
2. **Given** the per-user storage location does not exist yet (first run),
   **When** the library is used for the first time, **Then** the location is
   created automatically and the save/list operations succeed without the
   player needing to do anything manually.

---

### User Story 2 - One broken file never hides the rest of the library (Priority: P2)

A player has several saved blueprints. One of the stored files becomes
unreadable as a valid blueprint (for example, it was hand-edited, corrupted,
or refers to game content that is no longer available). The player must
still see and use all of their other, valid blueprints — the one broken file
must not block, crash, or hide the rest of the library.

**Why this priority**: Files outside the application's control can always
degrade (manual edits, disk issues, data/catalog updates). A single bad file
silently locking a player out of their entire library — or crashing the
app — would be a severe, trust-destroying regression compared to the
per-document safety already guaranteed for factories and individual
blueprints.

**Independent Test**: Place several valid blueprint files and one invalid
file in storage, list the library, and confirm every valid blueprint is
listed while the invalid one produces exactly one safe warning and nothing
else changes.

**Acceptance Scenarios**:

1. **Given** N valid blueprint files and 1 invalid file are present in
   storage, **When** the library is listed, **Then** all N valid blueprints
   appear and exactly one safe warning is shown for the invalid file.
2. **Given** an invalid file's warning is shown, **When** the player reads
   it, **Then** it contains no file path, no raw file content, and no
   technical identifier — only a safe, generic notice.

---

### User Story 3 - Unusual storage contents behave predictably (Priority: P3)

The storage location can end up containing things the library did not put
there itself: unrelated files, links to files elsewhere, or — in rare
cases — two files that both claim to be the same blueprint (for example
after a manual copy). In every case, the player sees a safe, predictable
result rather than confusing duplicate entries, a crash, or silent data
loss.

**Why this priority**: These are edge cases rather than the everyday path,
but a shared, per-user storage folder is not fully within the application's
control, and getting this wrong risks silent data loss or a confusing,
duplicated library — lower priority than P1/P2 but still required for the
capability to be trustworthy.

**Independent Test**: Add a non-blueprint file, a symbolic link, and two
files that decode to the same blueprint identity into storage, then list the
library and confirm: the unrelated file is silently ignored, the symbolic
link is silently ignored, and the duplicate identity produces a single safe
warning rather than two ambiguous list entries.

**Acceptance Scenarios**:

1. **Given** a file in storage that is not a blueprint file at all, **When**
   the library is listed, **Then** that file is silently ignored and does
   not appear as an entry or a warning.
2. **Given** a symbolic link is present in the storage location, **When**
   the library is listed, **Then** the link is ignored rather than being
   followed or listed.
3. **Given** two stored files both identify themselves as the same
   blueprint, **When** the library is listed, **Then** the player never
   sees two separate, ambiguous entries for that identity — the conflict is
   resolved safely and reported the same way any other invalid entry would
   be.
4. **Given** saving a new blueprint would collide with an identifier already
   present in storage, **When** the save happens, **Then** the system
   detects the collision and uses a different identifier instead of
   overwriting the existing, unrelated blueprint.

### Edge Cases

- What happens when the storage location exists but cannot be created or
  written to at all (e.g. permissions)? The save operation must fail safely
  and report a safe warning rather than crash; previously stored blueprints
  and the previously known list must remain unchanged (see FR-011).
- What happens when a save is interrupted partway (e.g. the process is
  killed mid-write)? The previously stored blueprint file must not be left
  in a partial or corrupted state (see FR-011).
- What happens when the same storage contents are listed twice in a row
  with nothing changed in between? The list must come back in the same
  order both times (see FR-004).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist a valid blueprint to per-user storage so
  that it can be found again after the application has fully stopped and
  started again.
- **FR-002**: System MUST create the storage location automatically the
  first time it is needed, without failing the save and without requiring
  manual setup by the player.
- **FR-003**: System MUST list every valid blueprint found in storage,
  showing at minimum its name, its module (node) count, and when it was
  last updated.
- **FR-004**: System MUST list valid blueprints sorted alphabetically by
  name (ascending), using the blueprint's underlying identifier as a
  tiebreak when two or more blueprints share the same display name, so
  that repeated listings of the same unchanged storage contents always
  return entries in identical order.
- **FR-005**: System MUST ignore any file in the storage location that is
  not a blueprint file, without treating its presence as an error.
- **FR-006**: System MUST ignore symbolic links found in the storage
  location rather than following or listing them.
- **FR-007**: When a file in storage cannot be read as a valid blueprint,
  system MUST exclude only that specific file from the list and surface a
  safe warning for it, while every other valid blueprint remains listed.
- **FR-008**: Any warning shown about an invalid or conflicting blueprint
  file MUST NOT reveal that file's full path, its raw contents, or any
  technical identifier — only a safe, generic notice.
- **FR-009**: When two or more stored files identify themselves as the same
  blueprint, system MUST NOT present them as separate, ambiguous list
  entries; the conflict MUST be resolved safely and reported like any other
  invalid-entry warning (see FR-007, FR-008).
- **FR-010**: When saving a new blueprint would collide with an identifier
  already present in storage, system MUST detect the collision and use a
  different identifier before persisting, rather than overwriting the
  existing, unrelated blueprint.
- **FR-011**: A save that fails or is interrupted MUST leave any
  previously stored blueprint file, and the previously known library
  listing, unchanged — no partial or corrupted writes are ever visible.
- **FR-012**: System MUST NOT insert a blueprint into a canvas, and MUST NOT
  edit, delete, rename, import, or export blueprints — this capability is
  limited to saving and listing.

### Key Entities *(include if feature involves data)*

- **Blueprint Library**: The collection of blueprints discovered in the
  per-user storage location. Not a game entity — a persistence-facing
  concept with a save behavior and a listing behavior.
- **Blueprint Library Entry**: The summary shown per blueprint in the
  library listing — name, module (node) count, last-updated time. Distinct
  from the full blueprint document itself (already defined by the blueprint
  document schema delivered in the prior capability).
- **Invalid Entry Warning**: A safe, non-leaking notice associated with one
  stored file that could not be read as a valid, unambiguous blueprint.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A blueprint saved in one application run is found in the
  library listing in every subsequent run, 100% of the time, with no
  manual recovery steps.
- **SC-002**: With exactly one corrupted or invalid file present among N
  otherwise-valid blueprint files, all N valid blueprints still appear in
  the listing, and exactly one safe warning is shown.
- **SC-003**: Across every tested invalid-file or conflicting-file
  scenario, zero warning messages contain a file system path, raw file
  content, or a technical identifier.
- **SC-004**: Listing the same, unchanged storage contents twice in a row
  always returns the entries in identical order both times.
- **SC-005**: A save that is interrupted or fails never results in a
  missing or corrupted previously-saved blueprint, verified by re-listing
  the library after a simulated failure.

## Assumptions

- The storage location is a single, fixed, per-user, per-application,
  OS-standard location and is not user-configurable in this phase —
  consistent with the product's offline-first, single-user design.
- When two stored files claim the same blueprint identity (FR-009), exactly
  one is treated as the valid entry (chosen by the same deterministic order
  used for listing) and the other is treated as an invalid/conflicting file
  for warning purposes (FR-007, FR-008); the precise mechanical tie-break is
  a technical decision for the implementation plan, not fixed by this spec.
- This capability has no user interface of its own. Triggering a save from
  player action and rendering the listing on screen belong to a separate,
  later capability that calls into this one.
- No network access, cloud sync, or telemetry are involved — the library is
  fully offline and local to the machine, consistent with the project's
  offline-first requirement.
- No new or external game/catalog data is required to implement this
  capability. It operates on blueprints that already validate against
  whichever catalog is active; catalog-compatibility handling (mismatch as
  warning vs. real incompatibility as error) was already defined by the
  blueprint document schema delivered in the prior capability and is reused
  as-is, not redesigned here.
