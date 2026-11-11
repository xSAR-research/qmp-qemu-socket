//! Replayed complete/maximum-partial route sequencing, independent of Solver/HALO play.
//!
//! A delivery ACK never advances the route. Each accepted endpoint is observed
//! twice; an attempted or interrupted transfer retires its caller's live permit.
use xsar::games::freecell::{
    InitialDeal,
    rules::{Position, RecordedMove},
};


/// ABS diagnostic interval copied from an explicit operator request.
pub(crate) use xsar::qmp::DiagnosticDragTiming;


/// Frozen application policy for recorded automatic foundation transfers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AutoMode {
    /// Transfer the next record, then observe its consecutive recorded AUTO follow-ups.
    #[default]
    ObserveGame,
    /// Transfer each record explicitly, with separate semantic proof after each drag.
    ExecuteRecorded,
}


/// Pointer protocol selected for one explicit diagnostic SINGLE STEP invocation.
///
/// This selection never retries an uncertain transfer or changes normal route playback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DragMode {
    /// Position and drag using absolute pointer events.
    #[default]
    Absolute,
    /// Position absolutely, then make a bounded diagnostic motion probe in relative units.
    Relative,
}


/// Independently replayed offline endpoint; maximum quality is established by search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
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
pub(crate) enum Observation {
    /// Index of an exact physical endpoint in the supplied candidate vector.
    Position(usize),
    /// Acquired pixels do not prove a supplied physical endpoint.
    Unmatched,
    /// A live-bound challenge reached the supported common completion scene.
    ChallengeComplete,
}


/// Delivery outcome of one pre-input fresh acquisition and guarded transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransferOutcome {
    /// The drag was delivered; semantic effect remains unproved.
    Delivered,
    /// A bounded motion probe was delivered; no route effect is claimed.
    MovementProbe,
    /// Completion pre-empted the drag and retired live authority.
    ChallengeComplete,
}


/// Why an invocation ended successfully; only an incomplete verified step can resume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Completion {
    /// One explicit transfer and its required recorded AUTO follow-ups were proved.
    StepVerified,
    /// A bounded relative motion probe retired input without consuming any route edge.
    MovementProbe,
    /// The complete physical route ended with all 52 foundations.
    FullBoard,
    /// The exact replayed maximum-removal endpoint was physically proved.
    PartialRouteExhausted,
    /// Supported challenge completion retired input without advancing the route cursor.
    ChallengeComplete,
}


/// One contiguous physically replayed route endpoint supplied to observation.
#[derive(Clone, Copy)]
pub(crate) struct Candidate {
    /// Next unapplied recorded move at this endpoint.
    pub cursor: usize,
    /// Exact physical board, never compared through canonical Position equality.
    pub position: Position,
}


/// Immutable diagnostic context; it never grants input or changes the cursor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stage {
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
pub(crate) trait RouteIo {
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


    /// Publish authoritative accepted route progress.
    fn progress(&mut self, cursor: usize, total: usize, position: &Position);
}


/// Same-run accepted physical state; permit ownership lives in the worker store.
#[derive(Clone, Copy)]
pub(crate) struct State {
    /// Immutable physical endpoint last proved twice.
    pub position: Position,
    /// Next recorded transfer; never incremented on delivery alone.
    pub cursor: usize,
    /// True only after independently matching the original deal in this run.
    pub bound: bool,
}


impl State {
    /// Start with no live authority, even for an independently replayed offline plan.
    pub(crate) fn new(deal: &InitialDeal) -> Self {
        Self {
            position: Position::initial(deal),
            cursor: 0,
            bound: false,
        }
    }
}


/// Compare every physical sequence/slot; Position's canonical Eq is deliberately unsuitable.
pub(crate) fn same_physical(a: &Position, b: &Position) -> bool {
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
            // The adapter has already retired input authority. No settle, capture,
            // candidate acceptance or cursor increment follows this observation.
            Observation::ChallengeComplete => return Ok(None),
        };
        io.check()?;
        if let Some(index) = observed {
            let candidate = choices
                .get(index)
                .ok_or("Invalid observed route endpoint")?;
            if candidate.cursor < furthest {
                return Err("Irreversible Free Cell position regressed".into());
            }
            furthest = candidate.cursor;
            let endpoint = !require_closure || index + 1 == choices.len();
            if endpoint && previous == Some(index) {
                return Ok(Some(*candidate));
            }
        }
        previous = observed;
        io.settle()?;
    }
    Err("Free Cell physical position was not confirmed twice; no further input authorised".into())
}


/// Accept an observed endpoint and publish its physical cursor together.
fn accept(state: &mut State, candidate: Candidate, total: usize, io: &mut impl RouteIo) {
    state.position = candidate.position;
    state.cursor = candidate.cursor;
    io.progress(state.cursor, total, &state.position);
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


/// Run one bounded motion experiment without accepting route progress or another input.
///
/// Readiness may observe a safe automatic closure locally. Neither it nor delivery
/// updates the retained route cursor/position. The adapter retires its input permit
/// before acquiring read-only diagnostic pixels and returning MovementProbe.
pub(crate) fn movement_probe(
    state: &mut State,
    deal: &InitialDeal,
    moves: &[RecordedMove],
    mode: AutoMode,
    io: &mut impl RouteIo,
) -> Result<Completion, String> {
    let (stage, initial) = readiness(state, deal, moves, mode, io)?;
    let Some(initial) = initial else {
        return Ok(Completion::ChallengeComplete);
    };
    let action = moves
        .get(initial.cursor)
        .copied()
        .ok_or("No remaining Free Cell source for motion probe")?;
    io.stage(Stage {
        cursor: initial.cursor,
        action: Some(action),
        phase: "movement-probe",
        observation: 0,
        ..stage
    });
    io.check()?;
    match io.transfer(&initial.position, action)? {
        TransferOutcome::MovementProbe => Ok(Completion::MovementProbe),
        TransferOutcome::ChallengeComplete => Ok(Completion::ChallengeComplete),
        TransferOutcome::Delivered => {
            Err("Motion probe adapter returned ordinary route delivery".into())
        }
    }
}


/// Execute one explicit transfer plus recorded AUTO follow-ups, or the verified route.
///
/// The next unapplied record uses one guarded explicit-target held drag, including
/// when it is marked AUTO. Delivery ACK remains separate from two fresh physical
/// proofs of the child and required recorded AUTO follow-ups together. Any error
/// permanently ends the live invocation without retrying the transfer.
pub(crate) fn drive(
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
    accept(state, initial, moves.len(), io);
    while let Some(&action) = moves.get(state.cursor) {
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
        match io.transfer(&state.position, action)? {
            TransferOutcome::ChallengeComplete => return Ok(Completion::ChallengeComplete),
            TransferOutcome::MovementProbe => {
                return Err("Motion probe cannot be accepted as a route transfer".into());
            }
            TransferOutcome::Delivered => {}
        }
        io.settle()?;
        stage.phase = "effect";
        let Some(endpoint) = stable(io, &children, stage, true)? else {
            return Ok(Completion::ChallengeComplete);
        };
        accept(state, endpoint, moves.len(), io);
        if single_step {
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
