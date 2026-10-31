//! Synthetic native-state derivatives exercise actual capture and whole-action authority.
//!
//! CSV/rules supply identities independently. Generated ink is explicitly synthetic;
//! profiles, recognition tolerances and guest evidence remain unchanged.

use super::*;


/// Parse the complete unchanged original used by the independent accepted route.
fn original_deal() -> CompleteDeal {
    xsar::pyramid_json::read_from(
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../xsar/tests/fixtures/pyramid-planning/pyramid-recon-20261019-v7-01.json"),
        )
        .unwrap()
        .as_slice(),
    )
    .unwrap()
}


/// Render only rules-derived card removals and pile identities into a synthetic frame.
///
/// Scene/header pixels and remaining tableau cards retain the original bytes. Test
/// geometry, labels and complete glyph masks are checked and invalid fixtures panic.
fn synthetic_state(deal: &CompleteDeal, state: SearchState) -> CapturedFrame {
    let profile: Value = serde_json::from_slice(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v7/profile.json"
    ))
    .unwrap();
    let (tableau, left, right) = expected(deal, state).unwrap();
    let mut frame = synthetic_start();
    for (index, card) in tableau.iter().enumerate() {
        if card.is_none() {
            let (row, column) = xsar::games::pyramid::planning::tableau_coordinates(index).unwrap();
            let kind = crate::pyramid::PyramidTargetKind::Card {
                row: row.try_into().unwrap(),
                column: column.try_into().unwrap(),
            };
            paint(
                &mut frame,
                crate::pyramid::PYRAMID_TARGETS[kind.slot_index().unwrap()].bounds,
                [30, 95, 65],
            );
        }
    }
    for (slot, card, empty) in [
        (crate::pyramid::LEFT_TARGET, left, "left_empty"),
        (crate::pyramid::RIGHT_TARGET, right, "right_empty"),
    ] {
        if let Some(card) = card {
            paint(&mut frame, slot.bounds, [255; 3]);
            let red = matches!(
                card.suit(),
                xsar::games::cards::Suit::Diamonds | xsar::games::cards::Suit::Hearts
            );
            let origin = (slot.bounds.x, slot.bounds.y);
            synthetic_glyph(
                &mut frame,
                &profile["stock_reader"],
                "ranks",
                card.rank().as_str(),
                origin,
                red,
            );
            synthetic_glyph(
                &mut frame,
                &profile["stock_reader"],
                "suits",
                card.suit().as_str(),
                origin,
                red,
            );
        } else {
            paint(&mut frame, slot.bounds, [30, 95, 65]);
            empty_pile_patch(&mut frame, empty);
        }
    }
    frame
}


/// Paint a retained full glyph shape as labelled synthetic black/red ink on paper.
///
/// Card identities come from independent rules; invalid labels, masks, hex or bounds panic.
pub(super) fn synthetic_glyph(
    frame: &mut CapturedFrame,
    reader: &Value,
    bank: &str,
    label: &str,
    origin: (u32, u32),
    red: bool,
) {
    let roi = &reader[match bank {
        "ranks" => "rank_roi",
        "suits" => "suit_roi",
        _ => panic!("unsupported fixture glyph bank"),
    }];
    let number = |index: usize| u32::try_from(roi[index].as_u64().unwrap()).unwrap();
    let bounds = PixelRect::new(
        origin.0.checked_add(number(0)).unwrap(),
        origin.1.checked_add(number(1)).unwrap(),
        number(2),
        number(3),
    );
    let entry = reader[bank]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["label"].as_str() == Some(label))
        .unwrap();
    let variant = &entry["variants"][0];
    assert!(
        variant["mask"]
            .as_str()
            .unwrap()
            .bytes()
            .all(|byte| byte == b'1')
    );
    let encoded = variant["rgb"].as_str().unwrap().as_bytes();
    assert_eq!(encoded.len() % 2, 0);
    let mut rgb: Vec<u8> = encoded
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(rgb.len() % 3, 0);
    for pixel in rgb.as_chunks_mut::<3>().0 {
        let ink = pixel.iter().any(|channel| *channel < 128);
        pixel.copy_from_slice(if ink {
            if red { &[220, 0, 0] } else { &[0; 3] }
        } else {
            &[255; 3]
        });
    }
    patch(frame, bounds, &rgb);
}


/// Check two actions with two settled frames per removal and fresh per-gesture probes.
fn check_two_actions(actions: [RecordedAction; 2]) {
    check_two_actions_from(SearchState::initial(), actions);
}


/// Check cadence from a rules-derived position with independently verified source pixels.
fn check_two_actions_from(initial: SearchState, actions: [RecordedAction; 2]) {
    check_two_actions_with_live_settings(initial, actions, false);
}


