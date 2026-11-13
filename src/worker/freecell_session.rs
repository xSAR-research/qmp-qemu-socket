//! Worker-owned automatic Free Cell session across Recon, A* planning, execution, and post-win restart.
//!
//! Only normal 52-card deals are played automatically. Post-win restarts reuse the
//! parameterised [`post_game`] controller (`GAME WIN` -> optional `LEVEL UP` -> `NEW GAME` -> `PLAY`)
//! with no Solver or CV clicks. Exhaustively unsolvable deals (`Option 2A`) end cleanly
//! with zero mid-game guest input and no manufactured error packet.

use super::*;
use crate::{
    freecell_recon::objective::{self, Objective},
    geometry::PixelPoint,
    pyramid_workflow_model::{
        FreeCellPlayStyle, FreeCellSessionCompletion, FreeCellSessionProgress,
        FreeCellSessionRequest, RunTermination, SessionPhase,
    },
};
use std::{cell::Cell, io::Read};
use xsar::games::freecell::search::BestEffortOutcome as Outcome;

/// Number of cards on all four Free Cell foundations at a complete 52-card victory.
const COMPLETE_FOUNDATION_CARDS: u32 = 52;

/// Number of completed boards before emitting progress throttle updates during A* search.
const PROGRESS_EXPANSION_INTERVAL: u64 = 10_000;

/// Immutable 52-card Free Cell deal captured and exported during one session cycle.
#[derive(Clone, Debug)]
struct AcquiredDeal {
    /// Exported unique JSON path for operator inspection and immutable replay provenance.
    path: PathBuf,
    /// Bounded raw JSON bytes loaded back from the published artifact.
    bytes: Vec<u8>,
    /// Recognised immutable 52-card starting deal.
    deal: xsar::games::freecell::InitialDeal,
}

/// Socket-free A* planning outcome passed back into [`drive_session`].
enum PlanDecision {
    /// Replay-verified full 52-card route ready for guarded execution.
    Solved(freecell_route::Prepared),
    /// Exhaustive proof (`foundation_cards < 52`) with zero mid-game guest clicks (`Option 2A`).
    UnsolvablePartial {
        /// Maximum cards reachable on foundations across the explored state space.
        foundation_cards: u8,
        /// Cumulative states expanded by the exhaustive A* search.
        expanded_states: u64,
    },
}

/// Socket-free execution outcome returned to [`drive_session`] after a solved plan runs.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ExecutedBoard {
    /// Verified physical completion reason at the final captured board.
    completion: Option<crate::freecell_route::Completion>,
    /// Foundation card count proved at the final board (`52` required for victory).
    foundation_cards: u32,
}

/// Pure session result returned by [`drive_session`].
#[derive(Clone, Debug, PartialEq, Eq)]
struct SessionOutcome {
    /// Total 52-card Free Cell victories completed during this session.
    completed_games: usize,
    /// Populated only when Option 2A proved an untouched deal unsolvable (`< 52`).
    unsolvable_foundation_cards: Option<u8>,
    /// Last published 52-card deal JSON path, when at least one Recon completed.
    export_path: Option<PathBuf>,
    /// Concise human-readable completion summary.
    summary: String,
}

/// Boundary abstraction isolating capture, QMP, search, and restart I/O for unit tests.
trait SessionIo {
    /// Enforce monotonic cancellation, active controller context, and reservation health.
    fn check(&self) -> Result<(), String>;

    /// Announce the active lifecycle phase and transition cooperative Pause fences.
    fn phase(&mut self, generation: u64, phase: SessionPhase);

    /// Inspect cold session entry and advance past any already-finished terminal/PLAY screen.
    fn prepare_entry(&mut self) -> Result<(), String>;

    /// Capture, recognise, verify `Objective::FullBoard`, and publish one untouched 52-card deal.
    fn acquire(&mut self, generation: u64) -> Result<AcquiredDeal, String>;

    /// Exhaustively search one immutable normal deal, surfacing Option 2A when unsolvable.
    fn plan(&mut self, generation: u64, acquired: &AcquiredDeal) -> Result<PlanDecision, String>;

    /// Execute one verified full-board Free Cell plan and return its proved board outcome.
    fn execute(
        &mut self,
        generation: u64,
        acquired: &AcquiredDeal,
        prepared: freecell_route::Prepared,
    ) -> Result<ExecutedBoard, String>;

    /// Advance across `GAME WIN`, optional `LEVEL UP`, `NEW GAME`, and `PLAY` to a fresh deal.
    fn restart(
        &mut self,
        generation: u64,
        previous_deal: &xsar::games::freecell::InitialDeal,
    ) -> Result<(), String>;
}

