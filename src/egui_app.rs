use crate::egui_canvas::CanvasState;
use eframe::egui::{
    self, vec2, Align, Button, CentralPanel, Color32, Frame, Layout, Margin, RichText, Sense,
    Stroke, Ui, Vec2,
};
use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BlockTemplate, BuildableDefinition, Catalog,
};
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{
    BlockInstance, EntityId, FactoryLayout, InstanceEditError, PlacementError, ResolvedInstance,
};

use crate::selected_set::{SelectedSet, SelectionMode};

const APP_BACKGROUND: Color32 = Color32::from_rgb(8, 13, 20);
const HEADER_BACKGROUND: Color32 = Color32::from_rgb(11, 18, 28);
const SIDEBAR_BACKGROUND: Color32 = Color32::from_rgb(13, 22, 33);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 237, 242);
const TEXT_MUTED: Color32 = Color32::from_rgb(130, 151, 163);
const ACCENT: Color32 = Color32::from_rgb(91, 221, 199);
const ACCENT_DIM: Color32 = Color32::from_rgb(25, 92, 86);
const BORDER: Color32 = Color32::from_rgb(35, 53, 67);

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1_280.0, 800.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Factory Canvas — Arknights: Endfield",
        options,
        Box::new(|creation_context| Ok(Box::new(FactoryCanvasApp::new(creation_context)))),
    )
}

fn base_option_label(definition: &BaseDefinition) -> String {
    let bounds = definition.bounds();
    format!(
        "{} · {} × {}",
        definition.display_name(),
        bounds.width(),
        bounds.height()
    )
}

fn buildable_for_template(template: BlockTemplate, catalog: &Catalog) -> &BuildableDefinition {
    catalog
        .buildable(&template.buildable_id())
        .expect("temporary UI adapter must cover the active catalog")
}

fn block_option_label(template: BlockTemplate, catalog: &Catalog) -> String {
    let definition = buildable_for_template(template, catalog);
    let footprint = definition.footprint();
    format!(
        "{} · {} × {}",
        definition.display_name(),
        footprint.width(),
        footprint.height()
    )
}

fn notice_text(notice: &EditorNotice, current_base_name: &str, catalog: &Catalog) -> String {
    match notice {
        EditorNotice::SelectBlock => "Select a block to get started.".to_owned(),
        EditorNotice::ReadyToPlace { template } => format!(
            "Selected block: {}. Click the grid to place it.",
            buildable_for_template(*template, catalog).display_name()
        ),
        EditorNotice::InstanceSelected { id, template } => format!(
            "Block #{} selected: {}.",
            id.value(),
            buildable_for_template(*template, catalog).display_name()
        ),
        EditorNotice::InstancesSelected { count } => {
            format!("{count} blocks selected.")
        }
        EditorNotice::InstanceRemoved { id, template } => format!(
            "Block #{} removed: {}.",
            id.value(),
            buildable_for_template(*template, catalog).display_name()
        ),
        EditorNotice::InstancesRemoved { count } => format!("{count} blocks removed."),
        EditorNotice::InstanceMoved { id, origin } => format!(
            "Block #{} moved to ({}, {}).",
            id.value(),
            origin.x,
            origin.y
        ),
        EditorNotice::InstancesMoved { count } => format!("{count} blocks moved."),
        EditorNotice::InstanceRotated { id, rotation } => {
            let degrees = match rotation {
                Rotation::Zero => 0,
                Rotation::Clockwise90 => 90,
                Rotation::Clockwise180 => 180,
                Rotation::Clockwise270 => 270,
            };
            format!("Block #{} rotated to {}°.", id.value(), degrees)
        }
        EditorNotice::InstancesRotated { count } => format!("{count} blocks rotated 90°."),
        EditorNotice::InstanceEditRejected(InstanceEditError::EntityNotFound { id }) => {
            format!("Block #{} no longer exists.", id.value())
        }
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds { .. }) => {
            "The block does not fit at this position.".to_owned()
        }
        EditorNotice::InstanceEditRejected(InstanceEditError::Collision {
            conflicting_id, ..
        }) => {
            format!("Position occupied by block #{}.", conflicting_id.value())
        }
        EditorNotice::Placed {
            id,
            template,
            origin,
        } => format!(
            "Block #{} placed at ({}, {}): {}.",
            id.value(),
            origin.x,
            origin.y,
            buildable_for_template(*template, catalog).display_name()
        ),
        EditorNotice::PlacementRejected(PlacementError::DuplicateEntityId { id }) => {
            format!("Internal ID #{} is already in use.", id.value())
        }
        EditorNotice::PlacementRejected(PlacementError::BuildableNotFound {
            buildable_id, ..
        }) => format!(
            "Construction '{}' is not available in this catalog.",
            buildable_id.as_str()
        ),
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds { .. }) => {
            "The block does not fit at this position.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::Collision { conflicting_id, .. }) => {
            format!("Position occupied by block #{}.", conflicting_id.value())
        }
        EditorNotice::EntityIdsExhausted => "No IDs are available for new blocks.".to_owned(),
        EditorNotice::BaseChanged => format!("Base changed to {current_base_name}."),
    }
}

