use eframe::egui::{self, pos2, vec2, Color32, Rect};
use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::catalog::{BuildableDefinition, BuildableId};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId};

use super::geometry::{
    block_screen_rect, blueprint_preview_for_hover, focus_selected_instances, grid_point_at,
    marquee_ids, marquee_start_at, placement_preview_at, placement_preview_for_hover,
    placement_preview_screen_rect, resolve_grid_interaction, selected_base_rect,
    selection_mode_from_modifiers, GridSelectionRect, PlacementPreview,
};
use super::painting::{
    block_visual, canvas_paint_layers, paint_instances, paint_orientation_dot,
    placement_preview_visual, CanvasPaintLayer,
};
use super::rotation::rotation_degrees;
use super::viewport::{apply_canvas_viewport_gesture, zoom_factor_from_wheel_delta};
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

/// Drives one `egui` frame at an explicit simulated `time`, running
/// `f` inside it. Mirrors this feature's `/speckit-plan` spike, which
/// confirmed `egui::Context::animate_value_with_time` reads its
/// elapsed-time calculations from `RawInput.time`, not wall-clock
/// time — this lets tests assert exact interpolated values at
/// specific, deterministic points in a transition.
fn frame_at(context: &egui::Context, time: f64, mut f: impl FnMut(&egui::Context)) {
    let input = egui::RawInput {
        time: Some(time),
        predicted_dt: 1.0 / 60.0,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| f(ui.ctx()));
    output.platform_output.accesskit_update.take();
    output.drop_without_applying_deltas();
}

#[test]
fn orientation_dot_stays_within_the_footprints_shorter_half_extent_through_the_full_sweep() {
    // Regression coverage for the reverted "custom buildable icons"
    // feature's bug (Diogo's report): that feature rotated the
    // fallback text label itself, which made plain text spin
    // illegibly. paint_orientation_dot replaces that with a small
    // orbiting dot instead — this test locks in the geometric safety
    // property a visual check can't: the dot's screen-space distance
    // from the rect's center never exceeds the shorter half-extent,
    // for a full continuous sweep of angles (not just the four
    // resting 0/90/180/270 values), matching how RotationVisuals
    // actually interpolates through arbitrary intermediate angles
    // mid-transition.
    let context = egui::Context::default();
    let narrow_rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(60.0, 30.0));
    let shorter_half_extent = narrow_rect.width().min(narrow_rect.height()) / 2.0;

    let mut angle_degrees = 0.0;
    while angle_degrees < 360.0 {
        let input = egui::RawInput::default();
        let mut output = context.run_ui(input, |ui| {
            let painter = ui.painter();
            paint_orientation_dot(painter, narrow_rect, angle_degrees, Color32::WHITE);
        });
        output.platform_output.accesskit_update.take();
        let shapes = output.shapes.clone();
        output.drop_without_applying_deltas();

        let circle = shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Circle(circle) => Some(circle),
                _ => None,
            })
            .expect("paint_orientation_dot must emit exactly one circle shape");
        let distance_from_center = (circle.center - narrow_rect.center()).length();
        assert!(
            distance_from_center <= shorter_half_extent,
            "at angle {angle_degrees}: dot center is {distance_from_center} \
             from rect center, exceeding the shorter half-extent {shorter_half_extent}"
        );

        angle_degrees += 15.0;
    }
}

