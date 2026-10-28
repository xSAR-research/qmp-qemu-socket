//! Original 157 refusal images exercised through the actual private native reader.

use super::*;


/// Decode a retained original without contacting a guest or changing its pixels.
fn original(name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-158")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Manually labelled original tableau identities; locations retain their fixed row order.
fn underwater_tableau(second: bool) -> [Card; 28] {
    if second {
        [
            (Rank::Six, Suit::Hearts),
            (Rank::Two, Suit::Hearts),
            (Rank::Ten, Suit::Hearts),
            (Rank::Nine, Suit::Diamonds),
            (Rank::Ace, Suit::Spades),
            (Rank::Ace, Suit::Diamonds),
            (Rank::Nine, Suit::Clubs),
            (Rank::Two, Suit::Spades),
            (Rank::Seven, Suit::Diamonds),
            (Rank::Queen, Suit::Diamonds),
            (Rank::Four, Suit::Hearts),
            (Rank::Six, Suit::Diamonds),
            (Rank::Jack, Suit::Clubs),
            (Rank::Ten, Suit::Clubs),
            (Rank::Queen, Suit::Hearts),
            (Rank::Five, Suit::Clubs),
            (Rank::Three, Suit::Diamonds),
            (Rank::Seven, Suit::Hearts),
            (Rank::Six, Suit::Spades),
            (Rank::Four, Suit::Clubs),
            (Rank::Nine, Suit::Spades),
            (Rank::King, Suit::Diamonds),
            (Rank::Three, Suit::Hearts),
            (Rank::Five, Suit::Hearts),
            (Rank::Five, Suit::Spades),
            (Rank::King, Suit::Hearts),
            (Rank::Ten, Suit::Diamonds),
            (Rank::Eight, Suit::Clubs),
        ]
    } else {
        [
            (Rank::King, Suit::Spades),
            (Rank::Seven, Suit::Clubs),
            (Rank::Four, Suit::Diamonds),
            (Rank::Four, Suit::Hearts),
            (Rank::Eight, Suit::Hearts),
            (Rank::Two, Suit::Diamonds),
            (Rank::Jack, Suit::Hearts),
            (Rank::Ten, Suit::Spades),
            (Rank::Ten, Suit::Clubs),
            (Rank::Ace, Suit::Spades),
            (Rank::Two, Suit::Clubs),
            (Rank::Four, Suit::Spades),
            (Rank::Jack, Suit::Clubs),
            (Rank::Ace, Suit::Hearts),
            (Rank::Queen, Suit::Diamonds),
            (Rank::Two, Suit::Spades),
            (Rank::Two, Suit::Hearts),
            (Rank::Seven, Suit::Spades),
            (Rank::Jack, Suit::Spades),
            (Rank::Five, Suit::Diamonds),
            (Rank::Seven, Suit::Hearts),
            (Rank::Five, Suit::Spades),
            (Rank::Six, Suit::Hearts),
            (Rank::Eight, Suit::Spades),
            (Rank::Six, Suit::Spades),
            (Rank::Ace, Suit::Clubs),
            (Rank::Five, Suit::Hearts),
            (Rank::Jack, Suit::Diamonds),
        ]
    }
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// A readable native Five of Spades must pass complete rank and suit matching.
#[test]
fn native_158_retained_five_spades_is_a_complete_supported_board() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    let frame = original("five-spades.png");
    let observed = vision.observe(&frame);
    assert!(observed.is_ok(), "actual whole-board reader: {observed:?}");
    let ReconScene::Board(observed) = observed.unwrap() else {
        panic!("complete board")
    };
    let expected = [
        (Rank::Seven, Suit::Clubs),
        (Rank::Five, Suit::Clubs),
        (Rank::Jack, Suit::Clubs),
        (Rank::Ten, Suit::Spades),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Four, Suit::Spades),
        (Rank::King, Suit::Clubs),
        (Rank::Nine, Suit::Clubs),
        (Rank::Jack, Suit::Diamonds),
        (Rank::Six, Suit::Spades),
        (Rank::Eight, Suit::Spades),
        (Rank::Six, Suit::Clubs),
        (Rank::King, Suit::Spades),
        (Rank::Five, Suit::Spades),
        (Rank::Two, Suit::Clubs),
        (Rank::Nine, Suit::Hearts),
        (Rank::Four, Suit::Diamonds),
        (Rank::Three, Suit::Spades),
        (Rank::Ten, Suit::Clubs),
        (Rank::Nine, Suit::Diamonds),
        (Rank::Three, Suit::Clubs),
        (Rank::Queen, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Nine, Suit::Spades),
        (Rank::Jack, Suit::Spades),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Ten, Suit::Hearts),
        (Rank::Three, Suit::Hearts),
    ]
    .map(|(rank, suit)| Card::new(rank, suit));
    assert_eq!(observed.tableau, expected);
    assert_eq!(
        observed.left,
        StockFace::Card(Card::new(Rank::Six, Suit::Diamonds))
    );
    assert_eq!(observed.right, StockFace::Empty);
}


