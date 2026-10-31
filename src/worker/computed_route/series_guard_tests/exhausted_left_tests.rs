//! Native exhausted-LEFT regression with an independently reconstructed legal suffix.
//!
//! The first frame is unchanged retained evidence. Later frames are explicitly
//! synthetic effects; this adapter test grants no live acquisition or resume authority.

use super::*;
use std::sync::atomic::Ordering;


/// Unchanged retained full frame and complete deal from the stopped 2.9.136 run.
const FRAME: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-exhausted-left-endgame/last-observed.png");


/// Original card inventory, decoded through the strict shared codec.
const DEAL: &[u8] =
    include_bytes!("../../../../tests/fixtures/pyramid-exhausted-left-endgame/deal.json");


/// Resolve an explicitly zero-based original identity through the current legal piles.
/// Panics when reconstruction data selects an unavailable identity or invalid index.
fn location(state: SearchState, index: u64) -> Location {
    let index = u8::try_from(index).unwrap();
    if usize::from(index) < TABLEAU_LEN {
        return Location::Tableau(TableauLocation::from_index(usize::from(index)).unwrap());
    }
    [Location::DL, Location::DR]
        .into_iter()
        .find(|&location| rules::card_index(state, location) == Some(index))
        .expect("reconstructed stock identity must be a currently visible pile top")
}


/// Independently apply and replay the new reconstruction, retaining every real state.
/// This is not the original unrecorded route. Invalid fixture data or legality panics.
fn reconstruction() -> (CompleteDeal, Vec<RecordedAction>, Vec<SearchState>) {
    let deal = xsar::pyramid_json::read_from(DEAL).unwrap();
    let rows: Vec<Vec<Value>> = serde_json::from_slice(include_bytes!(
        "../../../../tests/fixtures/pyramid-exhausted-left-endgame/reconstructed-actions.json"
    ))
    .unwrap();
    let mut state = SearchState::initial();
    let mut states = vec![state];
    let mut actions = Vec::new();
    for row in rows {
        let action = match row[0].as_str().unwrap() {
            "KING" => RecordedAction::King(location(state, row[1].as_u64().unwrap())),
            "MATCH" => RecordedAction::Match(
                location(state, row[1].as_u64().unwrap()),
                location(state, row[2].as_u64().unwrap()),
            ),
            "MOVE" => RecordedAction::Move,
            "RECYCLE" => RecordedAction::Recycle,
            other => panic!("unknown reconstruction operation: {other}"),
        };
        state = rules::apply(&deal, state, action).unwrap();
        states.push(state);
        actions.push(action);
    }
    assert_eq!(actions.len(), 52);
    assert!(state.is_goal());
    assert_eq!(replay::replay(&deal, &actions, true).unwrap().len(), 52);
    let before = states[50];
    assert_eq!(before.tableau_remaining(), 2);
    assert!(before.tableau_present(TableauLocation::from_index(0).unwrap()));
    assert!(before.tableau_present(TableauLocation::from_index(2).unwrap()));
    assert_eq!(rules::card_index(before, Location::DL), None);
    assert_eq!(rules::card_index(before, Location::DR), Some(50));
    assert_eq!(
        actions[50],
        RecordedAction::Match(
            Location::DR,
            Location::Tableau(TableauLocation::from_index(2).unwrap())
        )
    );
    (deal, actions, states)
}


