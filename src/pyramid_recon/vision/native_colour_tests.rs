//! Actual Xbox appearance and explicitly synthetic hue/occlusion regressions.
//!
//! These are read-only image proofs, never live acquisition/restoration evidence.

use super::*;
use crate::pyramid::FaceEvidence;


/// Actual textured Xbox stock exhaustion is empty despite the circle's sharp edges.
#[test]
fn native_colour_xbox_recon_exhaustion_preserves_inventory_and_empty_left() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let baseline =
        original("tests/fixtures/pyramid-colour-independence/xbox-recon-exhausted/baseline.png");
    let latest =
        original("tests/fixtures/pyramid-colour-independence/xbox-recon-exhausted/latest.png");
    let ReconScene::Board(before) = reader.observe(&baseline).unwrap() else {
        panic!("actual Xbox initial board");
    };
    let ReconScene::Board(after) = reader.observe(&latest).unwrap() else {
        panic!("actual Xbox exhausted stock");
    };
    assert_eq!(after.tableau, before.tableau);
    assert!(matches!(before.left, StockFace::Card(_)));
    assert_eq!(before.right, StockFace::Empty);
    assert_eq!(after.left, StockFace::Empty);
    assert_eq!(
        after.right,
        StockFace::Card(Card::new(Rank::Two, Suit::Spades))
    );
    assert!(
        after.undo_all.is_some(),
        "restoration availability is separately recognised"
    );
}


/// Low paper coverage is empty; an unreadable high-paper face remains unknown.
#[test]
fn native_colour_stock_absence_inverts_high_paper_coverage() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let frame =
        original("tests/fixtures/pyramid-colour-independence/xbox-recon-exhausted/latest.png");
    for (bounds, rgb) in [
        (PixelRect::new(771, 700, 2, 2), [255; 3]),
        (PixelRect::new(759, 678, 1, 1), [247, 245, 226]),
        (PixelRect::new(897, 864, 1, 1), [220; 3]),
    ] {
        let mut changed = frame.clone();
        paint(&mut changed, bounds, rgb);
        assert_eq!(
            reader
                .stock_face(
                    changed.as_view().unwrap(),
                    LEFT_BOUNDS,
                    &reader.board.left_empty
                )
                .unwrap(),
            StockFace::Empty,
            "{bounds:?}"
        );
    }
    let mut unreadable = frame;
    paint(&mut unreadable, LEFT_BOUNDS, [220; 3]);
    assert!(
        reader
            .stock_face(
                unreadable.as_view().unwrap(),
                LEFT_BOUNDS,
                &reader.board.left_empty
            )
            .is_err()
    );
}


/// Decode a retained original relative to the repository, preserving its pixels.
fn original(relative: &str) -> CapturedFrame {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap();
    crate::capture::decode_png(&bytes).unwrap()
}


/// Paint a labelled synthetic RGB rectangle with checked native coordinates.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let offset =
                usize::try_from(y).unwrap() * frame.stride + usize::try_from(x).unwrap() * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}


/// Recolour only measured green surface pixels, excluding supplied actual card rectangles.
/// This is a synthetic pixel derivative, not a screenshot of another guest theme.
fn recolour(mut frame: CapturedFrame, cards: &[PixelRect], rgb: [u8; 3]) -> CapturedFrame {
    // The fixed header/toolbar is UI, not a tableau-surface derivative.
    for y in 112..frame.height {
        for x in 0..frame.width {
            if cards.iter().any(|bounds| {
                x >= bounds.x
                    && y >= bounds.y
                    && x < bounds.x + bounds.width
                    && y < bounds.y + bounds.height
            }) {
                continue;
            }
            let offset =
                usize::try_from(y).unwrap() * frame.stride + usize::try_from(x).unwrap() * 4;
            let pixel = frame.pixels[offset..offset + 3].try_into().unwrap();
            if crate::pyramid::is_felt(pixel) {
                frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
            }
        }
    }
    frame
}


