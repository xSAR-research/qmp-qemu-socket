//! Native evaluation originals and explicitly synthetic refusal derivatives.
use super::*;
const FC01: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC01.png");
const FC01_LABELS: [&[&str]; 8] = [
    &["JD", "TC", "8C", "7H", "8H", "KS", "3S"],
    &["4S", "9C", "6S", "JS", "JH", "6C", "KH"],
    &["2D", "AD", "5H", "8S", "9S", "4H", "9D"],
    &["3C", "2S", "JC", "KC", "9H", "AC", "QH"],
    &["TS", "7C", "2H", "TH", "QC", "QS"],
    &["KD", "7S", "2C", "5S", "4D", "QD"],
    &["8D", "3H", "6D", "AS", "3D", "AH"],
    &["5D", "5C", "4C", "7D", "6H", "TD"],
];
fn metadata() -> Metadata {
    Metadata::new("fixture", "native-freecell-v1", "retained-native").unwrap()
}
#[test]
fn freecell_300_originals_read_all_printed_physical_identities() {
    for (png, labels) in [(FC01, FC01_LABELS), (REFERENCE, LABELS)] {
        let frame = decode_png(png).unwrap();
        let deal = recognise(&frame, metadata()).unwrap();
        for (stack, expected) in deal.board().cascades().iter().zip(labels) {
            assert_eq!(
                stack.iter().collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|code| label(code).unwrap())
                    .collect::<Vec<_>>()
            );
        }
    }
}
fn paint(frame: &mut CapturedFrame, x: u32, y: u32, w: u32, h: u32, rgb: [u8; 3]) {
    for yy in y..y + h {
        for xx in x..x + w {
            let i = (yy as usize * frame.stride) + xx as usize * 4;
            frame.pixels[i..i + 3].copy_from_slice(&rgb);
        }
    }
}
#[test]
fn freecell_300_missing_unknown_mixed_duplicate_and_scene_refuse() {
    let original = decode_png(FC01).unwrap();
    for (x, y, w, h, rgb) in [
        (220, 338, 26, 23, [255, 255, 255]),
        (223, 361, 18, 16, [255, 255, 255]),
        (166, 112, 1, 1, [0, 0, 0]),
        (825, 36, 1, 1, [0, 0, 0]),
        (218, 835, 1, 1, [255, 255, 255]),
        (226, 331, 1, 1, [255, 255, 255]),
        (220, 338, 26, 23, [170, 30, 190]),
    ] {
        let mut frame = original.clone();
        paint(&mut frame, x, y, w, h, rgb);
        assert!(recognise(&frame, metadata()).is_err(), "{x},{y}");
    }
    let mut duplicate = original.clone();
    for (dx, dy, w, h) in [(2, 7, 26, 23), (5, 30, 18, 16)] {
        for yy in 0..h {
            for xx in 0..w {
                let from =
                    (331 + dy + yy) as usize * original.stride + (218 + dx + xx) as usize * 4;
                let to = (384 + dy + yy) as usize * duplicate.stride + (218 + dx + xx) as usize * 4;
                duplicate.pixels[to..to + 4].copy_from_slice(&original.pixels[from..from + 4]);
            }
        }
    }
    assert_eq!(
        recognise(&duplicate, metadata()).unwrap_err().kind,
        FailureKind::DuplicateIdentity
    );
    let mut malformed = original;
    malformed.width = 1919;
    assert!(recognise(&malformed, metadata()).is_err());
}


/// Every directly read identity and each positive empty-slot/stack-boundary proof is necessary.
#[test]
fn freecell_300_every_identity_and_empty_slot_is_required() {
    let original = decode_png(FC01).unwrap();
    for (column, (&x, n)) in PLAY_X.iter().zip(INITIAL_LENGTHS).enumerate() {
        for (depth, &y) in ROW_Y.iter().take(n).enumerate() {
            for (dx, dy, w, h) in [(2, 7, 26, 23), (5, 30, 18, 16)] {
                let mut damaged = original.clone();
                paint(&mut damaged, x + dx, y + dy, w, h, [255, 255, 255]);
                assert!(
                    recognise(&damaged, metadata()).is_err(),
                    "column {column}, depth {depth}, kind {dx}"
                );
            }
        }
    }
    for x in CELL_X.into_iter().chain(FOUNDATION_X) {
        let mut damaged = original.clone();
        paint(&mut damaged, x + 60, 160, 4, 4, [255, 255, 255]);
        assert!(
            recognise(&damaged, metadata()).is_err(),
            "occupied/missing positive top proof at {x}"
        );
    }
    for (x, n) in PLAY_X.into_iter().zip(INITIAL_LENGTHS) {
        let mut damaged = original.clone();
        paint(
            &mut damaged,
            x + 60,
            if n == 7 { 850 } else { 800 },
            4,
            4,
            [255, 255, 255],
        );
        assert!(recognise(&damaged, metadata()).is_err(), "extra row at {x}");
    }
}

