//! Replayed complete/maximum-partial route sequencing, independent of Solver/HALO play.
//!
//! A delivery ACK never advances the route. Each accepted endpoint is observed
//! twice; an attempted or interrupted transfer retires its caller's live permit.
use xsar::games::freecell::{
    InitialDeal,
    rules::{Position, RecordedMove},
};


/// Frozen application policy for recorded automatic foundation transfers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutoMode {
    /// Transfer the next record, then observe its consecutive recorded AUTO follow-ups.
    #[default]
    ObserveGame,
    /// Transfer each record explicitly, with separate semantic proof after each drag.
    ExecuteRecorded,
}


/// Independently replayed offline endpoint; maximum quality is established by search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Every original identity must reach its foundation.
    FullBoard,
    /// Exhaustive search proved this foundation count is the maximum reachable.
    Partial {
        /// Conserved foundation cards at the closed route endpoint, below 52.
        foundation_cards: u32,
    },
}


/// One fresh observation, with terminal evidence kept separate from route effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    /// Index of an exact physical endpoint in the supplied candidate vector.
    Position(usize),
    /// Acquired pixels do not prove a supplied physical endpoint.
    Unmatched,
}


/// Delivery outcome of one pre-input fresh acquisition and guarded transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferOutcome {
    /// The drag was delivered; semantic effect remains unproved.
    Delivered,
    /// Completion pre-empted the drag and retired live authority.
    ChallengeComplete,
}


/// Why an invocation ended successfully; only an incomplete verified step can resume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    /// One explicit transfer and its required recorded AUTO follow-ups were proved.
    StepVerified,
    /// The complete physical route ended with all 52 foundations.
    FullBoard,
    /// The exact replayed maximum-removal endpoint was physically proved.
    PartialRouteExhausted,
    /// Supported challenge completion retired input without advancing the route cursor.
    ChallengeComplete,
}


/// One contiguous physically replayed route endpoint supplied to observation.
#[derive(Clone, Copy)]
pub struct Candidate {
    /// Next unapplied recorded move at this endpoint.
    pub cursor: usize,
    /// Exact physical board, never compared through canonical Position equality.
    pub position: Position,
}


/// Immutable diagnostic context; it never grants input or changes the cursor.
#[derive(Clone, Copy, Debug)]
pub struct Stage {
    /// Zero-based next recorded transfer and complete route length.
    pub cursor: usize,
    /// Number of recorded transfers, including automatic records.
    pub total: usize,
    /// Frozen automatic-transfer policy.
    pub mode: AutoMode,
    /// Requested checked physical edge, when one is pending.
    pub action: Option<RecordedMove>,
    /// Binding, readiness, transfer, effect or completion.
    pub phase: &'static str,
    /// One-based observation number; zero outside a proof window.
    pub observation: usize,
}


/// Existing worker I/O with one fresh observation per call and no automatic input retry.
pub trait RouteIo {
    /// Check current owner, STOP, shutdown and cooperative pause.
    fn check(&mut self) -> Result<(), String>;


    /// Independently recognise all 52 original identities before retaining hidden history.
    fn bind(&mut self, deal: &InitialDeal) -> Result<(), String>;


    /// Capture once; return one exact candidate index, or unsupported/transient evidence.
    fn observe(&mut self, candidates: &[Candidate]) -> Result<Observation, String>;


    /// Deliver one held drag with both endpoints from one fresh physical parent proof.
    fn transfer(
        &mut self,
        parent: &Position,
        action: RecordedMove,
    ) -> Result<TransferOutcome, String>;


    /// Retain diagnostic context without extra I/O or sequencing effects.
    fn stage(&mut self, _stage: Stage) {}


    /// Existing interruptible settling policy; no input or extra acquisition here.
    fn settle(&mut self) -> Result<(), String>;


    /// Space the two eligible matching observations without extending transient retries.
    fn confirmation_wait(&mut self) -> Result<(), String> {
        self.settle()
    }


    /// Allow the delivered gesture to settle before acquiring its first effect frame.
    fn post_transfer_wait(&mut self) -> Result<(), String> {
        self.settle()
    }


    /// Read the exact invocation's request to stop after its current proved step.
    fn stop_after_step(&mut self) -> Result<bool, String> {
        Ok(false)
    }


    /// Allow recorded AUTO animation to finish while retaining STOP and owner checks.
    fn animation_wait(&mut self, duration: std::time::Duration) -> Result<(), String>;


    /// Publish authoritative accepted route progress.
    fn progress(&mut self, cursor: usize, total: usize, position: &Position);


