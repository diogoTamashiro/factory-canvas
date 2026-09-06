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

#[derive(Default)]
struct StubFactoryFileDialogs {
    open_paths: std::collections::VecDeque<Option<std::path::PathBuf>>,
    save_paths: std::collections::VecDeque<Option<std::path::PathBuf>>,
    open_calls: usize,
    save_calls: usize,
}

impl FactoryFileDialogs for StubFactoryFileDialogs {
    fn pick_open_path(&mut self, _current_path: Option<&Path>) -> Option<std::path::PathBuf> {
        self.open_calls += 1;
        self.open_paths.pop_front().flatten()
    }

    fn pick_save_path(&mut self, _current_path: Option<&Path>) -> Option<std::path::PathBuf> {
        self.save_calls += 1;
        self.save_paths.pop_front().flatten()
    }
}

fn key_press(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn document_shortcut_frame(
    context: &egui::Context,
    event: egui::Event,
    blocked: bool,
) -> Option<DocumentCommand> {
    let mut action = None;
    let output = context.run_ui(
        egui::RawInput {
            events: vec![event],
            ..Default::default()
        },
        |ui| action = document_shortcut_for_frame(ui.ctx(), blocked),
    );
    output.drop_without_applying_deltas();
    action
}

fn dispatch_document_shortcut_frame(
    app: &mut FactoryCanvasApp,
    dialogs: &mut impl FactoryFileDialogs,
    event: egui::Event,
) {
    let context = egui::Context::default();
    let output = context.run_ui(
        egui::RawInput {
            events: vec![event],
            ..Default::default()
        },
        |ui| {
            app.dispatch_document_command_for_frame(
                ui.ctx(),
                None,
                dialogs,
                time::OffsetDateTime::UNIX_EPOCH,
            );
        },
    );
    output.drop_without_applying_deltas();
}

fn close_request_frame(app: &mut FactoryCanvasApp) -> Vec<egui::ViewportCommand> {
    let context = egui::Context::default();
    let mut input = egui::RawInput::default();
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    let output = context.run_ui(input, |ui| app.handle_close_request(ui.ctx()));
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    output.drop_without_applying_deltas();
    commands
}

fn confirm_unsaved_frame(
    app: &mut FactoryCanvasApp,
    now: time::OffsetDateTime,
) -> Vec<egui::ViewportCommand> {
    let context = egui::Context::default();
    let output = context.run_ui(Default::default(), |ui| {
        app.confirm_pending_unsaved_action(ui.ctx(), now);
    });
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    output.drop_without_applying_deltas();
    commands
}

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

fn write_factory_with_mismatched_catalog(path: &Path, app: &FactoryCanvasApp) {
    let metadata = DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH);
    let bytes = factory_canvas::persistence::factory_document::encode_factory_document(
        &app.layout,
        app.next_entity_id,
        metadata.metadata(),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["catalog_id"] = serde_json::Value::String("different_catalog".to_owned());
    value["catalog_data_version"] = serde_json::Value::String("9.9.9".to_owned());
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
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

fn header_frame(
    context: &egui::Context,
    app: &FactoryCanvasApp,
    events: Vec<egui::Event>,
) -> (
    Vec<(egui::accesskit::NodeId, egui::accesskit::Node)>,
    Option<DocumentCommand>,
) {
    let mut requested_command = None;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            vec2(1_200.0, 80.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        requested_command = app.header_ui(ui);
    });
    let nodes = output
        .platform_output
        .accesskit_update
        .take()
        .expect("accessibility tree must be enabled")
        .nodes;
    output.drop_without_applying_deltas();
    (nodes, requested_command)
}

fn unsaved_modal_frame(
    context: &egui::Context,
    app: &mut FactoryCanvasApp,
    events: Vec<egui::Event>,
) -> Vec<(egui::accesskit::NodeId, egui::accesskit::Node)> {
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            vec2(720.0, 480.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| app.unsaved_changes_modal(ui.ctx()));
    let nodes = output
        .platform_output
        .accesskit_update
        .take()
        .expect("accessibility tree must be enabled")
        .nodes;
    output.drop_without_applying_deltas();
    nodes
}

fn full_app_frame(
    context: &egui::Context,
    app: &mut FactoryCanvasApp,
    dialogs: &mut impl FactoryFileDialogs,
    events: Vec<egui::Event>,
    close_requested: bool,
) -> (
    Vec<(egui::accesskit::NodeId, egui::accesskit::Node)>,
    Vec<egui::ViewportCommand>,
) {
    context.enable_accesskit();
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(1_400.0, 900.0),
        )),
        events,
        ..Default::default()
    };
    if close_requested {
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .events
            .push(egui::ViewportEvent::Close);
    }

    let mut output = context.run_ui(input, |ui| app.ui_with_dialogs(ui, dialogs));
    let nodes = output
        .platform_output
        .accesskit_update
        .take()
        .map(|update| update.nodes)
        .unwrap_or_default();
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    output.drop_without_applying_deltas();
    (nodes, commands)
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

    app.pending_instance_removal = None;
    app.pending_unsaved_action = Some(PendingUnsavedAction::New);
    assert_eq!(app.placement_buildable_for_canvas(), None);
}

#[test]
fn cancelling_base_change_restores_placement_preview_without_losing_selected_block() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    app.select_block(buildable_id("refinery_unit"));
    app.request_base_change(base_id("wuling_sub_standard"));
    assert!(!app.session.is_dirty());
    assert!(app.pending_base_change.is_some());

    app.cancel_base_change();

    assert!(!app.session.is_dirty());
    assert_eq!(app.selected_block, Some(buildable_id("refinery_unit")));
    assert_eq!(
        app.placement_buildable_for_canvas(),
        Some(&buildable_id("refinery_unit"))
    );
}

#[test]
fn cancelling_base_change_preserves_dirty_session() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(0, 0));
    assert!(app.session.is_dirty());

    app.request_base_change(base_id("wuling_sub_standard"));
    assert!(app.pending_base_change.is_some());
    app.cancel_base_change();

    assert!(app.session.is_dirty());
    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.len(), 1);
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
    assert!(!app.session.is_dirty());

    app.apply_selected_instance_action(SelectedInstanceAction::SetProductionTarget(Some(
        missing.clone(),
    )));

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert!(!app.session.is_dirty());
    assert_eq!(
        app.notice,
        EditorNotice::ProductionTargetRejected(ProductionTargetError::ProductNotFound {
            product_id: missing,
        })
    );
}

#[test]
fn rejected_product_action_preserves_dirty_session() {
    let mut app = production_test_app();
    app.move_selected_by(GridPoint::new(1, 0));
    assert!(app.session.is_dirty());

    app.set_selected_production_target(Some(product_id("missing_test_product")));

    assert!(app.session.is_dirty());
    assert!(matches!(
        app.notice,
        EditorNotice::ProductionTargetRejected(ProductionTargetError::ProductNotFound { .. })
    ));
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
    app.session = DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH);
    let layout_before = app.layout.clone();
    let selection_before = app.selected.clone();
    assert!(!app.session.is_dirty());

    app.rotate_selected_clockwise();

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selection_before);
    assert_eq!(app.next_entity_id, Some(3));
    assert!(!app.session.is_dirty());
    assert_eq!(
        app.notice,
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds { id: second_id })
    );

    app.session.mark_dirty();
    app.rotate_selected_clockwise();
    assert!(app.session.is_dirty());
    assert_eq!(app.layout, layout_before);
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
fn document_shortcuts_route_specific_commands_and_respect_modal_blocking() {
    let ctrl = egui::Modifiers::CTRL;
    let ctrl_shift = egui::Modifiers {
        ctrl: true,
        shift: true,
        ..Default::default()
    };

    assert_eq!(
        document_shortcut_frame(
            &egui::Context::default(),
            key_press(egui::Key::O, ctrl),
            false,
        ),
        Some(DocumentCommand::Open)
    );
    assert_eq!(
        document_shortcut_frame(
            &egui::Context::default(),
            key_press(egui::Key::S, ctrl),
            false,
        ),
        Some(DocumentCommand::Save)
    );
    assert_eq!(
        document_shortcut_frame(
            &egui::Context::default(),
            key_press(egui::Key::S, ctrl_shift),
            false,
        ),
        Some(DocumentCommand::SaveAs)
    );
    assert_eq!(
        document_shortcut_frame(
            &egui::Context::default(),
            key_press(egui::Key::O, ctrl),
            true,
        ),
        None
    );
}

