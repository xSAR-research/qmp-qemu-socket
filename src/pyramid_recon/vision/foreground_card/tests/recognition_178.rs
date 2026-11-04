//! Retained Diamond refusals, additive calibration bounds and unchanged native guards.

use super::*;


/// Decode an unchanged saved failure, without a guest capture.
fn retained(name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-178")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Every printed corner was independently read in both retained originals.
fn manual_tableau() -> [Card; 28] {
    [
        (Rank::Two, Suit::Spades),
        (Rank::Three, Suit::Hearts),
        (Rank::Seven, Suit::Clubs),
        (Rank::Five, Suit::Hearts),
        (Rank::King, Suit::Clubs),
        (Rank::Eight, Suit::Clubs),
        (Rank::Jack, Suit::Diamonds),
        (Rank::King, Suit::Hearts),
        (Rank::Nine, Suit::Spades),
        (Rank::Six, Suit::Diamonds),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Ace, Suit::Spades),
        (Rank::Jack, Suit::Hearts),
        (Rank::Two, Suit::Hearts),
        (Rank::Eight, Suit::Spades),
        (Rank::Seven, Suit::Spades),
        (Rank::Nine, Suit::Diamonds),
        (Rank::Ace, Suit::Clubs),
        (Rank::Three, Suit::Spades),
        (Rank::Six, Suit::Hearts),
        (Rank::Five, Suit::Spades),
        (Rank::Nine, Suit::Hearts),
        (Rank::Five, Suit::Clubs),
        (Rank::Six, Suit::Spades),
        (Rank::Four, Suit::Diamonds),
        (Rank::Five, Suit::Diamonds),
        (Rank::King, Suit::Diamonds),
        (Rank::Nine, Suit::Clubs),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Both complete boards must pass, including all cards after the former first refusal.
#[test]
fn native_178_saved_coral_boards_read_all_manual_identities() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    for name in ["coral-entry.png", "coral-transition.png"] {
        let frame = retained(name);
        let ReconScene::Board(board) = vision.observe(&frame).unwrap() else {
            panic!("complete retained Coral board")
        };
        assert_eq!(board.tableau, manual_tableau());
        assert_eq!(board.tableau[25], Card::new(Rank::Five, Suit::Diamonds));
        assert_eq!(
            board.left,
            StockFace::Card(Card::new(Rank::King, Suit::Spades))
        );
        assert_eq!(board.right, StockFace::Empty);
    }
}


/// Read the bundled metadata for strictly in-memory malformed calibration derivatives.
fn metadata() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../assets/pyramid-glyphs/underwater-corners-v1.json"
    ))
    .unwrap()
}


/// Compile test calibration through the same private production loader.
fn compile(value: &Value) -> Result<Family, String> {
    underwater_family_from_json(&serde_json::to_string(value).unwrap())
}


/// Only one Diamond glyph is added; all previous ranks and suit variants remain exact.
#[test]
fn native_178_additive_suit_preserves_banks_and_resolves_measured_refusal() {
    let full = metadata();
    let mut original = full.clone();
    original.as_object_mut().unwrap().remove("suit_variants");
    let base = compile(&original).unwrap();
    let expanded = compile(&full).unwrap();
    assert_eq!(base.ranks, expanded.ranks);
    for ((suit, old), (new_suit, new)) in base.suits.iter().zip(&expanded.suits) {
        assert_eq!(suit, new_suit);
        assert!(new.starts_with(old));
        assert_eq!(new.len(), old.len() + usize::from(*suit == Suit::Diamonds));
    }
    let frame = retained("coral-transition.png");
    let pixels = corner_pixels(frame.as_view().unwrap(), TABLEAU_ORIGINS[25]).unwrap();
    let corners = corners_with_ink(
        &pixels,
        None,
        Some(UNDERWATER_SEPARATOR),
        false,
        InkPolicy::Sepia,
    );
    assert_eq!(corners.len(), 1);
    let corner = &corners[0];
    assert_eq!(
        matching_label_with_tolerance(
            &corner.rank,
            &base.ranks,
            RANK_ERROR,
            RANK_GAP,
            base.native_tolerance()
        ),
        Some(Rank::Five)
    );
    let old_bank = base.suit_bank(corner, Some(Rank::Five));
    let scores = ranked_labels(&corner.suit, &old_bank);
    assert_eq!(scores[0].0, Suit::Diamonds);
    assert_eq!((scores[0].1.difference, scores[0].1.mass), (16292, 80964));
    assert_eq!(
        matching_label_with_tolerance(
            &corner.suit,
            &old_bank,
            SUIT_ERROR,
            SUIT_GAP,
            base.native_tolerance()
        ),
        None
    );
    let new_bank = expanded.suit_bank(corner, Some(Rank::Five));
    let scores = ranked_labels(&corner.suit, &new_bank);
    assert_eq!((scores[0].1.difference, scores[0].1.mass), (8416, 76288));
    assert_eq!(
        matching_label_with_tolerance(
            &corner.suit,
            &new_bank,
            SUIT_ERROR,
            SUIT_GAP,
            expanded.native_tolerance()
        ),
        Some(Suit::Diamonds)
    );
}


