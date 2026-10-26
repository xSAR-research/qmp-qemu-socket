//! Live session output in a separate window; visibility never pauses the worker.

use super::*;


impl QmpQemuSocketApp {
    /// Display a live selectable tail, complete-session copy and existing idle-only clear controls.
    /// Closing retains the log and font preference; file history continues while hidden.
    pub(super) fn output_window(&mut self, context: &egui::Context) {
        if !self.show_output {
            return;
        }
        let mut open = self.show_output;
        let mut chrome = std::mem::take(&mut self.output_chrome);
        chrome.show(context, window_chrome::WindowAppearance {
            id: egui::Id::new("live-output-owned-chrome-v139"), title: "LOG",
            colour: egui::Color32::from_rgb(0, 125, 148),
            default_size: crate::parameters::DETAILS_WINDOW_DEFAULT_SIZE_POINTS.into(),
            min_size: crate::parameters::DETAILS_WINDOW_MIN_SIZE_POINTS.into(),
        }, &mut open, |ui| {
            let mut output_text_size_points = self.output_text_size_points.unwrap_or_else(|| {
                egui::TextStyle::Monospace
                    .resolve(ui.style())
                    .size
                    .clamp(MINIMUM_OUTPUT_TEXT_SIZE_POINTS, MAXIMUM_OUTPUT_TEXT_SIZE_POINTS)
            });
            ui.horizontal(|ui| {


                if ui
                    .button("Copy Output")
                    .on_hover_text(
                        "Copy the complete file-backed session output to the clipboard",
                    )
                    .clicked()
                {


                    match self.complete_output() {
                        Ok(output) => ui.ctx().copy_text(output),
                        Err(error) => self.push_log(error),
                    }
                }


                if ui
                    .add_enabled(
                        controls_are_mutable(self.active_run, self.worker_state, self.stop_requested) && !self.show_snapshot_dialog,
                        egui::Button::new("Clear Output"),
                    )
                    .on_hover_text(
                        "Clear the visible log and reset board, game, and row-scan progress; the complete session log is retained",
                    )
                    .clicked()
                {


                    match self.worker.reset_progress(self.current_preview_context()) {
                        Ok(()) => {
                            self.completed_games = 0;
                            self.current_board = 1;
                            self.board_position_established = false;
                            self.current_status = "Ready".to_owned();
                            let clear_result = self.append_session_only(
                                "Visible output was manually cleared; board, game, and row-scan progress were reset; complete session history retained.",
                            );


                            match clear_result {
                                Ok(()) => self.log_lines.clear(),
                                Err(error) => self.push_log(format!(
                                    "Progress tracking was reset, but visible output was retained because {error}."
                                )),
                            }
                        }
                        Err(error) => self.push_log(format!(
                            "Clear Output refused because progress tracking could not be reset: {error}; visible history was retained."
                        )),
                    }
                }
                ui.label("Text size");
                ui.add(
                    egui::DragValue::new(&mut output_text_size_points)
                        .range(MINIMUM_OUTPUT_TEXT_SIZE_POINTS..=MAXIMUM_OUTPUT_TEXT_SIZE_POINTS)
                        .speed(1.0)
                        .suffix(" pt"),
                )
                .on_hover_text("Change only the detailed-output text size; preview scaling and copied text remain unchanged");
            });
            egui::ScrollArea::both()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let output = self.log_lines.join("\n");
                    let mut text = output.as_str();
                    ui.add(egui::TextEdit::multiline(&mut text)
                        .font(FontId::monospace(output_text_size_points))
                        .desired_width(f32::INFINITY));
                });
            self.output_text_size_points = Some(output_text_size_points);
            });
        self.output_chrome = chrome;
        self.show_output = open;
    }
}
