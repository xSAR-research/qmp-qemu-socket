//! Persistent, read-only failure presentation, independent of controller authority.

use super::*;
use crate::pyramid_diagnostics::PyramidDiagnostic;


/// Identity of an accepted failure; packet updates must not reopen a closed window.
#[derive(Clone, PartialEq)]
struct FailureKey {
    /// Controller selection that originally owned the failed request.
    context: ControllerContext,
    /// Worker phase that failed.
    phase: PyramidPhase,
    /// Automatic execution identity, absent for an independent local failure.
    run_id: Option<u64>,
    /// Local identity retained through asynchronous packet persistence.
    ui_failure_id: Option<u64>,
}


/// Latest failure and frozen request fields, retained when the workflow resets.
#[derive(Default)]
pub(super) struct FailureWindow {
    /// Session-only visibility; closing retains the record for reopening.
    open: bool,
    /// Presentation geometry survives closing and new diagnostic receipts.
    chrome: window_chrome::WindowChrome,
    /// Accepted identity, absent for general application errors.
    key: Option<FailureKey>,
    /// Original request fields; later settings changes must not replace them.
    fields: serde_json::Value,
    /// Complete selectable/copyable text, including persistence updates.
    text: String,
}


impl FailureWindow {
    /// Retain an accepted Pyramid failure; refresh matching packet metadata without reopening.
    pub(super) fn receive(
        &mut self,
        diagnostic: &PyramidDiagnostic,
        fields: serde_json::Value,
        new_failure: bool,
    ) {
        let key = FailureKey {
            context: diagnostic.context.clone(),
            phase: diagnostic.phase,
            run_id: if diagnostic.ui_failure_id.is_some() {
                None
            } else {
                diagnostic.run_id
            },
            ui_failure_id: diagnostic.ui_failure_id,
        };
        if new_failure || self.key.as_ref() != Some(&key) {
            self.fields = fields;
            self.open = true;
        }
        self.key = Some(key);
        let directory = diagnostic.directory.as_ref().map_or_else(
            || "Unavailable or persistence pending".to_owned(),
            |path| path.display().to_string(),
        );
        self.text = format!(
            "Application version: {}\nStep: {:?}\nRequest: {:?}\nLocal failure: {:?}\n\n{}\n\nSaved failure packet: {}\nPersistence error: {}\n\nFields at the requested step:\n{}\n\nFailed or stopped routes cannot resume. Inspect and acquire a fresh board before another execution.",
            env!("CARGO_PKG_VERSION"),
            diagnostic.phase,
            diagnostic.run_id,
            diagnostic.ui_failure_id,
            diagnostic.error,
            directory,
            diagnostic.persistence_error.as_deref().unwrap_or("None"),
            serde_json::to_string_pretty(&self.fields)
                .unwrap_or_else(|error| format!("Could not format request fields: {error}")),
        );
    }


    /// Retain a distinct general application failure with its current configuration snapshot.
    pub(super) fn general(&mut self, label: &str, error: &str, fields: serde_json::Value) {
        self.key = None;
        self.fields = fields;
        self.text = format!(
            "Application version: {}\nStep: {label}\n\n{error}\n\nConfiguration at failure:\n{}",
            env!("CARGO_PKG_VERSION"),
            serde_json::to_string_pretty(&self.fields)
                .unwrap_or_else(|error| format!("Could not format configuration: {error}")),
        );
        self.open = true;
    }


    /// Replace stale game details from immutable worker source facts, without UI settings or input.
    pub(super) fn worker_run(
        &mut self,
        failure: &crate::worker::WorkerRunFailure,
        context: Option<&ControllerContext>,
        emitted_at: Duration,
    ) {
        let fields = serde_json::json!({
            "game_mode": format!("{:?}", failure.game_mode),
            "source_context": context.map(|value| format!("{value:#?}")),
            "run_settings": format!("{:#?}", failure.settings),
            "requested_operations": failure.settings.operation_limit(),
            "run_started_seconds": failure.started_at.map(|value| value.as_secs_f64()),
            "failed_at_seconds": emitted_at.as_secs_f64(),
            "guest_input_attempted": failure.input_attempted,
        });
        self.general(
            &format!("{:?} run stopped", failure.game_mode),
            &failure.error,
            fields,
        );
    }


