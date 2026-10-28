//! Native transition pairs with one nonrenewable late normal-board confirmation.
//!
//! Repeated PNGs are independently served mock captures, not retained live pairs.
//! The old-board identity and clear handoff are test inputs; no guest is contacted.

use super::*;


/// Original Request 2 sixth capture, independently accepted by native-v23.
const NEXT_BOARD: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-transition-157/next-board.png");


/// Invoke the actual input-free transition with a retired normal-board handoff.
fn transition(
    frames: Vec<Vec<u8>>,
    revocation: Revocation,
    mode: WorkflowMode,
    stop_condition: StopCondition,
) -> Episode {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let mut fixture = Fixture::new();
    fixture.request.reconnaissance.profile = ReconProfileSource::BundledNativeV23;
    fixture.request.mode = mode;
    fixture.request.stop_condition = stop_condition;
    let listener = UnixListener::bind(&fixture.context.socket_path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let directory = fixture.directory.0.clone();
    let context = fixture.context.clone();
    let control = fixture.events.latest_frame.reconnaissance.clone();
    let cancel = Arc::clone(&fixture.cancel);
    let server = thread::spawn(move || {
        serve(
            listener,
            directory,
            frames.into(),
            control,
            context,
            cancel,
            revocation,
        )
    });
    let mut adapter = fixture.adapter();
    adapter.original_tableau = Some(
        adapter
            .starting_board(&native(NORMAL_SERIES[0]))
            .unwrap()
            .tableau,
    );
    adapter.distinct_start = true;
    adapter.series = Some(BoardCount {
        completed: 0,
        target: 3,
    });
    let result = adapter.transition();
    let starting = adapter.starting.as_ref().map(|witness| witness.progress);
    let series = adapter.series;
    if result.is_err() {
        assert!(starting.is_none(), "unconfirmed board cannot seed Recon");
        assert_eq!(
            series,
            Some(BoardCount {
                completed: 0,
                target: 3
            })
        );
        assert!(adapter.distinct_start);
    }
    assert!(
        adapter.restart_entry.is_none(),
        "no third-clear receipt fabricated"
    );
    let retained_frame = adapter.latest.is_some();
    let wire = server.join().expect("input-free transition server");
    assert_eq!(
        wire.commands.first().map(String::as_str),
        Some("qmp_capabilities")
    );
    assert!(
        wire.commands
            .iter()
            .skip(1)
            .all(|command| command == "screendump")
    );
    assert!(!fixture.request.reconnaissance.output_path.exists());
    Episode {
        result,
        wire,
        retained_frame,
        starting,
        series,
        cancelled: fixture.cancel.load(Ordering::Acquire),
        owns_context: fixture
            .events
            .latest_frame
            .reconnaissance
            .check_active(&fixture.context, &fixture.cancel)
            .is_ok(),
    }
}


/// Place one new native observation after five explicitly unqualified mock frames.
fn late(frame: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = vec![PARTIAL.to_vec(); TRANSITION_OBSERVATIONS - 1];
    frames.push(frame.to_vec());
    frames
}


/// The retained sixth frame requires one matching fresh seventh before Recon authority.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_late_complete_next_normal_board_gets_one_confirmation() {
    let mut frames = late(NEXT_BOARD);
    frames.push(NEXT_BOARD.to_vec());
    let episode = transition(
        frames,
        Revocation::None,
        WorkflowMode::Continuous,
        StopCondition::RepeatNormalGames,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Board {
            progress: Some(BoardCount {
                completed: 1,
                target: 3
            }),
            standard_three: true,
        }
    );
    assert_eq!(
        episode.starting,
        Some(BoardCount {
            completed: 1,
            target: 3
        })
    );
    assert_eq!(episode.wire.captures, 7);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
}


/// An early pair returns immediately without spending the normal or reserved allowance.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_early_pair_does_not_request_a_reserve_capture() {
    let episode = transition(
        vec![NEXT_BOARD.to_vec(); 3],
        Revocation::None,
        WorkflowMode::Continuous,
        StopCondition::RepeatNormalGames,
    );
    assert!(episode.result.is_ok());
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 1);
}


/// Unknown, original and incorrectly advanced sixth frames cannot expand the budget.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_unqualified_sixth_observations_keep_the_six_capture_bound() {
    for sixth in [PARTIAL, NORMAL_SERIES[0], NORMAL_SERIES[2]] {
        let mut frames = late(sixth);
        frames.push(NEXT_BOARD.to_vec());
        let episode = transition(
            frames,
            Revocation::None,
            WorkflowMode::Continuous,
            StopCondition::RepeatNormalGames,
        );
        assert!(
            episode
                .result
                .unwrap_err()
                .contains("within transition budget")
        );
        assert_eq!(episode.wire.captures, 6);
        assert_eq!(episode.wire.remaining, 1);
    }
}


/// A missing or different seventh frame refuses; a queued eighth is never requested.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_confirmation_mismatch_cannot_renew_the_reserve() {
    for seventh in [PARTIAL, NORMAL_SERIES[2]] {
        let mut frames = late(NEXT_BOARD);
        frames.extend([seventh.to_vec(), NEXT_BOARD.to_vec()]);
        let episode = transition(
            frames,
            Revocation::None,
            WorkflowMode::Continuous,
            StopCondition::RepeatNormalGames,
        );
        assert!(
            episode
                .result
                .unwrap_err()
                .contains("within transition budget")
        );
        assert_eq!(episode.wire.captures, 7);
        assert_eq!(episode.wire.remaining, 1);
    }
}


/// STOP and latched context loss on frame six revoke the additional observation.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_reserve_never_survives_stop_or_context_revocation() {
    for revocation in [
        Revocation::StopAfterCapture(6),
        Revocation::ContextRoundTripAfterCapture(6),
    ] {
        let mut frames = late(NEXT_BOARD);
        frames.push(NEXT_BOARD.to_vec());
        let episode = transition(
            frames,
            revocation,
            WorkflowMode::Continuous,
            StopCondition::RepeatNormalGames,
        );
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.captures, 6);
        assert_eq!(episode.wire.remaining, 1);
        assert!(episode.cancelled && !episode.owns_context);
    }
}


/// The new allowance belongs only to Continuous Repeat normal games.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_157_other_session_policies_do_not_receive_the_normal_reserve() {
    for (mode, stop_condition) in [
        (WorkflowMode::SingleBoard, StopCondition::RepeatNormalGames),
        (WorkflowMode::Continuous, StopCondition::ChallengeTarget),
        (WorkflowMode::Continuous, StopCondition::NoChallenge),
    ] {
        let mut frames = late(NEXT_BOARD);
        frames.push(NEXT_BOARD.to_vec());
        let episode = transition(frames, Revocation::None, mode, stop_condition);
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.captures, 6);
        assert_eq!(episode.wire.remaining, 1);
    }
}
