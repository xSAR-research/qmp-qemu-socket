//! Exposed native gold-Seven regression with the unchanged graded matching policy.
//!
//! The original contains three objective-highlighted Sevens. Synthetic damaged
//! derivatives below exercise refusal only; they establish no guest effect,
//! challenge planning, missing stock identities or live acceptance.

use super::*;


/// Decode the first refused original without changing its pixels.
fn gold_frame() -> CapturedFrame {
    crate::capture::decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v16/gold-seven.png"
    ))
    .expect("retained native gold-Seven original")
}


/// Independent direct visual labels in fixed row/column order.
fn visible_tableau() -> [Card; 28] {
    use Rank::{Ace, Eight, Four, Jack, King, Nine, Queen, Seven, Six, Ten, Three};
    use Suit::{Clubs, Diamonds, Hearts, Spades};
    [
        (King, Hearts),
        (Ten, Hearts),
        (Eight, Diamonds),
        (Queen, Spades),
        (Three, Diamonds),
        (Seven, Clubs),
        (King, Diamonds),
        (Ten, Diamonds),
        (Jack, Hearts),
        (Ten, Spades),
        (Three, Spades),
        (Seven, Hearts),
        (Eight, Spades),
        (Four, Spades),
        (Six, Clubs),
        (Six, Spades),
        (Six, Diamonds),
        (Nine, Hearts),
        (Nine, Spades),
        (Queen, Clubs),
        (Four, Diamonds),
        (Queen, Hearts),
        (Seven, Spades),
        (King, Spades),
        (Nine, Clubs),
        (Queen, Diamonds),
        (Ace, Spades),
        (Eight, Hearts),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Both unchanged reader geometries must identify every directly labelled visible card.
#[test]
fn measured_gold_seven_reads_all_visible_cards_in_both_banks() {
    let frame = gold_frame();
    let reader = ReconVision::bundled_native_v16().expect("strict embedded v16");
    let old = ReconVision::bundled_native_v15().expect("unchanged frozen v15");
    let view = frame.as_view().expect("checked original layout");
    let expected = visible_tableau();
    assert_eq!(
        old.tableau_reader
            .recognise(view, TABLEAU_ORIGINS[5])
            .unwrap(),
        None
    );
    for index in [11, 22] {
        assert_eq!(
            old.tableau_reader
                .recognise(view, TABLEAU_ORIGINS[index])
                .unwrap(),
            Some(expected[index])
        );
    }
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        for (origin, card) in TABLEAU_ORIGINS.into_iter().zip(expected) {
            assert_eq!(bank.recognise(view, origin).unwrap(), Some(card));
        }
        assert_eq!(
            bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .unwrap(),
            Some(Card::new(Rank::Eight, Suit::Clubs))
        );
    }
    let ReconScene::Board(board) = reader.observe(&frame).expect("complete native observation")
    else {
        panic!("original complete board must be recognised");
    };
    assert_eq!(board.tableau, expected);
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Eight, Suit::Clubs))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert!(board.undo_all.is_none());
    reader
        .verify_position(
            &frame,
            &expected.map(Some),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
            None,
        )
        .unwrap();
}


/// Exactly one appended rank-seven variant per bank and the identifier distinguish v16.
#[test]
fn measured_gold_seven_rolls_back_to_the_entire_frozen_v15_profile() {
    let old: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v15/profile.json"
    ))
    .unwrap();
    let mut new: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v16/profile.json"
    ))
    .unwrap();
    let mut additions = Vec::new();
    for bank in ["tableau_reader", "stock_reader"] {
        let entry = new[bank]["ranks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["label"] == "7")
            .unwrap();
        let variants = entry["variants"].as_array_mut().unwrap();
        assert_eq!(variants.len(), 2);
        additions.push(variants.pop().unwrap());
    }
    assert_eq!(
        additions[0], additions[1],
        "both banks use the same measured RGB appearance"
    );
    new["profile_id"] = old["profile_id"].clone();
    assert_eq!(
        new, old,
        "all previous policy, glyphs, bounds, masks and modal/stock data remain exact"
    );
}


/// One measured appearance cannot replace an unknown rank or a different expected identity.
#[test]
fn measured_gold_seven_still_requires_positive_rank_and_exact_identity() {
    let reader = ReconVision::bundled_native_v16().unwrap();
    let frame = gold_frame();
    let mut wrong = visible_tableau().map(Some);
    wrong[5] = Some(Card::new(Rank::Two, Suit::Clubs));
    assert!(
        reader
            .verify_position(
                &frame,
                &wrong,
                Some(Card::new(Rank::Eight, Suit::Clubs)),
                None
            )
            .is_err()
    );
    let mut damaged = frame;
    for y in 224..248usize {
        for x in 1093..1120usize {
            let offset = y * damaged.stride + x * 4;
            damaged.pixels[offset..offset + 3].copy_from_slice(&[231; 3]);
        }
    }
    let error = reader
        .observe(&damaged)
        .expect_err("erasing the complete rank cannot infer Seven");
    assert!(
        error.contains("unknown rank/suit at tableau slot 5"),
        "{error}"
    );
}


/// Re-evaluate all 423 frozen direct labels through both current production banks.
#[test]
fn measured_gold_seven_preserves_the_entire_earlier_labelled_corpus() {
    let reader = ReconVision::bundled_native_v16().unwrap();
    let corpus: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v12/corpus.json"
    ))
    .unwrap();
    let observations = corpus["observations"].as_array().unwrap();
    let mut labels = 0;
    let mut decisions = 0;
    for source in corpus["sources"].as_array().unwrap() {
        assert_eq!(source["original_bytes"], true);
        assert_eq!(source["transformation"], "none");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v12")
            .join(source["file"].as_str().unwrap());
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(bytes.len() as u64, source["bytes"].as_u64().unwrap());
        let frame = crate::capture::decode_png(&bytes).unwrap();
        let view = frame.as_view().unwrap();
        for observation in observations
            .iter()
            .filter(|label| label["source_id"] == source["id"])
        {
            assert_eq!(observation["source_sha256"], source["sha256"]);
            assert_eq!(observation["recognition_authority"], false);
            let rank = Rank::ALL
                .into_iter()
                .find(|rank| Some(rank.as_str()) == observation["rank"].as_str())
                .unwrap();
            let suit = Suit::ALL
                .into_iter()
                .find(|suit| Some(suit.as_str()) == observation["suit"].as_str())
                .unwrap();
            let origin = (
                integer(&observation["origin"][0]).unwrap(),
                integer(&observation["origin"][1]).unwrap(),
            );
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, origin).unwrap(),
                    Some(Card::new(rank, suit)),
                    "direct label {}",
                    observation["id"]
                );
                decisions += 1;
            }
            labels += 1;
        }
    }
    assert_eq!((labels, decisions), (423, 846));
}