/// Independent complete native faces exercise typography beyond the exposed-strip training bank.
#[test]
fn freecell_300_native_full_faces_keep_rank_and_suit_competition() {
    let reference = decode_png(REFERENCE).unwrap();
    let mut bank = Vec::new();
    for (column, (&x, labels)) in PLAY_X.iter().zip(LABELS).enumerate() {
        for (depth, code) in labels.iter().enumerate().take(INITIAL_LENGTHS[column] - 1) {
            bank.push(Template {
                card: label(code).unwrap(),
                rank: glyph(&reference, x, ROW_Y[depth], true).unwrap(),
                suit: glyph(&reference, x, ROW_Y[depth], false).unwrap(),
            });
        }
    }
    for (png, x, y, expected) in [
        (
            include_bytes!("../../tests/fixtures/freecell-FC05.png").as_slice(),
            795,
            437,
            "JH",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC05.png").as_slice(),
            1180,
            384,
            "7S",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC08.png").as_slice(),
            410,
            437,
            "6S",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC18.png").as_slice(),
            410,
            543,
            "4S",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC18.png").as_slice(),
            1373,
            543,
            "4H",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC20.png").as_slice(),
            603,
            331,
            "7H",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC21.png").as_slice(),
            603,
            437,
            "JH",
        ),
        // These three are full FOUNDATION faces; no native PLAY-bottom Two is claimed.
        (
            include_bytes!("../../tests/fixtures/freecell-FC14.png").as_slice(),
            1425,
            112,
            "2D",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC18.png").as_slice(),
            1233,
            112,
            "2C",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC18.png").as_slice(),
            1618,
            112,
            "2S",
        ),
    ] {
        let frame = decode_png(png).unwrap();
        let card = label(expected).unwrap();
        assert_eq!(
            identify(
                &glyph_variants(&frame, x, y, true).unwrap(),
                &bank,
                |t| (t.card.rank(), &t.rank),
                48,
                20
            )
            .unwrap(),
            card.rank()
        );
        assert_eq!(
            identify(
                &glyph_variants(&frame, x, y, false).unwrap(),
                &bank,
                |t| (t.card.suit(), &t.suit),
                16,
                12
            )
            .unwrap(),
            card.suit()
        );
    }
}

/// A readable corner cannot replace the required complete bottom face and outer rails.
#[test]
fn freecell_300_clipped_final_faces_and_rails_refuse_with_physical_evidence() {
    let original = decode_png(FC01).unwrap();
    for (column, (&x, n)) in PLAY_X.iter().zip(INITIAL_LENGTHS).enumerate() {
        let depth = n - 1;
        let top = ROW_Y[depth];
        let bottom = if n == 7 { 832 } else { 779 };
        for (region, xx, yy, w, h) in [
            (
                Region::BottomEdge,
                x,
                top + 50,
                CARD_WIDTH,
                bottom - top - 49,
            ),
            (Region::SideEdges, x, top + 55, 1, 4),
            (Region::SideEdges, x + CARD_WIDTH - 1, top + 55, 1, 4),
        ] {
            let mut cut = original.clone();
            paint(&mut cut, xx, yy, w, h, [20, 100, 20]);
            let error = recognise(&cut, metadata()).unwrap_err();
            assert_eq!(error.kind, FailureKind::InitialGeometry);
            assert_eq!(error.place, Place::Cascade { column, depth });
            assert_eq!(error.region, region);
            assert!(error.bounds.is_some());
            assert_eq!(error.profile, PROFILE);
        }
    }
}

/// Cancellation is checked between bounded regions/cards and never invents an identity.
#[test]
fn freecell_300_between_unit_cancellation_and_colour_conflicts_are_typed() {
    let original = decode_png(FC01).unwrap();
    let mut count = 0;
    recognise_checked(&original, metadata(), &mut || {
        count += 1;
        Ok(())
    })
    .unwrap();
    assert!(count > 52 * 2);
    for stop_at in [1, 70, count - 1] {
        let mut visited = 0;
        let error = recognise_checked(&original, metadata(), &mut || {
            visited += 1;
            if visited == stop_at {
                Err("reserved context revoked".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(visited, stop_at);
        assert_eq!(error.kind, FailureKind::Revoked);
        assert_eq!(error.cause, "reserved context revoked");
    }
    let mut conflict = original.clone();
    // Genuine black 3S suit replaces only the red JD suit; all rank/scene evidence is untouched.
    for y in 0..16 {
        for x in 0..18 {
            let from = (650 + 30 + y) * original.stride + (218 + 5 + x) * 4;
            let to = (331 + 30 + y) * conflict.stride + (218 + 5 + x) * 4;
            conflict.pixels[to..to + 4].copy_from_slice(&original.pixels[from..from + 4]);
        }
    }
    let error = recognise(&conflict, metadata()).unwrap_err();
    assert_eq!(error.kind, FailureKind::ColourDisagreement);
    assert_eq!(
        error.place,
        Place::Cascade {
            column: 0,
            depth: 0
        }
    );
    assert_eq!(error.region, Region::Suit);
    assert!(error.to_string().contains("PLAY 1 depth 1"));
    let mut slot = original;
    paint(&mut slot, CELL_X[2], 112, 1, 1, [0, 0, 0]);
    let error = recognise(&slot, metadata()).unwrap_err();
    assert_eq!(error.kind, FailureKind::TopSlotNotEmpty);
    assert_eq!(error.place, Place::Cell(2));
    assert_eq!(error.region, Region::TopSlot);
}