#[test]
fn document_shortcuts_are_suppressed_while_text_edit_has_focus() {
    let context = egui::Context::default();
    let mut text = String::new();
    let first_output = context.run_ui(Default::default(), |ui| {
        ui.text_edit_singleline(&mut text).request_focus();
    });
    first_output.drop_without_applying_deltas();

    let mut action = None;
    let output = context.run_ui(
        egui::RawInput {
            events: vec![key_press(egui::Key::S, egui::Modifiers::CTRL)],
            ..Default::default()
        },
        |ui| {
            ui.text_edit_singleline(&mut text);
            assert!(ui.ctx().text_edit_focused());
            action = document_shortcut_for_frame(ui.ctx(), false);
        },
    );
    output.drop_without_applying_deltas();

    assert_eq!(action, None);
}

#[test]
fn ctrl_s_dispatches_through_frame_and_saves_document() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("shortcut-save.factory.json");
    let mut dialogs = StubFactoryFileDialogs {
        save_paths: std::collections::VecDeque::from([Some(path.clone())]),
        ..Default::default()
    };
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    assert!(app.session.is_dirty());

    dispatch_document_shortcut_frame(
        &mut app,
        &mut dialogs,
        key_press(egui::Key::S, egui::Modifiers::CTRL),
    );

    assert_eq!(dialogs.save_calls, 1);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());
    assert!(path.is_file());
}

#[test]
fn full_app_frame_routes_ctrl_s_through_injected_dialogs() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("full-frame.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    assert!(app.session.is_dirty());

    let mut dialogs = StubFactoryFileDialogs {
        save_paths: std::collections::VecDeque::from([Some(path.clone())]),
        ..Default::default()
    };
    let context = egui::Context::default();
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1_440.0, 900.0),
            )),
            events: vec![key_press(egui::Key::S, egui::Modifiers::CTRL)],
            ..Default::default()
        },
        |ui| app.ui_with_dialogs(ui, &mut dialogs),
    );
    output.drop_without_applying_deltas();

    assert_eq!(dialogs.save_calls, 1);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());
    assert!(path.is_file());
}

#[test]
fn full_app_frame_intercepts_dirty_close_and_renders_confirmation() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let mut dialogs = StubFactoryFileDialogs::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1400.0, 900.0),
        )),
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);

    let output = context.run_ui(input, |ui| app.ui_with_dialogs(ui, &mut dialogs));
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    let nodes = output
        .platform_output
        .accesskit_update
        .as_ref()
        .map(|update| update.nodes.clone())
        .unwrap_or_default();
    let texts: Vec<_> = nodes
        .iter()
        .filter_map(|(_, node)| accesskit_node_text(node))
        .collect();
    output.drop_without_applying_deltas();

    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::Close)
    ));
    assert!(texts.contains(&"Unsaved changes"));
    assert!(texts.contains(&"Discard and close"));
    assert_eq!(dialogs.open_calls, 0);
    assert_eq!(dialogs.save_calls, 0);
}

#[test]
fn full_app_frame_cancels_close_when_removal_confirmation_makes_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    let id = EntityId::new(1);
    app.layout
        .place(BlockInstance::new(
            id,
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.selected.apply(SelectionMode::Replace, [id]);
    app.request_selected_instance_removal();
    assert!(!app.session.is_dirty());

    let mut dialogs = StubFactoryFileDialogs::default();
    let context = egui::Context::default();
    let (nodes, _) = full_app_frame(&context, &mut app, &mut dialogs, vec![], false);
    let remove_button_node_id = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button
                && accesskit_node_text(node) == Some("Remove")
        })
        .map(|(node_id, _)| *node_id)
        .expect("removal modal must render its confirmation button");

    let (_, commands) = full_app_frame(
        &context,
        &mut app,
        &mut dialogs,
        vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: remove_button_node_id,
                data: None,
            },
        )],
        true,
    );

    assert!(app.layout.is_empty());
    assert!(app.session.is_dirty());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::Close)
    ));
}

#[test]
fn full_app_frame_cancels_close_when_base_confirmation_makes_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    let target_base = base_id("wuling_sub_standard");
    app.request_base_change(target_base.clone());
    assert!(!app.session.is_dirty());

    let mut dialogs = StubFactoryFileDialogs::default();
    let context = egui::Context::default();
    let (nodes, _) = full_app_frame(&context, &mut app, &mut dialogs, vec![], false);
    let confirm_button_node_id = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button
                && accesskit_node_text(node) == Some("Change and clear")
        })
        .map(|(node_id, _)| *node_id)
        .expect("base-change modal must render its confirmation button");

    let (_, commands) = full_app_frame(
        &context,
        &mut app,
        &mut dialogs,
        vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: confirm_button_node_id,
                data: None,
            },
        )],
        true,
    );

    assert_eq!(app.layout.base_id(), &target_base);
    assert!(app.layout.is_empty());
    assert!(app.session.is_dirty());
    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::Close)
    ));
}

#[test]
fn header_command_has_priority_over_shortcut_in_shared_dispatcher() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("associated.factory.json");
    let mut app = FactoryCanvasApp::default();
    let default_base = app.layout.base_id().clone();
    app.replace_base(base_id("wuling_sub_standard"));
    app.save_document_to(&path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    assert_eq!(app.layout.base_id(), &base_id("wuling_sub_standard"));
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());

    let mut dialogs = StubFactoryFileDialogs::default();
    let context = egui::Context::default();
    let output = context.run_ui(
        egui::RawInput {
            events: vec![key_press(egui::Key::S, egui::Modifiers::CTRL)],
            ..Default::default()
        },
        |ui| {
            app.dispatch_document_command_for_frame(
                ui.ctx(),
                Some(DocumentCommand::New),
                &mut dialogs,
                time::OffsetDateTime::UNIX_EPOCH,
            );
        },
    );
    output.drop_without_applying_deltas();

    assert_eq!(app.layout.base_id(), &default_base);
    assert!(app.layout.is_empty());
    assert_eq!(app.session.path(), None);
    assert!(!app.session.is_dirty());
    assert_eq!(dialogs.save_calls, 0);
}

#[test]
fn header_exposes_document_commands_and_dirty_indicator_semantically() {
    let clean = FactoryCanvasApp::default();
    let clean_context = egui::Context::default();
    clean_context.enable_accesskit();
    let (clean_nodes, command) = header_frame(&clean_context, &clean, vec![]);
    assert_eq!(command, None);
    for label in ["New", "Open", "Save", "Save As"] {
        assert!(clean_nodes.iter().any(|(_, node)| {
            node.role() == egui::accesskit::Role::Button && accesskit_node_text(node) == Some(label)
        }));
    }
    assert!(!clean_nodes
        .iter()
        .any(|(_, node)| accesskit_node_text(node) == Some("* Unsaved")));

    let new_button = clean_nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button && accesskit_node_text(node) == Some("New")
        })
        .map(|(_, node)| node)
        .unwrap();
    let (_, command) = header_frame(
        &clean_context,
        &clean,
        primary_click(accesskit_node_center(new_button)),
    );
    assert_eq!(command, Some(DocumentCommand::New));

    let mut dirty = FactoryCanvasApp::default();
    dirty.session.mark_dirty();
    let dirty_context = egui::Context::default();
    dirty_context.enable_accesskit();
    let (dirty_nodes, _) = header_frame(&dirty_context, &dirty, vec![]);
    assert!(dirty_nodes
        .iter()
        .any(|(_, node)| accesskit_node_text(node) == Some("* Unsaved")));
}

#[test]
fn header_document_commands_are_semantically_disabled_during_modal() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(app.pending_unsaved_action.is_some());

    let context = egui::Context::default();
    context.enable_accesskit();
    let (nodes, command) = header_frame(&context, &app, vec![]);
    assert_eq!(command, None);
    let command_buttons: Vec<_> = nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == egui::accesskit::Role::Button
                && matches!(
                    accesskit_node_text(node),
                    Some("New" | "Open" | "Save" | "Save As")
                )
        })
        .map(|(_, node)| node)
        .collect();
    assert_eq!(command_buttons.len(), 4);
    assert!(command_buttons.iter().all(|node| node.is_disabled()));
}

