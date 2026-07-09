#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod capture;
mod config;
mod editor;
mod fonts;
mod hotkey;
mod output;
mod overlay;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hijessy")
            .with_decorations(false)
            .with_fullscreen(true)
            .with_transparent(true)
            .with_mouse_passthrough(true)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Hijessy",
        options,
        Box::new(|cc| Ok(Box::new(app::HijessyApp::new(cc)))),
    )
}
