# Phase 0 Research: Sidebar Instance Row Accessibility

No item in Technical Context was marked `NEEDS CLARIFICATION`. This
feature carries exactly one real technical uncertainty — the one
spec.md's FR-005 already named explicitly ("if no available control can
satisfy [selectable semantics] without... clipping") — and it is
resolved empirically below, not assumed.

## Decision 1: Replace the instance row's `egui::Label` with `egui::Button::selectable`

**Decision**: In `src/egui_app.rs`'s `sidebar_ui` "INSTANCES ON CANVAS"
loop, replace:

```rust
ui.add_sized(
    [ui.available_width(), 0.0],
    egui::Label::new(RichText::new(instance_semantic_label(...)).size(11.0).color(...))
        .wrap()
        .sense(egui::Sense::click()),
)
```

with:

```rust
ui.add_sized(
    [ui.available_width(), 0.0],
    Button::new(RichText::new(instance_semantic_label(...)).size(11.0))
        .selected(self.selected.contains(id))
        .wrap(),
)
```

The existing `response.clicked_by(egui::PointerButton::Primary)` +
modifier-reading dispatch immediately below is untouched: `Button::ui()`
returns the same `egui::Response` type `Label::ui()` does, with the same
`clicked_by`/`ctx().input()` API surface, so the click/`Shift`/`Ctrl`
selection-mode logic requires zero changes (spec FR-004).

**Rationale — resolving FR-005's named uncertainty empirically**: A
disposable spike (`tests/spike_button_wrap.rs`, written, run, and
deleted before this document was authored, per
`docs/engineering-standards.md` §YAGNI "treat spikes as disposable")
measured `Button::new(text).selected(true).wrap()`'s rendered height at
this project's real sidebar width (264px, `Panel::left("base_sidebar")
.exact_size(264.0)`) against this project's real row-label format
(`instance_semantic_label`'s `"#{id} · {name} · origin ({x}, {y}) ·
{w} × {h} · {rotation}° · {product}"`) for both a short label and a
long one:

| Widget | Short label height | Long label height (3 wrapped rows) |
|---|---:|---:|
| Current `Label` (`.wrap()`) | 13px | 39px |
| Candidate `Button` (`.selected(true).wrap()`) | 18px | 41px |