fn layout_count_label(count: usize) -> String {
    match count {
        0 => "No blocks placed".to_owned(),
        1 => "1 block placed".to_owned(),
        _ => format!("{count} blocks placed"),
    }
}

fn selection_count_label(count: usize) -> String {
    match count {
        0 => "No blocks selected".to_owned(),
        1 => "1 block selected".to_owned(),
        _ => format!("{count} blocks selected"),
    }
}

fn template_for_instance(instance: &BlockInstance) -> BlockTemplate {
    BlockTemplate::from_buildable_id(instance.buildable_id())
        .expect("temporary UI adapter must cover the public compatibility catalog")
}

fn instance_semantic_label(resolved: ResolvedInstance<'_>) -> String {
    let instance = resolved.instance();
    let definition = resolved.definition();
    let origin = instance.origin();
    let rotation = match instance.rotation() {
        Rotation::Zero => 0,
        Rotation::Clockwise90 => 90,
        Rotation::Clockwise180 => 180,
        Rotation::Clockwise270 => 270,
    };
    let footprint = resolved.effective_footprint();

    format!(
        "#{} · {} · origin ({}, {}) · {} × {} · {}°",
        instance.id().value(),
        definition.display_name(),
        origin.x,
        origin.y,
        footprint.width(),
        footprint.height(),
        rotation
    )
}

