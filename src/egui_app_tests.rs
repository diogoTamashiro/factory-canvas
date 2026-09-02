use crate::egui_canvas::{CanvasInteraction, CanvasViewport};
use eframe::egui::{self, vec2};
use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CatalogValidationError, CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{
    BlockInstance, EntityId, InstanceEditError, PlacementError, ProductionTargetError,
};
use semver::Version;

use super::*;

fn base_id(value: &str) -> BaseId {
    BaseId::new(value).expect("test base IDs must be valid")
}

fn buildable_id(value: &str) -> BuildableId {
    BuildableId::new(value).expect("test buildable IDs must be valid")
}

fn product_id(value: &str) -> ProductId {
    ProductId::new(value).expect("test product IDs must be valid")
}

fn startup_test_catalog(catalog_id: &str, base_id: &str) -> Catalog {
    let catalog_id = CatalogId::new(catalog_id).expect("test catalog ID must be valid");
    let base_id = BaseId::new(base_id).expect("test base ID must be valid");
    let region_id = RegionId::new("test_region").expect("test region ID must be valid");

    Catalog::new(
        CatalogMetadata::new(
            catalog_id,
            Version::parse("1.0.0").expect("test version must be valid"),
            "Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Test Region")],
        vec![BaseDefinition::new(
            base_id,
            "Test Base",
            region_id,
            GridSize::new(20, 20).expect("test base dimensions must be valid"),
        )],
        vec![],
        vec![],
    )
    .expect("test catalog must be valid")
}

fn production_test_app() -> FactoryCanvasApp {
    let region_id = RegionId::new("production_test_region").unwrap();
    let base_id = BaseId::new("production_test_base").unwrap();
    let category_id = CategoryId::new("production_test_category").unwrap();
    let product_a = product_id("test_product_a");
    let product_b = product_id("test_product_b");
    let hidden_product = product_id("test_hidden_product");
    let catalog = Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("production_test_catalog").unwrap(),
            Version::new(1, 0, 0),
            "Production Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Test Region")],
        vec![BaseDefinition::new(
            base_id.clone(),
            "Test Base",
            region_id,
            GridSize::new(20, 20).unwrap(),
        )],
        vec![
            BuildableDefinition::new(
                buildable_id("test_machine"),
                "Test Machine",
                category_id.clone(),
                "TM",
                GridSize::new(2, 2).unwrap(),
                vec![product_b.clone(), product_a.clone()],
            ),
            BuildableDefinition::new(
                buildable_id("test_incapable_block"),
                "Test Incapable Block",
                category_id,
                "TI",
                GridSize::new(2, 2).unwrap(),
                vec![],
            ),
        ],
        vec![
            ProductDefinition::new(product_a, "Product A"),
            ProductDefinition::new(product_b, "Product B"),
            ProductDefinition::new(hidden_product, "Hidden Product"),
        ],
    )
    .unwrap();
    let mut app = FactoryCanvasApp::from_startup_catalog(StartupCatalog {
        catalog,
        warning: None,
    });
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("test_machine"),
            GridPoint::new(1, 1),
            Rotation::Zero,
        ))
        .unwrap();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(2),
            buildable_id("test_incapable_block"),
            GridPoint::new(5, 1),
            Rotation::Zero,
        ))
        .unwrap();
    app.select_instance(EntityId::new(1));
    app
}

fn right_sidebar_frame(
    context: &egui::Context,
    app: &mut FactoryCanvasApp,
    events: Vec<egui::Event>,
) -> (
    Vec<(egui::accesskit::NodeId, egui::accesskit::Node)>,
    Option<SelectedInstanceAction>,
) {
    let mut requested_action = None;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            vec2(420.0, 900.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        requested_action = app.sidebar_ui(ui);
    });
    let nodes = output
        .platform_output
        .accesskit_update
        .take()
        .expect("accessibility tree must be enabled")
        .nodes;
    output.drop_without_applying_deltas();
    (nodes, requested_action)
}

fn primary_click(position: egui::Pos2) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(position),
        egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}

