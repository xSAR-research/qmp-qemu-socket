//! Original repeated effect failures; direct fixed-slot truth and strict ink boundaries.

use super::*;


/// Decode retained original pixels without capture, replay or acquisition authority.
fn retained(name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-effect-155")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read direct original card labels without consulting the image classifier.
fn original_cards() -> [Option<Card>; 28] {
    let bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-effect-155/deal.json"),
    )
    .unwrap();
    let document: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document["tableau"].as_array().unwrap().len(), 28);
    std::array::from_fn(|index| {
        let label = &document["tableau"][index];
        let rank = Rank::ALL
            .into_iter()
            .find(|rank| u64::from(rank.value()) == label["value"].as_u64().unwrap())
            .unwrap();
        let suit = Suit::ALL
            .into_iter()
            .find(|suit| label["suit"] == suit.as_str())
            .unwrap();
        Some(Card::new(rank, suit))
    })
}


/// Both unchanged retained frames must prove their complete visible position and next DL.
#[test]
fn native_155_retained_king_diamond_stock_and_position() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let king = Card::new(Rank::King, Suit::Diamonds);
    let right = Card::new(Rank::Four, Suit::Clubs);
    let mut failures = Vec::new();
    for (name, removed) in [
        ("153-last.png", &[17usize, 18, 23, 24, 25][..]),
        ("154-last.png", &[17usize, 18, 23, 24, 25, 26][..]),
    ] {
        let frame = retained(name);
        let view = frame.as_view().unwrap();
        let mut expected = original_cards();
        for index in removed {
            expected[*index] = None;
        }
        let actual = vision.stock_reader.recognise(view, (759, 678)).unwrap();
        let position = vision.verify_position(&frame, &expected, Some(king), Some(right));
        println!("{name}: DL={actual:?}; position={position:?}");
        if actual != Some(king) || position.is_err() {
            failures.push(format!(
                "{name}: DL={actual:?}; position={position:?}; {}",
                vision.stock_reader.failure_details(view, (759, 678))
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}


/// Genuine contradictory black ink still vetoes a complete red corner, even at two seeds.
#[test]
fn native_155_two_neutral_black_seeds_still_refuse_a_red_corner() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let mut frame = retained("154-last.png");
    fill(&mut frame, PixelRect::new(763, 683, 1, 2), [0; 3]);
    assert_eq!(
        vision
            .stock_reader
            .recognise(frame.as_view().unwrap(), (759, 678))
            .unwrap(),
        None
    );
}


/// Paper and portrait ink cannot replace either independently required printed glyph.
#[test]
fn native_155_damaged_rank_or_suit_cannot_use_the_retained_portrait() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let mut expected = original_cards();
    for index in [17usize, 18, 23, 24, 25, 26] {
        expected[index] = None;
    }
    for bounds in [
        PixelRect::new(762, 682, 26, 26),
        PixelRect::new(763, 708, 22, 21),
    ] {
        let mut frame = retained("154-last.png");
        fill(&mut frame, bounds, [252; 3]);
        assert_eq!(
            vision
                .stock_reader
                .recognise(frame.as_view().unwrap(), (759, 678))
                .unwrap(),
            None
        );
        assert!(
            vision
                .verify_position(
                    &frame,
                    &expected,
                    Some(Card::new(Rank::King, Suit::Diamonds)),
                    Some(Card::new(Rank::Four, Suit::Clubs))
                )
                .is_err()
        );
    }
}


/// A correctly read face cannot satisfy a different planned suit or an unremoved source.
#[test]
fn native_155_complete_position_still_requires_the_exact_expected_effect() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let frame = retained("154-last.png");
    let mut expected = original_cards();
    for index in [17usize, 18, 23, 24, 25, 26] {
        expected[index] = None;
    }
    assert!(
        vision
            .verify_position(
                &frame,
                &expected,
                Some(Card::new(Rank::King, Suit::Hearts)),
                Some(Card::new(Rank::Four, Suit::Clubs))
            )
            .is_err()
    );
    expected[18] = original_cards()[18];
    assert!(
        vision
            .verify_position(
                &frame,
                &expected,
                Some(Card::new(Rank::King, Suit::Diamonds)),
                Some(Card::new(Rank::Four, Suit::Clubs))
            )
            .is_err()
    );
}
