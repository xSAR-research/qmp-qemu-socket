//! Reader explanations must describe the supplied frame without admitting input.

use super::*;


/// Read unchanged native test evidence; duplicated mock observations are synthetic.
fn retained_underwater(second: bool) -> Vec<u8> {
    fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-158")
            .join(if second {
                "underwater-three-nine.png"
            } else {
                "underwater.png"
            }),
    )
    .unwrap()
}


/// The retained post-REDEAL board can confirm its literal bound Card progress.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_158_redeal_five_spades_board_confirms_bound_transition() {
    let bytes = fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-158/five-spades.png"),
    )
    .unwrap();
    let bound =
        ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Cards, 4, 10).unwrap();
    let episode = run(
        Operation::ColourBoundTransition(bound),
        vec![bytes; 6],
        Revocation::None,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Cards, 8, 10).unwrap()
        )
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 4);
    assert!(
        episode
            .wire
            .commands
            .iter()
            .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
    );
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// Two complete underwater observations bind entry; STOP/context still prevent input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_158_underwater_entry_pairs_complete_board_and_preserves_revocation() {
    let bytes = retained_underwater(false);
    let episode = run(
        Operation::ColourEntry,
        vec![bytes.clone(), bytes.clone()],
        Revocation::None,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Challenge(
            ChallengeProgress::new(crate::pyramid_progress::ChallengeKind::Boards, 0, 3).unwrap()
        )
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(
        episode
            .wire
            .commands
            .iter()
            .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
    );
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let stopped = run(
            Operation::ColourEntry,
            vec![bytes.clone(), bytes.clone()],
            revocation,
        );
        assert!(
            stopped
                .result
                .expect_err("revoked capture")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(stopped.wire.captures, 1);
        assert_eq!(stopped.wire.remaining, 1);
        assert!(stopped.retained_frame && stopped.cancelled && !stopped.owns_context);
    }
}


/// A labelled reference with occupied RIGHT is not a fresh untouched start.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_158_underwater_drawn_reference_cannot_authorise_entry() {
    let bytes = retained_underwater(true);
    let episode = run(
        Operation::ColourEntry,
        vec![bytes.clone(), bytes],
        Revocation::None,
    );
    let error = episode
        .result
        .expect_err("occupied RIGHT cannot supply fresh entry");
    assert!(error.contains("RIGHT"), "{error}");
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(
        episode
            .wire
            .commands
            .iter()
            .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
    );
    assert!(episode.starting.is_none());
}


/// Erase only a native rank in a labelled fixture; the derivative is synthetic.
fn missing_rank(origin: (u32, u32)) -> Vec<u8> {
    let mut frame = native(ZERO);
    for y in origin.1 + 4..origin.1 + 30 {
        for x in origin.0 + 3..origin.0 + 34 {
            let at = y as usize * frame.stride + x as usize * 4;
            frame
                .pixels
                .get_mut(at..at + 4)
                .unwrap()
                .copy_from_slice(&[238, 238, 238, 255]);
        }
    }
    encode_png(&frame).unwrap()
}


/// Entry retains each reader refusal and the unchanged two-capture/no-input bound.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_158_entry_reports_both_exact_frame_refusals() {
    let episode = run(
        Operation::ColourEntry,
        vec![missing_rank((890, 112)), missing_rank((790, 165))],
        Revocation::None,
    );
    let error = episode
        .result
        .expect_err("missing ranks cannot authorise a new run");
    assert!(error.contains("first capture: unknown or incomplete scene; native reader refusal: unknown rank/suit at tableau slot 0"), "{error}");
    assert!(error.contains("second capture: unknown or incomplete scene; native reader refusal: unknown rank/suit at tableau slot 1"), "{error}");
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    assert!(episode.starting.is_none());
}


/// Transition exhaustion reports the last frame, preserving six captures and no input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_158_transition_reports_latest_refusal_without_extending_budget() {
    let mut frames = vec![missing_rank((890, 112)); 5];
    frames.push(missing_rank((790, 165)));
    let episode = run(Operation::Transition, frames, Revocation::None);
    let error = episode
        .result
        .expect_err("unknown ranks cannot qualify a board pair");
    assert!(error.contains("last capture 6: unknown or incomplete scene; native reader refusal: unknown rank/suit at tableau slot 1"), "{error}");
    assert!(
        !error.contains("tableau slot 0"),
        "a previous frame must not masquerade as the final refusal"
    );
    assert_eq!(episode.wire.captures, 6);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    assert!(episode.starting.is_none());
}


/// A new matched observation cannot retain an earlier frame's reader refusal.
#[test]
fn native_158_matched_frame_has_no_stale_reader_refusal() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let missing = native(&missing_rank((890, 112)));
    let refused = adapter.observe_with_reason(&missing).unwrap();
    assert_eq!(refused.fact, NativeObservation::Unknown);
    assert!(refused.refusal.unwrap().contains("tableau slot 0"));
    let matched = adapter.observe_with_reason(&native(ZERO)).unwrap();
    assert!(matches!(matched.fact, NativeObservation::Challenge { .. }));
    assert!(matched.refusal.is_none());
    assert!(!matched.diagnostic_text().contains("reader refusal"));
}