/// Execute the Free Cell automatic lifecycle (`ContinuousNormalGames` or `SingleBoard`).
fn drive_session(
    style: FreeCellPlayStyle,
    io: &mut impl SessionIo,
) -> Result<SessionOutcome, String> {
    if !style.uses_session() {
        return Err(
            "Free Cell automatic session requires Continuous normal games or Single board"
                .to_owned(),
        );
    }
    io.check()?;
    io.phase(0, SessionPhase::Entry);
    io.check()?;
    io.prepare_entry()?;

    let mut generation = 0_u64;
    let mut completed_games = 0_usize;
    loop {
        io.check()?;
        generation = generation
            .checked_add(1)
            .ok_or("Free Cell board generation exhausted")?;
        io.phase(generation, SessionPhase::Reconnaissance);
        io.check()?;
        let acquired = io.acquire(generation)?;
        io.check()?;

        io.phase(generation, SessionPhase::Planning);
        io.check()?;
        let prepared = match io.plan(generation, &acquired)? {
            PlanDecision::Solved(prepared) => prepared,
            PlanDecision::UnsolvablePartial {
                foundation_cards,
                expanded_states,
            } => {
                let summary = format!(
                    "Unsolvable Free Cell deal after {completed_games} completed game(s): exhaustive A* reached at most {foundation_cards}/52 foundation cards across {expanded_states} expanded states (export: {}); zero mid-game clicks sent. Deal a new game and press START.",
                    acquired.path.display()
                );
                return Ok(SessionOutcome {
                    completed_games,
                    unsolvable_foundation_cards: Some(foundation_cards),
                    export_path: Some(acquired.path),
                    summary,
                });
            }
        };
        io.check()?;

        io.phase(generation, SessionPhase::Execution);
        io.check()?;
        let executed = io.execute(generation, &acquired, prepared)?;
        io.check()?;
        if executed.completion != Some(crate::freecell_route::Completion::FullBoard)
            || executed.foundation_cards != COMPLETE_FOUNDATION_CARDS
        {
            return Err(format!(
                "Free Cell route ended without a proved 52-card victory ({:?}, {}/52); no restart input authorised",
                executed.completion, executed.foundation_cards
            ));
        }

        completed_games = completed_games
            .checked_add(1)
            .ok_or("Completed Free Cell game counter exhausted")?;
        io.phase(generation, SessionPhase::Completion);

        if style == FreeCellPlayStyle::SingleBoard {
            let summary = format!(
                "Single board complete — 52/52 cards cleared (board {generation}, export: {}); leaving GAME WIN untouched.",
                acquired.path.display()
            );
            return Ok(SessionOutcome {
                completed_games,
                unsolvable_foundation_cards: None,
                export_path: Some(acquired.path),
                summary,
            });
        }

        io.check()?;
        io.phase(generation, SessionPhase::Restart);
        io.check()?;
        io.restart(generation, &acquired.deal)?;
    }
}

/// Production [`SessionIo`] adapter holding the active worker context and latest captured frame.
struct Native<'a> {
    /// Causal STOP/cancellation tracker local to this session.
    stop_trace: super::pyramid_stop::StopTrace,
    /// First cooperative phase/pause error preserved ahead of later STOP polling.
    phase_error: Option<String>,
    /// Controller identity validated at every capture and input fence.
    context: &'a ControllerContext,
    /// Immutable operator request options for this session.
    request: &'a FreeCellSessionRequest,
    /// Shared worker UI event channel and authority slots.
    events: &'a WorkerEventSink,
    /// Monotonic session cancellation latch.
    cancel: &'a AtomicBool,
    /// Active lifecycle phase stored for diagnostics and UI progress.
    phase: SessionPhase,
    /// Active one-based board cycle index (`0` during `Entry`).
    generation: u64,
    /// Number of 52-card Free Cell victories completed so far.
    completed_games: usize,
    /// Most recent exported JSON path for failure diagnostics and UI display.
    export_path: Option<PathBuf>,
    /// Latest captured frame retained for cold entry and post-win restarts.
    latest: Option<CapturedFrame>,
    /// Set when an inner worker (Recon/Execution) already persisted a failure packet.
    failure_saved: bool,
}