/// Create a labelled synthetic successor, preserving original header and scene pixels.
/// Only newly removed slots and changed visible pile faces are painted; bad geometry panics.
fn successor(
    frame: &CapturedFrame,
    deal: &CompleteDeal,
    before: SearchState,
    after: SearchState,
) -> CapturedFrame {
    let mut frame = frame.clone();
    for index in 0..TABLEAU_LEN {
        let location = TableauLocation::from_index(index).unwrap();
        if before.tableau_present(location) && !after.tableau_present(location) {
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
    let (_, old_left, old_right) = expected(deal, before).unwrap();
    let (_, left, right) = expected(deal, after).unwrap();
    let profile: Value = serde_json::from_slice(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v7/profile.json"
    ))
    .unwrap();
    for (target, old, card, empty) in [
        (crate::pyramid::LEFT_TARGET, old_left, left, "left_empty"),
        (
            crate::pyramid::RIGHT_TARGET,
            old_right,
            right,
            "right_empty",
        ),
    ] {
        if old == card {
            continue;
        }
        if let Some(card) = card {
            paint(&mut frame, target.bounds, [255; 3]);
            let red = matches!(
                card.suit(),
                xsar::games::cards::Suit::Diamonds | xsar::games::cards::Suit::Hearts
            );
            for (bank, label) in [
                ("ranks", card.rank().as_str()),
                ("suits", card.suit().as_str()),
            ] {
                super::cadence_tests::synthetic_glyph(
                    &mut frame,
                    &profile["stock_reader"],
                    bank,
                    label,
                    (target.bounds.x, target.bounds.y),
                    red,
                );
            }
        } else {
            paint(&mut frame, target.bounds, [30, 95, 65]);
            empty_pile_patch(&mut frame, empty);
        }
    }
    frame
}


/// The real frame supplies only exact position authority for the two remaining planned actions.
#[test]
fn exhausted_left_winning_suffix_keeps_match_then_king_without_redeal() {
    let (deal, actions, states) = reconstruction();
    let frame = decode_png(FRAME).unwrap();
    let vision = ReconVision::bundled_native_v23().unwrap();
    let (tableau, left, right) = expected(&deal, states[50]).unwrap();
    assert!(
        vision
            .verify_position(&frame, &tableau, left, right)
            .unwrap_err()
            .contains("scene[2]")
    );
    vision
        .verify_native_stock_continuation(&frame, &tableau, left, right)
        .unwrap();
    let bound = crate::pyramid_progress::observe_challenge(&frame)
        .unwrap()
        .unwrap();
    let middle = successor(&frame, &deal, states[50], states[51]);
    let mut last = successor(&middle, &deal, states[51], states[52]);
    // A synthetic completed-board effect uses an actual retained 2/3 counter crop.
    // Unchanged 1/3 would correctly fail the existing final-clear increment contract.
    let counter_source = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"
    ))
    .unwrap();
    let counter = PixelRect::new(1_038, 48, 72, 32);
    let view = counter_source.as_view().unwrap();
    let mut rgb = Vec::new();
    for y in counter.y..counter.bottom() {
        for x in counter.x..counter.right() {
            rgb.extend_from_slice(&view.pixel_rgb(x, y).unwrap());
        }
    }
    patch(&mut last, counter, &rgb);
    let evidence = adapter_with_setup(
        vec![frame, middle.clone(), middle, last.clone(), last],
        None,
        |plan| {
            plan.acquisition.deal = xsar::pyramid_json::read_from(DEAL).unwrap();
            plan.acquisition.bytes = DEAL.to_vec();
            plan.acquisition.vision = vision;
            plan.trace = replay::replay(&plan.acquisition.deal, &actions, true).unwrap();
            plan.objective = RouteObjective::Clear;
        },
        |io| {
            io.challenge = Some(bound);
            io.challenge_initial = Some(bound);
            retain_verified_position(io, states[50]);
            for index in 50..52 {
                assert!(matches!(
                    io.observe_source(states[index]).unwrap(),
                    EffectObservation::Position(true)
                ));
                let operations = operations(actions[index]).unwrap();
                io.begin_transition(states[index], states[index + 1], &operations).unwrap();
                for operation in operations {
                    io.deliver(operation, states[index]).unwrap();
                }
                for _ in 0..1 {
                    assert!(matches!(io.observe_effect(states[index + 1], index == 51).unwrap(), EffectObservation::Position(false)));
                }
                let observation = io.observe_effect(states[index + 1], index == 51).unwrap();
                assert!(
                    matches!(observation, EffectObservation::Position(true)),
                    "suffix action {index}: {:?}",
                    io.first_mismatch.as_ref().map(|(reason, _)| reason)
                );
                assert!(!io.stock_continuation);
            }
        },
    );
    assert_eq!(
        (
            evidence.captures,
            evidence.counts.attempted,
            evidence.counts.acknowledged
        ),
        (5, 3, 3)
    );
    assert!(evidence.first.is_none());
    assert_eq!(evidence.wire.unused_frames, 0);
    assert_eq!(
        evidence
            .wire
            .commands
            .iter()
            .filter(|name| *name == "query-mice")
            .count(),
        3
    );
    assert_eq!(
        evidence
            .wire
            .events
            .iter()
            .filter_map(|event| event["data"]["down"].as_bool())
            .collect::<Vec<_>>(),
        [true, false, true, false, true, false]
    );
    assert!(
        evidence
            .wire
            .events
            .iter()
            .all(|event| event["type"] != "key")
    );
}


/// Wrong replayed piles or a revoked STOP cannot turn native stock controls into input authority.
#[test]
fn exhausted_left_wrong_state_and_stopped_source_authorise_no_input() {
    let (_, actions, states) = reconstruction();
    for stopped in [false, true] {
        let frame = decode_png(FRAME).unwrap();
        let bound = crate::pyramid_progress::observe_challenge(&frame)
            .unwrap()
            .unwrap();
        let evidence = adapter_with_setup(
            if stopped { vec![] } else { vec![frame] },
            None,
            |plan| {
                plan.acquisition.deal = xsar::pyramid_json::read_from(DEAL).unwrap();
                plan.acquisition.bytes = DEAL.to_vec();
                plan.acquisition.vision = ReconVision::bundled_native_v23().unwrap();
                plan.trace = replay::replay(&plan.acquisition.deal, &actions, true).unwrap();
                plan.objective = RouteObjective::Clear;
            },
            |io| {
                io.challenge = Some(bound);
                io.challenge_initial = Some(bound);
                if stopped {
                    io.cancel.store(true, Ordering::SeqCst);
                }
                let result = io.observe_source(if stopped { states[50] } else { states[51] });
                if stopped {
                    assert!(result.is_err());
                } else {
                    assert!(matches!(
                        result.unwrap(),
                        EffectObservation::Position(false)
                    ));
                }
                assert!(io.source_proof.is_none());
                assert!(io.action_permit.is_none());
            },
        );
        assert_eq!(
            (evidence.counts.attempted, evidence.counts.acknowledged),
            (0, 0)
        );
        assert!(evidence.wire.events.is_empty());
    }
}
