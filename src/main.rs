//! Desktop QMP controller with shared capture/input and per-game HALO policies.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

mod app;
mod capture;
mod cards;
mod detector;
mod game;
mod geometry;
mod klondike;
mod parameters;
mod pyramid;
mod qmp;
mod session_log;
mod snapshot;
mod stepper;
mod tracker;
mod tripeaks;
mod worker;

use app::QmpQemuSocketApp;
use eframe::egui;
use parameters::{APP_NAME, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, RELEASE_LABEL};


/// Configure the native window and run the controller until its viewport closes.
///
/// Returns the platform error if eframe cannot create or run the native window.
fn main() -> eframe::Result {
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
