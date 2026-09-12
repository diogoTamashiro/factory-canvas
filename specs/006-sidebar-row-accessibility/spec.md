# Feature Specification: Sidebar Instance Row Accessibility

**Feature Branch**: `006-sidebar-row-accessibility`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "fase 7" — Roadmap Phase 7 (Post-MVP phases §
"7. Accessibility and polish"): "Review the selectable sidebar row and, if
the egui version allows it without clipping, use a control with more
explicit button/focus semantics. Always keep the complete label with ID,
name, origin, footprint, and rotation." (`docs/roadmap.md`). Per the
project constitution's "Workflow and Branching" section, this phase is
developed on its own spec-kit feature branch, same as every phase from
Phase 5 onward.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Assistive technology recognizes a sidebar row as a real selectable control (Priority: P1)

A player using a screen reader or other assistive technology navigates the
"INSTANCES ON CANVAS" list in the sidebar. Today, every row is exposed as
plain, static text: assistive technology has no way to tell that clicking
a row selects that instance, and no way to tell which rows are currently
selected other than a color difference that is invisible to non-sighted
navigation. This player needs each row to be announced as an actionable,
selectable control, and to hear its current selected/unselected state
directly, exactly the way the sidebar's block palette already announces
its own selectable options.

**Why this priority**: This is the concrete accessibility gap the roadmap
names for this phase. Canvas painting already has a parallel semantic
list specifically so assistive technology has an equivalent way to
perceive the layout (per this project's UI architecture decision); a
semantic list item that cannot itself be perceived as selectable or
report its own selection state defeats that purpose for exactly the
rows that need it most.

**Independent Test**: Can be fully tested by inspecting the accessibility
information exposed for each row in a layout with several instances, some
selected and some not, and confirming every row reports both an
interactive/selectable role and an accurate, individual selected or
unselected state — independently of any visual/color-only inspection.

**Acceptance Scenarios**:

1. **Given** a layout with at least one placed instance, **When** the
   player inspects that instance's sidebar row through assistive
   technology, **Then** the row is exposed as an interactive, selectable
   control rather than plain static text.
2. **Given** a layout with two or more placed instances where at least
   one is selected and at least one is not, **When** the player inspects
   each row through assistive technology, **Then** each row reports its
   own selected or unselected state accurately, matching the current
   selection exactly.
3. **Given** two or more instances selected together (via `Shift` or
   `Ctrl`), **When** the player inspects each selected row, **Then**
   every one of them independently reports itself as selected, not only
   the most recently clicked one.
4. **Given** any row's reported information, **When** the player compares
   it against what the row visibly displays, **Then** the identifier,
   name, origin, footprint, rotation, and configured product (or the
   absence of one) are all present, exactly as the row shows today —
   nothing is shortened, cut off, or omitted.

---

### User Story 2 - Sidebar rows look and behave like the app's other selectable controls (Priority: P2)

A sighted player using the mouse or keyboard looks at the "INSTANCES ON
CANVAS" list. Today, each row looks like plain text with only a subtle
color change to hint at selection, unlike the block palette just above
it, whose options already look and feel like real, selectable buttons.
This player wants the instance rows to carry the same clear, familiar
selectable-control affordance the rest of the sidebar already uses, and
wants to be able to see, while tabbing through the sidebar with the
keyboard, exactly which row currently has focus.

**Why this priority**: This is the "and polish" half of this phase: a
visual/interaction consistency improvement that makes the existing
selection mechanic easier to notice and trust. It is valuable on its own
once User Story 1's underlying control change (if adopted) is in place,
but it does not carry the accessibility-correctness stakes User Story 1
does.

**Independent Test**: Can be fully tested by comparing the rendered
appearance and keyboard-focus behavior of an instance row against the
block palette's existing option rows, and confirming a focused row is
visually distinguishable from an unfocused one.

**Acceptance Scenarios**:

1. **Given** the sidebar's instance list is rendered, **When** the player
   compares a row's appearance to a block palette option's appearance,
   **Then** both convey the same kind of selectable-control affordance
   (for example, a matching frame, highlight, or press/hover feedback
   style), unless User Story 1's fallback (see Edge Cases) applies.
2. **Given** the player moves keyboard focus onto a sidebar instance row,
   **When** they observe the sidebar, **Then** they can visually tell
   that row currently has focus, distinct from every other row.

---

### Edge Cases

- What happens if no available control can report the row as selectable
  with accurate state (User Story 1) while still showing every row's
  complete information without cutting any of it off, across the longest
  labels the active catalog can produce? The sidebar list keeps its
  current rendering unchanged rather than lose any information — this is
  a fully acceptable outcome of this feature, not a defect, and User
  Story 2's visual-consistency change does not apply either in that case.
- What happens to the existing color-based selected/unselected distinction
  if the underlying control changes? It may be replaced by whatever
  selected-state affordance the new control provides natively, provided
  selected and unselected rows remain at least as easy to tell apart as
  they are today.
- What happens when the layout has no instances at all? No rows render,
  exactly as today; this feature does not change that.
- What happens to a row's keyboard focus when the instance list changes
  between frames (an instance is added, removed, or the set of IDs
  otherwise changes)? Focus is ephemeral UI state with no requirement to
  persist across such a change, consistent with how selection and other
  transient editor state are already treated elsewhere in this project.
- What happens to the existing mouse selection modifiers (plain click
  replaces, `Shift` adds, `Ctrl` toggles)? They continue to work exactly
  as they do today, for every row, regardless of which outcome this
  feature reaches.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The sidebar's "INSTANCES ON CANVAS" list MUST expose each
  row's selected or unselected state to assistive technology directly,
  not only through a visual-only color difference.
- **FR-002**: The sidebar's "INSTANCES ON CANVAS" list MUST expose each
  row as an interactive, selectable control to assistive technology,
  rather than as plain static text, subject to FR-005's fallback.
- **FR-003**: Every row MUST continue to display its complete existing
  information — identifier, name, origin, footprint, rotation, and
  configured product or the absence of one — in full, with no part of it
  truncated, clipped, or hidden, regardless of which control renders the
  row.
- **FR-004**: The existing mouse selection interaction (a plain click
  replaces the selection, `Shift` adds, `Ctrl` toggles) MUST remain
  exactly as it is today for every row, regardless of which outcome this
  feature reaches.
- **FR-005**: If no available control can satisfy FR-002 without
  violating FR-003 for any label the active catalog and layout can
  produce, the sidebar list MUST keep its current rendering unchanged;
  not satisfying FR-002 under this specific condition is an acceptable,
  fully compliant outcome, not a defect.
- **FR-006**: Whenever FR-005's fallback does not apply, each row's
  visual presentation MUST convey the same kind of selectable-control
  affordance already used by the sidebar's block palette options.
- **FR-007**: Whenever FR-005's fallback does not apply, a sidebar
  instance row that currently holds keyboard focus MUST be visually
  distinguishable from a row that does not.
- **FR-008**: This feature MUST NOT change which instances can be
  selected, how many can be selected at once, or any sidebar section
  other than the "INSTANCES ON CANVAS" list (the block palette, the
  blueprint library section, and the editor status section are
  unaffected).
- **FR-009**: This feature MUST NOT alter any domain, catalog, or
  persistence behavior — canvas geometry, layout validation, and saved
  document contents are unaffected.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A screen reader user navigating the sidebar's instance list
  can determine, for every row and without relying on color, whether that
  row is currently selected.
- **SC-002**: A screen reader user navigating the sidebar's instance list
  can determine that each row is an actionable, selectable control, not
  inert text.
- **SC-003**: 100% of the information a row currently displays
  (identifier, name, origin, footprint, rotation, and product) remains
  fully visible with zero rows truncated or clipped, across the full
  range of labels the active catalog and layout can produce.
- **SC-004**: A sighted keyboard-only user can see which row currently
  holds keyboard focus while tabbing through the sidebar's instance list,
  whenever FR-005's fallback does not apply.
- **SC-005**: Mouse-driven selection (plain click, `Shift`+click,
  `Ctrl`+click) produces identical results before and after this feature,
  in 100% of cases.

## Assumptions

- The sidebar's block palette already renders its options with explicit
  selectable/focus semantics and a matching visual affordance; this
  feature evaluates extending the same class of control to the instance
  list, not introducing a new one.
- "Complete information" means everything a row currently displays for
  that instance: identifier, name, origin, rotated footprint, rotation,
  and configured product (or its absence) — exactly as today, with
  nothing removed.
- This feature is scoped to the "INSTANCES ON CANVAS" list only. The
  roadmap's own wording ("ID, name, origin, footprint, and rotation")
  names exactly that row's content and no other sidebar section.
- Whether a control exists that satisfies both FR-002 and FR-003 across
  every label the active catalog can produce is a technical question
  resolved during planning and implementation, not predetermined by this
  spec; either resulting outcome (adopt the new control, or keep today's
  rendering per FR-005) fully satisfies this feature.
- No new keyboard shortcut is introduced beyond whatever activation a
  chosen control natively provides to an already-focused row.
- This is presentation- and accessibility-layer work only; no domain,
  catalog, or persistence contract changes.
