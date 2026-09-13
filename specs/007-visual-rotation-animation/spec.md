# Feature Specification: Visual Rotation Animation

**Feature Branch**: `007-visual-rotation-animation`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "fase 8 rotação visual dos itens, não foram
desenvolvidos icones ainda, mas poderia já ter a rotação visual das
máquinas, quando o usuário aperta R a máquina/item roda em seu eixo, ou
se tiver em seleção a seleção rotaciona visualmente." Refined through
clarification: the transition is animated (not instantaneous), orientation
is shown via a small arrow indicator (a placeholder until real per-block
icons exist), the arrow appears on every placed instance regardless of
footprint shape, and a multi-instance orbital rotation animates both the
shared-pivot position change and each instance's orientation together.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A rotated block visibly turns in place (Priority: P1)

A player selects a single placed instance and rotates it (the "Rotate 90°"
button or the `R` key). Today, rotation already happens in the underlying
layout — the domain accepts it, the semantic sidebar list already reports
the new rotation — but for every buildable in the confirmed public
catalog, the footprint is square, so the canvas painting looks visually
identical before and after: nothing on screen tells the player a rotation
actually happened. The player wants to see the block visibly turn, and
wants a simple way to read its current orientation now, since blocks do
not have their own icons yet.

**Why this priority**: Single-instance rotation is the common case, and it
is the one this gap affects most severely today: a square block's
rotation is currently invisible in practice. This is the concrete,
immediately valuable slice — the multi-instance case (User Story 2) builds
on the same visual mechanism this story establishes.

**Independent Test**: Can be fully tested by placing a single instance,
rotating it, and confirming (a) an orientation indicator is visible on
every placed instance regardless of footprint shape, and (b) the indicator
turns smoothly from the old angle to the new one over a perceptible
transition, rather than jumping instantly.

**Acceptance Scenarios**:

1. **Given** any placed instance, **When** the player looks at it on the
   canvas, **Then** a small orientation indicator is visible on it,
   pointing in the direction that matches its current stored rotation.
2. **Given** a single selected instance with its orientation indicator
   pointing in a given direction, **When** the player triggers rotation,
   **Then** the indicator turns smoothly from the old direction to the new
   one over a perceptible transition, instead of changing instantly.
3. **Given** a rotation transition in progress, **When** the transition
   ends, **Then** the indicator rests exactly at the new orientation,
   matching the rotation value the domain actually stored.
4. **Given** a rotation attempt the domain rejects (for example, out of
   bounds), **When** the player triggers rotation, **Then** no transition
   occurs at all and the indicator stays exactly at its unchanged current
   orientation.

---

### User Story 2 - A rotated group visibly turns and resettles together (Priority: P2)

A player selects two or more instances and rotates them together. Today,
the domain already recalculates each instance's new position and
orientation around their shared pivot in one atomic batch, but every
member jumps instantly to its final spot. The player wants to see the
whole group visibly turn around that shared pivot, each member sliding
from its old position to its new one while turning, together.

**Why this priority**: This extends User Story 1's visual mechanism to
the batch-rotation case, which already has more complex domain behavior
(position and orientation both change at once around a shared pivot). It
depends on User Story 1's visual foundation existing first, so it is the
second priority rather than an independent P1.

**Independent Test**: Can be fully tested by selecting two or more
instances, rotating them, and confirming every member visibly slides and
turns from its old position/orientation to the new one the domain already
calculates, all resettling together rather than jumping instantly.

**Acceptance Scenarios**:

1. **Given** two or more instances selected around a valid shared pivot,
   **When** the player triggers rotation, **Then** every selected instance
   visibly slides and turns from its old position/orientation to the new
   one the domain's orbital rotation already computes, instead of jumping
   instantly.
2. **Given** a group rotation transition in progress, **When** it ends,
   **Then** every instance in the group rests exactly at the final
   position and orientation the domain already validated, and the shared
   pivot behaves exactly as it does today for any rotation triggered
   afterward.
3. **Given** a group rotation attempt the domain rejects (for example, one
   member would collide or leave the bounds), **When** the player triggers
   rotation, **Then** no transition occurs at all and every member of the
   group stays exactly where it was.

---

### Edge Cases

- What happens if the player triggers another rotation while a previous
  rotation's transition is still in progress? The new transition starts
  from the current, possibly mid-turn visual state — it does not wait for
  the prior transition to finish, and it does not jump to the old final
  state before starting the new one.
- What happens to undo/redo while a rotation transition is pending or has
  just finished? Undo/redo behaves exactly as it does today (an instant,
  whole-layout snapshot restore, per Phase 6) — animating an undo/redo
  transition is out of this feature's scope.
- What happens if the active layout changes (New, Open, or a base change)
  while a rotation transition is in progress? Any pending transition is
  discarded without finishing; the newly loaded layout appears immediately
  in its own resting state, inheriting no transition from the previous
  session.
