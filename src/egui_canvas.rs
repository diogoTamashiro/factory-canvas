use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, PointerButton, Pos2, Rect, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};
use factory_canvas::domain::catalog::{BuildableDefinition, BuildableId};
use factory_canvas::domain::geometry::{GridPoint, GridSize};
use factory_canvas::domain::layout::{EntityId, FactoryLayout, ResolvedInstance};

use crate::selected_set::{SelectedSet, SelectionMode};

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

    fn frame_rect(&mut self, target: Rect, available: Rect, anchor: Pos2) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlacementPreview {
    buildable_id: BuildableId,
    origin: GridPoint,
}

fn placement_preview_at(
    grid_rect: Rect,
    bounds: GridSize,
    buildable_id: &BuildableId,
    pointer_position: Pos2,
) -> Option<PlacementPreview> {
    grid_point_at(grid_rect, bounds, pointer_position).map(|origin| PlacementPreview {
        buildable_id: buildable_id.clone(),
        origin,
    })
}

fn placement_preview_for_hover(
    grid_rect: Rect,
    bounds: GridSize,
    selected_block: Option<&BuildableId>,
    hover_position: Option<Pos2>,
) -> Option<PlacementPreview> {
    selected_block
        .zip(hover_position)
        .and_then(|(buildable_id, position)| {
            placement_preview_at(grid_rect, bounds, buildable_id, position)
        })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GridSelectionRect {
    min: Pos2,
    max: Pos2,
}

impl GridSelectionRect {
    fn from_points(a: Pos2, b: Pos2) -> Self {
        Self {
            min: pos2(a.x.min(b.x), a.y.min(b.y)),
            max: pos2(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    fn contains_origin(self, origin: GridPoint) -> bool {
        let x = origin.x as f32;
        let y = origin.y as f32;
        self.min.x <= x && x <= self.max.x && self.min.y <= y && y <= self.max.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MarqueeDrag {
    start: Pos2,
    mode: SelectionMode,
}

#[derive(Debug, Default)]
struct CanvasInteractionState {
    marquee: Option<MarqueeDrag>,
}

#[derive(Debug, Default)]
pub(crate) struct CanvasState {
    pub(crate) viewport: CanvasViewport,
    interaction: CanvasInteractionState,
    pub(crate) focus_selection_requested: bool,
}

impl CanvasState {
    pub(crate) fn clear_transient_interaction(&mut self) {
        self.interaction = CanvasInteractionState::default();
        self.focus_selection_requested = false;
    }
}

#[derive(Debug, Clone, Copy)]
struct MarqueeFrameInput {
    drag_started: bool,
    dragging: bool,
    drag_stopped: bool,
    press_origin: Option<Pos2>,
    pointer_position: Option<Pos2>,
    mode: SelectionMode,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct MarqueeFrameResult {
    screen_rect: Option<Rect>,
    interaction: Option<CanvasInteraction>,
}

fn update_marquee_frame(
    state: &mut CanvasInteractionState,
    layout: &FactoryLayout,
    grid_rect: Rect,
    bounds: GridSize,
    selected_block: Option<&BuildableId>,
    input: MarqueeFrameInput,
) -> MarqueeFrameResult {
    if input.drag_started {
        state.marquee = input.press_origin.and_then(|origin| {
            marquee_start_at(
                layout,
                grid_rect,
                bounds,
                selected_block,
                origin,
                input.mode,
            )
        });
    }

    if input.drag_stopped {
        let interaction = state.marquee.take().and_then(|drag| {
            let pointer = input.pointer_position?;
            let end = grid_space_at_clamped(grid_rect, bounds, pointer);
            let rect = GridSelectionRect::from_points(drag.start, end);
            Some(CanvasInteraction::Marquee {
                ids: marquee_ids(layout, rect),
                mode: drag.mode,
            })
        });
        return MarqueeFrameResult {
            screen_rect: None,
            interaction,
        };
    }

    let screen_rect = if input.dragging {
        state.marquee.and_then(|drag| {
            let pointer = input.pointer_position?;
            let end = grid_space_at_clamped(grid_rect, bounds, pointer);
            let rect = GridSelectionRect::from_points(drag.start, end);
            Some(Rect::from_min_max(
                grid_space_to_screen(grid_rect, bounds, rect.min),
                grid_space_to_screen(grid_rect, bounds, rect.max),
            ))
        })
    } else {
        None
    };

    MarqueeFrameResult {
        screen_rect,
        interaction: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanvasInteraction {
    Select {
        id: EntityId,
        mode: SelectionMode,
    },
    Place(GridPoint),
    Deselect,
    Marquee {
        ids: Vec<EntityId>,
        mode: SelectionMode,
    },
}

fn selection_mode_from_modifiers(shift: bool, ctrl: bool) -> SelectionMode {
    if ctrl {
        SelectionMode::Toggle
    } else if shift {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    }
}

pub(crate) fn resolve_grid_interaction(
    layout: &FactoryLayout,
    point: GridPoint,
    selected_block: Option<&BuildableId>,
    mode: SelectionMode,
) -> Option<CanvasInteraction> {
    if let Some(instance) = layout.instance_at(point) {
        Some(CanvasInteraction::Select {
            id: instance.id(),
            mode,
        })
    } else if selected_block.is_some() {
        Some(CanvasInteraction::Place(point))
    } else if mode == SelectionMode::Replace {
        Some(CanvasInteraction::Deselect)
    } else {
        None
    }
}

fn grid_space_at_clamped(grid_rect: Rect, bounds: GridSize, position: Pos2) -> Pos2 {
    let position = pos2(
        position.x.clamp(grid_rect.left(), grid_rect.right()),
        position.y.clamp(grid_rect.top(), grid_rect.bottom()),
    );
    pos2(
        (position.x - grid_rect.left()) / grid_rect.width() * f32::from(bounds.width()),
        (position.y - grid_rect.top()) / grid_rect.height() * f32::from(bounds.height()),
    )
}

fn grid_space_to_screen(grid_rect: Rect, bounds: GridSize, position: Pos2) -> Pos2 {
    pos2(
        grid_rect.left() + position.x / f32::from(bounds.width()) * grid_rect.width(),
        grid_rect.top() + position.y / f32::from(bounds.height()) * grid_rect.height(),
    )
}

fn marquee_ids(layout: &FactoryLayout, rect: GridSelectionRect) -> Vec<EntityId> {
    layout
        .instances()
        .filter(|instance| rect.contains_origin(instance.origin()))
        .map(|instance| instance.id())
        .collect()
}

fn marquee_start_at(
    layout: &FactoryLayout,
    grid_rect: Rect,
    bounds: GridSize,
    selected_block: Option<&BuildableId>,
    start_screen: Pos2,
    mode: SelectionMode,
) -> Option<MarqueeDrag> {
    if selected_block.is_some() {
        return None;
    }
    let point = grid_point_at(grid_rect, bounds, start_screen)?;
    if layout.instance_at(point).is_some() {
        return None;
    }

    Some(MarqueeDrag {
        start: grid_space_at_clamped(grid_rect, bounds, start_screen),
        mode,
    })
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

fn block_screen_rect(grid_rect: Rect, bounds: GridSize, resolved: ResolvedInstance<'_>) -> Rect {
    footprint_screen_rect(
        grid_rect,
        bounds,
        resolved.instance().origin(),
        resolved.effective_footprint(),
    )
}

fn selected_base_rect(
    neutral_grid: Rect,
    layout: &FactoryLayout,
    selected: &SelectedSet,
) -> Option<Rect> {
    selected
        .iter()
        .filter_map(|id| layout.resolved_instance(id))
        .map(|resolved| block_screen_rect(neutral_grid, layout.bounds(), resolved))
        .reduce(|combined, rect| combined.union(rect))
}

fn focus_selected_instances(
    viewport: &mut CanvasViewport,
    neutral_grid: Rect,
    available: Rect,
    layout: &FactoryLayout,
    selected: &SelectedSet,
) -> bool {
    let Some(target) = selected_base_rect(neutral_grid, layout, selected) else {
        return false;
    };
    viewport.frame_rect(target, available, available.center())
}

fn placement_preview_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    preview: &PlacementPreview,
    definition: &BuildableDefinition,
) -> Rect {
    footprint_screen_rect(grid_rect, bounds, preview.origin, definition.footprint())
}

fn block_visual(definition: &BuildableDefinition) -> (Color32, Color32, &str) {
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

fn placement_preview_visual(definition: &BuildableDefinition) -> (Color32, Color32) {
    let (fill, stroke, _) = block_visual(definition);
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
    selected: &SelectedSet,
) {
    let bounds = layout.bounds();

    for instance in layout.instances() {
        let resolved = layout
            .resolved_instance(instance.id())
            .expect("stored instance must resolve through the layout catalog");
        let definition = resolved.definition();
        let screen_rect = block_screen_rect(grid_rect, bounds, resolved).shrink(1.0);
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
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    layout: &FactoryLayout,
    title: &str,
    selected: &SelectedSet,
    selected_block: Option<&BuildableId>,
    state: &mut CanvasState,
) -> Option<CanvasInteraction> {
    let CanvasState {
        viewport,
        interaction,
        focus_selection_requested,
    } = state;
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
    let neutral_grid = fitted_grid_rect(grid_available, bounds);
    if *focus_selection_requested {
        if focus_selected_instances(viewport, neutral_grid, grid_available, layout, selected) {
            response.mark_changed();
        }
        *focus_selection_requested = false;
    }
    let grid_rect = viewport.transform_grid_rect(neutral_grid, viewport_anchor);
    let preview =
        placement_preview_for_hover(grid_rect, bounds, selected_block, response.hover_pos());
    let (selection_mode, press_origin) = ui.input(|input| {
        (
            selection_mode_from_modifiers(input.modifiers.shift, input.modifiers.ctrl),
            input.pointer.press_origin(),
        )
    });
    let marquee_frame = update_marquee_frame(
        interaction,
        layout,
        grid_rect,
        bounds,
        selected_block,
        MarqueeFrameInput {
            drag_started: response.drag_started_by(PointerButton::Primary),
            dragging: response.dragged_by(PointerButton::Primary),
            drag_stopped: response.drag_stopped_by(PointerButton::Primary),
            press_origin,
            pointer_position: response.interact_pointer_pos(),
            mode: selection_mode,
        },
    );

    for layer in canvas_paint_layers() {
        match layer {
            CanvasPaintLayer::Grid => paint_grid(&painter, grid_rect, bounds),
            CanvasPaintLayer::Preview => {
                if let Some(preview) = &preview {
                    let definition = layout
                        .catalog()
                        .buildable(&preview.buildable_id)
                        .expect("preview buildable ID must exist in the active catalog");
                    let screen_rect =
                        placement_preview_screen_rect(grid_rect, bounds, preview, definition)
                            .shrink(1.0);
                    let (fill, stroke) = placement_preview_visual(definition);
                    painter.rect_filled(screen_rect, 2, fill);
                    painter.rect_stroke(
                        screen_rect,
                        2,
                        Stroke::new(1.5, stroke),
                        StrokeKind::Inside,
                    );
                }
            }
            CanvasPaintLayer::Instances => paint_instances(&painter, grid_rect, layout, selected),
        }
    }
    if let Some(rect) = marquee_frame.screen_rect {
        painter.rect_filled(rect, 1, Color32::from_rgba_unmultiplied(91, 221, 199, 32));
        painter.rect_stroke(rect, 1, Stroke::new(1.5, ACCENT), StrokeKind::Inside);
    }
    painter.rect_stroke(grid_rect, 2, Stroke::new(1.5, ACCENT), StrokeKind::Inside);

    if let Some(interaction) = marquee_frame.interaction {
        return Some(interaction);
    }
    if !response.clicked_by(PointerButton::Primary) {
        return None;
    }

    response
        .interact_pointer_pos()
        .and_then(|position| grid_point_at(grid_rect, bounds, position))
        .and_then(|point| resolve_grid_interaction(layout, point, selected_block, selection_mode))
}

#[cfg(test)]
mod tests {
    use eframe::egui::{pos2, vec2, Rect};
    use factory_canvas::catalog_loader::load_embedded_public_catalog;
    use factory_canvas::domain::catalog::BuildableId;
    use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
    use factory_canvas::domain::layout::{BlockInstance, EntityId};

    use super::*;

    fn buildable_id(value: &str) -> BuildableId {
        BuildableId::new(value).expect("test buildable IDs must be valid")
    }

    fn main_layout() -> FactoryLayout {
        let catalog = load_embedded_public_catalog().expect("public test catalog must load");
        let base_id = catalog.default_base_id().clone();
        FactoryLayout::new(catalog, base_id).expect("public default base must exist")
    }

    fn public_buildable(buildable_id: BuildableId) -> BuildableDefinition {
        let catalog = load_embedded_public_catalog().expect("public test catalog must load");
        catalog
            .buildable(&buildable_id)
            .expect("template buildable exists in public catalog")
            .clone()
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn block_visual_uses_english_symbols() {
        assert_eq!(
            block_visual(&public_buildable(buildable_id("xiranite_power_pole"))).2,
            "XPP"
        );
        assert_eq!(
            block_visual(&public_buildable(buildable_id("refinery_unit"))).2,
            "RU"
        );
        assert_eq!(
            block_visual(&public_buildable(buildable_id("crushing_unit"))).2,
            "CU"
        );
    }

    #[test]
    fn block_visual_uses_neutral_colors_for_unknown_category() {
        let definition = BuildableDefinition::new(
            buildable_id("unknown_machine"),
            "Unknown Machine",
            factory_canvas::domain::catalog::CategoryId::new("unknown_category")
                .expect("test category ID must be valid"),
            "U",
            GridSize::new(1, 1).expect("test footprint must be valid"),
            vec![],
        );

        let (fill, stroke, symbol) = block_visual(&definition);

        assert_eq!(fill, Color32::from_rgb(65, 72, 82));
        assert_eq!(stroke, Color32::from_rgb(164, 174, 188));
        assert_eq!(symbol, "U");
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
    fn focus_selection_frames_complete_physical_bounds_without_mutating_layout() {
        let first_id = EntityId::new(1);
        let second_id = EntityId::new(2);
        let mut layout = main_layout();
        assert_eq!(
            layout.place(BlockInstance::new(
                first_id,
                buildable_id("xiranite_power_pole"),
                GridPoint::new(10, 10),
                Rotation::Zero,
            )),
            Ok(())
        );
        assert_eq!(
            layout.place(BlockInstance::new(
                second_id,
                buildable_id("refinery_unit"),
                GridPoint::new(20, 20),
                Rotation::Zero,
            )),
            Ok(())
        );
        let before = layout.clone();
        let mut selected = SelectedSet::new();
        selected.apply(SelectionMode::Replace, [first_id, second_id]);
        let available = Rect::from_min_max(pos2(100.0, 100.0), pos2(900.0, 900.0));
        let neutral_grid = fitted_grid_rect(available, layout.bounds());
        let target = selected_base_rect(neutral_grid, &layout, &selected).unwrap();
        let mut viewport = CanvasViewport::default();

        assert!(focus_selected_instances(
            &mut viewport,
            neutral_grid,
            available,
            &layout,
            &selected,
        ));

        let focused = viewport.transform_grid_rect(target, available.center());
        let safe = available.shrink(24.0);
        assert_close(focused.center().x, safe.center().x);
        assert_close(focused.center().y, safe.center().y);
        assert!(focused.left() >= safe.left());
        assert!(focused.right() <= safe.right());
        assert!(focused.top() >= safe.top());
        assert!(focused.bottom() <= safe.bottom());
        assert_eq!(layout, before);
    }

    #[test]
    fn focus_selection_without_selected_instances_is_noop() {
        let layout = main_layout();
        let selected = SelectedSet::new();
        let available = Rect::from_min_max(pos2(100.0, 100.0), pos2(900.0, 900.0));
        let neutral_grid = fitted_grid_rect(available, layout.bounds());
        let mut viewport = CanvasViewport::default();
        viewport.pan_by(vec2(10.0, 20.0));
        let before = viewport;

        assert!(!focus_selected_instances(
            &mut viewport,
            neutral_grid,
            available,
            &layout,
            &selected,
        ));
        assert_eq!(viewport, before);
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
    fn grid_interaction_preserves_occupancy_priority_and_selection_mode() {
        let id = EntityId::new(7);
        let instance = BlockInstance::new(
            id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(0, 0),
            Rotation::Zero,
        );
        let mut layout = main_layout();
        assert_eq!(layout.place(instance), Ok(()));

        assert_eq!(
            resolve_grid_interaction(
                &layout,
                GridPoint::new(1, 1),
                Some(&buildable_id("refinery_unit")),
                SelectionMode::Add,
            ),
            Some(CanvasInteraction::Select {
                id,
                mode: SelectionMode::Add,
            })
        );
        assert_eq!(
            resolve_grid_interaction(
                &layout,
                GridPoint::new(2, 0),
                Some(&buildable_id("refinery_unit")),
                SelectionMode::Toggle,
            ),
            Some(CanvasInteraction::Place(GridPoint::new(2, 0)))
        );
        assert_eq!(
            resolve_grid_interaction(&layout, GridPoint::new(2, 0), None, SelectionMode::Replace,),
            Some(CanvasInteraction::Deselect)
        );
        assert_eq!(
            resolve_grid_interaction(&layout, GridPoint::new(2, 0), None, SelectionMode::Add,),
            None
        );
    }

    #[test]
    fn modifier_mapping_prefers_ctrl_toggle_over_shift_add() {
        assert_eq!(
            selection_mode_from_modifiers(false, false),
            SelectionMode::Replace
        );
        assert_eq!(
            selection_mode_from_modifiers(true, false),
            SelectionMode::Add
        );
        assert_eq!(
            selection_mode_from_modifiers(false, true),
            SelectionMode::Toggle
        );
        assert_eq!(
            selection_mode_from_modifiers(true, true),
            SelectionMode::Toggle
        );
    }

    #[test]
    fn marquee_normalizes_drag_direction_and_selects_only_origins() {
        let mut layout = main_layout();
        for (value, origin) in [
            (1, GridPoint::new(1, 1)),
            (2, GridPoint::new(5, 5)),
            (3, GridPoint::new(7, 7)),
        ] {
            assert_eq!(
                layout.place(BlockInstance::new(
                    EntityId::new(value),
                    buildable_id("xiranite_power_pole"),
                    origin,
                    Rotation::Zero,
                )),
                Ok(())
            );
        }

        let rect = GridSelectionRect::from_points(pos2(5.0, 5.0), pos2(1.0, 1.0));
        assert_eq!(
            marquee_ids(&layout, rect),
            vec![EntityId::new(1), EntityId::new(2)]
        );

        let footprint_only = GridSelectionRect::from_points(pos2(8.0, 8.0), pos2(9.0, 9.0));
        assert!(marquee_ids(&layout, footprint_only).is_empty());
    }

    #[test]
    fn marquee_starts_only_on_empty_grid_without_placement_tool() {
        let id = EntityId::new(1);
        let mut layout = main_layout();
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                buildable_id("xiranite_power_pole"),
                GridPoint::new(0, 0),
                Rotation::Zero,
            )),
            Ok(())
        );
        let grid_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(900.0, 900.0));
        let bounds = GridSize::new(80, 80).unwrap();

        assert!(marquee_start_at(
            &layout,
            grid_rect,
            bounds,
            None,
            pos2(135.0, 135.0),
            SelectionMode::Replace,
        )
        .is_some());
        assert!(marquee_start_at(
            &layout,
            grid_rect,
            bounds,
            None,
            pos2(105.0, 105.0),
            SelectionMode::Replace,
        )
        .is_none());
        assert!(marquee_start_at(
            &layout,
            grid_rect,
            bounds,
            Some(&buildable_id("refinery_unit")),
            pos2(135.0, 135.0),
            SelectionMode::Replace,
        )
        .is_none());
    }

    #[test]
    fn marquee_frame_cycle_draws_emits_and_clears_captured_mode() {
        let mut layout = main_layout();
        for (value, origin) in [(1, GridPoint::new(3, 3)), (2, GridPoint::new(6, 6))] {
            assert_eq!(
                layout.place(BlockInstance::new(
                    EntityId::new(value),
                    buildable_id("xiranite_power_pole"),
                    origin,
                    Rotation::Zero,
                )),
                Ok(())
            );
        }
        let grid_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(900.0, 900.0));
        let bounds = GridSize::new(80, 80).unwrap();
        let mut state = CanvasInteractionState::default();

        let dragging = update_marquee_frame(
            &mut state,
            &layout,
            grid_rect,
            bounds,
            None,
            MarqueeFrameInput {
                drag_started: true,
                dragging: true,
                drag_stopped: false,
                press_origin: Some(pos2(125.0, 125.0)),
                pointer_position: Some(pos2(175.0, 175.0)),
                mode: SelectionMode::Add,
            },
        );
        assert_eq!(
            dragging.screen_rect,
            Some(Rect::from_min_max(pos2(125.0, 125.0), pos2(175.0, 175.0)))
        );
        assert_eq!(dragging.interaction, None);

        let released = update_marquee_frame(
            &mut state,
            &layout,
            grid_rect,
            bounds,
            None,
            MarqueeFrameInput {
                drag_started: false,
                dragging: false,
                drag_stopped: true,
                press_origin: None,
                pointer_position: Some(pos2(175.0, 175.0)),
                mode: SelectionMode::Replace,
            },
        );
        assert_eq!(released.screen_rect, None);
        assert_eq!(
            released.interaction,
            Some(CanvasInteraction::Marquee {
                ids: vec![EntityId::new(1), EntityId::new(2)],
                mode: SelectionMode::Add,
            })
        );
        assert!(state.marquee.is_none());
    }

    #[test]
    fn placement_preview_derives_active_template_and_hovered_tile_as_candidate_origin() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();

        assert_eq!(
            placement_preview_at(
                grid_rect,
                bounds,
                &buildable_id("refinery_unit"),
                pos2(128.0, 243.0),
            ),
            Some(PlacementPreview {
                buildable_id: buildable_id("refinery_unit"),
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
                Some(&buildable_id("refinery_unit")),
                Some(pos2(900.0, 600.0)),
            ),
            None
        );
        assert_eq!(
            placement_preview_for_hover(
                grid_rect,
                bounds,
                Some(&buildable_id("refinery_unit")),
                Some(pos2(128.0, 243.0)),
            ),
            Some(PlacementPreview {
                buildable_id: buildable_id("refinery_unit"),
                origin: GridPoint::new(2, 4),
            })
        );
    }

    #[test]
    fn placement_preview_screen_rect_uses_candidate_origin_and_template_footprint() {
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 600.0));
        let bounds = GridSize::new(80, 40).unwrap();
        let preview = PlacementPreview {
            buildable_id: buildable_id("refinery_unit"),
            origin: GridPoint::new(2, 4),
        };
        let definition = public_buildable(preview.buildable_id.clone());

        let screen_rect = placement_preview_screen_rect(grid_rect, bounds, &preview, &definition);

        assert_close(screen_rect.left(), 120.0);
        assert_close(screen_rect.top(), 240.0);
        assert_close(screen_rect.right(), 150.0);
        assert_close(screen_rect.bottom(), 270.0);
    }

    #[test]
    fn placement_preview_visual_keeps_block_colors_with_translucent_fill() {
        let definition = public_buildable(buildable_id("refinery_unit"));
        let (block_fill, block_stroke, _) = block_visual(&definition);

        assert_eq!(
            placement_preview_visual(&definition),
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
        let grid_rect = Rect::from_min_max(pos2(100.0, 200.0), pos2(900.0, 1000.0));
        let mut layout = main_layout();
        let id = EntityId::new(7);
        let instance = BlockInstance::new(
            id,
            buildable_id("refinery_unit"),
            GridPoint::new(2, 4),
            Rotation::Zero,
        );
        layout.place(instance).expect("test instance should fit");
        let resolved = layout
            .resolved_instance(id)
            .expect("placed instance should resolve");

        let screen_rect = block_screen_rect(grid_rect, layout.bounds(), resolved);

        assert_close(screen_rect.left(), 120.0);
        assert_close(screen_rect.top(), 240.0);
        assert_close(screen_rect.right(), 150.0);
        assert_close(screen_rect.bottom(), 270.0);
    }
}
