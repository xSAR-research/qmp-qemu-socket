//! One worker reservation across independent Pyramid acquisition, planning and play.
//!
//! Unknown entry progress never means zero. Full starting-board occupancy and
//! New Game/Play delivery do not prove a standard three-board target. Native
//! acquisition requires separately calibrated positive series evidence. A cold
//! supported terminal entry must prove the new target after restart.

use super::{
    WorkerEvent, WorkerEventSink, WorkerState, analyse_captured_frame, cancellable_wait,
    capture_screen, computed_route, post_game, publish_pyramid_failure, pyramid_reconnaissance,
    send_log, send_state, send_status,
};
use crate::{
    capture::CapturedFrame,
    game::GameMode,
    pyramid_diagnostics::{FailureEvidence, PyramidPhase},
    pyramid_progress::{BoardCount, ChallengeCompletion, ChallengeProgress},
    pyramid_recon::{
        ReconRequest,
        vision::{BoardObservation, ReconScene, ReconVision, StockFace},
    },
    pyramid_workflow_model::{
        SessionCompletion, SessionPhase, SessionProgress, SessionRequest, StopCondition,
        WorkflowMode,
    },
    qmp::QmpClient,
    strategy::ControllerContext,
    tracker::TableauScanState,
};
use computed_route::BoardCompletion;
use std::{
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};


#[cfg(test)]
use std::time::Duration;


mod challenge;


/// Standard-game target, independent of any challenge requirement.
const BOARDS_PER_GAME: u8 = 3;


/// Fresh input-free observations permitted for an automatic board transition.
const TRANSITION_OBSERVATIONS: usize = 6;


/// Positive entry facts, independent of a UI counter or previous run's log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Entry {
    /// Actual challenge requirement on a complete untouched starting board.
    Challenge(ChallengeProgress),
    /// Positive actual terminal already present at entry or transition.
    ChallengeComplete(ChallengeCompletion),
    /// Current untouched board; a displayed fraction does not establish game identity.
    Board {
        /// Positively observed header values, never an inferred local count.
        progress: Option<BoardCount>,
        /// Independent standard-game evidence; a denominator of three is insufficient.
        standard_three: bool,
    },
    /// Supported Pyramid game-end evidence; the next target is verified after restart.
    Terminal {
        /// Positive Pyramid end/expected-stage evidence, not the previous target count.
        supported_pyramid: bool,
    },
}


/// Distinct terminal stages must each remain stable across their own fresh pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeTerminal {
    /// Existing specific Pyramid Level Up over its recognised reward layout.
    LevelUp,
    /// Expected New Game control after actual bound normal third-board completion.
    NewGame,
}


/// One native scene fact; board identity and header belong to the same captured frame.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NativeObservation {
    /// Starting identities and an actual challenge requirement from the same frame.
    Challenge {
        /// Complete starting identities independently recognised in the same frame.
        position: BoardObservation,
        /// Actual objective, fixed target and observed progress from that frame.
        progress: ChallengeProgress,
    },
    /// Literal native Challenge Complete evidence, distinct from normal GAME WIN.
    ChallengeComplete(ChallengeCompletion),
    /// Full original fixed-position identities and the separately observed header.
    Board {
        /// All tableau identities, initial DL and positive empty DR.
        position: BoardObservation,
        /// Unknown header pixels remain unknown even on a complete board.
        progress: Option<BoardCount>,
        /// Literal normal ordinal and New footer, independently of challenge counts.
        standard_three: bool,
    },
    /// Specific Pyramid game-end or caller-witnessed expected terminal stage.
    Terminal(NativeTerminal),
    /// Neither supported full gameplay nor the specific terminal scene.
    Unknown,
}


/// Pair equal board/header/mode facts or two observations of the same supported terminal kind.
fn paired_entry(first: &NativeObservation, second: &NativeObservation) -> Option<Entry> {
    if first != second {
        return None;
    }
    match second {
        NativeObservation::Challenge { progress, .. } => Some(Entry::Challenge(*progress)),
        NativeObservation::ChallengeComplete(proof) => Some(Entry::ChallengeComplete(*proof)),
        NativeObservation::Board {
            progress,
            standard_three,
            ..
        } => Some(Entry::Board {
            progress: *progress,
            standard_three: *standard_three,
        }),
        NativeObservation::Terminal(_) => Some(Entry::Terminal {
            supported_pyramid: true,
        }),
        NativeObservation::Unknown => None,
    }
}


/// Require both positive standard-game identity and a supported observed starting count.
fn standard_progress(entry: Entry) -> Result<BoardCount, String> {
    if let Entry::Board {
        progress: Some(progress),
        standard_three: true,
    } = entry
        && progress.target == BOARDS_PER_GAME
        && progress.completed < BOARDS_PER_GAME
    {
        return Ok(progress);
    }
    Err(format!(
        "Continuous cannot establish standard-game identity and observed 0/3, 1/3 or 2/3 progress from {entry:?}; no acquisition or continuation input authorised"
    ))
}


/// Accept a new game only from a positively observed supported zero-of-three starting board.
fn restarted_progress(entry: Entry) -> Result<BoardCount, String> {
    let progress = standard_progress(entry)?;
    if progress.completed != 0 {
        return Err(format!(
            "Restart returned observed {}/{} gameplay instead of a verified fresh 0/3 board; no acquisition input authorised",
            progress.completed, progress.target
        ));
    }
    Ok(progress)
}


/// Exact starting identities retained only with a positively qualified series entry.
struct StartingWitness {
    /// Original fixed tableau, initial LEFT and positive empty RIGHT from that entry.
    position: BoardObservation,
    /// Literal completed/target values belonging to the same observed starting board.
    progress: BoardCount,
}