/// Add card-specific synthetic quarter-body colour outside identity glyphs for positive cadence.
/// This supplies explicit test occurrence evidence; it is not a captured card appearance.
fn decorate_synthetic_stock(frame: &mut CapturedFrame, deal: &CompleteDeal, state: SearchState) {
    let (_, left, right) = expected(deal, state).unwrap();
    for (slot, card) in [
        (crate::pyramid::LEFT_TARGET, left),
        (crate::pyramid::RIGHT_TARGET, right),
    ] {
        if let Some(card) = card {
            let rank = [
                "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
            ]
            .iter()
            .position(|s| *s == card.rank().as_str())
            .unwrap();
            let suit = ["clubs", "diamonds", "hearts", "spades"]
                .iter()
                .position(|s| *s == card.suit().as_str())
                .unwrap();
            let shade = u8::try_from(((rank * 4 + suit) * 17) % 200).unwrap();
            paint(
                frame,
                PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 10, 20, 30),
                [shade; 3],
            );
        }
    }
}


/// Run identical native cadence with optional live edits between action boundaries.
fn check_two_actions_with_live_settings(
    initial: SearchState,
    actions: [RecordedAction; 2],
    live: bool,
) {
    let deal = original_deal();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let last = rules::apply(&deal, middle, actions[1]).unwrap();
    let position_frames: Vec<_> = [initial, middle, last]
        .into_iter()
        .map(|state| {
            let mut frame=synthetic_state(&deal,state);
            if matches!(actions,[RecordedAction::Move,RecordedAction::Move]) {
                decorate_synthetic_stock(&mut frame,&deal,state);
            }
            frame
        })
        .collect();
    let reader = ReconVision::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v7/profile.json"),
    )
    .unwrap();
    for (state, frame) in [initial, middle, last].into_iter().zip(&position_frames) {
        let (tableau, left, right) = expected(&deal, state).unwrap();
        reader
            .verify_position(frame, &tableau, left, right)
            .expect("synthetic frame independently matches the rules-derived full state");
    }
    let first = operations(actions[0]).unwrap();
    let second = operations(actions[1]).unwrap();
    let gestures = first.len() + second.len();
    let first_observations = 2;
    let second_observations = 2;
    let total = 1 + first_observations + second_observations;
    let mut frames = vec![position_frames[0].clone()];
    frames.extend(std::iter::repeat_n(
        position_frames[1].clone(),
        first_observations,
    ));
    frames.extend(std::iter::repeat_n(
        position_frames[2].clone(),
        second_observations,
    ));
    let evidence = adapter(frames, None, |io| {
        assert!(matches!(
            io.observe_source(initial).unwrap(),
            EffectObservation::Position(true)
        ));
        let first_settings=crate::parameters::PyramidEffectSettings::default();
        let next_settings=crate::parameters::PyramidEffectSettings::new(9,85).unwrap();
        let pending_edit=crate::parameters::PyramidEffectSettings::new(255,1000).unwrap();
        io.begin_transition(initial, middle, &first).unwrap();
        if live { io.events.latest_frame.set_effect_settings(pending_edit); }
        assert_eq!(io.effect_settings,first_settings);
        for &operation in &first {
            io.deliver(operation, initial).unwrap();
        }
        for observation in 0..first_observations {
            let observed=io.observe_effect(middle, false).unwrap();
            assert!(matches!(observed, EffectObservation::Position(ready) if ready == (observation + 1 == first_observations)),"{:?}",io.latest_observation.as_ref().and_then(|x|x.mismatch.as_ref()));
        }
        assert_eq!(io.captures, 1 + first_observations);
        assert!(matches!(
            io.observe_source(middle).unwrap(),
            EffectObservation::Position(true)
        ));
        assert_eq!(
            io.captures, 1 + first_observations,
            "previous effect is the next source; no recapture"
        );
        if live { io.events.latest_frame.set_effect_settings(next_settings); }
        io.begin_transition(middle, last, &second).unwrap();
        if live {
            assert_eq!(io.effect_settings,first_settings);
            io.events.latest_frame.set_effect_settings(pending_edit);
        }
        for &operation in &second {
            io.deliver(operation, middle).unwrap();
        }
        for observation in 0..second_observations {
            assert!(matches!(io.observe_effect(last, false).unwrap(), EffectObservation::Position(ready) if ready == (observation + 1 == second_observations)));
        }
        if live {
            assert_eq!(io.effect_settings,first_settings, "card actions do not freeze RGB settings");
            io.freeze_effect_settings("REDEAL fixture boundary");
            assert_eq!(io.effect_settings,pending_edit);
        }
    });
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (total, gestures, gestures)
    );
    assert!(evidence.first.is_none());
    assert_eq!(evidence.wire.unused_frames, 0);
    let boundaries: Vec<_> = evidence
        .wire
        .commands
        .iter()
        .enumerate()
        .filter_map(|(i, name)| (name == "screendump").then_some(i))
        .collect();
    assert_eq!(boundaries.len(), total);
    let input_intervals = [
        [boundaries[0], boundaries[1]],
        [
            boundaries[first_observations],
            boundaries[first_observations + 1],
        ],
    ];
    for (interval, operations) in input_intervals.into_iter().zip([first, second]) {
        let commands = &evidence.wire.commands[interval[0] + 1..interval[1]];
        assert_eq!(
            commands
                .iter()
                .filter(|command| *command == "query-status")
                .count(),
            operations.len()
        );
        assert_eq!(
            commands
                .iter()
                .filter(|command| *command == "query-mice")
                .count(),
            operations.len()
        );
        let events: usize = operations
            .iter()
            .map(|operation| {
                if matches!(operation, InputOperation::Click(_)) {
                    3
                } else {
                    2
                }
            })
            .sum();
        assert_eq!(
            commands
                .iter()
                .filter(|command| *command == "input-send-event")
                .count(),
            events
        );
    }
}


