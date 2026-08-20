#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod egui_app;
mod egui_canvas;

fn main() -> eframe::Result {
    egui_app::run()
}