fn accesskit_node_center(node: &egui::accesskit::Node) -> egui::Pos2 {
    let bounds = node.bounds().expect("interactive node must have bounds");
    egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn accesskit_node_text(node: &egui::accesskit::Node) -> Option<&str> {
    node.label().or_else(|| node.value())
}

fn open_product_combo_box(
    context: &egui::Context,
    app: &mut FactoryCanvasApp,
) -> Vec<(egui::accesskit::NodeId, egui::accesskit::Node)> {
    let (nodes, action) = right_sidebar_frame(context, app, vec![]);
    assert_eq!(action, None);
    let combo_box = nodes
        .iter()
        .find(|(_, node)| node.role() == egui::accesskit::Role::ComboBox)
        .map(|(_, node)| node)
        .expect("capable selection must render a ComboBox");

    let (_, action) = right_sidebar_frame(
        context,
        app,
        primary_click(accesskit_node_center(combo_box)),
    );
    assert_eq!(action, None);
    let (nodes, action) = right_sidebar_frame(context, app, vec![]);
    assert_eq!(action, None);
    nodes
}

#[test]
fn production_target_control_offers_only_declared_products_in_catalog_order() {
    let app = production_test_app();

    assert_eq!(
        production_target_control(&app.layout, &app.selected),
        Some(ProductionTargetControl {
            current: None,
            options: vec![
                ProductionTargetOption {
                    product_id: product_id("test_product_b"),
                    display_name: "Product B".to_owned(),
                },
                ProductionTargetOption {
                    product_id: product_id("test_product_a"),
                    display_name: "Product A".to_owned(),
                },
            ],
        })
    );
}

#[test]
fn production_target_combo_box_renders_clear_choice_and_dispatches_clear() {
    let mut app = production_test_app();
    app.layout
        .set_production_target(EntityId::new(1), Some(product_id("test_product_b")))
        .unwrap();
    let context = egui::Context::default();
    context.enable_accesskit();

    let nodes = open_product_combo_box(&context, &mut app);
    let clear_choice = nodes
        .iter()
        .find(|(_, node)| accesskit_node_text(node) == Some("No product"))
        .map(|(_, node)| node)
        .expect("open product ComboBox must render the explicit clear choice");

    let (_, action) = right_sidebar_frame(
        &context,
        &mut app,
        primary_click(accesskit_node_center(clear_choice)),
    );

    assert_eq!(
        action,
        Some(SelectedInstanceAction::SetProductionTarget(None))
    );
}

#[test]
fn production_target_combo_box_dispatches_selected_product() {
    let mut app = production_test_app();
    let context = egui::Context::default();
    context.enable_accesskit();

    let nodes = open_product_combo_box(&context, &mut app);
    let product_choice = nodes
        .iter()
        .find(|(_, node)| accesskit_node_text(node) == Some("Product A"))
        .map(|(_, node)| node)
        .expect("open product ComboBox must render declared products");

    let (_, action) = right_sidebar_frame(
        &context,
        &mut app,
        primary_click(accesskit_node_center(product_choice)),
    );

    assert_eq!(
        action,
        Some(SelectedInstanceAction::SetProductionTarget(Some(
            product_id("test_product_a")
        )))
    );
}

#[test]
fn production_target_control_requires_one_capable_selection() {
    let mut app = production_test_app();

    app.deselect_instance();
    assert_eq!(production_target_control(&app.layout, &app.selected), None);

    app.select_instance(EntityId::new(2));
    assert_eq!(production_target_control(&app.layout, &app.selected), None);

    app.selected.apply(SelectionMode::Add, [EntityId::new(1)]);
    assert_eq!(production_target_control(&app.layout, &app.selected), None);
}

#[test]
fn valid_private_catalog_is_selected_without_warning() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let private = startup_test_catalog("private_catalog", "private_base");

    let choice = choose_startup_catalog(public, Ok(private.clone()));

    assert_eq!(choice.catalog, private);
    assert_eq!(choice.warning, None);
}

#[test]
fn missing_private_catalog_uses_public_without_warning() {
    let public = startup_test_catalog("public_catalog", "public_base");

    let choice = choose_startup_catalog(
        public.clone(),
        Err(CatalogLoadError::ManifestRead(std::io::ErrorKind::NotFound)),
    );

    assert_eq!(choice.catalog, public);
    assert_eq!(choice.warning, None);
}

#[test]
fn invalid_private_catalog_uses_public_with_safe_warning() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let error = CatalogLoadError::InvalidJson {
        module: factory_canvas::catalog_loader::CatalogModule::Buildables,
        kind: factory_canvas::catalog_loader::CatalogJsonErrorKind::Schema,
        line: 8,
        column: 13,
    };

    let choice = choose_startup_catalog(public.clone(), Err(error));

    assert_eq!(choice.catalog, public);
    let warning = choice.warning.expect("invalid private catalog must warn");
    assert!(warning.contains("Private catalog could not be loaded"));
    assert!(warning.contains("using the public catalog"));
    assert!(warning.contains("buildables"));
    assert!(!warning.contains("private-sentinel"));
}

#[test]
fn invalid_private_catalog_warning_redacts_catalog_identifiers() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let private_buildable = buildable_id("private_buildable_sentinel");
    let private_product = product_id("private_product_sentinel");
    let error = CatalogLoadError::InvalidCatalog(CatalogValidationError::MissingProductionTarget {
        buildable_id: private_buildable.clone(),
        product_id: private_product.clone(),
    });

    let choice = choose_startup_catalog(public.clone(), Err(error));

    assert_eq!(choice.catalog, public);
    let warning = choice.warning.expect("invalid private catalog must warn");
    assert!(warning.contains("failed integrity validation"));
    assert!(warning.contains("using the public catalog"));
    assert!(!warning.contains(private_buildable.as_str()));
    assert!(!warning.contains(private_product.as_str()));
}

#[test]
fn invalid_private_catalog_warning_redacts_dimension_value() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let private_value = 4_242_424_242_u64;
    let error = CatalogLoadError::InvalidDimension {
        module: factory_canvas::catalog_loader::CatalogModule::Buildables,
        item_index: 6,
        field: "width",
        value: private_value,
    };

    let choice = choose_startup_catalog(public, Err(error));

    let warning = choice.warning.expect("invalid private catalog must warn");
    assert!(warning.contains("width"));
    assert!(warning.contains("buildables item 7"));
    assert!(!warning.contains(&private_value.to_string()));
}

