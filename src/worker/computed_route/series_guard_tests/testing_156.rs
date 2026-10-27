//! Retained Score completion through the real execution observation adapter.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// A complete current bound event retires input before obscured cards are examined.
///
/// The bound requirement and observation phase are supplied fixture state; the
/// test neither asserts live action ACKs nor uses the running guest transport.
#[test]
fn native_156_score_terminal_retires_bound_input_before_card_checks() {
    let bytes = include_bytes!("../../../../tests/fixtures/pyramid-testing-156/score-complete.png");
    let frame = decode_png(bytes).unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Score, 1660, 1700).unwrap();
    let evidence = adapter(vec![frame], None, |io| {
        io.challenge = Some(bound);
        let EffectObservation::Challenge(proof) =
            io.observe_effect(SearchState::initial(), false).unwrap()
        else {
            panic!("complete bound Score event must retire ordinary route input");
        };
        assert_eq!(
            proof.progress(),
            ChallengeProgress::new(ChallengeKind::Score, 1755, 1700).unwrap()
        );
        assert!(proof.reward_counting() && !proof.operator_continue());
        assert!(
            io.begin_action(SearchState::initial(), &[InputOperation::PressDrawKey])
                .is_err()
        );
    });
    assert_eq!(evidence.captures, 1);
    assert!(evidence.first.is_none());
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
    let frame = decode_png(bytes).unwrap();
    let wrong = ChallengeProgress::new(ChallengeKind::Boards, 1, 3).unwrap();
    let evidence = adapter(vec![frame], None, |io| {
        io.challenge = Some(wrong);
        io.observe_effect(SearchState::initial(), false)
            .err()
            .expect("wrong challenge binding must refuse")
    });
    assert!(
        evidence
            .result
            .contains("Challenge Complete requirement differs from the bound challenge")
    );
    assert_eq!(evidence.captures, 1);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
}
