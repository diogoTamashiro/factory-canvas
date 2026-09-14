use super::super::colors::{ACCENT, TEXT_MUTED, TEXT_PRIMARY};
use super::super::editing_commands::{production_target_action_for_choice, SelectedInstanceAction};
use super::super::notices::{catalog_compatibility_mismatch_text, notice_color, notice_text};
use super::super::FactoryCanvasApp;
use crate::selected_set::{SelectedSet, SelectionMode};
use eframe::egui::{self, Button, Color32, RichText, Stroke, Ui};
use factory_canvas::domain::catalog::{BaseDefinition, BuildableDefinition, Catalog, ProductId};
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{FactoryLayout, ResolvedInstance};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProductionTargetOption {
    pub(in super::super) product_id: ProductId,
    pub(in super::super) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProductionTargetControl {
    pub(in super::super) current: Option<ProductId>,
    pub(in super::super) options: Vec<ProductionTargetOption>,
}

pub(in super::super) fn production_target_control(
    layout: &FactoryLayout,
    selected: &SelectedSet,
) -> Option<ProductionTargetControl> {
    if selected.len() != 1 {
        return None;
    }
    let resolved = layout.resolved_instance(selected.iter().next()?)?;
    let production_targets = resolved.definition().production_targets();
    if production_targets.is_empty() {
        return None;
    }
    let options = production_targets
        .iter()
        .map(|product_id| {
            let product = layout
                .catalog()
                .product(product_id)
                .expect("validated catalog production target must resolve");
            ProductionTargetOption {
                product_id: product_id.clone(),
                display_name: product.display_name().to_owned(),
            }
        })
        .collect();

    Some(ProductionTargetControl {
        current: resolved.instance().production_target().cloned(),
        options,
    })
}

pub(in super::super) fn base_option_label(definition: &BaseDefinition) -> String {
    let bounds = definition.bounds();
    format!(
        "{} · {} × {}",
        definition.display_name(),
        bounds.width(),
        bounds.height()
    )
}

pub(in super::super) fn block_option_label(definition: &BuildableDefinition) -> String {
    let footprint = definition.footprint();
    format!(
        "{} · {} × {}",
        definition.display_name(),
        footprint.width(),
        footprint.height()
    )
}

pub(in super::super) fn layout_count_label(count: usize) -> String {
    match count {
        0 => "No blocks placed".to_owned(),
        1 => "1 block placed".to_owned(),
        _ => format!("{count} blocks placed"),
    }
}

pub(in super::super) fn selection_count_label(count: usize) -> String {
    match count {
        0 => "No blocks selected".to_owned(),
        1 => "1 block selected".to_owned(),
        _ => format!("{count} blocks selected"),
    }
}

/// Formats a blueprint's `updated_at` timestamp for the sidebar listing as
/// a fixed, human-readable `YYYY-MM-DD HH:MM UTC` string. No existing
/// display-formatting convention exists elsewhere in this file to reuse
/// (the only prior `time` formatting in this codebase, `Rfc3339`, is for
/// the JSON document codec, not UI display) — this is a new, minimal,
/// fixed format rather than a new dependency.
pub(in super::super) fn format_blueprint_timestamp(timestamp: time::OffsetDateTime) -> String {
    let format =
        time::format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute] UTC")
            .expect("fixed format string is valid at compile time in practice");
    timestamp
        .to_offset(time::UtcOffset::UTC)
        .format(&format)
        .unwrap_or_else(|_| "unknown time".to_owned())
}

pub(in super::super) fn instance_semantic_label(
    resolved: ResolvedInstance<'_>,
    catalog: &Catalog,
) -> String {
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
    let production = instance.production_target().map_or_else(
        || "no product".to_owned(),
        |product_id| {
            let product = catalog
                .product(product_id)
                .expect("stored production target must resolve through the layout catalog");
            format!("product {}", product.display_name())
        },
    );

    format!(
        "#{} · {} · origin ({}, {}) · {} × {} · {}° · {}",
        instance.id().value(),
        definition.display_name(),
        origin.x,
        origin.y,
        footprint.width(),
        footprint.height(),
        rotation,
        production
    )
}

