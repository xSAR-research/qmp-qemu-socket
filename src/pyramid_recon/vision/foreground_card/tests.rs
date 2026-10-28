//! Complete exposed-card evidence, native raster families and input-free corruptions.

use super::super::{ReconScene, StockFace, TABLEAU_ORIGINS};
use super::*;
use crate::capture::{CapturedFrame, decode_png};
use serde_json::Value;
use std::path::PathBuf;


/// Load one sealed unchanged image; this never captures a live guest.
fn fixture(name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-146")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read independently visual labels, without synthesising a complete hidden deck.
fn labels() -> Value {
    serde_json::from_slice(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recognition-146/labels.json"),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Resolve the explicit visible label rather than infer it from deck membership.
fn card(value: &Value) -> Card {
    Card::new(
        Rank::ALL
            .into_iter()
            .find(|rank| value["rank"] == rank.as_str())
            .unwrap(),
        Suit::ALL
            .into_iter()
            .find(|suit| value["suit"] == suit.as_str())
            .unwrap(),
    )
}


/// Every exposed identity and complete board must pass the actual native reader.
#[test]
fn native_foreground_reads_all_three_whole_boards_and_87_visible_cards() {
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let labels = labels();
    let mut checked = 0;
    for board in labels["boards"].as_array().unwrap() {
        let frame = fixture(board["fixture"].as_str().unwrap());
        let ReconScene::Board(observed) = vision.observe(&frame).unwrap() else {
            panic!("native whole board");
        };
        for value in board["cards"].as_array().unwrap() {
            let expected = card(value);
            let origin = (
                value["origin"][0].as_u64().unwrap() as u32,
                value["origin"][1].as_u64().unwrap() as u32,
            );
            for bank in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    bank.recognise(frame.as_view().unwrap(), origin).unwrap(),
                    Some(expected),
                    "{} {}",
                    board["id"],
                    value["location"]
                );
            }
            if let Some(index) = value["tableau_index"].as_u64() {
                assert_eq!(observed.tableau[index as usize], expected);
            } else {
                assert_eq!(observed.left, StockFace::Card(expected));
            }
            checked += 1;
        }
        // This is the configured low-paper classification, not inferred face-down contents.
        assert_eq!(observed.right, StockFace::Empty);
    }
    assert_eq!(checked, 87);
}


/// Preserve every earlier independent direct label through both actual native banks.
#[test]
fn native_foreground_preserves_all_423_classic_labels_in_both_banks() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-recon/native-v12");
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(base.join("corpus.json")).unwrap()).unwrap();
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let mut checked = 0;
    for source in corpus["sources"].as_array().unwrap() {
        let frame =
            decode_png(&std::fs::read(base.join(source["file"].as_str().unwrap())).unwrap())
                .unwrap();
        for value in corpus["observations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|value| value["source_id"] == source["id"])
        {
            let expected = card(value);
            let origin = (
                value["origin"][0].as_u64().unwrap() as u32,
                value["origin"][1].as_u64().unwrap() as u32,
            );
            for bank in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    bank.recognise(frame.as_view().unwrap(), origin).unwrap(),
                    Some(expected),
                    "{}",
                    value["id"]
                );
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 423);
}


/// An intact lower-corner suit remains distinct from adjacent King illustration ink.
#[test]
fn native_foreground_keeps_complete_suits_separate_from_body_art() {
    let frame = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-recon/native-redeal/exhausted.png"
    ))
    .unwrap();
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    assert_eq!(
        vision
            .tableau_reader
            .recognise(frame.as_view().unwrap(), TABLEAU_ORIGINS[25])
            .unwrap(),
        Some(Card::new(Rank::King, Suit::Diamonds))
    );
    // The real Retro DL diamond includes its final antialias pixel at card row50.
    // Keeping this complete face guards against clipping the label to exclude art.
    let retro = fixture("retro.png");
    assert_eq!(
        vision
            .stock_reader
            .recognise(retro.as_view().unwrap(), (759, 678))
            .unwrap(),
        Some(Card::new(Rank::King, Suit::Diamonds))
    );
}