/// The genuine Xbox surface retains positive DRAW/legend proof and exclusive empty RIGHT.
/// Its visibly disabled Undo All must not become enabled through normalisation.
#[test]
fn native_colour_xbox_original_preserves_shapes_stock_and_disabled_control() {
    let frame = original("tests/fixtures/pyramid-colour-independence/xbox-original.png");
    let legacy = ReconVision::bundled_native_v22().unwrap();
    let reader = ReconVision::bundled_native_v23().unwrap();
    let view = frame.as_view().unwrap();
    assert!(!legacy.board_scene_matches(view).unwrap());
    assert!(reader.board_scene_matches(view).unwrap());
    assert_eq!(
        reader
            .stock_face(view, LEFT_BOUNDS, &reader.board.left_empty)
            .unwrap(),
        StockFace::Card(Card::new(Rank::Two, Suit::Clubs))
    );
    assert_eq!(
        reader
            .stock_face(view, RIGHT_BOUNDS, &reader.board.right_empty)
            .unwrap(),
        StockFace::Empty
    );
    assert!(!reader.native_undo(view).unwrap());
    let occupancy = reader.native_occupancy(&frame).unwrap();
    let highlighted = crate::pyramid::PyramidTargetKind::Card { row: 7, column: 3 }
        .slot_index()
        .unwrap();
    assert_eq!(occupancy[highlighted], FaceEvidence::Unknown);
    assert_eq!(
        crate::pyramid::tableau_evidence(&frame).unwrap()[highlighted],
        FaceEvidence::Unknown
    );
    let ReconScene::Board(board) = reader.observe(&frame).unwrap() else {
        panic!("actual intact Xbox board")
    };
    assert_eq!(board.tableau[23], Card::new(Rank::Jack, Suit::Hearts));
    assert_eq!(board.undo_all, None);
}


/// Multiple background hues leave complete card identities and scene geometry unchanged.
#[test]
fn native_colour_initial_recolours_preserve_all_identities_without_altering_card_banks() {
    let frame = original("tests/fixtures/pyramid-recon/native-v22/recon-jd.png");
    let legacy = ReconVision::bundled_native_v22().unwrap();
    let reader = ReconVision::bundled_native_v23().unwrap();
    let ReconScene::Board(expected) = legacy.observe(&frame).unwrap() else {
        panic!("actual board")
    };
    let mut protected: Vec<_> = TABLEAU_ORIGINS
        .iter()
        .map(|(x, y)| PixelRect::new(*x, *y, 140, 187))
        .collect();
    protected.push(LEFT_BOUNDS);
    for rgb in [[72, 28, 95], [24, 50, 110], [105, 30, 35], [90, 90, 90]] {
        let changed = recolour(frame.clone(), &protected, rgb);
        assert!(
            !legacy
                .board_scene_matches(changed.as_view().unwrap())
                .unwrap()
        );
        let ReconScene::Board(actual) = reader.observe(&changed).unwrap() else {
            panic!("synthetic hue {rgb:?}")
        };
        assert_eq!(actual.tableau, expected.tableau);
        assert_eq!(actual.left, expected.left);
        assert_eq!(actual.right, expected.right);
        assert_eq!(actual.undo_all, expected.undo_all);
    }
}


/// Return the human-read exact retained two-card position; indices stay fixed.
fn partial() -> ([Option<Card>; 28], Option<Card>) {
    let mut tableau = [None; 28];
    tableau[0] = Some(Card::new(Rank::King, Suit::Spades));
    tableau[2] = Some(Card::new(Rank::Eight, Suit::Spades));
    (tableau, Some(Card::new(Rank::Five, Suit::Hearts)))
}


