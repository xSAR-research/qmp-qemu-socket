#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Automatic multi-board TriPeaks session state machine and Manual RECON runner.
//!
//! Coordinates `T = 0` entry/restart, Stock/probe reconnaissance, A* + multi-board `ClearBoard`
//! escalation planning, verified QMP route playback, Challenge quota/score tracking across deals,
//! partial progress banking vs `0`-click dead-end stops, and `GAME WIN` sequence recording.

use std::{
    cell::Cell,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use super::{
    CaptureBackend, CaptureMode, QmpClient, TableauScanState, WorkerEvent, WorkerEventSink,
    WorkerState, analyse_captured_frame, capture_screen_causal,
    game_operations::{NativeOperations, TrackedOperations},
    post_game, pyramid_pause,
    pyramid_stop::StopTrace,
    selected_capture_backend, send_log, send_state, send_status, tripeaks_calculation,
    tripeaks_reconnaissance::{
        FrameClassification, LiveTriPeaksDomain, TriPeaksReconConfig, authorise_tripeaks_input,
        observe_frame, run_tripeaks_recon, semantic_board_hash,
    },
    tripeaks_route,
};
use crate::{
    capture::CapturedFrame,
    game::GameMode,
    game_operations::OperationError,
    parameters::BOARD_TRANSITION_REOBSERVE_DELAY,
    pyramid_diagnostics::{FailureEvidence, PyramidPhase, persist_failure},
    pyramid_workflow_model::{RunTermination, SessionPhase},
    recon::{Objective, ReconOutcome},
    strategy::{ControllerContext, SolvingStrategy},
    tripeaks_model::{MAX_DOCUMENT_BYTES, TriPeaksDealDocument},
    tripeaks_progress::TriPeaksChallenge,
    tripeaks_workflow_model::{
        SlotKnowledgeCounts, TriPeaksManualReconSummary, TriPeaksPlanningSnapshot,
        TriPeaksPlayStyle, TriPeaksSessionCompletion, TriPeaksSessionProgress,
        TriPeaksSessionRequest, format_tripeaks_objective,
    },
};

/// Minimum wall-clock spacing between throttled TriPeaks planning UI progress updates.
const PLANNING_PROGRESS_INTERVAL: Duration = Duration::from_millis(200);

/// Maximum observation captures after clearing board `1/3` or `2/3` while waiting for the next deal.
const MAX_TRANSITION_CAPTURES: usize = 20;

/// Validated `52`-slot TriPeaks deal returned by `acquire`.
pub(crate) struct AcquiredBoard {
    /// Atomic JSON destination written for this board cycle.
    pub export_path: PathBuf,
    /// Validated `52`-slot deal document.
    pub document: TriPeaksDealDocument,
    /// Raw bounded JSON bytes for offline planning reports.
    pub bytes: Vec<u8>,
    /// Active objective resolved for this board (HUD Challenge or tracked continuation).
    pub objective: Objective,
    /// Live Challenge header detected at `T = 0`, if any.
    pub detected_challenge: Option<TriPeaksChallenge>,
    /// Semantic hash of the verified `T = 0` root board.
    pub root_hash: u64,
    /// `true` when probing during recon finished the board or Challenge directly.
    pub completed_during_recon: bool,
    /// `true` when `completed_during_recon` already reached `Challenge Complete!`.
    pub terminal_challenge_complete: bool,
    /// `true` when `completed_during_recon` already reached Normal 3/3 `Congratulations`.
    pub terminal_board_win: bool,
    /// Plain operator card counts (`Known / Unknown / Unreachable`).
    pub counts: SlotKnowledgeCounts,
    /// Number of `Undo All` probe cycles executed during recon.
    pub probes_used: usize,
}

/// Planning decision produced for one acquired board cycle.
pub(crate) enum PlanDecision<R> {
    /// Complete or escalated `ClearBoard` route independently verified from `T = 0`.
    Solved {
        /// Prepared executable route handle (production [`tripeaks_route::Prepared`] or test route).
        route: R,
        /// `true` when direct quota/score exceeded one deck and escalated to full `28`-card clear.
        escalated_clear_board: bool,
    },
    /// Deal proved unsolvable for both target and `ClearBoard`; carries optional partial progress route.
    UnsolvablePartial {
        /// Positive rank/score progress route to bank, or `None` (`0` clicks for dead-end `ClearBoard`).
        route: Option<R>,
        /// Original board objective proved unsolvable.
        objective: Objective,
        /// Target-rank cards cleared along `route` (`0` outside `ClearRankQuota`).
        banked_rank_count: u8,
        /// Verified score delta along `route`.
        score_delta: u32,
        /// Maximum clearable tableau cards across the known subgraph (`0..=28`).
        cards_cleared: u8,
    },
}

/// Outcome observed after executing one planned or partial route on a board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExecutedBoard {
    /// Physical endpoint classification reported by [`tripeaks_route`].
    pub completion: tripeaks_route::Completion,
    /// Verified tableau cards cleared along the executed moves (`0..=28`).
    pub cards_cleared: u8,
    /// Verified score delta earned along the executed moves.
    pub score_delta: u32,
}

/// Verified board state observed after `BoardCleared` settles into the next screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostClearTransition {
    /// A fresh untouched `T = 0` next board dealt automatically (`new_root_hash != prev`).
    NextBoardDealt(u64),
    /// Normal 3rd-board `Congratulations` or `LEVEL-UP!` reward overlay appeared.
    BoardWinOverlay,
    /// `Challenge Complete!` (or XP/Level-Up Challenge handoff) appeared.
    ChallengeComplete,
}

/// Final summary returned when [`drive_tripeaks_session`] exits without error.
#[derive(Clone, Debug)]
pub(crate) struct SessionOutcome {
    /// Total TriPeaks boards cleared/solved during this session.
    pub completed_boards: usize,
    /// Total full games/challenges completed during this session.
    pub completed_games: usize,
    /// Present when an unsolvable board stopped with zero Redeal clicks.
    pub unsolvable_max_cleared: Option<u8>,
    /// Most recent exported `52`-slot JSON path.
    pub export_path: Option<PathBuf>,
    /// Final card-knowledge counts reported to the UI.
    pub counts: Option<SlotKnowledgeCounts>,
    /// Plain-English description of the final active objective.
    pub objective_label: Option<String>,
    /// Final operator status summary.
    pub summary: String,
}

/// Cross-board Challenge mode and remaining quota/score tracker.
#[derive(Clone, Debug, Default)]
struct ChallengeTracker {
    /// `true` once any Challenge banner has been observed in this session.
    challenge_active: bool,
    /// Tracked Challenge variant derived from `T = 0` headers and cleared deltas.
    last_objective: Option<Objective>,
    /// Tracked remaining `Boards cleared` count when HUD reported `Boards { current, target }`.
    boards_remaining: Option<u32>,
}

impl ChallengeTracker {
    /// Update Challenge tracking upon acquiring a board; returns effective `Objective`.
    fn on_acquired(
        &mut self,
        objective: Objective,
        detected_challenge: Option<TriPeaksChallenge>,
    ) -> Objective {
        if let Some(challenge) = detected_challenge {
            self.challenge_active = true;
            match challenge {
                TriPeaksChallenge::Boards { current, target } => {
                    self.boards_remaining = Some(target.saturating_sub(current));
                }
                TriPeaksChallenge::ClearRank { .. }
                | TriPeaksChallenge::Score { .. }
                | TriPeaksChallenge::SolveDeck
                | TriPeaksChallenge::UnsupportedSingleCard => {
                    self.boards_remaining = None;
                }
            }
            self.last_objective = Some(objective);
            return objective;
        }
        if self.challenge_active {
            let effective = self.last_objective.unwrap_or(objective);
            self.last_objective = Some(effective);
            return effective;
        }
        self.last_objective = Some(objective);
        objective
    }

    /// Update fallback remaining counters after executing a board (used only if HUD lacks live header).
    fn on_board_progressed(&mut self, score_delta: u32, executed_board_clear: bool) {
        if executed_board_clear && let Some(rem) = &mut self.boards_remaining {
            *rem = rem.saturating_sub(1);
        }
        if let Some(Objective::ReachScoreDelta { points_needed }) = self.last_objective {
            let next_needed = points_needed.saturating_sub(score_delta);
            self.last_objective = Some(Objective::ReachScoreDelta {
                points_needed: next_needed.max(1),
            });
        }
    }
}

/// Pure domain abstraction separating [`drive_tripeaks_session`] from QMP and capture I/O.
pub(crate) trait TriPeaksSessionIo {
    /// Concrete executable route type produced by `plan` and consumed by `execute`.
    type Route;

    /// Check cooperative `STOP`, shutdown, and active controller context.
    fn check(&self) -> Result<(), String>;

    /// Notify UI/pause gate of lifecycle phase transitions (`generation == 0` during `Entry`).
    fn phase(&mut self, generation: u64, phase: SessionPhase);

    /// Inspect entry screen and advance out of Normal post-game screens onto a fresh `T = 0` deal.
    fn prepare_entry(&mut self) -> Result<(), String>;

    /// Acquire, sweep/probe, and export one untouched `T = 0` TriPeaks deal.
    fn acquire(
        &mut self,
        generation: u64,
        fallback_objective: Option<Objective>,
        force_challenge: bool,
    ) -> Result<AcquiredBoard, String>;

    /// Plan an A* / escalated `ClearBoard` / banked partial route without sending guest input.
    fn plan(
        &mut self,
        generation: u64,
        acquired: &AcquiredBoard,
    ) -> Result<PlanDecision<Self::Route>, String>;

    /// Execute the prepared route under full card-level verification.
    fn execute(
        &mut self,
        generation: u64,
        acquired: &AcquiredBoard,
        route: Self::Route,
        allow_final_modal: bool,
    ) -> Result<ExecutedBoard, String>;

    /// Wait cleanly for Microsoft Solitaire to finish transitioning after all `28` tableau cards clear.
    fn wait_next_board_after_clear(
        &mut self,
        generation: u64,
        previous_root_hash: u64,
        challenge_mode: bool,
    ) -> Result<PostClearTransition, String>;

    /// Persist standalone `GAME WIN` entry evidence before returning from a winning board/challenge.
    fn finish_terminal_win_images(&mut self) -> Result<(), String>;

    /// Advance past a verified 3rd-board Normal win (`Congratulations` / `LEVEL-UP!`) into Board `1/3`.
    fn restart_game(&mut self, generation: u64, previous_root_hash: u64) -> Result<(), String>;
}

/// Format an unsolvable partial stop summary clearly without promising Redeal automation.
fn partial_stop_summary(
    objective: Objective,
    banked: bool,
    banked_rank_count: u8,
    score_delta: u32,
    cards_cleared: u8,
    path: &Path,
) -> String {
    let target = format_tripeaks_objective(objective, None);
    if banked {
        match objective {
            Objective::ClearRankQuota { .. } => format!(
                "Deal unsolvable for {target}; banked {banked_rank_count} target-rank cards (+{score_delta} pts, export: {}); release control without Redeal clicks.",
                path.display()
            ),
            Objective::ReachScoreDelta { .. } => format!(
                "Deal unsolvable for {target}; banked +{score_delta} score points (export: {}); release control without Redeal clicks.",
                path.display()
            ),
            Objective::ClearBoard => format!(
                "Deal unsolvable for {target} (max {cards_cleared}/28 clearable, export: {}); zero post-recon clicks sent.",
                path.display()
            ),
        }
    } else {
        format!(
            "Proved untouched T=0 deal unsolvable for {target} (max {cards_cleared}/28 tableau clearable, export: {}); zero post-recon clicks sent — Redeal manually.",
            path.display()
        )
    }
}