/// Challenge starting identities retain their actual immutable requirement separately.
struct ChallengeWitness {
    /// Same-frame complete tableau, LEFT and positive empty RIGHT identities.
    position: BoardObservation,
    /// Current counter and fixed kind/rank/target from the qualified entry pair.
    progress: ChallengeProgress,
}


/// Extract a challenge witness only from its complete equal starting-board pair.
fn challenge_witness(entry: Entry, observation: &NativeObservation) -> Option<ChallengeWitness> {
    match (entry, observation) {
        (Entry::Challenge(expected), NativeObservation::Challenge { position, progress })
            if expected == *progress =>
        {
            Some(ChallengeWitness {
                position: position.clone(),
                progress: *progress,
            })
        }
        _ => None,
    }
}


/// Preserve identities with the qualified count instead of reducing entry to a counter.
fn starting_witness(entry: Entry, observation: &NativeObservation) -> Option<StartingWitness> {
    let progress = standard_progress(entry).ok()?;
    match observation {
        NativeObservation::Board {
            position,
            progress: Some(observed),
            standard_three: true,
        } if *observed == progress
            && matches!(position.left, StockFace::Card(_))
            && position.right == StockFace::Empty =>
        {
            Some(StartingWitness {
                position: position.clone(),
                progress,
            })
        }
        _ => None,
    }
}


/// Use the positive normal header selector for entry and guarded reconnaissance frames.
///
/// Centred challenge fractions cannot satisfy this same-session mode/target check.
/// The caller separately requires the full Pyramid starting board and a fresh pair.
fn observed_progress(frame: &CapturedFrame) -> Result<Option<BoardCount>, String> {
    crate::pyramid_progress::observe_normal(frame)
}


/// Phase boundary used by the serial controller and deterministic headless tests.
trait Pipeline {
    /// Return the highest actual counter observed during the last execution, if supported.
    fn challenge_progress(&self) -> Option<ChallengeProgress> {
        None
    }


    /// Refuse STOP or context drift before advancing to another phase.
    fn check(&self) -> Result<(), String>;
    /// Publish the exact board generation without releasing ownership.
    fn phase(&mut self, generation: u64, phase: SessionPhase);
    /// Read supported entry evidence, never assuming unknown progress is zero.
    fn entry(&mut self, mode: WorkflowMode) -> Result<Entry, String>;
    /// Acquire a complete restored deal and mint the private same-session receipt.
    fn acquire(&mut self, generation: u64) -> Result<(), String>;
    /// Consume that receipt for input-free calculation and independent replay.
    fn plan(&mut self, generation: u64) -> Result<(), String>;
    /// Consume the prepared plan and return typed final evidence.
    fn execute(&mut self, generation: u64) -> Result<BoardCompletion, String>;
    /// Finish shared reward controls under the retained completion receipt, without NewGame.
    fn finish_challenge(
        &mut self,
        proof: ChallengeCompletion,
    ) -> Result<ChallengeCompletion, String>;
    /// Observe a fresh scene and actual progress after every positive execution outcome.
    fn transition(&mut self) -> Result<Entry, String>;
    /// Use verified terminal controls and return the freshly observed entry, without Solver.
    fn restart(&mut self) -> Result<Entry, String>;
}