/// Sparse mass comparison must equal the independent complete-raster definition.
#[test]
fn native_foreground_sparse_distance_preserves_every_translated_negative_space() {
    let shapes = [
        vec![(0, 0, 255), (1, 0, 130), (0, 1, 80), (2, 3, 255)],
        vec![
            (0, 0, 255),
            (2, 0, 220),
            (1, 1, 180),
            (0, 2, 255),
            (2, 2, 255),
        ],
        vec![
            (0, 0, 35),
            (1, 0, 60),
            (2, 1, 100),
            (1, 3, 255),
            (2, 3, 150),
        ],
    ];
    let glyphs: Vec<_> = shapes
        .iter()
        .map(|shape| {
            Glyph::from_points(
                &shape
                    .iter()
                    .map(|&(x, y, ink)| Point {
                        x,
                        y,
                        ink,
                        strong: true,
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        })
        .collect();
    for actual in &glyphs {
        for expected in &glyphs {
            let mut dense = Distance {
                difference: 1,
                mass: 1,
            };
            for dy in SHIFTS {
                for dx in SHIFTS {
                    let mut score = Distance {
                        difference: 0,
                        mass: 0,
                    };
                    for y in -2..actual.height.max(expected.height) as i32 + 2 {
                        for x in -2..actual.width.max(expected.width) as i32 + 2 {
                            let a = actual.sample(x, y) * 4;
                            let b = expected.translated(x, y, dx, dy);
                            score.difference += a.abs_diff(b);
                            score.mass += a.max(b);
                        }
                    }
                    if score.less(dense) {
                        dense = score;
                    }
                }
            }
            let sparse = distance(actual, expected);
            assert_eq!(
                u128::from(sparse.difference) * u128::from(dense.mass),
                u128::from(dense.difference) * u128::from(sparse.mass)
            );
        }
    }
}


/// Change a synthetic RGBA8 region without altering fixture bytes or claiming capture.
fn fill(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    assert_eq!(frame.stride, frame.width as usize * 4);
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let at = y as usize * frame.stride + x as usize * 4;
            frame.pixels[at..at + 3].copy_from_slice(&colour);
        }
    }
}


/// Retain all thirteen labels, exact raw pixels and sixteen whole-card held-outs.
#[test]
fn native_foreground_calibration_uses_only_13_measured_rank_cards() {
    let data: Value = serde_json::from_str(include_str!(
        "../../../../assets/pyramid-glyphs/retro-corners-v1.json"
    ))
    .unwrap();
    let labels = labels();
    let split = &labels["retro_split"];
    let frame = fixture("retro.png");
    let view = frame.as_view().unwrap();
    let held = split["held_out_locations"].as_array().unwrap();
    assert_eq!(held.len(), 16);
    let mut seen = std::collections::BTreeSet::new();
    for entry in data["corners"].as_array().unwrap() {
        assert!(!held.contains(&entry["location"]));
        assert!(seen.insert(entry["rank"].as_str().unwrap()));
        let x = entry["origin"][0].as_u64().unwrap() as u32;
        let y = entry["origin"][1].as_u64().unwrap() as u32;
        let text = entry["rgb"].as_str().unwrap();
        let raw: Vec<_> = text
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                super::super::hex(pair[0]).unwrap() * 16 + super::super::hex(pair[1]).unwrap()
            })
            .collect();
        let actual: Vec<_> = (y + 4..y + 53)
            .flat_map(|row| {
                (x + 3..x + 34).flat_map(move |column| view.pixel_rgb(column, row).unwrap())
            })
            .collect();
        assert_eq!(raw, actual);
    }
    assert_eq!(seen.len(), 13);
}


/// Serialized profiles retain frozen v2; naming a private native ID is insufficient.
#[test]
fn native_foreground_cannot_be_enabled_by_a_custom_profile_identifier() {
    let mut data: Value = serde_json::from_slice(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v22/profile.json"),
        )
        .unwrap(),
    )
    .unwrap();
    data["profile_id"] = Value::String("pyramid-native-1920-classic-v23".to_owned());
    let custom =
        super::super::ReconVision::from_bytes(&serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(custom.tableau_reader.foreground.is_none());
    let frame = fixture("cream_first.png");
    assert_eq!(
        custom
            .tableau_reader
            .recognise(frame.as_view().unwrap(), (890, 112))
            .unwrap(),
        None
    );
}


