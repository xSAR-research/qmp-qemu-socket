//! Desktop QMP controller with shared capture/input and per-game HALO policies.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

mod app;
mod capture;
mod cards;
mod detector;
mod freecell;
mod freecell_terminal;
mod game;
mod geometry;
mod image_diagnostics;
mod image_evidence;
mod klondike;
mod parameters;
mod pyramid;
mod pyramid_diagnostics;
mod pyramid_effect;
mod pyramid_progress;
mod pyramid_recon;
mod pyramid_search_policy;
mod pyramid_workflow_model;
mod qmp;
mod session_log;
mod shutdown;
mod snapshot;
mod spider;
mod spider_terminal;
mod stepper;
mod strategy;
mod terminal_controls;
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
    let options =
        image_diagnostics::parse_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{APP_NAME}: {error}\nUsage: qmp-qemu-socket [--image]");
            std::process::exit(2);
        });
    let images = match options {
        image_diagnostics::StartupOptions::Run { images } => images,
        image_diagnostics::StartupOptions::Help => {
            println!(
                "Usage: qmp-qemu-socket [--image]\n\n--image  Save GAME WIN sequence PNGs in /home/charlie/tmp/images.\n         Successful ordinary gameplay captures are not saved; Pyramid failures may retain existing evidence in private error packets.\n\nEach application start removes that directory; --image recreates it.\n--help   Show this help without starting the application."
            );
            return Ok(());
        }
    };
    let diagnostics =
        image_diagnostics::ImageDiagnostics::prepare(images).unwrap_or_else(|error| {
            eprintln!("{APP_NAME}: cannot reset /home/charlie/tmp/images: {error}");
            std::process::exit(1);
        });
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
        Box::new(move |creation_context| {
            Ok(Box::new(QmpQemuSocketApp::new(
                creation_context,
                diagnostics,
                shutdown::ShutdownRequest::install()?,
            )))
        }),
    )
}
