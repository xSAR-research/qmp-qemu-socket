#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Application-owned TriPeaks workflow modes, card-knowledge counters, and advisory events.
//!
//! Uses plain operational terminology (`Known / Unknown / Unreachable`, `Probes used`) without
//! granting QMP input or execution authority.

use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::{
    pyramid_workflow_model::{RunTermination, SessionPhase},
    recon::{CardSlot, Objective},
    strategy::ControllerContext,
    tripeaks_model::DEAL_SLOT_COUNT,
    tripeaks_progress::TriPeaksChallenge,
};

/// Operator-visible card knowledge counts across the `52`-slot TriPeaks deal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SlotKnowledgeCounts {
    /// Slots whose rank and suit are positively known.
    pub known: u8,
    /// Slots still hidden (`CardSlot::Unknown`).
    pub unknown: u8,
    /// Hidden slots proved legally unreachable (`CardSlot::Unreachable`).
    pub unreachable: u8,
}

impl SlotKnowledgeCounts {
    /// Count `Known`, `Unknown`, and `Unreachable` slots from a slice (normalised to `52` max).
    pub fn from_slots(slots: &[CardSlot]) -> Self {
        let mut counts = Self::default();
        for entry in slots.iter().take(DEAL_SLOT_COUNT) {
            match entry {
                CardSlot::Known(_) => counts.known = counts.known.saturating_add(1),
                CardSlot::Unknown => counts.unknown = counts.unknown.saturating_add(1),
                CardSlot::Unreachable => counts.unreachable = counts.unreachable.saturating_add(1),
            }
        }
        counts
    }

    /// Format plain operator-facing summary text for status panels and logs.
    pub fn summary_line(self, probes_used: usize) -> String {
        format!(
            "Known: {}/{} · Unknown: {} · Unreachable: {} · Probes used: {}",
            self.known, DEAL_SLOT_COUNT, self.unknown, self.unreachable, probes_used
        )
    }
}

/// Format an active Challenge/Classic TriPeaks objective in plain operator terms.
pub fn format_tripeaks_objective(
    objective: Objective,
    challenge: Option<TriPeaksChallenge>,
) -> String {
    if let Some(challenge) = challenge {
        return match challenge {
            TriPeaksChallenge::ClearRank {
                rank,
                current,
                target,
            } => format!("Clear {} ({current}/{target})", rank_plural_label(rank)),
            TriPeaksChallenge::Boards { current, target } => {
                format!("Boards cleared ({current}/{target})")
            }
            TriPeaksChallenge::Score { current, target } => {
                format!("Score ({current}/{target})")
            }
            TriPeaksChallenge::SolveDeck => "Solve the deck".to_owned(),
            TriPeaksChallenge::UnsupportedSingleCard => {
                "Clear specific card (unsupported)".to_owned()
            }
        };
    }
    match objective {
        Objective::ClearBoard => "Clear board".to_owned(),
        Objective::ClearRankQuota { rank, remaining } => {
            format!("Clear {remaining} rank-{} cards", rank.saturating_add(1))
        }
        Objective::ReachScoreDelta { points_needed } => {
            format!("Reach +{points_needed} points")
        }
    }
}

/// Return a plural English label for a standard playing-card rank.
fn rank_plural_label(rank: xsar::games::cards::Rank) -> &'static str {
    use xsar::games::cards::Rank;
    match rank {
        Rank::Ace => "Aces",
        Rank::Two => "Twos",
        Rank::Three => "Threes",
        Rank::Four => "Fours",
        Rank::Five => "Fives",
        Rank::Six => "Sixes",
        Rank::Seven => "Sevens",
        Rank::Eight => "Eights",
        Rank::Nine => "Nines",
        Rank::Ten => "Tens",
        Rank::Jack => "Jacks",
        Rank::Queen => "Queens",
        Rank::King => "Kings",
    }
}

/// Operator-selected execution mode for independent A* TriPeaks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TriPeaksPlayStyle {
    /// Plays 3-board normal games or multi-board challenges continuously across clears until
    /// `Challenge Complete!`, `STOP`, or an unsolvable dead end without `ClearBoard`.
    #[default]
    Continuous,
    /// Runs Stock sweep, probing, route planning, and execution on one current board, then stops.
    SingleBoard,
    /// Exposes explicit manual `RECON / Export deal`, offline `CALCULATE`, `SINGLE STEP` /
    /// `MULTI STEP` playback, and `UNDO ALL`.
    Manual,
}

impl TriPeaksPlayStyle {
    /// Fixed selector order rendered in the TriPeaks `Mode` drop-down.
    pub const ALL: [Self; 3] = [Self::Continuous, Self::SingleBoard, Self::Manual];

    /// Concise operator-facing label rendered in the TriPeaks `Mode` drop-down.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Continuous => "Continuous",
            Self::SingleBoard => "Single board",
            Self::Manual => "Manual",
        }
    }

    /// Plain summary rendered below the TriPeaks `Mode` drop-down.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Continuous => {
                "Sweeps Stock, probes hidden cards as needed, solves normal 3-board games or multi-board challenges until Challenge Complete, an unsolvable deal, or STOP."
            }
            Self::SingleBoard => {
                "Runs RECON, A* planning, and execution on one current TriPeaks board and stops cleanly without starting a new game."
            }
            Self::Manual => {
                "Manual deal export, offline route calculation (with multi-board ClearBoard fallback), SINGLE STEP / MULTI STEP playback, and UNDO ALL."
            }
        }
    }

    /// Whether pressing `START` runs the worker-owned automatic TriPeaks session.
    pub const fn uses_session(self) -> bool {
        !matches!(self, Self::Manual)
    }
}

