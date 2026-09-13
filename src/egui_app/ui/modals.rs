use super::super::colors::{BORDER, SIDEBAR_BACKGROUND};
use super::super::document_commands::PendingUnsavedAction;
use super::super::notices::EditorNotice;
use super::super::FactoryCanvasApp;
use eframe::egui::{self, Button, Color32, Frame, Stroke};

impl FactoryCanvasApp {
    pub(in super::super) fn instance_removal_modal(&mut self, context: &egui::Context) {
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

    pub(in super::super) fn unsaved_changes_modal(&mut self, context: &egui::Context) {
        let Some(pending_action) = self.pending_unsaved_action.as_ref() else {
            return;
        };
        let (message, confirm_label) = match pending_action {
            PendingUnsavedAction::New => (
                "Creating a new factory will discard your unsaved changes.",
                "Discard and create",
            ),
            PendingUnsavedAction::Open(_) => (
                "Opening another factory will discard your unsaved changes.",
                "Discard and open",
            ),
            PendingUnsavedAction::Close => (
                "Closing Factory Canvas will discard your unsaved changes.",
                "Discard and close",
            ),
        };

        let modal_response =
            egui::Modal::new(egui::Id::new("unsaved_changes_modal")).show(context, |ui| {
                ui.heading("Unsaved changes");
                ui.add_space(8.0);
                ui.label(message);
                ui.add_space(16.0);

                let mut action = None;
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(false);
                    }
                    if ui
                        .add(
                            Button::new(confirm_label)
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
        let should_close = modal_response.should_close();
        match modal_response.inner {
            Some(true) => {
                self.confirm_pending_unsaved_action(context, time::OffsetDateTime::now_utc())
            }
            Some(false) => self.cancel_pending_unsaved_action(),
            None if should_close => self.cancel_pending_unsaved_action(),
            None => {}
        }
    }

    pub(in super::super) fn base_change_modal(&mut self, context: &egui::Context) {
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

    pub(in super::super) fn save_as_blueprint_modal(
        &mut self,
        context: &egui::Context,
        now: time::OffsetDateTime,
    ) {
        if self.blueprint_library.pending_save().is_none() {
            return;
        }

        let modal_response = egui::Modal::new(egui::Id::new("save_as_blueprint"))
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BACKGROUND)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(10)
                    .inner_margin(24),
            )
            .show(context, |ui| {
                ui.set_min_width(360.0);
                ui.heading("Save as blueprint");
                ui.add_space(8.0);
                ui.label("Name this blueprint:");
                ui.add_space(4.0);
                let name_input = self
                    .blueprint_library
                    .pending_save_name_mut()
                    .expect("modal is only shown while a save is pending");
                ui.text_edit_singleline(name_input);
                let name_is_blank = name_input.trim().is_empty();
                ui.add_space(16.0);

                ui.label("Interfaces (optional — purely descriptive, no connection is implied):");
                ui.add_space(4.0);
                let boundary_points = self.blueprint_library.pending_boundary_points().to_vec();
                let mut removed_index = None;
                let interfaces = self
                    .blueprint_library
                    .pending_interfaces_mut()
                    .expect("modal is only shown while a save is pending");
                for (index, interface) in interfaces.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut interface.name_input);
                        egui::ComboBox::new(("interface_boundary_point", index), "")
                            .selected_text(match interface.boundary_point_index {
                                Some(point_index) => boundary_points
                                    .get(point_index)
                                    .map(|(anchor, side)| {
                                        format!("{side:?} @ ({}, {})", anchor.x, anchor.y)
                                    })
                                    .unwrap_or_else(|| "Choose a location…".to_owned()),
                                None => "Choose a location…".to_owned(),
                            })
                            .show_ui(ui, |ui| {
                                for (point_index, (anchor, side)) in
                                    boundary_points.iter().enumerate()
                                {
                                    ui.selectable_value(
                                        &mut interface.boundary_point_index,
                                        Some(point_index),
                                        format!("{side:?} @ ({}, {})", anchor.x, anchor.y),
                                    );
                                }
                            });
                        if ui.button("Remove").clicked() {
                            removed_index = Some(index);
                        }
                    });
                }
                let has_incomplete_interface = interfaces.iter().any(|interface| {
                    interface.name_input.trim().is_empty()
                        || interface.boundary_point_index.is_none()
                });
                if ui
                    .add_enabled(!has_incomplete_interface, Button::new("+ Add interface"))
                    .clicked()
                {
                    self.blueprint_library.add_pending_interface();
                }
                if let Some(index) = removed_index {
                    self.blueprint_library.remove_pending_interface(index);
                }
                ui.add_space(16.0);

                let mut action = None;
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(false);
                    }
                    if ui
                        .add_enabled(
                            !name_is_blank && !has_incomplete_interface,
                            Button::new("Save"),
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
            Some(true) => {
                let catalog = self.layout.catalog().clone();
                let result = self
                    .blueprint_library
                    .confirm_save(&self.layout, &catalog, now);
                if let Some(result) = result {
                    self.notice = match result {
                        Ok(()) => EditorNotice::BlueprintSaved,
                        Err(error) => EditorNotice::BlueprintSaveFailed(error),
                    };
                }
            }
            Some(false) => self.blueprint_library.cancel_save(),
            None if should_close => self.blueprint_library.cancel_save(),
            None => {}
        }
    }
}
