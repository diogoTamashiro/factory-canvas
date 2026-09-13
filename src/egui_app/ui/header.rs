use super::super::colors::{ACCENT, ACCENT_DIM, BORDER, TEXT_MUTED, TEXT_PRIMARY};
use super::super::document_commands::{DocumentCommand, HistoryCommand};
use super::super::FactoryCanvasApp;
use eframe::egui::{
    Align, Button, Color32, Frame, Layout, Margin, RichText, Sense, Stroke, Ui, Vec2,
};

impl FactoryCanvasApp {
    pub(in super::super) fn header_ui(
        &self,
        ui: &mut Ui,
    ) -> (Option<DocumentCommand>, Option<HistoryCommand>) {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let mut command = None;
            let mut history_command = None;
            let commands_enabled = !self.destructive_modal_open();
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
            ui.add_space(16.0);

            for (label, tooltip, candidate) in [
                ("New", "New factory", DocumentCommand::New),
                ("Open", "Open factory (Ctrl+O)", DocumentCommand::Open),
                ("Save", "Save factory (Ctrl+S)", DocumentCommand::Save),
                (
                    "Save As",
                    "Save factory as (Ctrl+Shift+S)",
                    DocumentCommand::SaveAs,
                ),
            ] {
                if ui
                    .add_enabled(
                        commands_enabled,
                        Button::new(RichText::new(label).size(11.0).color(TEXT_PRIMARY))
                            .fill(Color32::from_rgb(20, 34, 45))
                            .stroke(Stroke::new(1.0, BORDER)),
                    )
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    command.get_or_insert(candidate);
                }
            }

            ui.add_space(12.0);

            for (label, tooltip, candidate, enabled) in [
                (
                    "Undo",
                    "Undo (Ctrl+Z)",
                    HistoryCommand::Undo,
                    self.history.can_undo(),
                ),
                (
                    "Redo",
                    "Redo (Ctrl+Y)",
                    HistoryCommand::Redo,
                    self.history.can_redo(),
                ),
            ] {
                if ui
                    .add_enabled(
                        commands_enabled && enabled,
                        Button::new(RichText::new(label).size(11.0).color(TEXT_PRIMARY))
                            .fill(Color32::from_rgb(20, 34, 45))
                            .stroke(Stroke::new(1.0, BORDER)),
                    )
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    history_command.get_or_insert(candidate);
                }
            }

            if self.session.is_dirty() {
                ui.label(
                    RichText::new("* Unsaved")
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(255, 186, 92)),
                );
            }

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

                if let Some(warning) = &self.catalog_warning {
                    ui.label(
                        RichText::new(warning)
                            .size(10.0)
                            .strong()
                            .color(Color32::from_rgb(255, 186, 92)),
                    );
                }

                if let Some(warning) = self.icons.first_warning() {
                    ui.label(
                        RichText::new(warning)
                            .size(10.0)
                            .strong()
                            .color(Color32::from_rgb(255, 186, 92)),
                    );
                }
            });
            (command, history_command)
        })
        .inner
    }
}