/// Two independent legal DRAWs share each boundary frame, preserving their distinct effects.
#[test]
fn native_cadence_move_move_uses_five_frames_for_two_actions() {
    let deal = original_deal();
    let mut source = SearchState::initial();
    // The positive cadence fixture includes explicitly synthetic quarter-body decoration.
    // Glyph-only paper remains covered by the separate conservative stock refusal.
    for _ in 0..15 {
        source = rules::apply(&deal, source, RecordedAction::Move).unwrap();
    }
    check_two_actions_from(source, [RecordedAction::Move, RecordedAction::Move]);
}


/// Stock-only MATCH and KING use actual native transition staging, not tableau history.
/// Synthetic rules-derived pixels verify unchanged refusal and exact capture/input counts.
#[test]
fn source_172_native_stock_only_match_and_king_require_two_identity_checks() {
    let deal = original_deal();
    for (draws, action, accepted) in [
        (3, RecordedAction::Match(Location::DL, Location::DR), true),
        (8, RecordedAction::King(Location::DL), true),
    ] {
        let mut before = SearchState::initial();
        for _ in 0..draws {
            before = rules::apply(&deal, before, RecordedAction::Move).unwrap();
        }
        let after = rules::apply(&deal, before, action).unwrap();
        assert_eq!(
            expected(&deal, before).unwrap().0,
            expected(&deal, after).unwrap().0
        );
        let mut source = synthetic_state(&deal, before);
        let mut result = synthetic_state(&deal, after);
        if accepted {
            decorate_synthetic_stock(&mut source,&deal,before);
            decorate_synthetic_stock(&mut result,&deal,after);
        }
        let gestures = operations(action).unwrap();
        let evidence = adapter(vec![source.clone(), source, result.clone(), result], None, |io| {
            assert!(matches!(
                io.observe_source(before).unwrap(),
                EffectObservation::Position(true)
            ));
            let frozen = crate::parameters::PyramidEffectSettings::new(26, 100).unwrap();
            io.events.latest_frame.set_effect_settings(frozen);
            io.begin_transition(before, after, &gestures).unwrap();
            io.events.latest_frame.set_effect_settings(
                crate::parameters::PyramidEffectSettings::new(255, 999).unwrap(),
            );
            assert_ne!(io.effect_settings, frozen);
            assert!(io.source_transition.is_some());
            for &operation in &gestures {
                io.deliver(operation, before).unwrap();
            }
            assert!(matches!(
                io.observe_effect(after, false).unwrap(),
                EffectObservation::Position(false)
            ));
            assert!(io.source_proof.is_none());
            assert!(io.source_transition.is_some());
            assert!(
                matches!(io.observe_effect(after, false).unwrap(), EffectObservation::Position(ready) if !ready),
                "{action:?}: {:?}",
                io.latest_observation
                    .as_ref()
                    .and_then(|o| o.mismatch.as_ref())
            );
            assert!(matches!(io.observe_effect(after,false).unwrap(),EffectObservation::Position(true)));
            assert!(io.source_proof.is_some());
        });
        assert_eq!(
            (
                evidence.captures,
                evidence.counts.attempted,
                evidence.counts.acknowledged
            ),
            (4, gestures.len(), gestures.len())
        );
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Both legal initial MATCH pairs use one source each and no capture between pair clicks.
#[test]
fn native_cadence_match_match_uses_five_frames_without_midpair_capture() {
    let actions = accepted_actions();
    assert!(matches!(actions[0], RecordedAction::Match(..)));
    assert!(matches!(actions[1], RecordedAction::Match(..)));
    check_two_actions([actions[0], actions[1]]);
}


/// The actual driver shares its effect/source boundary and confirms a legal partial endpoint.
///
/// Two retained oracle MATCH actions are independently replayed. Their synthetic
/// frames prove controller capture ordering, not maximum optimality or guest effects.
#[test]
fn native_cadence_driver_two_matches_has_five_frames_and_typed_partial_completion() {
    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let final_state = rules::apply(&deal, middle, actions[1]).unwrap();
    let frames = [initial, middle, middle, final_state, final_state]
        .into_iter()
        .map(|state| synthetic_state(&deal, state))
        .collect();
    let evidence = adapter_with_progress(
        frames,
        None,
        |plan| {
            plan.trace = replay::replay(&plan.acquisition.deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        2,
        |io| drive(io.plan, io).unwrap(),
    );
    assert_eq!(
        evidence.result.completion,
        BoardCompletion::PartialPositionVerified {
            removed: 4,
            remaining: 24
        }
    );
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (5, 4, 4)
    );
    assert!(evidence.first.is_none());
    assert_eq!(evidence.wire.unused_frames, 0);
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|name| *name == "screendump")
            .count(),
        5
    );
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|name| *name == "query-mice")
            .count(),
        4
    );
    assert_eq!(
        evidence
            .wire
            .events
            .iter()
            .filter_map(|event| event["data"]["down"].as_bool())
            .collect::<Vec<_>>(),
        [true, false, true, false, true, false, true, false]
    );
}