#[test]
fn paint_instances_never_emits_a_rotated_text_shape_for_a_fallback_label() {
    // Regression test for the actual reported bug: a placed instance
    // with no usable icon must show its label upright at every point
    // during an in-flight rotation transition, never as a rotated
    // Shape::Text. Drives paint_instances directly (the real
    // production call path), at a genuinely mid-transition angle, and
    // inspects the emitted Shape::Text's `angle` field rather than
    // reasoning about the source — the same standard this project's
    // T053 test already set for this function.
    use crate::egui_app::icons::BuildableIcons;
    use crate::selected_set::SelectedSet;

    let icons = BuildableIcons::empty();
    let mut layout = main_layout();
    let id = EntityId::new(1);
    layout
        .place(BlockInstance::new(
            id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("test instance must be placeable");

    let selected = SelectedSet::new();
    let mut visuals = RotationVisuals::default();
    let grid_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 800.0));
    let context = egui::Context::default();

    // Establish the resting state, then trigger a rotation and read a
    // mid-flight frame — the exact sequence single_instance_rotation_
    // animates_smoothly_between_old_and_new_angle above already
    // confirms lands strictly between the old and new angle.
    for time in [0.0, 0.0] {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 800.0))),
            time: Some(time),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            let painter = ui.painter();
            paint_instances(painter, grid_rect, &layout, &selected, &mut visuals, &icons);
        });
        output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        if time == 0.0 {
            layout
                .rotate_instance(id, Rotation::Clockwise90)
                .expect("test rotation must be accepted");
        }
    }

    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 800.0))),
        time: Some(0.1),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        let painter = ui.painter();
        paint_instances(painter, grid_rect, &layout, &selected, &mut visuals, &icons);
    });
    output.platform_output.accesskit_update.take();
    let shapes = output.shapes.clone();
    output.drop_without_applying_deltas();

    let text_shapes: Vec<_> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text_shape) => Some(text_shape),
            _ => None,
        })
        .collect();
    assert!(
        !text_shapes.is_empty(),
        "the text-only instance must still paint its label"
    );
    for text_shape in text_shapes {
        assert_eq!(
            text_shape.angle, 0.0,
            "a fallback text label must never rotate, even mid-transition \
             (this is the exact bug Diogo reported: plain text spinning)"
        );
    }
}

#[test]
fn paint_preview_representation_never_rotates_the_fallback_text_but_still_rotates_an_icon() {
    // Same bug class as paint_instances_never_emits_a_rotated_text_shape_
    // for_a_fallback_label above, but for the shared preview helper
    // (single-buildable AND blueprint-member previews both call this).
    // A non-zero angle must still rotate a real icon texture — icons
    // are unaffected by Diogo's bug report — while the text-only
    // fallback branch must never rotate.
    let context = egui::Context::default();
    let rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(60.0, 60.0));
    let non_zero_angle = 37.0;

    let input = egui::RawInput::default();
    let mut output = context.run_ui(input, |ui| {
        let painter = ui.painter();
        super::painting::paint_preview_representation(
            painter,
            rect,
            None,
            "XPP",
            non_zero_angle,
            0.65,
        );
    });
    output.platform_output.accesskit_update.take();
    let shapes = output.shapes.clone();
    output.drop_without_applying_deltas();

    let text_shape = shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text_shape) => Some(text_shape),
            _ => None,
        })
        .expect("the text-only fallback branch must still paint a label");
    assert_eq!(
        text_shape.angle, 0.0,
        "a preview's fallback text must never rotate, matching the placed-instance fix"
    );

    let color_image = egui::ColorImage::filled([1, 1], egui::Color32::WHITE);
    let handle = context.load_texture(
        "paint_preview_representation_icon_rotation_test_texture",
        color_image,
        egui::TextureOptions::LINEAR,
    );
    let input = egui::RawInput::default();
    let mut output = context.run_ui(input, |ui| {
        let painter = ui.painter();
        super::painting::paint_preview_representation(
            painter,
            rect,
            Some(&handle),
            "XPP",
            non_zero_angle,
            0.65,
        );
    });
    output.platform_output.accesskit_update.take();
    let shapes = output.shapes.clone();
    output.drop_without_applying_deltas();

    let has_no_upright_text_shape = !shapes
        .iter()
        .any(|clipped| matches!(&clipped.shape, egui::Shape::Text(_)));
    assert!(
        has_no_upright_text_shape,
        "an icon-bearing preview must paint the icon, not a text fallback"
    );
    let has_mesh_shape = shapes
        .iter()
        .any(|clipped| matches!(&clipped.shape, egui::Shape::Mesh(_)));
    assert!(
        has_mesh_shape,
        "an icon-bearing preview must paint the rotated icon as a mesh \
         (egui::paint_texture_at's underlying shape), still honoring \
         angle_degrees — only the TEXT fallback stopped rotating"
    );
}