#[test]
fn cancelled_file_dialogs_preserve_complete_document_and_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    app.canvas.viewport.pan_by(vec2(31.0, -17.0));
    let layout_before = app.layout.clone();
    let metadata_before = app.session.metadata().clone();
    let path_before = app.session.path().map(Path::to_path_buf);
    let compatibility_before = app.session.compatibility();
    let selected_before = app.selected.clone();
    let selected_block_before = app.selected_block.clone();
    let next_id_before = app.next_entity_id;
    let notice_before = app.notice.clone();
    let viewport_before = app.canvas.viewport;
    assert!(app.session.is_dirty());
    assert_ne!(viewport_before, CanvasViewport::default());

    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    app.execute_document_command(
        DocumentCommand::SaveAs,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(dialogs.open_calls, 1);
    assert_eq!(dialogs.save_calls, 1);
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert_eq!(app.session.path(), path_before.as_deref());
    assert_eq!(app.session.compatibility(), compatibility_before);
    assert!(app.session.is_dirty());
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.selected_block, selected_block_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(app.notice, notice_before);
    assert_eq!(app.canvas.viewport, viewport_before);
}

#[test]
fn save_without_current_path_uses_dialog_and_persists_document() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("saved.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    assert!(app.session.is_dirty());
    assert_eq!(app.session.path(), None);
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.save_paths.push_back(Some(path.clone()));

    app.execute_document_command(
        DocumentCommand::Save,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(dialogs.save_calls, 1);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());
    let loaded = factory_canvas::persistence::factory_document::load_factory_document(
        &path,
        app.layout.catalog().clone(),
    )
    .unwrap();
    assert_eq!(loaded.layout, app.layout);
    assert_eq!(loaded.next_entity_id, app.next_entity_id);
}

#[test]
fn save_with_current_path_skips_dialog_and_updates_existing_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("current.factory.json");
    let unused_path = directory.path().join("unused.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.save_document_to(&path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    app.select_instance(EntityId::new(1));
    app.move_selected_by(GridPoint::new(1, 0));
    assert!(app.session.is_dirty());
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.save_paths.push_back(Some(unused_path.clone()));

    app.execute_document_command(
        DocumentCommand::Save,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(dialogs.save_calls, 0);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());
    assert!(!unused_path.exists());
    let loaded = factory_canvas::persistence::factory_document::load_factory_document(
        &path,
        app.layout.catalog().clone(),
    )
    .unwrap();
    assert_eq!(
        loaded
            .layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(5, 5))
    );
}

#[test]
fn save_as_uses_dialog_reassociates_and_preserves_previous_file() {
    let directory = tempfile::tempdir().unwrap();
    let first_path = directory.path().join("first.factory.json");
    let second_path = directory.path().join("second.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.save_document_to(&first_path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    let first_bytes = std::fs::read(&first_path).unwrap();
    app.select_instance(EntityId::new(1));
    app.move_selected_by(GridPoint::new(1, 0));
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.save_paths.push_back(Some(second_path.clone()));

    app.execute_document_command(
        DocumentCommand::SaveAs,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(dialogs.save_calls, 1);
    assert_eq!(app.session.path(), Some(second_path.as_path()));
    assert!(!app.session.is_dirty());
    assert_eq!(std::fs::read(&first_path).unwrap(), first_bytes);
    let loaded = factory_canvas::persistence::factory_document::load_factory_document(
        &second_path,
        app.layout.catalog().clone(),
    )
    .unwrap();
    assert_eq!(
        loaded
            .layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(5, 5))
    );
}

#[test]
fn failed_save_as_preserves_session_and_redacts_selected_path() {
    let directory = tempfile::tempdir().unwrap();
    let private_path = directory
        .path()
        .join("missing-private-sentinel")
        .join("factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let metadata_before = app.session.metadata().clone();
    let path_before = app.session.path().map(Path::to_path_buf);
    let layout_before = app.layout.clone();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.save_paths.push_back(Some(private_path.clone()));

    app.execute_document_command(
        DocumentCommand::SaveAs,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert_eq!(app.session.path(), path_before.as_deref());
    assert!(app.session.is_dirty());
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("could not be saved"));
    assert!(!rendered.contains("missing-private-sentinel"));
    assert!(!rendered.contains(&private_path.display().to_string()));
}

#[test]
fn successful_save_reports_completion_without_exposing_path() {
    let directory = tempfile::tempdir().unwrap();
    let private_path = directory.path().join("private-sentinel.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.save_paths.push_back(Some(private_path.clone()));

    app.execute_document_command(
        DocumentCommand::Save,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert_eq!(rendered, "Factory saved.");
    assert!(!rendered.contains("private-sentinel"));
    assert!(!rendered.contains(&private_path.display().to_string()));
}

#[test]
fn open_in_clean_session_loads_selected_document_immediately() {
    let directory = tempfile::tempdir().unwrap();
    let private_path = directory.path().join("private-open.factory.json");
    let mut source = FactoryCanvasApp::default();
    source.select_block(buildable_id("xiranite_power_pole"));
    source.place_selected_at(GridPoint::new(11, 12));
    source
        .save_document_to(&private_path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();

    let mut app = FactoryCanvasApp::default();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(private_path.clone()));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(dialogs.open_calls, 1);
    assert_eq!(app.layout, source.layout);
    assert_eq!(app.session.path(), Some(private_path.as_path()));
    assert!(!app.session.is_dirty());
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert_eq!(rendered, "Factory opened.");
    assert!(!rendered.contains("private-open"));
    assert!(!rendered.contains(&private_path.display().to_string()));
}

#[test]
fn open_reports_catalog_mismatch_classification_without_values() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mismatched.factory.json");
    let source = FactoryCanvasApp::default();
    write_factory_with_mismatched_catalog(&path, &source);
    let mut app = FactoryCanvasApp::default();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(path));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::CatalogAndDataVersionMismatch
    );
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("catalog ID and data version"));
    assert!(!rendered.contains("different_catalog"));
    assert!(!rendered.contains("9.9.9"));
}

#[test]
fn open_of_malformed_json_reports_safe_notice_without_leaking_raw_content() {
    let directory = tempfile::tempdir().unwrap();
    let invalid_path = directory
        .path()
        .join("malformed-private-sentinel.factory.json");
    std::fs::write(
        &invalid_path,
        b"{ \"private_raw_json_sentinel\": \"leak-me-not\", not valid json",
    )
    .unwrap();
    let mut app = FactoryCanvasApp::default();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(invalid_path.clone()));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert!(matches!(app.notice, EditorNotice::DocumentOpenFailed(_)));
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("could not be opened"));
    assert!(!rendered.contains("private_raw_json_sentinel"));
    assert!(!rendered.contains("leak-me-not"));
    assert!(!rendered.contains("malformed-private-sentinel"));
    assert!(!rendered.contains(&invalid_path.display().to_string()));
}

#[test]
fn open_of_invalid_buildable_id_reports_safe_notice_without_leaking_the_value() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("invalid-buildable-private.factory.json");
    let source = FactoryCanvasApp::default();
    let metadata = DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH);
    let bytes = factory_canvas::persistence::factory_document::encode_factory_document(
        &source.layout,
        source.next_entity_id,
        metadata.metadata(),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["next_entity_id"] = serde_json::Value::from(2);
    value["entities"] = serde_json::json!([{
        "id": 1,
        "buildable_id": "Private-Sentinel-Value!",
        "origin": { "x": 1, "y": 1 },
        "rotation_degrees": 0,
        "production_target": null
    }]);
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let mut app = FactoryCanvasApp::default();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(path));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert!(matches!(app.notice, EditorNotice::DocumentOpenFailed(_)));
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("could not be opened"));
    assert!(rendered.contains("invalid buildable ID"));
    assert!(!rendered.contains("Private-Sentinel-Value"));
}