/// Known empty tableau, exhausted X and complete widget one survive hue derivatives.
/// The optional returned point is image evidence only; no QMP input occurs here.
#[test]
fn native_colour_partial_recolours_preserve_exact_absence_and_control_count() {
    let frame = original("tests/fixtures/pyramid-exhausted-left-endgame/last-observed.png");
    let reader = ReconVision::bundled_native_v23().unwrap();
    let (tableau, right) = partial();
    let protected = [
        PixelRect::new(TABLEAU_ORIGINS[0].0, TABLEAU_ORIGINS[0].1, 140, 187),
        PixelRect::new(TABLEAU_ORIGINS[2].0, TABLEAU_ORIGINS[2].1, 140, 187),
        RIGHT_BOUNDS,
    ];
    for rgb in [[72, 28, 95], [24, 50, 110], [105, 30, 35], [90, 90, 90]] {
        let changed = recolour(frame.clone(), &protected, rgb);
        assert_eq!(
            reader
                .verify_native_stock_continuation(&changed, &tableau, None, right)
                .unwrap(),
            Some(PixelPoint::new(960, 825))
        );
        let mut wrong = tableau;
        wrong[2] = Some(Card::new(Rank::Eight, Suit::Hearts));
        assert!(
            reader
                .verify_native_stock_continuation(&changed, &wrong, None, right)
                .is_err()
        );
    }
}


/// Dark corruption and warm selected paper stay unknown rather than exposing parents.
#[test]
fn native_colour_absence_rejects_occlusions_and_possible_tinted_paper() {
    let frame = original("tests/fixtures/pyramid-exhausted-left-endgame/last-observed.png");
    let reader = ReconVision::bundled_native_v23().unwrap();
    let (tableau, right) = partial();
    let index = crate::pyramid::PyramidTargetKind::Card { row: 7, column: 3 }
        .slot_index()
        .unwrap();
    assert_eq!(
        reader.native_occupancy(&frame).unwrap()[index],
        FaceEvidence::Absent
    );
    for (bounds, rgb) in [
        (PixelRect::new(730, 610, 60, 4), [11, 13, 17]),
        (PixelRect::new(720, 600, 80, 24), [11, 13, 17]),
        (PixelRect::new(690, 432, 140, 187), [11, 13, 17]),
        (PixelRect::new(730, 610, 60, 4), [247, 245, 226]),
        (PixelRect::new(730, 610, 60, 4), [220, 220, 220]),
    ] {
        let mut changed = frame.clone();
        paint(&mut changed, bounds, rgb);
        assert_ne!(
            reader.native_occupancy(&changed).unwrap()[index],
            FaceEvidence::Absent,
            "{bounds:?} {rgb:?}"
        );
        assert!(
            reader
                .verify_native_stock_continuation(&changed, &tableau, None, right)
                .is_err()
        );
    }
}


/// Missing dynamic ink, the widget numeral or dimmed modal scene cannot grant authority.
#[test]
fn native_colour_requires_control_negative_space_and_complete_scene() {
    let frame = original("tests/fixtures/pyramid-colour-independence/xbox-original.png");
    let reader = ReconVision::bundled_native_v23().unwrap();
    let mut missing = frame.clone();
    paint(&mut missing, PixelRect::new(931, 695, 60, 50), [20, 40, 60]);
    assert!(
        !reader
            .board_scene_matches(missing.as_view().unwrap())
            .unwrap()
    );
    let mut modal = original("tests/fixtures/pyramid-recon/originals/gate3-undo-modal.png");
    assert!(matches!(
        reader.observe(&modal).unwrap(),
        ReconScene::UndoConfirmation { .. }
    ));
    for pattern in reader.dialog.signature.iter().skip(5) {
        paint(&mut modal, pattern.bounds, [8, 9, 12]);
    }
    assert!(matches!(
        reader.observe(&modal).unwrap(),
        ReconScene::UndoConfirmation { .. }
    ));
    paint(&mut modal, reader.dialog.signature[1].bounds, [8, 9, 12]);
    assert!(matches!(
        reader.observe(&modal).unwrap(),
        ReconScene::Unknown
    ));
    let mut wrong = original("tests/fixtures/pyramid-exhausted-left-endgame/last-observed.png");
    let (tableau, right) = partial();
    paint(&mut wrong, PixelRect::new(936, 837, 48, 18), [20, 40, 60]);
    assert!(
        reader
            .verify_native_redeal_position(&wrong, &tableau, right)
            .is_err()
    );
}