    /// After board recognition fails, check challenge-mode Challenge Complete (5 frames / 4 x 1s waits).
    fn resolve_challenge_complete(&mut self) -> Result<bool, String> {
        Ok(false)
    }


    /// Poll normal-game win entry only after the final 52-foundation endpoint is obscured by GAME WIN.
    fn resolve_normal_game_win(&mut self, _goal: &Position) -> Result<bool, String> {
        Ok(false)
    }
}


/// Same-run accepted physical state; permit ownership lives in the worker store.
#[derive(Clone, Copy)]
pub struct State {
    /// Immutable physical endpoint last proved twice.
    pub position: Position,
    /// Next recorded transfer; never incremented on delivery alone.
    pub cursor: usize,
    /// True only after independently matching the original deal in this run.
    pub bound: bool,
}


impl State {
    /// Start with no live authority, even for an independently replayed offline plan.
    pub fn new(deal: &InitialDeal) -> Self {
        Self {
            position: Position::initial(deal),
            cursor: 0,
            bound: false,
        }
    }
}


/// Compare every physical sequence/slot; Position's canonical Eq is deliberately unsuitable.
pub fn same_physical(a: &Position, b: &Position) -> bool {
    (0..8).all(|n| a.column(n) == b.column(n))
        && a.cells() == b.cells()
        && a.foundations() == b.foundations()
}


/// Build only the current endpoint and its contiguous recorded automatic successors.
fn candidates(
    position: Position,
    cursor: usize,
    moves: &[RecordedMove],
) -> Result<Vec<Candidate>, String> {
    let mut result = vec![Candidate { cursor, position }];
    let mut current = position;
    for (index, action) in moves.iter().enumerate().skip(cursor) {
        if !action.is_auto {
            break;
        }
        current = current.apply(*action).map_err(|e| e.to_string())?;
        result.push(Candidate {
            cursor: index + 1,
            position: current,
        });
    }
    Ok(result)
}


/// Two successive fresh equal endpoints within the existing six-observation policy.
///
/// Readiness accepts any recorded prefix, including the unchanged parent. After a
/// transfer, `require_closure` requires every consecutive recorded AUTO follow-up.
fn stable(
    io: &mut impl RouteIo,
    choices: &[Candidate],
    stage: Stage,
    require_closure: bool,
) -> Result<Option<Candidate>, String> {
    let mut previous = None;
    let mut furthest = 0;
    for observation in 1..=6 {
        io.stage(Stage {
            observation,
            ..stage
        });
        io.check()?;
        let observed = match io.observe(choices)? {
            Observation::Position(index) => Some(index),
            Observation::Unmatched => None,
        };
        io.check()?;
        let mut eligible = false;
        if let Some(index) = observed {
            let candidate = choices
                .get(index)
                .ok_or("Invalid observed route endpoint")?;
            if candidate.cursor < furthest {
                return Err("Irreversible Free Cell position regressed".into());
            }
            furthest = candidate.cursor;
            eligible = !require_closure || index + 1 == choices.len();
            if eligible && previous == Some(index) {
                return Ok(Some(*candidate));
            }
        }
        previous = observed;
        if observation < 6 {
            if eligible {
                io.confirmation_wait()?;
            } else {
                io.settle()?;
            }
        }
    }
    if io.resolve_challenge_complete()? {
        return Ok(None);
    }
    Err("Free Cell physical position was not confirmed twice; no further input authorised".into())
}


/// Accept an observed endpoint and publish its physical cursor together.
fn accept(state: &mut State, candidate: Candidate, total: usize, io: &mut impl RouteIo) {
    state.position = candidate.position;
    state.cursor = candidate.cursor;
    io.progress(state.cursor, total, &state.position);
}


/// Wait once for each newly reached recorded AUTO edge before another input.
fn auto_animation(
    io: &mut impl RouteIo,
    moves: &[RecordedMove],
    previous_cursor: usize,
    cursor: usize,
) -> Result<(), String> {
    let reached = moves
        .get(previous_cursor..cursor)
        .ok_or("Invalid AUTO animation cursor range")?;
    for _ in reached.iter().filter(|action| action.is_auto) {
        io.check()?;
        io.animation_wait(std::time::Duration::from_millis(500))?;
        io.check()?;
    }
    Ok(())
}