#[test]
fn open_of_metadata_with_updated_before_created_reports_safe_notice_without_leaking_the_name() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("invalid-metadata-private.factory.json");
    let source = FactoryCanvasApp::default();
    let metadata = DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH);
    let bytes = factory_canvas::persistence::factory_document::encode_factory_document(
        &source.layout,
        source.next_entity_id,
        metadata.metadata(),
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["metadata"]["name"] = serde_json::Value::String("Private-Metadata-Sentinel".to_owned());
    value["metadata"]["created_at"] = serde_json::Value::String("2024-01-02T00:00:00Z".to_owned());
    value["metadata"]["updated_at"] = serde_json::Value::String("2024-01-01T00:00:00Z".to_owned());
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let mut app = FactoryCanvasApp::default();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(path));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert!(matches!(app.notice, EditorNotice::DocumentOpenFailed(_)));
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("could not be opened"));
    assert!(rendered.contains("metadata is invalid"));
    assert!(!rendered.contains("Private-Metadata-Sentinel"));
}

#[test]
fn dirty_open_waits_for_confirmation_and_cancel_preserves_state() {
    let directory = tempfile::tempdir().unwrap();
    let source_path = directory.path().join("source.factory.json");
    let mut source = FactoryCanvasApp::default();
    source.select_block(buildable_id("xiranite_power_pole"));
    source.place_selected_at(GridPoint::new(21, 13));
    source
        .save_document_to(&source_path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();

    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(2, 3));
    app.select_instance(EntityId::new(1));
    app.canvas.viewport.pan_by(vec2(17.0, -11.0));
    let layout_before = app.layout.clone();
    let selected_before = app.selected.clone();
    let viewport_before = app.canvas.viewport;
    let notice_before = app.notice.clone();
    let metadata_before = app.session.metadata().clone();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(source_path));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert!(matches!(
        app.pending_unsaved_action.as_ref(),
        Some(PendingUnsavedAction::Open(_))
    ));
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.notice, notice_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert_eq!(app.session.path(), None);
    assert!(app.session.is_dirty());

    app.cancel_pending_unsaved_action();

    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.notice, notice_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert_eq!(app.session.path(), None);
    assert!(app.session.is_dirty());
}

#[test]
fn unsaved_open_modal_is_semantic_and_never_exposes_pending_path() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let private_path = PathBuf::from("C:/private-path-sentinel/secret.factory.json");
    let mut dialogs = StubFactoryFileDialogs {
        open_paths: std::collections::VecDeque::from([Some(private_path.clone())]),
        ..Default::default()
    };
    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        app.pending_unsaved_action.as_ref(),
        Some(PendingUnsavedAction::Open(path)) if path == &private_path
    ));

    let context = egui::Context::default();
    let nodes = unsaved_modal_frame(&context, &mut app, vec![]);
    let texts: Vec<_> = nodes
        .iter()
        .filter_map(|(_, node)| accesskit_node_text(node))
        .collect();

    assert!(texts.contains(&"Unsaved changes"));
    assert!(texts.contains(&"Cancel"));
    assert!(texts.contains(&"Discard and open"));
    assert!(texts
        .iter()
        .any(|text| text.contains("Opening another factory")));
    assert!(texts
        .iter()
        .all(|text| !text.contains("private-path-sentinel")));
    assert!(texts
        .iter()
        .all(|text| !text.contains("secret.factory.json")));
}

#[test]
fn unsaved_modal_uses_action_specific_copy_for_new_and_close() {
    let mut new_app = FactoryCanvasApp::default();
    new_app.select_block(buildable_id("xiranite_power_pole"));
    new_app.place_selected_at(GridPoint::new(4, 5));
    let mut dialogs = StubFactoryFileDialogs::default();
    new_app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    let new_context = egui::Context::default();
    let new_nodes = unsaved_modal_frame(&new_context, &mut new_app, vec![]);
    let new_texts: Vec<_> = new_nodes
        .iter()
        .filter_map(|(_, node)| accesskit_node_text(node))
        .collect();
    assert!(new_texts.contains(&"Discard and create"));
    assert!(new_texts
        .iter()
        .any(|text| text.contains("Creating a new factory")));

    let mut close_app = FactoryCanvasApp::default();
    close_app.select_block(buildable_id("xiranite_power_pole"));
    close_app.place_selected_at(GridPoint::new(4, 5));
    let _ = close_request_frame(&mut close_app);
    let close_context = egui::Context::default();
    let close_nodes = unsaved_modal_frame(&close_context, &mut close_app, vec![]);
    let close_texts: Vec<_> = close_nodes
        .iter()
        .filter_map(|(_, node)| accesskit_node_text(node))
        .collect();
    assert!(close_texts.contains(&"Discard and close"));
    assert!(close_texts
        .iter()
        .any(|text| text.contains("Closing Factory Canvas")));
}

#[test]
fn unsaved_modal_cancel_button_preserves_complete_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    app.canvas.viewport.pan_by(vec2(18.0, -11.0));
    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    let layout_before = app.layout.clone();
    let next_id_before = app.next_entity_id;
    let selected_before = app.selected.clone();
    let viewport_before = app.canvas.viewport;
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    let context = egui::Context::default();
    let nodes = unsaved_modal_frame(&context, &mut app, vec![]);
    let cancel_node_id = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button
                && accesskit_node_text(node) == Some("Cancel")
        })
        .map(|(node_id, _)| *node_id)
        .expect("cancel button must be accessible");
    let _ = unsaved_modal_frame(
        &context,
        &mut app,
        vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: cancel_node_id,
                data: None,
            },
        )],
    );

    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(app.selected, selected_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert!(app.session.is_dirty());
}

#[test]
fn unsaved_modal_escape_cancels_without_discarding_document() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let layout_before = app.layout.clone();
    let next_id_before = app.next_entity_id;
    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(app.pending_unsaved_action.is_some());

    let context = egui::Context::default();
    let _ = unsaved_modal_frame(&context, &mut app, vec![]);
    let _ = unsaved_modal_frame(
        &context,
        &mut app,
        vec![key_press(egui::Key::Escape, egui::Modifiers::NONE)],
    );

    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert!(app.session.is_dirty());
}

#[test]
fn unsaved_modal_confirm_button_executes_pending_open() {
    let directory = tempfile::tempdir().unwrap();
    let source_path = directory.path().join("modal-source.factory.json");
    let mut source = FactoryCanvasApp::default();
    source.select_block(buildable_id("xiranite_power_pole"));
    source.place_selected_at(GridPoint::new(18, 19));
    source
        .save_document_to(&source_path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();

    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(2, 3));
    let mut dialogs = StubFactoryFileDialogs {
        open_paths: std::collections::VecDeque::from([Some(source_path.clone())]),
        ..Default::default()
    };
    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(app.pending_unsaved_action.is_some());

    let context = egui::Context::default();
    let nodes = unsaved_modal_frame(&context, &mut app, vec![]);
    let confirm_node_id = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button
                && accesskit_node_text(node) == Some("Discard and open")
        })
        .map(|(node_id, _)| *node_id)
        .expect("open confirmation button must be accessible");
    let _ = unsaved_modal_frame(
        &context,
        &mut app,
        vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: confirm_node_id,
                data: None,
            },
        )],
    );

    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, source.layout);
    assert_eq!(app.session.path(), Some(source_path.as_path()));
    assert!(!app.session.is_dirty());
}

#[test]
fn confirmed_dirty_open_executes_once_and_clears_pending_action() {
    let directory = tempfile::tempdir().unwrap();
    let source_path = directory.path().join("confirmed-source.factory.json");
    let mut source = FactoryCanvasApp::default();
    source.select_block(buildable_id("xiranite_power_pole"));
    source.place_selected_at(GridPoint::new(25, 17));
    source
        .save_document_to(&source_path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    let expected_layout = source.layout.clone();

    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(2, 3));
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(source_path.clone()));
    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(app.pending_unsaved_action.is_some());

    app.confirm_pending_unsaved_action(&egui::Context::default(), time::OffsetDateTime::UNIX_EPOCH);

    assert_eq!(app.layout, expected_layout);
    assert_eq!(app.session.path(), Some(source_path.as_path()));
    assert!(!app.session.is_dirty());
    assert!(app.pending_unsaved_action.is_none());
    let layout_after_confirmation = app.layout.clone();
    let metadata_after_confirmation = app.session.metadata().clone();

    app.confirm_pending_unsaved_action(&egui::Context::default(), time::OffsetDateTime::UNIX_EPOCH);

    assert_eq!(app.layout, layout_after_confirmation);
    assert_eq!(app.session.metadata(), &metadata_after_confirmation);
    assert!(app.pending_unsaved_action.is_none());
}

