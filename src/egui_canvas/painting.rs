use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, StrokeKind, Vec2};
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

/// Maps an orientation angle (degrees, 0 = up, increasing clockwise as
/// drawn on screen — matching `Rotation`'s own documented clockwise
/// direction) to a unit direction vector in screen space. Restored from
/// the pre-icons placeholder-arrow implementation (Phase 8); the corner
/// orientation dot below reuses the identical convention so it orbits
/// exactly the way the removed arrow used to point.
fn orientation_direction(angle_degrees: f32) -> Vec2 {
    let radians = angle_degrees.to_radians();
    vec2(radians.sin(), -radians.cos())
}

/// Paints a small filled dot orbiting `rect`'s center as `angle_degrees`
/// turns, sitting in the corner that is top-right at rest (a 45° offset
/// from the "up" direction `orientation_direction` returns) and stepping
/// exactly one corner clockwise per accepted 90° turn — the orientation
/// indicator for a block whose label must stay upright.
///
/// Fix follow-up to the reverted "custom buildable icons" feature: that
/// feature rotated the fallback TEXT LABEL itself to show orientation,
/// which made plain text spin illegibly (Diogo's bug report). This dot
/// replaces that behavior — `paint_instances` below never rotates text
/// again — while keeping every other part of that feature that worked
/// fine: a buildable WITH a usable custom icon still rotates that icon
/// image exactly as before, unaffected by this change.
///
/// `orbit_radius` is bounded by `rect`'s shorter half-extent (not a
/// larger corner-hugging radius) because `direction` is always a UNIT
/// vector — its length never changes, only its angle does — and during
/// an in-flight rotation transition `angle_degrees` sweeps continuously
/// through every intermediate value, not just the four resting corner
/// angles. Partway through a turn `direction` briefly points purely
/// horizontally or vertically (e.g. exactly between two corners), which
/// would push the dot outside a narrower rect if the radius were sized
/// only for the resting diagonal case.
pub(super) fn paint_orientation_dot(
    painter: &egui::Painter,
    rect: Rect,
    angle_degrees: f32,
    color: Color32,
) {
    let shortest_side = rect.width().min(rect.height());
    let dot_radius = (shortest_side * 0.09).clamp(2.0, 5.0);
    let orbit_radius = (shortest_side * 0.5 - dot_radius - 2.0).max(0.0);
    let direction = orientation_direction(angle_degrees + 45.0);
    let center = rect.center() + direction * orbit_radius;
    painter.circle_filled(center, dot_radius, color);
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

/// Paints `texture` (if present) or `fallback_label` (otherwise) inside
/// `rect`, at `opacity` (0.0-1.0) to preserve the existing "translucent,
/// does not imply acceptance" preview semantic. Shared by both the
/// single-buildable and blueprint-member preview paint sites
/// (research.md Decision 7, item 3 note: "extending, not duplicating,
/// call sites").
///
/// `angle_degrees` (the candidate's rest orientation) still rotates a
/// custom icon image exactly as before — an icon inherently shows its
/// own orientation as a picture, so this is unaffected by Diogo's
/// bug report. `fallback_label` is now ALWAYS painted upright, never
/// rotated: the reverted-and-reapplied fix from `paint_instances`
/// below applies identically here, so a text-only buildable's preview
/// matches its placed-instance appearance instead of contradicting it.
pub(super) fn paint_preview_representation(
    painter: &egui::Painter,
    rect: Rect,
    texture: Option<&egui::TextureHandle>,
    fallback_label: &str,
    angle_degrees: f32,
    opacity: f32,
) {
    match texture {
        Some(texture) => {
            let sized_texture = egui::load::SizedTexture::from_handle(texture);
            let options = egui::ImageOptions {
                tint: Color32::from_white_alpha((opacity * 255.0) as u8),
                rotation: Some((
                    egui::emath::Rot2::from_angle(angle_degrees.to_radians()),
                    Vec2::splat(0.5),
                )),
                ..Default::default()
            };
            egui::paint_texture_at(painter, rect, &options, &sized_texture);
        }
        None => {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                fallback_label,
                FontId::proportional((rect.height() * 0.4).clamp(8.0, 11.0)),
                TEXT_PRIMARY.gamma_multiply(opacity),
            );
            paint_orientation_dot(
                painter,
                rect,
                angle_degrees,
                TEXT_PRIMARY.gamma_multiply(opacity),
            );
        }
    }
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
        match texture {
            Some(texture) => {
                paint_rotated_icon(painter, texture, screen_rect, angle);
            }
            None => {
                painter.text(
                    screen_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional((screen_rect.height() * 0.4).clamp(8.0, 11.0)),
                    TEXT_PRIMARY,
                );
                paint_orientation_dot(painter, screen_rect, angle, TEXT_PRIMARY);
            }
        }
    }

    rotation_visuals.consume_instant_sync();
}