/// Execute the selected lifetime without queueing another command or clearing STOP.
fn drive(mode: WorkflowMode, io: &mut impl Pipeline) -> Result<String, String> {
    if mode == WorkflowMode::Manual {
        return Err("Manual mode requires its three separate buttons".to_owned());
    }
    io.check()?;
    io.phase(0, SessionPhase::Entry);
    io.check()?;
    let mut progress = match io.entry(mode)? {
        Entry::Board { .. } if mode == WorkflowMode::SingleBoard => None,
        board @ Entry::Board { .. } => Some(standard_progress(board)?),
        Entry::Terminal {
            supported_pyramid: true,
        } if mode == WorkflowMode::Continuous => {
            io.check()?;
            io.phase(0, SessionPhase::Restart);
            io.check()?;
            let observed = io.restart()?;
            io.check()?;
            Some(restarted_progress(observed)?)
        }
        Entry::Terminal {
            supported_pyramid: false,
        } if mode == WorkflowMode::Continuous => {
            return Err("Continuous terminal entry lacks supported Pyramid game-end evidence; no restart input authorised.".to_owned());
        }
        Entry::Terminal { .. } => {
            return Err("Single board requires an untouched gameplay board".to_owned());
        }
        Entry::Challenge(_) | Entry::ChallengeComplete(_) => return Err("Observed challenge requires the Challenge target stop condition; normal-game repetition is not authorised".to_owned()),
    };
    let mut generation = 0_u64;
    loop {
        io.check()?;
        generation = generation
            .checked_add(1)
            .ok_or("Session board generation exhausted")?;
        io.phase(generation, SessionPhase::Reconnaissance);
        io.check()?;
        io.acquire(generation)?;
        io.check()?;
        io.phase(generation, SessionPhase::Planning);
        io.check()?;
        io.plan(generation)?;
        io.check()?;
        io.phase(generation, SessionPhase::Execution);
        io.check()?;
        let completion = io.execute(generation)?;
        io.check()?;
        if let BoardCompletion::ChallengeTargetReached(progress) = completion {
            if mode == WorkflowMode::SingleBoard {
                return Ok(format!(
                    "Challenge target counter verified: {progress:?}; exact card effect verified, terminal completion still pending; control released without terminal input."
                ));
            }
            return Err("Challenge target appeared during normal-game repetition; no restart input authorised".to_owned());
        }
        if let BoardCompletion::ChallengeTargetVerified(proof) = completion {
            if mode == WorkflowMode::SingleBoard {
                return Ok(format!(
                    "Challenge target verified: {:?}; control released; no NewGame input.",
                    proof.progress()
                ));
            }
            return Err("Challenge completion appeared during normal-game repetition; no restart input authorised".to_owned());
        }
        if completion == BoardCompletion::DeliveredUnconfirmed {
            return Err("Final input acknowledged; completion unconfirmed. Automatic session stopped; inspect and reacquire, no further input.".to_owned());
        }
        if let BoardCompletion::PartialPositionVerified { removed, remaining } = completion {
            return Ok(format!(
                "Partial route complete — {removed}/28 tableau cards removed, {remaining} remain; exact endpoint verified. Automatic session stopped; no next-board or restart input."
            ));
        }
        if mode == WorkflowMode::SingleBoard {
            return Ok(format!(
                "Single board complete — {completion:?}; control released; no next-board or terminal input."
            ));
        }
        let previous = progress.ok_or("Missing positively observed series progress")?;
        // Existing execution outcomes prove final effects, not current header values.
        io.phase(generation, SessionPhase::Transition);
        io.check()?;
        let observed = io.transition()?;
        io.check()?;
        // This is a comparison expectation only; accepted state always comes from pixels.
        let expected = previous
            .completed
            .checked_add(1)
            .ok_or("Series progress exhausted")?;
        match observed {
            board @ Entry::Board { .. } if expected < BOARDS_PER_GAME => {
                let next = standard_progress(board)?;
                if next.completed != expected {
                    return Err(format!(
                        "Observed series progress is {}/{} after {}/{}; expected {expected}/3. No continuation input authorised",
                        next.completed, next.target, previous.completed, previous.target
                    ));
                }
                progress = Some(next);
            }
            Entry::Terminal { supported_pyramid: true } if expected == BOARDS_PER_GAME => {
                io.phase(generation, SessionPhase::Restart);
                io.check()?;
                let observed = io.restart()?;
                io.check()?;
                progress = Some(restarted_progress(observed)?);
            }
            Entry::Terminal { supported_pyramid: true } => return Err(
                "Terminal evidence arrived before the supported three-board target; session stopped without restart".to_owned(),
            ),
            Entry::Board { .. } => return Err(
                "A new board appeared after the expected third clear; unsupported series target, no continuation".to_owned(),
            ),
            Entry::Terminal { supported_pyramid: false } => return Err(
                "Terminal lacks supported Pyramid game-end evidence; no restart input authorised".to_owned(),
            ),
            Entry::Challenge(_) | Entry::ChallengeComplete(_) => return Err("Normal-game objective changed to a challenge; no continuation input authorised".to_owned()),
        }
    }
}


/// Repeat Single-board phases without interpreting any challenge or normal-series header.
///
/// Only a scheduled fresh-board handoff or one acknowledged guarded REDEAL may
/// start another acquisition. Fresh Recon supplies the new untouched-board proof.
fn drive_no_challenge(io: &mut impl Pipeline) -> Result<String, String> {
    io.check()?;
    io.phase(0, SessionPhase::Entry);
    io.check()?;
    io.entry(WorkflowMode::Continuous)?;
    let mut generation = 0_u64;
    loop {
        io.check()?;
        generation = generation
            .checked_add(1)
            .ok_or("Board generation exhausted")?;
        io.phase(generation, SessionPhase::Reconnaissance);
        io.check()?;
        io.acquire(generation)?;
        io.check()?;
        io.phase(generation, SessionPhase::Planning);
        io.check()?;
        io.plan(generation)?;
        io.check()?;
        io.phase(generation, SessionPhase::Execution);
        io.check()?;
        let result = io.execute(generation)?;
        io.check()?;
        if !matches!(
            result,
            BoardCompletion::AwaitingFreshBoard | BoardCompletion::StockRedealt
        ) {
            return Err(format!(
                "No-challenge continuation lacks a fresh-board handoff: {result:?}; no further input"
            ));
        }
    }
}


/// Complete native state for one immutable request and its one active reservation.
struct Native<'a> {
    /// Context whose authority is checked at every boundary.
    context: &'a ControllerContext,
    /// Explicit start consent and fixed method/profile choices.
    request: &'a SessionRequest,
    /// Ordered events and bounded advisory slots.
    events: &'a WorkerEventSink,
    /// Immediate monotonic STOP latch.
    cancel: &'a AtomicBool,
    /// Current phase for failure correlation.
    phase: SessionPhase,
    /// Monotonic identity of the current board, zero only before acquisition.
    generation: u64,
    /// Actual calibration, replaced by the consumed execution receipt's calibration.
    vision: Option<ReconVision>,
    /// Published path for this board, never inferred from the next suggested UI path.
    output: Option<PathBuf>,
    /// Latest existing observation for diagnostics or a verified restart entry.
    latest: Option<CapturedFrame>,
    /// Single-use identities and series header from the current qualified entry pair.
    starting: Option<StartingWitness>,
    /// Qualified normal-series count for the board currently being acquired and played.
    series: Option<BoardCount>,
    /// Same-session challenge identities, consumed once before reconnaissance.
    challenge_starting: Option<ChallengeWitness>,
    /// Actual bound challenge requirement for the current acquired board.
    challenge_progress: Option<ChallengeProgress>,
    /// Actual last-board completion, distinct from a planned goal or delivered gesture.
    restart_entry: Option<post_game::PyramidRestartEntry>,
    /// Whether a phase helper already persisted this failure.
    failure_saved: bool,
    /// Current board's effective method; reset after a new-board acquisition.
    method: crate::pyramid_workflow_model::RouteMethod,
    /// One optional first-board reuse request, retired before any later acquisition.
    reuse_acquisition: Option<PathBuf>,
    /// Advisory observed header, separate from qualified progress and input witnesses.
    observed_game: crate::pyramid_workflow_model::ObservedGame,
}


