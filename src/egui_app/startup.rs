use super::colors::{ACCENT, ACCENT_DIM, APP_BACKGROUND, BORDER, SIDEBAR_BACKGROUND, TEXT_PRIMARY};
use super::icons::BuildableIcons;
use super::notices::{safe_catalog_load_detail, EditorNotice};
use super::FactoryCanvasApp;
use crate::blueprint_library_view::BlueprintLibraryView;
use crate::document_session::DocumentSession;
use crate::egui_canvas::CanvasState;
use crate::history::EditHistory;
use crate::selected_set::SelectedSet;
use eframe::egui::{self, vec2, Color32, Stroke};
use factory_canvas::catalog_loader::{
    load_catalog_from_directory, load_embedded_public_catalog, CatalogLoadError,
};
use factory_canvas::domain::catalog::Catalog;
use factory_canvas::domain::layout::FactoryLayout;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StartupCatalog {
    pub(super) catalog: Catalog,
    pub(super) warning: Option<String>,
}

pub(super) fn choose_startup_catalog(
    public: Catalog,
    private: Result<Catalog, CatalogLoadError>,
) -> StartupCatalog {
    match private {
        Ok(catalog) => StartupCatalog {
            catalog,
            warning: None,
        },
        Err(error) if error.is_manifest_not_found() => StartupCatalog {
            catalog: public,
            warning: None,
        },
        Err(error) => {
            let detail = safe_catalog_load_detail(&error);
            StartupCatalog {
                catalog: public,
                warning: Some(format!(
                    "Private catalog could not be loaded; using the public catalog. {detail}"
                )),
            }
        }
    }
}

pub(super) fn load_startup_catalog_from_directory(root: &Path) -> StartupCatalog {
    let public =
        load_embedded_public_catalog().expect("versioned embedded public catalog must be valid");
    choose_startup_catalog(public, load_catalog_from_directory(root))
}

pub(super) fn configure_style(context: &egui::Context) {
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

impl FactoryCanvasApp {
    pub(super) fn from_startup_catalog(startup: StartupCatalog) -> Self {
        let base_id = startup.catalog.default_base_id().clone();
        Self {
            layout: FactoryLayout::new(startup.catalog, base_id)
                .expect("selected startup catalog default base must exist"),
            canvas: CanvasState::default(),
            session: DocumentSession::default(),
            blueprint_library: BlueprintLibraryView::new(),
            catalog_warning: startup.warning,
            icons: BuildableIcons::empty(),
            selected_block: None,
            armed_blueprint: None,
            selected: SelectedSet::new(),
            next_entity_id: Some(1),
            history: EditHistory::new(),
            notice: EditorNotice::SelectBlock,
            pending_base_change: None,
            pending_instance_removal: None,
            pending_unsaved_action: None,
            close_confirmed: false,
        }
    }

    pub(super) fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_style(&creation_context.egui_ctx);
        let mut app = Self::from_startup_catalog(load_startup_catalog_from_directory(Path::new(
            "data/catalog",
        )));
        app.icons = BuildableIcons::load(
            &creation_context.egui_ctx,
            app.layout.catalog(),
            Path::new("assets/icons"),
        );
        app.blueprint_library
            .connect_to_default_storage(app.layout.catalog());
        app
    }
}