/// Execute the complete multi-board TriPeaks session loop (`SingleBoard` or `Continuous`).
pub(crate) fn drive_tripeaks_session<I: TriPeaksSessionIo>(
    style: TriPeaksPlayStyle,
    io: &mut I,
) -> Result<SessionOutcome, String> {
    if !style.uses_session() {
        return Err(
            "Select Continuous or Single board before starting a TriPeaks session".to_owned(),
        );
    }
    io.phase(0, SessionPhase::Entry);
    io.check()?;
    io.prepare_entry()?;

    let mut generation = 0_u64;
    let mut completed_boards = 0_usize;
    let mut completed_games = 0_usize;
    let mut previous_root_hash: Option<u64> = None;
    let mut tracker = ChallengeTracker::default();
    let mut last_export: Option<PathBuf>;
    let mut last_counts: Option<SlotKnowledgeCounts>;
    let mut last_objective_label: Option<String>;

    loop {
        io.check()?;
        generation = generation
            .checked_add(1)
            .ok_or("TriPeaks session board generation counter exhausted")?;
        io.phase(generation, SessionPhase::Reconnaissance);
        io.check()?;

        let mut acquired =
            io.acquire(generation, tracker.last_objective, tracker.challenge_active)?;
        if previous_root_hash == Some(acquired.root_hash) {
            return Err(format!(
                "TriPeaks session observed unchanged board root hash {:#018x} on cycle {generation}; refusing duplicate deal",
                acquired.root_hash
            ));
        }
        previous_root_hash = Some(acquired.root_hash);

        let effective_objective =
            tracker.on_acquired(acquired.objective, acquired.detected_challenge);
        acquired.objective = effective_objective;
        let obj_label = format_tripeaks_objective(effective_objective, acquired.detected_challenge);
        last_export = Some(acquired.export_path.clone());
        last_counts = Some(acquired.counts);
        last_objective_label = Some(obj_label.clone());

        if acquired.completed_during_recon {
            completed_boards = completed_boards
                .checked_add(1)
                .ok_or("Completed TriPeaks board counter exhausted")?;
            if acquired.terminal_challenge_complete {
                completed_games = completed_games
                    .checked_add(1)
                    .ok_or("Completed TriPeaks game counter exhausted")?;
                io.phase(generation, SessionPhase::Completion);
                io.finish_terminal_win_images()?;
                let probes_note = acquired.probes_used;
                return Ok(SessionOutcome {
                    completed_boards,
                    completed_games,
                    unsolvable_max_cleared: None,
                    export_path: last_export,
                    counts: last_counts,
                    objective_label: last_objective_label,
                    summary: format!(
                        "Challenge Complete verified during TriPeaks recon ({completed_boards} board(s), {obj_label}, {probes_note} probe cycle(s)); zero Continue clicks sent."
                    ),
                });
            }
            if acquired.terminal_board_win {
                completed_games = completed_games
                    .checked_add(1)
                    .ok_or("Completed TriPeaks game counter exhausted")?;
                io.phase(generation, SessionPhase::Completion);
                if style == TriPeaksPlayStyle::SingleBoard {
                    io.finish_terminal_win_images()?;
                    return Ok(SessionOutcome {
                        completed_boards,
                        completed_games,
                        unsolvable_max_cleared: None,
                        export_path: last_export,
                        counts: last_counts,
                        objective_label: last_objective_label,
                        summary: format!(
                            "Single TriPeaks board win verified during recon (board {generation}, games: {completed_games}, export: {}).",
                            acquired.export_path.display()
                        ),
                    });
                }
                io.phase(generation, SessionPhase::Restart);
                io.check()?;
                io.restart_game(generation, acquired.root_hash)?;
                continue;
            }
            if style == TriPeaksPlayStyle::SingleBoard {
                io.phase(generation, SessionPhase::Completion);
                return Ok(SessionOutcome {
                    completed_boards,
                    completed_games,
                    unsolvable_max_cleared: None,
                    export_path: last_export,
                    counts: last_counts,
                    objective_label: last_objective_label,
                    summary: format!(
                        "Single TriPeaks board completed during recon ({obj_label}, board {generation}, export: {}).",
                        acquired.export_path.display()
                    ),
                });
            }
            io.phase(generation, SessionPhase::Transition);
            io.check()?;
            match io.wait_next_board_after_clear(
                generation,
                acquired.root_hash,
                tracker.challenge_active,
            )? {
                PostClearTransition::ChallengeComplete => {
                    completed_games = completed_games
                        .checked_add(1)
                        .ok_or("Completed TriPeaks game counter exhausted")?;
                    io.phase(generation, SessionPhase::Completion);
                    io.finish_terminal_win_images()?;
                    return Ok(SessionOutcome {
                        completed_boards,
                        completed_games,
                        unsolvable_max_cleared: None,
                        export_path: last_export,
                        counts: last_counts,
                        objective_label: last_objective_label,
                        summary: format!(
                            "Challenge Complete verified after recon board clear ({completed_boards} board(s)); zero Continue clicks sent."
                        ),
                    });
                }
                PostClearTransition::BoardWinOverlay => {
                    completed_games = completed_games
                        .checked_add(1)
                        .ok_or("Completed TriPeaks game counter exhausted")?;
                    io.phase(generation, SessionPhase::Restart);
                    io.check()?;
                    io.restart_game(generation, acquired.root_hash)?;
                }
                PostClearTransition::NextBoardDealt(_) => {
                    tracker.on_board_progressed(0, true);
                }
            }
            continue;
        }

        io.phase(generation, SessionPhase::Planning);
        io.check()?;
        let decision = io.plan(generation, &acquired)?;

        match decision {
            PlanDecision::UnsolvablePartial {
                route,
                objective,
                banked_rank_count,
                score_delta,
                cards_cleared,
            } => {
                let worth_banking = match objective {
                    Objective::ClearRankQuota { .. } => banked_rank_count > 0,
                    Objective::ReachScoreDelta { .. } => score_delta > 0,
                    Objective::ClearBoard => false,
                };
                if worth_banking && let Some(partial_route) = route {
                    io.phase(generation, SessionPhase::Execution);
                    io.check()?;
                    let executed = io.execute(generation, &acquired, partial_route, true)?;
                    io.phase(generation, SessionPhase::Completion);
                    if executed.completion == tripeaks_route::Completion::ChallengeComplete {
                        completed_boards = completed_boards
                            .checked_add(1)
                            .ok_or("Completed TriPeaks board counter exhausted")?;
                        completed_games = completed_games
                            .checked_add(1)
                            .ok_or("Completed TriPeaks game counter exhausted")?;
                        io.finish_terminal_win_images()?;
                        return Ok(SessionOutcome {
                            completed_boards,
                            completed_games,
                            unsolvable_max_cleared: None,
                            export_path: last_export,
                            counts: last_counts,
                            objective_label: last_objective_label,
                            summary: format!(
                                "Challenge Complete verified while banking partial Challenge progress ({completed_games} completed); zero Continue clicks sent."
                            ),
                        });
                    }
                    let summary = partial_stop_summary(
                        objective,
                        true,
                        banked_rank_count,
                        executed.score_delta,
                        cards_cleared,
                        &acquired.export_path,
                    );
                    return Ok(SessionOutcome {
                        completed_boards,
                        completed_games,
                        unsolvable_max_cleared: Some(cards_cleared),
                        export_path: last_export,
                        counts: last_counts,
                        objective_label: last_objective_label,
                        summary,
                    });
                }

                io.phase(generation, SessionPhase::Completion);
                let summary = partial_stop_summary(
                    objective,
                    false,
                    0,
                    0,
                    cards_cleared,
                    &acquired.export_path,
                );
                return Ok(SessionOutcome {
                    completed_boards,
                    completed_games,
                    unsolvable_max_cleared: Some(cards_cleared),
                    export_path: last_export,
                    counts: last_counts,
                    objective_label: last_objective_label,
                    summary,
                });
            }
            PlanDecision::Solved {
                route,
                escalated_clear_board,
                ..
            } => {
                io.phase(generation, SessionPhase::Execution);
                io.check()?;
                let executed = io.execute(generation, &acquired, route, false)?;
                completed_boards = completed_boards
                    .checked_add(1)
                    .ok_or("Completed TriPeaks board counter exhausted")?;
                tracker.on_board_progressed(executed.score_delta, executed.cards_cleared == 28);

                match executed.completion {
                    tripeaks_route::Completion::ChallengeComplete => {
                        completed_games = completed_games
                            .checked_add(1)
                            .ok_or("Completed TriPeaks game counter exhausted")?;
                        io.phase(generation, SessionPhase::Completion);
                        io.finish_terminal_win_images()?;
                        return Ok(SessionOutcome {
                            completed_boards,
                            completed_games,
                            unsolvable_max_cleared: None,
                            export_path: last_export,
                            counts: last_counts,
                            objective_label: last_objective_label,
                            summary: format!(
                                "TriPeaks Challenge Complete verified ({completed_boards} board(s), final +{0} pts); zero Continue clicks sent.",
                                executed.score_delta
                            ),
                        });
                    }
                    tripeaks_route::Completion::BoardWinOverlay => {
                        completed_games = completed_games
                            .checked_add(1)
                            .ok_or("Completed TriPeaks game counter exhausted")?;
                        io.phase(generation, SessionPhase::Completion);
                        if style == TriPeaksPlayStyle::SingleBoard {
                            io.finish_terminal_win_images()?;
                            return Ok(SessionOutcome {
                                completed_boards,
                                completed_games,
                                unsolvable_max_cleared: None,
                                export_path: last_export,
                                counts: last_counts,
                                objective_label: last_objective_label,
                                summary: format!(
                                    "Single TriPeaks 3/3 game win verified (board {generation}, games: {completed_games}, export: {}).",
                                    acquired.export_path.display()
                                ),
                            });
                        }
                        io.phase(generation, SessionPhase::Restart);
                        io.check()?;
                        io.restart_game(generation, acquired.root_hash)?;
                    }
                    tripeaks_route::Completion::NextBoardDealt {
                        root_hash: next_hash,
                    } => {
                        if next_hash == acquired.root_hash {
                            return Err(
                                "TriPeaks dealt next board repeated identical root hash".into()
                            );
                        }
                        if style == TriPeaksPlayStyle::SingleBoard {
                            io.phase(generation, SessionPhase::Completion);
                            return Ok(SessionOutcome {
                                completed_boards,
                                completed_games,
                                unsolvable_max_cleared: None,
                                export_path: last_export,
                                counts: last_counts,
                                objective_label: last_objective_label,
                                summary: format!(
                                    "Single TriPeaks board {generation} cleared ({obj_label}, export: {}); stopped cleanly before playing next board.",
                                    acquired.export_path.display()
                                ),
                            });
                        }
                        io.phase(generation, SessionPhase::Transition);
                    }
                    tripeaks_route::Completion::BoardCleared => {
                        if style == TriPeaksPlayStyle::SingleBoard {
                            io.phase(generation, SessionPhase::Completion);
                            return Ok(SessionOutcome {
                                completed_boards,
                                completed_games,
                                unsolvable_max_cleared: None,
                                export_path: last_export,
                                counts: last_counts,
                                objective_label: last_objective_label,
                                summary: format!(
                                    "Single TriPeaks board {generation} cleared ({obj_label}, export: {}).",
                                    acquired.export_path.display()
                                ),
                            });
                        }
                        io.phase(generation, SessionPhase::Transition);
                        io.check()?;
                        match io.wait_next_board_after_clear(
                            generation,
                            acquired.root_hash,
                            tracker.challenge_active,
                        )? {
                            PostClearTransition::ChallengeComplete => {
                                completed_games = completed_games
                                    .checked_add(1)
                                    .ok_or("Completed TriPeaks game counter exhausted")?;
                                io.phase(generation, SessionPhase::Completion);
                                io.finish_terminal_win_images()?;
                                return Ok(SessionOutcome {
                                    completed_boards,
                                    completed_games,
                                    unsolvable_max_cleared: None,
                                    export_path: last_export,
                                    counts: last_counts,
                                    objective_label: last_objective_label,
                                    summary: format!(
                                        "TriPeaks Challenge Complete verified ({completed_boards} board(s)); zero Continue clicks sent."
                                    ),
                                });
                            }
                            PostClearTransition::BoardWinOverlay => {
                                completed_games = completed_games
                                    .checked_add(1)
                                    .ok_or("Completed TriPeaks game counter exhausted")?;
                                io.phase(generation, SessionPhase::Restart);
                                io.check()?;
                                io.restart_game(generation, acquired.root_hash)?;
                            }
                            PostClearTransition::NextBoardDealt(_) => {}
                        }
                    }
                    tripeaks_route::Completion::ObjectiveReached => {
                        io.phase(generation, SessionPhase::Completion);
                        if escalated_clear_board {
                            return Err("Escalated ClearBoard route returned ObjectiveReached instead of board clear or next deal".into());
                        }
                        return Ok(SessionOutcome {
                            completed_boards,
                            completed_games,
                            unsolvable_max_cleared: None,
                            export_path: last_export,
                            counts: last_counts,
                            objective_label: last_objective_label,
                            summary: format!(
                                "TriPeaks target ({obj_label}) reached on board {generation} (+{} pts, export: {}); stopped cleanly for operator confirmation.",
                                executed.score_delta,
                                acquired.export_path.display()
                            ),
                        });
                    }
                    tripeaks_route::Completion::PartialComplete
                    | tripeaks_route::Completion::Step
                    | tripeaks_route::Completion::RestoredToRoot => {
                        return Err(format!(
                            "Unexpected TriPeaks completion {:?} during solved session cycle {generation}",
                            executed.completion
                        ));
                    }
                }
            }
        }
    }
}

/// Check that the selected capture backend is TCPv3 (live Beast or deterministic test server).
fn require_tcp_v3_backend() -> Result<(), String> {
    match selected_capture_backend()? {
        CaptureBackend::Tcp {
            mode: CaptureMode::BgrV3 | CaptureMode::PersistentBgrV3,
            ..
        } => Ok(()),
        _ => Err("Independent TriPeaks reconnaissance requires native TCPv3 capture".to_owned()),
    }
}

/// Classify terminal screens when `ReconOutcome::CompletedDuringRecon` occurred.
fn classify_recon_completion_screen(frame: Option<&CapturedFrame>) -> (bool, bool) {
    let Some(frame) = frame else {
        return (false, false);
    };
    match observe_frame(frame) {
        Ok(FrameClassification::ChallengeComplete) => (true, false),
        Ok(FrameClassification::BoardWinOverlay) => (false, true),
        _ => (false, false),
    }
}