/// Missing or contradictory ink must not be replaced by a guessed nearest card.
#[test]
fn native_foreground_refuses_blank_conflicting_clipped_and_unknown_glyphs() {
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let original = fixture("retro.png");
    let origin = (890u32, 112u32);
    let paper = [250, 248, 235];
    let rank = PixelRect::new(origin.0 + 3, origin.1 + 4, 26, 28);
    let suit = PixelRect::new(origin.0 + 4, origin.1 + 29, 22, 23);
    for (bounds, colour) in [
        (rank, paper),
        (suit, paper),
        (
            PixelRect::new(origin.0 + 3, origin.1 + 4, 31, 49),
            [24, 60, 30],
        ),
        (PixelRect::new(origin.0 + 3, origin.1 + 4, 31, 49), [12; 3]),
    ] {
        let mut changed = original.clone();
        fill(&mut changed, bounds, colour);
        assert_eq!(
            vision
                .tableau_reader
                .recognise(changed.as_view().unwrap(), origin)
                .unwrap(),
            None
        );
    }
    let mut mixed_ink = original.clone();
    fill(&mut mixed_ink, PixelRect::new(894, 121, 2, 4), [0; 3]);
    assert_eq!(
        vision
            .tableau_reader
            .recognise(mixed_ink.as_view().unwrap(), origin)
            .unwrap(),
        None
    );
    let mut rectangle = original.clone();
    fill(&mut rectangle, rank, paper);
    fill(
        &mut rectangle,
        PixelRect::new(origin.0 + 8, origin.1 + 8, 13, 19),
        [230, 0, 0],
    );
    assert_eq!(
        vision
            .tableau_reader
            .recognise(rectangle.as_view().unwrap(), origin)
            .unwrap(),
        None
    );
    let mut conflict = original.clone();
    for y in suit.y..suit.y + suit.height {
        for x in suit.x..suit.x + suit.width {
            let at = y as usize * conflict.stride + x as usize * 4;
            if conflict.pixels[at] >= 120
                && conflict.pixels[at].saturating_sub(conflict.pixels[at + 1]) >= 24
            {
                conflict.pixels[at..at + 3].copy_from_slice(&[0; 3]);
            }
        }
    }
    assert_eq!(
        vision
            .tableau_reader
            .recognise(conflict.as_view().unwrap(), origin)
            .unwrap(),
        None
    );
    let view = original.as_view().unwrap();
    for position in [(890u32, 112u32), (990, 272), (1091, 218)] {
        let pixels: Vec<_> = (position.1 + 4..position.1 + 53)
            .flat_map(|y| {
                (position.0 + 3..position.0 + 34).map(move |x| view.pixel_rgb(x, y).unwrap())
            })
            .collect();
        let shapes = corners(&pixels, None, None);
        let max_y = components(&pixels, None)
            .into_iter()
            .flat_map(|part| part.points)
            .filter(|point| point.y >= 33 && point.x < 26)
            .map(|point| point.y)
            .max()
            .unwrap();
        assert!(!shapes.is_empty());
        let mut clipped = original.clone();
        fill(
            &mut clipped,
            PixelRect::new(position.0 + 4, position.1 + u32::from(max_y) - 2, 22, 3),
            paper,
        );
        assert_eq!(
            vision
                .tableau_reader
                .recognise(clipped.as_view().unwrap(), position)
                .unwrap(),
            None,
            "clipped suit {position:?}"
        );
    }
    let mut clipped_queen = original.clone();
    fill(&mut clipped_queen, PixelRect::new(893, 247, 26, 3), paper);
    assert_eq!(
        vision
            .tableau_reader
            .recognise(clipped_queen.as_view().unwrap(), (890, 218))
            .unwrap(),
        None,
        "complete retro Queen tail required"
    );
    let mut missing_one = original.clone();
    fill(&mut missing_one, PixelRect::new(1395, 382, 8, 28), paper);
    assert_eq!(
        vision
            .tableau_reader
            .recognise(missing_one.as_view().unwrap(), (1392, 378))
            .unwrap(),
        None,
        "Ten first digit required"
    );
    assert!(
        vision
            .tableau_reader
            .recognise(original.as_view().unwrap(), (1919, 1079))
            .is_err()
    );
}


