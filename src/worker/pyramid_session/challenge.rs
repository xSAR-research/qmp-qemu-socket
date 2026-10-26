//! Challenge lifetime uses actual bound requirements and never normal NewGame.

use super::*;
use crate::pyramid_progress::ChallengeKind;


/// Require an actual incomplete challenge on an untouched starting board.
fn starting(entry: Entry) -> Result<ChallengeProgress, String> {
    match entry {
        Entry::Challenge(progress) if !progress.target_reached() => Ok(progress),
        _ => Err(format!(
            "Challenge target requires a positively recognised incomplete requirement and untouched board; observed {entry:?}; no acquisition input authorised"
        )),
    }
}


/// Compare actual terminal pixels with the session's immutable kind/rank/target.
fn completed(bound: ChallengeProgress, proof: ChallengeCompletion) -> Result<String, String> {
    let actual = proof.progress();
    if !actual.follows(bound) {
        return Err("Challenge terminal differs from the bound requirement; no continuation input authorised".to_owned());
    }
    let endpoint = if proof.operator_continue() {
        "Challenge Complete; Continue left to the operator"
    } else {
        "Challenge Complete; reward animation left to the operator with control released"
    };
    Ok(format!(
        "Challenge target verified: {}; {endpoint}.",
        proof.description()
    ))
}


/// Retain the actual completion receipt through input-free handoff, without a new board.
fn finish(
    io: &mut impl Pipeline,
    generation: u64,
    bound: ChallengeProgress,
    proof: ChallengeCompletion,
) -> Result<String, String> {
    completed(bound, proof)?;
    io.phase(generation, SessionPhase::Completion);
    io.check()?;
    let actual = io.finish_challenge(proof)?;
    io.check()?;
    completed(bound, actual)
}


/// Run supported challenge boards until their actual target is positively complete.
///
/// Partial endpoints stop. A full clear permits only a fresh input-free transition;
/// changed or missing requirements, unknown effects and STOP never permit input.
pub(super) fn drive(io: &mut impl Pipeline) -> Result<String, String> {
    io.check()?;
    io.phase(0, SessionPhase::Entry);
    io.check()?;
    let entry = io.entry(WorkflowMode::Continuous)?;
    io.check()?;
    if let Entry::ChallengeComplete(proof) = entry {
        return finish(io, 0, proof.progress(), proof);
    }
    let mut previous = starting(entry)?;
    let mut generation = 0_u64;
    loop {
        io.check()?;
        generation = generation
            .checked_add(1)
            .ok_or("Challenge board generation exhausted")?;
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
        let witnessed = io.challenge_progress().unwrap_or(previous);
        if !witnessed.follows(previous) {
            return Err("Execution challenge progress changed or regressed from its bound requirement; no continuation input authorised".to_owned());
        }
        match result {
            BoardCompletion::ChallengeTargetVerified(proof) => return finish(io, generation, witnessed, proof),
            BoardCompletion::ChallengeTargetReached(progress) => {
                if progress != witnessed || !progress.target_reached() {
                    return Err("Pending challenge terminal differs from its actual reached requirement; no continuation input authorised".to_owned());
                }
            }
            BoardCompletion::PartialPositionVerified { removed, remaining } => return Ok(format!("Challenge partial endpoint verified: {removed}/28 tableau cards removed, {remaining} remain. Last observed requirement is {witnessed:?}; control released. No positively authorised continuation remains; inspect and reacquire before another execution. No further input authorised.")),
            BoardCompletion::DeliveredUnconfirmed => return Err("Challenge final input acknowledged but effect unconfirmed; inspect and reacquire, no continuation or retry".to_owned()),
            BoardCompletion::BoardClear | BoardCompletion::NextBoardReady | BoardCompletion::GameComplete | BoardCompletion::StockRedealt => {}
            // Final whole-route ACK permits only transition observations. A fresh
            // distinct complete-board pair and the actual bound counter are still
            // required below; this result is never completion or input authority.
            BoardCompletion::AwaitingFreshBoard => {}
        }
        io.phase(generation, SessionPhase::Transition);
        io.check()?;
        let entry = io.transition()?;
        io.check()?;
        if let Entry::ChallengeComplete(proof) = entry {
            return finish(io, generation, witnessed, proof);
        }
        let next = starting(entry)?;
        if !next.follows(witnessed) {
            return Err(format!(
                "Challenge requirement changed or progress regressed from {witnessed:?} to {next:?}; no continuation input authorised"
            ));
        }
        if previous.kind == ChallengeKind::Boards
            && result != BoardCompletion::StockRedealt
            && previous.current.checked_add(1) != Some(next.current)
        {
            return Err(format!(
                "Observed board-clear progress {next:?} does not advance exactly once from {previous:?}; no continuation input authorised"
            ));
        }
        if previous.kind == ChallengeKind::Boards
            && result == BoardCompletion::StockRedealt
            && next.current != previous.current
        {
            return Err(
                "REDEAL changed Boards progress without a clear; no continuation".to_owned(),
            );
        }
        previous = next;
    }
}