impl Native<'_> {
    /// Emit structured UI progress for the active cycle while preserving existing fields.
    fn emit_progress(
        &self,
        foundation_cards: Option<u8>,
        route_cursor: Option<(usize, usize)>,
        expanded_states: Option<u64>,
        message: String,
    ) {
        let _ = self.events.send(WorkerEvent::FreeCellSessionProgress(
            FreeCellSessionProgress {
                context: self.context.clone(),
                run_id: self.request.run_id,
                deal_generation: self.generation,
                completed_games: self.completed_games,
                phase: self.phase,
                foundation_cards,
                route_cursor,
                expanded_states,
                export_path: self.export_path.clone(),
                message,
            },
        ));
    }

    /// Capture one frame via the causal TCP pipeline so the PNG cache stays populated
    /// for default `GAME WIN` recording.
    fn capture_causal(&mut self, qmp: &mut QmpClient) -> Result<CapturedFrame, String> {
        self.check()?;
        send_state(self.events, WorkerState::Capturing);
        let (frame, _) = capture_screen_causal(qmp, &self.context.socket_path, self.events)
            .map_err(|error| {
                self.stop_trace.capture_error(
                    error,
                    &self.events.latest_frame.reconnaissance,
                    self.context,
                    self.cancel,
                )
            })?;
        self.latest = Some(frame.clone());
        self.events.publish_diagnostic_frame(frame.clone());
        self.check()?;
        Ok(frame)
    }

    /// Construct a unique temporary JSON export path free of hardcoded user directories.
    fn next_export_destination(&self, generation: u64) -> PathBuf {
        std::env::temp_dir().join(format!(
            "freecell-session-{}-{}-g{generation}.json",
            std::process::id(),
            self.request.run_id
        ))
    }
}