impl FactoryCanvasApp {
    pub(in super::super) fn sidebar_ui(&mut self, ui: &mut Ui) -> Option<SelectedInstanceAction> {
        self.base_picker_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.block_palette_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        let action = self.editor_state_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        self.blueprint_library_section_ui(ui);

        action
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

        let options: Vec<_> = self
            .layout
            .catalog()
            .buildables()
            .iter()
            .map(|definition| (definition.id().clone(), block_option_label(definition)))
            .collect();
        let mut requested_block = None;

        for (buildable_id, option_label) in options {
            let selected = self.selected_block.as_ref() == Some(&buildable_id);
            let label = RichText::new(option_label)
                .size(12.0)
                .strong()
                .color(if selected { ACCENT } else { TEXT_PRIMARY });
            let response = ui.add_sized(
                [ui.available_width(), 40.0],
                Button::new(label).selected(selected),
            );

            if response.clicked() {
                requested_block = Some(buildable_id);
            }
        }

        if let Some(buildable_id) = requested_block {
            self.select_block(buildable_id);
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
        let notice_color = notice_color(&self.notice);
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
            if let Some(control) = production_target_control(&self.layout, &self.selected) {
                ui.label(
                    RichText::new("PRODUCT")
                        .size(10.0)
                        .strong()
                        .color(TEXT_MUTED),
                );
                let mut choice = control.current.clone();
                let selected_text = choice
                    .as_ref()
                    .and_then(|product_id| {
                        control
                            .options
                            .iter()
                            .find(|option| option.product_id == *product_id)
                    })
                    .map_or("No product", |option| option.display_name.as_str());
                egui::ComboBox::from_id_salt("selected_production_target")
                    .width(ui.available_width())
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut choice, None, "No product");
                        for option in &control.options {
                            ui.selectable_value(
                                &mut choice,
                                Some(option.product_id.clone()),
                                &option.display_name,
                            );
                        }
                    });
                if let Some(action) = production_target_action_for_choice(&control.current, choice)
                {
                    requested_action = Some(action);
                }
                ui.add_space(8.0);
            }
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
            let library_connected = self.blueprint_library.is_connected();
            let save_label = if library_connected {
                "Save as blueprint"
            } else {
                "Blueprint library unavailable"
            };
            if ui
                .add_enabled_ui(library_connected, |ui| {
                    ui.add_sized(
                        [ui.available_width(), 0.0],
                        Button::new(RichText::new(save_label).size(11.0).strong()),
                    )
                })
                .inner
                .clicked()
            {
                self.request_save_as_blueprint();
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
                Button::new(
                    RichText::new(instance_semantic_label(resolved, self.layout.catalog()))
                        .size(11.0),
                )
                .selected(self.selected.contains(id))
                .wrap(),
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

    /// Renders the "BLUEPRINT LIBRARY" sidebar section (spec.md US2/US3):
    /// every valid blueprint's name, module count, and last-saved time
    /// (FR-006), an explicit empty-library indication (FR-007), a safe
    /// generic count of unreadable/duplicate entries (FR-009), and a
    /// per-entry catalog-compatibility indication (FR-010). Reads only the
    /// already-cached `listing` (research.md Decision 2) — never triggers
    /// I/O itself.
    fn blueprint_library_section_ui(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("BLUEPRINT LIBRARY")
                .size(11.0)
                .strong()
                .color(ACCENT),
        );
        ui.add_space(8.0);

        if !self.blueprint_library.is_connected() {
            ui.label(
                RichText::new("Blueprint library unavailable.")
                    .size(12.0)
                    .color(Color32::from_rgb(244, 190, 96)),
            );
            return;
        }

        let listing = self.blueprint_library.listing();

        let mut requested_insertion = None;
        if listing.entries.is_empty() {
            ui.label(
                RichText::new("No blueprints saved yet.")
                    .size(12.0)
                    .color(TEXT_MUTED),
            );
        } else {
            for entry in &listing.entries {
                ui.label(
                    RichText::new(entry.name())
                        .size(12.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                let module_label = if entry.node_count() == 1 {
                    "1 module".to_owned()
                } else {
                    format!("{} modules", entry.node_count())
                };
                ui.label(
                    RichText::new(format!(
                        "{module_label} · saved {}",
                        format_blueprint_timestamp(entry.updated_at())
                    ))
                    .size(11.0)
                    .color(TEXT_MUTED),
                );
                if let Some(mismatch_text) =
                    catalog_compatibility_mismatch_text(entry.compatibility())
                {
                    ui.label(
                        RichText::new(mismatch_text)
                            .size(11.0)
                            .color(Color32::from_rgb(244, 190, 96)),
                    );
                }
                if !entry.interface_names().is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "Interfaces: {}",
                            entry.interface_names().join(", ")
                        ))
                        .size(11.0)
                        .color(TEXT_MUTED),
                    );
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 0.0],
                        Button::new(RichText::new("Insert").size(11.0)),
                    )
                    .clicked()
                {
                    requested_insertion = Some(entry.id().clone());
                }
                ui.add_space(6.0);
            }
        }

        if !listing.invalid_entries.is_empty() {
            ui.add_space(4.0);
            let count = listing.invalid_entries.len();
            let notice = if count == 1 {
                "1 entry could not be read.".to_owned()
            } else {
                format!("{count} entries could not be read.")
            };
            ui.label(
                RichText::new(notice)
                    .size(11.0)
                    .color(Color32::from_rgb(244, 190, 96)),
            );
        }

        if let Some(id) = requested_insertion {
            self.request_insert_blueprint(id);
        }
    }
}