/// STOP after the first acknowledged pair click revokes the second without any capture or retry.
#[test]
fn native_cadence_stop_between_pair_clicks_revokes_remaining_permit() {
    let operations = operations(accepted_actions()[0]).unwrap();
    assert_eq!(operations.len(), 2);
    let evidence = adapter(vec![synthetic_start()], None, |io| {
        assert!(matches!(
            io.observe_source(SearchState::initial()).unwrap(),
            EffectObservation::Position(true)
        ));
        io.begin_action(SearchState::initial(), &operations)
            .unwrap();
        io.deliver(operations[0], SearchState::initial()).unwrap();
        io.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        assert!(io.deliver(operations[1], SearchState::initial()).is_err());
    });
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (1, 1, 1)
    );
    assert_eq!(
        evidence
            .wire
            .events
            .iter()
            .filter_map(|event| event["data"]["down"].as_bool())
            .collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|command| *command == "query-mice")
            .count(),
        1
    );
}


/// No source proof can survive a wait or authorise a different ordered operation.
#[test]
fn native_cadence_wait_or_wrong_operation_grants_no_input() {
    for wait in [false, true] {
        let evidence = adapter(vec![synthetic_start()], None, |io| {
            assert!(matches!(
                io.observe_source(SearchState::initial()).unwrap(),
                EffectObservation::Position(true)
            ));
            if wait {
                io.wait(Duration::ZERO).unwrap();
                assert!(
                    io.begin_action(SearchState::initial(), &[InputOperation::PressDrawKey])
                        .is_err()
                );
            } else {
                io.begin_action(SearchState::initial(), &[InputOperation::PressDrawKey])
                    .unwrap();
                assert!(
                    io.deliver(operation(location("7|6")).unwrap(), SearchState::initial())
                        .is_err()
                );
                assert!(
                    io.deliver(InputOperation::PressDrawKey, SearchState::initial())
                        .is_err(),
                    "uncertainty consumes the permit"
                );
            }
        });
        assert_eq!(
            (
                evidence.captures,
                evidence.counts.attempted,
                evidence.counts.acknowledged
            ),
            (1, 0, 0)
        );
        assert!(evidence.wire.events.is_empty());
    }
}


/// A real parked native source resumes with a new capture and no input from cached pixels.
#[test]
fn native_cadence_resume_reacquires_source_without_replaying_input() {
    let state = SearchState::initial();
    let evidence = adapter(vec![synthetic_start(), synthetic_start()], None, |io| {
        assert!(matches!(
            io.observe_source(state).unwrap(),
            EffectObservation::Position(true)
        ));
        assert_eq!(io.captures, 1);
        let gate = std::sync::Arc::clone(&io.events.latest_frame.pause);
        let _work = gate
            .begin(super::super::super::pyramid_pause::Phase::Gameplay)
            .unwrap();
        let epoch = gate.snapshot().unwrap().epoch;
        gate.request(epoch, true).unwrap();
        let waking_gate = std::sync::Arc::clone(&gate);
        let resumer = std::thread::spawn(move || {
            let started = std::time::Instant::now();
            while !waking_gate
                .snapshot()
                .is_some_and(|snapshot| snapshot.acknowledged)
            {
                if started.elapsed() >= Duration::from_secs(2) {
                    waking_gate.abort();
                    panic!("native worker never acknowledged the pause");
                }
                std::thread::yield_now();
            }
            waking_gate.request(epoch, false).unwrap();
        });
        assert!(io.pause_checkpoint().unwrap());
        resumer.join().unwrap();
        assert!(io.source_proof.is_none());
        assert!(io.action_permit.is_none());
        assert!(matches!(
            io.observe_source(state).unwrap(),
            EffectObservation::Position(true)
        ));
        assert_eq!(io.captures, 2, "the pre-pause source cannot be reused");
    });
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (2, 0, 0)
    );
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// Compose only an original counter/header crop onto a labelled synthetic state.
/// Other scene/card/pile pixels retain their independently replayed identities.
fn challenge_header(mut frame: CapturedFrame, source: &CapturedFrame) -> CapturedFrame {
    let bounds = PixelRect::new(814, 41, 296, 48);
    let view = source.as_view().unwrap();
    view.checked_bounds(bounds).unwrap();
    let mut rgb = Vec::new();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            rgb.extend_from_slice(&view.pixel_rgb(x, y).unwrap());
        }
    }
    patch(&mut frame, bounds, &rgb);
    frame
}


