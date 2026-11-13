//! Thin native desktop entrypoint for the `qmp-qemu-socket` controller.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use eframe::egui;
use qmp_qemu_socket::{
    APP_NAME, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, QmpQemuSocketApp, RELEASE_LABEL,
    image_diagnostics::{self, ImageDiagnostics, StartupOptions},
    shutdown::ShutdownRequest,
};


/// Configure the native window and run the controller until its viewport closes.
///
/// Returns the platform error if eframe cannot create or run the native window.
fn main() -> eframe::Result {
    let options =
        image_diagnostics::parse_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{APP_NAME}: {error}\nUsage: qmp-qemu-socket [--no-image]");
            std::process::exit(2);
        });
    let images = match options {
        StartupOptions::Run { images } => images,
        StartupOptions::Help => {
            println!(
                "Usage: qmp-qemu-socket [--no-image]\n\nSave GAME WIN sequence PNGs in /home/charlie/tmp/images by default.\nSuccessful ordinary gameplay captures are not saved; Pyramid failures may retain existing evidence in private error packets.\n\n--no-image  Disable GAME WIN image storage.\nEach application start removes that directory; default startup recreates it.\n--help      Show this help without starting the application."
            );
            return Ok(());
        }
    };
    let diagnostics = ImageDiagnostics::prepare(images).unwrap_or_else(|error| {
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
                ShutdownRequest::install()?,
            )))
        }),
    )
}
