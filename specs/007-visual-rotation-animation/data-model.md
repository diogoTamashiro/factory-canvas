# Data Model: Visual Rotation Animation

This feature introduces no domain entity, no catalog concept, and no
persisted field — `FactoryLayout`, `BlockInstance`, `Rotation`,
`FactoryDocument`, and `BlueprintDocument` are all completely unchanged
(spec FR-010; plan.md Constitution Check, Principle V). What follows is
the one new UI-local presentation type this feature adds, and how it
reads already-existing domain values without altering them.

## `RotationVisuals` (new, `egui_canvas.rs`, `pub(crate)`)

A purely presentational, session-local, non-persisted store of
per-instance animation bookkeeping — analogous in spirit to how
`data-model.md`'s existing "no viewport, camera, or other editor-only
metadata is persisted" rule already treats `CanvasViewport`, extended
here to a second kind of ephemeral canvas-only state.

**Fields** (conceptual — exact field names and types are an
implementation detail for `tasks.md`):

| Field | Meaning |
|---|---|
| per-entity record, keyed by `EntityId::value()` (a `u64`) | `EntityId` itself does not derive `Hash` and this feature's scope excludes any `src/domain/` change (FR-010), so the map key is the entity's existing public `u64` accessor rather than the `EntityId` wrapper — semantically identical, just as unique. For each currently- or recently-rendered instance: its last-known `Rotation` (to detect a real rotation per research.md Decision 4), its last-known `GridPoint` origin (to detect and animate a rotation-driven move), and its current accumulated target angle in degrees (research.md Decision 2, monotonically increasing, reduced modulo 360 only at paint time) |
| one-shot instant-snap flag | Set by `resync()` (research.md Decision 3); consumed and cleared by the very next `show()` call, forcing that frame's `animate_value_with_time` calls to use a `0.0` duration instead of the normal transition window |

**Lifecycle**:

- **Created**: as a new field on `FactoryCanvasApp` (analogous to the
  existing `canvas: CanvasState` field), initialized empty.
- **Read and updated**: every `egui_canvas::show()` call, once per
  currently-placed instance — this is the only place that computes an
  animated angle/origin for painting.
- **Explicitly resynced**: by `egui_app.rs`, immediately after
  `new_document_at`, `open_document_from`, `replace_base`, and
  undo/redo's shared `apply_restored_snapshot` each replace
  `self.layout` (research.md Decision 3). Resyncing rebuilds every
  per-entity record from the new layout's actual current state and
  arms the instant-snap flag.
- **Pruned**: entries for an `EntityId` no longer present in the current
  layout are dropped opportunistically during the same per-instance pass
  `show()` already performs (research.md Decision 5) — no separate
  cleanup mechanism.
- **Never persisted**: absent from `FactoryDocument`, `BlueprintDocument`,
  and any file `BlueprintLibrary` writes. A closed-and-reopened
  application, or a freshly opened factory, always starts with this
  store empty — every instance renders at its resting orientation with
  no animation on first appearance (research.md Decision 1's confirmed
  "first call returns the target immediately" behavior).
- **Never affects undo/redo**: `src/history.rs`'s `EditorSnapshot` and
  `EditHistory` are completely unaware this type exists; the reverse is
  also true (`RotationVisuals` never mutates `self.history`) — the only
  connection between the two is that undo/redo's existing
  `apply_restored_snapshot` gains one additional call to
  `RotationVisuals::resync` after it already replaces `self.layout`.

## Existing values this feature reads (unchanged)

| Value | Source | Used for |
|---|---|---|
| `BlockInstance::rotation()` | `src/domain/layout.rs` (existing, public) | Detecting a per-entity rotation (research.md Decision 4) and the resting angle to paint when no transition is active |
| `BlockInstance::origin()` (via `ResolvedInstance`/`layout.instances()`) | `src/domain/layout.rs` (existing, public) | Detecting and animating a rotation-driven move (research.md Decision 4) |
| `BlockInstance::id()` | `src/domain/layout.rs` (existing, public) | The key `RotationVisuals`'s per-entity records and `egui::Id`s are built from |
| `layout.instances()` | `src/domain/layout.rs` (existing, public) | Enumerating every instance to render and resync against — `paint_instances` already iterates this today |

No new getter, setter, or method is added to any `domain/` type.

## Orientation indicator (new visual element, no new data type)

A small arrow drawn by a new painting helper alongside
`paint_instances`'s existing per-instance drawing (fill, stroke, symbol
text). It has no stored state of its own beyond the angle
`RotationVisuals` already computes for that instance each frame — it is
pure paint output, exactly like the existing symbol text and selection
outline are, and requires no new field on any struct.