/// Bound Standard and Continuous gameplay use two settled after frames per MATCH.
/// Real moving-card header contamination cannot reclassify the objective or replay a pair.
#[test]
fn native_cadence_bound_challenge_keeps_five_captures_and_four_acknowledged_gestures() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress, observe_challenge};
    let moving = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-challenge-motion/last-observed.png"
    ))
    .unwrap();
    let settled = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"
    ))
    .unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap();
    assert_eq!(observe_challenge(&moving).unwrap(), None);
    assert_eq!(observe_challenge(&settled).unwrap(), Some(bound));
    for policy in [ExecutionPolicy::Standard, ExecutionPolicy::Continuous] {
        let deal = original_deal();
        let actions = accepted_actions();
        let initial = SearchState::initial();
        let middle = rules::apply(&deal, initial, actions[0]).unwrap();
        let end = rules::apply(&deal, middle, actions[1]).unwrap();
        let evidence = adapter_with_progress(
            vec![
                challenge_header(synthetic_state(&deal, initial), &settled),
                challenge_header(synthetic_state(&deal, middle), &moving),
                challenge_header(synthetic_state(&deal, middle), &moving),
                challenge_header(synthetic_state(&deal, end), &moving),
                challenge_header(synthetic_state(&deal, end), &moving),
            ],
            None,
            |plan| {
                plan.acquisition.challenge_progress = Some(bound);
                plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
                plan.objective = RouteObjective::Removal {
                    removed: 4,
                    proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
                };
            },
            2,
            |io| {
                io.policy = policy;
                let report = drive(io.plan, io).unwrap();
                assert_eq!(io.challenge, Some(bound));
                report
            },
        );
        assert!(matches!(
            evidence.result.completion,
            BoardCompletion::PartialPositionVerified {
                removed: 4,
                remaining: 24
            }
        ));
        assert_eq!(
            (
                evidence.captures,
                evidence.counts.attempted,
                evidence.counts.acknowledged
            ),
            (5, 4, 4)
        );
        assert_eq!(evidence.wire.unused_frames, 0);
        assert_eq!(
            evidence
                .wire
                .commands
                .iter()
                .filter(|command| *command == "query-mice")
                .count(),
            4
        );
    }
}


/// Preserve an independent guard refusal after one whole pair with an unreadable label.
/// The challenge latch must never bypass the separate same-frame guard.
#[test]
fn native_cadence_pending_header_preserves_independent_guard_refusal() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress, observe_challenge};

    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let settled = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"
    ))
    .unwrap();
    let moving = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-challenge-guard/last-observed.png"
    ))
    .unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap();
    let guard = |frame: &CapturedFrame, phase| {
        if phase == FrameGuardPhase::Effect {
            assert_eq!(observe_challenge(frame).unwrap(), None);
            return Err("sentinel independent effect guard refusal".to_owned());
        }
        Ok(FrameGuardOutcome::Ready)
    };
    let evidence = adapter_with_progress(
        vec![
            challenge_header(synthetic_state(&deal, initial), &settled),
            challenge_header(synthetic_state(&deal, middle), &moving),
        ],
        Some(&guard),
        |plan| {
            plan.acquisition.challenge_progress = Some(bound);
            plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        0,
        |io| {
            io.policy = ExecutionPolicy::Continuous;
            let error = drive(io.plan, io).unwrap_err();
            assert!(io.source_proof.is_none());
            assert!(io.action_permit.is_none());
            error
        },
    );
    assert!(
        evidence
            .result
            .contains("sentinel independent effect guard refusal"),
        "{}",
        evidence.result
    );
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (2, 2, 2)
    );
    assert_eq!(evidence.wire.unused_frames, 0);
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|command| *command == "query-mice")
            .count(),
        2
    );
}


/// Once qualified, even a readable different action-frame label cannot replace the binding.
/// Unbound entry remains strict when a challenge appears after a headerless acquisition.
#[test]
fn native_cadence_action_headers_do_not_rebind_but_unbound_entry_stays_strict() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};
    let moving = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-challenge-motion/last-observed.png"
    ))
    .unwrap();
    let other = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"
    ))
    .unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Cards, 8, 10).unwrap();
    for header in [&moving, &other] {
        for effect in [false, true] {
            let frame = challenge_header(
                synthetic_state(&original_deal(), SearchState::initial()),
                header,
            );
            let evidence = adapter_with_setup(
                vec![frame],
                None,
                |plan| plan.acquisition.challenge_progress = Some(bound),
                |io| {
                    io.challenge = Some(bound);
                    io.challenge_initial = Some(bound);
                    let observation = if effect {
                        io.observe_effect(SearchState::initial(), false)
                    } else {
                        io.observe_source(SearchState::initial())
                    };
                    assert!(matches!(
                        observation.unwrap(),
                        EffectObservation::Position(true)
                    ));
                    assert_eq!(io.challenge, Some(bound));
                },
            );
            assert_eq!(
                (evidence.counts.attempted, evidence.counts.acknowledged),
                (0, 0)
            );
            assert!(evidence.wire.events.is_empty());
        }
    }
    let frame = challenge_header(
        synthetic_state(&original_deal(), SearchState::initial()),
        &other,
    );
    let evidence = adapter_with_setup(
        vec![frame],
        None,
        |plan| plan.acquisition.challenge_progress = None,
        |io| {
            io.challenge = None;
            io.challenge_initial = None;
            assert!(
                io.observe_source(SearchState::initial())
                    .err()
                    .unwrap()
                    .contains("restored Recon binding")
            );
            assert!(io.action_permit.is_none());
        },
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
}