impl Native<'_> {
    /// Publish observed facts with exact run/board ownership, never as a control permit.
    fn publish_observed(
        &mut self,
        observed: crate::pyramid_workflow_model::ObservedGame,
        source: &str,
    ) {
        self.observed_game = observed;
        let message = format!(
            "Observed {}: {}; selected mode={}; stop condition={}",
            source,
            observed.label(),
            self.request.mode.label(),
            self.request.stop_condition.label()
        );
        send_log(self.events, message.clone());
        let _ = self
            .events
            .send(WorkerEvent::PyramidSessionProgress(SessionProgress {
                context: self.context.clone(),
                run_id: self.request.run_id,
                board_generation: self.generation,
                phase: self.phase,
                effective_method: self.method,
                observed_game: observed,
                message,
            }));
    }


    /// Apply this board's authenticated default, preserving explicit first-board choice.
    fn apply_acquisition_default(&mut self, generation: u64) -> Result<(), String> {
        if (self.request.mode == WorkflowMode::Continuous
            && self.request.stop_condition == StopCondition::NoChallenge)
            || (generation == 1 && !self.request.use_challenge_default)
        {
            return Ok(());
        }
        let path = self
            .output
            .as_ref()
            .ok_or("Missing acquisition for challenge default")?;
        self.method =
            computed_route::acquisition_default(self.context, path, self.events, self.cancel)?;
        send_log(
            self.events,
            format!(
                "Current-board default: {}; Kings Card challenges use Maximum, all other recognised challenges use Shortest.",
                self.method.route_goal().label()
            ),
        );
        Ok(())
    }


    /// Capture once under the retained reservation; preview pixels grant no authority.
    fn capture(&mut self, qmp: &mut QmpClient) -> Result<CapturedFrame, String> {
        self.check()?;
        send_state(self.events, WorkerState::Capturing);
        let (frame, _) = capture_screen(qmp, &self.context.socket_path, self.events)?;
        self.latest = Some(frame.clone());
        self.events.publish_diagnostic_frame(frame.clone());
        self.check()?;
        let observed =
            crate::pyramid_workflow_model::ObservedGame::observe(&frame).unwrap_or_default();
        if observed != self.observed_game {
            self.publish_observed(observed, "entry/transition header (advisory)");
        }
        Ok(frame)
    }


    /// Wait cooperatively without releasing or re-reserving the parent run.
    fn wait(&self) -> Result<(), String> {
        self.check()?;
        cancellable_wait(crate::parameters::PYRAMID_REOBSERVE_DELAY, self.cancel)
            .map_err(|_| "STOP during automatic transition; no cleanup input".to_owned())?;
        self.check()
    }


    /// Recognise all starting identities and positive empty RIGHT using the actual profile.
    fn starting_board(&self, frame: &CapturedFrame) -> Option<BoardObservation> {
        match self.vision.as_ref()?.observe(frame).ok()? {
            ReconScene::Board(board)
                if matches!(board.left, StockFace::Card(_)) && board.right == StockFace::Empty =>
            {
                Some(board)
            }
            _ => None,
        }
    }


    /// Read scene and header from the same existing frame without granting input authority.
    fn observe(&self, frame: &CapturedFrame) -> Result<NativeObservation, String> {
        let challenge_complete = match self.challenge_progress {
            Some(bound) => crate::pyramid_progress::observe_bound_challenge_complete(frame, bound)?,
            None => crate::pyramid_progress::observe_challenge_complete(frame)?,
        };
        if let Some(proof) = challenge_complete {
            return Ok(NativeObservation::ChallengeComplete(proof));
        }
        if post_game::recognised_pyramid_terminal(frame)? {
            return Ok(NativeObservation::Terminal(NativeTerminal::LevelUp));
        }
        if self.restart_entry.is_some() && post_game::expected_pyramid_new_game(frame)? {
            return Ok(NativeObservation::Terminal(NativeTerminal::NewGame));
        }
        match self.starting_board(frame) {
            Some(position) => {
                let normal = observed_progress(frame)?;
                if normal.is_none()
                    && let Some(progress) = crate::pyramid_progress::observe_challenge(frame)?
                {
                    return Ok(NativeObservation::Challenge { position, progress });
                }
                let progress = match normal {
                    Some(progress) => Some(progress),
                    None => crate::pyramid_progress::observe(frame)?,
                };
                Ok(NativeObservation::Board {
                    position,
                    progress,
                    standard_three: normal.is_some(),
                })
            }
            None => Ok(NativeObservation::Unknown),
        }
    }


    /// Await two equal untouched positions only after a no-challenge new-board handoff.
    ///
    /// At most six existing-paced fresh captures are allowed. Static/modal/frame
    /// failures, transport errors, STOP and context changes are fatal. Pending
    /// glyphs reset the pair; no input, header classification or receipt is issued.
    fn await_no_challenge_start(&mut self) -> Result<BoardObservation, String> {
        self.check()?;
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|error| format!("Starting-readiness QMP connection failed: {error}"))?;
        let mut previous: Option<BoardObservation> = None;
        let mut reason = "No two equal complete starting positions".to_owned();
        for attempt in 0..TRANSITION_OBSERVATIONS {
            self.check()?;
            let frame = self.capture(&mut qmp)?;
            let vision = self
                .vision
                .as_ref()
                .ok_or("Starting readiness lacks calibration")?;
            match vision.starting_readiness(&frame)? {
                crate::pyramid_recon::vision::StartingReadiness::Ready(position) => {
                    if previous.as_ref().is_some_and(|old| {
                        old.tableau == position.tableau
                            && old.left == position.left
                            && old.right == position.right
                    }) {
                        self.check()?;
                        send_log(self.events, "New board settled: two equal complete starting positions; fresh guarded Recon follows.".to_owned());
                        return Ok(position);
                    }
                    previous = Some(position);
                    reason = "Waiting for an equal fresh complete starting position".to_owned();
                }
                crate::pyramid_recon::vision::StartingReadiness::Pending(error) => {
                    previous = None;
                    reason = error;
                }
            }
            send_status(
                self.events,
                format!(
                    "Waiting for new board to settle — observation {}/{}; no input",
                    attempt + 1,
                    TRANSITION_OBSERVATIONS
                ),
            );
            if attempt + 1 < TRANSITION_OBSERVATIONS {
                self.wait()?;
            }
        }
        Err(format!(
            "New board did not settle within {TRANSITION_OBSERVATIONS} fresh observations; no acquisition input: {reason}"
        ))
    }


    /// Capture a fresh equal pair without repeating the original output-path preflight.
    fn observed_entry(&mut self) -> Result<Entry, String> {
        self.starting = None;
        self.series = None;
        self.challenge_starting = None;
        self.challenge_progress = None;
        self.restart_entry = None;
        let mut qmp =
            QmpClient::connect(&self.context.socket_path).map_err(|error| error.to_string())?;
        let first = self.capture(&mut qmp)?;
        let first = self.observe(&first)?;
        self.wait()?;
        let second = self.capture(&mut qmp)?;
        let second = self.observe(&second)?;
        let entry = paired_entry(&first, &second).ok_or_else(||
            "Automatic entry lacks two matching supported board/header or terminal observations; no input authorised".to_owned()
        )?;
        self.starting = starting_witness(entry, &second);
        self.series = self.starting.as_ref().map(|witness| witness.progress);
        self.challenge_starting = challenge_witness(entry, &second);
        self.challenge_progress = self
            .challenge_starting
            .as_ref()
            .map(|witness| witness.progress);
        if self.reuse_acquisition.is_some()
            && self.request.stop_condition == StopCondition::ChallengeTarget
        {
            let path = self
                .reuse_acquisition
                .as_ref()
                .ok_or("Missing retained Recon path")?;
            let (_, bound) =
                computed_route::reuse_metadata(self.context, path, self.events, self.cancel)?;
            if bound != self.challenge_progress {
                return Err("Retained Recon challenge requirement differs from fresh entry; no input authorised".to_owned());
            }
        }
        Ok(entry)
    }


    /// Generate a bounded unused destination after the first literal requested output.
    fn next_request(&self, generation: u64) -> Result<ReconRequest, String> {
        let mut request = self.request.reconnaissance.clone();
        if generation == 1 {
            return Ok(request);
        }
        let parent = request
            .output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        let parent = std::fs::canonicalize(parent)
            .map_err(|error| format!("Automatic output parent: {error}"))?;
        let millis = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "System clock precedes epoch")?
                .as_millis(),
        )
        .map_err(|_| "Clock exceeds naming bound")?;
        for attempt in 0..64_u8 {
            let id = format!(
                "p{millis:x}-{:x}-{:x}-{generation:x}-{attempt:x}",
                std::process::id(),
                self.request.run_id
            );
            let output = parent.join(format!("pyramid-recon-{id}.json"));
            match std::fs::symlink_metadata(&output) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    request.deal_id = id;
                    request.output_path = output;
                    request.validate()?;
                    return Ok(request);
                }
                Ok(_) => {}
                Err(error) => return Err(format!("Cannot inspect automatic destination: {error}")),
            }
        }
        Err("Automatic output name collision budget exhausted".to_owned())
    }
}


