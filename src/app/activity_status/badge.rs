//! Fixed geometry and nonblocking activity glow for the main-window indicator.

use std::time::Duration;

use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Rect, Sense, Stroke};


/// Constant face size in logical points, independent of the current label.
pub(crate) const FACE_SIZE: egui::Vec2 = egui::vec2(240.0, 80.0);


/// Reserved space around the face so the glow cannot overlap other controls.
const HALO_MARGIN: f32 = 10.0;


/// Named bold font family used only by the activity label.
const BOLD_FAMILY: &str = "activity-status-bold";


/// User-facing activity states; these have no worker-control behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Status {
    /// No active work, acknowledged pause or operator STOP.
    Waiting,
    /// Acquiring or checking fresh image evidence.
    Recon,
    /// Calculating a route without delivering guest input.
    Route,
    /// Delivering and observing supported gameplay operations.
    Solve,
    /// Current failure or uncertain outcome needs operator inspection.
    Error,
}


impl Status {
    /// Return the exact fixed label requested for the current activity.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Waiting => "WAITING",
            Self::Recon => "RECON",
            Self::Route => "ROUTE",
            Self::Solve => "SOLVING",
            Self::Error => "ERROR",
        }
    }


    /// Return the solid face colour; pulsing affects only the surrounding glow.
    const fn colour(self) -> Color32 {
        match self {
            Self::Waiting => Color32::from_rgb(52, 56, 62),
            Self::Recon => Color32::from_rgb(25, 111, 236),
            Self::Route => Color32::from_rgb(234, 159, 0),
            Self::Solve => Color32::from_rgb(132, 222, 24),
            Self::Error => Color32::from_rgb(214, 41, 54),
        }
    }


    /// Animate actual work only; waiting and error remain entirely stationary.
    const fn active(self) -> bool {
        matches!(self, Self::Recon | Self::Route | Self::Solve)
    }


    /// Return a bounded glow level from egui time without sleeping or spawning work.
    fn glow(self, time: f64) -> f32 {
        if !self.active() {
            return 0.0;
        }
        let time = if time.is_finite() {
            time.rem_euclid(1.6)
        } else {
            0.0
        };
        (0.50 + 0.35 * (time * std::f64::consts::TAU / 1.6).sin()) as f32
    }
}


/// Add an unmodified licensed bold face while preserving egui's default families.
pub(crate) fn install_fonts(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        BOLD_FAMILY.to_owned(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/DejaVuSans-Bold.ttf"))
            .into(),
    );
    fonts.families.insert(
        FontFamily::Name(BOLD_FAMILY.into()),
        vec![BOLD_FAMILY.to_owned()],
    );
    context.set_fonts(fonts);
}