/// The supplement is exactly an independent retained full corner, without resampling.
#[test]
fn native_178_supplement_is_complete_original_five_diamonds() {
    let data = metadata();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-158/underwater.png");
    let frame = decode_png(&std::fs::read(source).unwrap()).unwrap();
    let pixels = corner_pixels(frame.as_view().unwrap(), TABLEAU_ORIGINS[19]).unwrap();
    let expected: String = pixels
        .iter()
        .flatten()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(data["suit_variants"].as_array().unwrap().len(), 1);
    assert_eq!(data["suit_variants"][0]["rank"], "5");
    assert_eq!(data["suit_variants"][0]["suit"], "diamonds");
    assert_eq!(data["suit_variants"][0]["rgb"], expected);
}


/// Malformed, unmeasured, duplicate and mismatched declarations never supply a suit bank.
#[test]
fn native_178_supplement_rejects_malformed_over_budget_and_wrong_identity() {
    let full = metadata();
    let mut empty = full.clone();
    empty["suit_variants"] = serde_json::json!([]);
    assert!(compile(&empty).is_ok());
    for value in [
        Value::Null,
        serde_json::json!("wrong"),
        serde_json::json!({}),
    ] {
        let mut bad = full.clone();
        bad["suit_variants"] = value;
        assert!(compile(&bad).is_err());
    }
    let mut over = full.clone();
    over["suit_variants"] = serde_json::json!([full["suit_variants"][0], full["suit_variants"][0]]);
    assert!(compile(&over).unwrap_err().contains("budget"));
    for (field, value) in [
        ("rank", serde_json::json!("6")),
        ("suit", serde_json::json!("hearts")),
        ("rgb", Value::Null),
        ("rgb", serde_json::json!("00")),
    ] {
        let mut bad = full.clone();
        bad["suit_variants"][0][field] = value;
        assert!(compile(&bad).is_err());
    }
    let mut bad_hex = full.clone();
    let rgb = full["suit_variants"][0]["rgb"].as_str().unwrap();
    bad_hex["suit_variants"][0]["rgb"] = serde_json::json!(format!("zz{}", &rgb[2..]));
    assert!(compile(&bad_hex).is_err());
    let mut wrong_rank = full.clone();
    let three = full["corners"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["rank"] == "3")
        .unwrap();
    wrong_rank["suit_variants"][0]["rgb"] = three["rgb"].clone();
    assert!(compile(&wrong_rank).unwrap_err().contains("identity"));
    let mut wrong_suit = full.clone();
    let hearts_frame = retained("coral-entry.png");
    let pixels = corner_pixels(hearts_frame.as_view().unwrap(), TABLEAU_ORIGINS[3]).unwrap();
    let hearts: String = pixels
        .iter()
        .flatten()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    wrong_suit["suit_variants"][0]["rgb"] = serde_json::json!(hearts);
    assert!(compile(&wrong_suit).unwrap_err().contains("identity"));
    let mut duplicate = full.clone();
    let five = duplicate["corners"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["rank"] == "5")
        .unwrap();
    *five = full["suit_variants"][0].clone();
    assert!(compile(&duplicate).unwrap_err().contains("Duplicate"));
    let mut black = full.clone();
    let reference = full["corners"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["suit"] == "clubs")
        .unwrap();
    black["suit_variants"][0]["rgb"] = reference["rgb"].clone();
    assert!(compile(&black).is_err());
    let mut blank = full.clone();
    blank["suit_variants"][0]["rgb"] =
        serde_json::json!("ededed".repeat(CORNER_WIDTH * CORNER_HEIGHT));
    assert!(compile(&blank).is_err());
}


/// Added suit coverage cannot infer an erased, clipped or conflicting printed identity.
#[test]
fn native_178_damaged_retained_corner_still_refuses_recognition() {
    let vision = super::super::super::ReconVision::bundled_native_v23().unwrap();
    for (left, top, right, bottom, rgb) in [
        (3, 4, 34, 30, [237, 237, 237, 255]),
        (3, 30, 34, 53, [237, 237, 237, 255]),
        (13, 30, 34, 53, [237, 237, 237, 255]),
        (8, 15, 10, 19, [123, 90, 26, 255]),
    ] {
        let mut frame = retained("coral-entry.png");
        let (x, y) = TABLEAU_ORIGINS[25];
        for py in y + top..y + bottom {
            for px in x + left..x + right {
                let at = py as usize * frame.stride + px as usize * 4;
                frame
                    .pixels
                    .get_mut(at..at + 4)
                    .unwrap()
                    .copy_from_slice(&rgb);
            }
        }
        assert_eq!(
            vision
                .tableau_reader
                .recognise(frame.as_view().unwrap(), (x, y))
                .unwrap(),
            None,
            "damaged bounds {left},{top},{right},{bottom}"
        );
        assert!(vision.observe(&frame).is_err());
    }
}