/// A custom identifier cannot opt into appearance, continuation or input capabilities.
#[test]
fn native_colour_custom_profile_name_does_not_enable_new_roles() {
    let mut profile: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
    ))
    .unwrap();
    profile["profile_id"] = Value::String("pyramid-native-1920-classic-v23".to_owned());
    let custom = ReconVision::from_bytes(&serde_json::to_vec(&profile).unwrap()).unwrap();
    assert!(!custom.colour_independent);
    assert!(custom.native_redeal.is_none());
    let frame = original("tests/fixtures/pyramid-colour-independence/xbox-original.png");
    assert!(matches!(
        custom.observe(&frame).unwrap(),
        ReconScene::Unknown
    ));
}


/// Bound full-clear evidence survives hue changes without borrowing a generic gold marker.
#[test]
fn native_colour_final_bands_and_bound_counter_survive_surface_recolours() {
    use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};
    let reader = ReconVision::bundled_native_v23().unwrap();
    for (name, bound) in [
        (
            "boards",
            ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap(),
        ),
        (
            "score",
            ChallengeProgress::new(ChallengeKind::Score, 0, 1400).unwrap(),
        ),
    ] {
        let frame = original(&format!(
            "tests/fixtures/pyramid-recon/native-v21/{name}-last-observed.png"
        ));
        let view = frame.as_view().unwrap();
        let pile = |bounds, pattern| match reader.stock_face(view, bounds, pattern).unwrap() {
            StockFace::Card(card) => Some(card),
            StockFace::Empty => None,
        };
        let left = pile(LEFT_BOUNDS, &reader.board.left_empty);
        let right = pile(RIGHT_BOUNDS, &reader.board.right_empty);
        let protected: Vec<_> = [(LEFT_BOUNDS, left), (RIGHT_BOUNDS, right)]
            .into_iter()
            .filter_map(|(bounds, card)| card.map(|_| bounds))
            .collect();
        for rgb in [[72, 28, 95], [24, 50, 110], [105, 30, 35], [90, 90, 90]] {
            let changed = recolour(frame.clone(), &protected, rgb);
            reader
                .verify_challenge_final_clear(&changed, &[None; 28], left, right, bound)
                .unwrap();
            let mut damaged = changed;
            paint(
                &mut damaged,
                PixelRect::new(930, 118, 60, 4),
                [247, 245, 226],
            );
            assert!(
                reader
                    .verify_challenge_final_clear(&damaged, &[None; 28], left, right, bound)
                    .is_err()
            );
        }
    }
}


