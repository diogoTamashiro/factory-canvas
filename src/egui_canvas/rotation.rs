use eframe::egui;
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{EntityId, FactoryLayout};
use std::collections::HashMap;

/// Duration of a visual rotation transition (research.md Decision 1),
/// matching `egui`'s own default `Style::animation_time` — reusing the
/// framework's own standard duration rather than picking an arbitrary
/// one.
const ROTATION_TRANSITION_SECONDS: f32 = 0.2;

/// Degrees a `Rotation`'s resting orientation represents, matching the
/// same clockwise convention `Rotation::clockwise()` already uses.
pub(super) const fn rotation_degrees(rotation: Rotation) -> f32 {
    match rotation {
        Rotation::Zero => 0.0,
        Rotation::Clockwise90 => 90.0,
        Rotation::Clockwise180 => 180.0,
        Rotation::Clockwise270 => 270.0,
    }
}

/// Per-instance animation bookkeeping for the visual rotation transition
/// (spec.md Key Entities; data-model.md; research.md Decisions 2-5).
///
/// Purely presentational, session-local, never persisted — absent from
/// `FactoryDocument`, `BlueprintDocument`, and any file `BlueprintLibrary`
/// writes, and never touched by `src/history.rs`'s undo/redo (research.md
/// Decision 3's `resync` is the only bridge between the two, called by
/// `egui_app.rs` after undo/redo already restores `self.layout`, not the
/// other way around).
#[derive(Debug, Clone, Copy, PartialEq)]
struct RotationEntry {
    /// The instance's last-known `Rotation`, used to detect a real
    /// rotation (research.md Decision 4): compared against the current
    /// value on every `paint_instances` call.
    last_rotation: Rotation,
    /// The instance's last-known origin, used to detect a
    /// rotation-driven move (research.md Decision 4). Kept as `GridPoint`
    /// (this is a comparison-only field, never itself interpolated).
    last_origin: GridPoint,
    /// Accumulated target angle in degrees, monotonically increasing by
    /// exactly +90.0 per detected rotation (research.md Decision 2) —
    /// never normalized modulo 360 except at the point of painting.
    target_angle: f32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RotationVisuals {
    entries: HashMap<u64, RotationEntry>,
    /// One-shot flag: synchronizes with the layout replaced on the same
    /// frame, so the very next `paint_instances` call snaps every
    /// instance's animation to its resting value instantly instead of
    /// interpolating (research.md Decision 3, FR-008/FR-009).
    instant_sync_pending: bool,
}

impl RotationVisuals {
    /// Rebuilds every per-entity record from `layout`'s actual current
    /// state and arms the instant-snap flag. Called by `egui_app.rs`
    /// immediately after `new_document_at`, `open_document_from`,
    /// `replace_base`, and undo/redo's shared `apply_restored_snapshot`
    /// each replace `self.layout` wholesale (research.md Decision 3) —
    /// every one of those is a layout replacement this feature must
    /// never animate, regardless of what rotation values happen to
    /// differ from the previous layout.
    pub(crate) fn resync(&mut self, layout: &FactoryLayout) {
        self.entries.clear();
        for instance in layout.instances() {
            self.entries.insert(
                instance.id().value(),
                RotationEntry {
                    last_rotation: instance.rotation(),
                    last_origin: instance.origin(),
                    target_angle: rotation_degrees(instance.rotation()),
                },
            );
        }
        self.instant_sync_pending = true;
    }

    /// Reads `ctx`'s animation manager for `id`, targeting `target` over
    /// `duration` seconds. When `duration` is `0.0` this performs one
    /// extra throwaway read first: `egui`'s own vendored
    /// `AnimationManager::animate_value` always computes its return
    /// value from the PRE-update state, so the exact call that first
    /// reports a new target — regardless of that call's own duration —
    /// returns the OLD value, not the new one; only the next read
    /// reports the settled value. A caller wanting a true same-frame
    /// instant snap (duration `0.0`) must therefore prime the entry with
    /// a throwaway read before its real one. When the target has not
    /// changed, the discard read is a harmless redundant lookup.
    fn animate_instant_or_transition(
        ctx: &egui::Context,
        id: egui::Id,
        target: f32,
        duration: f32,
    ) -> f32 {
        if duration == 0.0 {
            ctx.animate_value_with_time(id, target, 0.0);
        }
        ctx.animate_value_with_time(id, target, duration)
    }