/// Production [`TriPeaksSessionIo`] implementation wired to [`QmpClient`], TCPv3, and `tripeaks_*` workers.
struct Native<'a> {
    /// Causal STOP/failure tracker local to this session.
    stop_trace: StopTrace,
    /// First cooperative phase/pause error preserved ahead of later STOP polling.
    phase_error: Option<String>,
    /// Controller identity validated at every capture and input fence.
    context: &'a ControllerContext,
    /// Immutable session request options.
    request: &'a TriPeaksSessionRequest,
    /// Shared worker event channel and authority slots.
    events: &'a WorkerEventSink,
    /// Cooperative cancellation flag.
    cancel: &'a AtomicBool,
    /// Current lifecycle phase.
    phase: SessionPhase,
    /// Current one-based board cycle index.
    generation: u64,
    /// Verified completed boards in this session.
    completed_boards: usize,
    /// Verified completed full games/challenges in this session.
    completed_games: usize,
    /// Plain operator card counts from the current deal.
    counts: Option<SlotKnowledgeCounts>,
    /// Number of Undo All probe cycles run on the current deal.
    probes_used: usize,
    /// Plain-English active objective label for UI display.
    objective_label: Option<String>,
    /// Most recent exported JSON path.
    export_path: Option<PathBuf>,
    /// Determined calculation report for UI stats display.
    latest_calculation: Option<Arc<tripeaks_calculation::Report>>,
    /// Most recent verified guest frame retained for post-game restart or failure packets.
    latest: Option<CapturedFrame>,
    /// `true` once an inner worker already persisted a failure packet.
    failure_saved: bool,
    /// Directory of persisted failure packet, when available.
    failure_directory: Option<PathBuf>,
}

impl Native<'_> {
    /// Emit one structured [`TriPeaksSessionProgress`] event.
    fn send_progress_event(&self, progress: TriPeaksSessionProgress) {
        let _ = self
            .events
            .send(WorkerEvent::TriPeaksSessionProgress(progress));
    }

    /// Emit concise session progress while retaining known counts and objective labels.
    fn emit_progress(
        &self,
        route_cursor: Option<(usize, usize)>,
        expanded_states: Option<u64>,
        planning_progress: Option<TriPeaksPlanningSnapshot>,
        calculation_report: Option<Arc<tripeaks_calculation::Report>>,
        message: String,
    ) {
        self.send_progress_event(TriPeaksSessionProgress {
            context: self.context.clone(),
            run_id: self.request.run_id,
            deal_generation: self.generation,
            completed_boards: self.completed_boards,
            completed_games: self.completed_games,
            phase: self.phase,
            counts: self.counts,
            probes_used: self.probes_used,
            objective_label: self.objective_label.clone(),
            route_cursor,
            expanded_states,
            planning_progress,
            calculation_report,
            export_path: self.export_path.clone(),
            message,
        });
    }

    /// Capture one frame causally so TCP PNG caching for `GAME WIN` recording remains active.
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

    /// Generate a unique temporary export file path for one board cycle.
    fn next_export_destination(&self, generation: u64) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tripeaks-session-{}-{}-g{generation}.json",
            std::process::id(),
            self.request.run_id
        ))
    }

    /// Persist a failure packet if an inner worker hasn't already done so.
    fn persist_session_failure(&mut self, error: &str, latest: Option<&CapturedFrame>) {
        if self.failure_saved {
            return;
        }
        let phase = match self.phase {
            SessionPhase::Entry | SessionPhase::Reconnaissance => PyramidPhase::Reconnaissance,
            SessionPhase::Planning => PyramidPhase::Planning,
            SessionPhase::Execution
            | SessionPhase::Transition
            | SessionPhase::Completion
            | SessionPhase::Restart => PyramidPhase::Execution,
        };
        let packet = persist_failure(FailureEvidence {
            progress: None,
            context: self.context,
            phase,
            run_id: Some(self.request.run_id),
            error,
            details: serde_json::json!({
                "game": "TriPeaks",
                "phase": self.phase.label(),
                "deal_generation": self.generation,
                "completed_boards": self.completed_boards,
                "completed_games": self.completed_games,
                "play_style": self.request.play_style.label(),
                "export_path": self.export_path,
            }),
            source_document: None,
            first_frame: None,
            latest_frame: latest,
        });
        if let Some(dir) = &packet.directory {
            send_log(
                self.events,
                format!(
                    "Saved TriPeaks failure evidence packet to {}.",
                    dir.display()
                ),
            );
            self.failure_directory = Some(dir.clone());
        }
        if let Some(persist_err) = &packet.persistence_error {
            send_log(
                self.events,
                format!("Failed to persist TriPeaks failure evidence packet: {persist_err}"),
            );
        }
        self.failure_saved = true;
    }
}

