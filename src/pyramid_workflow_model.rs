//! Application-owned requests and advisory progress for independent Pyramid work.
//!
//! These messages contain no acquisition receipt or input permit. The serial
//! worker retains authority across automatic phases and validates each operation.

use crate::{pyramid_recon::ReconRequest, strategy::ControllerContext};


mod observed;


pub(crate) use observed::ObservedGame;


/// Operator-selected lifetime of independent Pyramid work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum WorkflowMode {
    /// Each acquisition, calculation and execution needs a separate button press.
    Manual,
    /// Acquire, calculate and execute one current board, then stop.
    SingleBoard,
    /// Continue under the separately selected normal-repeat or challenge-target condition.
    #[default]
    Continuous,
}


impl WorkflowMode {
    /// Return the stable operator-facing mode label without granting authority.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::SingleBoard => "Single board",
            Self::Continuous => "Continuous",
        }
    }


    /// Describe the exact work authorised by the corresponding explicit buttons.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Manual => "Recon, Plan route and Solve board remain three separate steps.",
            Self::SingleBoard => "Acquire, plan and solve the current board once, then stop.",
            Self::Continuous => {
                "Continue under the selected stop condition; STOP, uncertainty or a verified partial route releases control."
            }
        }
    }
}


/// What the operator wants the calculated route to achieve, independent of session lifetime.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum RouteGoal {
    /// Seek a winning route with minimum MATCH/KING count under exact bounded search.
    #[default]
    ShortestRoute,
    /// Seek a win first, then useful tableau removal if a complete route is unavailable.
    MaximumCardMatches,
}


impl RouteGoal {
    /// Return the two requested outcome labels, without an algorithm-comparison menu.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ShortestRoute => "Shortest route",
            Self::MaximumCardMatches => "Maximum card matches",
        }
    }


    /// State match units and bounded-result limits in operator terms.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ShortestRoute => {
                "Seek a winning route with the fewest pair matches and single Kings. Draws/recycles are counted separately; a bounded search may not prove a minimum."
            }
            Self::MaximumCardMatches => {
                "Seek a win first; otherwise remove as many tableau cards as found possible. Stock-only matches add no tableau progress. A partial route stops at its verified endpoint."
            }
        }
    }


    /// Use exact search for a shortest-route request and the useful-route policy for fallback.
    pub(crate) fn method(self) -> RouteMethod {
        match self {
            Self::ShortestRoute => RouteMethod::FewestActions,
            Self::MaximumCardMatches => RouteMethod::Fast,
        }
    }
}


/// Independent Continuous lifetime; a challenge target never authorises normal repetition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum StopCondition {
    /// Follow verified normal three-board GAME WIN, Solver-free NewGame and repeat.
    RepeatNormalGames,
    /// Continue only the observed challenge and stop when its actual target is confirmed.
    #[default]
    ChallengeTarget,
    /// Repeat current-board work without classifying or evaluating challenge targets.
    NoChallenge,
}


impl StopCondition {
    /// Present a lifetime choice independently of the route objective.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RepeatNormalGames => "Repeat normal games",
            Self::ChallengeTarget => "Challenge",
            Self::NoChallenge => "Continuous - no challenge",
        }
    }


    /// Describe the actual terminal authority without implying a redeal policy.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::RepeatNormalGames => {
                "Repeat supported normal three-board games through the established GAME WIN and Solver-free NewGame sequence."
            }
            Self::ChallengeTarget => {
                "Follow the observed Score, Boards or rank/count requirement. Stop on confirmed target success; no NewGame after a challenge win."
            }
            Self::NoChallenge => {
                "Repeat Single board work, reusing current Recon and continuing through supported new deals; ignore challenge targets."
            }
        }
    }
}


/// Application search preference, independent of route execution authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum RouteMethod {
    /// Seek useful wins with logical-action costs before scored tableau-removal fallback.
    Fast,
    /// Search for minimum card-match cost until a result, exhaustion or operator STOP.
    #[default]
    FewestActions,
}


impl RouteMethod {
    /// Translate the previous idle preference into the corresponding operator outcome.
    pub(crate) fn route_goal(self) -> RouteGoal {
        match self {
            Self::Fast => RouteGoal::MaximumCardMatches,
            Self::FewestActions => RouteGoal::ShortestRoute,
        }
    }