/// The original underwater artwork must retain complete native identity evidence.
#[test]
fn native_158_retained_underwater_is_a_complete_supported_board() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    let frame = original("underwater.png");
    let observed = vision.observe(&frame);
    assert!(observed.is_ok(), "actual whole-board reader: {observed:?}");
    let ReconScene::Board(observed) = observed.unwrap() else {
        panic!("complete board")
    };
    assert_eq!(observed.tableau, underwater_tableau(false));
    assert_eq!(
        observed.left,
        StockFace::Card(Card::new(Rank::Eight, Suit::Clubs))
    );
    assert_eq!(observed.right, StockFace::Empty);
}


/// The independent frame's Threes, highlighted Nines and both stocks keep exact identities.
#[test]
fn native_158_underwater_three_nine_and_highlighted_cards_are_read() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    let frame = original("underwater-three-nine.png");
    let observed = vision.observe(&frame);
    assert!(observed.is_ok(), "actual whole-board reader: {observed:?}");
    let ReconScene::Board(observed) = observed.unwrap() else {
        panic!("complete board")
    };
    assert_eq!(observed.tableau, underwater_tableau(true));
    assert_eq!(
        observed.left,
        StockFace::Card(Card::new(Rank::Nine, Suit::Hearts))
    );
    assert_eq!(
        observed.right,
        StockFace::Card(Card::new(Rank::Eight, Suit::Spades))
    );
}


/// All thirteen labelled references must be exact native crops from the two originals.
#[test]
fn native_158_underwater_references_are_complete_original_pixels() {
    let asset: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../assets/pyramid-glyphs/underwater-corners-v1.json"
    ))
    .unwrap();
    let references = [
        (false, 9),
        (false, 10),
        (true, 16),
        (false, 2),
        (false, 21),
        (false, 22),
        (false, 1),
        (false, 4),
        (true, 6),
        (false, 7),
        (false, 6),
        (false, 14),
        (false, 0),
    ];
    let entries = asset["corners"].as_array().unwrap();
    assert_eq!(entries.len(), 13);
    for (entry, (second, slot)) in entries.iter().zip(references) {
        let frame = original(if second {
            "underwater-three-nine.png"
        } else {
            "underwater.png"
        });
        let card = underwater_tableau(second)[slot];
        assert_eq!(entry["rank"], card.rank().as_str());
        assert_eq!(entry["suit"], card.suit().as_str());
        let pixels = corner_pixels(frame.as_view().unwrap(), TABLEAU_ORIGINS[slot]).unwrap();
        let rgb: String = pixels
            .iter()
            .flatten()
            .map(|v| format!("{v:02x}"))
            .collect();
        assert_eq!(entry["rgb"], rgb);
        let corners = corners_with_ink(&pixels, None, Some(30), false, InkPolicy::Sepia);
        assert_eq!(corners.len(), 1);
        assert!(corners[0].whole_separator);
        if card.rank() == Rank::Ten {
            assert!(corners[0].rank.width >= 26);
        }
    }
}