#[test]
fn invalid_private_catalog_warning_redacts_schema_version() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let private_version = 987_654_321_u64;

    let choice = choose_startup_catalog(
        public,
        Err(CatalogLoadError::UnsupportedSchemaVersion(private_version)),
    );

    let warning = choice.warning.expect("invalid private catalog must warn");
    assert!(warning.contains("schema version is not supported"));
    assert!(!warning.contains(&private_version.to_string()));
}

#[test]
fn app_preserves_startup_catalog_warning() {
    let public = startup_test_catalog("public_catalog", "public_base");
    let choice = StartupCatalog {
        catalog: public.clone(),
        warning: Some("Private catalog failed; using public catalog.".to_owned()),
    };

    let app = FactoryCanvasApp::from_startup_catalog(choice);

    assert_eq!(app.layout.catalog(), &public);
    assert_eq!(
        app.catalog_warning.as_deref(),
        Some("Private catalog failed; using public catalog.")
    );
}

#[test]
fn base_labels_use_confirmed_names_and_derived_dimensions() {
    let app = FactoryCanvasApp::default();
    let labels: Vec<_> = app
        .layout
        .catalog()
        .bases()
        .iter()
        .map(base_option_label)
        .collect();

    assert_eq!(
        labels,
        vec![
            "Main PAC · 80 × 80",
            "Standard Sub-PAC · 30 × 30",
            "Sub-PAC Expansion I · 40 × 40",
            "Sub-PAC Expansion II · 50 × 50",
        ]
    );
}

#[test]
fn block_labels_use_catalog_names_and_footprints() {
    let app = FactoryCanvasApp::default();
    let labels: Vec<_> = app
        .layout
        .catalog()
        .buildables()
        .iter()
        .map(block_option_label)
        .collect();

    assert_eq!(
        labels,
        vec![
            "Xiranite Power Pole · 2 × 2",
            "Refinery Unit · 3 × 3",
            "Crushing Unit · 3 × 3",
        ]
    );
}

#[test]
fn app_starts_with_main_base_layout() {
    let app = FactoryCanvasApp::default();

    assert_eq!(app.layout.base_id().as_str(), "wuling_main");
    assert_eq!(app.layout.bounds(), GridSize::new(80, 80).unwrap());
    assert!(app.layout.is_empty());
    assert_eq!(app.selected_block, None);
}

#[test]
fn app_uses_embedded_public_catalog_during_base_migration() {
    let app = FactoryCanvasApp::default();

    assert_eq!(
        app.layout.catalog().metadata().catalog_id().as_str(),
        "factory_canvas_public"
    );
    assert_eq!(
        app.layout
            .catalog()
            .bases()
            .iter()
            .map(|base| base.id().as_str())
            .collect::<Vec<_>>(),
        vec![
            "wuling_main",
            "wuling_sub_standard",
            "wuling_sub_area_expansion_i",
            "wuling_sub_area_expansion_ii",
        ]
    );
}

#[test]
fn app_starts_with_neutral_canvas_viewport() {
    let app = FactoryCanvasApp::default();

    assert_eq!(app.canvas.viewport, CanvasViewport::default());
}

#[test]
fn home_requests_frame_all_only_without_destructive_modal() {
    assert_eq!(
        canvas_navigation_action_for_frame(true, false),
        Some(CanvasNavigationAction::FrameAll)
    );
    assert_eq!(canvas_navigation_action_for_frame(false, false), None);
    assert_eq!(canvas_navigation_action_for_frame(true, true), None);
}

#[test]
fn frame_all_navigation_action_restores_app_viewport_without_mutating_layout() {
    let mut app = FactoryCanvasApp::default();
    app.canvas.viewport.pan_by(vec2(120.0, -80.0));

    app.apply_canvas_navigation_action(CanvasNavigationAction::FrameAll);

    assert_eq!(app.canvas.viewport, CanvasViewport::default());
    assert!(app.layout.is_empty());
}

#[test]
fn focus_selection_action_requests_canvas_focus_without_mutating_layout() {
    let mut app = FactoryCanvasApp::default();
    let id = EntityId::new(1);
    assert_eq!(
        app.layout.place(BlockInstance::new(
            id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 10),
            Rotation::Zero,
        )),
        Ok(())
    );
    app.selected.apply(SelectionMode::Replace, [id]);
    let before = app.layout.clone();

    app.apply_selected_instance_action(SelectedInstanceAction::FocusSelection);

    assert!(app.canvas.focus_selection_requested);
    assert_eq!(app.layout, before);
}

#[test]
fn selecting_block_keeps_template_ready_for_repeated_placements() {
    let mut app = FactoryCanvasApp::default();

    app.select_block(buildable_id("refinery_unit"));
    assert_eq!(app.selected_block, Some(buildable_id("refinery_unit")));

    app.select_block(buildable_id("crushing_unit"));
    assert_eq!(app.selected_block, Some(buildable_id("crushing_unit")));
}

#[test]
fn placement_preview_is_hidden_while_a_destructive_modal_is_open() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("refinery_unit"));

    assert_eq!(
        app.placement_buildable_for_canvas(),
        Some(&buildable_id("refinery_unit"))
    );

    app.pending_base_change = Some(base_id("wuling_sub_standard"));
    assert_eq!(app.placement_buildable_for_canvas(), None);

    app.pending_base_change = None;
    app.pending_instance_removal = Some(vec![EntityId::new(1)]);
    assert_eq!(app.placement_buildable_for_canvas(), None);
}

