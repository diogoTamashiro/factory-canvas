use super::notices::EditorNotice;
use super::FactoryCanvasApp;
use crate::document_session::DocumentSession;
use eframe::egui;
use factory_canvas::domain::catalog::BaseId;
use factory_canvas::domain::layout::FactoryLayout;
use factory_canvas::persistence::factory_document::{
    load_factory_document, FactoryDocumentError, LoadedFactoryDocument,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocumentCommand {
    New,
    Open,
    Save,
    SaveAs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HistoryCommand {
    Undo,
    Redo,
}

pub(super) enum PendingUnsavedAction {
    New,
    Open(PathBuf),
    Close,
}

pub(super) trait FactoryFileDialogs {
    fn pick_open_path(&mut self, current_path: Option<&Path>) -> Option<PathBuf>;

    fn pick_save_path(&mut self, current_path: Option<&Path>) -> Option<PathBuf>;
}

pub(super) struct NativeFactoryFileDialogs;

fn configured_factory_file_dialog(current_path: Option<&Path>) -> rfd::FileDialog {
    let mut dialog = rfd::FileDialog::new().add_filter("Factory Canvas JSON", &["json"]);
    if let Some(directory) = current_path.and_then(Path::parent) {
        dialog = dialog.set_directory(directory);
    }
    dialog
}

impl FactoryFileDialogs for NativeFactoryFileDialogs {
    fn pick_open_path(&mut self, current_path: Option<&Path>) -> Option<PathBuf> {
        configured_factory_file_dialog(current_path).pick_file()
    }

    fn pick_save_path(&mut self, current_path: Option<&Path>) -> Option<PathBuf> {
        let file_name = current_path
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("untitled.factory.json");
        configured_factory_file_dialog(current_path)
            .set_file_name(file_name)
            .save_file()
    }
}

pub(super) fn document_shortcut_for_frame(
    context: &egui::Context,
    blocked: bool,
) -> Option<DocumentCommand> {
    if blocked || context.text_edit_focused() {
        return None;
    }

    context.input_mut(|input| {
        let ctrl_shift = egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        let save_as = egui::KeyboardShortcut::new(ctrl_shift, egui::Key::S);
        let save = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);
        let open = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::O);

        if input.consume_shortcut(&save_as) {
            Some(DocumentCommand::SaveAs)
        } else if input.consume_shortcut(&save) {
            Some(DocumentCommand::Save)
        } else if input.consume_shortcut(&open) {
            Some(DocumentCommand::Open)
        } else {
            None
        }
    })
}

/// `Ctrl+Z` undoes, `Ctrl+Y` redoes — the long-standing Windows convention
/// (research.md Decision 4), mirroring `document_shortcut_for_frame`'s
/// exact shape and guard (`blocked` — `destructive_modal_open()` — and
/// text-edit focus both suppress the shortcut, same as every document
/// shortcut already does).
pub(super) fn history_shortcut_for_frame(
    context: &egui::Context,
    blocked: bool,
) -> Option<HistoryCommand> {
    if blocked || context.text_edit_focused() {
        return None;
    }

    context.input_mut(|input| {
        let undo = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        let redo = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Y);

        if input.consume_shortcut(&undo) {
            Some(HistoryCommand::Undo)
        } else if input.consume_shortcut(&redo) {
            Some(HistoryCommand::Redo)
        } else {
            None
        }
    })
}

impl FactoryCanvasApp {
    pub(super) fn replace_base(&mut self, base_id: BaseId) {
        self.history.record(self.snapshot());
        let catalog = self.layout.catalog().clone();
        self.layout = FactoryLayout::new(catalog, base_id)
            .expect("base selected from the active catalog must exist");
        self.canvas.rotation_visuals.resync(&self.layout);
        self.selected.clear();
        self.canvas.clear_transient_interaction();
        self.pending_base_change = None;
        self.pending_instance_removal = None;
        self.session.mark_dirty();
        self.notice = EditorNotice::BaseChanged;
    }