/// Immutable request dispatched by the UI to run one worker-owned TriPeaks session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TriPeaksSessionRequest {
    /// Monotonic session identity, never reused across runs.
    pub(crate) run_id: u64,
    /// Automatic stopping policy (`Continuous` or `SingleBoard`).
    pub(crate) play_style: TriPeaksPlayStyle,
    /// Step, observation, and restoration timings applied during recon and route execution.
    pub(crate) step_timing: crate::worker::tripeaks_reconnaissance::TriPeaksStepTiming,
    /// Settle delay after clearing board `1/3` or `2/3` before polling for the next deal.
    pub(crate) board_redeal_settle: Duration,
}

/// Replaceable live search snapshot emitted while planning a TriPeaks board route.
#[derive(Clone, Debug)]
pub(crate) struct TriPeaksPlanningSnapshot {
    /// Bounded A* / best-effort search statistics from `xsar::search`.
    pub(crate) search: xsar::search::SearchProgress,
    /// Cumulative cooperative Pause time during planning.
    pub(crate) paused: Duration,
}

/// Advisory progress emitted by the worker during an automatic TriPeaks session.
#[derive(Clone, Debug)]
pub struct TriPeaksSessionProgress {
    /// Controller identity that owns this TriPeaks session.
    pub(crate) context: ControllerContext,
    /// Monotonic session identity matching the dispatched request.
    pub(crate) run_id: u64,
    /// One-based board cycle counter within this session.
    pub(crate) deal_generation: u64,
    /// Completed TriPeaks boards verified in this session so far.
    pub(crate) completed_boards: usize,
    /// Completed full 3-board normal games verified in this session so far.
    pub(crate) completed_games: usize,
    /// Current worker-owned session phase.
    pub(crate) phase: SessionPhase,
    /// Latest operator card-knowledge counts (`Known / Unknown / Unreachable`).
    pub(crate) counts: Option<SlotKnowledgeCounts>,
    /// Cumulative `Undo All` probing cycles completed on the current board.
    pub(crate) probes_used: usize,
    /// Plain-English description of the active board's objective.
    pub(crate) objective_label: Option<String>,
    /// Execution cursor `(completed_moves, total_moves)` during playback.
    pub(crate) route_cursor: Option<(usize, usize)>,
    /// Expanded A* states observed on the active planning pass.
    pub(crate) expanded_states: Option<u64>,
    /// Latest A* search statistics while planning the active board.
    pub(crate) planning_progress: Option<TriPeaksPlanningSnapshot>,
    /// Determined A* calculation report for UI summary panels once ready.
    pub(crate) calculation_report: Option<Arc<crate::worker::tripeaks_calculation::Report>>,
    /// Exported `52`-slot JSON path for the active board once published.
    pub(crate) export_path: Option<PathBuf>,
    /// Concise operator status line.
    pub(crate) message: String,
}

/// Final completion event releasing automatic TriPeaks session ownership in the UI.
#[derive(Clone, Debug)]
pub struct TriPeaksSessionCompletion {
    /// Exact controller identity under which this TriPeaks session ran.
    pub(crate) context: ControllerContext,
    /// Session identity that must still own the UI.
    pub(crate) run_id: u64,
    /// Completed TriPeaks boards verified during this session.
    pub(crate) completed_boards: usize,
    /// Completed full 3-board normal games verified during this session.
    pub(crate) completed_games: usize,
    /// Causal completion classification, never inferred from UI text.
    pub(crate) termination: RunTermination,
    /// Final card-knowledge summary from the last processed deal.
    pub(crate) counts: Option<SlotKnowledgeCounts>,
    /// Plain-English description of the final active objective, if determined.
    pub(crate) objective_label: Option<String>,
    /// Populated when an unsolvable board halted without `ClearBoard` (`0..=28` max clearable cards).
    pub(crate) unsolvable_max_cleared: Option<u8>,
    /// Final determined A* calculation report from the last board processed, if any.
    pub(crate) calculation_report: Option<Arc<crate::worker::tripeaks_calculation::Report>>,
    /// Last exported `52`-slot deal JSON path, when published in this session.
    pub(crate) export_path: Option<PathBuf>,
    /// Retained failure diagnostic directory, when saved.
    pub(crate) failure_directory: Option<PathBuf>,
    /// Final operator summary or causal error description.
    pub(crate) result: Result<String, String>,
}

/// Summary published when a standalone Manual TriPeaks reconnaissance run completes.
#[derive(Clone, Debug)]
pub struct TriPeaksManualReconSummary {
    /// Published `52`-slot deal JSON path.
    pub path: PathBuf,
    /// Resolved Challenge or Classic [`Objective`] for the acquired deal.
    pub objective: Objective,
    /// Challenge banner details observed on the initial `T = 0` capture, if present.
    pub challenge: Option<TriPeaksChallenge>,
    /// Card knowledge counts (`Known / Unknown / Unreachable`) across all `52` slots.
    pub counts: SlotKnowledgeCounts,
    /// Cumulative `Undo All` probe cycles executed after the Stock sweep.
    pub probes_used: usize,
    /// Deterministic 64-bit semantic hash of the verified `T = 0` root board.
    pub root_hash: u64,
    /// `true` when probing satisfied the active win/challenge directly without restoring to root.
    pub completed_during_recon: bool,
    /// Plain operator summary free of jargon.
    pub summary: String,
}