#[test]
fn cancelling_base_change_restores_placement_preview_without_losing_selected_block() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("refinery_unit"));
    app.pending_base_change = Some(base_id("wuling_sub_standard"));

    app.cancel_base_change();

    assert_eq!(app.selected_block, Some(buildable_id("refinery_unit")));
    assert_eq!(
        app.placement_buildable_for_canvas(),
        Some(&buildable_id("refinery_unit"))
    );
}

#[test]
fn selecting_existing_instance_clears_placement_tool_without_mutating_layout() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));

    app.select_instance(EntityId::new(1));

    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.selected_block, None);
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceSelected {
            id: EntityId::new(1),
            buildable_id: buildable_id("xiranite_power_pole"),
        }
    );
}

#[test]
fn moving_selected_instance_updates_origin_without_changing_identity() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));

    app.move_selected_by(GridPoint::new(1, 0));

    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(5, 5))
    );
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceMoved {
            id: EntityId::new(1),
            origin: GridPoint::new(5, 5),
        }
    );
}

#[test]
fn rejected_selected_move_at_base_edge_preserves_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(0, 0));
    app.select_instance(EntityId::new(1));

    app.move_selected_by(GridPoint::new(-1, 0));

    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(0, 0))
    );
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds {
            id: EntityId::new(1),
        })
    );
}

#[test]
fn rotating_selected_instance_advances_clockwise_without_changing_id_or_origin() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));

    app.rotate_selected_clockwise();

    let instance = app.layout.instance(EntityId::new(1)).cloned().unwrap();
    assert_eq!(instance.id(), EntityId::new(1));
    assert_eq!(instance.origin(), GridPoint::new(4, 5));
    assert_eq!(instance.rotation(), Rotation::Clockwise90);
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(app.selected.rotation_pivot(), None);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceRotated {
            id: EntityId::new(1),
            rotation: Rotation::Clockwise90,
        }
    );
}

#[test]
fn selected_instance_move_action_uses_editor_transition() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));

    app.apply_selected_instance_action(SelectedInstanceAction::Move(GridPoint::new(0, 1)));

    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(4, 6))
    );
    assert!(app.selected.contains(EntityId::new(1)));
}

#[test]
fn selected_product_action_routes_to_domain_without_changing_editor_identity() {
    let mut app = production_test_app();
    let id = EntityId::new(1);
    let target = product_id("test_product_b");
    let selected_before = app.selected.clone();
    let next_id_before = app.next_entity_id;

    app.apply_selected_instance_action(SelectedInstanceAction::SetProductionTarget(Some(
        target.clone(),
    )));

    assert_eq!(
        app.layout.instance(id).unwrap().production_target(),
        Some(&target)
    );
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(
        app.notice,
        EditorNotice::ProductionTargetChanged {
            id,
            product_id: Some(target),
        }
    );
}

#[test]
fn rejected_product_action_preserves_editor_state() {
    let mut app = production_test_app();
    let missing = product_id("missing_test_product");
    let layout_before = app.layout.clone();
    let selected_before = app.selected.clone();
    let next_id_before = app.next_entity_id;

    app.apply_selected_instance_action(SelectedInstanceAction::SetProductionTarget(Some(
        missing.clone(),
    )));

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(
        app.notice,
        EditorNotice::ProductionTargetRejected(ProductionTargetError::ProductNotFound {
            product_id: missing,
        })
    );
}

#[test]
fn production_target_rejection_uses_error_notice_color() {
    let notice = EditorNotice::ProductionTargetRejected(ProductionTargetError::ProductNotFound {
        product_id: product_id("missing_test_product"),
    });

    assert_eq!(notice_color(&notice), Color32::from_rgb(245, 132, 124));
}

#[test]
fn production_target_choice_clears_configured_product() {
    let current = Some(product_id("test_product_b"));

    assert_eq!(
        production_target_action_for_choice(&current, None),
        Some(SelectedInstanceAction::SetProductionTarget(None))
    );
}

#[test]
fn selected_product_clear_action_routes_to_domain() {
    let mut app = production_test_app();
    app.layout
        .set_production_target(EntityId::new(1), Some(product_id("test_product_b")))
        .unwrap();

    app.apply_selected_instance_action(SelectedInstanceAction::SetProductionTarget(None));

    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .unwrap()
            .production_target(),
        None
    );
    assert_eq!(
        app.notice,
        EditorNotice::ProductionTargetChanged {
            id: EntityId::new(1),
            product_id: None,
        }
    );
}

#[test]
fn instance_semantic_label_includes_configured_product() {
    let mut app = production_test_app();
    app.layout
        .set_production_target(EntityId::new(1), Some(product_id("test_product_b")))
        .unwrap();
    let resolved = app.layout.resolved_instance(EntityId::new(1)).unwrap();

    assert_eq!(
        instance_semantic_label(resolved, app.layout.catalog()),
        "#1 · Test Machine · origin (1, 1) · 2 × 2 · 0° · product Product B"
    );
}

