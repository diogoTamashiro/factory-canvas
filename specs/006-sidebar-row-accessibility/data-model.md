# Phase 1 Data Model: Sidebar Instance Row Accessibility

This feature adds no domain, catalog, or persistence entity, and no new
Rust type at all — it is a widget substitution inside one existing
rendering method (research.md Decision 1). There is nothing under
`src/domain/`, `src/persistence/`, or any document schema for this
document to describe.

What follows instead is the observable **AccessKit contract** each
sidebar instance row must satisfy after this change — the closest
analog to "entity/fields" for a presentation-only feature, and exactly
what this feature's tests assert against.

## Sidebar instance row — AccessKit contract

For every instance currently in `self.layout` (unchanged: rendered by
`sidebar_ui`'s existing "INSTANCES ON CANVAS" loop, one row per
instance, in the same existing ascending-`EntityId` order):

```text
role            : accesskit::Role::Button          (was: Role::Label)
toggled         : Some(Toggled::True)  if the row's EntityId is in
                                        self.selected
                  Some(Toggled::False) otherwise
                                                    (was: absent — no
                                                     selection state was
                                                     ever exposed)
label           : instance_semantic_label(resolved, catalog) — UNCHANGED
                  string content and format: "#{id} · {name} ·
                  origin ({x}, {y}) · {w} × {h} · {rotation}° ·
                  {product}"
disabled        : false (rows are never individually disabled by this
                  feature; unchanged from today)
```

- **`role`**: `Role::Button` for every row, satisfying spec FR-002 (an
  interactive, selectable control, not static text) — the exact role
  `egui`'s `Button`/`Button::selectable` already reports for the block
  palette's own options (`egui-0.36.1/src/response.rs`'s
  `fill_accesskit_node_from_widget_info`: `WidgetType::Button |
  WidgetType::SelectableLabel => Role::Button`).
- **`toggled`**: present and accurate for every row independently,
  satisfying spec FR-001 — set from `.selected(self.selected.contains(id))`
  on the `Button`, which `egui` propagates through
  `WidgetInfo::selected`/`Toggled::True`/`Toggled::False`
  (`egui-0.36.1/src/response.rs`). A multi-selected layout (spec
  Acceptance Scenario 3) reports every selected row's `toggled` as `True`
  independently — this is a property of iterating the loop once per
  instance and reading `self.selected.contains(id)` fresh for each,
  already true of the current code and unaffected by the widget swap.
- **`label`**: byte-identical to today's row text (spec FR-003) —
  `instance_semantic_label` itself is not modified by this feature; only
  the widget it is wrapped in changes.

## What this feature does NOT change

- `src/domain/layout.rs`, `src/domain/catalog.rs`, `src/domain/geometry.rs`
  — no method, field, or derive changes; this feature never touches
  domain code.
- `src/persistence/*` — no document schema, encode/decode path is
  affected; this is a rendering-only change.
- `instance_semantic_label`'s string format — unchanged byte-for-byte
  (spec FR-003).
- The click/`Shift`/`Ctrl` selection-mode dispatch immediately following
  the row's `ui.add_sized(...)` call — unchanged (spec FR-004); it reads
  the same `Response::clicked_by`/`ui.input(|i| i.modifiers)` API either
  widget type already provides identically.
- `block_palette_ui` and `blueprint_library_section_ui` — untouched
  (research.md Decision 4, spec FR-008).