/// Authenticate the original deal and require two fresh physical readiness observations.
fn readiness(
    state: &mut State,
    deal: &InitialDeal,
    moves: &[RecordedMove],
    mode: AutoMode,
    io: &mut impl RouteIo,
) -> Result<(Stage, Option<Candidate>), String> {
    crate::freecell_recon::objective::require_acquired(deal.metadata())?;
    let stage = Stage {
        cursor: state.cursor,
        total: moves.len(),
        mode,
        action: moves.get(state.cursor).copied(),
        phase: "binding",
        observation: 0,
    };
    io.stage(stage);
    io.check()?;
    if !state.bound {
        if state.cursor != 0 {
            return Err("Unbound route has a noninitial physical position".into());
        }
        io.bind(deal)?;
        state.bound = true;
    }
    let initial = match mode {
        AutoMode::ObserveGame => candidates(state.position, state.cursor, moves)?,
        AutoMode::ExecuteRecorded => vec![Candidate {
            cursor: state.cursor,
            position: state.position,
        }],
    };
    let stage = Stage {
        phase: "readiness",
        ..stage
    };
    Ok((stage, stable(io, &initial, stage, false)?))
}


/// Execute one explicit transfer plus recorded AUTO follow-ups, or the verified route.
///
/// The next unapplied record uses one guarded explicit-target held drag, including
/// when it is marked AUTO. Delivery ACK remains separate from two fresh physical
/// proofs of the child and required recorded AUTO follow-ups together. Any error
/// permanently ends the live invocation without retrying the transfer.
pub fn drive(
    state: &mut State,
    deal: &InitialDeal,
    moves: &[RecordedMove],
    target: Target,
    single_step: bool,
    mode: AutoMode,
    io: &mut impl RouteIo,
) -> Result<Completion, String> {
    let (mut stage, initial) = readiness(state, deal, moves, mode, io)?;
    let Some(initial) = initial else {
        return Ok(Completion::ChallengeComplete);
    };
    let previous_cursor = state.cursor;
    accept(state, initial, moves.len(), io);
    auto_animation(io, moves, previous_cursor, state.cursor)?;
    while let Some(&action) = moves.get(state.cursor) {
        if io.stop_after_step()? {
            break;
        }
        stage = Stage {
            cursor: state.cursor,
            action: Some(action),
            phase: "transfer",
            observation: 0,
            ..stage
        };
        io.stage(stage);
        let next_cursor = state.cursor.checked_add(1).ok_or("Route cursor overflow")?;
        let next = state.position.apply(action).map_err(|e| e.to_string())?;
        let children = match mode {
            AutoMode::ObserveGame => candidates(next, next_cursor, moves)?,
            AutoMode::ExecuteRecorded => vec![Candidate {
                cursor: next_cursor,
                position: next,
            }],
        };
        io.check()?;
        if io.stop_after_step()? {
            break;
        }
        match io.transfer(&state.position, action)? {
            TransferOutcome::ChallengeComplete => return Ok(Completion::ChallengeComplete),
            TransferOutcome::Delivered => {}
        }
        io.post_transfer_wait()?;
        stage.phase = "effect";
        let endpoint = match stable(io, &children, stage, true) {
            Ok(Some(endpoint)) => endpoint,
            Ok(None) => return Ok(Completion::ChallengeComplete),
            Err(error) => {
                if target == Target::FullBoard
                    && let Some(&final_goal) = children.last().filter(|candidate| {
                        candidate.cursor == moves.len() && candidate.position.is_goal()
                    })
                    && io.resolve_normal_game_win(&final_goal.position)?
                {
                    accept(state, final_goal, moves.len(), io);
                    break;
                }
                return Err(error);
            }
        };
        let previous_cursor = state.cursor;
        accept(state, endpoint, moves.len(), io);
        auto_animation(io, moves, previous_cursor, state.cursor)?;
        if single_step || io.stop_after_step()? {
            break;
        }
    }
    io.stage(Stage {
        cursor: state.cursor,
        action: None,
        phase: "final-check",
        observation: 0,
        ..stage
    });
    io.check()?;
    let completion = if state.cursor == moves.len() {
        match target {
            Target::FullBoard if state.position.is_goal() => Completion::FullBoard,
            Target::FullBoard => {
                return Err("Free Cell route ended before all 52 foundation cards".into());
            }
            Target::Partial { foundation_cards }
                if foundation_cards < 52
                    && state.position.foundation_cards() == foundation_cards =>
            {
                Completion::PartialRouteExhausted
            }
            Target::Partial { .. } => {
                return Err("Free Cell partial route ended at a different foundation count".into());
            }
        }
    } else {
        Completion::StepVerified
    };
    io.stage(Stage {
        cursor: state.cursor,
        action: None,
        phase: "complete",
        observation: 0,
        ..stage
    });
    Ok(completion)
}


#[cfg(test)]
mod tests;