/// The new genuine green Twos failure has an exact whole widget-one white shape.
/// Its RGB mismatch and independently recorded five-card position are retained unchanged.
#[test]
fn native_colour_twos_redeal_accepts_complete_widget_without_new_signature() {
    let frame = original("tests/fixtures/pyramid-colour-independence/twos-exhausted-green.png");
    let mut tableau = [None; 28];
    for (index, rank, suit) in [
        (0, Rank::Five, Suit::Clubs),
        (1, Rank::Nine, Suit::Hearts),
        (3, Rank::Five, Suit::Diamonds),
        (6, Rank::Eight, Suit::Hearts),
        (10, Rank::Eight, Suit::Spades),
    ] {
        tableau[index] = Some(Card::new(rank, suit));
    }
    let right = Some(Card::new(Rank::Ten, Suit::Hearts));
    let legacy = ReconVision::bundled_native_v22().unwrap();
    assert!(
        legacy
            .verify_native_redeal_position(&frame, &tableau, right)
            .is_err()
    );
    let reader = ReconVision::bundled_native_v23().unwrap();
    assert_eq!(reader.remaining_native_redeals(&frame).unwrap(), Some(1));
    assert_eq!(
        reader
            .verify_native_stock_continuation(&frame, &tableau, None, right)
            .unwrap(),
        Some(PixelPoint::new(960, 825))
    );
    assert_eq!(
        reader
            .verify_native_redeal_position(&frame, &tableau, right)
            .unwrap(),
        PixelPoint::new(960, 825)
    );
    let mut damaged = frame;
    paint(&mut damaged, PixelRect::new(946, 834, 24, 20), [30, 95, 65]);
    assert!(
        reader
            .verify_native_redeal_position(&damaged, &tableau, right)
            .is_err()
    );
}


/// Both empty piles and the complete enabled widget prove the retained circle layout.
/// Original pixels are unchanged; expected fixed identities are checked against its deal.
#[test]
fn native_colour_empty_circle_redeal_proves_both_piles_and_widget() {
    let frame = original("tests/fixtures/pyramid-empty-circle-redeal/last-observed.png");
    let mut tableau = [None; 28];
    tableau[0] = Some(Card::new(Rank::Nine, Suit::Clubs));
    tableau[2] = Some(Card::new(Rank::Four, Suit::Spades));
    for reader in [
        ReconVision::bundled_native_v22().unwrap(),
        ReconVision::bundled_native_v23().unwrap(),
    ] {
        assert!(
            reader
                .verify_position(&frame, &tableau, None, None)
                .is_err()
        );
        assert_eq!(
            reader
                .verify_native_stock_continuation(&frame, &tableau, None, None)
                .unwrap(),
            Some(PixelPoint::new(960, 825))
        );
        assert_eq!(
            reader
                .verify_native_redeal_position(&frame, &tableau, None)
                .unwrap(),
            PixelPoint::new(960, 825)
        );
        assert!(
            reader
                .verify_native_stock_continuation(
                    &frame,
                    &tableau,
                    None,
                    Some(Card::new(Rank::Ace, Suit::Hearts))
                )
                .is_err()
        );
        let mut wrong = tableau;
        wrong[2] = Some(Card::new(Rank::Four, Suit::Hearts));
        assert!(
            reader
                .verify_native_stock_continuation(&frame, &wrong, None, None)
                .is_err()
        );
    }
}


/// The circle cannot infer control availability from model emptiness or the lower caption.
/// Synthetic numeral, caption and scene mutations preserve every unrelated source pixel.
#[test]
fn native_colour_empty_circle_redeal_refuses_incomplete_or_conflicting_evidence() {
    let original = original("tests/fixtures/pyramid-empty-circle-redeal/last-observed.png");
    let reader = ReconVision::bundled_native_v23().unwrap();
    let mut tableau = [None; 28];
    tableau[0] = Some(Card::new(Rank::Nine, Suit::Clubs));
    tableau[2] = Some(Card::new(Rank::Four, Suit::Spades));
    for bounds in [
        PixelRect::new(952, 836, 16, 18),
        PixelRect::new(1816, 169, 1, 1),
    ] {
        let mut damaged = original.clone();
        paint(&mut damaged, bounds, [20, 80, 55]);
        assert!(
            reader
                .verify_native_stock_continuation(&damaged, &tableau, None, None)
                .is_err()
        );
    }
    let mut contradiction = original.clone();
    let zero = include_bytes!("../../../tests/fixtures/pyramid-recon/native-redeal/deals-zero.rgb");
    for (index, rgb) in zero.as_chunks::<3>().0.iter().enumerate() {
        let x = 890 + u32::try_from(index % 140).unwrap();
        let y = 874 + u32::try_from(index / 140).unwrap();
        paint(&mut contradiction, PixelRect::new(x, y, 1, 1), *rgb);
    }
    assert!(
        reader
            .verify_native_stock_continuation(&contradiction, &tableau, None, None)
            .is_err()
    );
}


