use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};
use factory_canvas::domain::catalog::{BlockCategory, BlockTemplate};
use factory_canvas::domain::geometry::{GridPoint, GridSize};
use factory_canvas::domain::layout::{BlockInstance, FactoryLayout};

pub(crate) const CANVAS_BACKGROUND: Color32 = Color32::from_rgb(10, 17, 26);
const GRID_BACKGROUND: Color32 = Color32::from_rgb(15, 29, 41);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 237, 242);
const TEXT_MUTED: Color32 = Color32::from_rgb(130, 151, 163);
const ACCENT: Color32 = Color32::from_rgb(91, 221, 199);
const BORDER: Color32 = Color32::from_rgb(35, 53, 67);

pub(crate) fn fitted_grid_rect(available: Rect, bounds: GridSize) -> Rect {
    let tile_size = (available.width() / f32::from(bounds.width()))
        .min(available.height() / f32::from(bounds.height()));
    let size = vec2(
        tile_size * f32::from(bounds.width()),
        tile_size * f32::from(bounds.height()),
    );

    Rect::from_center_size(available.center(), size)
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
    let x = ((position.x - grid_rect.left()) / tile_width).floor() as i32;
    let y = ((position.y - grid_rect.top()) / tile_height).floor() as i32;

    Some(GridPoint::new(x, y))
}

fn block_screen_rect(grid_rect: Rect, bounds: GridSize, instance: BlockInstance) -> Rect {
    let tile_width = grid_rect.width() / f32::from(bounds.width());
    let tile_height = grid_rect.height() / f32::from(bounds.height());
    let origin = instance.origin();
    let footprint = instance
        .rotation()
        .apply_to(instance.template().definition().footprint());
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

fn paint_instances(painter: &egui::Painter, grid_rect: Rect, layout: &FactoryLayout) {
    let bounds = layout.bounds();

    for instance in layout.instances().copied() {
        let screen_rect = block_screen_rect(grid_rect, bounds, instance).shrink(1.0);
        let (fill, stroke, label) = block_visual(instance.template());
        painter.rect_filled(screen_rect, 2, fill);
        painter.rect_stroke(screen_rect, 2, Stroke::new(1.5, stroke), StrokeKind::Inside);
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
    placement_enabled: bool,
) -> Option<GridPoint> {
    let available_size = ui.available_size().max(Vec2::splat(1.0));
    let (response, painter) = ui.allocate_painter(available_size, Sense::click());
    let response = if placement_enabled {
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
    let grid_rect = fitted_grid_rect(grid_available, bounds);

    paint_grid(&painter, grid_rect, bounds);
    paint_instances(&painter, grid_rect, layout);
    painter.rect_stroke(grid_rect, 2, Stroke::new(1.5, ACCENT), StrokeKind::Inside);

    if !placement_enabled || !response.clicked() {
        return None;
    }

    response
        .interact_pointer_pos()
        .and_then(|position| grid_point_at(grid_rect, bounds, position))
}

#[cfg(test)]
mod tests {
    use eframe::egui::{pos2, vec2, Rect};
    use factory_canvas::domain::catalog::BlockTemplate;
    use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
    use factory_canvas::domain::layout::{BlockInstance, EntityId};

    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
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