    /// Computes this frame's displayed angle (degrees) and FRACTIONAL
    /// grid-space origin (x, y — deliberately not rounded to a
    /// `GridPoint`, so a caller mapping these to screen space renders a
    /// smooth slide mid-transition rather than jumping tile-to-tile) for
    /// `id`, given its CURRENT domain-stored `rotation`/`origin`.
    /// Updates the per-entity record when a real `Rotation` change is
    /// detected (research.md Decision 4: this is the only signal that
    /// distinguishes "just rotated" from an ordinary move or the very
    /// first frame an instance is seen), then reads the interpolated
    /// value from `ctx`'s own animation manager — using a `0.0` duration
    /// (an instant snap, research.md Decision 1) whenever this frame's
    /// one-shot instant-sync flag from `resync()` is still armed. Origin
    /// only animates alongside a detected rotation (spec.md FR-004); an
    /// origin that changes without its own `Rotation` changing (a plain
    /// move) is returned unanimated, exactly as given.
    pub(crate) fn visual_state_for(
        &mut self,
        ctx: &egui::Context,
        id: EntityId,
        current_rotation: Rotation,
        current_origin: GridPoint,
    ) -> (f32, f32, f32) {
        let key = id.value();
        let rotated = match self.entries.get(&key) {
            Some(entry) => entry.last_rotation != current_rotation,
            // First time this instance is seen: not a rotation, just an
            // initial resting appearance (research.md Decision 1's
            // confirmed "first call returns the target immediately").
            None => false,
        };
        // A plain move (research.md Decision 4): the origin changed but
        // this exact frame did NOT also detect a rotation. Deliberately
        // computed BEFORE `entry.last_origin` is overwritten below, and
        // deliberately distinct from `rotated`, which is only ever true
        // on the single frame a rotation is first detected — origin
        // must keep animating on every subsequent frame of that SAME
        // transition too, long after `rotated` has already gone back to
        // `false` for that instance.
        let plain_move = match self.entries.get(&key) {
            Some(entry) => !rotated && entry.last_origin != current_origin,
            None => false,
        };

        let entry = self.entries.entry(key).or_insert(RotationEntry {
            last_rotation: current_rotation,
            last_origin: current_origin,
            target_angle: rotation_degrees(current_rotation),
        });
        if rotated {
            entry.target_angle += 90.0;
        }
        entry.last_rotation = current_rotation;
        entry.last_origin = current_origin;

        let duration = if self.instant_sync_pending {
            0.0
        } else {
            ROTATION_TRANSITION_SECONDS
        };
        let angle_id = egui::Id::new((key, "rotation_angle"));
        let angle =
            Self::animate_instant_or_transition(ctx, angle_id, entry.target_angle, duration);

        // The x/y animation entries default to the SAME duration the
        // angle uses (continuing to pass a consistent non-zero duration
        // on every frame of a transition is required: `egui`'s
        // `AnimationManager::animate_value` recomputes its interpolated
        // fraction from THIS call's duration every single time, so
        // switching back to a `0.0` duration on any later frame of an
        // already-in-flight transition — simply because `rotated` is
        // only ever true on the ONE frame a rotation is first detected —
        // would prematurely and incorrectly snap it there, confirmed by
        // this exact bug surfacing in
        // `group_rotation_animates_position_and_angle_together_for_every_member`).
        // The one deliberate override is a genuine plain move (spec.md
        // FR-004: origin changed this frame with no rotation this frame)
        // — that always forces an instant `0.0`, so an ordinary move
        // never animates.
        let origin_duration = if plain_move { 0.0 } else { duration };
        let x_id = egui::Id::new((key, "rotation_origin_x"));
        let y_id = egui::Id::new((key, "rotation_origin_y"));
        let x = Self::animate_instant_or_transition(
            ctx,
            x_id,
            current_origin.x as f32,
            origin_duration,
        );
        let y = Self::animate_instant_or_transition(
            ctx,
            y_id,
            current_origin.y as f32,
            origin_duration,
        );

        (angle.rem_euclid(360.0), x, y)
    }

    /// Clears the one-shot instant-sync flag `resync()` armed. Called by
    /// `paint_instances` once per frame, AFTER every instance in that
    /// frame has already had a chance to read it via `visual_state_for`
    /// — so a `resync()` still correctly forces the very next frame (and
    /// only that frame) to snap instantly, never any frame after it.
    pub(super) fn consume_instant_sync(&mut self) {
        self.instant_sync_pending = false;
    }
}