/// Render a wrapping left column beside the fixed activity slot.
///
/// The left column starts at the parent's top edge. Its width excludes the
/// badge and halo; subsequent parent widgets regain the full panel width.
/// Return the selection result and painted face bounds for geometry checks.
pub(crate) fn header<R>(
    ui: &mut egui::Ui,
    state: Status,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> (R, Rect) {
    let slot = FACE_SIZE + egui::Vec2::splat(2.0 * HALO_MARGIN);
    let left_width = (ui.available_width() - slot.x - ui.spacing().item_spacing.x).max(0.0);
    ui.horizontal_top(|ui| {
        let content = ui.allocate_ui_with_layout(
            egui::vec2(left_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(left_width);
                add_contents(ui)
            },
        );
        let face = show(ui, state);
        (content.inner, face)
    })
    .inner
}


/// Reserve the face and halo without consuming the left selection column.
fn show(ui: &mut egui::Ui, state: Status) -> Rect {
    let (header, _) = ui.allocate_exact_size(
        FACE_SIZE + egui::Vec2::splat(2.0 * HALO_MARGIN),
        Sense::hover(),
    );
    let face = Rect::from_min_size(
        egui::pos2(
            header.right() - HALO_MARGIN - FACE_SIZE.x,
            header.top() + HALO_MARGIN,
        ),
        FACE_SIZE,
    );
    let painter = ui.painter();
    let glow = state.glow(ui.input(|input| input.time));
    if state.active() {
        for extent in [8.0_f32, 6.0, 4.0, 2.0] {
            let alpha = (glow * (1.0 - extent / 11.0) * 100.0) as u8;
            let colour = state.colour();
            painter.rect_stroke(
                face.expand(extent),
                13.0 + extent,
                Stroke::new(
                    3.0,
                    Color32::from_rgba_unmultiplied(colour.r(), colour.g(), colour.b(), alpha),
                ),
                egui::StrokeKind::Inside,
            );
        }
        ui.ctx().request_repaint_after(Duration::from_millis(25));
    }
    painter.rect_filled(face, 13.0, state.colour());
    painter.rect_stroke(
        face,
        13.0,
        Stroke::new(3.0, Color32::WHITE),
        egui::StrokeKind::Inside,
    );
    let font = FontId::new(26.0, FontFamily::Name(BOLD_FAMILY.into()));
    painter.text(
        face.center() + egui::vec2(0.0, 1.0),
        Align2::CENTER_CENTER,
        state.label(),
        font.clone(),
        Color32::from_black_alpha(70),
    );
    painter.text(
        face.center(),
        Align2::CENTER_CENTER,
        state.label(),
        font,
        Color32::WHITE,
    );
    face
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Every state occupies the same right-aligned face and preserves downstream
    /// controls; the actual action buttons fit below half its width and height.
    #[test]
    fn activity_badge_keeps_geometry_and_only_active_states_pulse() {
        let context = egui::Context::default();
        install_fonts(&context);
        let mut faces = Vec::new();
        for state in [
            Status::Waiting,
            Status::Recon,
            Status::Route,
            Status::Solve,
            Status::Error,
        ] {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(840.0, 600.0),
                    )),
                    ..Default::default()
                },
                |root_ui| {
                    egui::CentralPanel::default().show(root_ui, |ui| {
                        let (_, face) = header(ui, state, |_| ());
                        faces.push(face);
                        for label in ["1. RECON", "2. Plan route", "3. Solve board", "START"] {
                            let response = super::super::super::bevel_button(
                                ui,
                                label,
                                Color32::BLUE,
                                false,
                                true,
                            );
                            assert!(face.width() >= response.rect.width() * 2.0);
                            assert!(face.height() >= response.rect.height() * 2.0);
                            assert!(response.rect.top() > face.bottom());
                        }
                    });
                },
            );
            output.textures_delta.clear();
            if state.active() {
                assert_ne!(state.glow(0.4), state.glow(1.2));
            } else {
                assert_eq!(state.glow(0.4), 0.0);
                assert_eq!(state.glow(1.2), 0.0);
            }
        }
        assert!(faces.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(faces[0].size(), FACE_SIZE);
    }



    /// Selection content begins at the top left, wraps clear of the halo and
    /// leaves subsequent operational controls the complete supported panel width.
    #[test]
    fn activity_header_keeps_selection_and_full_width_body_clear_of_badge() {
        let context = egui::Context::default();
        install_fonts(&context);
        for width in [840.0, 1120.0, 1440.0] {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 820.0),
                    )),
                    ..Default::default()
                },
                |root_ui| {
                    egui::CentralPanel::default().show(root_ui, |ui| {
                        let panel = ui.available_rect_before_wrap();
                        let ((heading, selection), face) = header(ui, Status::Route, |ui| {
                            let heading = ui.heading("Pyramid").rect;
                            ui.add(egui::Label::new(
                                "Mode: Continuous; Route objective: Maximum card matches; Objective: Challenge target. Options and their descriptions must wrap before the activity indicator.",
                            ).wrap());
                            (heading, ui.min_rect())
                        });
                        assert_eq!(heading.left_top(), panel.left_top());
                        assert!(selection.right() <= face.left() - HALO_MARGIN);
                        assert_eq!(face.size(), FACE_SIZE);
                        assert_eq!(face.right(), panel.right() - HALO_MARGIN);
                        assert_eq!(face.top(), panel.top() + HALO_MARGIN);
                        let (body, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 24.0),
                            Sense::hover(),
                        );
                        assert_eq!(body.left(), panel.left());
                        assert_eq!(body.right(), panel.right());
                        assert!(body.top() > selection.bottom());
                        assert!(body.top() > face.bottom() + HALO_MARGIN);
                    });
                },
            );
            output.textures_delta.clear();
        }
    }


    /// Real long option selections cannot shift the badge or right-aligned EXIT.
    ///
    /// Uses the same option renderer as the application, including the disabled
    /// Manual Objective and both long Continuous labels; no app or worker starts.
    #[test]
    fn pyramid_options_keep_badge_and_exit_inside_original_panel() {
        use crate::pyramid_workflow_model::{RouteGoal, StopCondition, WorkflowMode};

        for (width, height) in [(840.0, 600.0), (1120.0, 820.0)] {
            for (selected_mode, selected_stop) in [
                (WorkflowMode::Manual, StopCondition::ChallengeTarget),
                (WorkflowMode::Continuous, StopCondition::RepeatNormalGames),
                (WorkflowMode::Continuous, StopCondition::NoChallenge),
            ] {
                for idle in [false, true] {
                    let context = egui::Context::default();
                    install_fonts(&context);
                    let mut mode = selected_mode;
                    let mut route_goal = RouteGoal::MaximumCardMatches;
                    let mut stop_condition = selected_stop;
                    let mut method_explicit = false;
                    for _ in 0..2 {
                        let mut output = context.run_ui(
                            egui::RawInput {
                                screen_rect: Some(Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, height),
                                )),
                                ..Default::default()
                            },
                            |root_ui| {
                                egui::CentralPanel::default().show(root_ui, |ui| {
                                    let panel = ui.available_rect_before_wrap();
                                    let left_width = panel.width()
                                        - FACE_SIZE.x
                                        - 2.0 * HALO_MARGIN
                                        - ui.spacing().item_spacing.x;
                                    let (selection, face) = header(ui, Status::Route, |ui| {
                                        let original_bound = ui.max_rect();
                                        ui.heading("Pyramid");
                                        crate::app::pyramid_workflow::pyramid_options_row(
                                            ui,
                                            idle,
                                            &mut mode,
                                            &mut route_goal,
                                            &mut stop_condition,
                                            &mut method_explicit,
                                        );
                                        let selection = ui.min_rect();
                                        assert!(
                                            selection.right() <= original_bound.right() + 0.01,
                                            "{width}pt {selected_mode:?} {selected_stop:?} idle={idle}: \
                                             selection {selection:?} exceeds {original_bound:?}",
                                        );
                                        selection
                                    });
                                    assert!((selection.width() - left_width).abs() < 0.01);
                                    assert_eq!(selection.left(), panel.left());
                                    assert_eq!(face.size(), FACE_SIZE);
                                    assert!((face.right() - (panel.right() - HALO_MARGIN)).abs() < 0.01);
                                    assert_eq!(face.top(), panel.top() + HALO_MARGIN);
                                    assert!(selection.right() <= face.left() - HALO_MARGIN);
                                    assert!(ui.min_rect().right() <= panel.right() + 0.01);
                                    assert!((ui.available_width() - panel.width()).abs() < 0.01);
                                    let exit = ui
                                        .with_layout(
                                            egui::Layout::right_to_left(egui::Align::Min),
                                            |ui| ui.button("EXIT"),
                                        )
                                        .inner;
                                    assert!((exit.rect.right() - panel.right()).abs() < 0.01);
                                    assert!(exit.rect.top() >= selection.bottom());
                                    assert!(exit.rect.top() >= face.bottom() + HALO_MARGIN);
                                });
                            },
                        );
                        output.textures_delta.clear();
                        assert_eq!(mode, selected_mode);
                        assert_eq!(route_goal, RouteGoal::MaximumCardMatches);
                        assert_eq!(stop_condition, selected_stop);
                        assert!(!method_explicit);
                    }
                }
            }
        }
    }
}
