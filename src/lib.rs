//! Reusable Solitaire QMP controller, capture/recognition observers and planning adapters.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod app;
pub mod capture;
mod cards;
mod detector;
mod freecell;
pub mod freecell_recon;
pub mod freecell_route;
mod freecell_terminal;
mod game;
pub mod game_controls;
pub mod game_operations;
pub mod geometry;
pub mod image_diagnostics;
mod image_evidence;
mod klondike;
mod parameters;
mod pyramid;
mod pyramid_diagnostics;
mod pyramid_effect;
mod pyramid_progress;
pub mod pyramid_recon;
pub mod pyramid_search_policy;
mod pyramid_workflow_model;
pub mod qmp;
mod session_log;
pub mod shutdown;
mod single_step_telemetry;
mod snapshot;
mod spider;
mod spider_terminal;
mod stepper;
pub mod strategy;
mod terminal_controls;
mod tracker;
mod tripeaks;
mod ui_style;
mod worker;

pub use app::QmpQemuSocketApp;
pub use game::GameMode;
pub use parameters::{APP_NAME, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, RELEASE_LABEL};
pub use qmp::{QmpClient, QmpError, QmpProbe, SupportedKey};

#[cfg(test)]
mod tests {
    //! Crate-root API export and startup argument smoke checks.

    use super::{
        APP_NAME, GameMode, RELEASE_LABEL,
        game_controls::{LocationKind, LocationRequest},
        image_diagnostics::{StartupOptions, parse_args},
    };
    use std::ffi::OsString;

    /// Confirm startup flag parsing validates `--no-image`, `--help` and rejections cleanly.
    #[test]
    fn startup_args_validate_cleanly_at_library_root() {
        assert_eq!(
            parse_args(std::iter::empty::<OsString>()),
            Ok(StartupOptions::Run { images: true })
        );
        assert_eq!(
            parse_args([OsString::from("--no-image")]),
            Ok(StartupOptions::Run { images: false })
        );
        assert_eq!(
            parse_args([OsString::from("--help")]),
            Ok(StartupOptions::Help)
        );
        assert!(parse_args([OsString::from("--unknown-flag")]).is_err());
    }

    /// Confirm crate-root re-exports and game-control location selectors remain intact.
    #[test]
    fn crate_root_exports_match_supported_modes_and_locations() {
        assert_eq!(APP_NAME, "qmp-qemu-socket");
        assert!(RELEASE_LABEL.starts_with('v'));
        assert_eq!(GameMode::AVAILABLE.len(), 5);
        let request = LocationRequest::freecell_play(3, 5);
        assert_eq!(request.game, GameMode::FreeCell);
        assert_eq!(request.kind, LocationKind::Play);
        assert_eq!(request.column, 3);
        assert_eq!(request.row, 5);
    }
}
