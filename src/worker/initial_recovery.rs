//! Bounded Solver setup for explicitly requested TriPeaks and Pyramid runs.
//!
//! Read-only captures never enter this controller. A missing-HALO request may
//! adopt only a fresh canonical target; Solver setup is separate from the
//! gameplay action budget and never declares a board or game complete.


use super::{
    ActionProfile, FrameObservation, GuardedActionContext, WorkerState, capture_with_client,
    cancellable_wait, click_shared_solver, send_log, send_state, send_status, validate_probe,
};
use crate::{
    game::{ActionTarget, GameMode},
    parameters::{SOLVER_MOUSE_HOLD, StepRunSettings},
    pyramid,
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
};
use std::{sync::atomic::Ordering, time::Instant};


/// Three delayed captures on either side of the sole optional Solver click.
pub(super) const INITIAL_HALO_REOBSERVATIONS: usize = 3;


/// Capture, cancellable wait and non-retrying input boundary used by production and tests.
pub(super) trait InitialRecoveryIo {
    /// Read the cooperative STOP flag without clearing it.
    fn stopped(&self) -> bool;


    /// Acquire and classify one fresh frame, retaining its exact pixels.
    fn capture(&mut self) -> Result<FrameObservation, String>;


    /// Wait before one delayed observation, checking STOP throughout.
    fn wait(&mut self, delayed_round: usize, after_solver: bool) -> Result<(), String>;


    /// Validate the live QMP probe and attempt the already-reserved Solver click once.
    fn refresh_solver(&mut self) -> Result<(), String>;
}


/// Latest evidence and reservation status when startup cannot establish an action.
pub(super) struct InitialRecoveryFailure {
    /// Detailed reason; setup input is never described as a gameplay action.
    pub(super) message: String,
    /// Latest capture, including a frame acquired immediately before STOP.
    pub(super) observation: Option<FrameObservation>,
    /// True before entering the sole Solver input boundary, including delivery errors.
    pub(super) solver_reserved: bool,
    /// Distinguish a cooperative stop from unsupported or uncertain evidence.
    pub(super) state: WorkerState,
}


/// Require mode ownership and the immutable input definition for the fresh action.
fn canonical_plan(mode: GameMode, prediction: PredictedAction) -> Result<StepPlan, String> {
    let plan = plan_step(prediction).map_err(|error| error.to_string())?;
    let action = plan.input().action();
    let canonical = match action.target {
        ActionTarget::Bottom { index, .. } if mode == GameMode::TriPeaks => {
            mode.profile().bottom_action(usize::from(index), action.anchor)
        }
        ActionTarget::Tableau(target) if mode == GameMode::TriPeaks => {
            mode.profile().tableau_action(target.slot_index, action.anchor)
        }
        ActionTarget::Pyramid(kind) if mode == GameMode::Pyramid => {
            pyramid::action_for_kind(kind)
        }
        _ => None,
    };


    if canonical != Some(action) {
        return Err("fresh target is not a canonical action for the selected game".to_owned());
    }
    Ok(plan)
}


/// Require positive occupied-tableau evidence, never an unknown Pyramid row mask.
fn nonempty_scene(mode: GameMode, observation: &FrameObservation) -> Result<bool, String> {


    if !observation.gameplay_scene {
        return Ok(false);
    }


    match mode {
        GameMode::TriPeaks => Ok(observation.observed_rows.is_some_and(|rows| !rows.is_empty())),
        GameMode::Pyramid => pyramid::has_visible_tableau_card(&observation.frame)
            .map_err(|error| format!("Pyramid positive-card analysis failed: {error}")),
        _ => Ok(false),
    }
}