/// Genuine replacement ink must report the donor value, independent of a deck guess.
#[test]
fn native_foreground_reads_intact_changed_labels_instead_of_inferring_the_old_card() {
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let original = fixture("retro.png");
    let mut changed = original.clone();
    let target = (890usize, 112usize);
    let donor = (790usize, 272usize);
    for y in 4..29 {
        for x in 3..29 {
            let from = (donor.1 + y) * original.stride + (donor.0 + x) * 4;
            let to = (target.1 + y) * changed.stride + (target.0 + x) * 4;
            changed.pixels[to..to + 3].copy_from_slice(&original.pixels[from..from + 3]);
        }
    }
    assert_eq!(
        vision
            .tableau_reader
            .recognise(changed.as_view().unwrap(), (890, 112))
            .unwrap(),
        Some(Card::new(Rank::Two, Suit::Hearts))
    );
    let mut changed = original.clone();
    let donor = (990usize, 165usize);
    for y in 32..53 {
        for x in 4..26 {
            let from = (donor.1 + y) * original.stride + (donor.0 + x) * 4;
            let to = (target.1 + y) * changed.stride + (target.0 + x) * 4;
            changed.pixels[to..to + 3].copy_from_slice(&original.pixels[from..from + 3]);
        }
    }
    assert_eq!(
        vision
            .tableau_reader
            .recognise(changed.as_view().unwrap(), (890, 112))
            .unwrap(),
        Some(Card::new(Rank::Three, Suit::Diamonds))
    );
}


/// A differently labelled competitor cannot be hidden by a native-size prefilter.
#[test]
fn native_foreground_competitors_survive_geometry_until_after_label_selection() {
    let actual = Glyph::from_points(&[
        Point {
            x: 4,
            y: 4,
            ink: 255,
            strong: true,
        },
        Point {
            x: 5,
            y: 5,
            ink: 255,
            strong: true,
        },
    ])
    .unwrap();
    let mut wider = actual.clone();
    wider.width += 3;
    wider.ink = vec![0; wider.width * wider.height];
    for &(x, y, ink) in &actual.nonzero {
        wider.ink[y as usize * wider.width + x as usize] = (ink / 4) as u8;
    }
    let other = Glyph::from_points(&[
        Point {
            x: 4,
            y: 4,
            ink: 255,
            strong: true,
        },
        Point {
            x: 5,
            y: 4,
            ink: 255,
            strong: true,
        },
        Point {
            x: 4,
            y: 5,
            ink: 255,
            strong: true,
        },
        Point {
            x: 5,
            y: 5,
            ink: 255,
            strong: true,
        },
    ])
    .unwrap();
    let bank = vec![
        (Rank::Two, vec![actual.clone()]),
        (Rank::Three, vec![wider]),
        (Rank::Four, vec![other]),
    ];
    assert!(matching_label(&actual, &bank, 220_000, 80_000, true).is_none());
}


/// External artwork changes cannot choose a font, label, paper colour or card value.
#[test]
fn native_foreground_identity_ignores_undefined_artwork_outside_complete_cards() {
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let labels = labels();
    let image = fixture("cream_first.png");
    let board = labels["boards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|board| board["fixture"] == "cream_first.png")
        .unwrap();
    for palette in [
        [[18, 22, 27], [120, 35, 140]],
        [[245, 245, 245], [35, 190, 115]],
    ] {
        let mut changed = image.clone();
        for y in 0..image.height {
            for x in 0..image.width {
                let inside = TABLEAU_ORIGINS
                    .iter()
                    .chain(std::iter::once(&(759, 678)))
                    .any(|&(left, top)| x >= left && x < left + 140 && y >= top && y < top + 187);
                if !inside {
                    let at = y as usize * changed.stride + x as usize * 4;
                    let colour = palette[((x / 13 + y / 17) % 2) as usize];
                    changed.pixels[at..at + 3].copy_from_slice(&colour);
                }
            }
        }
        for value in board["cards"].as_array().unwrap() {
            let origin = (
                value["origin"][0].as_u64().unwrap() as u32,
                value["origin"][1].as_u64().unwrap() as u32,
            );
            assert_eq!(
                vision
                    .tableau_reader
                    .recognise(changed.as_view().unwrap(), origin)
                    .unwrap(),
                Some(card(value))
            );
        }
    }
}


/// Refusal diagnostics retain same-frame extraction and competition measurements.
#[test]
fn native_foreground_refusal_details_explain_the_actual_failed_guard() {
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let original = fixture("retro.png");
    let mut changed = original.clone();
    fill(
        &mut changed,
        PixelRect::new(893, 116, 26, 28),
        [250, 248, 235],
    );
    let details = vision
        .tableau_reader
        .failure_details(changed.as_view().unwrap(), (890, 112));
    assert!(details.contains("extraction rejected"), "{details}");
    fill(&mut changed, PixelRect::new(898, 120, 13, 19), [230, 0, 0]);
    let details = vision
        .tableau_reader
        .failure_details(changed.as_view().unwrap(), (890, 112));
    for expected in [
        "paper_reference=",
        "family=classic",
        "rank best=",
        "error=",
        "quality=",
        "gap=",
        "native_size=",
    ] {
        assert!(details.contains(expected), "{expected}: {details}");
    }
}


/// Both actual Wild West boards retain every visible identity and occupied stock.
#[test]
fn native_149_wild_west_reads_two_complete_boards_and_58_faces() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-wild-west-149");
    let labels: Value =
        serde_json::from_slice(&std::fs::read(base.join("labels.json")).unwrap()).unwrap();
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let mut count = 0;
    for board in labels["boards"].as_array().unwrap() {
        let frame =
            decode_png(&std::fs::read(base.join(board["fixture"].as_str().unwrap())).unwrap())
                .unwrap();
        let ReconScene::Board(observed) = vision.observe(&frame).unwrap() else {
            panic!("supported Wild West board");
        };
        for value in board["cards"].as_array().unwrap() {
            let expected = card(value);
            let origin = (
                value["origin"][0].as_u64().unwrap() as u32,
                value["origin"][1].as_u64().unwrap() as u32,
            );
            for bank in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    bank.recognise(frame.as_view().unwrap(), origin).unwrap(),
                    Some(expected),
                    "{} {}",
                    board["id"],
                    value["location"]
                );
            }
            if let Some(index) = value["tableau_index"].as_u64() {
                assert_eq!(observed.tableau[index as usize], expected);
            } else {
                assert_eq!(observed.left, StockFace::Card(expected));
            }
            count += 1;
        }
        assert_eq!(observed.right, StockFace::Empty);
    }
    assert_eq!(count, 58);
}


