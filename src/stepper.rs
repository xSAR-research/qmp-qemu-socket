//! Single-action planning and shared post-action validation without input retries.

use std::time::Duration;

use thiserror::Error;

use crate::{
    game::{ActionTarget, GameMode, GuidedAction, InputOperation, RepeatTargetPolicy},
    geometry::PixelRect,
    parameters::AnimationSettleDelays,
    tracker::PredictedAction,
};


/// The single input selected from freshly captured Solver evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlannedInput(
    /// Canonical action definition selected from the fresh prediction.
    GuidedAction,
);


impl PlannedInput {


    /// Return the calibrated action carried by this single-input plan.
    pub const fn action(self) -> GuidedAction {
        self.0
    }


    /// Return the sole non-idempotent operation for this action.
    pub const fn operation(self) -> InputOperation {
        self.0.operation()
    }


    /// Select the session's immutable delay snapshot for this action type.
    pub const fn animation_settle_delay(self, delays: AnimationSettleDelays) -> Duration {
        self.0.animation_settle_delay(delays)
    }


    /// Region expected to change if the planned input reached Solitaire.
    pub const fn effect_bounds(self) -> PixelRect {
        self.0.effect_bounds()
    }


    /// Minimum material change required for this action's effect region.
    pub const fn minimum_changed_pixels(self) -> usize {
        self.0.minimum_changed_pixels()
    }


    /// Deliberate sleeps performed by QMP input delivery for this action.
    ///
    /// Command round trips and animation settling are profiled separately.
    pub fn intentional_input_wait(self) -> Duration {
        self.0.intentional_input_wait()
    }


    /// Region excluded from effect comparison because the guest cursor moves
    /// there. Draw leaves the pointer unchanged and therefore introduces no
    /// new cursor movement to exclude from its stock/waste comparison.
    pub const fn effect_exclusion_bounds(self) -> Option<PixelRect> {
        self.0.effect_exclusion_bounds()
    }


    /// Number of QMP commands emitted by the complete input sequence.
    ///
    /// A click is one absolute-movement command plus separate down and up
    /// commands. The draw qcode uses separate key-down and key-up commands.
    pub const fn qmp_command_count(self) -> usize {
        self.0.qmp_command_count()
    }


    /// Number of individual input events carried by those QMP commands.
    pub const fn qmp_event_count(self) -> usize {
        self.0.qmp_event_count()
    }
}


/// A one-action plan derived only from a fresh pre-action frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepPlan {
    /// Fresh prediction that authorised construction of this single-action plan.
    before: PredictedAction,
    /// Canonical action to deliver once and then observe.
    input: PlannedInput,
}


impl StepPlan {


    /// Return the fresh prediction from which this plan was constructed.
    pub const fn before(self) -> PredictedAction {
        self.before
    }


    /// Return the sole planned guest-input operation and its verification settings.
    pub const fn input(self) -> PlannedInput {
        self.input
    }
}


/// Reasons a prediction or observed result cannot be accepted as a guarded step.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum StepValidationError {
    /// Selected mode has no permission to emit guest input.
    #[error("{mode} is available for read-only calibration only; guest input is disabled")]
    CalibrationOnly {
        /// Mode whose profile permits only read-only calibration.
        mode: GameMode,
    },
    /// The planning capture contains no eligible Solver halo.
    #[error("pre-action validation found no Solver highlight")]
    NoPreActionHighlight,
    /// The planning capture does not select a unique authorised action.
    #[error("pre-action validation found {highlight_count} Solver highlights")]
    AmbiguousPreAction {
        /// Number of conflicting eligible highlights before input.
        highlight_count: usize,
    },
    /// The result capture does not select a unique next action.
    #[error("post-action validation found {highlight_count} Solver highlights")]
    AmbiguousPostAction {
        /// Number of conflicting eligible highlights after input.
        highlight_count: usize,
    },
    /// A target whose policy requires disappearance remains recommended.
    #[error("post-action still recommends {target}")]
    TargetStillHighlighted {
        /// Previously clicked target that has not cleared its highlight.
        target: ActionTarget,
    },
    /// Independent effect evidence is insufficient to prove the previous action.
    #[error("post-action effect region did not change materially; action result is uncertain")]
    NoObservedEffect,
    /// Supplied Pyramid action differs from its immutable calibrated definition.
    #[error("Pyramid target does not match its calibrated input action")]
    InvalidPyramidAction,
    /// Supplied Klondike source bounds or input differ from the canonical action.
    #[error("Klondike target does not match its bounded source action")]
    InvalidKlondikeAction,
    /// Supplied Free Cell source bounds or click differ from its canonical action.
    #[error("Free Cell target does not match its bounded source action")]
    InvalidFreeCellAction,
}