/// Resolve an explicit missing-HALO request without changing history or sending gameplay input.
///
/// Each phase starts with an immediate capture and permits three delayed ones.
/// A target ends recovery immediately. Only the last positively recognised,
/// nonempty scene can reserve Solver; that reservation is consumed before I/O.
pub(super) fn recover_initial_target(
    mode: GameMode,
    io: &mut impl InitialRecoveryIo,
) -> Result<(FrameObservation, StepPlan), Box<InitialRecoveryFailure>> {
    let mut latest = None;
    let mut solver_reserved = false;
    let result = (|| -> Result<(FrameObservation, StepPlan), String> {


        if !matches!(mode, GameMode::TriPeaks | GameMode::Pyramid) {
            return Err("initial Solver setup is unsupported for the selected game".to_owned());
        }


        for after_solver in [false, true] {


            for delayed_round in 0..=INITIAL_HALO_REOBSERVATIONS {


                if io.stopped() {
                    return Err("STOP was requested during initial HALO recovery".to_owned());
                }


                if delayed_round > 0 {
                    io.wait(delayed_round, after_solver)?;
                }


                if io.stopped() {
                    return Err("STOP was requested before an initial recovery capture".to_owned());
                }
                latest = Some(io.capture()?);


                if io.stopped() {
                    return Err("STOP was requested after an initial recovery capture".to_owned());
                }
                let observation = latest.as_ref().expect("capture retained above");


                if observation.prediction != PredictedAction::NoHighlight {


                    if !observation.gameplay_scene {
                        return Err("fresh target is outside a recognised gameplay scene".to_owned());
                    }
                    let plan = canonical_plan(mode, observation.prediction)?;
                    return Ok((latest.take().expect("accepted fresh frame"), plan));
                }
            }


            if after_solver {
                return Err(format!(
                    "no fresh canonical HALO after one Solver click and {INITIAL_HALO_REOBSERVATIONS} delayed observations"
                ));
            }
            let observation = latest.as_ref().expect("initial phase captured a frame");


            if !nonempty_scene(mode, observation)? {
                return Err("latest scene is unsupported, empty or lacks positive tableau-card evidence; Solver setup refused".to_owned());
            }


            if io.stopped() {
                return Err("STOP was requested before initial Solver setup".to_owned());
            }
            // Consume this context's sole setup operation before probe or input I/O.
            solver_reserved = true;
            io.refresh_solver()?;
            // The next phase captures immediately after acknowledgement, then delays only if needed.
        }
        unreachable!("both bounded observation phases return")
    })();
    result.map_err(|reason| Box::new(InitialRecoveryFailure {
        message: format!(
            "{mode} initial HALO recovery stopped: {reason}; Solver setup reserved={solver_reserved}, gameplay input sent: 0. No Solver input was retried."
        ),
        observation: latest,
        solver_reserved,
        state: if io.stopped() { WorkerState::Ready } else { WorkerState::Uncertain },
    }))
}


/// Production boundary keeps capture publication, timing and QMP guards in one place.
pub(super) struct QmpInitialRecovery<'a> {
    /// Serial worker connection; no reconnection or delivery retry is permitted here.
    pub(super) qmp: &'a mut QmpClient,
    /// Immutable row hints used to classify every fresh startup frame.
    pub(super) scan_state: &'a TableauScanState,
    /// Selected socket, latest-frame publication and STOP flag.
    pub(super) context: GuardedActionContext<'a>,
    /// Immutable timing snapshot from the explicitly requested run.
    pub(super) settings: StepRunSettings,
    /// Setup work contributes to the surrounding action's profile, not its budget.
    pub(super) profile: &'a mut ActionProfile,
}


impl InitialRecoveryIo for QmpInitialRecovery<'_> {
    fn stopped(&self) -> bool {
        self.context.cancel_requested.load(Ordering::Acquire)
    }


    fn capture(&mut self) -> Result<FrameObservation, String> {
        send_status(self.context.event_tx, "Screengrab — initial HALO recovery".to_owned());
        let (observation, timing) = capture_with_client(
            self.qmp, self.context.socket_path, self.scan_state, self.context.event_tx,
        )?;
        self.profile.capture.add_assign(timing);
        Ok(observation)
    }


    fn wait(&mut self, delayed_round: usize, after_solver: bool) -> Result<(), String> {
        let delays = self.settings.animation_delays();
        let delay = if self.scan_state.mode() == GameMode::Pyramid {
            delays.pyramid_reobserve
        } else {
            delays.tripeaks_reobserve
        };
        send_log(self.context.event_tx, format!(
            "{} initial missing-HALO observation {delayed_round}/{INITIAL_HALO_REOBSERVATIONS} {} Solver setup: waiting {} ms for one fresh input-free capture; no gameplay input authorised.",
            self.scan_state.mode(), if after_solver { "after" } else { "before" }, delay.as_millis(),
        ));


        match cancellable_wait(delay, self.context.cancel_requested) {
            Ok(waited) => { self.profile.intentional_wait += waited; Ok(()) }
            Err(waited) => {
                self.profile.intentional_wait += waited;
                Err("STOP was requested during the initial observation delay".to_owned())
            }
        }
    }


    fn refresh_solver(&mut self) -> Result<(), String> {
        let probe_started = Instant::now();
        let probe = self.qmp.probe().map_err(|error| format!("Solver pre-input QMP probe failed: {error}"))?;
        validate_probe(&probe)?;
        self.profile.validation_effect += probe_started.elapsed();


        if self.stopped() {
            return Err("STOP was requested after the Solver pre-input probe; Solver input sent: 0".to_owned());
        }
        send_state(self.context.event_tx, WorkerState::Acting);
        send_status(self.context.event_tx, "Click Solver — initial setup".to_owned());
        send_log(self.context.event_tx, format!(
            "{} initial Solver setup reserved once on the latest recognised nonempty scene; one existing held Solver click, then an immediate capture and at most {INITIAL_HALO_REOBSERVATIONS} delayed captures. Setup does not consume a gameplay action.",
            self.scan_state.mode(),
        ));
        let input_started = Instant::now();
        self.profile.input_attempted = true;
        let result = click_shared_solver(self.qmp, self.context.cancel_requested);
        self.profile.input_wall += input_started.elapsed();
        result?;
        self.profile.intentional_wait += SOLVER_MOUSE_HOLD;
        send_state(self.context.event_tx, WorkerState::Validating);
        Ok(())
    }
}
