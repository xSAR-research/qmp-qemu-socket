//! Native Jack-of-diamonds regression and unchanged strict recognition boundaries.

use super::tests::labelled_card;
use super::*;


/// Decode one original PNG or JSON relative to its permanent repository fixture.
fn fixture(generation: &str, name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon")
            .join(generation)
            .join(name),
    )
    .expect("retained native bytes")
}


/// Return the exact first failure; no derivative or replay grants acquisition authority.
fn original() -> CapturedFrame {
    crate::capture::decode_png(&fixture("native-v22", "recon-jd.png")).unwrap()
}


/// Convert an independently read conventional card label without consulting recognition.
fn card(label: &str) -> Card {
    let (rank, suit) = label.split_at(label.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|value| value.as_str() == rank)
        .unwrap();
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("invalid direct suit"),
    };
    Card::new(rank, suit)
}


/// Preserve all direct cards and read the original Sixes counter as generic Card progress.
#[test]
fn native_v22_reads_the_original_whole_board_and_jd_in_both_banks() {
    let frame = original();
    let old = ReconVision::bundled_native_v21().unwrap();
    let error = old.observe(&frame).unwrap_err();
    assert!(error.contains("tableau slot 6"), "{error}");
    assert!(error.contains("120414ppm"), "{error}");
    let reader = ReconVision::bundled_native_v22().unwrap();
    let ReconScene::Board(board) = reader.observe(&frame).unwrap() else {
        panic!("native original must retain complete board evidence");
    };
    let direct = [
        "9S", "7C", "8S", "7S", "3D", "10S", "JD", "AS", "AD", "9H", "6H", "4H", "8H", "2D", "QC",
        "AC", "6D", "9C", "JH", "QD", "3H", "KD", "KH", "9D", "KC", "5H", "5D", "KS",
    ];
    assert_eq!(board.tableau, direct.map(card));
    assert_eq!(board.left, StockFace::Card(card("8C")));
    assert_eq!(board.right, StockFace::Empty);
    let view = frame.as_view().unwrap();
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, TABLEAU_ORIGINS[6]).unwrap(),
            Some(card("JD"))
        );
    }
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&frame).unwrap(),
        crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Cards,
            0,
            12,
        )
    );
}


/// Removing exactly the two measured appends restores every decoded native-v21 field.
#[test]
fn native_v22_changes_only_profile_id_and_one_diamond_in_each_bank() {
    let prior: Value = serde_json::from_slice(&fixture("native-v21", "profile.json")).unwrap();
    let mut restored: Value =
        serde_json::from_slice(&fixture("native-v22", "profile.json")).unwrap();
    assert_eq!(
        restored["profile_id"],
        "pyramid-native-1920-classic-blue-v22"
    );
    restored["profile_id"] = prior["profile_id"].clone();
    let frame = original();
    let mut expected_rgb = String::new();
    for y in 302..318 {
        for x in 594..614 {
            let offset = y * frame.stride + x * 4;
            for channel in &frame.pixels[offset..offset + 3] {
                use std::fmt::Write;
                write!(&mut expected_rgb, "{channel:02x}").unwrap();
            }
        }
    }
    for name in ["tableau_reader", "stock_reader"] {
        let variants = restored[name]["suits"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["label"] == "diamonds")
            .unwrap()["variants"]
            .as_array_mut()
            .unwrap();
        assert_eq!(variants.len(), 14);
        let added = variants.pop().unwrap();
        assert_eq!(added["rgb"], expected_rgb);
        assert_eq!(added["mask"], "1".repeat(320));
        assert_eq!(added["tolerance"], 0);
        assert_eq!(
            added["source_sha256"],
            "445eaa6bbec37a98370791117111e453c34e8ac73d4d156eb1a587c05bf7da41"
        );
    }
    assert_eq!(restored, prior);
}


/// The extra appearance cannot turn blank paper, missing paper or mismatched colours into JD.
#[test]
fn native_v22_retains_paper_shape_and_exclusive_colour_refusals() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    for colour in [[255, 255, 255], [0, 128, 0], [0, 0, 0]] {
        let mut bad = original();
        for y in 302..318 {
            for x in 594..614 {
                let offset = y * bad.stride + x * 4;
                bad.pixels[offset..offset + 3].copy_from_slice(&colour);
            }
        }
        assert!(reader.observe(&bad).is_err());
    }
    let mut black_suit = original();
    for y in 302..318 {
        for x in 594..614 {
            let offset = y * black_suit.stride + x * 4;
            if black_suit.pixels[offset..offset + 3]
                .iter()
                .copied()
                .min()
                .unwrap()
                < 128
            {
                black_suit.pixels[offset..offset + 3].copy_from_slice(&[0, 0, 0]);
            }
        }
    }
    assert!(reader.observe(&black_suit).is_err());
}


/// Check the new production profile against every exposed earlier direct label in both banks.
#[test]
fn native_v22_preserves_all_423_prior_direct_labels_in_both_banks() {
    let corpus: Value = serde_json::from_slice(&fixture("native-v12", "corpus.json")).unwrap();
    let reader = ReconVision::bundled_native_v22().unwrap();
    let mut checked = 0;
    for source in corpus["sources"].as_array().unwrap() {
        let frame =
            crate::capture::decode_png(&fixture("native-v12", source["file"].as_str().unwrap()))
                .unwrap();
        let view = frame.as_view().unwrap();
        for observation in corpus["observations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["source_id"] == source["id"])
        {
            assert_eq!(observation["source_sha256"], source["sha256"]);
            assert_eq!(observation["recognition_authority"], false);
            let origin = (
                integer(&observation["origin"][0]).unwrap(),
                integer(&observation["origin"][1]).unwrap(),
            );
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, origin).unwrap(),
                    Some(labelled_card(observation)),
                    "{}",
                    observation["id"]
                );
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 423);
}