#[test]
fn orientation_indicator_matches_resting_rotation_for_every_instance() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();

    for (id_value, rotation) in [
        (1_u64, Rotation::Zero),
        (2, Rotation::Clockwise90),
        (3, Rotation::Clockwise180),
        (4, Rotation::Clockwise270),
    ] {
        let id = EntityId::new(id_value);
        let origin = GridPoint::new(0, 0);
        frame_at(&context, 0.0, |ctx| {
            let (angle, x, y) = visuals.visual_state_for(ctx, id, rotation, origin);
            assert_close(angle, rotation_degrees(rotation));
            assert_close(x, 0.0);
            assert_close(y, 0.0);
        });
    }
}

#[test]
fn single_instance_rotation_animates_smoothly_between_old_and_new_angle() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id = EntityId::new(1);
    let origin = GridPoint::new(5, 5);

    // Establish the resting state at Zero (first-ever call: no
    // interpolation, per research.md Decision 1).
    frame_at(&context, 0.0, |ctx| {
        let (angle, _, _) = visuals.visual_state_for(ctx, id, Rotation::Zero, origin);
        assert_close(angle, 0.0);
    });

    // The frame a rotation is FIRST observed always reads 0% into
    // its own transition (egui's animation clock starts counting
    // from this exact frame, confirmed by this feature's spike) —
    // so triggering and reading progress require two frames.
    frame_at(&context, 0.05, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
    });

    // A later frame, still inside the transition window, must read
    // a value strictly between the old and new angle.
    frame_at(&context, 0.1, |ctx| {
        let (angle, _, _) = visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
        assert!(
            angle > 0.0 && angle < 90.0,
            "angle {angle} must be strictly between 0.0 and 90.0 mid-transition"
        );
    });
}

#[test]
fn rotation_transition_ends_exactly_at_the_domain_accepted_value() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id = EntityId::new(1);
    let origin = GridPoint::new(5, 5);

    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Zero, origin);
    });
    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
    });
    // Well past the transition window's end.
    frame_at(&context, 10.0, |ctx| {
        let (angle, x, y) = visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
        assert_close(angle, 90.0);
        assert_close(x, 5.0);
        assert_close(y, 5.0);
    });
}

#[test]
fn a_second_rotation_mid_transition_continues_from_the_current_angle() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id = EntityId::new(1);
    let origin = GridPoint::new(0, 0);

    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Zero, origin);
    });
    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
    });

    let mut mid_flight_angle = 0.0;
    frame_at(&context, 0.1, |ctx| {
        let (angle, _, _) = visuals.visual_state_for(ctx, id, Rotation::Clockwise90, origin);
        mid_flight_angle = angle;
    });

    // Second rotation, triggered while the first is still mid-flight.
    let mut angle_immediately_after_retarget = 0.0;
    frame_at(&context, 0.1, |ctx| {
        let (angle, _, _) = visuals.visual_state_for(ctx, id, Rotation::Clockwise180, origin);
        angle_immediately_after_retarget = angle;
    });

    assert_close(angle_immediately_after_retarget, mid_flight_angle);
}

#[test]
fn rejected_rotation_starts_no_transition_and_changes_nothing() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id = EntityId::new(1);
    let origin = GridPoint::new(5, 5);

    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Zero, origin);
    });

    let before = format!("{visuals:?}");

    // A rejected rotation means the caller never mutates the
    // domain's stored Rotation at all — simulated here by calling
    // `visual_state_for` again with the SAME (unchanged) rotation,
    // exactly what production code would observe from `self.layout`
    // after a rejected `rotate_selected_clockwise` attempt.
    frame_at(&context, 0.05, |ctx| {
        let (angle, x, y) = visuals.visual_state_for(ctx, id, Rotation::Zero, origin);
        assert_close(angle, 0.0);
        assert_close(x, 5.0);
        assert_close(y, 5.0);
    });

    let after = format!("{visuals:?}");
    assert_eq!(before, after, "bookkeeping must be byte-for-byte unchanged");
}