#[test]
fn new_in_clean_session_replaces_document_immediately() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("associated.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.replace_base(base_id("wuling_sub_standard"));
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.save_document_to(&path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    app.canvas.focus_selection_requested = true;
    assert_eq!(app.layout.base_id(), &base_id("wuling_sub_standard"));
    assert!(!app.layout.is_empty());
    assert!(app.selected_block.is_some());
    assert!(app.canvas.focus_selection_requested);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert!(!app.session.is_dirty());
    let mut dialogs = StubFactoryFileDialogs::default();

    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(app.layout.base_id(), &base_id("wuling_main"));
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert!(app.selected_block.is_none());
    assert!(app.selected.is_empty());
    assert!(!app.canvas.focus_selection_requested);
    assert!(app.pending_instance_removal.is_none());
    assert_eq!(app.session.path(), None);
    assert!(!app.session.is_dirty());
    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(dialogs.open_calls, 0);
    assert_eq!(dialogs.save_calls, 0);
}

#[test]
fn dirty_new_waits_for_confirmation_and_cancel_preserves_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    app.canvas.viewport.pan_by(egui::vec2(21.0, -13.0));
    let layout_before = app.layout.clone();
    let next_id_before = app.next_entity_id;
    let selected_block_before = app.selected_block.clone();
    let viewport_before = app.canvas.viewport;
    let metadata_before = app.session.metadata().clone();
    assert!(app.session.is_dirty());
    let mut dialogs = StubFactoryFileDialogs::default();

    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(app.selected_block, selected_block_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert!(app.session.is_dirty());

    app.cancel_pending_unsaved_action();

    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.next_entity_id, next_id_before);
    assert_eq!(app.selected_block, selected_block_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert!(app.session.is_dirty());
}

#[test]
fn confirmed_dirty_new_executes_once_and_clears_pending_action() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    app.confirm_pending_unsaved_action(&egui::Context::default(), time::OffsetDateTime::UNIX_EPOCH);

    assert_eq!(app.layout.base_id(), &base_id("wuling_main"));
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(
        app.session.metadata().created_at(),
        time::OffsetDateTime::UNIX_EPOCH
    );
    assert_eq!(app.session.path(), None);
    assert!(!app.session.is_dirty());
    assert!(app.pending_unsaved_action.is_none());
    let metadata_after_confirmation = app.session.metadata().clone();

    app.confirm_pending_unsaved_action(
        &egui::Context::default(),
        time::OffsetDateTime::UNIX_EPOCH + time::Duration::DAY,
    );

    assert_eq!(app.session.metadata(), &metadata_after_confirmation);
    assert!(app.pending_unsaved_action.is_none());
}

#[test]
fn dirty_close_is_cancelled_and_waits_for_confirmation() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    let layout_before = app.layout.clone();
    let metadata_before = app.session.metadata().clone();
    assert!(app.session.is_dirty());

    let commands = close_request_frame(&mut app);

    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::Close)
    ));
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.session.metadata(), &metadata_before);
    assert!(app.session.is_dirty());
}

#[test]
fn close_during_pending_base_change_does_not_open_second_modal() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    app.request_base_change(base_id("wuling_sub_standard"));
    let pending_base_before = app.pending_base_change.clone();
    assert!(pending_base_before.is_some());
    assert!(app.session.is_dirty());

    let commands = close_request_frame(&mut app);

    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert_eq!(app.pending_base_change, pending_base_before);
    assert!(app.pending_unsaved_action.is_none());
    assert!(app.session.is_dirty());
}

#[test]
fn close_during_pending_removal_does_not_open_second_modal() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    app.select_instance(EntityId::new(1));
    app.request_selected_instance_removal();
    let pending_removal_before = app.pending_instance_removal.clone();
    assert!(pending_removal_before.is_some());
    assert!(app.session.is_dirty());

    let commands = close_request_frame(&mut app);

    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert_eq!(app.pending_instance_removal, pending_removal_before);
    assert!(app.pending_unsaved_action.is_none());
    assert!(app.session.is_dirty());
}

#[test]
fn close_during_pending_unsaved_action_preserves_original_action() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    let mut dialogs = StubFactoryFileDialogs::default();
    app.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    let commands = close_request_frame(&mut app);

    assert!(commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));
    assert!(app.session.is_dirty());
}