impl TriPeaksSessionIo for Native<'_> {
    type Route = tripeaks_route::Prepared;

    fn check(&self) -> Result<(), String> {
        if let Some(err) = &self.phase_error {
            return Err(err.clone());
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
        if phase == SessionPhase::Reconnaissance {
            self.latest_calculation = None;
            self.counts = None;
            self.probes_used = 0;
        }
        let pause_phase = match phase {
            SessionPhase::Planning => Some(pyramid_pause::Phase::Search),
            SessionPhase::Reconnaissance | SessionPhase::Entry => Some(pyramid_pause::Phase::Recon),
            SessionPhase::Execution | SessionPhase::Restart => Some(pyramid_pause::Phase::Gameplay),
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
        let msg = format!("TriPeaks session cycle {generation}: {}", phase.label());
        send_status(self.events, msg.clone());
        self.emit_progress(None, None, None, None, msg);
    }

    fn prepare_entry(&mut self) -> Result<(), String> {
        self.check()?;
        require_tcp_v3_backend()?;
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|e| format!("TriPeaks entry QMP connection failed: {e}"))?;
        let attempts = self.request.step_timing.max_restore_observations.max(1);
        for attempt in 0..attempts {
            let frame = self.capture_causal(&mut qmp)?;
            match observe_frame(&frame).map_err(|e| e.to_string())? {
                FrameClassification::ChallengeComplete => {
                    return Err("Challenge Complete overlay is already showing; click Continue manually before starting a new TriPeaks session.".to_owned());
                }
                FrameClassification::Settled(_) => return Ok(()),
                FrameClassification::BoardWinOverlay | FrameClassification::Unsupported(_) => {
                    let mut scan = TableauScanState::for_mode(GameMode::TriPeaks);
                    let (observation, _) = analyse_captured_frame(frame, &scan)?;
                    let mut dummy_boards = self.completed_boards;
                    let outcome = post_game::run_independent_tripeaks_restart_retaining_frame(
                        post_game::TriPeaksRestartContext {
                            qmp: &mut qmp,
                            socket_path: &self.context.socket_path,
                            initial_observation: Some(&observation),
                            previous_root_hash: None,
                            scan_state: &mut scan,
                            completed_boards: &mut dummy_boards,
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
                            return Ok(());
                        }
                        post_game::PostGameOutcome::Completed => {
                            return Err(
                                "TriPeaks cold-entry restart ended without proving a fresh T=0 deal"
                                    .to_owned(),
                            );
                        }
                    }
                }
                FrameClassification::NoMovesModal(_) => {
                    return Err("TriPeaks entry is on a No Moves Remaining modal; restore or deal a fresh board before starting.".to_owned());
                }
                FrameClassification::Settling => {
                    if attempt + 1 < attempts
                        && super::cancellable_wait(
                            self.request.step_timing.reobserve_delay,
                            self.cancel,
                        )
                        .is_err()
                    {
                        self.check()?;
                        return Err("STOP requested during TriPeaks entry wait".to_owned());
                    }
                }
            }
        }
        Err("TriPeaks entry screen remained unsettled across observation budget".to_owned())
    }

    fn acquire(
        &mut self,
        generation: u64,
        fallback_objective: Option<Objective>,
        force_challenge: bool,
    ) -> Result<AcquiredBoard, String> {
        self.check()?;
        require_tcp_v3_backend()?;
        let destination = self.next_export_destination(generation);
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|e| format!("TriPeaks recon QMP connection failed: {e}"))?;
        let context = self.context;
        let events = self.events;
        let cancel = self.cancel;
        let stop_trace = &self.stop_trace;
        let run_id = self.request.run_id;
        let completed_boards = self.completed_boards;
        let completed_games = self.completed_games;

        send_state(self.events, WorkerState::Capturing);
        let inner_ops = NativeOperations {
            qmp: &mut qmp,
            socket: &context.socket_path,
            events,
            cancel,
            active: || {
                stop_trace
                    .check(&events.latest_frame.reconnaissance, context, cancel)
                    .map_err(|err| {
                        if cancel.load(Ordering::Acquire) {
                            OperationError::Cancelled
                        } else {
                            OperationError::Adapter(err)
                        }
                    })
            },
            guard: authorise_tripeaks_input,
            last_timing: None,
        };
        let recon_io = TrackedOperations::new(inner_ops);
        let session = crate::game_operations::Session::new(recon_io);
        let config = TriPeaksReconConfig {
            force_challenge_mode: force_challenge,
            move_settle_delay: self.request.step_timing.move_settle_delay,
            reobserve_delay: self.request.step_timing.reobserve_delay,
            restore_settle_delay: self.request.step_timing.restore_settle_delay,
            max_step_observations: self.request.step_timing.max_step_observations,
            max_restore_observations: self.request.step_timing.max_restore_observations,
            ..TriPeaksReconConfig::default()
        };
        let mut on_frame = |frame: &CapturedFrame| {
            events.publish_diagnostic_frame(frame.clone());
        };
        let mut domain = LiveTriPeaksDomain::new(session, cancel, config, Box::new(&mut on_frame));
        let last_emitted = Cell::new(Instant::now() - Duration::from_secs(10));
        let export_path_opt = Some(destination.clone());
        domain.set_progress_observer(Box::new(move |progress| {
            let now = Instant::now();
            if now.saturating_duration_since(last_emitted.get()) >= Duration::from_millis(120)
                || progress.probes_used > 0
            {
                last_emitted.set(now);
                let objective_label = progress
                    .objective
                    .map(|obj| format_tripeaks_objective(obj, progress.challenge));
                let msg = format!(
                    "Scanning board {generation}: {}",
                    progress.counts.summary_line(progress.probes_used)
                );
                let _ = events.send(WorkerEvent::TriPeaksSessionProgress(
                    TriPeaksSessionProgress {
                        context: context.clone(),
                        run_id,
                        deal_generation: generation,
                        completed_boards,
                        completed_games,
                        phase: SessionPhase::Reconnaissance,
                        counts: Some(progress.counts),
                        probes_used: progress.probes_used,
                        objective_label,
                        route_cursor: None,
                        expanded_states: None,
                        planning_progress: None,
                        calculation_report: None,
                        export_path: export_path_opt.clone(),
                        message: msg,
                    },
                ));
            }
        }));

        let run_result: Result<_, String> = (|| {
            let _ = domain
                .prepare_initial_observation()
                .map_err(|e| e.to_string())?;
            if domain.detected_challenge().is_none()
                && force_challenge
                && let Some(fallback) = fallback_objective
            {
                domain.apply_fallback_objective_if_normal(fallback);
            }
            run_tripeaks_recon(&mut domain, Some(&destination)).map_err(|e| e.to_string())
        })();

        let final_frame_opt = domain.current_frame().cloned();
        let finished_io = domain.into_session().into_io();
        let balanced = finished_io.attempted == finished_io.acknowledged;
        if let Some(frame) = final_frame_opt.or(finished_io.latest) {
            self.latest = Some(frame);
        }
        let report = match run_result {
            Ok(report) => report,
            Err(err) => {
                if !balanced {
                    self.stop_trace.adopt(RunTermination::Failed);
                }
                return Err(err);
            }
        };

        let counts = SlotKnowledgeCounts::from_slots(&report.document.slots);
        self.counts = Some(counts);
        self.probes_used = report.document.probes_used;
        self.objective_label = Some(format_tripeaks_objective(
            report.objective,
            report.detected_challenge,
        ));
        self.export_path = Some(destination.clone());

        let file = File::open(&destination).map_err(|e| {
            format!(
                "TriPeaks exported deal read failed ({}): {e}",
                destination.display()
            )
        })?;
        let mut bytes = Vec::new();
        file.take((MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("TriPeaks exported deal read: {e}"))?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "TriPeaks exported deal exceeds {} bytes",
                MAX_DOCUMENT_BYTES
            ));
        }

        let completed_during_recon =
            matches!(report.outcome, ReconOutcome::CompletedDuringRecon { .. });
        let (terminal_challenge_complete, terminal_board_win) = if completed_during_recon {
            classify_recon_completion_screen(self.latest.as_ref())
        } else {
            (false, false)
        };

        send_log(
            self.events,
            format!(
                "TriPeaks Recon complete (cycle {generation}, {}): {}; exported to {}.",
                self.objective_label.as_deref().unwrap_or("Clear board"),
                counts.summary_line(report.document.probes_used),
                destination.display()
            ),
        );
        self.emit_progress(
            None,
            None,
            None,
            None,
            format!("Acquired TriPeaks deal (cycle {generation})."),
        );

        Ok(AcquiredBoard {
            export_path: destination,
            document: report.document,
            bytes,
            objective: report.objective,
            detected_challenge: report.detected_challenge,
            root_hash: report.root_hash,
            completed_during_recon,
            terminal_challenge_complete,
            terminal_board_win,
            counts,
            probes_used: self.probes_used,
        })
    }

    fn plan(
        &mut self,
        generation: u64,
        acquired: &AcquiredBoard,
    ) -> Result<PlanDecision<Self::Route>, String> {
        self.check()?;
        let identity = tripeaks_calculation::Identity {
            context: self.context.clone(),
            request_id: self.request.run_id,
            source_generation: generation,
        };
        let last_emitted_at = Cell::new(None::<Instant>);
        let events = self.events;
        let stop_trace = &self.stop_trace;
        let context = self.context;
        let cancel = self.cancel;
        let run_id = self.request.run_id;
        let completed_boards = self.completed_boards;
        let completed_games = self.completed_games;
        let counts = self.counts;
        let probes_used = self.probes_used;
        let objective_label = self.objective_label.clone();
        let export_path = self.export_path.clone();

        let report = Arc::new(tripeaks_calculation::search_deal_with_control(
            tripeaks_calculation::SearchInput {
                document: acquired.document.clone(),
                objective_override: Some(acquired.objective),
                path: acquired.export_path.clone(),
                bytes: acquired.bytes.clone(),
                cancel: self.cancel,
                events: self.events,
            },
            || match stop_trace.check(&events.latest_frame.reconnaissance, context, cancel) {
                Ok(()) => Ok(false),
                Err(_) if cancel.load(Ordering::Acquire) => Ok(true),
                Err(error) => Err(error),
            },
            |search, paused| {
                let expanded = u64::try_from(search.stats.expanded).unwrap_or(u64::MAX);
                let now = Instant::now();
                let should_emit = expanded == 0
                    || search.finished
                    || last_emitted_at.get().is_none_or(|prev| {
                        now.saturating_duration_since(prev) >= PLANNING_PROGRESS_INTERVAL
                    });
                if should_emit {
                    last_emitted_at.set(Some(now));
                    let _ = events.send(WorkerEvent::TriPeaksSessionProgress(
                        TriPeaksSessionProgress {
                            context: context.clone(),
                            run_id,
                            deal_generation: generation,
                            completed_boards,
                            completed_games,
                            phase: SessionPhase::Planning,
                            counts,
                            probes_used,
                            objective_label: objective_label.clone(),
                            route_cursor: None,
                            expanded_states: Some(expanded),
                            planning_progress: Some(TriPeaksPlanningSnapshot { search, paused }),
                            calculation_report: None,
                            export_path: export_path.clone(),
                            message: format!(
                                "Searching TriPeaks cycle {generation} ({expanded} expanded states)..."
                            ),
                        },
                    ));
                }
            },
        )?);

        self.latest_calculation = Some(Arc::clone(&report));
        let summary = tripeaks_calculation::outcome_summary_message(&report.outcome);
        let stats = tripeaks_calculation::stats(&report.outcome);
        let expanded = u64::try_from(stats.expanded).unwrap_or(u64::MAX);
        self.emit_progress(
            None,
            Some(expanded),
            None,
            Some(Arc::clone(&report)),
            summary.clone(),
        );
        send_log(self.events, summary);

        match &report.outcome {
            tripeaks_calculation::Outcome::Solved {
                escalated_clear_board,
                ..
            } => {
                let escalated = *escalated_clear_board;
                let prepared = tripeaks_route::Prepared::new(identity, report, self.cancel)
                    .map_err(|err| err.to_string())?;
                Ok(PlanDecision::Solved {
                    route: prepared,
                    escalated_clear_board: escalated,
                })
            }
            tripeaks_calculation::Outcome::Partial {
                objective,
                moves,
                replay,
                max_tableau_cleared,
                banked_rank_count,
                ..
            } => {
                let score_delta = replay.final_position.score_delta;
                let banked_rank_count = *banked_rank_count;
                let max_tableau_cleared = *max_tableau_cleared;
                let objective = *objective;
                let partial_route = if !moves.is_empty() {
                    Some(
                        tripeaks_route::Prepared::new(identity, report, self.cancel)
                            .map_err(|err| err.to_string())?,
                    )
                } else {
                    None
                };
                Ok(PlanDecision::UnsolvablePartial {
                    route: partial_route,
                    objective,
                    banked_rank_count,
                    score_delta,
                    cards_cleared: max_tableau_cleared,
                })
            }
            tripeaks_calculation::Outcome::Cancelled(_) => {
                self.check()?;
                Err("TriPeaks search cancelled".to_owned())
            }
            tripeaks_calculation::Outcome::Failed { failure, .. } => {
                Err(format!("TriPeaks search failed: {failure}"))
            }
        }
    }

    fn execute(
        &mut self,
        generation: u64,
        _acquired: &AcquiredBoard,
        route: Self::Route,
        _allow_final_modal: bool,
    ) -> Result<ExecutedBoard, String> {
        self.check()?;
        let plan_id = tripeaks_calculation::Identity {
            context: self.context.clone(),
            request_id: self.request.run_id,
            source_generation: generation,
        };
        let route_run_id = {
            let mut store = self
                .events
                .latest_frame
                .tripeaks_route
                .lock()
                .map_err(|_| "TriPeaks route store poisoned".to_owned())?;
            store.install(Some(route))?;
            store.reserve(&plan_id, tripeaks_route::PlaybackAction::MultiStep)?
        };
        self.emit_progress(
            Some((0, 0)),
            None,
            None,
            None,
            format!("Executing TriPeaks route (cycle {generation})..."),
        );

        let execution = tripeaks_route::execute_for_session(
            plan_id,
            route_run_id,
            tripeaks_route::PlaybackAction::MultiStep,
            self.request.step_timing,
            false,
            self.events,
            self.cancel,
        );
        if let Some(frame) = execution.latest_frame {
            self.latest = Some(frame);
        }
        self.stop_trace.adopt(execution.termination);
        match execution.result {
            Ok(progress) => {
                let completion = progress
                    .completion
                    .ok_or("TriPeaks session route returned without a completion kind")?;
                self.emit_progress(
                    Some((progress.cursor, progress.total)),
                    None,
                    None,
                    None,
                    format!(
                        "Executed {}/{} TriPeaks moves ({}/28 tableau cleared, {:+} pts).",
                        progress.cursor,
                        progress.total,
                        progress.cards_cleared,
                        progress.score_delta
                    ),
                );
                Ok(ExecutedBoard {
                    completion,
                    cards_cleared: progress.cards_cleared,
                    score_delta: progress.score_delta,
                })
            }
            Err(failure) => {
                self.failure_saved = true;
                self.failure_directory = failure.directory;
                Err(failure.error)
            }
        }
    }

    fn wait_next_board_after_clear(
        &mut self,
        generation: u64,
        previous_root_hash: u64,
        challenge_mode: bool,
    ) -> Result<PostClearTransition, String> {
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|e| format!("TriPeaks post-clear QMP connection failed: {e}"))?;
        let mut challenge_confirm = 0_usize;
        let mut overlay_confirm = 0_usize;
        let mut next_confirm: Option<(u64, usize)> = None;

        for round in 0..MAX_TRANSITION_CAPTURES {
            self.check()?;
            let wait_dur = if round == 0 {
                self.request
                    .board_redeal_settle
                    .max(BOARD_TRANSITION_REOBSERVE_DELAY)
            } else {
                BOARD_TRANSITION_REOBSERVE_DELAY
            };
            if super::cancellable_wait(wait_dur, self.cancel).is_err() {
                self.check()?;
                return Err("STOP requested during TriPeaks next-board wait".to_owned());
            }
            let frame = self.capture_causal(&mut qmp)?;
            match observe_frame(&frame).map_err(|e| e.to_string())? {
                FrameClassification::ChallengeComplete => {
                    overlay_confirm = 0;
                    next_confirm = None;
                    challenge_confirm += 1;
                    if challenge_confirm >= 2 {
                        return Ok(PostClearTransition::ChallengeComplete);
                    }
                }
                FrameClassification::BoardWinOverlay => {
                    challenge_confirm = 0;
                    next_confirm = None;
                    overlay_confirm += 1;
                    if overlay_confirm >= 2 {
                        let _ = challenge_mode;
                        return Ok(PostClearTransition::BoardWinOverlay);
                    }
                }
                FrameClassification::Settled(snap) => {
                    challenge_confirm = 0;
                    overlay_confirm = 0;
                    if snap.is_untouched_root_layout() {
                        let hash = semantic_board_hash(&snap);
                        if hash != previous_root_hash {
                            let count = match next_confirm {
                                Some((prev, c)) if prev == hash => c + 1,
                                _ => 1,
                            };
                            next_confirm = Some((hash, count));
                            if count >= 2 {
                                return Ok(PostClearTransition::NextBoardDealt(hash));
                            }
                        } else {
                            next_confirm = None;
                        }
                    } else {
                        next_confirm = None;
                    }
                }
                FrameClassification::NoMovesModal(_) => {
                    return Err(format!(
                        "Unexpected No Moves Remaining modal after clearing TriPeaks board {generation}"
                    ));
                }
                FrameClassification::Settling | FrameClassification::Unsupported(_) => {
                    challenge_confirm = 0;
                    overlay_confirm = 0;
                    next_confirm = None;
                }
            }
        }
        Err(format!(
            "TriPeaks cycle {generation} did not settle into a next board, Board Win overlay, or Challenge Complete within {MAX_TRANSITION_CAPTURES} captures"
        ))
    }

    fn finish_terminal_win_images(&mut self) -> Result<(), String> {
        let _game_win_images = super::begin_game_win_images(self.events)?;
        Ok(())
    }

    fn restart_game(&mut self, _generation: u64, previous_root_hash: u64) -> Result<(), String> {
        self.check()?;
        let mut qmp = QmpClient::connect(&self.context.socket_path)
            .map_err(|e| format!("TriPeaks post-win QMP connection failed: {e}"))?;
        let frame = match self.latest.take() {
            Some(frame) => frame,
            None => self.capture_causal(&mut qmp)?,
        };
        let mut scan = TableauScanState::for_mode(GameMode::TriPeaks);
        let (observation, _) = analyse_captured_frame(frame, &scan)?;
        let mut dummy_boards = self.completed_boards;
        let outcome = post_game::run_independent_tripeaks_restart_retaining_frame(
            post_game::TriPeaksRestartContext {
                qmp: &mut qmp,
                socket_path: &self.context.socket_path,
                initial_observation: Some(&observation),
                previous_root_hash: Some(previous_root_hash),
                scan_state: &mut scan,
                completed_boards: &mut dummy_boards,
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
            post_game::PostGameOutcome::Completed => {
                Err("TriPeaks post-win restart ended without proving a fresh T=0 deal".to_owned())
            }
        }
    }
}

