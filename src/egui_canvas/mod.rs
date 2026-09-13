mod geometry;
mod painting;
mod rotation;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
mod viewport;

use eframe::egui::{Align2, Color32, FontId};
use eframe::egui::{CursorIcon, PointerButton, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use factory_canvas::domain::blueprint::Blueprint;
use factory_canvas::domain::catalog::BuildableId;
use factory_canvas::domain::geometry::{GridPoint, GridSize};
use factory_canvas::domain::layout::{EntityId, FactoryLayout};

use crate::egui_app::icons::BuildableIcons;
use crate::selected_set::{SelectedSet, SelectionMode};

use geometry::{
    blueprint_preview_for_hover, focus_selected_instances, grid_point_at, grid_space_at_clamped,
    grid_space_to_screen, marquee_ids, marquee_start_at, placement_preview_for_hover,
    placement_preview_screen_rect, resolve_grid_interaction, selection_mode_from_modifiers,
    GridSelectionRect, MarqueeDrag,
};
use painting::{
    canvas_paint_layers, paint_grid, paint_instances, placement_preview_visual, CanvasPaintLayer,
};
pub(crate) use rotation::RotationVisuals;
use viewport::apply_canvas_viewport_gesture;
pub(crate) use viewport::CanvasViewport;

pub(crate) const CANVAS_BACKGROUND: Color32 = Color32::from_rgb(10, 17, 26);
const GRID_BACKGROUND: Color32 = Color32::from_rgb(15, 29, 41);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 237, 242);
const TEXT_MUTED: Color32 = Color32::from_rgb(130, 151, 163);
const ACCENT: Color32 = Color32::from_rgb(91, 221, 199);
const BORDER: Color32 = Color32::from_rgb(35, 53, 67);

pub(crate) fn fitted_grid_rect(available: Rect, bounds: GridSize) -> Rect {
    let tile_size = (available.width() / f32::from(bounds.width()))
        .min(available.height() / f32::from(bounds.height()));
    let size = eframe::egui::vec2(
        tile_size * f32::from(bounds.width()),
        tile_size * f32::from(bounds.height()),
    );

    Rect::from_center_size(available.center(), size)
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
    /// Deliberately NOT touched by `clear_transient_interaction` — reset
    /// only by `RotationVisuals::resync`, called from `egui_app.rs`'s
    /// four whole-layout-replacing operations (research.md Decision 3).
    pub(crate) rotation_visuals: RotationVisuals,
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
    press_origin: Option<eframe::egui::Pos2>,
    pointer_position: Option<eframe::egui::Pos2>,
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
    has_armed_blueprint: bool,
    input: MarqueeFrameInput,
) -> MarqueeFrameResult {
    if input.drag_started {
        state.marquee = input.press_origin.and_then(|origin| {
            marquee_start_at(
                layout,
                grid_rect,
                bounds,
                selected_block,
                has_armed_blueprint,
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
    PlaceBlueprint(GridPoint),
    Deselect,
    Marquee {
        ids: Vec<EntityId>,
        mode: SelectionMode,
    },
}

pub(crate) struct CanvasFrameInput<'a> {
    pub(crate) layout: &'a FactoryLayout,
    pub(crate) title: &'a str,
    pub(crate) selected: &'a SelectedSet,
    pub(crate) selected_block: Option<&'a BuildableId>,
    pub(crate) armed_blueprint: Option<&'a Blueprint>,
    pub(crate) icons: &'a BuildableIcons,
}

pub(crate) fn show(
    ui: &mut Ui,
    input: CanvasFrameInput<'_>,
    state: &mut CanvasState,
) -> Option<CanvasInteraction> {
    let CanvasFrameInput {
        layout,
        title,
        selected,
        selected_block,
        armed_blueprint,
        icons,
    } = input;
    let CanvasState {
        viewport,
        interaction,
        focus_selection_requested,
        rotation_visuals,
    } = state;
    let available_size = ui.available_size().max(Vec2::splat(1.0));
    let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
    let mut response = if selected_block.is_some() || armed_blueprint.is_some() {
        response.on_hover_cursor(CursorIcon::Crosshair)
    } else {
        response
    };
    let outer_rect = response.rect;

    painter.rect_filled(outer_rect, 12, CANVAS_BACKGROUND);
    painter.rect_stroke(outer_rect, 12, Stroke::new(1.0, BORDER), StrokeKind::Inside);

    let bounds = layout.bounds();
    let title_position = eframe::egui::pos2(outer_rect.left() + 24.0, outer_rect.top() + 22.0);
    painter.text(
        title_position,
        Align2::LEFT_TOP,
        title,
        FontId::proportional(18.0),
        TEXT_PRIMARY,
    );
    painter.text(
        eframe::egui::pos2(title_position.x, title_position.y + 25.0),
        Align2::LEFT_TOP,
        format!("{} × {} tiles", bounds.width(), bounds.height()),
        FontId::proportional(11.0),
        TEXT_MUTED,
    );

    let mut grid_available = outer_rect.shrink2(eframe::egui::vec2(36.0, 32.0));
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
    let blueprint_preview = blueprint_preview_for_hover(
        grid_rect,
        bounds,
        armed_blueprint,
        layout.catalog(),
        response.hover_pos(),
    );
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
        armed_blueprint.is_some(),
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
                for rect in &blueprint_preview {
                    let screen_rect = rect.shrink(1.0);
                    painter.rect_filled(
                        screen_rect,
                        2,
                        Color32::from_rgba_unmultiplied(91, 221, 199, 60),
                    );
                    painter.rect_stroke(
                        screen_rect,
                        2,
                        Stroke::new(1.5, ACCENT),
                        StrokeKind::Inside,
                    );
                }
            }
            CanvasPaintLayer::Instances => paint_instances(
                &painter,
                grid_rect,
                layout,
                selected,
                rotation_visuals,
                icons,
            ),
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

    response.interact_pointer_pos().and_then(|position| {
        grid_point_at(grid_rect, bounds, position).and_then(|point| {
            resolve_grid_interaction(
                layout,
                point,
                selected_block,
                armed_blueprint,
                selection_mode,
            )
        })
    })
}