#[test]
fn group_actions_route_to_atomic_domain_operations() {
    let mut app = FactoryCanvasApp::default();
    for (value, origin) in [(1, GridPoint::new(10, 10)), (2, GridPoint::new(14, 10))] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                EntityId::new(value),
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    app.selected
        .apply(SelectionMode::Replace, [EntityId::new(1), EntityId::new(2)]);

    app.move_selected_by(GridPoint::new(1, 0));
    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|item| item.origin()),
        Some(GridPoint::new(11, 10))
    );
    assert_eq!(
        app.layout
            .instance(EntityId::new(2))
            .map(|item| item.origin()),
        Some(GridPoint::new(15, 10))
    );
    assert_eq!(app.notice, EditorNotice::InstancesMoved { count: 2 });

    app.rotate_selected_clockwise();
    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|item| (item.origin(), item.rotation())),
        Some((GridPoint::new(13, 8), Rotation::Clockwise90))
    );
    assert_eq!(
        app.layout
            .instance(EntityId::new(2))
            .map(|item| (item.origin(), item.rotation())),
        Some((GridPoint::new(13, 12), Rotation::Clockwise90))
    );
    assert_eq!(app.selected.rotation_pivot(), Some(GridPoint::new(14, 11)));
    assert_eq!(app.notice, EditorNotice::InstancesRotated { count: 2 });
}

#[test]
fn repeated_group_rotation_reuses_pivot_when_physical_center_shifts() {
    let first_id = EntityId::new(1);
    let second_id = EntityId::new(2);
    let mut app = FactoryCanvasApp::default();
    assert_eq!(
        app.layout.place(BlockInstance::new(
            first_id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 10),
            Rotation::Zero,
        )),
        Ok(())
    );
    assert_eq!(
        app.layout.place(BlockInstance::new(
            second_id,
            buildable_id("refinery_unit"),
            GridPoint::new(14, 10),
            Rotation::Zero,
        )),
        Ok(())
    );
    app.selected
        .apply(SelectionMode::Replace, [first_id, second_id]);

    app.rotate_selected_clockwise();
    app.rotate_selected_clockwise();

    assert_eq!(app.selected.rotation_pivot(), Some(GridPoint::new(13, 11)));
    assert_eq!(
        app.layout.instance(first_id).map(|item| item.origin()),
        Some(GridPoint::new(14, 10))
    );
    assert_eq!(
        app.layout.instance(second_id).map(|item| item.origin()),
        Some(GridPoint::new(9, 9))
    );
}

#[test]
fn successful_group_move_translates_remembered_rotation_pivot() {
    let first_id = EntityId::new(1);
    let second_id = EntityId::new(2);
    let mut app = FactoryCanvasApp::default();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                id,
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    app.selected
        .apply(SelectionMode::Replace, [first_id, second_id]);
    app.rotate_selected_clockwise();

    app.move_selected_by(GridPoint::new(2, 3));

    assert_eq!(app.selected.rotation_pivot(), Some(GridPoint::new(15, 14)));
    assert_eq!(
        app.layout.instance(first_id).map(|item| item.origin()),
        Some(GridPoint::new(14, 11))
    );
    assert_eq!(
        app.layout.instance(second_id).map(|item| item.origin()),
        Some(GridPoint::new(14, 15))
    );
}

#[test]
fn rejected_group_move_preserves_remembered_rotation_pivot() {
    let first_id = EntityId::new(1);
    let second_id = EntityId::new(2);
    let mut app = FactoryCanvasApp::default();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                id,
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    app.selected
        .apply(SelectionMode::Replace, [first_id, second_id]);
    app.rotate_selected_clockwise();
    app.move_selected_by(GridPoint::new(-12, 0));
    let layout_before = app.layout.clone();
    let pivot_before = app.selected.rotation_pivot();

    app.move_selected_by(GridPoint::new(-1, 0));

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected.rotation_pivot(), pivot_before);
    assert_eq!(
        app.notice,
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds { id: first_id })
    );
}

#[test]
fn rejected_group_rotation_preserves_layout_selection_allocator_and_pivot() {
    let first_id = EntityId::new(1);
    let second_id = EntityId::new(2);
    let mut app = FactoryCanvasApp::default();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                id,
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    app.next_entity_id = Some(3);
    app.selected
        .apply(SelectionMode::Replace, [first_id, second_id]);
    app.rotate_selected_clockwise();
    app.move_selected_by(GridPoint::new(-12, 0));
    let layout_before = app.layout.clone();
    let selection_before = app.selected.clone();

    app.rotate_selected_clockwise();

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selection_before);
    assert_eq!(app.next_entity_id, Some(3));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds { id: second_id })
    );
}

#[test]
fn group_removal_request_is_frozen_cancelable_and_confirmed_once() {
    let mut app = FactoryCanvasApp::default();
    for (value, origin) in [
        (1, GridPoint::new(0, 0)),
        (2, GridPoint::new(2, 0)),
        (3, GridPoint::new(4, 0)),
    ] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                EntityId::new(value),
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    app.selected
        .apply(SelectionMode::Replace, [EntityId::new(1), EntityId::new(2)]);

    app.request_selected_instance_removal();
    assert_eq!(
        app.pending_instance_removal,
        Some(vec![EntityId::new(1), EntityId::new(2)])
    );
    app.selected
        .apply(SelectionMode::Replace, [EntityId::new(3)]);
    app.cancel_instance_removal();
    assert_eq!(app.layout.len(), 3);
    assert_eq!(
        app.selected.iter().collect::<Vec<_>>(),
        vec![EntityId::new(3)]
    );

    app.selected
        .apply(SelectionMode::Replace, [EntityId::new(1), EntityId::new(2)]);
    app.request_selected_instance_removal();
    app.confirm_instance_removal();
    assert!(app.layout.instance(EntityId::new(1)).is_none());
    assert!(app.layout.instance(EntityId::new(2)).is_none());
    assert!(app.layout.instance(EntityId::new(3)).is_some());
    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert_eq!(app.notice, EditorNotice::InstancesRemoved { count: 2 });
}

