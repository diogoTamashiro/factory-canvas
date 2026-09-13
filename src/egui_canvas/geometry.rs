use eframe::egui::{pos2, Pos2, Rect};
use factory_canvas::domain::blueprint::Blueprint;
use factory_canvas::domain::catalog::{BuildableDefinition, BuildableId, Catalog};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{EntityId, FactoryLayout, ResolvedInstance};

use crate::selected_set::{SelectedSet, SelectionMode};

use super::viewport::CanvasViewport;
use super::CanvasInteraction;

pub(super) fn grid_point_at(
    grid_rect: Rect,
    bounds: GridSize,
    position: Pos2,
) -> Option<GridPoint> {
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
    pub(super) buildable_id: BuildableId,
    pub(super) origin: GridPoint,
}

pub(super) fn placement_preview_at(
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

pub(super) fn placement_preview_for_hover(
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

/// One screen-space rect per node of `armed_blueprint`, positioned at
/// `insertion_point + node.relative_origin` where `insertion_point` is
/// the grid tile under the cursor — the multi-node equivalent of
/// `placement_preview_for_hover`/`PlacementPreview`, per research.md
/// Decision 5 ("reusing the existing interaction pattern"). Returns an
/// empty `Vec` (not an `Option`) since "no preview" and "zero nodes" are
/// both simply nothing to paint.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct BlueprintPreviewNode {
    pub(super) screen_rect: Rect,
    pub(super) buildable_id: BuildableId,
    pub(super) rotation: Rotation,
}

pub(super) fn blueprint_preview_for_hover(
    grid_rect: Rect,
    bounds: GridSize,
    armed_blueprint: Option<&Blueprint>,
    catalog: &Catalog,
    hover_position: Option<Pos2>,
) -> Vec<BlueprintPreviewNode> {
    let Some((blueprint, position)) = armed_blueprint.zip(hover_position) else {
        return Vec::new();
    };
    let Some(insertion_point) = grid_point_at(grid_rect, bounds, position) else {
        return Vec::new();
    };

    blueprint
        .nodes()
        .iter()
        .filter_map(|node| {
            let footprint = catalog.buildable(node.buildable_id())?.footprint();
            let rotated_footprint = node.rotation().apply_to(footprint);
            let origin = GridPoint::new(
                insertion_point.x.checked_add(node.relative_origin().x)?,
                insertion_point.y.checked_add(node.relative_origin().y)?,
            );
            Some(BlueprintPreviewNode {
                screen_rect: footprint_screen_rect(grid_rect, bounds, origin, rotated_footprint),
                buildable_id: node.buildable_id().clone(),
                rotation: node.rotation(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct GridSelectionRect {
    pub(super) min: Pos2,
    pub(super) max: Pos2,
}

impl GridSelectionRect {
    pub(super) fn from_points(a: Pos2, b: Pos2) -> Self {
        Self {
            min: pos2(a.x.min(b.x), a.y.min(b.y)),
            max: pos2(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub(super) fn contains_origin(self, origin: GridPoint) -> bool {
        let x = origin.x as f32;
        let y = origin.y as f32;
        self.min.x <= x && x <= self.max.x && self.min.y <= y && y <= self.max.y
    }
}

pub(super) fn selection_mode_from_modifiers(shift: bool, ctrl: bool) -> SelectionMode {
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
    armed_blueprint: Option<&Blueprint>,
    mode: SelectionMode,
) -> Option<CanvasInteraction> {
    if let Some(instance) = layout.instance_at(point) {
        Some(CanvasInteraction::Select {
            id: instance.id(),
            mode,
        })
    } else if selected_block.is_some() {
        Some(CanvasInteraction::Place(point))
    } else if armed_blueprint.is_some() {
        Some(CanvasInteraction::PlaceBlueprint(point))
    } else if mode == SelectionMode::Replace {
        Some(CanvasInteraction::Deselect)
    } else {
        None
    }
}

pub(super) fn grid_space_at_clamped(grid_rect: Rect, bounds: GridSize, position: Pos2) -> Pos2 {
    let position = pos2(
        position.x.clamp(grid_rect.left(), grid_rect.right()),
        position.y.clamp(grid_rect.top(), grid_rect.bottom()),
    );
    pos2(
        (position.x - grid_rect.left()) / grid_rect.width() * f32::from(bounds.width()),
        (position.y - grid_rect.top()) / grid_rect.height() * f32::from(bounds.height()),
    )
}

pub(super) fn grid_space_to_screen(grid_rect: Rect, bounds: GridSize, position: Pos2) -> Pos2 {
    pos2(
        grid_rect.left() + position.x / f32::from(bounds.width()) * grid_rect.width(),
        grid_rect.top() + position.y / f32::from(bounds.height()) * grid_rect.height(),
    )
}

pub(super) fn marquee_ids(layout: &FactoryLayout, rect: GridSelectionRect) -> Vec<EntityId> {
    layout
        .instances()
        .filter(|instance| rect.contains_origin(instance.origin()))
        .map(|instance| instance.id())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MarqueeDrag {
    pub(super) start: Pos2,
    pub(super) mode: SelectionMode,
}

pub(super) fn marquee_start_at(
    layout: &FactoryLayout,
    grid_rect: Rect,
    bounds: GridSize,
    selected_block: Option<&BuildableId>,
    has_armed_blueprint: bool,
    start_screen: Pos2,
    mode: SelectionMode,
) -> Option<MarqueeDrag> {
    if selected_block.is_some() || has_armed_blueprint {
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

pub(super) fn footprint_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    origin: GridPoint,
    footprint: GridSize,
) -> Rect {
    footprint_screen_rect_fractional(
        grid_rect,
        bounds,
        origin.x as f32,
        origin.y as f32,
        footprint,
    )
}

/// Same mapping as `footprint_screen_rect`, but accepts a fractional
/// grid-space origin — used to paint an instance mid-way through a
/// rotation-driven move (research.md Decisions 2/5): interpolating
/// these floats frame-to-frame is what makes the block visibly slide
/// rather than jump tile-to-tile.
pub(super) fn footprint_screen_rect_fractional(
    grid_rect: Rect,
    bounds: GridSize,
    origin_x: f32,
    origin_y: f32,
    footprint: GridSize,
) -> Rect {
    let tile_width = grid_rect.width() / f32::from(bounds.width());
    let tile_height = grid_rect.height() / f32::from(bounds.height());
    let min = pos2(
        grid_rect.left() + origin_x * tile_width,
        grid_rect.top() + origin_y * tile_height,
    );
    let max = pos2(
        min.x + f32::from(footprint.width()) * tile_width,
        min.y + f32::from(footprint.height()) * tile_height,
    );

    Rect::from_min_max(min, max)
}

pub(super) fn block_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    resolved: ResolvedInstance<'_>,
) -> Rect {
    footprint_screen_rect(
        grid_rect,
        bounds,
        resolved.instance().origin(),
        resolved.effective_footprint(),
    )
}

pub(super) fn selected_base_rect(
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

pub(super) fn focus_selected_instances(
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

pub(super) fn placement_preview_screen_rect(
    grid_rect: Rect,
    bounds: GridSize,
    preview: &PlacementPreview,
    definition: &BuildableDefinition,
) -> Rect {
    footprint_screen_rect(grid_rect, bounds, preview.origin, definition.footprint())
}
