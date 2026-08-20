use eframe::egui::{
    self, vec2, Align, Button, CentralPanel, Color32, Frame, Layout, Margin, RichText, Sense,
    Stroke, Ui, Vec2,
};
use factory_canvas::domain::base::{BaseTemplate, SecondaryLevel};
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout, PlacementError};

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

fn base_name(template: BaseTemplate) -> &'static str {
    match template {
        BaseTemplate::MainCurrent => "PAC Principal",
        BaseTemplate::Secondary(SecondaryLevel::Standard) => "Sub-PAC Padrão",
        BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI) => "Sub-PAC Expansão I",
        BaseTemplate::Secondary(SecondaryLevel::AreaExpansionII) => "Sub-PAC Expansão II",
    }
}

fn base_option_label(template: BaseTemplate) -> String {
    let bounds = template.bounds();
    format!(
        "{} · {} × {}",
        base_name(template),
        bounds.width(),
        bounds.height()
    )
}

fn block_option_label(template: BlockTemplate) -> String {
    let definition = template.definition();
    let footprint = definition.footprint();
    format!(
        "{} · {} × {}",
        definition.display_name(),
        footprint.width(),
        footprint.height()
    )
}

fn notice_text(notice: EditorNotice) -> String {
    match notice {
        EditorNotice::SelectBlock => "Selecione um bloco para começar.".to_owned(),
        EditorNotice::ReadyToPlace { template } => format!(
            "Bloco selecionado: {}. Clique no grid para posicionar.",
            template.definition().display_name()
        ),
        EditorNotice::Placed {
            id,
            template,
            origin,
        } => format!(
            "Bloco #{} posicionado em ({}, {}): {}.",
            id.value(),
            origin.x,
            origin.y,
            template.definition().display_name()
        ),
        EditorNotice::PlacementRejected(PlacementError::DuplicateEntityId { id }) => {
            format!("O ID interno #{} já está em uso.", id.value())
        }
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds { .. }) => {
            "O bloco não cabe nessa posição.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::Collision { conflicting_id, .. }) => {
            format!("Posição ocupada pelo bloco #{}.", conflicting_id.value())
        }
        EditorNotice::EntityIdsExhausted => "Não há IDs disponíveis para novos blocos.".to_owned(),
        EditorNotice::BaseChanged { template } => {
            format!("Base alterada para {}.", base_name(template))
        }
    }
}

fn layout_count_label(count: usize) -> String {
    match count {
        0 => "Nenhum bloco posicionado".to_owned(),
        1 => "1 bloco posicionado".to_owned(),
        _ => format!("{count} blocos posicionados"),
    }
}

