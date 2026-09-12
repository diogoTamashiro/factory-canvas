#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod blueprint_library_view;
mod document_session;
mod egui_app;
mod egui_canvas;
mod history;
mod selected_set;

fn main() -> eframe::Result {
    egui_app::run()
}