/// Both actual Xbox stops preserve exact original slots, identities and exclusive piles.
/// The manually transcribed remaining slots are not an invented original route.
#[test]
fn native_141_xbox_artwork_removals_verify_both_actual_positions() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    for (name, remaining, left, right) in [
        (
            "zero",
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 20, 27][..],
            Card::new(Rank::Three, Suit::Clubs),
            Card::new(Rank::Four, Suit::Spades),
        ),
        (
            "one",
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 20, 21][..],
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
        ),
    ] {
        let frame = original(&format!(
            "tests/fixtures/pyramid-native-141/xbox-boards-{name}.png"
        ));
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/pyramid-native-141/xbox-boards-{name}-deal.json"
        ));
        let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
        let mut tableau = [None; 28];
        for &index in remaining {
            tableau[index] = Some(deal.tableau()[index]);
        }
        reader
            .verify_position(&frame, &tableau, Some(left), Some(right))
            .unwrap();
        let slot = crate::pyramid::PyramidTargetKind::Card { row: 5, column: 3 }
            .slot_index()
            .unwrap();
        assert_eq!(
            reader.native_occupancy(&frame).unwrap()[slot],
            FaceEvidence::Absent
        );
        for (bounds, rgb) in [
            (PixelRect::new(930, 503, 60, 4), [11, 13, 17]),
            (PixelRect::new(930, 503, 60, 4), [220; 3]),
            (PixelRect::new(930, 503, 60, 4), [247, 245, 226]),
            (PixelRect::new(960, 500, 1, 1), [255; 3]),
        ] {
            let mut corrupted = frame.clone();
            paint(&mut corrupted, bounds, rgb);
            assert_ne!(
                reader.native_occupancy(&corrupted).unwrap()[slot],
                FaceEvidence::Absent,
                "{name}: {bounds:?}"
            );
            assert!(
                reader
                    .verify_position(&corrupted, &tableau, Some(left), Some(right))
                    .is_err()
            );
        }
        let mut wrong = tableau;
        wrong[0] = None;
        assert!(
            reader
                .verify_position(&frame, &wrong, Some(left), Some(right))
                .is_err()
        );
        assert!(
            reader
                .verify_position(&frame, &tableau, Some(right), Some(left))
                .is_err()
        );
    }
}


/// The actual mixed legend frame must remain a complete native Score starting board.
#[test]
fn native_143_mixed_legend_actual_board_keeps_all_identities_and_piles() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let frame = original("tests/fixtures/pyramid-mixed-legend-143/score-entry.png");
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&frame).unwrap(),
        crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Score,
            775,
            2200
        )
    );
    let ReconScene::Board(board) = reader.observe(&frame).unwrap() else {
        panic!("actual mixed legend is a supported intact board");
    };
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Three, Suit::Spades))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
}