/// The real small modal stops an actual native driver mid-route after a whole MATCH.
/// Only the source header/state composition is synthetic; the terminal is unchanged.
#[test]
fn native_141_small_modal_retires_native_driver_without_second_action_or_skip() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress, observe_challenge};
    let terminal = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-native-141/jacks-small-complete.png"
    ))
    .unwrap();
    let header = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-native-141/jacks-before.png"
    ))
    .unwrap();
    let bound = observe_challenge(&header).unwrap().unwrap();
    assert_eq!(
        bound,
        ChallengeProgress::new(ChallengeKind::Cards, 8, 10).unwrap()
    );
    let deal = original_deal();
    let actions = accepted_actions();
    let evidence = adapter_with_progress(
        vec![
            challenge_header(synthetic_state(&deal, SearchState::initial()), &header),
            terminal,
        ],
        None,
        |plan| {
            plan.acquisition.challenge_progress = Some(bound);
            plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        0,
        |io| {
            io.policy = ExecutionPolicy::Continuous;
            drive(io.plan, io).unwrap()
        },
    );
    assert!(matches!(
        evidence.result.completion,
        BoardCompletion::ChallengeTargetVerified(_)
    ));
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (2, 2, 2)
    );
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// A contradictory whole literal on the small modal stops the actual native driver.
/// The calibrated header composition is synthetic; no second MATCH or skip is sent.
#[test]
fn native_142_small_modal_literal_conflict_stops_native_driver_without_replay() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress, observe_challenge};
    let mut terminal = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-native-141/jacks-small-complete.png"
    ))
    .unwrap();
    patch(
        &mut terminal,
        PixelRect::new(814, 48, 296, 32),
        include_bytes!(
            "../../../../tests/fixtures/pyramid-objectives/complete-boards-two-header.rgb"
        ),
    );
    let header = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-native-141/jacks-before.png"
    ))
    .unwrap();
    let bound = observe_challenge(&header).unwrap().unwrap();
    assert_eq!(
        bound,
        ChallengeProgress::new(ChallengeKind::Cards, 8, 10).unwrap()
    );
    let deal = original_deal();
    let actions = accepted_actions();
    let evidence = adapter_with_progress(
        vec![
            challenge_header(synthetic_state(&deal, SearchState::initial()), &header),
            terminal,
        ],
        None,
        |plan| {
            plan.acquisition.challenge_progress = Some(bound);
            plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        0,
        |io| {
            io.policy = ExecutionPolicy::Continuous;
            drive(io.plan, io).unwrap_err()
        },
    );
    assert!(evidence.result.contains(
        "Challenge Complete requirement differs from the bound challenge; no further input"
    ));
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (2, 2, 2)
    );
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// Actual 149 pixels retain strict source refusal while the bound effect may await settlement.
#[test]
fn native_150_normal_footer_actual_guard() {
    use crate::worker::pyramid_session::guard_normal_progress;
    let first = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-normal-footer-150/first-unmatched.png"
    ))
    .unwrap();
    let last = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-normal-footer-150/last-observed.png"
    ))
    .unwrap();
    let expected = BoardCount {
        completed: 1,
        target: 3,
    };
    assert_eq!(observe_normal(&first).unwrap(), Some(expected));
    assert_eq!(observe_normal(&last).unwrap(), None);
    assert_eq!(
        crate::pyramid_progress::observe_normal_header(&last).unwrap(),
        Some(expected)
    );
    assert_eq!(
        guard_normal_progress(&first, FrameGuardPhase::Source, expected).unwrap(),
        FrameGuardOutcome::Ready
    );
    assert!(guard_normal_progress(&last, FrameGuardPhase::Source, expected).is_err());
    assert_eq!(
        guard_normal_progress(&last, FrameGuardPhase::Effect, expected).unwrap(),
        FrameGuardOutcome::Pending
    );
    let mut contradictory = last.clone();
    patch(&mut contradictory, HEADER, NORMAL_HEADER);
    assert!(guard_normal_progress(&contradictory, FrameGuardPhase::Effect, expected).is_err());
    paint(&mut contradictory, HEADER, [255; 3]);
    assert_eq!(
        guard_normal_progress(&contradictory, FrameGuardPhase::Effect, expected).unwrap(),
        FrameGuardOutcome::Pending
    );
    assert!(guard_normal_progress(&contradictory, FrameGuardPhase::Source, expected).is_err());
    contradictory.stride = 1;
    assert!(guard_normal_progress(&contradictory, FrameGuardPhase::Effect, expected).is_err());
}


/// Compose only the measured occluded footer onto a labelled independent-route state.
fn native_150_occluded_footer(mut frame: CapturedFrame) -> CapturedFrame {
    let actual = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-normal-footer-150/last-observed.png"
    ))
    .unwrap();
    let view = actual.as_view().unwrap();
    let mut rgb = Vec::new();
    for y in FOOTER.y..FOOTER.bottom() {
        for x in FOOTER.x..FOOTER.right() {
            rgb.extend_from_slice(&view.pixel_rgb(x, y).unwrap());
        }
    }
    patch(&mut frame, FOOTER, &rgb);
    assert_eq!(observe_normal(&frame).unwrap(), None);
    frame
}