#[test]
fn group_rotation_animates_position_and_angle_together_for_every_member() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id_1 = EntityId::new(1);
    let id_2 = EntityId::new(2);
    // A group orbital rotation about a shared pivot moves both
    // members AND turns both members — exactly what the domain's
    // `rotate_instances_clockwise_about` already computes; this test
    // only exercises the presentation layer's reaction to it, using
    // representative before/after values for two members.
    let id_1_origin_before = GridPoint::new(10, 10);
    let id_1_origin_after = GridPoint::new(12, 10);
    let id_2_origin_before = GridPoint::new(10, 12);
    let id_2_origin_after = GridPoint::new(10, 10);

    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id_1, Rotation::Zero, id_1_origin_before);
        visuals.visual_state_for(ctx, id_2, Rotation::Zero, id_2_origin_before);
    });
    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id_1, Rotation::Clockwise90, id_1_origin_after);
        visuals.visual_state_for(ctx, id_2, Rotation::Clockwise90, id_2_origin_after);
    });

    // The frame a rotation is FIRST observed always reads 0% into
    // its own transition (confirmed by T006's identical fix) — an
    // extra frame is needed before reading mid-flight progress.
    frame_at(&context, 0.05, |ctx| {
        visuals.visual_state_for(ctx, id_1, Rotation::Clockwise90, id_1_origin_after);
        visuals.visual_state_for(ctx, id_2, Rotation::Clockwise90, id_2_origin_after);
    });

    frame_at(&context, 0.1, |ctx| {
        let (angle_1, x_1, y_1) =
            visuals.visual_state_for(ctx, id_1, Rotation::Clockwise90, id_1_origin_after);
        let (angle_2, x_2, y_2) =
            visuals.visual_state_for(ctx, id_2, Rotation::Clockwise90, id_2_origin_after);

        assert!(angle_1 > 0.0 && angle_1 < 90.0, "member 1 angle: {angle_1}");
        assert!(angle_2 > 0.0 && angle_2 < 90.0, "member 2 angle: {angle_2}");
        assert!(
            x_1 > id_1_origin_before.x as f32 && x_1 < id_1_origin_after.x as f32,
            "member 1 x: {x_1}"
        );
        assert!(
            y_2 < id_2_origin_before.y as f32 && y_2 > id_2_origin_after.y as f32,
            "member 2 y: {y_2}"
        );
        let _ = y_1;
        let _ = x_2;
    });

    frame_at(&context, 10.0, |ctx| {
        let (angle_1, x_1, y_1) =
            visuals.visual_state_for(ctx, id_1, Rotation::Clockwise90, id_1_origin_after);
        let (angle_2, x_2, y_2) =
            visuals.visual_state_for(ctx, id_2, Rotation::Clockwise90, id_2_origin_after);

        assert_close(angle_1, 90.0);
        assert_close(angle_2, 90.0);
        assert_close(x_1, id_1_origin_after.x as f32);
        assert_close(y_1, id_1_origin_after.y as f32);
        assert_close(x_2, id_2_origin_after.x as f32);
        assert_close(y_2, id_2_origin_after.y as f32);
    });
}

