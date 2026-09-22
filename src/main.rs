//! ArtFlow — a modern, professional image editor and digital drawing studio.
//!
//! Entry point. Wires up logging, then hands off to the eframe application.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod color;
mod document;
mod filters;
mod io;
mod render;
mod tools;
mod ui;

use app::ArtFlowApp;
use log::info;

fn main() -> eframe::Result<()> {
    // Pretty logs but not too chatty.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("ArtFlow starting up");

    let viewport = egui::ViewportBuilder::default()
        .with_title("ArtFlow — Creative Studio")
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([1100.0, 700.0])
        .with_app_id("studio.artflow.app");

    let options = eframe::NativeOptions {
        viewport,
        vsync: true,
        multisampling: 4,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    eframe::run_native(
        "ArtFlow",
        options,
        Box::new(|cc| {
            // Inject our fonts & theme.
            app::configure_egui(&cc.egui_ctx);
            Ok(Box::new(ArtFlowApp::new(cc)))
        }),
    )
}