/// The production driver waits read-only, commits the settled effect and reuses its source once.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_150_normal_footer_pending_settles_without_replay() {
    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let end = rules::apply(&deal, middle, actions[1]).unwrap();
    let guard = |frame: &CapturedFrame, phase| {
        crate::worker::pyramid_session::guard_normal_progress(
            frame,
            phase,
            BoardCount {
                completed: 0,
                target: 3,
            },
        )
    };
    let evidence = adapter_with_progress(
        vec![
            synthetic_state(&deal, initial),
            native_150_occluded_footer(synthetic_state(&deal, middle)),
            synthetic_state(&deal, middle),
            synthetic_state(&deal, middle),
            synthetic_state(&deal, end),
            synthetic_state(&deal, end),
        ],
        Some(&guard),
        |plan| {
            plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        2,
        |io| drive(io.plan, io).unwrap(),
    );
    assert!(matches!(
        evidence.result.completion,
        BoardCompletion::PartialPositionVerified {
            removed: 4,
            remaining: 24
        }
    ));
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (6, 4, 4)
    );
    assert_eq!(evidence.wire.unused_frames, 0);
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|command| *command == "query-mice")
            .count(),
        4
    );
}


/// Even an invalid caller returning Pending for a source cannot mint input authority.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_150_pending_source_never_authorises_input() {
    let guard = |_: &CapturedFrame, _| Ok(FrameGuardOutcome::Pending);
    let evidence = adapter(vec![synthetic_start()], Some(&guard), |io| {
        let result = io.observe_checked(SearchState::initial(), false, false);
        assert!(io.source_proof.is_none());
        assert!(io.action_permit.is_none());
        assert!(!io.position_matches);
        result
    });
    assert!(
        evidence
            .result
            .unwrap_err()
            .contains("Source frame guard is pending")
    );
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (1, 0, 0)
    );
}



/// Native policy admits only qualified normal boards one/two and never stock continuation.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_150_normal_read_only_handoff_excludes_third_and_manual() {
    let evidence = adapter(vec![], Some(&require_normal_one), |io| {
        for (completed, target, allowed) in
            [(0, 3, true), (1, 3, true), (2, 3, false), (0, 2, false)]
        {
            io.policy = ExecutionPolicy::NormalSeries(BoardCount { completed, target });
            assert_eq!(io.read_only_board_transition(), allowed);
            assert!(!io.policy.continues_stock());
            assert!(!io.handoff_after_final_ack());
        }
        io.policy = ExecutionPolicy::Standard;
        assert!(!io.read_only_board_transition());
        io.policy = ExecutionPolicy::NormalSeries(BoardCount {
            completed: 0,
            target: 3,
        });
        io.frame_guard = None;
        assert!(!io.read_only_board_transition());
    });
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (0, 0, 0)
    );
}



/// A perpetually occluded normal effect expires at the existing deadline after one action only.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_150_normal_footer_timeout_never_dispatches_next_action() {
    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let pending = native_150_occluded_footer(synthetic_state(&deal, middle));
    let limit = ordinary_effect_observation_limit().unwrap();
    let mut frames = vec![synthetic_state(&deal, initial)];
    frames.extend(std::iter::repeat_n(pending, limit));
    let guard = |frame: &CapturedFrame, phase| {
        crate::worker::pyramid_session::guard_normal_progress(
            frame,
            phase,
            BoardCount {
                completed: 0,
                target: 3,
            },
        )
    };
    let evidence = adapter_with_progress(
        frames,
        Some(&guard),
        |plan| {
            plan.trace = replay::replay(&deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        0,
        |io| {
            let result = drive(io.plan, io);
            assert!(io.source_proof.is_none() && io.action_permit.is_none());
            result
        },
    );
    assert!(
        evidence
            .result
            .unwrap_err()
            .contains("Ordinary effect observation window expired")
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (2, 2)
    );
    assert!(evidence.captures >= 2 && evidence.captures <= limit + 1);
}


/// STOP and a context round trip revoke a pending native effect before any later observation/input.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_150_normal_footer_pending_stop_and_context_revoke() {
    let deal = original_deal();
    let middle = rules::apply(&deal, SearchState::initial(), accepted_actions()[0]).unwrap();
    let guard = |frame: &CapturedFrame, phase| {
        crate::worker::pyramid_session::guard_normal_progress(
            frame,
            phase,
            BoardCount {
                completed: 0,
                target: 3,
            },
        )
    };
    for stop in [true, false] {
        let evidence = adapter(
            vec![native_150_occluded_footer(synthetic_state(&deal, middle))],
            Some(&guard),
            |io| {
                assert!(!io.observe_checked(middle, false, true).unwrap());
                assert!(
                    io.source_proof.is_none() && io.action_permit.is_none() && !io.position_matches
                );
                if stop {
                    io.cancel.store(true, std::sync::atomic::Ordering::Release);
                } else {
                    let control = &io.events.latest_frame.reconnaissance;
                    let other = ControllerContext::new(
                        io.context.socket_path.clone(),
                        GameMode::Pyramid,
                        SolvingStrategy::ShortestPath,
                        4,
                    );
                    control.select(&other, io.cancel).unwrap();
                    control.select(io.context, io.cancel).unwrap();
                }
                assert!(io.observe_checked(middle, false, true).is_err());
                assert!(io.source_proof.is_none() && io.action_permit.is_none());
            },
        );
        assert_eq!(
            (
                evidence.captures,
                evidence.counts.attempted,
                evidence.counts.acknowledged
            ),
            (1, 0, 0)
        );
    }
}