#[test]
fn plain_move_without_rotation_never_animates_position() {
    let context = egui::Context::default();
    let mut visuals = RotationVisuals::default();
    let id = EntityId::new(1);
    let origin_before = GridPoint::new(5, 5);
    let origin_after = GridPoint::new(6, 5);

    frame_at(&context, 0.0, |ctx| {
        visuals.visual_state_for(ctx, id, Rotation::Zero, origin_before);
    });

    // An ordinary move: the origin changes but Rotation does not —
    // must render at the new position immediately, never
    // interpolated (research.md Decision 4).
    frame_at(&context, 0.01, |ctx| {
        let (angle, x, y) = visuals.visual_state_for(ctx, id, Rotation::Zero, origin_after);
        assert_close(angle, 0.0);
        assert_close(x, origin_after.x as f32);
        assert_close(y, origin_after.y as f32);
    });
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
        None,
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
            None,
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
            None,
            SelectionMode::Toggle,
        ),
        Some(CanvasInteraction::Place(GridPoint::new(2, 0)))
    );
    assert_eq!(
        resolve_grid_interaction(
            &layout,
            GridPoint::new(2, 0),
            None,
            None,
            SelectionMode::Replace,
        ),
        Some(CanvasInteraction::Deselect)
    );
    assert_eq!(
        resolve_grid_interaction(
            &layout,
            GridPoint::new(2, 0),
            None,
            None,
            SelectionMode::Add,
        ),
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
        false,
        pos2(135.0, 135.0),
        SelectionMode::Replace,
    )
    .is_some());
    assert!(marquee_start_at(
        &layout,
        grid_rect,
        bounds,
        None,
        false,
        pos2(105.0, 105.0),
        SelectionMode::Replace,
    )
    .is_none());
    assert!(marquee_start_at(
        &layout,
        grid_rect,
        bounds,
        Some(&buildable_id("refinery_unit")),
        false,
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
        false,
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
        false,
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
            Color32::from_rgba_unmultiplied(block_fill.r(), block_fill.g(), block_fill.b(), 112),
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

#[test]
fn blueprint_preview_for_hover_carries_each_nodes_buildable_id_and_rotation() {
    use factory_canvas::domain::blueprint::{Blueprint, BlueprintId};
    use factory_canvas::domain::document::DocumentMetadata;

    let grid_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 800.0));
    let mut layout = main_layout();
    let bounds = layout.bounds();
    let pole_id = EntityId::new(1);
    let refinery_id = EntityId::new(2);
    layout
        .place(BlockInstance::new(
            pole_id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 10),
            Rotation::Clockwise90,
        ))
        .expect("pole must be placeable");
    layout
        .place(BlockInstance::new(
            refinery_id,
            buildable_id("refinery_unit"),
            GridPoint::new(20, 10),
            Rotation::Zero,
        ))
        .expect("refinery must be placeable");

    let blueprint_id = BlueprintId::parse("blueprint_00000000000000000000000000000001")
        .expect("test blueprint ID must be valid");
    let metadata = DocumentMetadata::new(
        "Test Blueprint",
        None,
        time::OffsetDateTime::UNIX_EPOCH,
        time::OffsetDateTime::UNIX_EPOCH,
    )
    .expect("test blueprint metadata must be valid");
    let blueprint = Blueprint::from_selection(
        &layout,
        vec![pole_id, refinery_id],
        blueprint_id,
        metadata,
        Vec::new(),
    )
    .expect("selection of two placed entities must produce a blueprint");

    let hover_position = pos2(grid_rect.left() + 5.0, grid_rect.top() + 5.0);
    let nodes = blueprint_preview_for_hover(
        grid_rect,
        bounds,
        Some(&blueprint),
        layout.catalog(),
        Some(hover_position),
    );

    assert_eq!(nodes.len(), 2);
    let by_buildable: std::collections::BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (node.buildable_id.clone(), node.rotation))
        .collect();
    assert_eq!(
        by_buildable.get(&buildable_id("xiranite_power_pole")),
        Some(&Rotation::Clockwise90)
    );
    assert_eq!(
        by_buildable.get(&buildable_id("refinery_unit")),
        Some(&Rotation::Zero)
    );
    for node in &nodes {
        assert!(node.screen_rect.width() > 0.0 && node.screen_rect.height() > 0.0);
    }
}

