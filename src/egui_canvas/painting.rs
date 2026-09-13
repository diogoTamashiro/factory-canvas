use eframe::egui::{self, pos2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use factory_canvas::domain::catalog::BuildableDefinition;
use factory_canvas::domain::geometry::GridSize;
use factory_canvas::domain::layout::FactoryLayout;

use crate::egui_app::icons::BuildableIcons;
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

/// Whether an instance's orientation is expressed by a rotated icon
/// image or by rotating its fallback text label — the placeholder
/// arrow (Phase 8) is removed entirely per spec.md FR-009; one of
/// these two representations always carries the orientation now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OrientationRepresentation {
    Icon,
    Text,
}

pub(super) fn orientation_representation_for(
    texture: Option<&egui::TextureHandle>,
) -> OrientationRepresentation {
    match texture {
        Some(_) => OrientationRepresentation::Icon,
        None => OrientationRepresentation::Text,
    }
}

/// Paints `label`, centered at rest but rotated by `angle_degrees`
/// clockwise around that same center — the fallback-text half of
/// research.md Decision 6. `painter.text(...)` has no rotation
/// parameter, so this lays out a galley once and paints it as a
/// `Shape::Text` with an explicit `angle`, whose pivot is the galley's
/// own top-left `pos` (not its center), so the center-based `pos` used
/// by the removed `painter.text(...)` call must be converted to that
/// top-left corner first.
fn paint_rotated_label(
    painter: &egui::Painter,
    center: Pos2,
    label: &str,
    font_id: FontId,
    color: Color32,
    angle_degrees: f32,
) {
    let galley = painter.layout_no_wrap(label.to_owned(), font_id, color);
    let pos = center - galley.size() * 0.5;
    let mut text_shape = egui::epaint::TextShape::new(pos, galley, color);
    text_shape.angle = angle_degrees.to_radians();
    painter.add(egui::Shape::Text(text_shape));
}

/// Paints `texture` inside `rect`, rotated by `angle_degrees` clockwise
/// around the rect's center — the icon half of research.md Decision 6.
/// Uses `egui::paint_texture_at` directly (the same free function
/// `Image::paint_at` delegates to internally) since `paint_instances`
/// only has a `&Painter`, not a `&Ui` — `Image::paint_at` itself
/// requires a `Ui` merely to read `pixels_per_point`/`ctx()`, both of
/// which `Painter` already exposes directly. `Image::rotate`'s
/// `origin` is in normalized UV space, so the center is always
/// `Vec2::splat(0.5)` regardless of `rect`'s actual size.
fn paint_rotated_icon(
    painter: &egui::Painter,
    texture: &egui::TextureHandle,
    rect: Rect,
    angle_degrees: f32,
) {
    let sized_texture = egui::load::SizedTexture::from_handle(texture);
    let options = egui::ImageOptions {
        rotation: Some((
            egui::emath::Rot2::from_angle(angle_degrees.to_radians()),
            Vec2::splat(0.5),
        )),
        ..Default::default()
    };
    egui::paint_texture_at(painter, rect, &options, &sized_texture);
}

pub(super) fn paint_instances(
    painter: &egui::Painter,
    grid_rect: Rect,
    layout: &FactoryLayout,
    selected: &SelectedSet,
    rotation_visuals: &mut RotationVisuals,
    icons: &BuildableIcons,
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

        let texture = icons.texture(definition.id());
        match orientation_representation_for(texture) {
            OrientationRepresentation::Icon => {
                paint_rotated_icon(
                    painter,
                    texture.expect("Icon branch only reached when texture is Some"),
                    screen_rect,
                    angle,
                );
            }
            OrientationRepresentation::Text => {
                paint_rotated_label(
                    painter,
                    screen_rect.center(),
                    label,
                    FontId::proportional((screen_rect.height() * 0.4).clamp(8.0, 11.0)),
                    TEXT_PRIMARY,
                    angle,
                );
            }
        }
    }

    rotation_visuals.consume_instant_sync();
}