impl Pipeline for Native<'_> {
    fn challenge_progress(&self) -> Option<ChallengeProgress> {
        self.challenge_progress
    }


    fn finish_challenge(
        &mut self,
        proof: ChallengeCompletion,
    ) -> Result<ChallengeCompletion, String> {
        self.check()?;
        let frame = self
            .latest
            .clone()
            .ok_or("Missing bound challenge completion frame")?;
        let mut scan = TableauScanState::for_mode(GameMode::Pyramid);
        let (initial, _) = analyse_captured_frame(frame, &scan)?;
        let mut qmp =
            QmpClient::connect(&self.context.socket_path).map_err(|error| error.to_string())?;
        post_game::finish_pyramid_challenge(
            &mut qmp,
            self.context,
            &initial,
            proof,
            &mut scan,
            self.events,
            self.cancel,
            &mut self.latest,
        )?;
        self.check()?;
        let frame = self
            .latest
            .as_ref()
            .ok_or("Missing challenge endpoint frame")?;
        match crate::pyramid_progress::observe_bound_challenge_complete(frame, proof.progress())? {
            Some(actual) if actual.progress().follows(proof.progress()) => Ok(actual),
            Some(_) => Err("Challenge endpoint requirement changed; no further input".to_owned()),
            None => Ok(proof),
        }
    }
    fn check(&self) -> Result<(), String> {
        self.events
            .latest_frame
            .reconnaissance
            .check_active(self.context, self.cancel)
    }


    fn phase(&mut self, generation: u64, phase: SessionPhase) {
        if generation > 1 && generation > self.generation {
            self.method = crate::pyramid_workflow_model::RouteMethod::FewestActions;
            self.observed_game = crate::pyramid_workflow_model::ObservedGame::Unknown;
        }
        self.generation = generation;
        self.phase = phase;
        let message = format!(
            "{} — board {generation}; selected mode={}; stop condition={}; observed={}; effective objective={}",
            phase.label(),
            self.request.mode.label(),
            self.request.stop_condition.label(),
            self.observed_game.label(),
            self.method.route_goal().label()
        );
        send_status(self.events, message.clone());
        send_log(self.events, format!("Pyramid operating state: {message}"));
        let _ = self
            .events
            .send(WorkerEvent::PyramidSessionProgress(SessionProgress {
                context: self.context.clone(),
                run_id: self.request.run_id,
                board_generation: generation,
                phase,
                effective_method: self.method,
                observed_game: self.observed_game,
                message,
            }));
    }


    fn entry(&mut self, mode: WorkflowMode) -> Result<Entry, String> {
        pyramid_reconnaissance::validate_request(self.context, &self.request.reconnaissance)?;
        if mode == WorkflowMode::SingleBoard
            || self.request.stop_condition == StopCondition::NoChallenge
        {
            // These entries skip header classification, including when reusing Recon.
            // Refresh inspection pixels under this run before reading any old receipt.
            self.check()?;
            let mut qmp = QmpClient::connect(&self.context.socket_path)
                .map_err(|error| format!("Pyramid entry capture connection failed: {error}"))?;
            self.capture(&mut qmp)?;
            send_log(
                self.events,
                "Explicit Pyramid start captured a fresh read-only entry frame.".to_owned(),
            );
        }
        if let Some(path) = self.reuse_acquisition.as_ref() {
            let (vision, progress) =
                computed_route::reuse_metadata(self.context, path, self.events, self.cancel)?;
            self.vision = Some(vision);
            self.challenge_progress = progress;
        } else {
            self.vision = Some(pyramid_reconnaissance::load_vision(
                &self.request.reconnaissance,
            )?);
            pyramid_reconnaissance::resolve_new_output(&self.request.reconnaissance.output_path)?;
        }
        self.check()?;
        if mode == WorkflowMode::SingleBoard
            || self.request.stop_condition == StopCondition::NoChallenge
        {
            self.starting = None;
            self.series = None;
            self.challenge_starting = None;
            self.challenge_progress = None;
            self.restart_entry = None;
            // Recon itself supplies the two fresh untouched-board observations.
            return Ok(Entry::Board {
                progress: None,
                standard_three: false,
            });
        }
        self.observed_entry()
    }


    fn acquire(&mut self, generation: u64) -> Result<(), String> {
        if generation == 1
            && let Some(path) = self.reuse_acquisition.take()
        {
            let (vision, _) =
                computed_route::reuse_metadata(self.context, &path, self.events, self.cancel)?;
            self.vision = Some(vision);
            self.output = Some(path);
            self.starting = None;
            self.challenge_starting = None;
            send_log(
                self.events,
                "Reusing current-board Recon; no acquisition input or rescan.".to_owned(),
            );
            self.apply_acquisition_default(generation)?;
            self.publish_observed(
                computed_route::acquisition_observed(
                    self.context,
                    self.output.as_ref().ok_or("Missing reused acquisition")?,
                    self.events,
                    self.cancel,
                )?,
                "authenticated reused Recon",
            );
            return Ok(());
        }
        self.reuse_acquisition = None;
        self.output = None;
        self.restart_entry = None;
        // Retire the witness before even output preflight: no failed attempt can reuse it.
        let witness = if self.request.mode == WorkflowMode::Continuous {
            if self.request.stop_condition != StopCondition::RepeatNormalGames {
                None
            } else {
                Some(self.starting.take().ok_or(
                "Continuous acquisition has no qualified current starting-board witness; no input authorised",
            )?)
            }
        } else {
            None
        };
        let request = self.next_request(generation)?;
        let challenge_witness = if self.request.mode == WorkflowMode::Continuous
            && self.request.stop_condition == StopCondition::ChallengeTarget
        {
            Some(self.challenge_starting.take().ok_or("Challenge acquisition has no qualified current starting-board witness; no input authorised")?)
        } else {
            None
        };
        let no_challenge_start = if self.request.mode == WorkflowMode::Continuous
            && self.request.stop_condition == StopCondition::NoChallenge
            && generation > 1
        {
            Some(self.await_no_challenge_start()?)
        } else {
            None
        };
        let result = if let Some(position) = no_challenge_start {
            let header = |_: &CapturedFrame| Ok(());
            pyramid_reconnaissance::run_owned_with_starting_guard(
                self.context,
                &request,
                self.request.run_id,
                generation,
                pyramid_reconnaissance::StartingGuard {
                    position: &position,
                    header: &header,
                },
                self.events,
                self.cancel,
            )
        } else if let Some(witness) = challenge_witness {
            let header = |frame: &CapturedFrame| {
                let actual = crate::pyramid_progress::observe_challenge(frame)?;
                if actual == Some(witness.progress) {
                    Ok(())
                } else {
                    Err(format!(
                        "Reconnaissance challenge requirement changed or became unknown: expected {:?}, observed {actual:?}; no further input authorised",
                        witness.progress
                    ))
                }
            };
            pyramid_reconnaissance::run_owned_with_starting_guard(
                self.context,
                &request,
                self.request.run_id,
                generation,
                pyramid_reconnaissance::StartingGuard {
                    position: &witness.position,
                    header: &header,
                },
                self.events,
                self.cancel,
            )
        } else if let Some(witness) = witness {
            let header = |frame: &CapturedFrame| {
                let actual = observed_progress(frame)?;
                if actual == Some(witness.progress) {
                    Ok(())
                } else {
                    Err(format!(
                        "Reconnaissance series header changed or became unknown: expected {:?}, observed {actual:?}; no further input authorised",
                        witness.progress,
                    ))
                }
            };
            let guard = pyramid_reconnaissance::StartingGuard {
                position: &witness.position,
                header: &header,
            };
            pyramid_reconnaissance::run_owned_with_starting_guard(
                self.context,
                &request,
                self.request.run_id,
                generation,
                guard,
                self.events,
                self.cancel,
            )
        } else {
            pyramid_reconnaissance::run_owned(
                self.context,
                &request,
                self.request.run_id,
                generation,
                self.events,
                self.cancel,
            )
        };
        self.failure_saved = result.is_err();
        self.output = Some(result?);
        self.apply_acquisition_default(generation)?;
        self.publish_observed(
            computed_route::acquisition_observed(
                self.context,
                self.output.as_ref().ok_or("Missing restored acquisition")?,
                self.events,
                self.cancel,
            )?,
            "authenticated restored Recon",
        );
        Ok(())
    }


    fn plan(&mut self, generation: u64) -> Result<(), String> {
        let output = self
            .output
            .as_ref()
            .ok_or("No current published acquisition")?;
        let result = computed_route::calculate_owned_with_policy(
            self.context,
            self.request.run_id,
            output,
            self.method,
            generation,
            self.events,
            self.cancel,
            self.request.stop_condition == StopCondition::NoChallenge,
        );
        self.failure_saved = result.is_err();
        let report = result?;
        self.method = report.method;
        self.phase(generation, SessionPhase::Planning);
        send_log(self.events, report.message);
        Ok(())
    }


    fn execute(&mut self, generation: u64) -> Result<BoardCompletion, String> {
        self.starting = None;
        self.restart_entry = None;
        let result = if self.request.mode == WorkflowMode::Continuous
            && self.request.stop_condition == StopCondition::ChallengeTarget
        {
            let previous = std::cell::Cell::new(self.challenge_progress.ok_or(
                "Continuous challenge execution lacks its bound requirement; no input authorised",
            )?);
            let header = |frame: &CapturedFrame| {
                let actual = crate::pyramid_progress::observe_challenge(frame)?;
                if let Some(current) = actual
                    && current.follows(previous.get())
                {
                    previous.set(current);
                    Ok(())
                } else {
                    Err(format!(
                        "Execution challenge requirement changed, regressed or became unknown: expected {:?}, observed {actual:?}; no further input authorised",
                        previous.get()
                    ))
                }
            };
            computed_route::execute_session(
                self.context,
                self.request.run_id,
                generation,
                computed_route::ExecutionPolicy::Continuous,
                Some(&header),
                self.events,
                self.cancel,
            )
        } else if self.request.mode == WorkflowMode::Continuous
            && self.request.stop_condition == StopCondition::NoChallenge
        {
            computed_route::execute_session(
                self.context,
                self.request.run_id,
                generation,
                computed_route::ExecutionPolicy::NoChallenge,
                None,
                self.events,
                self.cancel,
            )
        } else if self.request.mode == WorkflowMode::Continuous {
            let progress = self.series.ok_or(
                "Continuous execution has no qualified current series witness; no input authorised",
            )?;
            let header = |frame: &CapturedFrame| {
                let actual = observed_progress(frame)?;
                if actual == Some(progress) {
                    Ok(())
                } else {
                    Err(format!(
                        "Execution series header changed or became unknown: expected {progress:?}, observed {actual:?}; no further input authorised",
                    ))
                }
            };
            computed_route::execute_owned_with_frame_guard(
                self.context,
                self.request.run_id,
                self.request.run_id,
                true,
                generation,
                &header,
                self.events,
                self.cancel,
            )
        } else {
            computed_route::execute_owned(
                self.context,
                self.request.run_id,
                self.request.run_id,
                true,
                generation,
                self.events,
                self.cancel,
            )
        };
        self.failure_saved = result.is_err();
        let report = result?;
        if let Some(progress) = report.challenge_progress {
            self.challenge_progress = Some(progress);
        }
        if let Some(progress) = self.series
            && progress.completed.checked_add(1) == Some(BOARDS_PER_GAME)
            && matches!(
                report.completion,
                BoardCompletion::BoardClear | BoardCompletion::GameComplete
            )
        {
            self.restart_entry = Some(post_game::PyramidRestartEntry::verified_third_completion(
                progress,
                report.completion,
            )?);
        }
        self.failure_saved = report.completion == BoardCompletion::DeliveredUnconfirmed;
        self.latest = report.latest_frame;
        self.vision = Some(report.vision);
        send_log(self.events, report.message);
        Ok(report.completion)
    }


    fn transition(&mut self) -> Result<Entry, String> {
        self.starting = None;
        self.challenge_starting = None;
        let mut qmp =
            QmpClient::connect(&self.context.socket_path).map_err(|error| error.to_string())?;
        let mut previous = NativeObservation::Unknown;
        for observation in 0..TRANSITION_OBSERVATIONS {
            if observation > 0 {
                self.wait()?;
            }
            let frame = self.capture(&mut qmp)?;
            let current = self.observe(&frame)?;
            if let Some(entry) = paired_entry(&previous, &current)
                && !matches!(entry, Entry::Board { progress: None, .. })
            {
                self.starting = starting_witness(entry, &current);
                self.challenge_starting = challenge_witness(entry, &current);
                if let Some(witness) = &self.challenge_starting {
                    self.challenge_progress = Some(witness.progress);
                    self.series = None;
                    self.restart_entry = None;
                }
                if let Some(witness) = &self.starting {
                    self.series = Some(witness.progress);
                    self.restart_entry = None;
                }
                return Ok(entry);
            }
            previous = current;
        }
        Err("No positively confirmed next board or terminal scene within transition budget; no input sent".to_owned())
    }


    fn restart(&mut self) -> Result<Entry, String> {
        self.reuse_acquisition = None;
        self.check()?;
        let restart_entry = self.restart_entry.take();
        self.starting = None;
        self.series = None;
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or("Session board generation exhausted")?;
        let next = self.next_request(next_generation)?;
        pyramid_reconnaissance::resolve_new_output(&next.output_path)?;
        let frame = self
            .latest
            .clone()
            .ok_or("Missing verified terminal observation")?;
        if restart_entry.is_none() && !post_game::recognised_pyramid_terminal(&frame)? {
            return Err("Restart lacks supported positive Pyramid terminal evidence".to_owned());
        }
        let mut scan = TableauScanState::for_mode(GameMode::Pyramid);
        let (initial, _) = analyse_captured_frame(frame, &scan)?;
        let mut completed = usize::from(BOARDS_PER_GAME);
        let mut qmp =
            QmpClient::connect(&self.context.socket_path).map_err(|error| error.to_string())?;
        let vision = self.vision.as_ref().ok_or("Missing acquired calibration")?;
        let outcome = match restart_entry {
            Some(entry) => post_game::run_witnessed_pyramid_restart_retaining_frame(
                &mut qmp,
                &self.context.socket_path,
                &initial,
                entry,
                &mut scan,
                &mut completed,
                self.events,
                self.cancel,
                vision,
                &mut self.latest,
            ),
            None => post_game::run_independent_pyramid_restart_retaining_frame(
                &mut qmp,
                &self.context.socket_path,
                Some(&initial),
                &mut scan,
                &mut completed,
                self.events,
                self.cancel,
                vision,
                &mut self.latest,
            ),
        };
        let outcome = outcome?;
        self.check()?;
        let observation =
            match outcome {
                post_game::PostGameOutcome::Restarted(observation)
                | post_game::PostGameOutcome::RecoveredGameplay(observation) => observation,
                post_game::PostGameOutcome::Completed => return Err(
                    "Normal restart returned completion-only control; no acquisition authorised"
                        .to_owned(),
                ),
            };
        self.latest = Some(observation.frame);
        // The compatibility controller's local counter is not series-progress evidence.
        // Recovered gameplay must pass the same fresh actual-header pair as a restart.
        self.observed_entry()
    }
}