fn configure_style(context: &egui::Context) {
    context.set_theme(egui::Theme::Dark);
    context.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = vec2(8.0, 10.0);
        style.spacing.button_padding = vec2(12.0, 10.0);
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = APP_BACKGROUND;
        style.visuals.window_fill = APP_BACKGROUND;
        style.visuals.faint_bg_color = SIDEBAR_BACKGROUND;
        style.visuals.extreme_bg_color = crate::egui_canvas::CANVAS_BACKGROUND;
        style.visuals.override_text_color = Some(TEXT_PRIMARY);
        style.visuals.selection.bg_fill = ACCENT_DIM;
        style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(17, 31, 44);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(17, 31, 44);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(21, 45, 57);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(21, 45, 57);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_DIM);
        style.visuals.widgets.active.bg_fill = ACCENT_DIM;
        style.visuals.widgets.active.weak_bg_fill = ACCENT_DIM;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EditorNotice {
    SelectBlock,
    ReadyToPlace {
        template: BlockTemplate,
    },
    InstanceSelected {
        id: EntityId,
        template: BlockTemplate,
    },
    InstancesSelected {
        count: usize,
    },
    InstanceRemoved {
        id: EntityId,
        template: BlockTemplate,
    },
    InstancesRemoved {
        count: usize,
    },
    InstanceMoved {
        id: EntityId,
        origin: GridPoint,
    },
    InstancesMoved {
        count: usize,
    },
    InstanceRotated {
        id: EntityId,
        rotation: Rotation,
    },
    InstancesRotated {
        count: usize,
    },
    InstanceEditRejected(InstanceEditError),
    Placed {
        id: EntityId,
        template: BlockTemplate,
        origin: GridPoint,
    },
    PlacementRejected(PlacementError),
    EntityIdsExhausted,
    BaseChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedInstanceAction {
    Move(GridPoint),
    RotateClockwise,
    RequestRemoval,
    FocusSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanvasNavigationAction {
    FrameAll,
}

fn canvas_navigation_action_for_frame(
    home_pressed: bool,
    has_destructive_modal: bool,
) -> Option<CanvasNavigationAction> {
    home_pressed
        .then_some(CanvasNavigationAction::FrameAll)
        .filter(|_| !has_destructive_modal)
}

fn selected_instance_action_for_frame(
    sidebar_action: Option<SelectedInstanceAction>,
    keyboard_action: Option<SelectedInstanceAction>,
) -> Option<SelectedInstanceAction> {
    sidebar_action.or(keyboard_action)
}

struct FactoryCanvasApp {
    layout: FactoryLayout,
    canvas: CanvasState,
    selected_block: Option<BlockTemplate>,
    selected: SelectedSet,
    next_entity_id: Option<u64>,
    notice: EditorNotice,
    pending_base_change: Option<BaseId>,
    pending_instance_removal: Option<Vec<EntityId>>,
}

impl Default for FactoryCanvasApp {
    fn default() -> Self {
        let catalog = load_embedded_public_catalog()
            .expect("versioned embedded public catalog must be valid");
        let base_id = catalog.default_base_id().clone();
        Self {
            layout: FactoryLayout::new(catalog, base_id)
                .expect("embedded public catalog default base must exist"),
            canvas: CanvasState::default(),
            selected_block: None,
            selected: SelectedSet::new(),
            next_entity_id: Some(1),
            notice: EditorNotice::SelectBlock,
            pending_base_change: None,
            pending_instance_removal: None,
        }
    }
}

impl FactoryCanvasApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_style(&creation_context.egui_ctx);
        Self::default()
    }

    fn replace_base(&mut self, base_id: BaseId) {
        let catalog = self.layout.catalog().clone();
        self.layout = FactoryLayout::new(catalog, base_id)
            .expect("base selected from the active catalog must exist");
        self.selected.clear();
        self.canvas.clear_transient_interaction();
        self.next_entity_id = Some(1);
        self.pending_base_change = None;
        self.pending_instance_removal = None;
        self.notice = EditorNotice::BaseChanged;
    }

    fn request_base_change(&mut self, base_id: BaseId) {
        if self.pending_instance_removal.is_some() {
            return;
        }

        if &base_id == self.layout.base_id() {
            self.pending_base_change = None;
        } else if self.layout.is_empty() {
            self.replace_base(base_id);
        } else {
            self.pending_base_change = Some(base_id);
        }
    }

    fn cancel_base_change(&mut self) {
        self.pending_base_change = None;
    }

    fn confirm_base_change(&mut self) {
        if let Some(base_id) = self.pending_base_change.clone() {
            self.replace_base(base_id);
        }
    }

    fn select_block(&mut self, template: BlockTemplate) {
        self.selected_block = Some(template);
        self.selected.clear();
        self.notice = EditorNotice::ReadyToPlace { template };
    }

    fn placement_template_for_canvas(&self) -> Option<BlockTemplate> {
        if self.pending_base_change.is_some() || self.pending_instance_removal.is_some() {
            None
        } else {
            self.selected_block
        }
    }

    fn select_instance(&mut self, id: EntityId) {
        self.select_instance_with_mode(id, SelectionMode::Replace);
    }

    fn refresh_selection_notice(&mut self) {
        let layout = &self.layout;
        self.selected.retain(|id| layout.instance(id).is_some());
        self.notice = match self.selected.len() {
            0 => EditorNotice::SelectBlock,
            1 => {
                let id = self
                    .selected
                    .iter()
                    .next()
                    .expect("selection length is one");
                let instance = self
                    .layout
                    .instance(id)
                    .expect("selection was reconciled with layout");
                EditorNotice::InstanceSelected {
                    id,
                    template: template_for_instance(instance),
                }
            }
            count => EditorNotice::InstancesSelected { count },
        };
    }

    fn select_instance_with_mode(&mut self, id: EntityId, mode: SelectionMode) {
        if self.layout.instance(id).is_some() {
            self.selected_block = None;
            self.selected.apply(mode, [id]);
            self.refresh_selection_notice();
        }
    }

    fn deselect_instance(&mut self) {
        self.selected.clear();
        self.notice = EditorNotice::SelectBlock;
    }

    fn move_selected_by(&mut self, delta: GridPoint) {
        self.refresh_selection_notice();
        let ids: Vec<_> = self.selected.iter().collect();
        if ids.is_empty() {
            self.notice = EditorNotice::SelectBlock;
            return;
        }

        match self.layout.move_instances_by(&ids, delta) {
            Ok(()) => {
                self.selected.translate_rotation_pivot(delta);
                if ids.len() == 1 {
                    let id = ids[0];
                    let origin = self
                        .layout
                        .instance(id)
                        .expect("moved selected instance remains in layout")
                        .origin();
                    self.notice = EditorNotice::InstanceMoved { id, origin };
                } else {
                    self.notice = EditorNotice::InstancesMoved { count: ids.len() };
                }
            }
            Err(error) => self.notice = EditorNotice::InstanceEditRejected(error),
        }
    }

    fn rotate_selected_clockwise(&mut self) {
        self.refresh_selection_notice();
        let ids: Vec<_> = self.selected.iter().collect();
        if ids.is_empty() {
            self.notice = EditorNotice::SelectBlock;
            return;
        }

        let rotation_result = if ids.len() == 1 {
            let id = ids[0];
            let rotation = self
                .layout
                .instance(id)
                .expect("selected instance was reconciled with layout")
                .rotation()
                .clockwise();
            self.layout.rotate_instance(id, rotation).map(|()| None)
        } else {
            let pivot = match self.selected.rotation_pivot() {
                Some(pivot) => Ok(pivot),
                None => self
                    .layout
                    .selection_rotation_pivot(&ids)
                    .map(|pivot| pivot.expect("multiple selected instances have a rotation pivot")),
            };
            pivot.and_then(|pivot| {
                self.layout
                    .rotate_instances_clockwise_about(&ids, pivot)
                    .map(|()| Some(pivot))
            })
        };

        match rotation_result {
            Ok(None) => {
                let id = ids[0];
                let rotation = self
                    .layout
                    .instance(id)
                    .expect("rotated selected instance remains in layout")
                    .rotation();
                self.notice = EditorNotice::InstanceRotated { id, rotation };
            }
            Ok(Some(pivot)) => {
                self.selected.remember_rotation_pivot(pivot);
                self.notice = EditorNotice::InstancesRotated { count: ids.len() };
            }
            Err(error) => self.notice = EditorNotice::InstanceEditRejected(error),
        }
    }

    fn apply_selected_instance_action(&mut self, action: SelectedInstanceAction) {
        match action {
            SelectedInstanceAction::Move(delta) => self.move_selected_by(delta),
            SelectedInstanceAction::RotateClockwise => self.rotate_selected_clockwise(),
            SelectedInstanceAction::RequestRemoval => self.request_selected_instance_removal(),
            SelectedInstanceAction::FocusSelection => self.canvas.focus_selection_requested = true,
        }
    }

    fn apply_canvas_interaction(&mut self, interaction: crate::egui_canvas::CanvasInteraction) {
        match interaction {
            crate::egui_canvas::CanvasInteraction::Select {
                id,
                mode: SelectionMode::Replace,
            } => self.select_instance(id),
            crate::egui_canvas::CanvasInteraction::Select { id, mode } => {
                self.select_instance_with_mode(id, mode)
            }
            crate::egui_canvas::CanvasInteraction::Place(origin) => self.place_selected_at(origin),
            crate::egui_canvas::CanvasInteraction::Deselect => self.deselect_instance(),
            crate::egui_canvas::CanvasInteraction::Marquee { ids, mode } => {
                self.selected_block = None;
                self.selected.apply(mode, ids);
                self.refresh_selection_notice();
            }
        }
    }

    fn apply_canvas_navigation_action(&mut self, action: CanvasNavigationAction) {
        match action {
            CanvasNavigationAction::FrameAll => self.canvas.viewport.frame_all(),
        }
    }

    fn request_selected_instance_removal(&mut self) {
        if self.pending_base_change.is_some() {
            return;
        }

        let ids: Vec<_> = self
            .selected
            .iter()
            .filter(|id| self.layout.instance(*id).is_some())
            .collect();
        self.pending_instance_removal = (!ids.is_empty()).then_some(ids);
    }

    fn cancel_instance_removal(&mut self) {
        self.pending_instance_removal = None;
    }

    fn confirm_instance_removal(&mut self) {
        let Some(ids) = self.pending_instance_removal.take() else {
            return;
        };
        self.selected_block = None;
        let mut removed = Vec::new();
        for id in ids {
            self.selected.remove(id);
            if let Some(instance) = self.layout.remove_instance(id) {
                removed.push(instance);
            }
        }

        self.notice = match removed.as_slice() {
            [] => {
                self.refresh_selection_notice();
                return;
            }
            [instance] => EditorNotice::InstanceRemoved {
                id: instance.id(),
                template: template_for_instance(instance),
            },
            instances => EditorNotice::InstancesRemoved {
                count: instances.len(),
            },
        };
    }

    fn place_selected_at(&mut self, origin: GridPoint) {
        let Some(template) = self.selected_block else {
            self.notice = EditorNotice::SelectBlock;
            return;
        };
        let Some(next_id) = self.next_entity_id else {
            self.notice = EditorNotice::EntityIdsExhausted;
            return;
        };

        let id = EntityId::new(next_id);
        let instance = BlockInstance::new(id, template.buildable_id(), origin, Rotation::Zero);

        match self.layout.place(instance) {
            Ok(()) => {
                self.next_entity_id = next_id.checked_add(1);
                self.notice = EditorNotice::Placed {
                    id,
                    template,
                    origin,
                };
            }
            Err(error) => self.notice = EditorNotice::PlacementRejected(error),
        }
    }

    fn header_ui(&self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(
                RichText::new("FACTORY")
                    .size(20.0)
                    .strong()
                    .color(TEXT_PRIMARY),
            );
            ui.label(RichText::new("CANVAS").size(20.0).strong().color(ACCENT));
            ui.add_space(18.0);
            ui.label(
                RichText::new("LAYOUT EDITOR")
                    .size(11.0)
                    .strong()
                    .color(TEXT_MUTED),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Frame::new()
                    .fill(Color32::from_rgb(13, 43, 42))
                    .stroke(Stroke::new(1.0, ACCENT_DIM))
                    .corner_radius(10)
                    .inner_margin(Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 3.0, ACCENT);
                            ui.label(RichText::new("OFFLINE").size(10.0).strong().color(ACCENT));
                        });
                    });
            });
        });
    }

    fn sidebar_ui(&mut self, ui: &mut Ui) -> Option<SelectedInstanceAction> {
        self.base_picker_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.block_palette_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.editor_state_ui(ui)
    }

    fn base_picker_ui(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("CONSTRUCTION BASE")
                .size(11.0)
                .strong()
                .color(ACCENT),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Choose the confirmed area for the layout.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.add_space(10.0);

        let current_base_id = self.layout.base_id().clone();
        let base_options: Vec<_> = self
            .layout
            .catalog()
            .bases()
            .iter()
            .map(|definition| (definition.id().clone(), base_option_label(definition)))
            .collect();
        let mut requested_base_id = None;

        for (base_id, option_label) in base_options {
            let selected = current_base_id == base_id;
            let label = RichText::new(option_label)
                .size(12.0)
                .strong()
                .color(if selected { ACCENT } else { TEXT_PRIMARY });
            let response = ui.add_sized(
                [ui.available_width(), 40.0],
                Button::new(label).selected(selected),
            );

            if response.clicked() {
                requested_base_id = Some(base_id);
            }
        }

        if let Some(base_id) = requested_base_id {
            self.request_base_change(base_id);
        }
    }

    fn block_palette_ui(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("BLOCKS").size(11.0).strong().color(ACCENT));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Select a block, then click its origin tile.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.add_space(10.0);

        let mut requested_block = None;

        for template in BlockTemplate::ALL {
            let selected = self.selected_block == Some(template);
            let label = RichText::new(block_option_label(template, self.layout.catalog()))
                .size(12.0)
                .strong()
                .color(if selected { ACCENT } else { TEXT_PRIMARY });
            let response = ui.add_sized(
                [ui.available_width(), 40.0],
                Button::new(label).selected(selected),
            );

            if response.clicked() {
                requested_block = Some(template);
            }
        }

        if let Some(template) = requested_block {
            self.select_block(template);
        }
    }

    fn editor_state_ui(&mut self, ui: &mut Ui) -> Option<SelectedInstanceAction> {
        ui.label(
            RichText::new("EDITOR STATUS")
                .size(10.0)
                .strong()
                .color(TEXT_MUTED),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(layout_count_label(self.layout.len()))
                .size(13.0)
                .strong()
                .color(TEXT_PRIMARY),
        );
        if !self.selected.is_empty() {
            ui.label(
                RichText::new(selection_count_label(self.selected.len()))
                    .size(11.0)
                    .strong()
                    .color(ACCENT),
            );
        }
        let notice_color = match &self.notice {
            EditorNotice::PlacementRejected(_)
            | EditorNotice::InstanceEditRejected(_)
            | EditorNotice::EntityIdsExhausted => Color32::from_rgb(245, 132, 124),
            _ => TEXT_MUTED,
        };
        ui.label(
            RichText::new(notice_text(
                &self.notice,
                self.layout.base_definition().display_name(),
                self.layout.catalog(),
            ))
            .size(11.0)
            .color(notice_color),
        );

        if self.layout.is_empty() {
            return None;
        }

        let selection_count = self.selected.len();
        let selected_instance = (selection_count == 1)
            .then(|| self.selected.iter().next())
            .flatten()
            .and_then(|id| self.layout.instance(id).cloned());
        let instances: Vec<_> = self.layout.instances().cloned().collect();
        let mut requested_instance = None;
        let mut requested_action = None;

        if selection_count > 0 {
            ui.add_space(8.0);
            let heading = selected_instance.map_or_else(
                || selection_count_label(selection_count).to_uppercase(),
                |instance| format!("SELECTED BLOCK #{}", instance.id().value()),
            );
            ui.label(RichText::new(heading).size(10.0).strong().color(ACCENT));
            ui.add_space(4.0);
            if ui
                .add_sized(
                    [ui.available_width(), 0.0],
                    Button::new(RichText::new("Frame selection (F)").size(11.0).strong()),
                )
                .clicked()
            {
                requested_action = Some(SelectedInstanceAction::FocusSelection);
            }
            ui.label(
                RichText::new("MOVE 1 TILE · ARROW KEYS")
                    .size(10.0)
                    .strong()
                    .color(TEXT_MUTED),
            );
            ui.horizontal(|ui| {
                if ui.button("Up").clicked() {
                    requested_action = Some(SelectedInstanceAction::Move(GridPoint::new(0, -1)));
                }
                if ui.button("Down").clicked() {
                    requested_action = Some(SelectedInstanceAction::Move(GridPoint::new(0, 1)));
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Left").clicked() {
                    requested_action = Some(SelectedInstanceAction::Move(GridPoint::new(-1, 0)));
                }
                if ui.button("Right").clicked() {
                    requested_action = Some(SelectedInstanceAction::Move(GridPoint::new(1, 0)));
                }
            });
            if ui
                .add_sized(
                    [ui.available_width(), 0.0],
                    Button::new(RichText::new("Rotate 90° (R)").size(11.0).strong()),
                )
                .clicked()
            {
                requested_action = Some(SelectedInstanceAction::RotateClockwise);
            }
            if ui
                .add(
                    Button::new(
                        RichText::new(if selection_count == 1 {
                            "Remove block"
                        } else {
                            "Remove blocks"
                        })
                        .size(11.0)
                        .strong(),
                    )
                    .fill(Color32::from_rgb(125, 48, 48))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(230, 112, 104))),
                )
                .clicked()
            {
                requested_action = Some(SelectedInstanceAction::RequestRemoval);
            }
        }

        ui.add_space(14.0);
        ui.label(
            RichText::new("INSTANCES ON CANVAS")
                .size(10.0)
                .strong()
                .color(TEXT_MUTED),
        );
        ui.add_space(4.0);

        for instance in instances {
            let id = instance.id();
            let resolved = self
                .layout
                .resolved_instance(id)
                .expect("stored instance must resolve through the layout catalog");
            let response = ui.add_sized(
                [ui.available_width(), 0.0],
                egui::Label::new(
                    RichText::new(instance_semantic_label(resolved))
                        .size(11.0)
                        .color(if self.selected.contains(id) {
                            ACCENT
                        } else {
                            TEXT_PRIMARY
                        }),
                )
                .wrap()
                .sense(egui::Sense::click()),
            );
            if response.clicked_by(egui::PointerButton::Primary) {
                let mode = ui.input(|input| {
                    if input.modifiers.ctrl {
                        SelectionMode::Toggle
                    } else if input.modifiers.shift {
                        SelectionMode::Add
                    } else {
                        SelectionMode::Replace
                    }
                });
                requested_instance = Some((id, mode));
            }
        }

        if let Some((id, mode)) = requested_instance {
            self.select_instance_with_mode(id, mode);
        }

        requested_action
    }

    fn canvas_ui(&mut self, ui: &mut Ui) {
        let selected_block = self.placement_template_for_canvas();
        let selected = &self.selected;
        let interaction = crate::egui_canvas::show(
            ui,
            &self.layout,
            self.layout.base_definition().display_name(),
            selected,
            selected_block,
            &mut self.canvas,
        );

        if let Some(interaction) = interaction {
            self.apply_canvas_interaction(interaction);
            ui.ctx().request_repaint();
        }
    }

    fn instance_removal_modal(&mut self, context: &egui::Context) {
        let Some(ids) = self.pending_instance_removal.clone() else {
            return;
        };
        let instances: Vec<_> = ids
            .iter()
            .filter_map(|id| self.layout.instance(*id).cloned())
            .collect();
        if instances.is_empty() {
            self.pending_instance_removal = None;
            for id in ids {
                self.selected.remove(id);
            }
            self.refresh_selection_notice();
            return;
        }
        let count = instances.len();
        let description = if let [instance] = instances.as_slice() {
            let definition = self
                .layout
                .catalog()
                .buildable(instance.buildable_id())
                .expect("stored buildable ID must exist in the layout catalog");
            format!(
                "Block #{} ({}) will be removed.",
                instance.id().value(),
                definition.display_name()
            )
        } else {
            format!("{count} selected blocks will be removed.")
        };
        let heading = if count == 1 {
            "Remove block?"
        } else {
            "Remove blocks?"
        };
        let modal_response = egui::Modal::new(egui::Id::new("confirm_instance_removal"))
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BACKGROUND)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(10)
                    .inner_margin(24),
            )
            .show(context, |ui| {
                ui.set_min_width(360.0);
                ui.heading(heading);
                ui.add_space(8.0);
                ui.label(description);
                ui.add_space(16.0);

                let mut action = None;
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(false);
                    }
                    if ui
                        .add(
                            Button::new(if count == 1 { "Remove" } else { "Remove all" })
                                .fill(Color32::from_rgb(125, 48, 48))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(230, 112, 104))),
                        )
                        .clicked()
                    {
                        action = Some(true);
                    }
                });
                action
            });

        let action = modal_response.inner;
        let should_close = modal_response.should_close();
        match action {
            Some(true) => self.confirm_instance_removal(),
            Some(false) => self.cancel_instance_removal(),
            None if should_close => self.cancel_instance_removal(),
            None => {}
        }
    }

    fn base_change_modal(&mut self, context: &egui::Context) {
        let Some(target) = self.pending_base_change.clone() else {
            return;
        };
        let target_name = self
            .layout
            .catalog()
            .base(&target)
            .expect("pending base change must reference the active catalog")
            .display_name()
            .to_owned();
        let instance_count = self.layout.len();
        let removal_text = if instance_count == 1 {
            "1 block will be removed".to_owned()
        } else {
            format!("{instance_count} blocks will be removed")
        };
        let modal_response = egui::Modal::new(egui::Id::new("confirm_base_change"))
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BACKGROUND)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(10)
                    .inner_margin(24),
            )
            .show(context, |ui| {
                ui.set_min_width(360.0);
                ui.heading("Change base and clear the layout?");
                ui.add_space(8.0);
                ui.label(format!(
                    "The new base will be {}. {removal_text}.",
                    target_name,
                ));
                ui.add_space(16.0);

                let mut action = None;
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(false);
                    }
                    if ui
                        .add(
                            Button::new("Change and clear")
                                .fill(Color32::from_rgb(125, 48, 48))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(230, 112, 104))),
                        )
                        .clicked()
                    {
                        action = Some(true);
                    }
                });
                action
            });

        let action = modal_response.inner;
        let should_close = modal_response.should_close();
        match action {
            Some(true) => self.confirm_base_change(),
            Some(false) => self.cancel_base_change(),
            None if should_close => self.cancel_base_change(),
            None => {}
        }
    }
}