/// Mixed-scene proof keeps every strip, its bright grouping and independent board roles.
#[test]
fn native_143_mixed_legend_rejects_partial_ink_dim_overlays_and_scene_loss() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let frame = original("tests/fixtures/pyramid-mixed-legend-143/score-entry.png");
    let dim = crate::pyramid::WHITE_SCENE_PROBES[0];
    let bright = crate::pyramid::WHITE_SCENE_PROBES[3];
    let mut derivatives = Vec::new();
    for (bounds, rgb) in [
        (PixelRect::new(dim.x, dim.y, 1, 1), [255; 3]),
        (PixelRect::new(bright.x, bright.y, 1, 1), [80, 88, 84]),
        (dim, [255; 3]),
        (dim, [77, 111, 84]),
        (dim, [40, 48, 44]),
        (PixelRect::new(931, 695, 60, 50), [20, 40, 60]),
        (PixelRect::new(1796, 116, 109, 18), [255; 3]),
    ] {
        let mut changed = frame.clone();
        paint(&mut changed, bounds, rgb);
        derivatives.push(changed);
    }
    let mut nonadjacent = frame.clone();
    paint(
        &mut nonadjacent,
        crate::pyramid::WHITE_SCENE_PROBES[4],
        [80, 88, 84],
    );
    paint(
        &mut nonadjacent,
        crate::pyramid::WHITE_SCENE_PROBES[5],
        [255; 3],
    );
    derivatives.push(nonadjacent);
    let mut fully_dimmed = frame.clone();
    for bounds in crate::pyramid::WHITE_SCENE_PROBES {
        paint(&mut fully_dimmed, bounds, [80, 88, 84]);
    }
    derivatives.push(fully_dimmed);
    for changed in derivatives {
        assert!(
            !reader
                .board_scene_matches(changed.as_view().unwrap())
                .unwrap()
        );
    }
}


/// Broadening positive board evidence cannot suppress the specific modal fallback.
#[test]
fn native_143_mixed_legend_preserves_specific_modal_and_missing_component_refusal() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let source = original("tests/fixtures/pyramid-mixed-legend-143/score-entry.png");
    let mut modal = original("tests/fixtures/pyramid-recon/originals/gate3-undo-modal.png");
    for pattern in reader.dialog.signature.iter().skip(5) {
        paint(&mut modal, pattern.bounds, [8, 9, 12]);
    }
    // Synthetic composition: copy every bar and both surrounding rows into a real modal.
    for bounds in crate::pyramid::WHITE_SCENE_PROBES {
        for y in bounds.y - 3..bounds.y + 5 {
            for x in bounds.x..bounds.right() {
                let rgb = source.as_view().unwrap().pixel_rgb(x, y).unwrap();
                paint(&mut modal, PixelRect::new(x, y, 1, 1), rgb);
            }
        }
    }
    assert!(
        !reader
            .dialog
            .matches(modal.as_view().unwrap(), false)
            .unwrap()
    );
    assert!(matches!(
        reader.observe(&modal).unwrap(),
        ReconScene::UndoConfirmation { .. }
    ));
    paint(&mut modal, reader.dialog.signature[1].bounds, [8, 9, 12]);
    assert!(matches!(
        reader.observe(&modal).unwrap(),
        ReconScene::Unknown
    ));
}


/// Actual textured LEFT exhaustion uses temporal appearance with an exact occupied RIGHT.
/// No original consumed trace or pre-input frame is reconstructed by this image test.
#[test]
fn native_temporal_right_actual_position_keeps_all_card_pile_and_scene_guards() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let frame = original("tests/fixtures/pyramid-temporal-right-144/last-observed.png");
    let bytes = include_bytes!("../../../tests/fixtures/pyramid-temporal-right-144/deal.json");
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let mut tableau = [None; 28];
    for index in [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 17] {
        tableau[index] = Some(deal.tableau()[index]);
    }
    let right = Some(Card::new(Rank::Three, Suit::Diamonds));
    reader
        .verify_temporal_stock_position(&frame, &tableau, None, right, true)
        .expect("actual complete position with independent temporal appearance");
    assert!(
        reader
            .verify_temporal_stock_position(&frame, &tableau, None, right, false)
            .is_err()
    );
    let mut wrong = tableau;
    wrong[0] = None;
    assert!(
        reader
            .verify_temporal_stock_position(&frame, &wrong, None, right, true)
            .is_err()
    );
    assert!(
        reader
            .verify_temporal_stock_position(
                &frame,
                &tableau,
                None,
                Some(Card::new(Rank::Three, Suit::Clubs)),
                true,
            )
            .is_err()
    );
    assert!(
        reader
            .verify_temporal_stock_position(&frame, &tableau, right, right, true)
            .is_err()
    );
    for bounds in [
        PixelRect::new(759, 678, 1, 1),
        PixelRect::new(897, 864, 1, 1),
    ] {
        let mut background = frame.clone();
        paint(&mut background, bounds, [220; 3]);
        reader
            .verify_temporal_stock_position(&background, &tableau, None, right, true)
            .unwrap();
    }
    for bounds in [LEFT_BOUNDS, PixelRect::new(1816, 169, 1, 1)] {
        let mut damaged = frame.clone();
        paint(
            &mut damaged,
            bounds,
            if bounds.x == 1816 { [0; 3] } else { [220; 3] },
        );
        assert!(
            reader
                .verify_temporal_stock_position(&damaged, &tableau, None, right, true)
                .is_err(),
            "{bounds:?}"
        );
    }
}