#[test]
fn stale_selection_is_reconciled_before_group_edit() {
    let mut app = FactoryCanvasApp::default();
    app.selected.insert(EntityId::new(999));

    app.move_selected_by(GridPoint::new(1, 0));

    assert!(app.selected.is_empty());
    assert_eq!(app.notice, EditorNotice::SelectBlock);
    assert!(app.layout.is_empty());
}

#[test]
fn sidebar_action_has_priority_over_keyboard_action_within_frame() {
    let sidebar_action = Some(SelectedInstanceAction::Move(GridPoint::new(1, 0)));
    let keyboard_action = Some(SelectedInstanceAction::RotateClockwise);

    assert_eq!(
        selected_instance_action_for_frame(sidebar_action.clone(), keyboard_action),
        sidebar_action
    );
}

#[test]
fn cancelling_selected_instance_removal_preserves_complete_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    let notice_before_request = app.notice.clone();

    app.request_selected_instance_removal();

    assert_eq!(app.pending_instance_removal, Some(vec![EntityId::new(1)]));
    assert_eq!(app.layout.len(), 1);
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(app.next_entity_id, Some(2));

    app.cancel_instance_removal();

    assert_eq!(app.pending_instance_removal, None);
    assert_eq!(app.layout.len(), 1);
    assert!(app.selected.contains(EntityId::new(1)));
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.notice, notice_before_request);
}

#[test]
fn confirming_selected_instance_removal_clears_selection_without_reusing_ids() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    app.request_selected_instance_removal();

    app.confirm_instance_removal();

    assert!(app.layout.is_empty());
    assert_eq!(app.selected_block, None);
    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::InstanceRemoved {
            id: EntityId::new(1),
            buildable_id: buildable_id("xiranite_power_pole"),
        }
    );

    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(0, 0));
    assert!(app.layout.instance(EntityId::new(2)).is_some());
}

#[test]
fn confirming_stale_removal_request_clears_stale_selection_without_mutating_layout() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let stale_id = EntityId::new(99);
    app.selected = SelectedSet::new();
    app.selected.insert(stale_id);
    app.pending_instance_removal = Some(vec![stale_id]);
    app.notice = EditorNotice::InstanceSelected {
        id: stale_id,
        buildable_id: buildable_id("xiranite_power_pole"),
    };

    app.confirm_instance_removal();

    assert_eq!(app.layout.len(), 1);
    assert!(app.layout.instance(EntityId::new(1)).is_some());
    assert_eq!(app.selected_block, None);
    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn canvas_interactions_select_deselect_and_place_through_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(0, 0));

    app.apply_canvas_interaction(CanvasInteraction::Select {
        id: EntityId::new(1),
        mode: SelectionMode::Replace,
    });
    assert_eq!(app.selected_block, None);
    assert!(app.selected.contains(EntityId::new(1)));

    app.apply_canvas_interaction(CanvasInteraction::Deselect);
    assert!(app.selected.is_empty());

    app.select_block(buildable_id("xiranite_power_pole"));
    app.apply_canvas_interaction(CanvasInteraction::Place(GridPoint::new(2, 0)));
    assert_eq!(app.layout.len(), 2);
    assert!(app.layout.instance(EntityId::new(2)).is_some());
}