#[test]
fn paint_instances_leaves_rotation_visuals_state_identical_regardless_of_icon_presence() {
    // T053: confirms paint_instances's newly-added icon/text branch
    // (research.md Decision 6) does not change RotationVisuals's own
    // bookkeeping contract — a rejected rotation (simulated, as the
    // existing rejected_rotation_starts_no_transition_and_changes_nothing
    // test above already does, by calling with an unchanged Rotation)
    // must leave RotationVisuals byte-for-byte identical whether the
    // instance being painted has a real icon texture or falls back to
    // text.
    use crate::egui_app::icons::BuildableIcons;
    use crate::selected_set::SelectedSet;

    fn run_paint_instances_twice_and_compare(icons: &BuildableIcons, layout: &FactoryLayout) {
        let selected = SelectedSet::new();
        let mut visuals = RotationVisuals::default();
        let grid_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 800.0));

        let context = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 800.0))),
            time: Some(0.0),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            let painter = ui.painter();
            paint_instances(painter, grid_rect, layout, &selected, &mut visuals, icons);
        });
        output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();

        let before = format!("{visuals:?}");

        // Same unchanged Rotation on the next frame — simulates a
        // rejected rotation attempt exactly as the existing
        // rejected_rotation_starts_no_transition_and_changes_nothing
        // test does, but now going through the real paint_instances
        // call path instead of calling visual_state_for directly.
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 800.0))),
            time: Some(0.05),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            let painter = ui.painter();
            paint_instances(painter, grid_rect, layout, &selected, &mut visuals, icons);
        });
        output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();

        let after = format!("{visuals:?}");
        assert_eq!(
            before, after,
            "RotationVisuals bookkeeping must be byte-for-byte unchanged \
             by an unchanged (rejected-equivalent) rotation, regardless \
             of whether the painted instance has an icon or falls back \
             to text"
        );
    }

    let text_only_icons = BuildableIcons::empty();
    let mut text_only_layout = main_layout();
    text_only_layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("test instance must be placeable");
    run_paint_instances_twice_and_compare(&text_only_icons, &text_only_layout);

    // Real icon case: load an actual texture so the Icon branch of
    // paint_instances's match is genuinely exercised, not merely
    // argued from reading the source.
    let root = tempfile::tempdir().expect("temp icons directory must be created");
    const ONE_PIXEL_PNG_FIXTURE: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0xF7, 0x03, 0x41, 0x43, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::write(root.path().join("pole.png"), ONE_PIXEL_PNG_FIXTURE)
        .expect("PNG fixture must be written");
    let icon_buildable_id = buildable_id("icon_bearing_machine");
    let icon_catalog = {
        use factory_canvas::domain::catalog::{
            BaseDefinition, BaseId, Catalog, CatalogId, CatalogMetadata, CategoryId,
            RegionDefinition, RegionId,
        };
        let region_id = RegionId::new("t053_region").unwrap();
        let base_id = BaseId::new("t053_base").unwrap();
        Catalog::new(
            CatalogMetadata::new(
                CatalogId::new("t053_catalog").unwrap(),
                semver::Version::new(1, 0, 0),
                "T053 Catalog",
            ),
            base_id.clone(),
            vec![RegionDefinition::new(region_id.clone(), "Region")],
            vec![BaseDefinition::new(
                base_id,
                "Base",
                region_id,
                GridSize::new(80, 80).unwrap(),
            )],
            vec![BuildableDefinition::new(
                icon_buildable_id.clone(),
                "Icon Bearing Machine",
                CategoryId::new("t053_category").unwrap(),
                "IB",
                GridSize::new(2, 2).unwrap(),
                vec![],
                Some("pole.png"),
            )],
            vec![],
        )
        .unwrap()
    };
    let load_context = egui::Context::default();
    let icon_icons = BuildableIcons::load(&load_context, &icon_catalog, root.path());
    assert!(
        icon_icons.texture(&icon_buildable_id).is_some(),
        "test setup must actually produce a loaded texture for the Icon branch to run"
    );
    let icon_base_id = icon_catalog.default_base_id().clone();
    let mut icon_layout =
        FactoryLayout::new(icon_catalog, icon_base_id).expect("icon test base must exist");
    icon_layout
        .place(BlockInstance::new(
            EntityId::new(1),
            icon_buildable_id,
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("icon test instance must be placeable");
    run_paint_instances_twice_and_compare(&icon_icons, &icon_layout);
}