fn instance_semantic_label(instance: BlockInstance) -> String {
    let origin = instance.origin();
    let rotation = match instance.rotation() {
        Rotation::Zero => 0,
        Rotation::Clockwise90 => 90,
        Rotation::Clockwise180 => 180,
        Rotation::Clockwise270 => 270,
    };
    let footprint = instance
        .rotation()
        .apply_to(instance.template().definition().footprint());

    format!(
        "#{} · {} · origem ({}, {}) · {} × {} · {}°",
        instance.id().value(),
        instance.template().definition().display_name(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorNotice {
    SelectBlock,
    ReadyToPlace {
        template: BlockTemplate,
    },
    Placed {
        id: EntityId,
        template: BlockTemplate,
        origin: GridPoint,
    },
    PlacementRejected(PlacementError),
    EntityIdsExhausted,
    BaseChanged {
        template: BaseTemplate,
    },
}

struct FactoryCanvasApp {
    layout: FactoryLayout,
    selected_block: Option<BlockTemplate>,
    next_entity_id: Option<u64>,
    notice: EditorNotice,
    pending_base_change: Option<BaseTemplate>,
}

impl Default for FactoryCanvasApp {
    fn default() -> Self {
        Self {
            layout: FactoryLayout::new(BaseTemplate::MainCurrent),
            selected_block: None,
            next_entity_id: Some(1),
            notice: EditorNotice::SelectBlock,
            pending_base_change: None,
        }
    }
}

impl FactoryCanvasApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_style(&creation_context.egui_ctx);
        Self::default()
    }

    fn replace_base(&mut self, template: BaseTemplate) {
        self.layout = FactoryLayout::new(template);
        self.next_entity_id = Some(1);
        self.pending_base_change = None;
        self.notice = EditorNotice::BaseChanged { template };
    }

    fn request_base_change(&mut self, template: BaseTemplate) {
        if template == self.layout.base_template() {
            self.pending_base_change = None;
        } else if self.layout.is_empty() {
            self.replace_base(template);
        } else {
            self.pending_base_change = Some(template);
        }
    }

    fn cancel_base_change(&mut self) {
        self.pending_base_change = None;
    }

    fn confirm_base_change(&mut self) {
        if let Some(template) = self.pending_base_change {
            self.replace_base(template);
        }
    }

    fn select_block(&mut self, template: BlockTemplate) {
        self.selected_block = Some(template);
        self.notice = EditorNotice::ReadyToPlace { template };
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
        let instance = BlockInstance::new(id, template, origin, Rotation::Zero);

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
                RichText::new("EDITOR DE LAYOUT")
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

    fn sidebar_ui(&mut self, ui: &mut Ui) {
        self.base_picker_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.block_palette_ui(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.editor_state_ui(ui);
    }

    fn base_picker_ui(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("BASE DE CONSTRUÇÃO")
                .size(11.0)
                .strong()
                .color(ACCENT),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Escolha a área confirmada para o layout.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.add_space(10.0);

        let current_template = self.layout.base_template();
        let mut requested_template = None;

        for template in BaseTemplate::ALL {
            let selected = current_template == template;
            let label = RichText::new(base_option_label(template))
                .size(12.0)
                .strong()
                .color(if selected { ACCENT } else { TEXT_PRIMARY });
            let response = ui.add_sized(
                [ui.available_width(), 40.0],
                Button::new(label).selected(selected),
            );

            if response.clicked() {
                requested_template = Some(template);
            }
        }

        if let Some(template) = requested_template {
            self.request_base_change(template);
        }
    }

    fn block_palette_ui(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("BLOCOS").size(11.0).strong().color(ACCENT));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Selecione e clique no tile de origem.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
        ui.add_space(10.0);

        let mut requested_block = None;

        for template in BlockTemplate::ALL {
            let selected = self.selected_block == Some(template);
            let label = RichText::new(block_option_label(template))
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

    fn editor_state_ui(&self, ui: &mut Ui) {
        ui.label(
            RichText::new("ESTADO DO EDITOR")
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
        let notice_color = match self.notice {
            EditorNotice::PlacementRejected(_) | EditorNotice::EntityIdsExhausted => {
                Color32::from_rgb(245, 132, 124)
            }
            _ => TEXT_MUTED,
        };
        ui.label(
            RichText::new(notice_text(self.notice))
                .size(11.0)
                .color(notice_color),
        );

        if self.layout.is_empty() {
            return;
        }

        ui.add_space(14.0);
        ui.label(
            RichText::new("INSTÂNCIAS NO CANVAS")
                .size(10.0)
                .strong()
                .color(TEXT_MUTED),
        );
        ui.add_space(4.0);

        for instance in self.layout.instances().copied() {
            ui.label(
                RichText::new(instance_semantic_label(instance))
                    .size(11.0)
                    .color(TEXT_PRIMARY),
            );
        }
    }

    fn canvas_ui(&mut self, ui: &mut Ui) {
        let template = self.layout.base_template();
        let clicked_point = crate::egui_canvas::show(
            ui,
            &self.layout,
            base_name(template),
            self.selected_block.is_some(),
        );

        if let Some(origin) = clicked_point {
            self.place_selected_at(origin);
            ui.ctx().request_repaint();
        }
    }

    fn base_change_modal(&mut self, context: &egui::Context) {
        let Some(target) = self.pending_base_change else {
            return;
        };
        let instance_count = self.layout.len();
        let removal_text = if instance_count == 1 {
            "1 bloco será removido".to_owned()
        } else {
            format!("{instance_count} blocos serão removidos")
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
                ui.heading("Trocar base e limpar o layout?");
                ui.add_space(8.0);
                ui.label(format!(
                    "A nova base será {}. {removal_text}.",
                    base_name(target),
                ));
                ui.add_space(16.0);

                let mut action = None;
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        action = Some(false);
                    }
                    if ui
                        .add(
                            Button::new("Trocar e limpar")
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

        egui::Panel::left("base_sidebar")
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
                    .show(ui, |ui| self.sidebar_ui(ui));
            });

        CentralPanel::default()
            .frame(Frame::new().fill(APP_BACKGROUND).inner_margin(20))
            .show(ui, |ui| self.canvas_ui(ui));

        self.base_change_modal(ui.ctx());
    }
}

#[cfg(test)]
#[path = "egui_app_tests.rs"]
mod tests;