    /// Whether a complete last failure remains available, including after STOP or a new run.
    pub(super) fn available(&self) -> bool {
        !self.text.is_empty()
    }


    /// Reopen the retained record without dispatching requests or changing workflow state.
    pub(super) fn reopen(&mut self) {
        self.open = self.available();
    }


    /// Render selectable immutable text in a resizable window; never perform worker I/O.
    pub(super) fn show(&mut self, context: &egui::Context) {
        if !self.open {
            return;
        }
        self.chrome.show(
            context,
            window_chrome::WindowAppearance {
                id: egui::Id::new("failure-owned-chrome-v139"),
                title: "ANALYSIS",
                colour: egui::Color32::from_rgb(196, 76, 0),
                default_size: crate::parameters::DETAILS_WINDOW_DEFAULT_SIZE_POINTS.into(),
                min_size: crate::parameters::DETAILS_WINDOW_MIN_SIZE_POINTS.into(),
            },
            &mut self.open,
            |ui| {
                if ui.button("Copy all").clicked() {
                    ui.ctx().copy_text(self.text.clone());
                }
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut text = self.text.as_str();
                        ui.add(
                            egui::TextEdit::multiline(&mut text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        );
                    });
            },
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Packet completion preserves closed state and original fields; a new failure opens again.
    #[test]
    fn diagnostics_preserve_failure_fields_and_only_open_for_new_identity() {
        let mut window = FailureWindow::default();
        let mut diagnostic = PyramidDiagnostic {
            context: ControllerContext::new(
                "/tmp/test.sock".into(),
                GameMode::Pyramid,
                SolvingStrategy::ShortestPath,
                1,
            ),
            phase: PyramidPhase::Execution,
            run_id: Some(7),
            ui_failure_id: Some(1),
            error: "Identical failure text".to_owned(),
            directory: None,
            persistence_error: None,
        };
        window.receive(
            &diagnostic,
            serde_json::json!({"socket":"original", "session_log":"original.log"}),
            true,
        );
        assert!(window.open);
        window.open = false;
        diagnostic.directory = Some("/tmp/retained-packet".into());
        window.receive(&diagnostic, serde_json::json!({"socket":"edited"}), false);
        assert!(!window.open);
        assert!(window.text.contains("original.log"));
        assert!(window.text.contains("retained-packet"));
        assert!(!window.text.contains("edited"));
        window.reopen();
        assert!(window.open);
        window.open = false;
        diagnostic.ui_failure_id = Some(2);
        window.receive(
            &diagnostic,
            serde_json::json!({"socket":"new failure"}),
            true,
        );
        assert!(window.open);
        assert!(window.text.contains("new failure"));
    }


    /// A failed persistence submission updates retained details without replacing fields or reopening.
    #[test]
    fn diagnostics_display_persistence_failure_after_initial_presentation() {
        let mut window = FailureWindow::default();
        let mut diagnostic = PyramidDiagnostic {
            context: ControllerContext::new(
                "/tmp/test.sock".into(),
                GameMode::Pyramid,
                SolvingStrategy::ShortestPath,
                1,
            ),
            phase: PyramidPhase::Reconnaissance,
            run_id: None,
            ui_failure_id: Some(3),
            error: "Original acquisition refusal".to_owned(),
            directory: None,
            persistence_error: None,
        };
        window.receive(
            &diagnostic,
            serde_json::json!({"session_log":"frozen.log"}),
            true,
        );
        window.open = false;
        diagnostic.persistence_error = Some("Diagnostic worker unavailable".to_owned());
        window.receive(
            &diagnostic,
            serde_json::json!({"session_log":"later.log"}),
            false,
        );
        assert!(!window.open);
        assert!(window.text.contains("Original acquisition refusal"));
        assert!(window.text.contains("Diagnostic worker unavailable"));
        assert!(window.text.contains("frozen.log"));
        assert!(!window.text.contains("later.log"));
    }

    /// Current Spider source facts replace a retained Klondike error and survive reopening.
    #[test]
    fn native_150_spider_failure_replaces_old_game_fields() {
        let mut window = FailureWindow::default();
        window.general(
            "Klondike",
            "old Klondike failure",
            serde_json::json!({"old": true}),
        );
        let failure = crate::worker::WorkerRunFailure {
            game_mode: GameMode::Spider,
            settings: StepRunSettings::new(AnimationSettleDelays::default(), 0),
            started_at: Some(Duration::from_secs(12)),
            input_attempted: true,
            error: "OK missing-current-caption-glyph; no retry".to_owned(),
        };
        let context = ControllerContext::new(
            "/tmp/spider.sock".into(),
            GameMode::Spider,
            SolvingStrategy::ComputerVision,
            15,
        );
        window.worker_run(&failure, Some(&context), Duration::from_secs(22));
        assert!(window.open);
        assert_eq!(window.fields["game_mode"], "Spider");
        assert_eq!(window.fields["run_started_seconds"], 12.0);
        assert_eq!(window.fields["failed_at_seconds"], 22.0);
        assert_eq!(window.fields["guest_input_attempted"], true);
        assert!(window.text.contains("missing-current-caption-glyph"));
        assert!(window.text.contains("spider.sock"));
        assert!(!window.text.contains("Klondike"));
        assert!(!window.text.contains("old Klondike failure"));
        let frozen = window.text.clone();
        window.open = false;
        window.reopen();
        assert!(window.open);
        assert_eq!(window.text, frozen);
    }


    /// A current Klondike stop replaces the retained A* packet and survives reopening.
    #[test]
    fn native_154_klondike_failure_replaces_pyramid_astar_fields() {
        let mut window = FailureWindow::default();
        let old = PyramidDiagnostic {
            context: ControllerContext::new(
                "/tmp/old-pyramid.sock".into(),
                GameMode::Pyramid,
                SolvingStrategy::ShortestPath,
                1,
            ),
            phase: PyramidPhase::Execution,
            run_id: Some(2),
            ui_failure_id: None,
            error: "old A* execution observation expired".to_owned(),
            directory: Some("/tmp/old-pyramid-packet".into()),
            persistence_error: None,
        };
        window.receive(
            &old,
            serde_json::json!({
                "game": "Pyramid", "strategy": "A* Route Solution",
                "route_method": "Weighted A* (weight 2)",
            }),
            true,
        );
        assert!(window.key.is_some());
        let error = "Klondike terminal advancement exhausted 3 delayed read-only captures after Some(ScoreCounting); the last control was not retried";
        let failure = crate::worker::WorkerRunFailure {
            game_mode: GameMode::Klondike,
            settings: StepRunSettings::new(AnimationSettleDelays::default(), 0),
            started_at: Some(Duration::from_secs(10)),
            input_attempted: true,
            error: error.to_owned(),
        };
        let context = ControllerContext::new(
            "/tmp/current-klondike.sock".into(),
            GameMode::Klondike,
            SolvingStrategy::ComputerVision,
            3,
        );
        window.worker_run(&failure, Some(&context), Duration::from_secs(20));
        assert!(window.open && window.key.is_none());
        assert_eq!(window.fields["game_mode"], "Klondike");
        assert_eq!(window.fields["requested_operations"], 0);
        assert_eq!(window.fields["guest_input_attempted"], true);
        assert!(window.text.contains(error));
        assert!(window.text.contains("ComputerVision"));
        assert!(window.text.contains("current-klondike.sock"));
        for stale in ["old-pyramid", "old A*", "A* Route Solution", "Weighted A*"] {
            assert!(!window.text.contains(stale));
        }
        let frozen = window.text.clone();
        window.open = false;
        window.reopen();
        assert!(window.open);
        assert_eq!(window.text, frozen);
    }

}
