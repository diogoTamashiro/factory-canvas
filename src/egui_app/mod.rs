mod colors;
mod document_commands;
mod editing_commands;
mod history_bridge;
pub(crate) mod icons;
mod notices;
mod startup;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
mod ui;

use crate::blueprint_library_view::BlueprintLibraryView;
use crate::document_session::DocumentSession;
use crate::egui_canvas::CanvasState;
use crate::history::EditHistory;
use crate::selected_set::SelectedSet;
use colors::{APP_BACKGROUND, BORDER, HEADER_BACKGROUND, SIDEBAR_BACKGROUND};
use document_commands::{FactoryFileDialogs, NativeFactoryFileDialogs, PendingUnsavedAction};
use editing_commands::{
    canvas_navigation_action_for_frame, selected_instance_action_for_frame, SelectedInstanceAction,
};
use eframe::egui::{self, CentralPanel, Frame, Margin, Stroke, Ui};
use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::blueprint::Blueprint;
use factory_canvas::domain::catalog::{BaseId, BuildableId};
use factory_canvas::domain::geometry::GridPoint;
use factory_canvas::domain::layout::{EntityId, FactoryLayout};
use icons::BuildableIcons;
use notices::EditorNotice;
use startup::StartupCatalog;

struct FactoryCanvasApp {
    layout: FactoryLayout,
    canvas: CanvasState,
    session: DocumentSession,
    blueprint_library: BlueprintLibraryView,
    catalog_warning: Option<String>,
    icons: BuildableIcons,
    selected_block: Option<BuildableId>,
    armed_blueprint: Option<Blueprint>,
    selected: SelectedSet,
    next_entity_id: Option<u64>,
    history: EditHistory,
    notice: EditorNotice,
    pending_base_change: Option<BaseId>,
    pending_instance_removal: Option<Vec<EntityId>>,
    pending_unsaved_action: Option<PendingUnsavedAction>,
    close_confirmed: bool,
}

impl Default for FactoryCanvasApp {
    fn default() -> Self {
        let catalog = load_embedded_public_catalog()
            .expect("versioned embedded public catalog must be valid");
        Self::from_startup_catalog(StartupCatalog {
            catalog,
            warning: None,
        })
    }
}

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

impl FactoryCanvasApp {
    fn ui_with_dialogs(&mut self, ui: &mut Ui, dialogs: &mut impl FactoryFileDialogs) {
        let (header_command, header_history_command) = egui::Panel::top("app_header")
            .exact_size(64.0)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(HEADER_BACKGROUND)
                    .inner_margin(Margin::symmetric(20, 12)),
            )
            .show(ui, |ui| self.header_ui(ui))
            .inner;

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

        let has_destructive_modal = self.destructive_modal_open();
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

        self.dispatch_document_command_for_frame(
            ui.ctx(),
            header_command,
            dialogs,
            time::OffsetDateTime::now_utc(),
        );
        self.dispatch_history_command_for_frame(ui.ctx(), header_history_command);

        self.base_change_modal(ui.ctx());
        self.instance_removal_modal(ui.ctx());
        self.handle_close_request(ui.ctx());
        self.unsaved_changes_modal(ui.ctx());
        self.save_as_blueprint_modal(ui.ctx(), time::OffsetDateTime::now_utc());
    }
}

impl eframe::App for FactoryCanvasApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let mut dialogs = NativeFactoryFileDialogs;
        self.ui_with_dialogs(ui, &mut dialogs);
    }
}
