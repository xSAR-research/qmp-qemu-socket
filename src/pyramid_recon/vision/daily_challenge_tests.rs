//! Daily-challenge recognition over retained original native failure evidence.
//!
//! Solver toolbar appearance supplies no recognition authority. Supported scene,
//! exact identities, positive-empty piles and modal refusal remain required.
//! These offline observations do not establish inventory, restoration or input consent.

use super::*;


/// Resolve original daily evidence or existing calibration without rewriting it.
fn fixture_path(directory: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(directory)
        .join(name)
}


/// Decode the exact retained first capture through the production PNG decoder.
fn original() -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path(
            "daily-challenge-solver-unavailable",
            "baseline.png",
        ))
        .expect("retained original daily-challenge capture"),
    )
    .expect("native daily-challenge layout")
}


/// Load the unchanged v7 calibration through the bounded production file reader.
fn vision() -> ReconVision {
    ReconVision::load(&fixture_path("native-v7", "profile.json")).expect("unchanged v7 profile")
}


/// Return visually labelled original identities in fixed top-to-bottom row order.
fn labelled_tableau() -> [Card; 28] {
    [
        (Rank::Three, Suit::Spades),
        (Rank::Three, Suit::Hearts),
        (Rank::Jack, Suit::Diamonds),
        (Rank::Jack, Suit::Hearts),
        (Rank::Two, Suit::Spades),
        (Rank::Seven, Suit::Hearts),
        (Rank::Ace, Suit::Clubs),
        (Rank::Ace, Suit::Diamonds),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Five, Suit::Spades),
        (Rank::Two, Suit::Diamonds),
        (Rank::King, Suit::Hearts),
        (Rank::Six, Suit::Clubs),
        (Rank::Five, Suit::Diamonds),
        (Rank::Three, Suit::Diamonds),
        (Rank::King, Suit::Diamonds),
        (Rank::Queen, Suit::Clubs),
        (Rank::Nine, Suit::Diamonds),
        (Rank::Seven, Suit::Spades),
        (Rank::Four, Suit::Diamonds),
        (Rank::Ace, Suit::Spades),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Nine, Suit::Spades),
        (Rank::Five, Suit::Hearts),
        (Rank::Eight, Suit::Diamonds),
        (Rank::Six, Suit::Spades),
        (Rank::Ten, Suit::Diamonds),
        (Rank::Eight, Suit::Hearts),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Return the directly visible initial LEFT identity without unseen stock assumptions.
fn labelled_left() -> Card {
    Card::new(Rank::King, Suit::Clubs)
}


/// Paint a bounded in-memory derivative while retaining alpha and native layout.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    assert!(frame.as_view().is_ok());
    assert!(bounds.right() <= frame.width && bounds.bottom() <= frame.height);
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Full observation and route comparison recognise the original grey-unavailable scene.
#[test]
fn daily_challenge_original_recognises_exact_cards_without_solver_visual_gate() {
    let reader = vision();
    let frame = original();
    let pixels = frame.pixels.clone();
    let ReconScene::Board(board) = reader.observe(&frame).expect("supported daily scene") else {
        panic!("grey unavailable Solver cannot reject the supported native board");
    };
    assert_eq!(board.tableau, labelled_tableau());
    assert_eq!(board.left, StockFace::Card(labelled_left()));
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
    reader
        .verify_position(
            &frame,
            &labelled_tableau().map(Some),
            Some(labelled_left()),
            None,
        )
        .expect("exact original daily route position");
    assert_eq!(
        frame.pixels, pixels,
        "recognition preserves original pixels"
    );
}


/// Recolouring the Solver toolbar does not replace or alter recognised card evidence.
#[test]
fn daily_challenge_solver_toolbar_mutations_preserve_exact_observation() {
    let reader = vision();
    let frame = original();
    let expected = reader.observe(&frame).expect("supported daily board");
    assert!(matches!(expected, ReconScene::Board(_)));
    for colour in [[0, 0, 0], [255, 255, 255], [128, 128, 128], [255, 210, 0]] {
        let mut changed = frame.clone();
        fill_region(&mut changed, SOLVER_BOUNDS, colour);
        assert_eq!(
            reader
                .observe(&changed)
                .expect("toolbar is not a scene gate"),
            expected
        );
        reader
            .verify_position(
                &changed,
                &labelled_tableau().map(Some),
                Some(labelled_left()),
                None,
            )
            .expect("exact cards and supported scene remain mandatory");
    }
}


