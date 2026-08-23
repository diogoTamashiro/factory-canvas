use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, PointerButton, Pos2, Rect, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};
use factory_canvas::domain::catalog::{BlockCategory, BlockTemplate};
use factory_canvas::domain::geometry::{GridPoint, GridSize};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};

pub(crate) const CANVAS_BACKGROUND: Color32 = Color32::from_rgb(10, 17, 26);
const GRID_BACKGROUND: Color32 = Color32::from_rgb(15, 29, 41);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 237, 242);
const TEXT_MUTED: Color32 = Color32::from_rgb(130, 151, 163);
const ACCENT: Color32 = Color32::from_rgb(91, 221, 199);
const BORDER: Color32 = Color32::from_rgb(35, 53, 67);
const MIN_VIEWPORT_ZOOM: f32 = 0.25;
const MAX_VIEWPORT_ZOOM: f32 = 4.0;
const WHEEL_ZOOM_SENSITIVITY: f32 = 0.01;

pub(crate) fn fitted_grid_rect(available: Rect, bounds: GridSize) -> Rect {
    let tile_size = (available.width() / f32::from(bounds.width()))
        .min(available.height() / f32::from(bounds.height()));
    let size = vec2(
        tile_size * f32::from(bounds.width()),
        tile_size * f32::from(bounds.height()),
    );

    Rect::from_center_size(available.center(), size)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CanvasViewport {
    zoom: f32,
    pan: Vec2,
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
    fn to_screen(self, point: Pos2, anchor: Pos2) -> Pos2 {
        anchor + self.pan + (point - anchor) * self.zoom
    }

    fn to_base(self, point: Pos2, anchor: Pos2) -> Pos2 {
        anchor + ((point - anchor - self.pan) / self.zoom)
    }

    fn transform_grid_rect(self, rect: Rect, anchor: Pos2) -> Rect {
        Rect::from_min_max(
            self.to_screen(rect.min, anchor),
            self.to_screen(rect.max, anchor),
        )
    }

    fn zoom_by_at(&mut self, factor: f32, cursor: Pos2, anchor: Pos2) {
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
}

fn zoom_factor_from_wheel_delta(delta: f32) -> f32 {
    (delta * WHEEL_ZOOM_SENSITIVITY).exp()
}

fn apply_canvas_viewport_gesture(
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

fn grid_point_at(grid_rect: Rect, bounds: GridSize, position: Pos2) -> Option<GridPoint> {
    if position.x < grid_rect.left()
        || position.x >= grid_rect.right()
        || position.y < grid_rect.top()
        || position.y >= grid_rect.bottom()
    {
        return None;
    }

    let tile_width = grid_rect.width() / f32::from(bounds.width());
    let tile_height = grid_rect.height() / f32::from(bounds.height());
    let max_x = i32::from(bounds.width()) - 1;
    let max_y = i32::from(bounds.height()) - 1;
    let x = ((position.x - grid_rect.left()) / tile_width)
        .floor()
        .clamp(0.0, max_x as f32) as i32;
    let y = ((position.y - grid_rect.top()) / tile_height)
        .floor()
        .clamp(0.0, max_y as f32) as i32;

    Some(GridPoint::new(x, y))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlacementPreview {
    template: BlockTemplate,
    origin: GridPoint,
}

fn placement_preview_at(
    grid_rect: Rect,
    bounds: GridSize,
    template: BlockTemplate,
    pointer_position: Pos2,
) -> Option<PlacementPreview> {
    grid_point_at(grid_rect, bounds, pointer_position)
        .map(|origin| PlacementPreview { template, origin })
}

fn placement_preview_for_hover(
    grid_rect: Rect,
    bounds: GridSize,
    selected_block: Option<BlockTemplate>,
    hover_position: Option<Pos2>,
) -> Option<PlacementPreview> {
    selected_block
        .zip(hover_position)
        .and_then(|(template, position)| {
            placement_preview_at(grid_rect, bounds, template, position)
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanvasInteraction {
    Select(EntityId),
    Place(GridPoint),
    Deselect,
}

pub(crate) fn resolve_grid_interaction(
    layout: &FactoryLayout,
    point: GridPoint,
    selected_block: Option<BlockTemplate>,
) -> CanvasInteraction {
    if let Some(instance) = layout.instance_at(point) {
        CanvasInteraction::Select(instance.id())
    } else if selected_block.is_some() {
        CanvasInteraction::Place(point)
    } else {
        CanvasInteraction::Deselect
    }
}

fn footprint_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    origin: GridPoint,
    footprint: GridSize,
) -> Rect {
    let tile_width = grid_rect.width() / f32::from(bounds.width());
    let tile_height = grid_rect.height() / f32::from(bounds.height());
    let min = pos2(
        grid_rect.left() + origin.x as f32 * tile_width,
        grid_rect.top() + origin.y as f32 * tile_height,
    );
    let max = pos2(
        min.x + f32::from(footprint.width()) * tile_width,
        min.y + f32::from(footprint.height()) * tile_height,
    );

    Rect::from_min_max(min, max)
}

fn block_screen_rect(grid_rect: Rect, bounds: GridSize, instance: BlockInstance) -> Rect {
    let footprint = instance
        .rotation()
        .apply_to(instance.template().definition().footprint());

    footprint_screen_rect(grid_rect, bounds, instance.origin(), footprint)
}

fn placement_preview_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    preview: PlacementPreview,
) -> Rect {
    footprint_screen_rect(
        grid_rect,
        bounds,
        preview.origin,
        preview.template.definition().footprint(),
    )
}

fn block_visual(template: BlockTemplate) -> (Color32, Color32, &'static str) {
    let (fill, stroke) = match template.definition().category() {
        BlockCategory::Energy => (
            Color32::from_rgb(105, 73, 32),
            Color32::from_rgb(239, 180, 81),
        ),
        BlockCategory::ProductionI => (
            Color32::from_rgb(24, 82, 103),
            Color32::from_rgb(83, 191, 223),
        ),
    };
    let label = match template {
        BlockTemplate::XiranitePowerPole => "PX",
        BlockTemplate::RefineryUnit => "UR",
        BlockTemplate::CrushingUnit => "UT",
    };

    (fill, stroke, label)
}

fn placement_preview_visual(template: BlockTemplate) -> (Color32, Color32) {
    let (fill, stroke, _) = block_visual(template);
    let preview_fill = Color32::from_rgba_unmultiplied(fill.r(), fill.g(), fill.b(), 112);

    (preview_fill, stroke)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanvasPaintLayer {
    Grid,
    Preview,
    Instances,
}

fn canvas_paint_layers() -> [CanvasPaintLayer; 3] {
    [
        CanvasPaintLayer::Grid,
        CanvasPaintLayer::Preview,
        CanvasPaintLayer::Instances,
    ]
}

fn paint_grid(painter: &egui::Painter, grid_rect: Rect, bounds: GridSize) {
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

fn paint_instances(
    painter: &egui::Painter,
    grid_rect: Rect,
    layout: &FactoryLayout,
    selected_instance_id: Option<EntityId>,
) {
    let bounds = layout.bounds();

    for instance in layout.instances().copied() {
        let screen_rect = block_screen_rect(grid_rect, bounds, instance).shrink(1.0);
        let (fill, stroke, label) = block_visual(instance.template());
        painter.rect_filled(screen_rect, 2, fill);
        painter.rect_stroke(screen_rect, 2, Stroke::new(1.5, stroke), StrokeKind::Inside);
        if selected_instance_id == Some(instance.id()) {
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
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    layout: &FactoryLayout,
    title: &str,
    selected_instance_id: Option<EntityId>,
    selected_block: Option<BlockTemplate>,
    viewport: &mut CanvasViewport,
) -> Option<CanvasInteraction> {
    let available_size = ui.available_size().max(Vec2::splat(1.0));
    let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
    let mut response = if selected_block.is_some() {
        response.on_hover_cursor(CursorIcon::Crosshair)
    } else {
        response
    };
    let outer_rect = response.rect;

    painter.rect_filled(outer_rect, 12, CANVAS_BACKGROUND);
    painter.rect_stroke(outer_rect, 12, Stroke::new(1.0, BORDER), StrokeKind::Inside);

    let bounds = layout.bounds();
    let title_position = pos2(outer_rect.left() + 24.0, outer_rect.top() + 22.0);
    painter.text(
        title_position,
        Align2::LEFT_TOP,
        title,
        FontId::proportional(18.0),
        TEXT_PRIMARY,
    );
    painter.text(
        pos2(title_position.x, title_position.y + 25.0),
        Align2::LEFT_TOP,
        format!("{} × {} tiles", bounds.width(), bounds.height()),
        FontId::proportional(11.0),
        TEXT_MUTED,
    );

    let mut grid_available = outer_rect.shrink2(vec2(36.0, 32.0));
    grid_available.min.y += 54.0;
    let viewport_anchor = grid_available.center();

    let pan_delta = if response.dragged_by(PointerButton::Middle) {
        response.drag_delta()
    } else {
        Vec2::ZERO
    };
    let wheel_delta = if response.contains_pointer() {
        ui.input(|input| input.smooth_scroll_delta.y)
    } else {
        0.0
    };
    let cursor = response.hover_pos();
    let viewport_changed =
        apply_canvas_viewport_gesture(viewport, viewport_anchor, pan_delta, wheel_delta, cursor);
    if wheel_delta.is_finite() && wheel_delta != 0.0 && cursor.is_some() {
        ui.input_mut(|input| input.smooth_scroll_delta.y = 0.0);
    }
    if viewport_changed {
        response.mark_changed();
    }
    let grid_rect =
        viewport.transform_grid_rect(fitted_grid_rect(grid_available, bounds), viewport_anchor);
    let preview =
        placement_preview_for_hover(grid_rect, bounds, selected_block, response.hover_pos());

    for layer in canvas_paint_layers() {
        match layer {
            CanvasPaintLayer::Grid => paint_grid(&painter, grid_rect, bounds),
            CanvasPaintLayer::Preview => {
                if let Some(preview) = preview {
                    let screen_rect =
                        placement_preview_screen_rect(grid_rect, bounds, preview).shrink(1.0);
                    let (fill, stroke) = placement_preview_visual(preview.template);
                    painter.rect_filled(screen_rect, 2, fill);
                    painter.rect_stroke(
                        screen_rect,
                        2,
                        Stroke::new(1.5, stroke),
                        StrokeKind::Inside,
                    );
                }
            }
            CanvasPaintLayer::Instances => {
                paint_instances(&painter, grid_rect, layout, selected_instance_id)
            }
        }
    }
    painter.rect_stroke(grid_rect, 2, Stroke::new(1.5, ACCENT), StrokeKind::Inside);

    if !response.clicked() {
        return None;
    }

    response
        .interact_pointer_pos()
        .and_then(|position| grid_point_at(grid_rect, bounds, position))
        .map(|point| resolve_grid_interaction(layout, point, selected_block))
}

#[cfg(test)]
mod tests {
    use eframe::egui::{pos2, vec2, Rect};
    use factory_canvas::domain::base::BaseTemplate;
    use factory_canvas::domain::catalog::BlockTemplate;
    use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
    use factory_canvas::domain::layout::{BlockInstance, EntityId};

    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn neutral_canvas_viewport_preserves_base_points_and_rect() {
        let base_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let anchor = base_rect.center();
        let point = pos2(325.0, 475.0);
        let viewport = CanvasViewport::default();

        assert_eq!(viewport.to_screen(point, anchor), point);
        assert_eq!(viewport.to_base(point, anchor), point);
        assert_eq!(viewport.transform_grid_rect(base_rect, anchor), base_rect);
    }

    #[test]
    fn zooming_at_cursor_preserves_the_base_point_under_it() {
        let base_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let anchor = base_rect.center();
        let cursor = pos2(675.0, 325.0);
        let mut viewport = CanvasViewport::default();
        let point_before_zoom = viewport.to_base(cursor, anchor);

        viewport.zoom_by_at(1.5, cursor, anchor);

        let point_after_zoom = viewport.to_base(cursor, anchor);
        assert_close(point_after_zoom.x, point_before_zoom.x);
        assert_close(point_after_zoom.y, point_before_zoom.y);
    }

    #[test]
    fn viewport_clamps_zoom_to_supported_range() {
        let anchor = pos2(500.0, 400.0);
        let cursor = pos2(675.0, 325.0);
        let mut viewport = CanvasViewport::default();

        viewport.zoom_by_at(100.0, cursor, anchor);
        assert_close(viewport.zoom, 4.0);

        viewport.zoom_by_at(0.001, cursor, anchor);
        assert_close(viewport.zoom, 0.25);
    }

    #[test]
    fn panning_moves_screen_space_and_frame_all_restores_neutral_viewport() {
        let anchor = pos2(500.0, 400.0);
        let point = pos2(325.0, 475.0);
        let mut viewport = CanvasViewport::default();

        viewport.pan_by(vec2(120.0, -80.0));
        assert_eq!(viewport.to_screen(point, anchor), pos2(445.0, 395.0));

        viewport.frame_all();
        assert_eq!(viewport, CanvasViewport::default());
    }

    #[test]
    fn wheel_delta_maps_to_reversible_zoom_factor() {
        assert_close(zoom_factor_from_wheel_delta(0.0), 1.0);
        assert!(zoom_factor_from_wheel_delta(120.0) > 1.0);
        assert!(zoom_factor_from_wheel_delta(-120.0) < 1.0);
        assert_close(
            zoom_factor_from_wheel_delta(120.0) * zoom_factor_from_wheel_delta(-120.0),
            1.0,
        );
    }

    #[test]
    fn viewport_gesture_requires_cursor_for_wheel_zoom_and_ignores_empty_input() {
        let anchor = pos2(500.0, 400.0);
        let cursor = pos2(675.0, 325.0);
        let mut viewport = CanvasViewport::default();

        assert!(!apply_canvas_viewport_gesture(
            &mut viewport,
            anchor,
            Vec2::ZERO,
            0.0,
            None,
        ));
        assert_eq!(viewport, CanvasViewport::default());

        assert!(!apply_canvas_viewport_gesture(
            &mut viewport,
            anchor,
            Vec2::ZERO,
            120.0,
            None,
        ));
        assert_eq!(viewport, CanvasViewport::default());

        assert!(apply_canvas_viewport_gesture(
            &mut viewport,
            anchor,
            Vec2::ZERO,
            120.0,
            Some(cursor),
        ));
        assert!(viewport.zoom > 1.0);
    }

    #[test]
    fn transformed_grid_rect_keeps_hit_testing_in_world_coordinates() {
        let base_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let anchor = base_rect.center();
        let viewport = CanvasViewport {
            zoom: 2.0,
            pan: vec2(40.0, -20.0),
        };
        let grid_rect = viewport.transform_grid_rect(base_rect, anchor);

        assert_eq!(
            grid_point_at(
                grid_rect,
                GridSize::new(80, 40).unwrap(),
                pos2(-210.0, 70.0)
            ),
            Some(GridPoint::new(2, 4))
        );
    }

    #[test]
    fn fitted_grid_rect_stays_centered_and_preserves_aspect_ratio() {
        let cases = [
            (
                Rect::from_min_size(pos2(10.0, 20.0), vec2(500.0, 1_000.0)),
                GridSize::new(80, 40).unwrap(),
            ),
            (
                Rect::from_min_size(pos2(40.0, 10.0), vec2(1_000.0, 500.0)),
                GridSize::new(40, 80).unwrap(),
            ),
        ];

        for (available, bounds) in cases {
            let fitted = fitted_grid_rect(available, bounds);

            assert_close(fitted.center().x, available.center().x);
            assert_close(fitted.center().y, available.center().y);
            assert!(fitted.left() >= available.left());
            assert!(fitted.right() <= available.right());
            assert!(fitted.top() >= available.top());
            assert!(fitted.bottom() <= available.bottom());
            assert_close(
                fitted.width() / fitted.height(),
                f32::from(bounds.width()) / f32::from(bounds.height()),
            );
        }
    }

    #[test]
    fn grid_point_at_maps_inside_points_and_excludes_outer_edges() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();

        assert_eq!(
            grid_point_at(grid_rect, bounds, pos2(100.0, 200.0)),
            Some(GridPoint::new(0, 0))
        );
        assert_eq!(
            grid_point_at(grid_rect, bounds, pos2(110.0, 210.0)),
            Some(GridPoint::new(1, 1))
        );
        assert_eq!(
            grid_point_at(grid_rect, bounds, pos2(899.9, 599.9)),
            Some(GridPoint::new(79, 39))
        );

        for outside in [
            pos2(99.9, 200.0),
            pos2(100.0, 199.9),
            pos2(900.0, 200.0),
            pos2(100.0, 600.0),
        ] {
            assert_eq!(grid_point_at(grid_rect, bounds, outside), None);
        }
    }

    #[test]
    fn grid_point_at_keeps_points_immediately_inside_right_and_bottom_edges_in_final_tile() {
        let available = Rect::from_min_size(pos2(5.0, 10.0), vec2(321.0, 500.0));
        let bounds = GridSize::new(80, 80).unwrap();
        let grid_rect = fitted_grid_rect(available, bounds);
        assert_close(grid_rect.width(), 321.0);
        assert_close(grid_rect.height(), 321.0);
        let immediately_before_right = f32::from_bits(grid_rect.right().to_bits() - 1);
        let immediately_before_bottom = f32::from_bits(grid_rect.bottom().to_bits() - 1);

        assert_eq!(
            grid_point_at(
                grid_rect,
                bounds,
                pos2(immediately_before_right, immediately_before_bottom)
            ),
            Some(GridPoint::new(79, 79))
        );
    }

    #[test]
    fn grid_interaction_selects_occupied_tiles_before_placement() {
        let id = EntityId::new(7);
        let instance = BlockInstance::new(
            id,
            BlockTemplate::XiranitePowerPole,
            GridPoint::new(0, 0),
            Rotation::Zero,
        );
        let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
        assert_eq!(layout.place(instance), Ok(()));

        assert_eq!(
            resolve_grid_interaction(
                &layout,
                GridPoint::new(1, 1),
                Some(BlockTemplate::RefineryUnit),
            ),
            CanvasInteraction::Select(id)
        );
        assert_eq!(
            resolve_grid_interaction(
                &layout,
                GridPoint::new(2, 0),
                Some(BlockTemplate::RefineryUnit),
            ),
            CanvasInteraction::Place(GridPoint::new(2, 0))
        );
        assert_eq!(
            resolve_grid_interaction(&layout, GridPoint::new(2, 0), None),
            CanvasInteraction::Deselect
        );
    }

    #[test]
    fn placement_preview_derives_active_template_and_hovered_tile_as_candidate_origin() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();

        assert_eq!(
            placement_preview_at(
                grid_rect,
                bounds,
                BlockTemplate::RefineryUnit,
                pos2(128.0, 243.0),
            ),
            Some(PlacementPreview {
                template: BlockTemplate::RefineryUnit,
                origin: GridPoint::new(2, 4),
            })
        );
    }

    #[test]
    fn placement_preview_for_hover_requires_an_active_template_and_a_grid_tile() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();

        assert_eq!(
            placement_preview_for_hover(grid_rect, bounds, None, Some(pos2(128.0, 243.0))),
            None
        );
        assert_eq!(
            placement_preview_for_hover(
                grid_rect,
                bounds,
                Some(BlockTemplate::RefineryUnit),
                Some(pos2(900.0, 600.0)),
            ),
            None
        );
        assert_eq!(
            placement_preview_for_hover(
                grid_rect,
                bounds,
                Some(BlockTemplate::RefineryUnit),
                Some(pos2(128.0, 243.0)),
            ),
            Some(PlacementPreview {
                template: BlockTemplate::RefineryUnit,
                origin: GridPoint::new(2, 4),
            })
        );
    }

    #[test]
    fn placement_preview_screen_rect_uses_candidate_origin_and_template_footprint() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();
        let preview = PlacementPreview {
            template: BlockTemplate::RefineryUnit,
            origin: GridPoint::new(2, 4),
        };

        let screen_rect = placement_preview_screen_rect(grid_rect, bounds, preview);

        assert_close(screen_rect.left(), 120.0);
        assert_close(screen_rect.top(), 240.0);
        assert_close(screen_rect.right(), 150.0);
        assert_close(screen_rect.bottom(), 270.0);
    }

    #[test]
    fn placement_preview_visual_keeps_block_colors_with_translucent_fill() {
        let (block_fill, block_stroke, _) = block_visual(BlockTemplate::RefineryUnit);

        assert_eq!(
            placement_preview_visual(BlockTemplate::RefineryUnit),
            (
                Color32::from_rgba_unmultiplied(
                    block_fill.r(),
                    block_fill.g(),
                    block_fill.b(),
                    112
                ),
                block_stroke,
            )
        );
    }

    #[test]
    fn canvas_paint_layers_keep_persisted_instances_above_preview() {
        assert_eq!(
            canvas_paint_layers(),
            [
                CanvasPaintLayer::Grid,
                CanvasPaintLayer::Preview,
                CanvasPaintLayer::Instances,
            ]
        );
    }

    #[test]
    fn block_screen_rect_uses_instance_origin_and_footprint() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();
        let instance = BlockInstance::new(
            EntityId::new(7),
            BlockTemplate::RefineryUnit,
            GridPoint::new(2, 4),
            Rotation::Zero,
        );

        let screen_rect = block_screen_rect(grid_rect, bounds, instance);

        assert_close(screen_rect.left(), 120.0);
        assert_close(screen_rect.top(), 240.0);
        assert_close(screen_rect.right(), 150.0);
        assert_close(screen_rect.bottom(), 270.0);
    }
}