    /// Return the method label used in controls, progress and backend reports.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Fast => "Weighted A* (weight 2)",
            Self::FewestActions => "Exact A* (weight 1)",
        }
    }


    /// Explain the route guarantee without confusing a bounded stop with no solution.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Fast => {
                "Seek a useful winning route with weighted A* (weight 2) and logical-action costs; minimum cost is unproved. Fall back to beneficial tableau removals when no complete route is found."
            }
            Self::FewestActions => {
                "Exact A* (weight 1) seeks the fewest winning card matches. A bounded stop is not proof of a minimum, and this outcome does not authorise a partial route."
            }
        }
    }
}


/// Worker-owned phase of one uninterrupted automatic session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionPhase {
    /// Check the supported scene and actual progress before choosing the next operation.
    Entry,
    /// Acquire complete identities and prove original-board restoration.
    Reconnaissance,
    /// Calculate and independently replay the current board's route without input.
    Planning,
    /// Execute the current single-use route with fresh effect checks.
    Execution,
    /// Establish the resulting board or whole-game completion from fresh observations.
    Transition,
    /// Finish shared XP/optional Level Up controls, then stop before NewGame.
    Completion,
    /// Use the guarded existing terminal controls to start the next supported game.
    Restart,
}


impl SessionPhase {
    /// Return a concise display label without interpreting logs or guest pixels.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Entry => "Checking entry",
            Self::Reconnaissance => "Recon",
            Self::Planning => "Plan route",
            Self::Execution => "Solve board",
            Self::Transition => "Checking completion",
            Self::Completion => "Finish GAME WIN",
            Self::Restart => "Starting next game",
        }
    }
}


/// One explicit automatic-start request; the worker validates and owns its lifetime.
#[derive(Clone, Debug)]
pub(crate) struct SessionRequest {
    /// Checked UI request identity, never reused across manual or automatic requests.
    pub(crate) run_id: u64,
    /// Automatic stopping/repetition policy; Manual is not an automatic start.
    pub(crate) mode: WorkflowMode,
    /// First board's literal acquisition settings and single-use start consent.
    pub(crate) reconnaissance: ReconRequest,
    /// Optional existing current-board export; the worker must authenticate its sole receipt.
    pub(crate) reuse_acquisition: Option<std::path::PathBuf>,
    /// First-board preference; later acquisitions apply their independently observed default.
    pub(crate) method: RouteMethod,
    /// First-board method is a default unless the operator explicitly chose an objective.
    pub(crate) use_challenge_default: bool,
    /// Intended route outcome fixed before acquisition and calculation.
    pub(crate) route_goal: RouteGoal,
    /// Continuous lifetime; separate from route benefit and numerical search method.
    pub(crate) stop_condition: StopCondition,
}


/// Advisory automatic progress carrying exact session and board identities.
#[derive(Clone, Debug)]
pub struct SessionProgress {
    /// Socket, strategy, game and generation that own this session.
    pub(crate) context: ControllerContext,
    /// Checked request identity shared by every phase of this session.
    pub(crate) run_id: u64,
    /// Checked board identity, advanced before each acquisition rather than inferred by UI.
    pub(crate) board_generation: u64,
    /// Current worker-owned phase; changing phase does not release the session.
    pub(crate) phase: SessionPhase,
    /// Worker-selected method for this board, including automatic fallback or reset.
    pub(crate) effective_method: RouteMethod,
    /// Advisory header from this board's existing pixels, independent of stopping policy.
    pub(crate) observed_game: ObservedGame,
    /// Concise backend report, without any authority inferred from its text.
    pub(crate) message: String,
}


/// The sole automatic-session event permitted to release its UI ownership.
#[derive(Clone, Debug)]
pub struct SessionCompletion {
    /// Exact controller identity under which the session ran.
    pub(crate) context: ControllerContext,
    /// Session identity that must still be outstanding in the UI.
    pub(crate) run_id: u64,
    /// Actual final report or original error; neither result permits resume.
    pub(crate) result: Result<String, String>,
}


/// Latest-only bounded search statistics; values describe work, not guest effects.
#[derive(Clone, Debug)]
pub(crate) struct PlanningProgress {
    /// Exact controller identity under which planning runs.
    pub(crate) context: ControllerContext,
    /// Manual calculation or automatic-session request identity.
    pub(crate) run_id: u64,
    /// Zero for manual calculation; checked positive generation for automatic boards.
    pub(crate) board_generation: u64,
    /// Search method fixed when this planning operation was dispatched.
    pub(crate) method: RouteMethod,
    /// Header facts authenticated with this calculation's exact restored acquisition.
    pub(crate) observed_game: ObservedGame,
    /// Actual elapsed time and cumulative search work from the shared search engine.
    pub(crate) stats: xsar::search::SearchStats,
    /// True for the search's final notification, before independent replay/reporting.
    pub(crate) finished: bool,
}