impl SessionIo for Native<'_> {
    fn check(&self) -> Result<(), String> {
        if let Some(error) = &self.phase_error {
            return Err(error.clone());
        }
        self.stop_trace.check(
            &self.events.latest_frame.reconnaissance,
            self.context,
            self.cancel,
        )
    }

    fn phase(&mut self, generation: u64, phase: SessionPhase) {
        self.generation = generation;
        self.phase = phase;
        let pause_phase = match phase {
            SessionPhase::Planning => Some(super::pyramid_pause::Phase::Search),
            SessionPhase::Reconnaissance | SessionPhase::Entry => {
                Some(super::pyramid_pause::Phase::Recon)
            }
            SessionPhase::Execution | SessionPhase::Restart => {
                Some(super::pyramid_pause::Phase::Gameplay)
            }
            SessionPhase::Transition | SessionPhase::Completion => None,
        };
        if let Some(pause_phase) = pause_phase
            && let Err(error) = self
                .events
                .latest_frame
                .pause
                .checkpoint(pause_phase, || self.check())
        {
            if self.stop_trace.termination(true, true) != RunTermination::OperatorStopped {
                self.phase_error = Some(error.message.clone());
            }
            self.cancel.store(true, Ordering::Release);
            send_log(self.events, error.message);
            return;
        }
        if phase == SessionPhase::Completion {
            self.completed_games = self.completed_games.saturating_add(1);
            self.emit_progress(
                Some(52),
                None,
                None,
                format!(
                    "Completed Free Cell game {} (board {}).",
                    self.completed_games, self.generation
                ),
            );
            return;
        }
        let phase_message = format!("Free Cell session cycle {}: {}", generation, phase.label());
        send_status(self.events, phase_message.clone());
        self.emit_progress(None, None, None, phase_message);
    }

    fn prepare_entry(&mut self) -> Result<(), String> {
        self.check()?;
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|error| format!("Free Cell entry QMP connection failed: {error}"))?;
        let frame = self.capture_causal(&mut qmp)?;
        let mut scan = TableauScanState::for_mode(GameMode::FreeCell);
        let (observation, _) = analyse_captured_frame(frame, &scan)?;
        let is_terminal = crate::freecell_terminal::is_recognised_terminal(&observation.frame)
            .map_err(|error| error.to_string())?;
        let play_visible = !observation.gameplay_scene
            && crate::freecell_terminal::expected_control_point(
                &observation.frame,
                crate::freecell_terminal::TerminalStage::Play,
            )
            .map_err(|error| error.to_string())?
                != PixelPoint::new(0, 0);
        if !is_terminal && !play_visible {
            return Ok(());
        }
        send_log(
            self.events,
            "Free Cell cold entry detected post-game/PLAY screen; advancing to a fresh normal deal."
                .to_owned(),
        );
        let mut completed_boards = self.completed_games;
        let outcome = post_game::run_independent_freecell_restart_retaining_frame(
            post_game::FreeCellRestartContext {
                qmp: &mut qmp,
                socket_path: &self.context.socket_path,
                initial_observation: Some(&observation),
                previous_deal: None,
                scan_state: &mut scan,
                completed_boards: &mut completed_boards,
                event_tx: self.events,
                cancel_requested: self.cancel,
                latest_frame: &mut self.latest,
                authority: post_game::RestartAuthority {
                    context: self.context,
                    trace: &self.stop_trace,
                },
            },
        )?;
        match outcome {
            post_game::PostGameOutcome::Restarted(obs)
            | post_game::PostGameOutcome::RecoveredGameplay(obs) => {
                self.latest = Some(obs.frame);
                Ok(())
            }
            post_game::PostGameOutcome::Completed => Err(
                "Free Cell cold-entry restart ended without proving a fresh 52-card deal"
                    .to_owned(),
            ),
        }
    }

    fn acquire(&mut self, generation: u64) -> Result<AcquiredDeal, String> {
        self.check()?;
        let destination = self.next_export_destination(generation);
        let deal_id = format!("freecell-session-{}-g{generation}", self.request.run_id);
        let request = freecell_reconnaissance::Request {
            destination: destination.clone(),
            deal_id,
        };
        let mut acquisition = freecell_reconnaissance::Acquisition::new(self.request.run_id);
        send_state(self.events, WorkerState::Capturing);
        let (report, deal) = freecell_reconnaissance::run_checked_with_deal(
            self.context,
            &request,
            self.events,
            self.cancel,
            &self.stop_trace,
            &mut acquisition,
        )
        .map_err(|failure| {
            let report = freecell_reconnaissance::failure_report(
                self.context,
                &request,
                &acquisition,
                failure,
            );
            self.failure_saved = true;
            report.error
        })?;
        self.export_path = Some(report.destination.clone());
        let objective = objective::from_metadata(deal.metadata());
        if objective != Objective::FullBoard {
            return Err(format!(
                "Free Cell automatic normal games only plays FullBoard normal deals; acquired objective was {} (no planning or guest input authorised)",
                objective.label()
            ));
        }
        let file = std::fs::File::open(&report.destination).map_err(|error| {
            format!(
                "Free Cell exported deal read failed ({}): {error}",
                report.destination.display()
            )
        })?;
        let mut bytes = Vec::new();
        file.take((xsar::freecell_json::MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Free Cell exported deal read: {error}"))?;
        if bytes.len() > xsar::freecell_json::MAX_DOCUMENT_BYTES {
            return Err(format!(
                "Free Cell exported deal exceeds {} bytes",
                xsar::freecell_json::MAX_DOCUMENT_BYTES
            ));
        }
        send_log(
            self.events,
            format!(
                "Free Cell Recon complete (cycle {generation}): normal FullBoard deal exported to {}.",
                report.destination.display()
            ),
        );
        self.emit_progress(
            Some(0),
            None,
            None,
            format!("Acquired normal 52-card deal (cycle {generation})."),
        );
        Ok(AcquiredDeal {
            path: report.destination,
            bytes,
            deal,
        })
    }

    fn plan(&mut self, generation: u64, acquired: &AcquiredDeal) -> Result<PlanDecision, String> {
        self.check()?;
        let identity = freecell_calculation::Identity {
            context: self.context.clone(),
            request_id: self.request.run_id,
            source_generation: generation,
        };
        let last_emitted_expanded = Cell::new(0_u64);
        let last_emitted_foundations = Cell::new(0_u8);
        let events = self.events;
        let stop_trace = &self.stop_trace;
        let context = self.context;
        let cancel = self.cancel;
        let completed_games = self.completed_games;
        let run_id = self.request.run_id;
        let export_path = &self.export_path;

        let report = freecell_calculation::search_deal_with_control(
            freecell_calculation::SearchInput {
                deal: acquired.deal.clone(),
                path: acquired.path.clone(),
                bytes: acquired.bytes.clone(),
                cancel: self.cancel,
                events: self.events,
            },
            || match stop_trace.check(&events.latest_frame.reconnaissance, context, cancel) {
                Ok(()) => Ok(false),
                Err(_) if cancel.load(Ordering::Acquire) => Ok(true),
                Err(error) => Err(error),
            },
            |search, _paused| {
                let foundations = u8::try_from(search.best_foundation_cards).unwrap_or(u8::MAX);
                let expanded = u64::try_from(search.search.stats.expanded).unwrap_or(u64::MAX);
                let should_emit = expanded == 0
                    || foundations > last_emitted_foundations.get()
                    || expanded.saturating_sub(last_emitted_expanded.get())
                        >= PROGRESS_EXPANSION_INTERVAL;
                if should_emit {
                    last_emitted_expanded.set(expanded);
                    last_emitted_foundations.set(foundations);
                    let _ = events.send(WorkerEvent::FreeCellSessionProgress(
                        FreeCellSessionProgress {
                            context: context.clone(),
                            run_id,
                            deal_generation: generation,
                            completed_games,
                            phase: SessionPhase::Planning,
                            foundation_cards: Some(foundations),
                            route_cursor: None,
                            expanded_states: Some(expanded),
                            export_path: export_path.clone(),
                            message: format!(
                                "Searching Free Cell cycle {generation}: {foundations}/52 best foundations ({expanded} expanded)",
                            ),
                        },
                    ));
                }
            },
        )?;

        match &report.outcome {
            Outcome::Solved { moves, stats, .. } => {
                send_log(
                    self.events,
                    format!(
                        "Free Cell A* solved cycle {generation} in {} moves ({} expanded states).",
                        moves.len(),
                        stats.expanded
                    ),
                );
                let prepared =
                    freecell_route::Prepared::new(identity, Arc::new(report), self.cancel)
                        .map_err(|error| error.to_string())?;
                Ok(PlanDecision::Solved(prepared))
            }
            Outcome::Partial {
                foundation_cards,
                stats,
                ..
            } => {
                let foundation_cards = u8::try_from(*foundation_cards)
                    .map_err(|_| "Invalid Free Cell foundation count".to_owned())?;
                let expanded_states = u64::try_from(stats.expanded).unwrap_or(u64::MAX);
                send_log(
                    self.events,
                    format!(
                        "Option 2A unsolvable Free Cell deal proved cleanly: max {foundation_cards}/52 foundation cards across {expanded_states} expanded states (export: {}); zero mid-game guest clicks.",
                        acquired.path.display()
                    ),
                );
                Ok(PlanDecision::UnsolvablePartial {
                    foundation_cards,
                    expanded_states,
                })
            }
            Outcome::Cancelled(_) => {
                self.check()?;
                Err("Free Cell search cancelled".to_owned())
            }
            Outcome::Failed { failure, .. } => Err(format!("Free Cell search failed: {failure}")),
        }
    }

    fn execute(
        &mut self,
        generation: u64,
        _acquired: &AcquiredDeal,
        prepared: freecell_route::Prepared,
    ) -> Result<ExecutedBoard, String> {
        self.check()?;
        let plan_id = freecell_calculation::Identity {
            context: self.context.clone(),
            request_id: self.request.run_id,
            source_generation: generation,
        };
        let route_run_id = {
            let mut store = self
                .events
                .latest_frame
                .freecell_route
                .lock()
                .map_err(|_| "Free Cell route store poisoned".to_owned())?;
            store.install(Some(prepared))?;
            store.reserve_mode(&plan_id, self.request.auto_mode)?
        };
        self.emit_progress(
            None,
            Some((0, 0)),
            None,
            format!("Executing Free Cell route (cycle {generation})..."),
        );
        let execution = freecell_route::execute_for_session(
            plan_id,
            route_run_id,
            false,
            self.request.settle,
            self.events,
            self.cancel,
        );
        if let Some(frame) = execution.latest_frame {
            self.latest = Some(frame);
        }
        self.stop_trace.adopt(execution.termination);
        match execution.result {
            Ok(progress) => {
                let foundations = u8::try_from(progress.foundation_cards).unwrap_or(52);
                self.emit_progress(
                    Some(foundations),
                    Some((progress.cursor, progress.total)),
                    None,
                    format!(
                        "Executed {}/{} route transfers ({foundations}/52 foundations).",
                        progress.cursor, progress.total
                    ),
                );
                Ok(ExecutedBoard {
                    completion: progress.completion,
                    foundation_cards: progress.foundation_cards,
                })
            }
            Err(failure) => {
                self.failure_saved = true;
                Err(failure.error)
            }
        }
    }

    fn restart(
        &mut self,
        _generation: u64,
        previous_deal: &xsar::games::freecell::InitialDeal,
    ) -> Result<(), String> {
        self.check()?;
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|error| format!("Free Cell post-win QMP connection failed: {error}"))?;
        let frame = match self.latest.take() {
            Some(frame) => frame,
            None => self.capture_causal(&mut qmp)?,
        };
        let mut scan = TableauScanState::for_mode(GameMode::FreeCell);
        let (observation, _) = analyse_captured_frame(frame, &scan)?;
        let mut completed_boards = self.completed_games;
        let outcome = post_game::run_independent_freecell_restart_retaining_frame(
            post_game::FreeCellRestartContext {
                qmp: &mut qmp,
                socket_path: &self.context.socket_path,
                initial_observation: Some(&observation),
                previous_deal: Some(previous_deal),
                scan_state: &mut scan,
                completed_boards: &mut completed_boards,
                event_tx: self.events,
                cancel_requested: self.cancel,
                latest_frame: &mut self.latest,
                authority: post_game::RestartAuthority {
                    context: self.context,
                    trace: &self.stop_trace,
                },
            },
        )?;
        match outcome {
            post_game::PostGameOutcome::Restarted(obs)
            | post_game::PostGameOutcome::RecoveredGameplay(obs) => {
                self.latest = Some(obs.frame);
                Ok(())
            }
            post_game::PostGameOutcome::Completed => Err(
                "Free Cell post-win restart returned completion-only status without proving a fresh deal"
                    .to_owned(),
            ),
        }
    }
}