#[test]
fn canvas_selection_modes_and_marquee_update_stable_set_and_notices() {
    let mut app = FactoryCanvasApp::default();
    for (value, origin) in [
        (1, GridPoint::new(0, 0)),
        (2, GridPoint::new(3, 0)),
        (3, GridPoint::new(6, 0)),
    ] {
        assert_eq!(
            app.layout.place(BlockInstance::new(
                EntityId::new(value),
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }

    app.apply_canvas_interaction(CanvasInteraction::Select {
        id: EntityId::new(1),
        mode: SelectionMode::Replace,
    });
    app.apply_canvas_interaction(CanvasInteraction::Select {
        id: EntityId::new(2),
        mode: SelectionMode::Add,
    });
    assert_eq!(
        app.selected.iter().collect::<Vec<_>>(),
        vec![EntityId::new(1), EntityId::new(2)]
    );
    assert_eq!(app.notice, EditorNotice::InstancesSelected { count: 2 });

    app.apply_canvas_interaction(CanvasInteraction::Select {
        id: EntityId::new(1),
        mode: SelectionMode::Toggle,
    });
    assert_eq!(
        app.selected.iter().collect::<Vec<_>>(),
        vec![EntityId::new(2)]
    );
    assert_eq!(
        app.notice,
        EditorNotice::InstanceSelected {
            id: EntityId::new(2),
            buildable_id: buildable_id("xiranite_power_pole"),
        }
    );

    app.apply_canvas_interaction(CanvasInteraction::Marquee {
        ids: vec![EntityId::new(1), EntityId::new(3)],
        mode: SelectionMode::Add,
    });
    assert_eq!(
        app.selected.iter().collect::<Vec<_>>(),
        vec![EntityId::new(1), EntityId::new(2), EntityId::new(3)]
    );
    assert_eq!(app.notice, EditorNotice::InstancesSelected { count: 3 });

    app.apply_canvas_interaction(CanvasInteraction::Marquee {
        ids: vec![EntityId::new(2), EntityId::new(3), EntityId::new(3)],
        mode: SelectionMode::Toggle,
    });
    assert_eq!(
        app.selected.iter().collect::<Vec<_>>(),
        vec![EntityId::new(1)]
    );

    app.apply_canvas_interaction(CanvasInteraction::Marquee {
        ids: Vec::new(),
        mode: SelectionMode::Replace,
    });
    assert!(app.selected.is_empty());
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn selection_count_labels_are_semantic_and_pluralized() {
    assert_eq!(selection_count_label(0), "No blocks selected");
    assert_eq!(selection_count_label(1), "1 block selected");
    assert_eq!(selection_count_label(3), "3 blocks selected");
}

#[test]
fn layout_count_labels_are_semantic_and_pluralized() {
    assert_eq!(layout_count_label(0), "No blocks placed");
    assert_eq!(layout_count_label(1), "1 block placed");
    assert_eq!(layout_count_label(2), "2 blocks placed");
}

#[test]
fn successful_placements_use_monotonic_ids_and_allow_edge_contact() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));

    app.place_selected_at(GridPoint::new(0, 0));
    app.place_selected_at(GridPoint::new(2, 0));

    assert_eq!(app.layout.len(), 2);
    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(0, 0))
    );
    assert_eq!(
        app.layout
            .instance(EntityId::new(2))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(2, 0))
    );
    assert_eq!(app.next_entity_id, Some(3));
    assert_eq!(
        app.selected_block,
        Some(buildable_id("xiranite_power_pole"))
    );
}

#[test]
fn rejected_placements_preserve_layout_and_next_id() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(0, 0));

    app.place_selected_at(GridPoint::new(0, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::PlacementRejected(PlacementError::Collision {
            id: EntityId::new(2),
            conflicting_id: EntityId::new(1),
        })
    );

    app.place_selected_at(GridPoint::new(79, 79));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds {
            id: EntityId::new(2),
        })
    );
}

#[test]
fn placement_without_selection_does_not_change_layout_or_id() {
    let mut app = FactoryCanvasApp::default();

    app.place_selected_at(GridPoint::new(0, 0));

    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn entity_id_exhaustion_never_wraps_or_mutates_layout() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.next_entity_id = Some(u64::MAX);

    app.place_selected_at(GridPoint::new(0, 0));
    assert!(app.layout.instance(EntityId::new(u64::MAX)).is_some());
    assert_eq!(app.next_entity_id, None);

    app.place_selected_at(GridPoint::new(2, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, None);
    assert_eq!(app.notice, EditorNotice::EntityIdsExhausted);
}

#[test]
fn missing_buildable_notice_redacts_catalog_identifier() {
    let private_buildable = buildable_id("private_buildable_notice_sentinel");
    let catalog = load_embedded_public_catalog().expect("public catalog must load");
    let notice = EditorNotice::PlacementRejected(PlacementError::BuildableNotFound {
        id: EntityId::new(7),
        buildable_id: private_buildable.clone(),
    });

    let text = notice_text(&notice, "Standard Sub-PAC", &catalog);

    assert_eq!(
        text,
        "The selected construction is not available in this catalog."
    );
    assert!(!text.contains(private_buildable.as_str()));
}

#[test]
fn notice_text_describes_editor_state_and_domain_errors() {
    let id = EntityId::new(4);
    let conflicting_id = EntityId::new(2);
    let catalog = load_embedded_public_catalog().expect("public catalog must load");
    let notice_text = |notice| super::notice_text(&notice, "Standard Sub-PAC", &catalog);

    assert_eq!(
        notice_text(EditorNotice::SelectBlock),
        "Select a block to get started."
    );
    assert_eq!(
        notice_text(EditorNotice::ReadyToPlace {
            buildable_id: buildable_id("refinery_unit"),
        }),
        "Selected block: Refinery Unit. Click the grid to place it."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceSelected {
            id,
            buildable_id: buildable_id("refinery_unit"),
        }),
        "Block #4 selected: Refinery Unit."
    );
    assert_eq!(
        notice_text(EditorNotice::InstancesSelected { count: 3 }),
        "3 blocks selected."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceRemoved {
            id,
            buildable_id: buildable_id("refinery_unit"),
        }),
        "Block #4 removed: Refinery Unit."
    );
    assert_eq!(
        notice_text(EditorNotice::InstancesRemoved { count: 3 }),
        "3 blocks removed."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceMoved {
            id,
            origin: GridPoint::new(6, 7),
        }),
        "Block #4 moved to (6, 7)."
    );
    assert_eq!(
        notice_text(EditorNotice::InstancesMoved { count: 3 }),
        "3 blocks moved."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceRotated {
            id,
            rotation: Rotation::Clockwise90,
        }),
        "Block #4 rotated to 90°."
    );
    assert_eq!(
        notice_text(EditorNotice::InstancesRotated { count: 3 }),
        "3 blocks rotated 90°."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceEditRejected(
            InstanceEditError::EntityNotFound { id }
        )),
        "Block #4 no longer exists."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceEditRejected(
            InstanceEditError::OutOfBounds { id }
        )),
        "The block does not fit at this position."
    );
    assert_eq!(
        notice_text(EditorNotice::InstanceEditRejected(
            InstanceEditError::Collision { id, conflicting_id }
        )),
        "Position occupied by block #2."
    );
    let private_target = ProductId::new("private_target").unwrap();
    assert_eq!(
        notice_text(EditorNotice::ProductionTargetChanged {
            id,
            product_id: Some(private_target.clone()),
        }),
        "Block #4 product updated."
    );
    assert_eq!(
        notice_text(EditorNotice::ProductionTargetChanged {
            id,
            product_id: None,
        }),
        "Block #4 product cleared."
    );
    assert_eq!(
        notice_text(EditorNotice::ProductionTargetRejected(
            ProductionTargetError::ProductNotFound {
                product_id: private_target.clone(),
            }
        )),
        "The selected product is not available in this catalog."
    );
    assert_eq!(
        notice_text(EditorNotice::ProductionTargetRejected(
            ProductionTargetError::UnsupportedProduct {
                buildable_id: buildable_id("private_machine"),
                product_id: private_target,
            }
        )),
        "The selected product is not supported by this construction."
    );
    assert_eq!(
        notice_text(EditorNotice::Placed {
            id,
            buildable_id: buildable_id("refinery_unit"),
            origin: GridPoint::new(6, 7),
        }),
        "Block #4 placed at (6, 7): Refinery Unit."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(
            PlacementError::DuplicateEntityId { id }
        )),
        "Internal ID #4 is already in use."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(
            PlacementError::OutOfBounds { id }
        )),
        "The block does not fit at this position."
    );
    let private_product = ProductId::new("private_product").unwrap();
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(
            PlacementError::ProductNotFound {
                id,
                product_id: private_product.clone(),
            }
        )),
        "The configured product is not available in this catalog."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(
            PlacementError::UnsupportedProduct {
                id,
                buildable_id: buildable_id("private_machine"),
                product_id: private_product,
            }
        )),
        "The configured product is not supported by this construction."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(PlacementError::Collision {
            id,
            conflicting_id,
        })),
        "Position occupied by block #2."
    );
    assert_eq!(
        notice_text(EditorNotice::EntityIdsExhausted),
        "No IDs are available for new blocks."
    );
    assert_eq!(
        notice_text(EditorNotice::BaseChanged),
        "Base changed to Standard Sub-PAC."
    );
}