/// Calibration uses each observed rank/ink-colour pair once and preserves real holdouts.
#[test]
fn native_149_western_calibration_preserves_33_whole_card_holdouts() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-wild-west-149");
    let labels: Value =
        serde_json::from_slice(&std::fs::read(base.join("labels.json")).unwrap()).unwrap();
    let bank: Value = serde_json::from_str(include_str!(
        "../../../../assets/pyramid-glyphs/wild-west-corners-v1.json"
    ))
    .unwrap();
    let entries = bank["corners"].as_array().unwrap();
    assert_eq!(entries.len(), 25);
    let mut pairs = std::collections::BTreeSet::new();
    let mut heldouts = 0;
    for board in labels["boards"].as_array().unwrap() {
        let frame =
            decode_png(&std::fs::read(base.join(board["fixture"].as_str().unwrap())).unwrap())
                .unwrap();
        for value in board["cards"].as_array().unwrap() {
            let entry = entries.iter().find(|entry| {
                entry["fixture"] == board["fixture"] && entry["location"] == value["location"]
            });
            let Some(entry) = entry else {
                heldouts += 1;
                continue;
            };
            assert_eq!(entry["rank"], value["rank"]);
            assert_eq!(entry["suit"], value["suit"]);
            assert!(pairs.insert((
                value["rank"].as_str().unwrap().to_owned(),
                red_suit(card(value).suit())
            )));
            let origin = (
                value["origin"][0].as_u64().unwrap() as u32,
                value["origin"][1].as_u64().unwrap() as u32,
            );
            let actual = corner_pixels(frame.as_view().unwrap(), origin).unwrap();
            let hex: String = actual
                .iter()
                .flatten()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            assert_eq!(entry["rgb"].as_str().unwrap(), hex);
        }
    }
    assert_eq!(heldouts, 33);
    assert_eq!(pairs.len(), 25);
    assert_eq!(
        pairs
            .iter()
            .map(|(rank, _)| rank)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        13
    );
}