- What happens to a rotation transition while the canvas is panned or
  zoomed? The transition continues normally; the turning or sliding
  instance follows the current viewport transform every frame, exactly
  like an instance at rest already does today.
- What happens for an instance whose footprint is not square (already
  changes its visual outline on rotation today)? The same single rule
  applies to it too: its orientation indicator and turning transition work
  the same way a square instance's do.
- What happens if the application loses and regains focus, or is
  minimized, while a transition is in progress? The transition resumes or
  completes consistently once frames resume; no inconsistent state or
  incorrect jump results from any skipped frames.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST display a small orientation indicator on
  every placed instance on the canvas, regardless of that instance's
  footprint shape (square or rectangular).
- **FR-002**: The orientation indicator MUST point in the direction that
  matches the instance's currently stored rotation exactly, both at rest
  and at the end of any transition.
- **FR-003**: When a single selected instance's rotation is accepted by
  the domain, the system MUST show that instance's orientation indicator
  turning smoothly from the old orientation to the new one over a
  perceptible transition, rather than changing instantly.
- **FR-004**: When an orbital rotation of two or more selected instances
  is accepted by the domain, the system MUST show every instance in that
  group sliding and turning smoothly from its old position and orientation
  to its new one, synchronized by the same shared pivot and transition
  window the domain's own rotation already computes.
- **FR-005**: When a rotation attempt (single or group) is rejected by the
  domain, the system MUST NOT start any transition; the appearance of
  every involved instance remains exactly as it was before the attempt.
- **FR-006**: A new rotation triggered while a previous transition is
  still in progress MUST start its own transition from the current visual
  state, without waiting for the prior transition to finish and without
  discarding its in-progress visual state first.
- **FR-007**: At the end of any transition, every involved instance's
  appearance MUST match exactly the rotation and position the domain
  already validated and stored — the transition is a purely visual
  presentation of an already-accepted change and never introduces, delays,
  or alters the domain's actual outcome.
- **FR-008**: Changing the active layout (new document, open a different
  one, or a base change) while a transition is in progress MUST discard
  any pending transition without completing it; the newly loaded layout
  appears immediately in its own resting state.
- **FR-009**: This feature MUST NOT change existing undo/redo behavior —
  it remains an instant, whole-layout snapshot restore with no transition
  animation, per Phase 6.
- **FR-010**: This feature MUST NOT change any domain, catalog, or
  persistence behavior — rotation validation, collision, bounds checking,
  and saved document contents remain exactly as they are today; this
  feature is entirely a canvas presentation change.
- **FR-011**: This feature MUST NOT change the placement preview (the
  semi-transparent footprint shown while a block is armed for placement)
  or any other part of the UI outside the canvas's already-placed
  instances.

### Key Entities

- **Orientation indicator**: a small visual element (an arrow) associated
  with each instance rendered on the canvas, whose angle reflects that
  instance's current or in-transition rotation. Purely presentational —
  it has no persisted state and no domain counterpart.
- **Rotation transition**: an ephemeral, UI-local visual state that
  interpolates one or more instances' orientation (and, for a group, their
  position) between an already-domain-accepted "before" and "after" state
  over a short time window. It exists only while in progress; it is never
  persisted and is not part of any saved document.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can visually tell that an instance rotated,
  regardless of its footprint shape, in 100% of accepted rotations (both
  single-instance and group).
- **SC-002**: A player can identify any placed instance's current
  orientation at any moment when no transition is in progress, by looking
  at its orientation indicator alone.
- **SC-003**: Every accepted rotation's transition ends in exactly the
  same position and orientation the domain already validates today without
  this feature — 0% divergence between the final visual state and the
  actual stored state.
- **SC-004**: No rejected rotation attempt produces any visible motion or
  turning, in 100% of rejected attempts.
- **SC-005**: Triggering rotations repeatedly and rapidly never freezes,
  indefinitely delays, or drops the UI's responsiveness to further player
  input.

## Assumptions

- The exact transition duration is a short, perceptible window (roughly
  tens to a few hundred milliseconds) chosen during planning; this spec
  does not fix a precise value, only that it must be perceptible (User
  Story 1/2) yet not disrupt editing rhythm (SC-005).
- Undo/redo (Phase 6) stays out of this feature's scope: it keeps
  restoring a complete layout snapshot instantly, without replaying the
  visual transition an original rotation would have shown. Making it do so
  would require recognizing "this restore undoes a rotation specifically"
  from a generic whole-layout snapshot, which this project treats as
  unneeded complexity (YAGNI) until a concrete need exists.
- The orientation indicator (arrow) is a deliberately temporary
  presentation choice while buildables have no real per-block icons yet;
  it is expected to be replaced or complemented once real icons exist,
  which is not part of this feature.
- This feature introduces no new keyboard shortcut; it reuses the same
  `R` key / "Rotate 90°" button that already triggers rotation today.
- The transition is purely visual and local to the running UI session:
  closing and reopening the app, or saving and loading a document, never
  needs to reproduce a transition, since saved documents carry no
  transition state at all.