/// Execute and cleanly retire one worker-owned TriPeaks automatic session.
pub(super) fn run(
    context: &ControllerContext,
    request: &TriPeaksSessionRequest,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let mut io = Native {
        stop_trace: StopTrace::default(),
        phase_error: None,
        context,
        request,
        events,
        cancel,
        phase: SessionPhase::Entry,
        generation: 0,
        completed_boards: 0,
        completed_games: 0,
        counts: None,
        probes_used: 0,
        objective_label: None,
        export_path: None,
        latest_calculation: None,
        latest: None,
        failure_saved: false,
        failure_directory: None,
    };
    send_log(
        events,
        format!(
            "TriPeaks automatic session started: mode={}; move settle={} ms.",
            request.play_style.label(),
            request.step_timing.move_settle_delay.as_millis()
        ),
    );

    let mut outcome = drive_tripeaks_session(request.play_style, &mut io);
    let mut termination = io.stop_trace.termination(outcome.is_err(), true);
    if let Err(ref err) = outcome
        && termination != RunTermination::OperatorStopped
    {
        let err_owned = err.clone();
        let latest = io.latest.clone();
        io.persist_session_failure(&err_owned, latest.as_ref());
    }
    match events.latest_frame.tripeaks_route.lock() {
        Ok(mut store) => store.revoke(),
        Err(_) => {
            outcome = Err("TriPeaks session retirement lock poisoned; restart required".to_owned());
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
    let failure_directory = if outcome.is_err() {
        io.failure_directory
    } else {
        None
    };

    let (
        completed_boards,
        completed_games,
        counts,
        objective_label,
        unsolvable_max_cleared,
        export_path,
        result,
    ) = match outcome {
        Ok(done) => (
            done.completed_boards,
            done.completed_games,
            done.counts,
            done.objective_label,
            done.unsolvable_max_cleared,
            done.export_path,
            Ok(done.summary),
        ),
        Err(err) => (
            io.completed_boards,
            io.completed_games,
            io.counts,
            io.objective_label,
            None,
            io.export_path,
            Err(err),
        ),
    };
    let message = result.as_ref().map_or_else(
        |err| format!("TriPeaks automatic session stopped: {err}"),
        Clone::clone,
    );
    send_status(events, message.clone());
    send_log(events, message);
    let _ = events.send(WorkerEvent::TriPeaksSessionFinished(
        TriPeaksSessionCompletion {
            context: context.clone(),
            run_id: request.run_id,
            completed_boards,
            completed_games,
            termination,
            counts,
            objective_label,
            unsolvable_max_cleared,
            calculation_report: io.latest_calculation,
            export_path,
            failure_directory,
            result,
        },
    ));
}

/// Standalone Manual TriPeaks `RECON / Export 52-slot deal` runner emitting live progress events.
pub(super) fn run_recon(
    context: &ControllerContext,
    request_id: u64,
    destination: PathBuf,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let stop_trace = StopTrace::default();
    let mut latest: Option<CapturedFrame> = None;
    let mut balanced = true;

    let result: Result<TriPeaksManualReconSummary, String> = (|| {
        context.validate()?;
        if context.game_mode != GameMode::TriPeaks
            || context.strategy != SolvingStrategy::ShortestPath
        {
            return Err("TriPeaks RECON requires ShortestPath + TriPeaks".to_owned());
        }
        if destination.as_os_str().is_empty() {
            return Err("TriPeaks RECON destination path cannot be empty".to_owned());
        }
        require_tcp_v3_backend()?;
        stop_trace.check(&events.latest_frame.reconnaissance, context, cancel)?;
        send_state(events, WorkerState::Connecting);
        send_status(events, "Running TriPeaks RECON...".to_owned());

        let mut qmp = QmpClient::connect(&context.socket_path)
            .map_err(|e| format!("QMP connect failed: {e}"))?;
        let inner_ops = NativeOperations {
            qmp: &mut qmp,
            socket: &context.socket_path,
            events,
            cancel,
            active: || {
                stop_trace
                    .check(&events.latest_frame.reconnaissance, context, cancel)
                    .map_err(|err| {
                        if cancel.load(Ordering::Acquire) {
                            OperationError::Cancelled
                        } else {
                            OperationError::Adapter(err)
                        }
                    })
            },
            guard: authorise_tripeaks_input,
            last_timing: None,
        };
        let recon_io = TrackedOperations::new(inner_ops);
        let session = crate::game_operations::Session::new(recon_io);
        let config = TriPeaksReconConfig::default();
        let mut on_frame = |frame: &CapturedFrame| {
            events.publish_diagnostic_frame(frame.clone());
        };
        let mut domain = LiveTriPeaksDomain::new(session, cancel, config, Box::new(&mut on_frame));
        domain.set_progress_observer(Box::new(move |progress| {
            let _ = events.send_for_context(
                Some(context.clone()),
                WorkerEvent::TriPeaksReconProgress {
                    request_id,
                    counts: progress.counts,
                    probes_used: progress.probes_used,
                    objective: progress.objective,
                    challenge: progress.challenge,
                },
            );
        }));
        send_state(events, WorkerState::Capturing);
        let run_result = run_tripeaks_recon(&mut domain, Some(&destination));
        latest = domain.current_frame().cloned();
        let finished_io = domain.into_session().into_io();
        balanced = finished_io.attempted == finished_io.acknowledged;
        if latest.is_none() {
            latest = finished_io.latest;
        }
        let report = run_result.map_err(|e| e.to_string())?;
        let counts = SlotKnowledgeCounts::from_slots(&report.document.slots);
        let completed_during_recon =
            matches!(report.outcome, ReconOutcome::CompletedDuringRecon { .. });
        if completed_during_recon {
            let (cc, bw) = classify_recon_completion_screen(latest.as_ref());
            if cc || bw {
                let _game_win_images = super::begin_game_win_images(events)?;
            }
        }
        let summary = format!(
            "TriPeaks RECON finished ({}): {}; {} -> {}",
            format_tripeaks_objective(report.objective, report.detected_challenge),
            counts.summary_line(report.document.probes_used),
            if completed_during_recon {
                "completed live during probes"
            } else {
                "board restored to T=0"
            },
            destination.display()
        );
        Ok(TriPeaksManualReconSummary {
            path: destination.clone(),
            objective: report.objective,
            challenge: report.detected_challenge,
            counts,
            probes_used: report.document.probes_used,
            root_hash: report.root_hash,
            completed_during_recon,
            summary,
        })
    })();

    let termination = stop_trace.termination(result.is_err(), balanced);
    if let Err(ref error) = result
        && termination != RunTermination::OperatorStopped
    {
        let packet = persist_failure(FailureEvidence {
            progress: None,
            context,
            phase: PyramidPhase::Reconnaissance,
            run_id: Some(request_id),
            error,
            details: serde_json::json!({
                "game": "TriPeaks",
                "phase": "Manual RECON",
                "request_id": request_id,
                "destination": destination,
            }),
            source_document: None,
            first_frame: None,
            latest_frame: latest.as_ref(),
        });
        if let Some(dir) = packet.directory {
            send_log(
                events,
                format!("Saved TriPeaks RECON failure packet to {}.", dir.display()),
            );
        }
    }

    events.latest_frame.reconnaissance.finish();
    send_state(
        events,
        if result.is_ok() || termination == RunTermination::OperatorStopped {
            WorkerState::Ready
        } else {
            WorkerState::Uncertain
        },
    );
    match &result {
        Ok(summary) => {
            send_status(events, summary.summary.clone());
            send_log(events, summary.summary.clone());
        }
        Err(error) => {
            let msg = if termination == RunTermination::OperatorStopped {
                format!("TriPeaks RECON stopped: {error}")
            } else {
                format!("TriPeaks RECON failed: {error}")
            };
            send_status(events, msg.clone());
            send_log(events, msg);
        }
    }
    let _ = events.send_for_context(
        Some(context.clone()),
        WorkerEvent::TriPeaksReconFinished {
            request_id,
            termination,
            result,
        },
    );
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, VecDeque},
        fs,
        io::{BufRead, BufReader, ErrorKind, Read, Write},
        net::{TcpListener, TcpStream},
        os::unix::net::{UnixListener, UnixStream},
        path::PathBuf,
        sync::{Mutex, OnceLock, atomic::AtomicU64, mpsc},
        thread,
    };

    use serde_json::{Value, json};
    use xsar::{games::cards::Rank, tcp::protocol::v3};

    use super::super::{LatestFrameSlot, TestCaptureBackend, WorkerImageDiagnostics};
    use super::*;
    use crate::{
        capture::decode_png,
        freecell::PLAY_X,
        game_controls::{FREECELL_ROW_Y, LocationRequest, read_card},
        game_operations::CardRead,
        geometry::{PixelPoint, PixelRect},
        image_diagnostics::ImageDiagnostics,
        parameters::{
            NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, SHARED_UNDO_ALL_CONTROL, STOCK_CARD_BOUNDS,
            TABLEAU_CARD_REGIONS, TARGET_STOCK, WASTE_CARD_BOUNDS,
        },
        recon::{Card, CardSlot, SlotId},
        tripeaks_model::{
            COVERING_CHILD_MASKS, DEAL_SLOT_COUNT, DealProvenance, INITIAL_WASTE_SLOT,
            STOCK_CARD_COUNT, STOCK_START_SLOT, TriPeaksMove, TriPeaksPosition, apply_move,
            slot_to_tableau_coord, validate_slot_table,
        },
        worker::tripeaks_reconnaissance::{StockObs, TableauSlotObs, TriPeaksStepTiming},
    };

    /// Monotonic counter isolating socket, export JSON, and diagnostic directory paths.
    static NEXT_SESSION_TEST_ID: AtomicU64 = AtomicU64::new(1);

    /// Shared decoded native fixture bank used across TriPeaks session and readiness tests.
    struct SessionAssets {
        /// Free Cell fixture supplying `34 × 52` corner crops for all `52` cards.
        fc01: CapturedFrame,
        /// Untouched `T = 0` TriPeaks scene (`Undo All` disabled).
        left_ace: CapturedFrame,
        /// Mid-play TriPeaks scene (`Undo All` enabled).
        draw_first: CapturedFrame,
        /// Calibrated Draw-23 empty Stock tray fixture.
        stock_empty: CapturedFrame,
        /// Calibrated Normal `Boards: 1 of 3` header source fixture.
        normal_one: CapturedFrame,
        /// Calibrated Normal `Boards: 2 of 3` header source fixture.
        normal_two: CapturedFrame,
        /// Calibrated Challenge `Fours cleared 4/12` header fixture.
        fours_header: CapturedFrame,
        /// Calibrated `Challenge Complete!` overlay with `"Continue"`.
        challenge_complete: CapturedFrame,
        /// Discovered `(x, y)` corner origin inside `fc01` indexed by canonical card ID `0..52`.
        corner_origins: [(u32, u32); DEAL_SLOT_COUNT],
    }

    /// Global lazy decoded asset bank shared across session tests.
    static SESSION_ASSETS: OnceLock<SessionAssets> = OnceLock::new();

    /// Borrow decoded shared native fixtures and discovered `52`-card corner lookup table.
    fn assets() -> &'static SessionAssets {
        SESSION_ASSETS.get_or_init(|| {
            let decode = |bytes: &[u8], label: &str| {
                decode_png(bytes).unwrap_or_else(|err| panic!("failed decoding {label}: {err}"))
            };
            let fc01 = decode(
                include_bytes!("../../tests/fixtures/freecell-FC01.png"),
                "freecell-FC01",
            );
            let left_ace = decode(
                include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"),
                "left-ace",
            );
            let draw_first = decode(
                include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"),
                "draw-first",
            );
            let stock_empty = decode(
                include_bytes!(
                    "../../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"
                ),
                "stock-empty",
            );
            let normal_one = decode(
                include_bytes!("../../tests/fixtures/pyramid-standard-series/normal-1.png"),
                "normal-1",
            );
            let normal_two = decode(
                include_bytes!("../../tests/fixtures/pyramid-standard-series/normal-2.png"),
                "normal-2",
            );
            let fours_header = decode(
                include_bytes!("../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png"),
                "fours-four-of-twelve",
            );
            let challenge_complete = decode(
                include_bytes!(
                    "../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
                ),
                "boards-complete-continue",
            );
            let mut corner_origins = [None; DEAL_SLOT_COUNT];
            for (col_idx, &x) in PLAY_X.iter().enumerate() {
                let max_rows = if col_idx < 4 { 7 } else { 6 };
                for (row_idx, &y) in FREECELL_ROW_Y.iter().take(max_rows).enumerate() {
                    let col_u8 = u8::try_from(col_idx + 1).expect("col in 1..=8");
                    let row_u8 = u8::try_from(row_idx + 1).expect("row in 1..=7");
                    let CardRead::Card(xc) =
                        read_card(&fc01, LocationRequest::freecell_play(col_u8, row_u8))
                            .expect("valid fc01 read")
                    else {
                        panic!("FC01 ({col_u8}, {row_u8}) did not decode as Card");
                    };
                    let rc = Card::try_from(xc).expect("valid canonical card");
                    corner_origins[usize::from(rc.0)] = Some((x, y));
                }
            }
            SessionAssets {
                fc01,
                left_ace,
                draw_first,
                stock_empty,
                normal_one,
                normal_two,
                fours_header,
                challenge_complete,
                corner_origins: corner_origins
                    .map(|opt| opt.expect("FC01 contains all 52 canonical cards")),
            }
        })
    }

    /// Scripted board fixture emitted by the deterministic in-memory [`MockSessionIo`].
    struct ScriptedBoard {
        /// Semantic root hash returned from `acquire`.
        root_hash: u64,
        /// Active board objective reported by `acquire`.
        objective: Objective,
        /// Optional live Challenge HUD parsed at `T = 0`.
        detected_challenge: Option<TriPeaksChallenge>,
        /// `true` when recon probes finished the board/challenge directly.
        completed_during_recon: bool,
        /// `true` when `completed_during_recon` hit `Challenge Complete!`.
        terminal_challenge_complete: bool,
        /// `true` when `completed_during_recon` hit Normal 3/3 win.
        terminal_board_win: bool,
        /// Scripted planning decision returned by `plan`.
        plan_decision: PlanDecision<String>,
        /// Outcome returned when `execute` consumes `plan_decision`'s route.
        executed: ExecutedBoard,
        /// Post-clear transition returned when `executed.completion == BoardCleared`.
        post_clear: Option<PostClearTransition>,
    }

    impl ScriptedBoard {
        /// Construct a valid placeholder `52`-slot document satisfying `AcquiredBoard`.
        fn sample_document(objective: Objective) -> TriPeaksDealDocument {
            let slots = std::array::from_fn(|i| CardSlot::Known(Card(i as u8)));
            TriPeaksDealDocument {
                provenance: DealProvenance::RestoredAtRoot,
                probes_used: 1,
                objective,
                slots,
                planned_route: None,
            }
        }
    }

    /// Recorded call metadata captured by [`MockSessionIo`].
    #[derive(Default)]
    struct MockSessionLog {
        /// Total `prepare_entry` calls.
        prepare_calls: usize,
        /// `(generation, fallback_objective, force_challenge)` per `acquire`.
        acquire_calls: Vec<(u64, Option<Objective>, bool)>,
        /// `(generation, acquired_objective)` per `plan`.
        plan_calls: Vec<(u64, Objective)>,
        /// `(generation, route_tag, allow_final_modal)` per `execute`.
        execute_calls: Vec<(u64, String, bool)>,
        /// Total `finish_terminal_win_images` calls.
        finish_win_calls: usize,
        /// `(generation, previous_root_hash)` per `restart_game`.
        restart_calls: Vec<(u64, u64)>,
        /// Lifecycle phase sequence observed by `phase`.
        phases: Vec<(u64, SessionPhase)>,
    }

    /// Deterministic in-memory [`TriPeaksSessionIo`] driving `drive_tripeaks_session` state tests.
    struct MockSessionIo {
        /// FIFO queue of scripted boards dispensed on each `acquire` call.
        boards: VecDeque<ScriptedBoard>,
        /// Last board acquired on the current cycle so `plan` and `execute` inspect its script.
        current: Option<ScriptedBoard>,
        /// Optional cycle limit triggering clean `STOP` error in tests without natural termination.
        stop_on_generation: Option<u64>,
        /// Recorded call audit trail.
        log: MockSessionLog,
    }

    impl MockSessionIo {
        /// Wrap a sequence of scripted boards.
        fn new(boards: Vec<ScriptedBoard>) -> Self {
            Self {
                boards: boards.into(),
                current: None,
                stop_on_generation: None,
                log: MockSessionLog::default(),
            }
        }
    }

    impl TriPeaksSessionIo for MockSessionIo {
        type Route = String;

        fn check(&self) -> Result<(), String> {
            Ok(())
        }

        fn phase(&mut self, generation: u64, phase: SessionPhase) {
            self.log.phases.push((generation, phase));
        }

        fn prepare_entry(&mut self) -> Result<(), String> {
            self.log.prepare_calls += 1;
            Ok(())
        }

        fn acquire(
            &mut self,
            generation: u64,
            fallback_objective: Option<Objective>,
            force_challenge: bool,
        ) -> Result<AcquiredBoard, String> {
            if self.stop_on_generation == Some(generation) {
                return Err("Scripted stop after restart".to_owned());
            }
            self.log
                .acquire_calls
                .push((generation, fallback_objective, force_challenge));
            let script = self
                .boards
                .pop_front()
                .ok_or_else(|| format!("missing scripted board for cycle {generation}"))?;
            let objective = script.objective;
            let acquired = AcquiredBoard {
                export_path: PathBuf::from(format!("mock-tripeaks-g{generation}.json")),
                document: ScriptedBoard::sample_document(objective),
                bytes: Vec::new(),
                objective,
                detected_challenge: script.detected_challenge,
                root_hash: script.root_hash,
                completed_during_recon: script.completed_during_recon,
                terminal_challenge_complete: script.terminal_challenge_complete,
                terminal_board_win: script.terminal_board_win,
                counts: SlotKnowledgeCounts {
                    known: 52,
                    unknown: 0,
                    unreachable: 0,
                },
                probes_used: 1,
            };
            self.current = Some(script);
            Ok(acquired)
        }

        fn plan(
            &mut self,
            generation: u64,
            acquired: &AcquiredBoard,
        ) -> Result<PlanDecision<Self::Route>, String> {
            self.log.plan_calls.push((generation, acquired.objective));
            let script = self.current.as_ref().expect("current board");
            Ok(match &script.plan_decision {
                PlanDecision::Solved {
                    route,
                    escalated_clear_board,
                } => PlanDecision::Solved {
                    route: route.clone(),
                    escalated_clear_board: *escalated_clear_board,
                },
                PlanDecision::UnsolvablePartial {
                    route,
                    objective,
                    banked_rank_count,
                    score_delta,
                    cards_cleared,
                } => PlanDecision::UnsolvablePartial {
                    route: route.clone(),
                    objective: *objective,
                    banked_rank_count: *banked_rank_count,
                    score_delta: *score_delta,
                    cards_cleared: *cards_cleared,
                },
            })
        }

        fn execute(
            &mut self,
            generation: u64,
            _acquired: &AcquiredBoard,
            route: Self::Route,
            allow_final_modal: bool,
        ) -> Result<ExecutedBoard, String> {
            self.log
                .execute_calls
                .push((generation, route, allow_final_modal));
            Ok(self.current.as_ref().expect("current board").executed)
        }

        fn wait_next_board_after_clear(
            &mut self,
            _generation: u64,
            _previous_root_hash: u64,
            _challenge_mode: bool,
        ) -> Result<PostClearTransition, String> {
            self.current
                .as_ref()
                .and_then(|b| b.post_clear)
                .ok_or_else(|| "missing post_clear scripted transition".to_owned())
        }

        fn finish_terminal_win_images(&mut self) -> Result<(), String> {
            self.log.finish_win_calls += 1;
            Ok(())
        }

        fn restart_game(&mut self, generation: u64, previous_root_hash: u64) -> Result<(), String> {
            self.log
                .restart_calls
                .push((generation, previous_root_hash));
            Ok(())
        }
    }

    /// Build a solved board script with explicit transition completion.
    fn solved_script(
        root_hash: u64,
        objective: Objective,
        detected_challenge: Option<TriPeaksChallenge>,
        escalated_clear_board: bool,
        completion: tripeaks_route::Completion,
        score_delta: u32,
        cards_cleared: u8,
    ) -> ScriptedBoard {
        ScriptedBoard {
            root_hash,
            objective,
            detected_challenge,
            completed_during_recon: false,
            terminal_challenge_complete: false,
            terminal_board_win: false,
            plan_decision: PlanDecision::Solved {
                route: format!("route-{root_hash:#x}"),
                escalated_clear_board,
            },
            executed: ExecutedBoard {
                completion,
                cards_cleared,
                score_delta,
            },
            post_clear: None,
        }
    }

    /// Verify 3-board `Continuous` restart, duplicate-hash refusal, and 1-board `SingleBoard` stop.
    #[test]
    fn continuous_normal_three_board_game_and_post_win_restart() {
        let mut io = MockSessionIo::new(vec![
            solved_script(
                0x1001,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x1002 },
                2_400,
                28,
            ),
            solved_script(
                0x1002,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x1003 },
                2_400,
                28,
            ),
            solved_script(
                0x1003,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::BoardWinOverlay,
                2_400,
                28,
            ),
            solved_script(
                0x1004,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x1005 },
                2_400,
                28,
            ),
        ]);
        io.stop_on_generation = Some(5);
        let err = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut io)
            .expect_err("scripted stop on generation 5");
        assert!(err.contains("Scripted stop"));
        assert_eq!(io.log.prepare_calls, 1);
        assert_eq!(io.log.restart_calls, vec![(3, 0x1003)]);
        assert_eq!(io.log.execute_calls.len(), 4);

        // Repeating `previous_root_hash` on cycle 2 must fail closed.
        let mut dup_io = MockSessionIo::new(vec![
            solved_script(
                0x2001,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x2002 },
                2_000,
                28,
            ),
            solved_script(
                0x2001,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::BoardWinOverlay,
                2_000,
                28,
            ),
        ]);
        let dup_err = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut dup_io)
            .expect_err("duplicate root hash must refuse");
        assert!(dup_err.contains("unchanged board root hash"));

        // `SingleBoard` stops cleanly after board 1 with zero restarts.
        let mut single_io = MockSessionIo::new(vec![solved_script(
            0x3001,
            Objective::ClearBoard,
            None,
            false,
            tripeaks_route::Completion::NextBoardDealt { root_hash: 0x3002 },
            2_000,
            28,
        )]);
        let outcome = drive_tripeaks_session(TriPeaksPlayStyle::SingleBoard, &mut single_io)
            .expect("SingleBoard exits cleanly");
        assert_eq!(outcome.completed_boards, 1);
        assert_eq!(outcome.completed_games, 0);
        assert_eq!(single_io.log.restart_calls.len(), 0);
    }

    /// Multi-board `ClearRankQuota` and `ReachScoreDelta` escalate cleanly and stop at `Challenge Complete!` with `0` `Continue` clicks.
    #[test]
    fn multi_board_challenge_escalates_clear_board_and_completes_on_second_deal() {
        let mut rank_io = MockSessionIo::new(vec![
            solved_script(
                0x4001,
                Objective::ClearRankQuota {
                    rank: 3,
                    remaining: 6,
                },
                Some(TriPeaksChallenge::ClearRank {
                    rank: Rank::Four,
                    current: 0,
                    target: 6,
                }),
                true,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x4002 },
                10_000,
                28,
            ),
            solved_script(
                0x4002,
                Objective::ClearRankQuota {
                    rank: 3,
                    remaining: 2,
                },
                Some(TriPeaksChallenge::ClearRank {
                    rank: Rank::Four,
                    current: 4,
                    target: 6,
                }),
                false,
                tripeaks_route::Completion::ChallengeComplete,
                5_000,
                14,
            ),
        ]);
        let rank_done = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut rank_io)
            .expect("multi-board rank quota Challenge completes");
        assert_eq!(rank_done.completed_boards, 2);
        assert_eq!(rank_done.completed_games, 1);
        assert!(rank_done.summary.contains("zero Continue clicks sent"));
        assert_eq!(rank_io.log.finish_win_calls, 1);
        assert_eq!(rank_io.log.restart_calls.len(), 0);

        let mut score_io = MockSessionIo::new(vec![
            solved_script(
                0x5001,
                Objective::ReachScoreDelta {
                    points_needed: 60_000,
                },
                Some(TriPeaksChallenge::Score {
                    current: 0,
                    target: 60_000,
                }),
                true,
                tripeaks_route::Completion::NextBoardDealt { root_hash: 0x5002 },
                40_000,
                28,
            ),
            solved_script(
                0x5002,
                Objective::ClearBoard,
                None,
                false,
                tripeaks_route::Completion::ChallengeComplete,
                25_000,
                20,
            ),
        ]);
        let score_done = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut score_io)
            .expect("multi-board score Challenge completes");
        assert_eq!(score_done.completed_boards, 2);
        assert_eq!(score_done.completed_games, 1);
        assert_eq!(
            score_io.log.plan_calls[1],
            (
                2,
                Objective::ReachScoreDelta {
                    points_needed: 20_000
                }
            )
        );
        assert!(score_io.log.acquire_calls[1].2);
    }

    /// `completed_during_recon` bypasses A* planning and playback execution completely.
    #[test]
    fn completed_during_recon_skips_planning_and_execution() {
        let mut io = MockSessionIo::new(vec![ScriptedBoard {
            root_hash: 0x6001,
            objective: Objective::ClearRankQuota {
                rank: 3,
                remaining: 1,
            },
            detected_challenge: Some(TriPeaksChallenge::ClearRank {
                rank: Rank::Four,
                current: 0,
                target: 1,
            }),
            completed_during_recon: true,
            terminal_challenge_complete: true,
            terminal_board_win: false,
            plan_decision: PlanDecision::Solved {
                route: "unused".to_owned(),
                escalated_clear_board: false,
            },
            executed: ExecutedBoard {
                completion: tripeaks_route::Completion::ChallengeComplete,
                cards_cleared: 1,
                score_delta: 200,
            },
            post_clear: None,
        }]);
        let done = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut io)
            .expect("recon completion returns cleanly");
        assert_eq!(done.completed_boards, 1);
        assert_eq!(done.completed_games, 1);
        assert_eq!(io.log.plan_calls.len(), 0);
        assert_eq!(io.log.execute_calls.len(), 0);
        assert_eq!(io.log.finish_win_calls, 1);
    }

    /// Unsolvable partial paths bank positive Challenge rank/score deltas and send `0` clicks on `ClearBoard`.
    #[test]
    fn unsolvable_partial_banks_positive_rank_or_score_vs_zero_click_clear_board() {
        let mut rank_partial = MockSessionIo::new(vec![ScriptedBoard {
            root_hash: 0x7001,
            objective: Objective::ClearRankQuota {
                rank: 3,
                remaining: 5,
            },
            detected_challenge: Some(TriPeaksChallenge::ClearRank {
                rank: Rank::Four,
                current: 0,
                target: 5,
            }),
            completed_during_recon: false,
            terminal_challenge_complete: false,
            terminal_board_win: false,
            plan_decision: PlanDecision::UnsolvablePartial {
                route: Some("bank-rank".to_owned()),
                objective: Objective::ClearRankQuota {
                    rank: 3,
                    remaining: 5,
                },
                banked_rank_count: 2,
                score_delta: 600,
                cards_cleared: 12,
            },
            executed: ExecutedBoard {
                completion: tripeaks_route::Completion::PartialComplete,
                cards_cleared: 6,
                score_delta: 600,
            },
            post_clear: None,
        }]);
        let out_rank = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut rank_partial)
            .expect("rank partial banks cleanly");
        assert_eq!(out_rank.unsolvable_max_cleared, Some(12));
        assert_eq!(
            rank_partial.log.execute_calls,
            vec![(1, "bank-rank".to_owned(), true)]
        );

        let mut score_partial = MockSessionIo::new(vec![ScriptedBoard {
            root_hash: 0x7002,
            objective: Objective::ReachScoreDelta {
                points_needed: 99_000,
            },
            detected_challenge: Some(TriPeaksChallenge::Score {
                current: 0,
                target: 99_000,
            }),
            completed_during_recon: false,
            terminal_challenge_complete: false,
            terminal_board_win: false,
            plan_decision: PlanDecision::UnsolvablePartial {
                route: Some("bank-score".to_owned()),
                objective: Objective::ReachScoreDelta {
                    points_needed: 99_000,
                },
                banked_rank_count: 0,
                score_delta: 1_500,
                cards_cleared: 10,
            },
            executed: ExecutedBoard {
                completion: tripeaks_route::Completion::PartialComplete,
                cards_cleared: 5,
                score_delta: 1_500,
            },
            post_clear: None,
        }]);
        let _ = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut score_partial)
            .expect("score partial banks cleanly");
        assert_eq!(
            score_partial.log.execute_calls,
            vec![(1, "bank-score".to_owned(), true)]
        );

        let mut clear_dead = MockSessionIo::new(vec![ScriptedBoard {
            root_hash: 0x7003,
            objective: Objective::ClearBoard,
            detected_challenge: None,
            completed_during_recon: false,
            terminal_challenge_complete: false,
            terminal_board_win: false,
            plan_decision: PlanDecision::UnsolvablePartial {
                route: None,
                objective: Objective::ClearBoard,
                banked_rank_count: 0,
                score_delta: 0,
                cards_cleared: 9,
            },
            executed: ExecutedBoard {
                completion: tripeaks_route::Completion::PartialComplete,
                cards_cleared: 0,
                score_delta: 0,
            },
            post_clear: None,
        }]);
        let out_clear = drive_tripeaks_session(TriPeaksPlayStyle::Continuous, &mut clear_dead)
            .expect("clear dead end returns without clicks");
        assert_eq!(clear_dead.log.execute_calls.len(), 0);
        assert_eq!(out_clear.unsolvable_max_cleared, Some(9));
        assert!(out_clear.summary.contains("zero post-recon clicks sent"));
    }

    /// Copy `src_rect` from `src` into `dst` at `(dst_x, dst_y)`.
    fn copy_rect_pixels(
        dst: &mut CapturedFrame,
        dst_origin: PixelPoint,
        src: &CapturedFrame,
        src_rect: PixelRect,
    ) {
        let row_bytes = src_rect.width as usize * 4;
        for row in 0..src_rect.height {
            let src_y = (src_rect.y + row) as usize;
            let dst_y = (dst_origin.y as u32 + row) as usize;
            let src_off = src_y * src.stride + src_rect.x as usize * 4;
            let dst_off = dst_y * dst.stride + (dst_origin.x as u32) as usize * 4;
            dst.pixels[dst_off..dst_off + row_bytes]
                .copy_from_slice(&src.pixels[src_off..src_off + row_bytes]);
        }
    }

    /// Verify [`post_game::tripeaks_start_readiness`] accepts fresh 1-of-3 normal starts and rejects Challenge/duplicate/2-of-3 frames.
    #[test]
    fn start_readiness_accepts_fresh_normal_and_rejects_challenge_or_duplicate_hash() {
        let base = assets();
        let banner_rect = PixelRect::new(800, 30, 320, 66);
        let normal_hdr_rect = PixelRect::new(1_658, 46, 160, 32);
        let normal_ftr_rect = PixelRect::new(212, 998, 58, 31);
        let mut fresh_normal = base.left_ace.clone();
        copy_rect_pixels(
            &mut fresh_normal,
            PixelPoint::new(1_658, 46),
            &base.normal_one,
            normal_hdr_rect,
        );
        copy_rect_pixels(
            &mut fresh_normal,
            PixelPoint::new(212, 998),
            &base.normal_one,
            normal_ftr_rect,
        );

        let snap = post_game::tripeaks_start_readiness(&fresh_normal, None)
            .expect("fresh normal start outer Ok")
            .expect("fresh normal start ready");
        let hash = semantic_board_hash(&snap);

        let dup_pending = post_game::tripeaks_start_readiness(&fresh_normal, Some(hash))
            .expect("duplicate hash remains outer Ok");
        assert!(dup_pending.is_err());

        let mut board_two = base.left_ace.clone();
        copy_rect_pixels(
            &mut board_two,
            PixelPoint::new(1_658, 46),
            &base.normal_two,
            normal_hdr_rect,
        );
        copy_rect_pixels(
            &mut board_two,
            PixelPoint::new(212, 998),
            &base.normal_two,
            normal_ftr_rect,
        );
        let board_two_err = post_game::tripeaks_start_readiness(&board_two, None)
            .expect_err("2-of-3 header contradicts 1-of-3 start");
        assert!(board_two_err.contains("contradicts"));

        let mut challenge = base.left_ace.clone();
        copy_rect_pixels(
            &mut challenge,
            PixelPoint::new(800, 30),
            &base.fours_header,
            banner_rect,
        );
        let challenge_err = post_game::tripeaks_start_readiness(&challenge, None)
            .expect_err("Challenge header must not authorise normal restart");
        assert!(challenge_err.contains("challenge"));
    }

    /// RAII guard temporarily installing `WORKER_IMAGE_DIAGNOSTICS` on the current worker thread.
    struct DiagnosticsRestore(Option<WorkerImageDiagnostics>);

    impl DiagnosticsRestore {
        /// Install `ImageDiagnostics` with optional recording directory and save previous TLS state.
        fn install(directory: Option<PathBuf>) -> Self {
            let config = ImageDiagnostics {
                started_at: Instant::now(),
                directory,
            };
            Self(
                super::super::WORKER_IMAGE_DIAGNOSTICS
                    .with(|slot| slot.replace(Some(WorkerImageDiagnostics::new(config)))),
            )
        }
    }

    impl Drop for DiagnosticsRestore {
        /// Restore original TLS recording state even on assertion unwind.
        fn drop(&mut self) {
            super::super::WORKER_IMAGE_DIAGNOSTICS.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }

    /// Construct a Challenge test deal with `Waste = Five`, `slot 18 = Four`, and all other rows unreachable.
    fn single_four_challenge_slots() -> [CardSlot; DEAL_SLOT_COUNT] {
        let mut used = [false; 52];
        let mut suit_cursor = [0_u8; 13];
        let mut take = |rank_idx: u8| -> Card {
            let suit = suit_cursor[usize::from(rank_idx)];
            suit_cursor[usize::from(rank_idx)] = suit + 1;
            let card = Card(suit * 13 + rank_idx);
            used[usize::from(card.0)] = true;
            card
        };
        let mut cards = [Card(0); DEAL_SLOT_COUNT];
        cards[usize::from(INITIAL_WASTE_SLOT.0)] = take(4);
        cards[18] = take(3);
        for (slot, r) in (19..28).zip([0_u8, 0, 0, 0, 1, 1, 1, 12, 12]) {
            cards[slot] = take(r);
        }
        let hidden = [
            4_u8, 4, 4, 3, 3, 3, 1, 12, 12, 2, 2, 2, 2, 11, 11, 11, 11, 10,
        ];
        for (slot, r) in (0..18).zip(hidden) {
            cards[slot] = take(r);
        }
        let mut stock_idx = usize::from(STOCK_START_SLOT);
        for (id, &is_used) in used.iter().enumerate() {
            if !is_used {
                cards[stock_idx] = Card(id as u8);
                stock_idx += 1;
            }
        }
        assert_eq!(stock_idx, DEAL_SLOT_COUNT);
        let slots = cards.map(CardSlot::Known);
        assert!(validate_slot_table(&slots).is_ok());
        slots
    }

    /// Paint one `CLASSIC BLUE` face-down card back onto `dst`.
    fn paint_face_down(dst: &mut CapturedFrame, dst_x: u32, dst_y: u32, width: u32) {
        let base = assets();
        let copy_w = width.min(STOCK_CARD_BOUNDS.width);
        copy_rect_pixels(
            dst,
            PixelPoint::new(dst_x as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(STOCK_CARD_BOUNDS.x, STOCK_CARD_BOUNDS.y, copy_w, 181),
        );
        if width > copy_w {
            copy_rect_pixels(
                dst,
                PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
                &base.left_ace,
                PixelRect::new(
                    STOCK_CARD_BOUNDS.x + copy_w - 1,
                    STOCK_CARD_BOUNDS.y,
                    1,
                    181,
                ),
            );
        }
    }

    /// Paint one synthetic face-up card corner from `FC01`.
    fn paint_face_up(dst: &mut CapturedFrame, dst_x: u32, dst_y: u32, width: u32, target: Card) {
        let base = assets();
        let copy_w = width.min(WASTE_CARD_BOUNDS.width);
        copy_rect_pixels(
            dst,
            PixelPoint::new(dst_x as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(WASTE_CARD_BOUNDS.x, WASTE_CARD_BOUNDS.y, copy_w, 181),
        );
        if width > copy_w {
            copy_rect_pixels(
                dst,
                PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
                &base.left_ace,
                PixelRect::new(
                    WASTE_CARD_BOUNDS.x + copy_w - 1,
                    WASTE_CARD_BOUNDS.y,
                    1,
                    181,
                ),
            );
        }
        for dy in 4..177_u32 {
            for dx in 4..width.saturating_sub(4) {
                let offset = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
                dst.pixels[offset..offset + 4].copy_from_slice(&[254, 254, 254, 255]);
            }
        }
        let (src_x, src_y) = base.corner_origins[usize::from(target.0)];
        for dy in 0..52_u32 {
            for dx in 0..34_u32 {
                if dx + dy < 3 {
                    continue;
                }
                let so = (src_y + dy) as usize * base.fc01.stride + (src_x + dx) as usize * 4;
                let doff = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
                dst.pixels[doff..doff + 4].copy_from_slice(&base.fc01.pixels[so..so + 4]);
            }
        }
    }

    /// Convert one RGBA frame into packed 24-bit BGR bytes for TCP `BgrV3`.
    fn frame_to_bgr(frame: &CapturedFrame) -> Vec<u8> {
        let mut bgr = Vec::with_capacity(frame.pixels.len() / 4 * 3);
        for px in frame.pixels.as_chunks::<4>().0 {
            bgr.extend_from_slice(&[px[2], px[1], px[0]]);
        }
        bgr
    }

    /// Synthesize a packed 24-bit BGR Challenge gameplay or `Challenge Complete!` frame.
    fn render_challenge_bgr(
        slots: &[CardSlot; DEAL_SLOT_COUNT],
        position: TriPeaksPosition,
        undo_enabled: bool,
        challenge_complete: bool,
    ) -> Vec<u8> {
        let base = assets();
        if challenge_complete {
            return frame_to_bgr(&base.challenge_complete);
        }
        let mut frame = base.left_ace.clone();
        copy_rect_pixels(
            &mut frame,
            PixelPoint::new(800, 30),
            &base.fours_header,
            PixelRect::new(800, 30, 320, 66),
        );
        for y in 96..(96 + 545_u32) {
            let src_y = 650 + (y - 96) % 180;
            let mut x = 115_u32;
            while x < 115 + 1_690 {
                let chunk = ((115 + 1_690) - x).min(520);
                copy_rect_pixels(
                    &mut frame,
                    PixelPoint::new(x as i32, y as i32),
                    &base.left_ace,
                    PixelRect::new(128, src_y, chunk, 1),
                );
                x += chunk;
            }
        }
        for (idx, region) in TABLEAU_CARD_REGIONS.iter().enumerate() {
            let bounds = region.card_bounds;
            let bit = 1_u32 << idx;
            let obs = if (position.tableau_mask & bit) == 0 {
                TableauSlotObs::Empty
            } else if (COVERING_CHILD_MASKS[idx] & position.tableau_mask) != 0 {
                TableauSlotObs::FaceDown
            } else if let CardSlot::Known(card_val) = slots[idx] {
                TableauSlotObs::Card(card_val)
            } else {
                TableauSlotObs::FaceDown
            };
            match obs {
                TableauSlotObs::Empty => {}
                TableauSlotObs::FaceDown => {
                    paint_face_down(&mut frame, bounds.x, bounds.y, bounds.width)
                }
                TableauSlotObs::Card(c) => {
                    paint_face_up(&mut frame, bounds.x, bounds.y, bounds.width, c)
                }
            }
        }
        let stock = if position.stock_cursor >= STOCK_CARD_COUNT {
            StockObs::Empty
        } else {
            StockObs::FaceDown
        };
        if stock == StockObs::Empty {
            copy_rect_pixels(
                &mut frame,
                PixelPoint::new(TARGET_STOCK.x as i32, TARGET_STOCK.y as i32),
                &base.stock_empty,
                TARGET_STOCK,
            );
        }
        paint_face_up(
            &mut frame,
            WASTE_CARD_BOUNDS.x,
            WASTE_CARD_BOUNDS.y,
            WASTE_CARD_BOUNDS.width,
            position.waste_card,
        );
        if undo_enabled {
            let undo_bounds = SHARED_UNDO_ALL_CONTROL.bounds;
            copy_rect_pixels(
                &mut frame,
                PixelPoint::new(undo_bounds.x as i32, undo_bounds.y as i32),
                &base.draw_first,
                undo_bounds,
            );
        }
        frame_to_bgr(&frame)
    }

    /// Mutable live guest state shared between QMP and TCP `BgrV3` threads in session tests.
    struct LiveGuest {
        /// Ground-truth deal.
        slots: [CardSlot; DEAL_SLOT_COUNT],
        /// Root position at `T = 0`.
        root: TriPeaksPosition,
        /// Current gameplay position.
        position: TriPeaksPosition,
        /// Moves applied since `T = 0`.
        history: Vec<TriPeaksMove>,
        /// Set to `true` once tapping the Challenge Four finishes `Challenge Complete!`.
        challenge_complete: bool,
    }

    /// Cache key for live session `BgrV3` frames.
    #[derive(Clone, Copy, Eq, Hash, PartialEq)]
    struct LiveBgrKey {
        /// Remaining tableau bitmask.
        tableau_mask: u32,
        /// Stock draw cursor.
        stock_cursor: u8,
        /// Active Waste card ID.
        waste_id: u8,
        /// Whether `Undo All` toolbar control is active.
        undo_enabled: bool,
        /// Whether terminal `Challenge Complete!` overlay is displayed.
        challenge_complete: bool,
    }

    /// RAII local `AF_UNIX` QMP + TCP `BgrV3` loopback harness for `tripeaks_session::run`.
    struct LiveLoopback {
        /// Shutdown latch torn down on drop.
        shutdown: Arc<AtomicBool>,
        /// Thread-local `CaptureMode::BgrV3` guard.
        _backend: TestCaptureBackend,
        /// QMP listener thread handle.
        qmp_thread: Option<thread::JoinHandle<()>>,
        /// TCP `BgrV3` server thread handle.
        tcp_thread: Option<thread::JoinHandle<()>>,
        /// Temporary Unix socket file removed on drop.
        socket_path: PathBuf,
    }

    impl Drop for LiveLoopback {
        /// Stop background threads, join cleanly, and remove temporary socket file.
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::Release);
            if let Some(h) = self.qmp_thread.take() {
                let _ = h.join();
            }
            if let Some(h) = self.tcp_thread.take() {
                let _ = h.join();
            }
            let _ = fs::remove_file(&self.socket_path);
        }
    }

    /// Write one newline-delimited QMP reply packet.
    fn write_qmp(stream: &mut UnixStream, value: Value) -> bool {
        serde_json::to_writer(&mut *stream, &value).is_ok()
            && stream.write_all(b"\n").is_ok()
            && stream.flush().is_ok()
    }

    /// Inverse of QMP `px * 32767 / extent`.
    fn qmp_px(qmp: u32, extent: u32) -> i32 {
        ((u64::from(qmp) * u64::from(extent) + 16_383) / 32_767) as i32
    }

    /// Spawn local `AF_UNIX` QMP + TCP `BgrV3` Challenge loopback server triggering `Challenge Complete!` on tap `slot 18`.
    fn spawn_session_loopback(
        socket_path: PathBuf,
        slots: [CardSlot; DEAL_SLOT_COUNT],
    ) -> LiveLoopback {
        let _ = assets();
        let root = TriPeaksPosition::from_root(&slots).expect("valid root");
        let guest = Arc::new(Mutex::new(LiveGuest {
            slots,
            root,
            position: root,
            history: Vec::new(),
            challenge_complete: false,
        }));
        let shutdown = Arc::new(AtomicBool::new(false));

        let qmp_listener = UnixListener::bind(&socket_path).expect("bind session QMP socket");
        qmp_listener.set_nonblocking(true).expect("nonblocking");
        let qmp_guest = Arc::clone(&guest);
        let qmp_shutdown = Arc::clone(&shutdown);
        let qmp_thread = thread::spawn(move || {
            let mut last_abs: Option<(u32, u32)> = None;
            while !qmp_shutdown.load(Ordering::Acquire) {
                match qmp_listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(40)));
                        let mut reader = BufReader::new(stream.try_clone().expect("clone QMP"));
                        if !write_qmp(
                            &mut stream,
                            json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
                        ) {
                            continue;
                        }
                        while !qmp_shutdown.load(Ordering::Acquire) {
                            let mut line = String::new();
                            match reader.read_line(&mut line) {
                                Ok(0) => break,
                                Ok(_) => {}
                                Err(e)
                                    if matches!(
                                        e.kind(),
                                        ErrorKind::WouldBlock | ErrorKind::TimedOut
                                    ) =>
                                {
                                    continue;
                                }
                                Err(_) => break,
                            }
                            let Ok(req) = serde_json::from_str::<Value>(&line) else {
                                break;
                            };
                            let id = req["id"].as_u64().unwrap_or(0);
                            match req["execute"].as_str().unwrap_or("") {
                                "qmp_capabilities" => {
                                    let _ = write_qmp(&mut stream, json!({"return": {}, "id": id}));
                                }
                                "query-status" => {
                                    let _ = write_qmp(
                                        &mut stream,
                                        json!({"return": {"running": true, "status": "running"}, "id": id}),
                                    );
                                }
                                "query-mice" => {
                                    let _ = write_qmp(
                                        &mut stream,
                                        json!({"return": [{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}], "id": id}),
                                    );
                                }
                                "input-send-event" => {
                                    let events =
                                        req["arguments"]["events"].as_array().expect("events");
                                    let mut gx = None;
                                    let mut gy = None;
                                    for ev in events {
                                        match ev["type"].as_str() {
                                            Some("abs") => {
                                                let val = ev["data"]["value"].as_u64().unwrap_or(0)
                                                    as u32;
                                                match ev["data"]["axis"].as_str() {
                                                    Some("x") => gx = Some(val),
                                                    Some("y") => gy = Some(val),
                                                    _ => {}
                                                }
                                            }
                                            Some("btn") if ev["data"]["button"] == "left" => {
                                                let down =
                                                    ev["data"]["down"].as_bool().unwrap_or(false);
                                                if !down && let Some((qx, qy)) = last_abs {
                                                    let px = PixelPoint::new(
                                                        qmp_px(qx, NOMINAL_FRAME_WIDTH),
                                                        qmp_px(qy, NOMINAL_FRAME_HEIGHT),
                                                    );
                                                    let mut g = qmp_guest.lock().expect("guest");
                                                    if px == SHARED_UNDO_ALL_CONTROL.click_point {
                                                        g.history.clear();
                                                        g.position = g.root;
                                                    } else if let Some((slot_idx, _)) =
                                                        TABLEAU_CARD_REGIONS
                                                            .iter()
                                                            .enumerate()
                                                            .find(|(_, r)| {
                                                                r.click_point.x.abs_diff(px.x) <= 1
                                                                    && r.click_point
                                                                        .y
                                                                        .abs_diff(px.y)
                                                                        <= 1
                                                            })
                                                    {
                                                        let (row, col) = slot_to_tableau_coord(
                                                            SlotId(slot_idx as u8),
                                                        )
                                                        .expect("coord");
                                                        let mv = TriPeaksMove::TapTableau {
                                                            row,
                                                            column: col,
                                                        };
                                                        let (next, _) =
                                                            apply_move(&g.position, &g.slots, mv)
                                                                .expect("legal tap");
                                                        g.history.push(mv);
                                                        g.position = next;
                                                        g.challenge_complete = true;
                                                    }
                                                }
                                            }
                                            Some("key") => {
                                                let down =
                                                    ev["data"]["down"].as_bool().unwrap_or(false);
                                                if !down && ev["data"]["key"]["data"] == "d" {
                                                    let mut g = qmp_guest.lock().expect("guest");
                                                    let (next, _) = apply_move(
                                                        &g.position,
                                                        &g.slots,
                                                        TriPeaksMove::Draw,
                                                    )
                                                    .expect("legal draw");
                                                    g.history.push(TriPeaksMove::Draw);
                                                    g.position = next;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if let (Some(x), Some(y)) = (gx, gy) {
                                        last_abs = Some((x, y));
                                    }
                                    let _ = write_qmp(&mut stream, json!({"return": {}, "id": id}));
                                }
                                _ => {
                                    let _ = write_qmp(&mut stream, json!({"return": {}, "id": id}));
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });

        let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("bind BgrV3 listener");
        let address = tcp_listener.local_addr().expect("addr").to_string();
        tcp_listener.set_nonblocking(true).expect("nonblocking");
        let tcp_guest = Arc::clone(&guest);
        let tcp_shutdown = Arc::clone(&shutdown);
        let tcp_thread = thread::spawn(move || {
            let mut cache: HashMap<LiveBgrKey, Arc<Vec<u8>>> = HashMap::new();
            let mut seq = 0_u64;
            while !tcp_shutdown.load(Ordering::Acquire) {
                match tcp_listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(40)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                        let _ = serve_live_v3(
                            &mut stream,
                            &tcp_guest,
                            &tcp_shutdown,
                            &mut cache,
                            &mut seq,
                        );
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });

        LiveLoopback {
            shutdown,
            _backend: TestCaptureBackend::tcp_mode(address, CaptureMode::BgrV3),
            qmp_thread: Some(qmp_thread),
            tcp_thread: Some(tcp_thread),
            socket_path,
        }
    }

    /// Serve `BgrV3` frames cleanly for [`LiveLoopback`].
    fn serve_live_v3(
        stream: &mut TcpStream,
        guest: &Arc<Mutex<LiveGuest>>,
        shutdown: &AtomicBool,
        cache: &mut HashMap<LiveBgrKey, Arc<Vec<u8>>>,
        seq: &mut u64,
    ) -> std::io::Result<()> {
        let mut req_buf = [0_u8; v3::REQUEST_LEN];
        loop {
            if shutdown.load(Ordering::Acquire) {
                return Ok(());
            }
            match stream.read_exact(&mut req_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                    continue;
                }
                Err(e) => return Err(e),
            }
            let req = v3::Request::from_bytes(req_buf)
                .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, err))?;
            *seq += 1;
            let (slots, key) = {
                let g = guest.lock().expect("guest");
                (
                    g.slots,
                    LiveBgrKey {
                        tableau_mask: g.position.tableau_mask,
                        stock_cursor: g.position.stock_cursor,
                        waste_id: g.position.waste_card.0,
                        undo_enabled: !g.history.is_empty(),
                        challenge_complete: g.challenge_complete,
                    },
                )
            };
            let bgr = Arc::clone(cache.entry(key).or_insert_with(|| {
                let pos = TriPeaksPosition {
                    tableau_mask: key.tableau_mask,
                    stock_cursor: key.stock_cursor,
                    waste_card: Card(key.waste_id),
                    streak: 0,
                    score_delta: 0,
                    cleared_by_rank: [0; 13],
                };
                Arc::new(render_challenge_bgr(
                    &slots,
                    pos,
                    key.undo_enabled,
                    key.challenge_complete,
                ))
            }));
            let header = v3::ResponseHeader::success(
                1,
                req.request_id(),
                *seq,
                7,
                NOMINAL_FRAME_WIDTH,
                NOMINAL_FRAME_HEIGHT,
            )
            .expect("valid v3 response");
            stream.write_all(&header.to_bytes())?;
            stream.write_all(&bgr)?;
            stream.flush()?;
        }
    }

    /// Execute one live `SingleBoard` `tripeaks_session::run` and return `(completion, export_path)`.
    fn run_live_single_board(
        diag_dir: Option<PathBuf>,
    ) -> (TriPeaksSessionCompletion, Option<PathBuf>) {
        let id = NEXT_SESSION_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let sock = std::env::temp_dir().join(format!(
            "tripeaks-session-live-{}-{id}.sock",
            std::process::id()
        ));
        let slots = single_four_challenge_slots();
        let _loopback = spawn_session_loopback(sock.clone(), slots);
        let _diag_guard = DiagnosticsRestore::install(diag_dir);

        let context =
            ControllerContext::new(sock, GameMode::TriPeaks, SolvingStrategy::ShortestPath, id);
        let (tx, rx) = mpsc::channel();
        let sink = WorkerEventSink {
            events: tx,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(None),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let _cap_cancel =
            super::super::tcp_capture::CancellationScope::install(Arc::clone(&cancel));
        sink.set_capture_context(Some(context.clone()));
        sink.latest_frame
            .reconnaissance
            .select(&context, &cancel)
            .expect("select context");
        sink.latest_frame
            .reconnaissance
            .queue_reconnaissance(&context, &cancel, || Ok(()))
            .expect("queue recon");

        let request = TriPeaksSessionRequest {
            run_id: id,
            play_style: TriPeaksPlayStyle::SingleBoard,
            step_timing: TriPeaksStepTiming {
                move_settle_delay: Duration::from_millis(1),
                reobserve_delay: Duration::from_millis(1),
                restore_settle_delay: Duration::from_millis(1),
                max_step_observations: 6,
                max_restore_observations: 8,
            },
            board_redeal_settle: Duration::from_millis(1),
        };
        run(&context, &request, &sink, &cancel);

        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let rem = deadline.saturating_duration_since(Instant::now());
            let item = rx.recv_timeout(rem).expect("TriPeaksSessionFinished");
            if let WorkerEvent::TriPeaksSessionFinished(done) = item.event {
                let path = done.export_path.clone();
                return (done, path);
            }
        }
    }

    /// Exercise live `AF_UNIX` QMP + `BgrV3` `tripeaks_session::run` with `GAME WIN` image recording enabled vs `--no-image`.
    #[test]
    fn live_single_board_session_respects_game_win_image_recording_flag() {
        let id = NEXT_SESSION_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let enabled_dir = std::env::temp_dir().join(format!(
            "tripeaks-session-win-images-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&enabled_dir).expect("create enabled win-image dir");

        let (done_enabled, export_enabled) = run_live_single_board(Some(enabled_dir.clone()));
        if let Some(ref p) = export_enabled {
            let _ = fs::remove_file(p);
        }
        assert!(
            done_enabled.result.is_ok(),
            "term={:?} err={:?}",
            done_enabled.termination,
            done_enabled.result
        );
        assert_eq!(done_enabled.termination, RunTermination::Completed);
        assert_eq!(done_enabled.completed_boards, 1);
        assert_eq!(done_enabled.completed_games, 1);
        let saved_pngs: Vec<_> = fs::read_dir(&enabled_dir)
            .expect("read enabled win dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
            .collect();
        assert!(
            !saved_pngs.is_empty(),
            "enabled session must persist entry/terminal GAME WIN PNGs"
        );
        let _ = fs::remove_dir_all(&enabled_dir);

        let (done_disabled, export_disabled) = run_live_single_board(None);
        if let Some(ref p) = export_disabled {
            let _ = fs::remove_file(p);
        }
        assert!(
            done_disabled.result.is_ok(),
            "term={:?} err={:?}",
            done_disabled.termination,
            done_disabled.result
        );
        assert_eq!(done_disabled.termination, RunTermination::Completed);
        assert!(done_disabled.result.is_ok(), "{:?}", done_disabled.result);
        assert_eq!(done_disabled.completed_boards, 1);
        assert_eq!(done_disabled.completed_games, 1);
    }
}