/// Missing complete labels remain unreadable, and outside artwork has no identity role.
#[test]
fn native_149_western_refuses_damaged_labels_and_ignores_outside_artwork() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-wild-west-149");
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    for name in ["old.png", "new.png"] {
        let frame = decode_png(&std::fs::read(base.join(name)).unwrap()).unwrap();
        let origin = TABLEAU_ORIGINS[0];
        let expected = vision
            .tableau_reader
            .recognise(frame.as_view().unwrap(), origin)
            .unwrap();
        assert!(expected.is_some());
        for (x, y, width, height) in [(3, 4, 21, 22), (3, 27, 16, 18)] {
            let mut changed = frame.clone();
            fill(
                &mut changed,
                PixelRect::new(origin.0 + x, origin.1 + y, width, height),
                [226, 198, 157],
            );
            assert_eq!(
                vision
                    .tableau_reader
                    .recognise(changed.as_view().unwrap(), origin)
                    .unwrap(),
                None,
                "{name} missing label"
            );
        }
        let mut changed = frame.clone();
        fill(
            &mut changed,
            PixelRect::new(0, 0, 1920, 1080),
            [70, 120, 160],
        );
        for y in origin.1 + 4..origin.1 + 53 {
            for x in origin.0 + 3..origin.0 + 34 {
                let at = y as usize * frame.stride + x as usize * 4;
                changed.pixels[at..at + 4].copy_from_slice(&frame.pixels[at..at + 4]);
            }
        }
        assert_eq!(
            vision
                .tableau_reader
                .recognise(changed.as_view().unwrap(), origin)
                .unwrap(),
            expected
        );
        assert_eq!(
            vision
                .tableau_reader
                .recognise(frame.as_view().unwrap(), (100, 600))
                .unwrap(),
            None,
            "{name} wood"
        );
    }
}


/// Aged material proves presence only; unreadable occupied stock never becomes Empty.
#[test]
fn native_149_aged_stock_keeps_unreadable_presence_and_checked_bounds() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-wild-west-149");
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    for name in ["old.png", "new.png"] {
        let mut frame = decode_png(&std::fs::read(base.join(name)).unwrap()).unwrap();
        let left = PixelRect::new(759, 678, 139, 187);
        let right = PixelRect::new(1022, 678, 139, 187);
        let count = super::super::card_paper::stock_pixels(frame.as_view().unwrap(), left).unwrap();
        assert!(count >= 11_697);
        assert_eq!(
            super::super::card_paper::stock_pixels(frame.as_view().unwrap(), right).unwrap(),
            0
        );
        assert!(
            super::super::card_paper::stock_pixels(
                frame.as_view().unwrap(),
                PixelRect::new(u32::MAX, 678, 139, 187)
            )
            .is_err()
        );
        fill(
            &mut frame,
            PixelRect::new(762, 682, 31, 49),
            [226, 198, 157],
        );
        assert!(
            vision.observe(&frame).is_err(),
            "{name} unreadable occupied LEFT"
        );
    }
}


/// Intact aged faces cannot prove a removal through ordinary or final-clear bands.
#[test]
fn native_149_intact_aged_faces_never_prove_absence() {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pyramid-wild-west-149");
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    for name in ["old.png", "new.png"] {
        let frame = decode_png(&std::fs::read(base.join(name)).unwrap()).unwrap();
        let view = frame.as_view().unwrap();
        let ReconScene::Board(observed) = vision.observe(&frame).unwrap() else {
            panic!("supported actual aged board");
        };
        let occupancy = vision.native_occupancy(&frame).unwrap();
        for (index, (x, y)) in TABLEAU_ORIGINS.into_iter().enumerate().skip(21) {
            let slot = crate::pyramid::PyramidTargetKind::Card {
                row: 7,
                column: u8::try_from(index - 20).unwrap(),
            }
            .slot_index()
            .unwrap();
            assert_ne!(
                occupancy[slot],
                crate::pyramid::FaceEvidence::Absent,
                "{name}: intact 7|{}",
                index - 20
            );
            for offset in [4, 164, 168] {
                assert!(
                    !vision
                        .native_absence_band(view, PixelRect::new(x + 40, y + offset, 60, 4))
                        .unwrap(),
                    "{name}: intact 7|{} at band {offset}",
                    index - 20
                );
            }
        }
        let mut falsely_removed = observed.tableau.map(Some);
        falsely_removed[21] = None;
        let StockFace::Card(left) = observed.left else {
            panic!("visible LEFT card")
        };
        let error = vision
            .verify_cards(
                &frame,
                view,
                &falsely_removed,
                Some(left),
                None,
                &vision.board.left_empty,
            )
            .unwrap_err();
        assert!(
            error.contains("execution slot 21 lacks positive absence"),
            "{error}"
        );
        let band = PixelRect::new(300, 900, 60, 4);
        let mut boundary = frame.clone();
        fill(
            &mut boundary,
            PixelRect::new(298, 898, 64, 8),
            [165, 148, 115],
        );
        fill(
            &mut boundary,
            PixelRect::new(300, 900, 12, 1),
            [195, 168, 128],
        );
        assert!(
            vision
                .native_absence_band(boundary.as_view().unwrap(), band)
                .unwrap()
        );
        fill(
            &mut boundary,
            PixelRect::new(300, 900, 13, 1),
            [195, 168, 128],
        );
        assert!(
            !vision
                .native_absence_band(boundary.as_view().unwrap(), band)
                .unwrap()
        );
        for invalid in [
            PixelRect::new(u32::MAX, 900, 60, 4),
            PixelRect::new(300, 900, 0, 4),
        ] {
            assert!(super::super::card_paper::aged_pixels(view, invalid).is_err());
        }
    }
}


