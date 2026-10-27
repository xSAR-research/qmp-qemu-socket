//! Original effect positions and additive diagnostics; all transport is an isolated test peer.

use super::*;


/// Reconstruct a legal eleven-action prefix consistent with the retained pixels and counts.
/// This is explicitly a diagnostic reconstruction, not the unsaved consumed solver trace.
fn retained_prefix(plan: &mut Prepared, exact: bool) -> SearchState {
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-effect-155/deal.json"),
    )
    .unwrap();
    plan.acquisition.deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    plan.acquisition.vision = ReconVision::bundled_native_v23().unwrap();
    plan.acquisition.challenge_progress = None;
    let mut actions = if exact {
        vec![
            RecordedAction::Match(Location::DL, location("7|6")),
            RecordedAction::Move,
            RecordedAction::Match(Location::DL, location("7|3")),
        ]
    } else {
        vec![
            RecordedAction::Move,
            RecordedAction::Match(Location::DL, location("7|3")),
            RecordedAction::Move,
        ]
    };
    actions.extend([
        RecordedAction::Move,
        RecordedAction::Move,
        RecordedAction::Move,
        RecordedAction::King(Location::DL),
        RecordedAction::Match(Location::DL, location("7|4")),
        RecordedAction::Match(location("6|3"), location("7|5")),
        RecordedAction::Move,
        RecordedAction::Match(Location::DL, location("6|4")),
    ]);
    plan.trace = replay::replay(&plan.acquisition.deal, &actions, false).unwrap();
    actions
        .into_iter()
        .fold(SearchState::initial(), |state, action| {
            rules::apply(&plan.acquisition.deal, state, action).unwrap()
        })
}


/// Capture-local errors reset after success and then replace without losing first pixels.
#[test]
#[ignore = "requires host AF_UNIX; qualification includes ignored tests"]
fn native_155_latest_mismatch_tracks_its_own_frame_and_preserves_first() {
    let good = synthetic_start();
    let mut first = good.clone();
    paint(&mut first, PixelRect::new(890, 112, 50, 54), [255; 3]);
    let mut last = good.clone();
    paint(&mut last, PixelRect::new(790, 165, 50, 54), [255; 3]);
    let evidence = adapter(vec![first.clone(), good, last.clone()], None, |io| {
        assert!(
            !io.observe_checked(SearchState::initial(), false, false)
                .unwrap()
        );
        assert!(io.first_mismatch.as_ref().unwrap().0.contains("slot 0"));
        let observed = io.latest_observation.as_ref().unwrap();
        assert_eq!(observed.capture, 1);
        assert!(observed.mismatch.as_ref().unwrap().contains("slot 0"));
        assert!(
            io.observe_checked(SearchState::initial(), false, false)
                .unwrap()
        );
        let observed = io.latest_observation.as_ref().unwrap();
        assert_eq!(observed.capture, 2);
        assert!(observed.mismatch.is_none());
        assert!(io.first_mismatch.as_ref().unwrap().0.contains("slot 0"));
        assert!(
            !io.observe_checked(SearchState::initial(), false, true)
                .unwrap()
        );
        let observed = io.latest_observation.as_ref().unwrap();
        assert_eq!(observed.capture, 3);
        assert_eq!(observed.phase, FrameGuardPhase::Effect);
        assert!(observed.mismatch.as_ref().unwrap().contains("slot 1"));
        let value = observed.value(&io.plan.acquisition.deal).unwrap();
        assert_eq!(value["capture"], 3);
        assert_eq!(value["phase"], "effect");
        assert_eq!(value["acknowledged_gestures"], 0);
    });
    assert_eq!(evidence.captures, 3);
    assert_eq!(evidence.counts.attempted, 0);
    assert_eq!(evidence.counts.acknowledged, 0);
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
    assert_eq!(evidence.first.unwrap().1.pixels, first.pixels);
    assert_eq!(evidence.latest.unwrap().pixels, last.pixels);
}


/// Both original final frames satisfy the actual adapter after a legal reconstructed prefix.
/// No route is delivered, resumed or searched and no guest input is attempted.
#[test]
#[ignore = "requires host AF_UNIX; qualification includes ignored tests"]
fn native_155_actual_adapter_accepts_both_retained_effect_positions_without_input() {
    for (name, exact) in [("153-last.png", false), ("154-last.png", true)] {
        let frame = decode_png(
            &fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/pyramid-effect-155")
                    .join(name),
            )
            .unwrap(),
        )
        .unwrap();
        let final_state = Cell::new(SearchState::initial());
        let evidence = adapter_with_setup(
            vec![frame],
            None,
            |plan| final_state.set(retained_prefix(plan, exact)),
            |io| {
                assert!(io.observe_checked(final_state.get(), false, true).unwrap());
                assert!(io.latest_observation.as_ref().unwrap().mismatch.is_none());
            },
        );
        assert_eq!(evidence.captures, 1);
        assert_eq!(evidence.counts.attempted, 0);
        assert_eq!(evidence.counts.acknowledged, 0);
        assert!(evidence.wire.events.is_empty());
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Pending metadata names fixed one-based sources and exact original identities/states.
#[test]
fn native_155_pending_action_metadata_is_bounded_one_based_and_diagnostic_only() {
    let context = ControllerContext::new(
        "/unused/effect155.sock".into(),
        GameMode::Pyramid,
        SolvingStrategy::ShortestPath,
        1,
    );
    let mut prepared = plan(&context, 7);
    retained_prefix(&mut prepared, true);
    let pending = diagnostics::pending_action(&prepared, 10).unwrap().unwrap();
    assert_eq!(pending["action_number"], 11);
    assert_eq!(pending["action"], "MATCH");
    assert_eq!(pending["cards"][0]["location"], "DL");
    assert_eq!(pending["cards"][0]["card"], "8 of clubs");
    assert_eq!(pending["cards"][1]["location"], "6|4");
    assert_eq!(pending["cards"][1]["card"], "5 of clubs");
    assert_eq!(pending["before"]["DL"], "8 of clubs");
    assert_eq!(pending["after"]["DL"], "K of diamonds");
    assert_eq!(pending["after"]["DR"], "4 of clubs");
    assert_eq!(pending["before"]["tableau"][18]["location"], "6|4");
    assert_eq!(pending["after"]["tableau"][18]["card"], Value::Null);
    assert_eq!(pending["authority"], "diagnostic_only_no_resume_or_replay");
    assert!(
        diagnostics::pending_action(&prepared, prepared.trace.len())
            .unwrap()
            .is_none()
    );
    assert!(diagnostics::pending_action(&prepared, prepared.trace.len() + 1).is_err());
}
