use super::super::FactoryCanvasApp;
use eframe::egui::Ui;

impl FactoryCanvasApp {
    pub(in super::super) fn canvas_ui(&mut self, ui: &mut Ui) {
        let selected_block = self.placement_buildable_for_canvas().cloned();
        let armed_blueprint = self.armed_blueprint_for_canvas().cloned();
        let selected = &self.selected;
        let interaction = crate::egui_canvas::show(
            ui,
            crate::egui_canvas::CanvasFrameInput {
                layout: &self.layout,
                title: self.layout.base_definition().display_name(),
                selected,
                selected_block: selected_block.as_ref(),
                armed_blueprint: armed_blueprint.as_ref(),
                icons: &self.icons,
            },
            &mut self.canvas,
        );

        if let Some(interaction) = interaction {
            self.apply_canvas_interaction(interaction);
            ui.ctx().request_repaint();
        }
    }
}