impl eframe::App for FactoryCanvasApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("app_header")
            .exact_size(64.0)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(HEADER_BACKGROUND)
                    .inner_margin(Margin::symmetric(20, 12)),
            )
            .show(ui, |ui| self.header_ui(ui));

        let sidebar_action = egui::Panel::left("base_sidebar")
            .exact_size(264.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BACKGROUND)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(18, 20)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.sidebar_ui(ui))
                    .inner
            })
            .inner;

        CentralPanel::default()
            .frame(Frame::new().fill(APP_BACKGROUND).inner_margin(20))
            .show(ui, |ui| self.canvas_ui(ui));

        let has_destructive_modal =
            self.pending_base_change.is_some() || self.pending_instance_removal.is_some();
        let canvas_navigation_action = ui.input(|input| {
            canvas_navigation_action_for_frame(
                input.key_pressed(egui::Key::Home),
                has_destructive_modal,
            )
        });
        if let Some(action) = canvas_navigation_action {
            self.apply_canvas_navigation_action(action);
            ui.ctx().request_repaint();
        }

        let keyboard_action = if !self.selected.is_empty() && !has_destructive_modal {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace) {
                    Some(SelectedInstanceAction::RequestRemoval)
                } else if input.key_pressed(egui::Key::F) {
                    Some(SelectedInstanceAction::FocusSelection)
                } else if input.key_pressed(egui::Key::ArrowUp) {
                    Some(SelectedInstanceAction::Move(GridPoint::new(0, -1)))
                } else if input.key_pressed(egui::Key::ArrowDown) {
                    Some(SelectedInstanceAction::Move(GridPoint::new(0, 1)))
                } else if input.key_pressed(egui::Key::ArrowLeft) {
                    Some(SelectedInstanceAction::Move(GridPoint::new(-1, 0)))
                } else if input.key_pressed(egui::Key::ArrowRight) {
                    Some(SelectedInstanceAction::Move(GridPoint::new(1, 0)))
                } else if input.key_pressed(egui::Key::R) {
                    Some(SelectedInstanceAction::RotateClockwise)
                } else {
                    None
                }
            })
        } else {
            None
        };
        if let Some(action) = selected_instance_action_for_frame(sidebar_action, keyboard_action) {
            self.apply_selected_instance_action(action);
            ui.ctx().request_repaint();
        }

        self.base_change_modal(ui.ctx());
        self.instance_removal_modal(ui.ctx());
    }
}

#[cfg(test)]
#[path = "egui_app_tests.rs"]
mod tests;