/// Build one action from exactly one fresh prediction.
pub fn plan_step(before: PredictedAction) -> Result<StepPlan, StepValidationError> {


    let input = match before {
        PredictedAction::CalibrationOnly { mode } => {
            return Err(StepValidationError::CalibrationOnly { mode });
        }
        PredictedAction::NoHighlight => {
            return Err(StepValidationError::NoPreActionHighlight);
        }
        PredictedAction::Action(action) => {


            if let ActionTarget::FreeCell(kind) = action.target
                && crate::freecell::canonical_action(kind) != Some(action)
            {
                return Err(StepValidationError::InvalidFreeCellAction);
            }


            if let ActionTarget::Klondike(kind) = action.target
                && crate::klondike::canonical_action(kind) != Some(action)
            {
                return Err(StepValidationError::InvalidKlondikeAction);
            }


            if let ActionTarget::Pyramid(kind) = action.target
                && crate::pyramid::action_for_kind(kind) != Some(action)
            {
                return Err(StepValidationError::InvalidPyramidAction);
            }
            PlannedInput(action)
        }
        PredictedAction::Ambiguous { highlight_count } => {
            return Err(StepValidationError::AmbiguousPreAction { highlight_count });
        }
    };

    Ok(StepPlan { before, input })
}


/// Require a fresh post-action prediction and independent effect evidence.
///
/// This verifies observation only. It never authorises a retry if the result
/// is ambiguous or lacks a material change in the action's ROI.
pub fn verify_post_action(
    plan: &StepPlan,
    after: PredictedAction,
    effect_changed: bool,
) -> Result<PredictedAction, StepValidationError> {


    if let PredictedAction::CalibrationOnly { mode } = after {
        return Err(StepValidationError::CalibrationOnly { mode });
    }


    if let PredictedAction::Ambiguous { highlight_count } = after {
        return Err(StepValidationError::AmbiguousPostAction { highlight_count });
    }
    let action = plan.input().action();


    if action.repeat_target_policy() == RepeatTargetPolicy::MustClear
        && matches!(after, PredictedAction::Action(after_action) if action.target == after_action.target)
    {
        return Err(StepValidationError::TargetStillHighlighted {
            target: action.target,
        });
    }


    if !effect_changed {
        return Err(StepValidationError::NoObservedEffect);
    }
    Ok(after)
}


#[cfg(test)]
mod tests {
    //! Planning authority, action geometry and no-retry validation regressions.

    use super::*;
    use crate::{
        game::GameMode,
        geometry::PixelPoint,
        parameters::{
            DRAW_EFFECT_BOUNDS, KEY_HOLD, MINIMUM_DRAW_CHANGED_PIXELS,
            MINIMUM_TABLEAU_CHANGED_PIXELS, MOUSE_HOLD, POINTER_SETTLE_DELAY, TABLEAU_CARD_REGIONS,
        },
    };


    /// A Klondike draw key is allowed only with its exact canonical specification.
    #[test]
    fn klondike_planning_rejects_forged_input_and_effect_bounds() {
        let action = crate::klondike::canonical_action(crate::klondike::KlondikeTarget::Draw)
            .expect("Draw has a canonical action");
        let plan = plan_step(PredictedAction::Action(action)).unwrap();
        assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
        assert_eq!(plan.input().qmp_command_count(), 2);

        let mut forged = action;
        forged.specification.operation = InputOperation::Click(PixelPoint::new(456, 199));
        assert_eq!(
            plan_step(PredictedAction::Action(forged)),
            Err(StepValidationError::InvalidKlondikeAction)
        );

        forged = action;
        forged.specification.effect_bounds = PixelRect::new(0, 0, 1_920, 1_080);
        assert_eq!(
            plan_step(PredictedAction::Action(forged)),
            Err(StepValidationError::InvalidKlondikeAction)
        );
    }


