mod app;
mod capture;
mod card_reader;
mod cards;
mod detector;
mod game;
mod geometry;
mod parameters;
mod pyramid;
mod qmp;
mod snapshot;
mod stepper;
mod tracker;
mod tripeaks;
mod worker;

use app::QmpQemuSocketApp;
use eframe::egui;
use parameters::{APP_NAME, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, RELEASE_LABEL};

fn main() -> eframe::Result {
    // Configure the native window and launch the application.
    let window_title = format!("{APP_NAME} — {RELEASE_LABEL}");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&window_title)
            .with_inner_size([INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT])
            .with_min_inner_size([840.0, 600.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        &window_title,
        native_options,
        Box::new(|creation_context| Ok(Box::new(QmpQemuSocketApp::new(creation_context)))),
    )
}