/// Binary light-paper coverage uses the full calibrated footprint of either pile.
/// Synthetic boundary counts exercise the exact inclusive 45% occupied threshold.
#[test]
fn native_stock_paper_inverse_has_checked_exact_boundaries_for_both_piles() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let frame = original("tests/fixtures/pyramid-stock-paper-145/football-entry.png");
    for (bounds, pattern) in [
        (LEFT_BOUNDS, &reader.board.left_empty),
        (RIGHT_BOUNDS, &reader.board.right_empty),
    ] {
        for (paper_pixels, empty) in [
            (0, true),
            (1, true),
            (11_696, true),
            (11_697, false),
            (25_993, false),
        ] {
            let mut changed = frame.clone();
            paint(&mut changed, bounds, [24, 89, 61]);
            for index in 0..paper_pixels {
                let x = bounds.x + index % bounds.width;
                let y = bounds.y + index / bounds.width;
                paint(&mut changed, PixelRect::new(x, y, 1, 1), [220; 3]);
            }
            assert_eq!(
                reader
                    .native_empty(changed.as_view().unwrap(), bounds, pattern)
                    .unwrap(),
                empty,
                "{bounds:?}: {paper_pixels}"
            );
        }
        let mut pale = frame.clone();
        paint(&mut pale, bounds, [220; 3]);
        assert!(
            reader
                .stock_face(pale.as_view().unwrap(), bounds, pattern)
                .is_err(),
            "an unreadable high-paper face is not Empty"
        );
        assert!(
            reader
                .native_empty(
                    frame.as_view().unwrap(),
                    PixelRect::new(1919, 1079, 139, 187),
                    pattern
                )
                .is_err()
        );
    }
}


/// Retained cream QH, selected KH and textured empty stock keep their actual identities.
/// These pile-only assertions do not imply the football tableau is recognised.
#[test]
fn native_stock_paper_inverse_preserves_real_cream_and_selected_face_cards() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    for (path, card, paper) in [
        (
            "tests/fixtures/pyramid-stock-paper-145/football-entry.png",
            Card::new(Rank::Queen, Suit::Hearts),
            14_721,
        ),
        (
            "tests/fixtures/pyramid-recon/originals/issue1-13.png",
            Card::new(Rank::King, Suit::Hearts),
            13_883,
        ),
    ] {
        let frame = original(path);
        assert_eq!(
            crate::image_evidence::count_paper(frame.as_view().unwrap(), LEFT_BOUNDS, 180).unwrap(),
            paper
        );
        assert_eq!(
            reader
                .stock_face(
                    frame.as_view().unwrap(),
                    LEFT_BOUNDS,
                    &reader.board.left_empty
                )
                .unwrap(),
            StockFace::Card(card),
            "{path}"
        );
    }
    let frame = original("tests/fixtures/pyramid-stock-paper-145/football-entry.png");
    assert_eq!(
        reader
            .stock_face(
                frame.as_view().unwrap(),
                RIGHT_BOUNDS,
                &reader.board.right_empty
            )
            .unwrap(),
        StockFace::Empty
    );
}