#[test]
fn instance_labels_expose_painted_blocks_semantically() {
    let mut layout = FactoryCanvasApp::default().layout;
    let instance = BlockInstance::new(
        EntityId::new(7),
        buildable_id("refinery_unit"),
        GridPoint::new(3, 4),
        Rotation::Zero,
    );
    layout
        .place(instance)
        .expect("first test instance should fit");
    let resolved = layout
        .resolved_instance(EntityId::new(7))
        .expect("first test instance should resolve");

    assert_eq!(
        instance_semantic_label(resolved, layout.catalog()),
        "#7 · Refinery Unit · origin (3, 4) · 3 × 3 · 0° · no product"
    );

    let rotated = BlockInstance::new(
        EntityId::new(8),
        buildable_id("refinery_unit"),
        GridPoint::new(6, 2),
        Rotation::Clockwise90,
    );
    layout
        .place(rotated)
        .expect("second test instance should fit");
    let resolved = layout
        .resolved_instance(EntityId::new(8))
        .expect("second test instance should resolve");
    assert_eq!(
        instance_semantic_label(resolved, layout.catalog()),
        "#8 · Refinery Unit · origin (6, 2) · 3 × 3 · 90° · no product"
    );
}

#[test]
fn requesting_base_change_replaces_empty_layout_immediately() {
    let mut app = FactoryCanvasApp::default();
    let base_ids = [
        "wuling_sub_standard",
        "wuling_sub_area_expansion_i",
        "wuling_sub_area_expansion_ii",
        "wuling_main",
    ]
    .map(base_id);

    for base_id in base_ids {
        let expected_bounds = app
            .layout
            .catalog()
            .base(&base_id)
            .expect("test base must exist")
            .bounds();
        app.request_base_change(base_id.clone());

        assert_eq!(app.layout.base_id(), &base_id);
        assert_eq!(app.layout.bounds(), expected_bounds);
        assert!(app.layout.is_empty());
        assert_eq!(app.pending_base_change, None);
    }
}

#[test]
fn cancelling_nonempty_base_change_preserves_complete_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let notice_before_request = app.notice.clone();
    let target = base_id("wuling_sub_standard");

    app.request_base_change(target.clone());

    assert_eq!(app.pending_base_change, Some(target));
    assert_eq!(app.layout.base_id().as_str(), "wuling_main");
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));

    app.cancel_base_change();

    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.base_id().as_str(), "wuling_main");
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.notice, notice_before_request);
}

#[test]
fn confirming_nonempty_base_change_clears_layout_and_resets_ids() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let target = base_id("wuling_sub_standard");
    app.request_base_change(target.clone());

    app.confirm_base_change();

    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.base_id(), &target);
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(
        app.selected_block,
        Some(buildable_id("xiranite_power_pole"))
    );
    assert_eq!(app.notice, EditorNotice::BaseChanged);
}
