//! Final ordinary-position proof for an explicitly partial removal route.
//!
//! Only the exact independently replayed non-clear position permits a partial
//! result, which releases control without restart authority. A separately bound
//! Challenge Complete event returns its own completion type and never pretends
//! to verify a hidden partial position.

use super::*;


/// Observe a partial endpoint after the whole last action ACK, without further input.
///
/// Reuses the existing ACK-relative two-second window, 200 ms observation spacing
/// and independent capture cap. STOP/context, clock and ordinary position guards
/// remain fatal. A missing proof yields uncertainty, never a repeat operation.
pub(super) fn confirm_partial_action(
    state: SearchState,
    io: &mut impl RouteIo,
    actions: usize,
    gestures: usize,
    acknowledged_at: Duration,
    removed: u8,
) -> Result<DriveReport, String> {
    let remaining =
        u8::try_from(state.tableau_remaining()).map_err(|_| "Invalid partial tableau count")?;
    if state.is_goal()
        || removed == 0
        || usize::from(remaining) + usize::from(removed) != TABLEAU_LEN
    {
        return Err("Partial route endpoint differs from its removal objective".to_owned());
    }
    let deadline = acknowledged_at
        .checked_add(FINAL_CONFIRMATION_WINDOW)
        .ok_or("Partial confirmation deadline overflow")?;
    let mut previous = acknowledged_at;
    let mut observations = 0usize;
    loop {
        io.check()?;
        let now = io.elapsed();
        if now < previous {
            return Err("Partial confirmation clock moved backwards".to_owned());
        }
        previous = now;
        if now >= deadline {
            return Ok(DriveReport {
                message: format!(
                    "All {actions} partial-route actions delivered; {gestures} gestures acknowledged; exact endpoint unconfirmed after the two-second acknowledgement window. Inspect and reacquire; no continuation or retry."
                ),
                completion: BoardCompletion::DeliveredUnconfirmed,
            });
        }
        io.wait(FINAL_OBSERVATION_DELAY.min(deadline.saturating_sub(now)))?;
        io.check()?;
        let now = io.elapsed();
        if now <= previous {
            return Err("Partial confirmation clock did not advance after waiting".to_owned());
        }
        previous = now;
        if now >= deadline {
            continue;
        }
        if observations >= FINAL_OBSERVATIONS {
            io.wait(deadline.saturating_sub(now))?;
            continue;
        }
        observations += 1;
        let observation = io.observe_effect(state, false)?;
        io.check()?;
        let verified = match observation {
            EffectObservation::Position(verified) => verified,
            EffectObservation::ChallengePending(progress) => {
                io.progress(actions, actions);
                io.check()?;
                return challenge_pending_report(progress, actions, actions, gestures);
            }
            EffectObservation::Challenge(proof) => {
                return Ok(challenge_report(proof, actions, actions, gestures));
            }
        };
        if verified {
            io.progress(actions, actions);
            return Ok(DriveReport {
                message: format!(
                    "Partial route endpoint verified — {removed}/28 tableau cards removed, {remaining} remain; {actions} logical actions, {gestures} gestures acknowledged. Control released; no next-board or restart input."
                ),
                completion: BoardCompletion::PartialPositionVerified { removed, remaining },
            });
        }
    }
}