#[test]
fn document_commands_are_blocked_while_any_destructive_modal_is_open() {
    let directory = tempfile::tempdir().unwrap();
    let trap_path = directory.path().join("must-not-be-used.factory.json");

    let mut base_modal = FactoryCanvasApp::default();
    base_modal.select_block(buildable_id("xiranite_power_pole"));
    base_modal.place_selected_at(GridPoint::new(8, 9));
    base_modal.request_base_change(base_id("wuling_sub_standard"));
    let pending_base_before = base_modal.pending_base_change.clone();
    let mut base_dialogs = StubFactoryFileDialogs::default();
    base_dialogs.open_paths.push_back(Some(trap_path.clone()));
    base_modal.execute_document_command(
        DocumentCommand::Open,
        &mut base_dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert_eq!(base_dialogs.open_calls, 0);
    assert_eq!(base_modal.pending_base_change, pending_base_before);
    assert!(base_modal.pending_unsaved_action.is_none());

    let mut removal_modal = FactoryCanvasApp::default();
    removal_modal.select_block(buildable_id("xiranite_power_pole"));
    removal_modal.place_selected_at(GridPoint::new(8, 9));
    removal_modal.select_instance(EntityId::new(1));
    removal_modal.request_selected_instance_removal();
    let pending_removal_before = removal_modal.pending_instance_removal.clone();
    let mut removal_dialogs = StubFactoryFileDialogs::default();
    removal_dialogs
        .save_paths
        .push_back(Some(trap_path.clone()));
    removal_modal.execute_document_command(
        DocumentCommand::SaveAs,
        &mut removal_dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert_eq!(removal_dialogs.save_calls, 0);
    assert_eq!(
        removal_modal.pending_instance_removal,
        pending_removal_before
    );
    assert!(removal_modal.pending_unsaved_action.is_none());

    let mut unsaved_modal = FactoryCanvasApp::default();
    unsaved_modal.select_block(buildable_id("xiranite_power_pole"));
    unsaved_modal.place_selected_at(GridPoint::new(8, 9));
    let mut unsaved_dialogs = StubFactoryFileDialogs::default();
    unsaved_modal.execute_document_command(
        DocumentCommand::New,
        &mut unsaved_dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        unsaved_modal.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));
    unsaved_dialogs
        .save_paths
        .push_back(Some(trap_path.clone()));
    unsaved_modal.execute_document_command(
        DocumentCommand::SaveAs,
        &mut unsaved_dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert_eq!(unsaved_dialogs.save_calls, 0);
    assert!(matches!(
        unsaved_modal.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));
    assert!(!trap_path.exists());
}

#[test]
fn pending_unsaved_action_blocks_base_and_removal_requests() {
    let mut base_request = FactoryCanvasApp::default();
    base_request.select_block(buildable_id("xiranite_power_pole"));
    base_request.place_selected_at(GridPoint::new(8, 9));
    let mut dialogs = StubFactoryFileDialogs::default();
    base_request.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        base_request.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    base_request.request_base_change(base_id("wuling_sub_standard"));

    assert!(base_request.pending_base_change.is_none());
    assert!(matches!(
        base_request.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    let mut removal_request = FactoryCanvasApp::default();
    removal_request.select_block(buildable_id("xiranite_power_pole"));
    removal_request.place_selected_at(GridPoint::new(8, 9));
    removal_request.select_instance(EntityId::new(1));
    removal_request.execute_document_command(
        DocumentCommand::New,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );
    assert!(matches!(
        removal_request.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));

    removal_request.request_selected_instance_removal();

    assert!(removal_request.pending_instance_removal.is_none());
    assert!(matches!(
        removal_request.pending_unsaved_action,
        Some(PendingUnsavedAction::New)
    ));
}

#[test]
fn confirmed_dirty_close_emits_close_exactly_once() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    let layout_before = app.layout.clone();
    assert!(app.session.is_dirty());
    let request_commands = close_request_frame(&mut app);
    assert!(request_commands.contains(&egui::ViewportCommand::CancelClose));

    let confirm_commands = confirm_unsaved_frame(&mut app, time::OffsetDateTime::UNIX_EPOCH);

    assert!(confirm_commands.contains(&egui::ViewportCommand::Close));
    assert!(app.pending_unsaved_action.is_none());
    assert_eq!(app.layout, layout_before);
    assert!(app.session.is_dirty());

    let repeated_commands = confirm_unsaved_frame(&mut app, time::OffsetDateTime::UNIX_EPOCH);
    assert!(!repeated_commands.contains(&egui::ViewportCommand::Close));
    assert!(app.pending_unsaved_action.is_none());
}

#[test]
fn confirmed_close_is_not_intercepted_again() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(8, 9));
    let request_commands = close_request_frame(&mut app);
    assert!(request_commands.contains(&egui::ViewportCommand::CancelClose));
    let confirm_commands = confirm_unsaved_frame(&mut app, time::OffsetDateTime::UNIX_EPOCH);
    assert!(confirm_commands.contains(&egui::ViewportCommand::Close));

    let application_close_commands = close_request_frame(&mut app);

    assert!(!application_close_commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(app.pending_unsaved_action.is_none());

    let later_close_commands = close_request_frame(&mut app);
    assert!(later_close_commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(matches!(
        app.pending_unsaved_action,
        Some(PendingUnsavedAction::Close)
    ));
}

#[test]
fn clean_close_is_not_cancelled_or_turned_into_pending_action() {
    let mut app = FactoryCanvasApp::default();
    assert!(!app.session.is_dirty());

    let commands = close_request_frame(&mut app);

    assert!(!commands.contains(&egui::ViewportCommand::CancelClose));
    assert!(app.pending_unsaved_action.is_none());
    assert!(!app.session.is_dirty());
}

#[test]
fn failed_open_preserves_editor_state_and_redacts_selected_path() {
    let directory = tempfile::tempdir().unwrap();
    let private_path = directory.path().join("missing-private-open.factory.json");
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.canvas.viewport.pan_by(vec2(37.0, -19.0));
    app.catalog_warning = Some("safe startup warning".to_owned());
    let layout_before = app.layout.clone();
    let selected_block_before = app.selected_block.clone();
    let viewport_before = app.canvas.viewport;
    let warning_before = app.catalog_warning.clone();
    let session_before = app.session.metadata().clone();
    let mut dialogs = StubFactoryFileDialogs::default();
    dialogs.open_paths.push_back(Some(private_path.clone()));

    app.execute_document_command(
        DocumentCommand::Open,
        &mut dialogs,
        time::OffsetDateTime::UNIX_EPOCH,
    );

    assert_eq!(app.layout, layout_before);
    assert_eq!(app.selected_block, selected_block_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.catalog_warning, warning_before);
    assert_eq!(app.session.metadata(), &session_before);
    assert_eq!(app.session.path(), None);
    assert!(!app.session.is_dirty());
    let rendered = notice_text(&app.notice, "Main PAC", app.layout.catalog());
    assert!(rendered.contains("could not be opened"));
    assert!(!rendered.contains("missing-private-open"));
    assert!(!rendered.contains(&private_path.display().to_string()));
}

#[test]
fn cancelling_selected_instance_removal_preserves_complete_editor_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    let notice_before_request = app.notice.clone();
    assert!(app.session.is_dirty());

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
    assert!(app.session.is_dirty());
}

#[test]
fn cancelling_selected_instance_removal_preserves_clean_session() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    app.select_instance(EntityId::new(1));
    assert!(!app.session.is_dirty());

    app.request_selected_instance_removal();
    assert!(app.pending_instance_removal.is_some());
    assert!(!app.session.is_dirty());
    app.cancel_instance_removal();

    assert_eq!(app.pending_instance_removal, None);
    assert!(app.layout.instance(EntityId::new(1)).is_some());
    assert!(!app.session.is_dirty());
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
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    let stale_id = EntityId::new(99);
    app.selected = SelectedSet::new();
    app.selected.insert(stale_id);
    app.pending_instance_removal = Some(vec![stale_id]);
    app.notice = EditorNotice::InstanceSelected {
        id: stale_id,
        buildable_id: buildable_id("xiranite_power_pole"),
    };
    assert!(!app.session.is_dirty());

    app.confirm_instance_removal();

    assert_eq!(app.layout.len(), 1);
    assert!(app.layout.instance(EntityId::new(1)).is_some());
    assert_eq!(app.selected_block, None);
    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.notice, EditorNotice::SelectBlock);
    assert!(!app.session.is_dirty());
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
    assert!(app.session.is_dirty());

    app.place_selected_at(GridPoint::new(0, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert!(app.session.is_dirty());
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
    assert!(app.session.is_dirty());
    assert_eq!(
        app.notice,
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds {
            id: EntityId::new(2),
        })
    );
}

#[test]
fn rejected_placements_preserve_clean_session() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    app.select_block(buildable_id("xiranite_power_pole"));
    assert!(!app.session.is_dirty());

    app.place_selected_at(GridPoint::new(0, 0));
    assert!(!app.session.is_dirty());
    app.place_selected_at(GridPoint::new(79, 79));

    assert!(!app.session.is_dirty());
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
}

#[test]
fn placement_without_selection_does_not_change_layout_or_id() {
    let mut app = FactoryCanvasApp::default();

    app.place_selected_at(GridPoint::new(0, 0));

    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(app.notice, EditorNotice::SelectBlock);
    assert!(!app.session.is_dirty());
}

#[test]
fn entity_id_exhaustion_never_wraps_or_mutates_layout() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.next_entity_id = Some(u64::MAX);

    app.place_selected_at(GridPoint::new(0, 0));
    assert!(app.layout.instance(EntityId::new(u64::MAX)).is_some());
    assert_eq!(app.next_entity_id, None);
    assert!(app.session.is_dirty());

    app.place_selected_at(GridPoint::new(2, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, None);
    assert_eq!(app.notice, EditorNotice::EntityIdsExhausted);
    assert!(app.session.is_dirty());
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
fn confirming_nonempty_base_change_preserves_allocator_and_marks_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    app.select_block(buildable_id("xiranite_power_pole"));
    assert!(!app.session.is_dirty());
    let target = base_id("wuling_sub_standard");
    app.request_base_change(target.clone());

    app.confirm_base_change();

    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.base_id(), &target);
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(2));
    assert!(app.session.is_dirty());
    assert_eq!(
        app.selected_block,
        Some(buildable_id("xiranite_power_pole"))
    );
    assert_eq!(app.notice, EditorNotice::BaseChanged);
}

#[test]
fn successful_placement_marks_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    assert!(!app.session.is_dirty());

    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));

    assert!(app.session.is_dirty());
}

#[test]
fn successful_move_marks_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.select_instance(EntityId::new(1));
    assert!(!app.session.is_dirty());

    app.move_selected_by(GridPoint::new(1, 0));

    assert!(app.session.is_dirty());
}

#[test]
fn successful_single_rotation_marks_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.select_instance(EntityId::new(1));
    assert!(!app.session.is_dirty());

    app.rotate_selected_clockwise();

    assert!(app.session.is_dirty());
}

#[test]
fn successful_group_rotation_marks_document_dirty() {
    let mut app = FactoryCanvasApp::default();
    for (id, origin) in [
        (EntityId::new(1), GridPoint::new(4, 5)),
        (EntityId::new(2), GridPoint::new(8, 5)),
    ] {
        app.layout
            .place(BlockInstance::new(
                id,
                buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            ))
            .unwrap();
    }
    app.selected
        .apply(SelectionMode::Replace, [EntityId::new(1), EntityId::new(2)]);
    assert!(!app.session.is_dirty());

    app.rotate_selected_clockwise();

    assert!(app.session.is_dirty());
}

#[test]
fn successful_production_target_change_marks_document_dirty() {
    let mut app = production_test_app();
    assert!(!app.session.is_dirty());

    app.set_selected_production_target(Some(product_id("test_product_a")));

    assert!(app.session.is_dirty());
}

