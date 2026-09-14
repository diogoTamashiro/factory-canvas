use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2,
};
use factory_canvas::domain::catalog::BuildableDefinition;
use factory_canvas::domain::geometry::GridSize;
use factory_canvas::domain::layout::FactoryLayout;

use crate::selected_set::SelectedSet;

use super::geometry::footprint_screen_rect_fractional;
use super::rotation::RotationVisuals;
use super::{ACCENT, GRID_BACKGROUND, TEXT_PRIMARY};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanvasPaintLayer {
    Grid,
    Preview,
    Instances,
}

pub(super) fn canvas_paint_layers() -> [CanvasPaintLayer; 3] {
    [
        CanvasPaintLayer::Grid,
        CanvasPaintLayer::Preview,
        CanvasPaintLayer::Instances,
    ]
}

pub(super) fn paint_grid(painter: &egui::Painter, grid_rect: Rect, bounds: GridSize) {
    painter.rect_filled(grid_rect, 2, GRID_BACKGROUND);
    let grid_minor = Color32::from_rgba_unmultiplied(88, 120, 135, 44);
    let grid_major = Color32::from_rgba_unmultiplied(91, 221, 199, 94);

    for x in 0..=bounds.width() {
        let fraction = f32::from(x) / f32::from(bounds.width());
        let screen_x = egui::lerp(grid_rect.left()..=grid_rect.right(), fraction);
        let stroke = if x % 10 == 0 {
            Stroke::new(1.0, grid_major)
        } else {
            Stroke::new(0.5, grid_minor)
        };
        painter.line_segment(
            [
                pos2(screen_x, grid_rect.top()),
                pos2(screen_x, grid_rect.bottom()),
            ],
            stroke,
        );
    }

    for y in 0..=bounds.height() {
        let fraction = f32::from(y) / f32::from(bounds.height());
        let screen_y = egui::lerp(grid_rect.top()..=grid_rect.bottom(), fraction);
        let stroke = if y % 10 == 0 {
            Stroke::new(1.0, grid_major)
        } else {
            Stroke::new(0.5, grid_minor)
        };
        painter.line_segment(
            [
                pos2(grid_rect.left(), screen_y),
                pos2(grid_rect.right(), screen_y),
            ],
            stroke,
        );
    }
}

/// Maps an orientation angle (degrees, 0 = up, increasing clockwise as
/// drawn on screen — matching `Rotation`'s own documented clockwise
/// direction) to a unit direction vector in screen space.
fn orientation_arrow_direction(angle_degrees: f32) -> Vec2 {
    let radians = angle_degrees.to_radians();
    vec2(radians.sin(), -radians.cos())
}

/// Paints a small triangular arrow at `center`, pointing in
/// `angle_degrees`'s direction (spec.md FR-001/FR-002's orientation
/// indicator) — a deliberately simple placeholder shape, documented in
/// research.md/quickstart.md as provisional until real per-block
/// icons/sprites exist.
fn paint_orientation_arrow(painter: &egui::Painter, center: Pos2, radius: f32, angle_degrees: f32) {
    let forward = orientation_arrow_direction(angle_degrees);
    let right = vec2(forward.y, -forward.x);
    let tip = center + forward * radius;
    let base_left = center - forward * radius * 0.5 + right * radius * 0.5;
    let base_right = center - forward * radius * 0.5 - right * radius * 0.5;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, base_left, base_right],
        TEXT_PRIMARY,
        Stroke::NONE,
    ));
}

pub(super) fn block_visual(definition: &BuildableDefinition) -> (Color32, Color32, &str) {
    let (fill, stroke) = match definition.category_id().as_str() {
        "energy" => (
            Color32::from_rgb(105, 73, 32),
            Color32::from_rgb(239, 180, 81),
        ),
        "production_i" => (
            Color32::from_rgb(24, 82, 103),
            Color32::from_rgb(83, 191, 223),
        ),
        _ => (
            Color32::from_rgb(65, 72, 82),
            Color32::from_rgb(164, 174, 188),
        ),
    };

    (fill, stroke, definition.symbol())
}

pub(super) fn placement_preview_visual(definition: &BuildableDefinition) -> (Color32, Color32) {
    let (fill, stroke, _) = block_visual(definition);
    let preview_fill = Color32::from_rgba_unmultiplied(fill.r(), fill.g(), fill.b(), 112);

    (preview_fill, stroke)
}

pub(super) fn paint_instances(
    painter: &egui::Painter,
    grid_rect: Rect,
    layout: &FactoryLayout,
    selected: &SelectedSet,
    rotation_visuals: &mut RotationVisuals,
) {
    let bounds = layout.bounds();
    let ctx = painter.ctx();

    for instance in layout.instances() {
        let resolved = layout
            .resolved_instance(instance.id())
            .expect("stored instance must resolve through the layout catalog");
        let definition = resolved.definition();
        let (angle, origin_x, origin_y) = rotation_visuals.visual_state_for(
            ctx,
            instance.id(),
            instance.rotation(),
            instance.origin(),
        );
        let screen_rect = footprint_screen_rect_fractional(
            grid_rect,
            bounds,
            origin_x,
            origin_y,
            resolved.effective_footprint(),
        )
        .shrink(1.0);
        let (fill, stroke, label) = block_visual(definition);
        painter.rect_filled(screen_rect, 2, fill);
        painter.rect_stroke(screen_rect, 2, Stroke::new(1.5, stroke), StrokeKind::Inside);
        if selected.contains(instance.id()) {
            painter.rect_stroke(
                screen_rect.expand(2.0),
                3,
                Stroke::new(2.5, ACCENT),
                StrokeKind::Outside,
            );
        }
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional((screen_rect.height() * 0.4).clamp(8.0, 11.0)),
            TEXT_PRIMARY,
        );
        let arrow_radius = screen_rect.width().min(screen_rect.height()) * 0.18;
        paint_orientation_arrow(painter, screen_rect.center(), arrow_radius, angle);
    }

    rotation_visuals.consume_instant_sync();
}