Height scales proportionally with wrapped row count for `Button` exactly
as it already does for `Label` (roughly 3× for 3 wrapped lines in both
cases) — the text is not clipped, cut, or elided at any length this
project's actual catalog and layout can produce. `egui`'s own source
(`egui-0.36.1/src/atomics/atom_layout.rs` — the shared layout engine
`Button` is built on) confirms why: when a `Button`'s wrap mode is not
`TextWrapMode::Extend`, the text atom is marked `shrink`, which "will
make the text truncate **or shrink** depending on wrap_mode" — and
`Button::wrap()` sets `TextWrapMode::Wrap`, the same mode `Label::wrap()`
uses, not `TextWrapMode::Truncate`. `Button` and `Label` share the same
underlying text-wrapping code path in this egui version; there is no
version-specific gap the roadmap's "if the egui version allows it"
condition needs to guard against. **FR-005's fallback is therefore not
triggered**: this feature adopts the new control everywhere, and User
Story 2 (FR-006/FR-007's visual-consistency requirements) applies
unconditionally.

**Alternatives considered**:
- *`ui.selectable_label(selected, text)`*: rejected as the literal call —
  it is a thin convenience wrapper that constructs exactly
  `Button::selectable(selected, text)` internally
  (`egui-0.36.1/src/ui.rs`), so it is equivalent in every observable way.
  `Button::new(...).selected(...).wrap()` is used directly instead only
  because it must also set `.wrap()` explicitly (the block palette's own
  existing `Button::new(label).selected(selected)` call relies on a
  fixed-height `add_sized` row and does not need wrapping), keeping the
  explicit-over-implicit style this file's existing block-palette call
  already established.
- *A custom `Frame` + `Label` combination hand-rolling a selectable look*:
  rejected — `docs/engineering-standards.md` §KISS ("choose the solution
  with the fewest concepts") and §DRY ("do not abstract after the first
  repetition") both favor reusing the exact primitive the block palette
  two sections above already uses successfully, rather than inventing a
  second, parallel "looks selectable" mechanism in the same file.
- *Leaving the row as `Label` and only adding an AccessKit-only role
  override without changing the visual widget*: rejected — egui's
  `WidgetInfo`/AccessKit bridge derives role and toggled state from the
  widget that actually rendered (`Response::widget_info`); there is no
  supported way to report `Role::Button` / `Toggled` from a `Label`
  response without either hand-rolling the exact `WidgetInfo` construction
  `Button` already does (duplicating egui-internal logic un-KISS-ly) or
  switching to a widget that already does it correctly.

## Decision 2: Focus and hover visuals come from the widget swap itself, not new code

**Decision**: No new painting code is added for FR-006 (visual
selectable-control affordance) or FR-007 (visible keyboard focus). Both
are satisfied automatically by `Button`'s existing style resolution
(`Style::button_style`, `egui-0.36.1/src/widget_style.rs`), which already
paints a distinct frame/stroke per `WidgetState`
(`Inactive`/`Hovered`/`Active`, where `Active` already includes
`response.has_focus()` — `egui-0.36.1/src/widget_style.rs`'s
`Response::widget_state()`) and a distinct `selection.bg_fill`/
`selection.stroke` pair when `.selected(true)` is set — the exact same
style resolution the block palette's existing `Button::new(label)
.selected(selected)` call already goes through today.

**Rationale**: The current `Label`-based row has no built-in
hover/focus/active painting at all — its only visual selection cue is
the row's own hand-picked `RichText` color
(`if self.selected.contains(id) { ACCENT } else { TEXT_PRIMARY }`) with
no frame, no hover feedback, and no focus ring, which is exactly the "a
control with more explicit button/focus semantics" gap the roadmap names.
Switching to `Button` inherits the same visuals the block palette options
already display one section above, satisfying "the same kind of
selectable-control affordance" (FR-006) and "visually distinguishable...
focus" (FR-007) as a direct, un-hand-coded consequence of using a
standard, already-focusable (`Sense::click()` already implies
`Sense::FOCUSABLE`, confirmed in `egui-0.36.1/src/sense.rs`) egui
control — no bespoke focus-ring painting logic is written or needed.

**Alternatives considered**:
- *Manually painting a focus ring around the existing `Label` when
  `response.has_focus()`*: rejected — would duplicate egui's own
  already-correct, already-used-elsewhere `WidgetVisuals`/`WidgetState`
  painting logic instead of reusing it, contradicting §DRY and adding
  bespoke, untested paint code for a solved problem.

## Decision 3: The row's selected-state color logic is removed, not kept alongside the new control

**Decision**: The manual `if self.selected.contains(id) { ACCENT } else
{ TEXT_PRIMARY }` text-color branch is deleted from the row's
`RichText`; `.selected(self.selected.contains(id))` on the `Button`
fully replaces it as the single source of the row's selected appearance.

**Rationale**: `docs/engineering-standards.md` §DRY ("keep one source of
truth") argues against maintaining two parallel "this row is selected"
visual signals (the button's own built-in selection styling, plus a
hand-picked text color duplicating the same boolean) once the control
that already renders the correct, egui-native selected appearance is in
place. Spec.md's Edge Cases already anticipate and permit this
("[the color distinction] may be replaced by whatever selected-state
affordance the new control provides natively, provided selected and
unselected rows remain at least as easy to tell apart").

**Alternatives considered**:
- *Keep the manual color branch on top of `.selected(...)`*: rejected —
  redundant once the button's own selection styling already changes the
  row's foreground color (`self.visuals.selection.stroke.color`,
  confirmed in `egui-0.36.1/src/widget_style.rs`'s `button_style`); two
  simultaneous, independently-maintained sources for the same boolean is
  exactly the duplication §DRY warns against.

## Decision 4: Scope stays limited to the "INSTANCES ON CANVAS" loop; the block palette and blueprint library rows are untouched

**Decision**: Only the instance-row loop inside `sidebar_ui` changes.
`block_palette_ui` (already `Button`-based) and
`blueprint_library_section_ui` (its own separate, unaffected `Label`/
`Button` rows for name, metadata, and the "Insert" action) are not
touched.

**Rationale**: Spec.md's own scope boundary (FR-008, Assumptions) is
explicit: "The roadmap's own wording... names exactly that row's content
and no other sidebar section." The block palette already satisfies this
feature's own target state; touching it would be a scope-creeping,
opportunistic change `docs/engineering-standards.md` §Git explicitly
forbids ("do not mix broad reformatting with functional changes").

**Alternatives considered**:
- *Also apply the same treatment to the blueprint library's per-entry
  name/metadata labels for consistency*: rejected as out of scope for
  this feature — those rows are not selectable at all today (they are
  informational, with a separate explicit "Insert" `Button`), so there is
  no equivalent accessibility gap to close there; spec.md's Edge Cases
  and FR-008 already settle this explicitly.