/// Missing complete glyphs or a clipped zero must never become an underwater card.
#[test]
fn native_158_underwater_missing_glyphs_and_clipped_ten_are_refused() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    for (second, slot, left, top, right, bottom) in [
        (false, 0, 3, 4, 34, 30),
        (false, 0, 3, 30, 34, 53),
        (false, 7, 13, 4, 34, 30),
        (true, 14, 3, 4, 34, 30),
        (true, 14, 3, 30, 34, 53),
    ] {
        let mut frame = original(if second {
            "underwater-three-nine.png"
        } else {
            "underwater.png"
        });
        let (x, y) = TABLEAU_ORIGINS[slot];
        for py in y + top..y + bottom {
            for px in x + left..x + right {
                let at = py as usize * frame.stride + px as usize * 4;
                frame
                    .pixels
                    .get_mut(at..at + 4)
                    .unwrap()
                    .copy_from_slice(&[236, 236, 236, 255]);
            }
        }
        assert_eq!(
            vision
                .tableau_reader
                .recognise(frame.as_view().unwrap(), (x, y))
                .unwrap(),
            None,
            "synthetically damaged slot {slot}"
        );
        assert!(vision.observe(&frame).is_err());
    }
}


/// Conflicting brown print inside a real red rank remains mixed-colour evidence.
#[test]
fn native_158_underwater_mixed_print_is_refused() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    let mut frame = original("underwater.png");
    let (x, y) = TABLEAU_ORIGINS[22];
    for py in y + 15..y + 19 {
        for px in x + 8..x + 10 {
            let at = py as usize * frame.stride + px as usize * 4;
            frame
                .pixels
                .get_mut(at..at + 4)
                .unwrap()
                .copy_from_slice(&[123, 90, 26, 255]);
        }
    }
    assert_eq!(
        vision
            .tableau_reader
            .recognise(frame.as_view().unwrap(), (x, y))
            .unwrap(),
        None
    );
    assert!(vision.observe(&frame).is_err());
}



/// The calibrated source must remain the unchanged complete native corner.
#[test]
fn native_158_spade_reference_matches_its_original_pixels() {
    let frame = original("five-spades.png");
    let pixels = corner_pixels(frame.as_view().unwrap(), TABLEAU_ORIGINS[13]).unwrap();
    let raw: Vec<_> = pixels.into_iter().flatten().collect();
    assert_eq!(raw.as_slice(), include_bytes!("../classic-spade-158.rgb"));
}


/// A measured appearance does not stand in for a missing rank or suit.
#[test]
fn native_158_spade_missing_rank_or_suit_is_refused() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    for (top, bottom) in [(4, 30), (29, 53)] {
        let mut frame = original("five-spades.png");
        for y in 325 + top..325 + bottom {
            for x in 1094..1125 {
                let at = y as usize * frame.stride + x as usize * 4;
                frame
                    .pixels
                    .get_mut(at..at + 4)
                    .unwrap()
                    .copy_from_slice(&[238, 238, 238, 255]);
            }
        }
        let view = frame.as_view().unwrap();
        assert_eq!(
            vision
                .tableau_reader
                .recognise(view, TABLEAU_ORIGINS[13])
                .unwrap(),
            None
        );
        assert!(vision.observe(&frame).is_err());
    }
}


/// Complete Clubs and clipped Clubs cuts must not borrow the new Spades appearance.
#[test]
fn native_158_measured_spade_preserves_five_seven_and_ten_clubs() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    let frame = original("five-spades.png");
    let view = frame.as_view().unwrap();
    for (slot, rank) in [(1, Rank::Five), (18, Rank::Ten)] {
        assert_eq!(
            vision
                .tableau_reader
                .recognise(view, TABLEAU_ORIGINS[slot])
                .unwrap(),
            Some(Card::new(rank, Suit::Clubs))
        );
    }
    let seven = fixture("cream_first.png");
    assert_eq!(
        vision
            .tableau_reader
            .recognise(seven.as_view().unwrap(), (589, 272))
            .unwrap(),
        Some(Card::new(Rank::Seven, Suit::Clubs))
    );
}