#[test]
fn confirmed_instance_removal_marks_document_dirty_only_after_mutation() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.select_instance(EntityId::new(1));

    app.request_selected_instance_removal();
    assert!(!app.session.is_dirty());

    app.confirm_instance_removal();

    assert!(app.session.is_dirty());
}

#[test]
fn successful_save_commits_metadata_path_and_clean_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("saved.factory.json");
    let mut app = FactoryCanvasApp {
        session: DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH),
        ..FactoryCanvasApp::default()
    };
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    let saved_at = time::OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1);
    assert!(app.session.is_dirty());

    app.save_document_to(&path, saved_at).unwrap();

    assert!(!app.session.is_dirty());
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert_eq!(
        app.session.metadata().created_at(),
        time::OffsetDateTime::UNIX_EPOCH
    );
    assert_eq!(app.session.metadata().updated_at(), saved_at);
    let loaded = factory_canvas::persistence::factory_document::load_factory_document(
        &path,
        app.layout.catalog().clone(),
    )
    .unwrap();
    assert_eq!(loaded.layout, app.layout);
    assert_eq!(loaded.next_entity_id, app.next_entity_id);
    assert_eq!(&loaded.metadata, app.session.metadata());
}

#[test]
fn successful_open_atomically_replaces_document_and_resets_editor_transients() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("open.factory.json");
    let saved_at = time::OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1);
    let mut source = FactoryCanvasApp {
        session: DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH),
        ..FactoryCanvasApp::default()
    };
    source.select_block(buildable_id("xiranite_power_pole"));
    source.place_selected_at(GridPoint::new(4, 5));
    source.save_document_to(&path, saved_at).unwrap();
    let expected_layout = source.layout.clone();
    let expected_metadata = source.session.metadata().clone();

    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(10, 10));
    app.select_instance(EntityId::new(1));
    app.request_selected_instance_removal();
    app.canvas.focus_selection_requested = true;
    app.canvas.viewport.pan_by(vec2(37.0, -19.0));
    app.catalog_warning = Some("Safe startup warning".to_owned());
    let viewport_before = app.canvas.viewport;
    let catalog_warning_before = app.catalog_warning.clone();
    assert!(app.session.is_dirty());
    assert!(!app.selected.is_empty());
    assert!(app.pending_instance_removal.is_some());
    assert!(app.canvas.focus_selection_requested);
    assert_ne!(app.canvas.viewport, CanvasViewport::default());
    assert!(app.catalog_warning.is_some());

    app.open_document_from(&path).unwrap();

    assert_eq!(app.layout, expected_layout);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.session.metadata(), &expected_metadata);
    assert_eq!(app.session.path(), Some(path.as_path()));
    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::Exact
    );
    assert!(!app.session.is_dirty());
    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert!(!app.canvas.focus_selection_requested);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.catalog_warning, catalog_warning_before);
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn successful_open_clears_active_placement_tool_and_pending_base_change() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("open-empty.factory.json");
    let mut source = FactoryCanvasApp::default();
    source
        .save_document_to(&path, time::OffsetDateTime::UNIX_EPOCH)
        .unwrap();
    let expected_layout = source.layout.clone();

    let mut app = FactoryCanvasApp::default();
    let selected_block = buildable_id("xiranite_power_pole");
    let pending_base = base_id("wuling_sub_standard");
    app.select_block(selected_block.clone());
    app.place_selected_at(GridPoint::new(4, 5));
    app.request_base_change(pending_base.clone());
    assert_eq!(app.selected_block, Some(selected_block));
    assert_eq!(app.pending_base_change, Some(pending_base));
    assert!(app.session.is_dirty());

    app.open_document_from(&path).unwrap();

    assert_eq!(app.layout, expected_layout);
    assert_eq!(app.selected_block, None);
    assert_eq!(app.pending_base_change, None);
    assert!(!app.session.is_dirty());
}

#[test]
fn failed_save_preserves_previous_session_and_file() {
    let directory = tempfile::tempdir().unwrap();
    let original_path = directory.path().join("original.factory.json");
    let missing_path = directory
        .path()
        .join("missing")
        .join("replacement.factory.json");
    let first_save = time::OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1);
    let failed_save = first_save + time::Duration::hours(1);
    let mut app = FactoryCanvasApp {
        session: DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH),
        ..FactoryCanvasApp::default()
    };
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.save_document_to(&original_path, first_save).unwrap();
    let original_bytes = std::fs::read(&original_path).unwrap();
    app.select_instance(EntityId::new(1));
    app.move_selected_by(GridPoint::new(1, 0));
    let metadata_before = app.session.metadata().clone();

    let error = app
        .save_document_to(&missing_path, failed_save)
        .unwrap_err();

    assert!(matches!(
        error,
        FactoryDocumentError::Io {
            operation: factory_canvas::persistence::factory_document::FactoryDocumentIoOperation::CreateTemporary,
            kind: std::io::ErrorKind::NotFound,
        }
    ));
    assert_eq!(app.session.path(), Some(original_path.as_path()));
    assert_eq!(app.session.metadata(), &metadata_before);
    assert!(app.session.is_dirty());
    assert_eq!(std::fs::read(&original_path).unwrap(), original_bytes);
}

#[test]
fn invalid_open_preserves_complete_document_and_editor_state() {
    let directory = tempfile::tempdir().unwrap();
    let current_path = directory.path().join("current.factory.json");
    let invalid_path = directory.path().join("invalid.factory.json");
    std::fs::write(&invalid_path, b"{ private invalid bytes").unwrap();
    let mut app = FactoryCanvasApp {
        session: DocumentSession::untitled_at(time::OffsetDateTime::UNIX_EPOCH),
        ..FactoryCanvasApp::default()
    };
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.save_document_to(
        &current_path,
        time::OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1),
    )
    .unwrap();
    app.select_instance(EntityId::new(1));
    app.move_selected_by(GridPoint::new(1, 0));
    app.request_selected_instance_removal();

    let layout_before = app.layout.clone();
    let next_entity_id_before = app.next_entity_id;
    let path_before = app.session.path().map(Path::to_path_buf);
    let metadata_before = app.session.metadata().clone();
    let compatibility_before = app.session.compatibility();
    let selected_before: Vec<_> = app.selected.iter().collect();
    let selected_block_before = app.selected_block.clone();
    let pending_base_change_before = app.pending_base_change.clone();
    let pending_instance_removal_before = app.pending_instance_removal.clone();
    let notice_before = app.notice.clone();
    let viewport_before = app.canvas.viewport;
    let catalog_warning_before = app.catalog_warning.clone();

    let error = app.open_document_from(&invalid_path).unwrap_err();

    assert!(matches!(error, FactoryDocumentError::InvalidJson { .. }));
    assert_eq!(app.layout, layout_before);
    assert_eq!(app.next_entity_id, next_entity_id_before);
    assert_eq!(app.session.path(), path_before.as_deref());
    assert_eq!(app.session.metadata(), &metadata_before);
    assert_eq!(app.session.compatibility(), compatibility_before);
    assert!(app.session.is_dirty());
    assert_eq!(app.selected.iter().collect::<Vec<_>>(), selected_before);
    assert_eq!(app.selected_block, selected_block_before);
    assert_eq!(app.pending_base_change, pending_base_change_before);
    assert_eq!(
        app.pending_instance_removal,
        pending_instance_removal_before
    );
    assert_eq!(app.notice, notice_before);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.catalog_warning, catalog_warning_before);
}

#[test]
fn successful_open_records_catalog_compatibility_warning_classification() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mismatch.factory.json");
    let source = FactoryCanvasApp::default();
    write_factory_with_mismatched_catalog(&path, &source);
    let mut app = FactoryCanvasApp::default();

    app.open_document_from(&path).unwrap();

    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::CatalogAndDataVersionMismatch
    );
    assert!(!app.session.is_dirty());

    app.save_document_to(
        &path,
        time::OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1),
    )
    .unwrap();

    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::Exact
    );
    assert!(!app.session.is_dirty());
}