/// Run and retire one automatic reservation, emitting exactly one session completion.
pub(super) fn run(
    context: &ControllerContext,
    request: &SessionRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let mut io = Native {
        context,
        request,
        events,
        cancel,
        phase: SessionPhase::Entry,
        generation: 0,
        vision: None,
        output: None,
        latest: None,
        starting: None,
        series: None,
        challenge_starting: None,
        challenge_progress: None,
        restart_entry: None,
        failure_saved: false,
        method: request.method,
        reuse_acquisition: request.reuse_acquisition.clone(),
        observed_game: crate::pyramid_workflow_model::ObservedGame::Unknown,
    };
    send_log(
        events,
        format!(
            "Pyramid operating request: mode={}; selected stop condition={}; initial route objective={}; method={}; game header is independently observed, never inferred from these choices.",
            request.mode.label(),
            request.stop_condition.label(),
            request.route_goal.label(),
            request.method.label()
        ),
    );
    let mut result = if request.method != request.route_goal.method() {
        Err("Route outcome and search policy disagree; no input authorised".to_owned())
    } else if request.mode == WorkflowMode::Continuous
        && request.stop_condition == StopCondition::ChallengeTarget
    {
        challenge::drive(&mut io)
    } else if request.mode == WorkflowMode::Continuous
        && request.stop_condition == StopCondition::NoChallenge
    {
        drive_no_challenge(&mut io)
    } else {
        drive(request.mode, &mut io)
    };
    if let Err(error) = &result
        && !io.failure_saved
    {
        let phase = match io.phase {
            SessionPhase::Entry | SessionPhase::Reconnaissance => PyramidPhase::Reconnaissance,
            SessionPhase::Planning => PyramidPhase::Planning,
            SessionPhase::Execution
            | SessionPhase::Transition
            | SessionPhase::Completion
            | SessionPhase::Restart => PyramidPhase::Execution,
        };
        publish_pyramid_failure(
            events,
            FailureEvidence {
                context,
                phase,
                run_id: Some(request.run_id),
                error,
                details: serde_json::json!({"automatic_phase": io.phase.label(), "board_generation": io.generation,
                "mode": request.mode.label(), "method": request.method.label(), "route_objective": request.route_goal.label(), "stop_condition": request.stop_condition.label(), "no_resume": true,
                "no_cleanup_input": true, "published_output_debug": format!("{:?}", io.output)}),
                source_document: None,
                first_frame: None,
                latest_frame: io.latest.as_ref(),
            },
        );
    }
    match events.latest_frame.computed_route.lock() {
        Ok(mut store) => *store = computed_route::Store::default(),
        Err(_) => {
            result = Err("Session receipt retirement lock poisoned; restart required".to_owned())
        }
    }
    events.latest_frame.reconnaissance.finish();
    send_state(
        events,
        if result.is_ok() {
            WorkerState::Ready
        } else {
            WorkerState::Uncertain
        },
    );
    let message = result.as_ref().map_or_else(
        |error| format!("Pyramid automatic session stopped: {error}"),
        Clone::clone,
    );
    send_status(events, message.clone());
    send_log(events, message);
    let _ = events.send(WorkerEvent::PyramidSessionFinished(SessionCompletion {
        context: context.clone(),
        run_id: request.run_id,
        result,
    }));
}


#[cfg(test)]
mod tests;


#[cfg(test)]
mod native_tests;