/// Read the unchanged football failure frame without deriving cards from deck membership.
#[test]
fn native_foreground_reads_retained_football_board_and_29_visible_cards() {
    let frame = decode_png(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-stock-paper-145/football-entry.png"),
        )
        .unwrap(),
    )
    .unwrap();
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let expected = [
        (Rank::Ten, Suit::Diamonds),
        (Rank::Nine, Suit::Spades),
        (Rank::Eight, Suit::Clubs),
        (Rank::Five, Suit::Clubs),
        (Rank::Five, Suit::Hearts),
        (Rank::Five, Suit::Diamonds),
        (Rank::Five, Suit::Spades),
        (Rank::Three, Suit::Hearts),
        (Rank::Ace, Suit::Clubs),
        (Rank::Two, Suit::Hearts),
        (Rank::Nine, Suit::Clubs),
        (Rank::Nine, Suit::Diamonds),
        (Rank::Six, Suit::Diamonds),
        (Rank::Jack, Suit::Hearts),
        (Rank::Six, Suit::Clubs),
        (Rank::Ten, Suit::Spades),
        (Rank::Jack, Suit::Spades),
        (Rank::Two, Suit::Clubs),
        (Rank::Two, Suit::Diamonds),
        (Rank::Six, Suit::Hearts),
        (Rank::Queen, Suit::Spades),
        (Rank::King, Suit::Spades),
        (Rank::Ace, Suit::Diamonds),
        (Rank::Four, Suit::Spades),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Jack, Suit::Clubs),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Nine, Suit::Hearts),
    ]
    .map(|(rank, suit)| Card::new(rank, suit));
    let ReconScene::Board(observed) = vision.observe(&frame).unwrap() else {
        panic!("retained football scene must produce a supported board");
    };
    assert_eq!(observed.tableau, expected);
    let left = Card::new(Rank::Queen, Suit::Hearts);
    assert_eq!(observed.left, StockFace::Card(left));
    assert_eq!(observed.right, StockFace::Empty);
    let view = frame.as_view().unwrap();
    for (index, (&origin, &card)) in TABLEAU_ORIGINS.iter().zip(&expected).enumerate() {
        for bank in [&vision.tableau_reader, &vision.stock_reader] {
            assert_eq!(
                bank.recognise(view, origin).unwrap(),
                Some(card),
                "football tableau slot {index}"
            );
        }
    }
    let bounds = super::super::LEFT_BOUNDS;
    for bank in [&vision.tableau_reader, &vision.stock_reader] {
        assert_eq!(
            bank.recognise(view, (bounds.x, bounds.y)).unwrap(),
            Some(left),
            "football DL"
        );
    }
}


/// Missing football rank/suit ink must refuse the board instead of inferring its old card.
#[test]
fn native_foreground_refuses_blank_corner_in_retained_football_board() {
    let mut frame = decode_png(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-stock-paper-145/football-entry.png"),
        )
        .unwrap(),
    )
    .unwrap();
    let vision = super::super::ReconVision::bundled_native_v23().unwrap();
    let origin = TABLEAU_ORIGINS[1];
    fill(
        &mut frame,
        PixelRect::new(origin.0 + 3, origin.1 + 4, 31, 49),
        [237; 3],
    );
    for bank in [&vision.tableau_reader, &vision.stock_reader] {
        assert_eq!(
            bank.recognise(frame.as_view().unwrap(), origin).unwrap(),
            None,
            "football 2|1 has no readable rank/suit ink"
        );
    }
    assert!(
        vision.observe(&frame).is_err(),
        "blank football corner must block complete-board recognition"
    );
}


mod effect_155;


mod recognition_158;


mod effect_159;
