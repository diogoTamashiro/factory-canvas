use eframe::egui::{Pos2, Rect, Vec2};

pub(super) const MIN_VIEWPORT_ZOOM: f32 = 0.25;
pub(super) const MAX_VIEWPORT_ZOOM: f32 = 4.0;
const WHEEL_ZOOM_SENSITIVITY: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CanvasViewport {
    pub(super) zoom: f32,
    pub(super) pan: Vec2,
}

impl Default for CanvasViewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

impl CanvasViewport {
    pub(super) fn to_screen(self, point: Pos2, anchor: Pos2) -> Pos2 {
        anchor + self.pan + (point - anchor) * self.zoom
    }

    pub(super) fn to_base(self, point: Pos2, anchor: Pos2) -> Pos2 {
        anchor + ((point - anchor - self.pan) / self.zoom)
    }

    pub(super) fn transform_grid_rect(self, rect: Rect, anchor: Pos2) -> Rect {
        Rect::from_min_max(
            self.to_screen(rect.min, anchor),
            self.to_screen(rect.max, anchor),
        )
    }

    pub(super) fn zoom_by_at(&mut self, factor: f32, cursor: Pos2, anchor: Pos2) {
        let base_point_at_cursor = self.to_base(cursor, anchor);
        self.zoom = (self.zoom * factor).clamp(MIN_VIEWPORT_ZOOM, MAX_VIEWPORT_ZOOM);
        self.pan = (cursor - anchor) - (base_point_at_cursor - anchor) * self.zoom;
    }

    pub(crate) fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub(crate) fn frame_all(&mut self) {
        *self = Self::default();
    }

    pub(super) fn frame_rect(&mut self, target: Rect, available: Rect, anchor: Pos2) -> bool {
        let safe = available.shrink(24.0);
        if target.width() <= f32::EPSILON
            || target.height() <= f32::EPSILON
            || safe.width() <= f32::EPSILON
            || safe.height() <= f32::EPSILON
        {
            return false;
        }

        self.zoom = (safe.width() / target.width())
            .min(safe.height() / target.height())
            .clamp(MIN_VIEWPORT_ZOOM, MAX_VIEWPORT_ZOOM);
        self.pan = (safe.center() - anchor) - (target.center() - anchor) * self.zoom;
        true
    }
}

pub(super) fn zoom_factor_from_wheel_delta(delta: f32) -> f32 {
    (delta * WHEEL_ZOOM_SENSITIVITY).exp()
}

pub(super) fn apply_canvas_viewport_gesture(
    viewport: &mut CanvasViewport,
    anchor: Pos2,
    pan_delta: Vec2,
    wheel_delta: f32,
    cursor: Option<Pos2>,
) -> bool {
    let mut changed = false;

    if pan_delta != Vec2::ZERO {
        viewport.pan_by(pan_delta);
        changed = true;
    }
    if wheel_delta.is_finite() && wheel_delta != 0.0 {
        if let Some(cursor) = cursor {
            viewport.zoom_by_at(zoom_factor_from_wheel_delta(wheel_delta), cursor, anchor);
            changed = true;
        }
    }

    changed
}