    pub(super) fn request_base_change(&mut self, base_id: BaseId) {
        if self.pending_instance_removal.is_some() || self.pending_unsaved_action.is_some() {
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

    /// Opens the save-as-blueprint dialog for the current selection.
    /// No-op when nothing is selected (FR-003) or the library is unavailable
    /// (research.md Decision 9). The UI independently hides or disables the
    /// corresponding action, but this method remains the enforcement point.
    pub(super) fn request_save_as_blueprint(&mut self) {
        if self.selected.is_empty() || !self.blueprint_library.is_connected() {
            return;
        }
        self.blueprint_library
            .begin_save(self.selected.iter().collect(), &self.layout);
    }

    /// Loads and arms the blueprint identified by `id` for canvas
    /// insertion (spec FR-001). Leaves everything unchanged and shows a
    /// safe notice if the load fails (e.g. no library connected, or the
    /// file was removed from disk since the cached listing was built).
    pub(super) fn request_insert_blueprint(
        &mut self,
        id: factory_canvas::domain::blueprint::BlueprintId,
    ) {
        let catalog = self.layout.catalog().clone();
        match self.blueprint_library.request_insert(&id, &catalog) {
            Some(blueprint) => self.arm_blueprint_for_insertion(blueprint),
            None => self.notice = EditorNotice::BlueprintInsertionUnavailable,
        }
    }

    pub(super) fn save_document_to(
        &mut self,
        path: &Path,
        saved_at: time::OffsetDateTime,
    ) -> Result<(), FactoryDocumentError> {
        self.session
            .save_to(path, &self.layout, self.next_entity_id, saved_at)
    }

    pub(super) fn open_document_from(&mut self, path: &Path) -> Result<(), FactoryDocumentError> {
        let LoadedFactoryDocument {
            layout,
            next_entity_id,
            metadata,
            compatibility,
        } = load_factory_document(path, self.layout.catalog().clone())?;
        let session = DocumentSession::loaded(path, metadata, compatibility);

        self.layout = layout;
        self.next_entity_id = next_entity_id;
        self.canvas.rotation_visuals.resync(&self.layout);
        self.session = session;
        self.selected_block = None;
        self.selected.clear();
        self.canvas.clear_transient_interaction();
        self.pending_base_change = None;
        self.pending_instance_removal = None;
        self.history.clear();
        self.notice = EditorNotice::SelectBlock;
        Ok(())
    }

    pub(super) fn new_document_at(&mut self, created_at: time::OffsetDateTime) {
        let catalog = self.layout.catalog().clone();
        let base_id = catalog.default_base_id().clone();
        let layout = FactoryLayout::new(catalog, base_id)
            .expect("active catalog default base must remain available");
        let session = DocumentSession::untitled_at(created_at);

        self.layout = layout;
        self.next_entity_id = Some(1);
        self.canvas.rotation_visuals.resync(&self.layout);
        self.session = session;
        self.selected_block = None;
        self.selected.clear();
        self.canvas.clear_transient_interaction();
        self.pending_base_change = None;
        self.pending_instance_removal = None;
        self.history.clear();
        self.notice = EditorNotice::SelectBlock;
    }

    fn save_document_via_path(&mut self, path: &Path, now: time::OffsetDateTime) {
        self.notice = match self.save_document_to(path, now) {
            Ok(()) => EditorNotice::DocumentSaved,
            Err(error) => EditorNotice::DocumentSaveFailed(error),
        };
    }

    fn open_document_via_path(&mut self, path: &Path) {
        self.notice = match self.open_document_from(path) {
            Ok(()) => EditorNotice::DocumentOpened(self.session.compatibility()),
            Err(error) => EditorNotice::DocumentOpenFailed(error),
        };
    }

    pub(super) fn cancel_pending_unsaved_action(&mut self) {
        self.pending_unsaved_action = None;
    }

    pub(super) fn destructive_modal_open(&self) -> bool {
        self.pending_base_change.is_some()
            || self.pending_instance_removal.is_some()
            || self.pending_unsaved_action.is_some()
            || self.blueprint_library.has_pending_save()
    }

    pub(super) fn confirm_pending_unsaved_action(
        &mut self,
        context: &egui::Context,
        now: time::OffsetDateTime,
    ) {
        let Some(action) = self.pending_unsaved_action.take() else {
            return;
        };

        match action {
            PendingUnsavedAction::New => self.new_document_at(now),
            PendingUnsavedAction::Open(path) => self.open_document_via_path(&path),
            PendingUnsavedAction::Close => {
                self.close_confirmed = true;
                context.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    pub(super) fn handle_close_request(&mut self, context: &egui::Context) {
        if !context.input(|input| input.viewport().close_requested()) {
            return;
        }

        if self.close_confirmed {
            self.close_confirmed = false;
            return;
        }

        if self.session.is_dirty() {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if !self.destructive_modal_open() {
                self.pending_unsaved_action = Some(PendingUnsavedAction::Close);
            }
        }
    }

    pub(super) fn execute_document_command(
        &mut self,
        command: DocumentCommand,
        dialogs: &mut impl FactoryFileDialogs,
        now: time::OffsetDateTime,
    ) {
        if self.destructive_modal_open() {
            return;
        }

        match command {
            DocumentCommand::Open => {
                if let Some(path) = dialogs.pick_open_path(self.session.path()) {
                    if self.session.is_dirty() {
                        self.pending_unsaved_action = Some(PendingUnsavedAction::Open(path));
                    } else {
                        self.open_document_via_path(&path);
                    }
                }
            }
            DocumentCommand::SaveAs => {
                if let Some(path) = dialogs.pick_save_path(self.session.path()) {
                    self.save_document_via_path(&path, now);
                }
            }
            DocumentCommand::Save => {
                let path = self
                    .session
                    .path()
                    .map(Path::to_path_buf)
                    .or_else(|| dialogs.pick_save_path(None));
                if let Some(path) = path {
                    self.save_document_via_path(&path, now);
                }
            }
            DocumentCommand::New => {
                if self.session.is_dirty() {
                    self.pending_unsaved_action = Some(PendingUnsavedAction::New);
                } else {
                    self.new_document_at(now);
                }
            }
        }
    }

    pub(super) fn dispatch_document_command_for_frame(
        &mut self,
        context: &egui::Context,
        header_command: Option<DocumentCommand>,
        dialogs: &mut impl FactoryFileDialogs,
        now: time::OffsetDateTime,
    ) {
        let blocked = self.destructive_modal_open();
        let command = header_command.or_else(|| document_shortcut_for_frame(context, blocked));
        if let Some(command) = command {
            self.execute_document_command(command, dialogs, now);
        }
    }

    /// Mirrors `dispatch_document_command_for_frame`'s exact shape: a
    /// header-button command takes priority, falling back to the keyboard
    /// shortcut for this frame, executed only if one or the other fired.
    pub(super) fn dispatch_history_command_for_frame(
        &mut self,
        context: &egui::Context,
        header_command: Option<HistoryCommand>,
    ) {
        let blocked = self.destructive_modal_open();
        let command = header_command.or_else(|| history_shortcut_for_frame(context, blocked));
        match command {
            Some(HistoryCommand::Undo) => self.undo(),
            Some(HistoryCommand::Redo) => self.redo(),
            None => {}
        }
    }

    pub(super) fn cancel_base_change(&mut self) {
        self.pending_base_change = None;
    }

    pub(super) fn confirm_base_change(&mut self) {
        if let Some(base_id) = self.pending_base_change.clone() {
            self.replace_base(base_id);
        }
    }
}