/// In-flight edits do not reinterpret either action; a subsequent boundary copies both fields.
#[test]
fn live_168_native_two_actions_and_redeal_boundary_freeze_complete_pairs() {
    let actions = accepted_actions();
    check_two_actions_with_live_settings(SearchState::initial(), [actions[0], actions[1]], true);
}


/// A previous accepted effect can hide the next source; only fresh same-frame readiness permits input.
#[test]
fn readiness_173_occluded_boundary_recovers_before_the_next_pair() {
    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let middle = rules::apply(&deal, initial, actions[0]).unwrap();
    let last = rules::apply(&deal, middle, actions[1]).unwrap();
    let source = synthetic_state(&deal, initial);
    let middle_frame = synthetic_state(&deal, middle);
    let final_frame = synthetic_state(&deal, last);
    let operation = operations(actions[1]).unwrap()[0];
    let InputOperation::Click(point) = operation else {
        panic!("expected MATCH");
    };
    let slot = crate::pyramid::PYRAMID_TARGETS
        .iter()
        .find(|target| target.click_point == point)
        .unwrap();
    let mut obscured = middle_frame.clone();
    paint(
        &mut obscured,
        PixelRect::new(slot.bounds.x + 3, slot.bounds.y + 4, 31, 49),
        [30, 95, 65],
    );
    let evidence = adapter_with_progress(
        vec![
            source,
            obscured.clone(),
            obscured.clone(),
            obscured,
            middle_frame,
            final_frame.clone(),
            final_frame,
        ],
        None,
        |plan| {
            plan.trace = replay::replay(&plan.acquisition.deal, &actions[..2], false).unwrap();
            plan.objective = RouteObjective::Removal {
                removed: 4,
                proof: PartialProof::Bounded(xsar::search::SearchStop::ExpansionLimit),
            };
        },
        2,
        |io| {
            let result = drive(io.plan, io);
            (result, io.failure_progress.clone())
        },
    );
    assert!(evidence.result.0.is_ok(), "{:?}", evidence.result.0);
    assert_eq!(evidence.captures, 7);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (4, 4)
    );
    assert_eq!(evidence.result.1.initial_checks, 3);
    assert_eq!(evidence.result.1.fresh_retries, 2);
    assert_eq!(evidence.wire.unused_frames, 0);
    // Freshness and no mid-pair capture are verified by the existing wire harness.
}


/// The existing finite precondition budget stops unreadable sources without any input.
#[test]
fn readiness_173_six_misses_exhaust_without_authorising_a_gesture() {
    let deal = original_deal();
    let actions = accepted_actions();
    let initial = SearchState::initial();
    let operation = operations(actions[0]).unwrap()[0];
    let InputOperation::Click(point) = operation else {
        panic!("expected MATCH");
    };
    let slot = crate::pyramid::PYRAMID_TARGETS
        .iter()
        .find(|target| target.click_point == point)
        .unwrap();
    let mut obscured = synthetic_state(&deal, initial);
    paint(
        &mut obscured,
        PixelRect::new(slot.bounds.x + 3, slot.bounds.y + 4, 31, 49),
        [30, 95, 65],
    );
    let evidence = adapter(vec![obscured; ORDINARY_OBSERVATIONS], None, |io| {
        let result = drive(io.plan, io);
        (result, io.failure_progress.clone())
    });
    assert!(evidence.result.0.is_err());
    assert_eq!(evidence.captures, ORDINARY_OBSERVATIONS);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert_eq!(
        evidence.result.1.stage,
        crate::pyramid_diagnostics::progress::FailureStage::InitialSource
    );
    assert_eq!(evidence.result.1.initial_checks, ORDINARY_OBSERVATIONS);
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// A received unsupported frame is reported as retained even when validation stops immediately.
#[test]
fn readiness_173_unsupported_fresh_frame_is_counted_without_retry_or_input() {
    let deal = original_deal();
    let before = SearchState::initial();
    let after = rules::apply(&deal, before, accepted_actions()[0]).unwrap();
    let good = synthetic_state(&deal, before);
    let mut bad = good.clone();
    paint(&mut bad, PixelRect::new(0, 0, 1920, 1080), [0; 3]);
    let evidence = adapter(vec![good, bad], None, |io| {
        assert!(matches!(
            io.observe_source(before).unwrap(),
            EffectObservation::Position(true)
        ));
        io.source_proof = None;
        io.record_stage(
            crate::pyramid_diagnostics::progress::FailureStage::InitialSource,
            1,
            io.plan.trace.len(),
            1,
        );
        assert!(io.observe_action_source(before, after).is_err());
        assert_eq!(io.failure_progress.fresh_retries, 1);
        assert_eq!(io.failure_progress.capture, 2);
    });
    assert_eq!(evidence.captures, 2);
    assert_eq!(evidence.counts.attempted, 0);
    assert_eq!(evidence.wire.unused_frames, 0);
}
