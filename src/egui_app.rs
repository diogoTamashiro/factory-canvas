use eframe::egui::{
    self, pos2, vec2, Align, Align2, Button, CentralPanel, Color32, FontId, Frame, Layout, Margin,
    Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use factory_canvas::domain::base::{BaseTemplate, SecondaryLevel};
use factory_canvas::domain::geometry::GridSize;
use factory_canvas::domain::layout::FactoryLayout;

const APP_BACKGROUND: Color32 = Color32::from_rgb(8, 13, 20);
const HEADER_BACKGROUND: Color32 = Color32::from_rgb(11, 18, 28);
const SIDEBAR_BACKGROUND: Color32 = Color32::from_rgb(13, 22, 33);
const CANVAS_BACKGROUND: Color32 = Color32::from_rgb(10, 17, 26);
const GRID_BACKGROUND: Color32 = Color32::from_rgb(15, 29, 41);
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

fn fitted_grid_rect(available: Rect, bounds: GridSize) -> Rect {
    let tile_size = (available.width() / f32::from(bounds.width()))
        .min(available.height() / f32::from(bounds.height()));
    let size = vec2(
        tile_size * f32::from(bounds.width()),
        tile_size * f32::from(bounds.height()),
    );

    Rect::from_center_size(available.center(), size)
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

fn configure_style(context: &egui::Context) {
    context.set_theme(egui::Theme::Dark);
    context.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = vec2(8.0, 10.0);
        style.spacing.button_padding = vec2(12.0, 10.0);
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = APP_BACKGROUND;
        style.visuals.window_fill = APP_BACKGROUND;
        style.visuals.faint_bg_color = SIDEBAR_BACKGROUND;
        style.visuals.extreme_bg_color = CANVAS_BACKGROUND;
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

struct FactoryCanvasApp {
    layout: FactoryLayout,
}

impl Default for FactoryCanvasApp {
    fn default() -> Self {
        Self {
            layout: FactoryLayout::new(BaseTemplate::MainCurrent),
        }
    }
}

impl FactoryCanvasApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_style(&creation_context.egui_ctx);
        Self::default()
    }

    fn select_base(&mut self, template: BaseTemplate) {
        self.layout = FactoryLayout::new(template);
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
        ui.add_space(18.0);

        let current_template = self.layout.base_template();
        let mut requested_template = None;

        for template in BaseTemplate::ALL {
            let selected = current_template == template;
            let label = RichText::new(base_option_label(template))
                .size(12.0)
                .strong()
                .color(if selected { ACCENT } else { TEXT_PRIMARY });
            let response = ui.add_sized(
                [ui.available_width(), 44.0],
                Button::new(label).selected(selected),
            );

            if response.clicked() {
                requested_template = Some(template);
            }
        }

        if let Some(template) = requested_template {
            self.select_base(template);
        }

        ui.add_space(22.0);
        ui.separator();
        ui.add_space(12.0);
        ui.label(
            RichText::new("ESTADO DO EDITOR")
                .size(10.0)
                .strong()
                .color(TEXT_MUTED),
        );
        ui.add_space(6.0);
        ui.label(RichText::new("Canvas vazio").size(13.0).color(TEXT_PRIMARY));
        ui.label(
            RichText::new("Nenhum bloco posicionado.")
                .size(11.0)
                .color(TEXT_MUTED),
        );
    }

    fn canvas_ui(&self, ui: &mut Ui) {
        let available_size = ui.available_size().max(Vec2::splat(1.0));
        let (response, painter) = ui.allocate_painter(available_size, Sense::hover());
        let outer_rect = response.rect;

        painter.rect_filled(outer_rect, 12, CANVAS_BACKGROUND);
        painter.rect_stroke(outer_rect, 12, Stroke::new(1.0, BORDER), StrokeKind::Inside);

        let template = self.layout.base_template();
        let bounds = self.layout.bounds();
        let title_position = pos2(outer_rect.left() + 24.0, outer_rect.top() + 22.0);
        painter.text(
            title_position,
            Align2::LEFT_TOP,
            base_name(template),
            FontId::proportional(18.0),
            TEXT_PRIMARY,
        );
        painter.text(
            pos2(title_position.x, title_position.y + 25.0),
            Align2::LEFT_TOP,
            format!("{} × {} tiles", bounds.width(), bounds.height()),
            FontId::proportional(11.0),
            TEXT_MUTED,
        );

        let mut grid_available = outer_rect.shrink2(vec2(36.0, 32.0));
        grid_available.min.y += 54.0;
        let grid_rect = fitted_grid_rect(grid_available, bounds);

        painter.rect_filled(grid_rect, 2, GRID_BACKGROUND);
        let grid_minor = Color32::from_rgba_unmultiplied(88, 120, 135, 44);
        let grid_major = Color32::from_rgba_unmultiplied(91, 221, 199, 94);

        for x in 0..=bounds.width() {
            let fraction = f32::from(x) / f32::from(bounds.width());
            let screen_x = egui::lerp(grid_rect.left()..=grid_rect.right(), fraction);
            let stroke = if x % 10 == 0 {
                Stroke::new(1.0, grid_major)
            } else {
                Stroke::new(0.5, grid_minor)
            };
            painter.line_segment(
                [
                    pos2(screen_x, grid_rect.top()),
                    pos2(screen_x, grid_rect.bottom()),
                ],
                stroke,
            );
        }

        for y in 0..=bounds.height() {
            let fraction = f32::from(y) / f32::from(bounds.height());
            let screen_y = egui::lerp(grid_rect.top()..=grid_rect.bottom(), fraction);
            let stroke = if y % 10 == 0 {
                Stroke::new(1.0, grid_major)
            } else {
                Stroke::new(0.5, grid_minor)
            };
            painter.line_segment(
                [
                    pos2(grid_rect.left(), screen_y),
                    pos2(grid_rect.right(), screen_y),
                ],
                stroke,
            );
        }

        painter.rect_stroke(grid_rect, 2, Stroke::new(1.5, ACCENT), StrokeKind::Inside);
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
            .show(ui, |ui| self.sidebar_ui(ui));

        CentralPanel::default()
            .frame(Frame::new().fill(APP_BACKGROUND).inner_margin(20))
            .show(ui, |ui| self.canvas_ui(ui));
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui::{pos2, vec2, Rect};
    use factory_canvas::domain::base::{BaseTemplate, SecondaryLevel};
    use factory_canvas::domain::geometry::GridSize;

    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn fitted_grid_rect_stays_centered_and_preserves_aspect_ratio() {
        let cases = [
            (
                Rect::from_min_size(pos2(10.0, 20.0), vec2(500.0, 1_000.0)),
                GridSize::new(80, 40).unwrap(),
            ),
            (
                Rect::from_min_size(pos2(40.0, 10.0), vec2(1_000.0, 500.0)),
                GridSize::new(40, 80).unwrap(),
            ),
        ];

        for (available, bounds) in cases {
            let fitted = fitted_grid_rect(available, bounds);

            assert_close(fitted.center().x, available.center().x);
            assert_close(fitted.center().y, available.center().y);
            assert!(fitted.left() >= available.left());
            assert!(fitted.right() <= available.right());
            assert!(fitted.top() >= available.top());
            assert!(fitted.bottom() <= available.bottom());
            assert_close(
                fitted.width() / fitted.height(),
                f32::from(bounds.width()) / f32::from(bounds.height()),
            );
        }
    }

    #[test]
    fn base_labels_use_confirmed_names_and_derived_dimensions() {
        let labels = BaseTemplate::ALL.map(base_option_label);

        assert_eq!(
            labels,
            [
                "PAC Principal · 80 × 80",
                "Sub-PAC Padrão · 30 × 30",
                "Sub-PAC Expansão I · 40 × 40",
                "Sub-PAC Expansão II · 50 × 50",
            ]
        );
    }

    #[test]
    fn app_starts_with_main_base_layout() {
        let app = FactoryCanvasApp::default();

        assert_eq!(app.layout.base_template(), BaseTemplate::MainCurrent);
        assert_eq!(app.layout.bounds(), GridSize::new(80, 80).unwrap());
        assert!(app.layout.is_empty());
    }

    #[test]
    fn selecting_base_replaces_empty_layout_with_selected_template() {
        let mut app = FactoryCanvasApp::default();
        let templates = [
            BaseTemplate::Secondary(SecondaryLevel::Standard),
            BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI),
            BaseTemplate::Secondary(SecondaryLevel::AreaExpansionII),
            BaseTemplate::MainCurrent,
        ];

        for template in templates {
            app.select_base(template);

            assert_eq!(app.layout.base_template(), template);
            assert_eq!(app.layout.bounds(), template.bounds());
            assert!(app.layout.is_empty());
        }
    }
}