#[test]
fn successful_save_never_regresses_updated_timestamp() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("monotonic.factory.json");
    let created_at = time::OffsetDateTime::UNIX_EPOCH;
    let later = created_at + time::Duration::hours(2);
    let earlier = created_at + time::Duration::hours(1);
    let mut app = FactoryCanvasApp {
        session: DocumentSession::untitled_at(created_at),
        ..FactoryCanvasApp::default()
    };
    app.save_document_to(&path, later).unwrap();
    app.request_base_change(base_id("wuling_sub_standard"));

    app.save_document_to(&path, earlier).unwrap();

    assert_eq!(app.session.metadata().updated_at(), later);
    let loaded = factory_canvas::persistence::factory_document::load_factory_document(
        &path,
        app.layout.catalog().clone(),
    )
    .unwrap();
    assert_eq!(loaded.metadata.updated_at(), later);
}

#[test]
fn unchanged_production_target_keeps_document_clean() {
    let mut app = production_test_app();
    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .unwrap()
            .production_target(),
        None
    );
    assert!(!app.session.is_dirty());

    app.set_selected_production_target(None);

    assert!(!app.session.is_dirty());

    let target = product_id("test_product_b");
    app.set_selected_production_target(Some(target.clone()));
    assert!(app.session.is_dirty());
    app.set_selected_production_target(Some(target));
    assert!(app.session.is_dirty());
}

#[test]
fn zero_delta_move_keeps_document_clean() {
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.select_instance(EntityId::new(1));
    assert!(!app.session.is_dirty());

    app.move_selected_by(GridPoint::new(0, 0));

    assert!(!app.session.is_dirty());

    app.move_selected_by(GridPoint::new(1, 0));
    assert!(app.session.is_dirty());
    app.move_selected_by(GridPoint::new(0, 0));
    assert!(app.session.is_dirty());
}

#[test]
fn new_document_replaces_dirty_factory_with_clean_unassociated_state() {
    let directory = tempfile::tempdir().unwrap();
    let old_path = directory.path().join("old.factory.json");
    let created_at = time::OffsetDateTime::UNIX_EPOCH + time::Duration::days(1);
    let source = FactoryCanvasApp::default();
    write_factory_with_mismatched_catalog(&old_path, &source);
    let mut app = FactoryCanvasApp::default();
    app.open_document_from(&old_path).unwrap();
    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::CatalogAndDataVersionMismatch
    );
    assert!(!app.session.is_dirty());

    let default_base = app.layout.catalog().default_base_id().clone();
    let non_default_base = base_id("wuling_sub_standard");
    let selected_block = buildable_id("xiranite_power_pole");
    app.replace_base(non_default_base.clone());
    app.select_block(selected_block.clone());
    app.place_selected_at(GridPoint::new(4, 5));
    app.request_base_change(default_base.clone());
    app.canvas.focus_selection_requested = true;
    app.canvas.viewport.pan_by(vec2(-23.0, 41.0));
    app.catalog_warning = Some("Safe startup warning".to_owned());
    let viewport_before = app.canvas.viewport;
    let catalog_warning_before = app.catalog_warning.clone();

    assert_eq!(app.layout.base_id(), &non_default_base);
    assert_eq!(app.selected_block, Some(selected_block));
    assert_eq!(app.pending_base_change, Some(default_base.clone()));
    assert!(app.canvas.focus_selection_requested);
    assert_ne!(app.canvas.viewport, CanvasViewport::default());
    assert_eq!(app.session.path(), Some(old_path.as_path()));
    assert!(app.session.is_dirty());

    app.new_document_at(created_at);

    assert_eq!(app.layout.base_id(), &default_base);
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(app.session.path(), None);
    assert_eq!(app.session.metadata().created_at(), created_at);
    assert_eq!(app.session.metadata().updated_at(), created_at);
    assert_eq!(
        app.session.compatibility(),
        factory_canvas::persistence::factory_document::CatalogCompatibility::Exact
    );
    assert!(!app.session.is_dirty());
    assert!(app.selected.is_empty());
    assert_eq!(app.selected_block, None);
    assert_eq!(app.pending_base_change, None);
    assert!(!app.canvas.focus_selection_requested);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.catalog_warning, catalog_warning_before);
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn new_document_clears_nonempty_selection_and_pending_removal() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(buildable_id("xiranite_power_pole"));
    app.place_selected_at(GridPoint::new(4, 5));
    app.select_instance(EntityId::new(1));
    app.request_selected_instance_removal();
    assert!(!app.selected.is_empty());
    assert!(app.pending_instance_removal.is_some());
    assert!(app.session.is_dirty());

    app.new_document_at(time::OffsetDateTime::UNIX_EPOCH);

    assert!(app.selected.is_empty());
    assert_eq!(app.pending_instance_removal, None);
    assert!(app.layout.is_empty());
    assert!(!app.session.is_dirty());
}

#[test]
fn rejected_mutations_and_navigation_preserve_dirty_state() {
    let mut clean = FactoryCanvasApp::default();
    clean
        .layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .unwrap();
    clean.select_instance(EntityId::new(1));
    clean.move_selected_by(GridPoint::new(-1, 0));
    assert!(!clean.session.is_dirty());
    clean.move_selected_by(GridPoint::new(0, 0));
    clean.canvas.viewport.pan_by(vec2(90.0, -45.0));
    assert_ne!(clean.canvas.viewport, CanvasViewport::default());
    clean.apply_canvas_navigation_action(CanvasNavigationAction::FrameAll);
    assert_eq!(clean.canvas.viewport, CanvasViewport::default());
    assert!(!clean.session.is_dirty());

    let mut dirty = FactoryCanvasApp::default();
    dirty.select_block(buildable_id("xiranite_power_pole"));
    dirty.place_selected_at(GridPoint::new(4, 5));
    assert!(dirty.session.is_dirty());
    dirty.select_instance(EntityId::new(1));
    dirty.move_selected_by(GridPoint::new(0, 0));
    assert!(dirty.session.is_dirty());
    dirty.move_selected_by(GridPoint::new(-5, 0));
    assert!(dirty.session.is_dirty());
    dirty.canvas.viewport.pan_by(vec2(-70.0, 35.0));
    assert_ne!(dirty.canvas.viewport, CanvasViewport::default());
    dirty.apply_canvas_navigation_action(CanvasNavigationAction::FrameAll);
    assert_eq!(dirty.canvas.viewport, CanvasViewport::default());
    assert!(dirty.session.is_dirty());
    dirty.request_selected_instance_removal();
    assert!(dirty.session.is_dirty());
    dirty.cancel_instance_removal();
    assert!(dirty.session.is_dirty());
}

#[test]
fn invalid_open_preserves_active_placement_tool_and_viewport() {
    let directory = tempfile::tempdir().unwrap();
    let invalid_path = directory.path().join("invalid.factory.json");
    std::fs::write(&invalid_path, b"{ invalid").unwrap();
    let mut app = FactoryCanvasApp::default();
    app.layout
        .place(BlockInstance::new(
            EntityId::new(1),
            buildable_id("xiranite_power_pole"),
            GridPoint::new(4, 5),
            Rotation::Zero,
        ))
        .unwrap();
    app.next_entity_id = Some(2);
    app.select_block(buildable_id("xiranite_power_pole"));
    app.request_base_change(base_id("wuling_sub_standard"));
    app.canvas.focus_selection_requested = true;
    app.canvas.viewport.pan_by(vec2(37.0, -19.0));
    app.catalog_warning = Some("Safe startup warning".to_owned());
    let tool_before = app.selected_block.clone();
    let pending_base_before = app.pending_base_change.clone();
    let viewport_before = app.canvas.viewport;
    let catalog_warning_before = app.catalog_warning.clone();
    assert!(tool_before.is_some());
    assert!(pending_base_before.is_some());
    assert!(app.canvas.focus_selection_requested);
    assert_ne!(viewport_before, CanvasViewport::default());
    assert!(catalog_warning_before.is_some());
    assert!(!app.session.is_dirty());

    let error = app.open_document_from(&invalid_path).unwrap_err();

    assert!(matches!(error, FactoryDocumentError::InvalidJson { .. }));
    assert_eq!(app.selected_block, tool_before);
    assert_eq!(app.pending_base_change, pending_base_before);
    assert!(app.canvas.focus_selection_requested);
    assert_eq!(app.canvas.viewport, viewport_before);
    assert_eq!(app.catalog_warning, catalog_warning_before);
    assert!(!app.session.is_dirty());
}