/// Run and cleanly retire one worker-owned Free Cell automatic session.
pub(super) fn run(
    context: &ControllerContext,
    request: &FreeCellSessionRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let mut io = Native {
        stop_trace: Default::default(),
        phase_error: None,
        context,
        request,
        events,
        cancel,
        phase: SessionPhase::Entry,
        generation: 0,
        completed_games: 0,
        export_path: None,
        latest: None,
        failure_saved: false,
    };
    send_log(
        events,
        format!(
            "Free Cell automatic session started: mode={}; transfer_policy={:?}; settle={} ms.",
            request.play_style.label(),
            request.auto_mode,
            request.settle.as_millis()
        ),
    );
    let mut outcome = drive_session(request.play_style, &mut io);
    let mut termination = io.stop_trace.termination(outcome.is_err(), true);
    if let Err(error) = &outcome
        && termination != RunTermination::OperatorStopped
        && !io.failure_saved
    {
        let phase = match io.phase {
            SessionPhase::Entry | SessionPhase::Reconnaissance => {
                crate::pyramid_diagnostics::PyramidPhase::Reconnaissance
            }
            SessionPhase::Planning => crate::pyramid_diagnostics::PyramidPhase::Planning,
            SessionPhase::Execution
            | SessionPhase::Transition
            | SessionPhase::Completion
            | SessionPhase::Restart => crate::pyramid_diagnostics::PyramidPhase::Execution,
        };
        publish_pyramid_failure(
            events,
            FailureEvidence {
                progress: None,
                context,
                phase,
                run_id: Some(request.run_id),
                error,
                details: serde_json::json!({
                    "game": "Free Cell",
                    "automatic_phase": io.phase.label(),
                    "deal_generation": io.generation,
                    "completed_games": io.completed_games,
                    "play_style": request.play_style.label(),
                    "export_path": io.export_path,
                }),
                source_document: None,
                first_frame: None,
                latest_frame: io.latest.as_ref(),
            },
        );
    }
    match events.latest_frame.freecell_route.lock() {
        Ok(mut store) => store.revoke(),
        Err(_) => {
            outcome =
                Err("Free Cell session retirement lock poisoned; restart required".to_owned());
            termination = RunTermination::Failed;
        }
    }
    events.latest_frame.reconnaissance.finish();
    send_state(
        events,
        if outcome.is_ok() || termination == RunTermination::OperatorStopped {
            WorkerState::Ready
        } else {
            WorkerState::Uncertain
        },
    );
    let (completed_games, unsolvable_foundation_cards, export_path, result) = match outcome {
        Ok(done) => (
            done.completed_games,
            done.unsolvable_foundation_cards,
            done.export_path,
            Ok(done.summary),
        ),
        Err(error) => (io.completed_games, None, io.export_path, Err(error)),
    };
    let message = result.as_ref().map_or_else(
        |error| format!("Free Cell automatic session stopped: {error}"),
        Clone::clone,
    );
    send_status(events, message.clone());
    send_log(events, message);
    let _ = events.send(WorkerEvent::FreeCellSessionFinished(
        FreeCellSessionCompletion {
            context: context.clone(),
            run_id: request.run_id,
            completed_games,
            termination,
            unsolvable_foundation_cards,
            export_path,
            result,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use xsar::freecell_json;
    use xsar::games::cards::{Rank, Suit};
    use xsar::games::freecell::{BoardState, CascadeStack, InitialDeal, Metadata};

    fn test_deal(seed: u8) -> InitialDeal {
        let mut cards = Vec::with_capacity(52);
        for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for rank in 1_u8..=13 {
                cards.push(xsar::games::cards::Card::new(
                    Rank::from_value(rank).unwrap(),
                    suit,
                ));
            }
        }
        cards.rotate_left(usize::from(seed) % 52);
        let mut iter = cards.into_iter();
        let columns = std::array::from_fn(|column| {
            let count = if column < 4 { 7 } else { 6 };
            CascadeStack::from_cards(iter.by_ref().take(count).collect::<Vec<_>>()).unwrap()
        });
        let metadata = Metadata::new(
            format!("session-test-{seed}"),
            "freecell-session-test",
            crate::freecell_recon::objective::FULL_BOARD_SOURCE,
        )
        .unwrap();
        InitialDeal::new(
            BoardState::new(columns, [None; 4], [0; 4]).unwrap(),
            metadata,
        )
        .unwrap()
    }

    fn dummy_prepared(deal: &InitialDeal) -> freecell_route::Prepared {
        let context = ControllerContext {
            socket_path: PathBuf::from("/tmp/freecell-session-test.sock"),
            game_mode: GameMode::FreeCell,
            strategy: SolvingStrategy::ShortestPath,
            generation: 1,
        };
        let report = freecell_calculation::Report {
            path: PathBuf::from("/tmp/dummy-session-deal.json"),
            deal: deal.clone(),
            bytes: freecell_json::to_vec(deal).unwrap(),
            outcome: Outcome::Cancelled(xsar::search::SearchStats::default()),
            paused: Duration::ZERO,
        };
        freecell_route::Prepared::new(
            freecell_calculation::Identity {
                context,
                request_id: 1,
                source_generation: 1,
            },
            Arc::new(report),
            &AtomicBool::new(false),
        )
        .unwrap()
    }

    enum ScriptedPlan {
        Solve,
        Partial {
            foundation_cards: u8,
            expanded_states: u64,
        },
    }

    #[derive(Default)]
    struct ScriptedIo {
        events: Vec<String>,
        check_errors: VecDeque<Result<(), String>>,
        acquires: VecDeque<Result<AcquiredDeal, String>>,
        plans: VecDeque<Result<ScriptedPlan, String>>,
        executions: VecDeque<Result<ExecutedBoard, String>>,
        restarts: VecDeque<Result<(), String>>,
    }

    impl ScriptedIo {
        fn make_acquired(seed: u8) -> AcquiredDeal {
            let deal = test_deal(seed);
            let bytes = freecell_json::to_vec(&deal).unwrap();
            AcquiredDeal {
                path: PathBuf::from(format!("/tmp/freecell-session-script-{seed}.json")),
                bytes,
                deal,
            }
        }
    }

    impl SessionIo for ScriptedIo {
        fn check(&self) -> Result<(), String> {
            self.check_errors.front().cloned().unwrap_or(Ok(()))
        }

        fn phase(&mut self, generation: u64, phase: SessionPhase) {
            self.events.push(format!("phase:{generation}:{phase:?}"));
        }

        fn prepare_entry(&mut self) -> Result<(), String> {
            self.events.push("prepare_entry".to_owned());
            Ok(())
        }

        fn acquire(&mut self, generation: u64) -> Result<AcquiredDeal, String> {
            self.events.push(format!("acquire:{generation}"));
            self.acquires
                .pop_front()
                .unwrap_or_else(|| Err("unexpected acquire".to_owned()))
        }

        fn plan(
            &mut self,
            generation: u64,
            acquired: &AcquiredDeal,
        ) -> Result<PlanDecision, String> {
            self.events.push(format!("plan:{generation}"));
            match self
                .plans
                .pop_front()
                .unwrap_or_else(|| Err("unexpected plan".to_owned()))?
            {
                ScriptedPlan::Solve => Ok(PlanDecision::Solved(dummy_prepared(&acquired.deal))),
                ScriptedPlan::Partial {
                    foundation_cards,
                    expanded_states,
                } => Ok(PlanDecision::UnsolvablePartial {
                    foundation_cards,
                    expanded_states,
                }),
            }
        }

        fn execute(
            &mut self,
            generation: u64,
            _acquired: &AcquiredDeal,
            _prepared: freecell_route::Prepared,
        ) -> Result<ExecutedBoard, String> {
            self.events.push(format!("execute:{generation}"));
            self.executions
                .pop_front()
                .unwrap_or_else(|| Err("unexpected execute".to_owned()))
        }

        fn restart(
            &mut self,
            generation: u64,
            _previous_deal: &xsar::games::freecell::InitialDeal,
        ) -> Result<(), String> {
            self.events.push(format!("restart:{generation}"));
            self.restarts
                .pop_front()
                .unwrap_or_else(|| Err("unexpected restart".to_owned()))
        }
    }

    #[test]
    fn freecell_330_session_continuous_loop_solves_restarts_and_exits_cleanly_on_unsolvable() {
        let mut io = ScriptedIo::default();
        io.acquires.push_back(Ok(ScriptedIo::make_acquired(1)));
        io.plans.push_back(Ok(ScriptedPlan::Solve));
        io.executions.push_back(Ok(ExecutedBoard {
            completion: Some(crate::freecell_route::Completion::FullBoard),
            foundation_cards: 52,
        }));
        io.restarts.push_back(Ok(()));

        io.acquires.push_back(Ok(ScriptedIo::make_acquired(2)));
        io.plans.push_back(Ok(ScriptedPlan::Solve));
        io.executions.push_back(Ok(ExecutedBoard {
            completion: Some(crate::freecell_route::Completion::FullBoard),
            foundation_cards: 52,
        }));
        io.restarts.push_back(Ok(()));

        io.acquires.push_back(Ok(ScriptedIo::make_acquired(3)));
        io.plans.push_back(Ok(ScriptedPlan::Partial {
            foundation_cards: 41,
            expanded_states: 12_345,
        }));

        let outcome = drive_session(FreeCellPlayStyle::ContinuousNormalGames, &mut io).unwrap();
        assert_eq!(outcome.completed_games, 2);
        assert_eq!(outcome.unsolvable_foundation_cards, Some(41));
        assert_eq!(
            outcome.export_path.as_deref(),
            Some(Path::new("/tmp/freecell-session-script-3.json"))
        );
        assert!(!io.events.iter().any(|event| event == "execute:3"));
        assert!(!io.events.iter().any(|event| event == "restart:3"));
        assert!(io.events.contains(&"prepare_entry".to_owned()));
        assert!(io.events.contains(&"restart:1".to_owned()));
        assert!(io.events.contains(&"restart:2".to_owned()));
    }

    #[test]
    fn freecell_330_session_single_board_leaves_game_win_untouched_without_restart() {
        let mut io = ScriptedIo::default();
        io.acquires.push_back(Ok(ScriptedIo::make_acquired(1)));
        io.plans.push_back(Ok(ScriptedPlan::Solve));
        io.executions.push_back(Ok(ExecutedBoard {
            completion: Some(crate::freecell_route::Completion::FullBoard),
            foundation_cards: 52,
        }));

        let outcome = drive_session(FreeCellPlayStyle::SingleBoard, &mut io).unwrap();
        assert_eq!(outcome.completed_games, 1);
        assert_eq!(outcome.unsolvable_foundation_cards, None);
        assert!(!io.events.iter().any(|event| event.starts_with("restart:")));
    }

    #[test]
    fn freecell_330_session_manual_mode_and_incomplete_execution_refuse_without_restart() {
        let mut io = ScriptedIo::default();
        let manual_error = drive_session(FreeCellPlayStyle::Manual, &mut io).unwrap_err();
        assert!(manual_error.contains("Continuous normal games"));
        assert!(io.events.is_empty());

        let mut incomplete = ScriptedIo::default();
        incomplete
            .acquires
            .push_back(Ok(ScriptedIo::make_acquired(4)));
        incomplete.plans.push_back(Ok(ScriptedPlan::Solve));
        incomplete.executions.push_back(Ok(ExecutedBoard {
            completion: Some(crate::freecell_route::Completion::PartialRouteExhausted),
            foundation_cards: 30,
        }));
        let error =
            drive_session(FreeCellPlayStyle::ContinuousNormalGames, &mut incomplete).unwrap_err();
        assert!(error.contains("52-card victory"));
        assert!(!incomplete.events.iter().any(|e| e.starts_with("restart:")));
    }

    #[test]
    fn freecell_330_session_stop_and_challenge_refusal_halt_before_input() {
        let mut stopped = ScriptedIo::default();
        stopped
            .check_errors
            .push_back(Err("STOP requested; session cancelled".to_owned()));
        let stop_err =
            drive_session(FreeCellPlayStyle::ContinuousNormalGames, &mut stopped).unwrap_err();
        assert!(stop_err.contains("STOP"));
        assert!(stopped.events.is_empty());

        let mut challenge = ScriptedIo::default();
        challenge.acquires.push_back(Err(
            "FullBoard objective required; challenge refused".to_owned()
        ));
        let challenge_err =
            drive_session(FreeCellPlayStyle::ContinuousNormalGames, &mut challenge).unwrap_err();
        assert!(challenge_err.contains("challenge refused"));
        assert!(!challenge.events.iter().any(|e| e.starts_with("plan:")));
        assert!(!challenge.events.iter().any(|e| e.starts_with("execute:")));

        let mut cold_fail = ScriptedIo::default();
        cold_fail
            .check_errors
            .push_back(Err("Entry terminal lacks Level Up OK".to_owned()));
        let cold_err =
            drive_session(FreeCellPlayStyle::ContinuousNormalGames, &mut cold_fail).unwrap_err();
        assert!(cold_err.contains("Level Up OK"));
        assert!(!cold_fail.events.iter().any(|e| e.starts_with("acquire:")));
    }
}