/// Erasing any actual scene signature still refuses full and partial observations.
#[test]
fn daily_challenge_requires_every_supported_scene_patch() {
    let reader = vision();
    for pattern in &reader.board.scene {
        let mut changed = original();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader
                .observe(&changed)
                .expect("valid unsupported-scene layout"),
            ReconScene::Unknown
        );
        assert!(
            reader
                .verify_position(
                    &changed,
                    &labelled_tableau().map(Some),
                    Some(labelled_left()),
                    None,
                )
                .is_err()
        );
    }
}


/// Unknown tableau glyphs cannot become an accepted identity or inferred absence.
#[test]
fn daily_challenge_unknown_card_refuses_full_and_partial_observations() {
    let reader = vision();
    let mut changed = original();
    let (x, y) = TABLEAU_ORIGINS[0];
    fill_region(&mut changed, PixelRect::new(x, y, 40, 52), [17, 19, 21]);
    let error = reader
        .observe(&changed)
        .expect_err("an erased rank and suit cannot become a card");
    assert!(error.contains("tableau slot 0"), "{error}");
    assert!(
        reader
            .verify_position(
                &changed,
                &labelled_tableau().map(Some),
                Some(labelled_left()),
                None,
            )
            .is_err()
    );
}


/// The empty RIGHT pile still needs its independent positive appearance.
#[test]
fn daily_challenge_missing_positive_empty_pile_refuses() {
    let reader = vision();
    let mut changed = original();
    fill_region(&mut changed, reader.board.right_empty.bounds, [17, 19, 21]);
    assert!(
        reader
            .observe(&changed)
            .expect_err("missing empty-pile evidence")
            .starts_with("RIGHT:")
    );
    assert!(
        reader
            .verify_position(
                &changed,
                &labelled_tableau().map(Some),
                Some(labelled_left()),
                None,
            )
            .is_err()
    );
}


/// A recognised Undo All modal supplies no route-position authority.
#[test]
fn daily_challenge_expected_position_refuses_the_specific_undo_modal() {
    let reader = vision();
    let modal = crate::capture::decode_png(
        &std::fs::read(fixture_path("originals", "undo-all-confirmation.png"))
            .expect("retained original Undo All modal"),
    )
    .expect("native modal PNG");
    assert!(matches!(
        reader.observe(&modal).expect("specific calibrated modal"),
        ReconScene::UndoConfirmation { .. }
    ));
    assert!(
        reader
            .verify_position(
                &modal,
                &labelled_tableau().map(Some),
                Some(labelled_left()),
                None,
            )
            .is_err()
    );
}


/// Retained refusal metadata remains incomplete pre-input evidence after the correction.
#[test]
fn daily_challenge_original_refusal_is_preserved_without_retroactive_success() {
    let report: Value = serde_json::from_slice(
        &std::fs::read(fixture_path(
            "daily-challenge-solver-unavailable",
            "INCOMPLETE.json",
        ))
        .expect("retained original refusal report"),
    )
    .expect("original refusal JSON");
    assert_eq!(report["captures_attempted"], 1);
    assert_eq!(report["core_captures"], 1);
    assert_eq!(
        report["error"],
        "Unknown native scene; input authority revoked"
    );
    assert_eq!(report["publication_status"], "not_attempted");
    for field in [
        "core_draws",
        "draws_attempted",
        "draws_acknowledged",
        "undo_all_attempted",
        "confirm_attempted",
        "tableau_count",
        "stock_count",
    ] {
        assert_eq!(report[field], 0, "retained {field}");
    }
    for field in [
        "complete",
        "restored",
        "complete_output_published",
        "input_delivery_uncertain",
    ] {
        assert_eq!(report[field], false, "retained {field}");
    }
}


/// The runtime default embeds the measured profile and recognises daily evidence without a file.
#[test]
fn embedded_profile_recognises_daily_original_like_the_custom_file_loader() {
    let reader = ReconVision::bundled_native_v11().expect("embedded strict native-v11 profile");
    let frame = original();
    assert_eq!(reader.profile_id(), "pyramid-native-1920-classic-blue-v11");
    assert_eq!(reader.observe(&frame), vision().observe(&frame));
    let ReconScene::Board(board) = reader.observe(&frame).expect("embedded daily observation")
    else {
        panic!("embedded daily scene");
    };
    assert_eq!(board.tableau, labelled_tableau());
    assert_eq!(board.left, StockFace::Card(labelled_left()));
    assert_eq!(board.right, StockFace::Empty);
    reader
        .verify_position(
            &frame,
            &labelled_tableau().map(Some),
            Some(labelled_left()),
            None,
        )
        .expect("embedded default exact daily route position");
}