    /// Verify the Move/Recycle HALO permits D and rejects replacement mouse input.
    #[test]
    fn pyramid_move_plan_requires_the_canonical_draw_key() {
        use crate::pyramid::{PyramidTargetKind, action_for_kind};
        let action = action_for_kind(PyramidTargetKind::Move).unwrap();
        let plan = plan_step(PredictedAction::Action(action)).unwrap();
        assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
        assert_eq!(plan.input().qmp_command_count(), 2);
        assert_eq!(plan.input().qmp_event_count(), 2);
        assert_eq!(plan.input().intentional_input_wait(), KEY_HOLD);
        assert_eq!(
            plan.input()
                .animation_settle_delay(AnimationSettleDelays::default()),
            Duration::from_millis(1_000)
        );

        let mut forged = action;
        forged.specification.operation = InputOperation::Click(PixelPoint::new(960, 718));
        assert_eq!(
            plan_step(PredictedAction::Action(forged)),
            Err(StepValidationError::InvalidPyramidAction)
        );
        forged = action;
        forged.specification.operation = InputOperation::Click(PixelPoint::new(10, 10));
        assert_eq!(
            plan_step(PredictedAction::Action(forged)),
            Err(StepValidationError::InvalidPyramidAction)
        );
    }


    /// Keep lower-pile and tableau card targets restricted to their canonical click.
    #[test]
    fn pyramid_card_plans_reject_the_draw_key() {
        use crate::pyramid::{PYRAMID_TARGETS, PyramidTargetKind, action_for_kind};


        for slot in PYRAMID_TARGETS {


            if slot.kind == PyramidTargetKind::Move {
                continue;
            }
            let action = action_for_kind(slot.kind).unwrap();
            let plan = plan_step(PredictedAction::Action(action)).unwrap();
            assert_eq!(plan.input().operation(), InputOperation::Click(slot.click_point));
            assert_eq!(plan.input().qmp_command_count(), 3);
            assert_eq!(plan.input().qmp_event_count(), 4);

            let mut forged = action;
            forged.specification.operation = InputOperation::PressDrawKey;
            assert_eq!(
                plan_step(PredictedAction::Action(forged)),
                Err(StepValidationError::InvalidPyramidAction)
            );
        }
    }


    /// Build a test TriPeaks draw prediction using the calibrated bottom-action profile.
    fn draw(anchor_x: i32) -> PredictedAction {
        PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .bottom_action(0, PixelPoint::new(anchor_x, 870))
                .unwrap(),
        )
    }


    /// Build a bottom-row TriPeaks test prediction for a one-based column.
    fn tableau(column: u8) -> PredictedAction {
        let index = 18 + column as usize - 1;
        PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .tableau_action(index, PixelPoint::new(1_662, 630))
                .unwrap(),
        )
    }


    /// Verify draw input uses only the calibrated key sequence, timing and effect region.
    #[test]
    fn fresh_draw_uses_only_the_qcode_sequence() {
        let plan = plan_step(draw(842)).unwrap();

        assert_eq!(plan.before(), draw(842));
        assert_eq!(plan.input().operation(), InputOperation::PressDrawKey);
        assert_eq!(
            plan.input()
                .animation_settle_delay(AnimationSettleDelays::from_millis(123, 987, 2_000)),
            Duration::from_millis(123)
        );
        assert_eq!(plan.input().effect_bounds(), DRAW_EFFECT_BOUNDS);
        assert_eq!(
            plan.input().minimum_changed_pixels(),
            MINIMUM_DRAW_CHANGED_PIXELS
        );
        assert_eq!(plan.input().intentional_input_wait(), KEY_HOLD);
        assert_eq!(plan.input().effect_exclusion_bounds(), None);
        assert_eq!(plan.input().qmp_command_count(), 2);
        assert_eq!(plan.input().qmp_event_count(), 2);
    }


    /// Verify tableau input preserves the calibrated centre, cursor mask and timing.
    #[test]
    fn fresh_tableau_action_uses_only_the_canonical_card_centre() {
        let predicted = tableau(10);
        let plan = plan_step(predicted).unwrap();

        assert_eq!(
            plan.input().operation(),
            InputOperation::Click(PixelPoint::new(1_723, 533))
        );
        assert_eq!(
            plan.input()
                .animation_settle_delay(AnimationSettleDelays::from_millis(123, 987, 2_000)),
            Duration::from_millis(987)
        );
        assert_eq!(
            plan.input().effect_bounds(),
            TABLEAU_CARD_REGIONS[27].card_bounds
        );
        assert_eq!(
            plan.input().effect_exclusion_bounds(),
            Some(PixelRect::new(1_675, 485, 96, 96))
        );
        assert_eq!(plan.input().qmp_command_count(), 3);
        assert_eq!(plan.input().qmp_event_count(), 4);
        assert_eq!(
            plan.input().minimum_changed_pixels(),
            MINIMUM_TABLEAU_CHANGED_PIXELS
        );
        assert_eq!(
            plan.input().intentional_input_wait(),
            POINTER_SETTLE_DELAY + MOUSE_HOLD
        );
    }


    /// Verify planning rejects absent, ambiguous and calibration-only predictions.
    #[test]
    fn pre_action_validation_refuses_no_action_and_ambiguity() {
        assert_eq!(
            plan_step(PredictedAction::CalibrationOnly {
                mode: GameMode::Pyramid,
            }),
            Err(StepValidationError::CalibrationOnly {
                mode: GameMode::Pyramid,
            })
        );
        assert_eq!(
            plan_step(PredictedAction::NoHighlight),
            Err(StepValidationError::NoPreActionHighlight)
        );
        assert_eq!(
            plan_step(PredictedAction::Ambiguous { highlight_count: 2 }),
            Err(StepValidationError::AmbiguousPreAction { highlight_count: 2 })
        );
    }


    /// Verify the plan preserves the supplied fresh prediction unchanged.
    #[test]
    fn single_fresh_capture_is_the_only_planning_input() {
        assert_eq!(plan_step(draw(842)).unwrap().before(), draw(842));
    }


    /// Verify independently observed material effects accept the next fresh prediction.
    #[test]
    fn changed_fresh_post_action_is_verified() {
        let plan = plan_step(tableau(10)).unwrap();
        let after = draw(842);

        assert_eq!(verify_post_action(&plan, after, true), Ok(after));
        assert_eq!(
            verify_post_action(&plan, PredictedAction::NoHighlight, true),
            Ok(PredictedAction::NoHighlight)
        );
    }


    /// Verify missing effects and ambiguous results never authorise a retry.
    #[test]
    fn post_action_validation_never_turns_uncertainty_into_a_retry() {
        let before = draw(842);
        let plan = plan_step(before).unwrap();
        assert_eq!(plan.before(), before);

        assert_eq!(
            verify_post_action(&plan, before, false),
            Err(StepValidationError::NoObservedEffect)
        );
        assert_eq!(
            verify_post_action(&plan, before, true),
            Ok(before),
            "a second consecutive draw may legitimately retain the Draw prediction"
        );
        assert_eq!(
            verify_post_action(
                &plan,
                PredictedAction::Ambiguous { highlight_count: 3 },
                true,
            ),
            Err(StepValidationError::AmbiguousPostAction { highlight_count: 3 })
        );
        assert_eq!(
            verify_post_action(
                &plan,
                PredictedAction::CalibrationOnly {
                    mode: GameMode::Pyramid,
                },
                true,
            ),
            Err(StepValidationError::CalibrationOnly {
                mode: GameMode::Pyramid,
            })
        );
    }


    /// Verify a consumed TriPeaks tableau target cannot remain highlighted after success.
    #[test]
    fn tableau_target_must_not_remain_the_recommended_target() {
        let before = tableau(10);
        let plan = plan_step(before).unwrap();

        assert_eq!(
            verify_post_action(&plan, before, true),
            Err(StepValidationError::TargetStillHighlighted {
                target: GameMode::TriPeaks
                    .profile()
                    .tableau_target(27)
                    .map(ActionTarget::Tableau)
                    .unwrap(),
            })
        );
    }
}
