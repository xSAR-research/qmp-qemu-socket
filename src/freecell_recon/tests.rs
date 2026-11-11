//! Native identity acquisition and live binding without decorative gameplay vetoes.
use super::*;
const FC01: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC01.png");
/// Complete unchanged native RECON request1 frame from the retained highlighted-card incident.
const HIGHLIGHTED_CHALLENGE: &[u8] =
    include_bytes!("../../tests/fixtures/freecell-challenge-highlight-20261109.png");
/// Independently read physical cards from that native frame, without inventory elimination.
const HIGHLIGHTED_LABELS: [&[&str]; 8] = [
    &["TD", "9C", "JC", "2S", "9S", "KD", "KH"],
    &["9D", "2H", "2D", "6H", "TH", "8S", "3H"],
    &["5S", "JS", "JD", "KS", "JH", "6D", "7C"],
    &["QS", "7H", "AS", "7S", "TC", "TS", "7D"],
    &["AD", "KC", "QD", "4C", "5D", "AC"],
    &["3D", "8H", "5H", "9H", "QH", "AH"],
    &["4H", "5C", "2C", "8C", "QC", "8D"],
    &["6S", "4S", "4D", "3C", "6C", "3S"],
];
/// Complete unchanged3.1.7 failure capture with the four actual highlighted sevens.
const SEVENS_CHALLENGE: &[u8] =
    include_bytes!("../../tests/fixtures/freecell-sevens-halo-20261109.png");
/// Independently read native physical cards, including the actual final-face7C.
const SEVENS_LABELS: [&[&str]; 8] = [
    &["4S", "KH", "5S", "3H", "7H", "9D", "7C"],
    &["3C", "AS", "KD", "AD", "KC", "8C", "TH"],
    &["7S", "6C", "7D", "TD", "JC", "6S", "AC"],
    &["5D", "5C", "2H", "4C", "2S", "TS", "2C"],
    &["JH", "JS", "4D", "QC", "QH", "5H"],
    &["AH", "8S", "2D", "3D", "9C", "QD"],
    &["4H", "TC", "QS", "6D", "3S", "JD"],
    &["8H", "8D", "9S", "KS", "9H", "6H"],
];


/// Complete normal-board capture containing the newly calibrated native6D appearance.
const NORMAL_SIX: &[u8] = include_bytes!("../../tests/fixtures/freecell-normal-six-20261110.png");
/// Independently read physical identities; only PLAY7's final rank supplies new calibration.
const NORMAL_SIX_LABELS: [&[&str]; 8] = [
    &["QH", "TD", "9D", "JC", "2H", "JH", "5S"],
    &["7S", "AS", "8C", "KH", "AH", "QS", "QC"],
    &["4H", "KC", "3S", "9S", "5H", "7H", "4C"],
    &["5C", "9C", "4D", "TS", "4S", "7C", "KD"],
    &["TC", "6C", "KS", "8S", "JD", "AD"],
    &["2S", "9H", "3D", "8D", "2C", "7D"],
    &["TH", "AC", "6S", "QD", "8H", "6D"],
    &["3C", "3H", "6H", "5D", "JS", "2D"],
];


/// Separate native challenge capture, held out from the additional rank calibration.
const GOLD_FIVE_HEART: &[u8] =
    include_bytes!("../../tests/fixtures/freecell-gold-five-heart-20261110.png");
/// Independently read original columns, including the genuine highlighted5H at PLAY1depth2.
const GOLD_FIVE_HEART_LABELS: [&[&str]; 8] = [
    &["5D", "5H", "7H", "8H", "KS", "3H", "KC"],
    &["6D", "TC", "TS", "AD", "6C", "6H", "QH"],
    &["QC", "TH", "5C", "7S", "4D", "JD", "QS"],
    &["7D", "9C", "8S", "2C", "4S", "AS", "JS"],
    &["3S", "9H", "QD", "5S", "2D", "JH"],
    &["7C", "9D", "8C", "KD", "KH", "2S"],
    &["JC", "9S", "6S", "4H", "3D", "8D"],
    &["AH", "2H", "4C", "3C", "TD", "AC"],
];


/// Byte-identical current refusal capture, retaining the gold6H and ordinary PLAY3 rails.
const SIXES_CHALLENGE: &[u8] =
    include_bytes!("../../tests/fixtures/freecell-sixes-decorative-20261111.png");
/// Independently read physical cards from the native incident; no deck elimination supplies labels.
const SIXES_LABELS: [&[&str]; 8] = [
    &["8S", "3C", "TD", "7D", "7C", "KC", "4H"],
    &["8C", "9S", "5C", "JD", "9H", "KD", "4C"],
    &["6H", "9D", "8H", "AD", "JS", "QD", "QH"],
    &["3H", "KS", "7H", "6D", "3S", "4S", "5S"],
    &["9C", "6C", "5H", "TS", "TH", "TC"],
    &["JC", "3D", "AH", "2H", "8D", "JH"],
    &["AS", "7S", "2D", "6S", "QC", "2S"],
    &["QS", "2C", "AC", "5D", "KH", "4D"],
];


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


/// Exercise fresh scene admission and the independent52-card live initial binding.
fn playback_initial(frame: &CapturedFrame) -> Result<InitialDeal, RecognitionError> {
    recognise_playback_initial_checked(frame, metadata(), &mut || Ok(()))
}


/// Compare all manually read physical identities without inferring any from deck inventory.
fn assert_labels(deal: &InitialDeal, labels: [&[&str]; 8]) {
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
#[test]
fn freecell_300_originals_read_all_printed_physical_identities() {
    for (png, labels) in [(FC01, FC01_LABELS), (REFERENCE, LABELS)] {
        let frame = decode_png(png).unwrap();
        let deal = recognise(&frame, metadata()).unwrap();
        assert_eq!(
            objective::from_metadata(deal.metadata()),
            objective::Objective::FullBoard
        );
        assert_labels(&deal, labels);
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


/// Retain the original independent strip calibration for measured before/after competition.
fn original_strip_bank(reference: &CapturedFrame) -> Vec<Template> {
    let mut bank = Vec::new();
    for (column, (&x, labels)) in PLAY_X.iter().zip(LABELS).enumerate() {
        for (depth, code) in labels.iter().enumerate().take(INITIAL_LENGTHS[column] - 1) {
            bank.push(Template {
                card: label(code).unwrap(),
                rank: glyph(reference, x, ROW_Y[depth], true).unwrap(),
                suit: glyph(reference, x, ROW_Y[depth], false).unwrap(),
            });
        }
    }
    bank
}


/// The native6D prototype repairs its measured19-pixel gap without changing either cutoff.
#[test]
fn freecell_3110_native_six_rank_calibration_keeps_original_suit_and_ambiguity() {
    let normal = decode_png(NORMAL_SIX).unwrap();
    let reference = decode_png(REFERENCE).unwrap();
    let actual = glyph_variants(&normal, 1373, 597, true).unwrap();
    let mut bank = original_strip_bank(&reference);
    let refused = identify(&actual, &bank, |t| (t.card.rank(), &t.rank), 48, 20).unwrap_err();
    assert_eq!(refused.kind, FailureKind::AmbiguousIdentity);
    assert_eq!((refused.best, refused.next), (21, 40));
    append_native_six_rank(&mut bank).unwrap();
    let sample = bank.last().unwrap();
    let original = bank
        .iter()
        .find(|t| t.card == label("6D").unwrap())
        .unwrap();
    assert_eq!(sample.suit.red, original.suit.red);
    assert_eq!(sample.suit.ink, original.suit.ink);
    assert_eq!(distance(actual.first().unwrap(), &sample.rank), 0);
    assert_eq!(
        identify(&actual, &bank, |t| (t.card.rank(), &t.rank), 48, 20).unwrap(),
        label("6D").unwrap().rank()
    );
    let ambiguous = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-ambiguous-rank-20261109.png"
    ))
    .unwrap();
    let mut derivative = normal.clone();
    for row in 0..ambiguous.height as usize {
        let from = row * ambiguous.stride;
        let to = (603 + row) * derivative.stride + 1374 * 4;
        derivative.pixels[to..to + 28 * 4].copy_from_slice(&ambiguous.pixels[from..from + 28 * 4]);
    }
    let refused = recognise(&derivative, metadata()).unwrap_err();
    assert_eq!(refused.kind, FailureKind::AmbiguousIdentity);
    assert!(refused.cause.contains("best 34, next 50"));
}


/// Calibration and held-out challenge frames both retain every independently read physical card.
#[test]
fn freecell_3110_native_normal_and_gold_five_heart_bind_complete_printed_inventories() {
    for (png, labels, objective) in [
        (
            NORMAL_SIX,
            NORMAL_SIX_LABELS,
            objective::Objective::FullBoard,
        ),
        (
            GOLD_FIVE_HEART,
            GOLD_FIVE_HEART_LABELS,
            objective::Objective::Unresolved,
        ),
    ] {
        let frame = decode_png(png).unwrap();
        let acquired = recognise(&frame, metadata()).unwrap();
        assert_labels(&acquired, labels);
        assert_eq!(objective::from_metadata(acquired.metadata()), objective);
        let bound = playback_initial(&frame).unwrap();
        assert_labels(&bound, labels);
        assert_eq!(bound.board(), acquired.board());
    }
}


/// The authentic incident freshly binds the same52 printed cards without decorative vetoes.
#[test]
fn freecell_3114_native_sixes_incident_binds_independent_complete_inventory() {
    let frame = decode_png(SIXES_CHALLENGE).unwrap();
    let acquired = recognise(&frame, metadata()).unwrap();
    assert_labels(&acquired, SIXES_LABELS);
    let bound = playback_initial(&frame).unwrap();
    assert_labels(&bound, SIXES_LABELS);
    assert_eq!(bound.board(), acquired.board());
    assert_eq!(
        objective::from_metadata(bound.metadata()),
        objective::Objective::Unresolved
    );
    let expected = xsar::games::freecell::rules::Position::initial(&bound);
    position::verify(&frame, &expected, &mut || Ok(())).unwrap();
    for red in [237, 238] {
        let mut decorated = frame.clone();
        paint(
            &mut decorated,
            PLAY_X[2] + CARD_WIDTH - 1,
            690,
            1,
            1,
            [red, 206, 120],
        );
        assert_eq!(playback_initial(&decorated).unwrap().board(), bound.board());
        position::verify(&decorated, &expected, &mut || Ok(())).unwrap();
    }
}


/// New native appearances still require complete, coherent, unique visible identities.
#[test]
fn freecell_3110_native_six_and_five_heart_reject_glyph_damage_and_duplicate_identity() {
    for (png, x, y) in [(NORMAL_SIX, 1373, 597), (GOLD_FIVE_HEART, 218, 384)] {
        let original = decode_png(png).unwrap();
        for (dx, dy, w, h, rgb) in [
            (1, 6, 28, 25, [255, 255, 255]),
            (2, 7, 4, 4, [0, 0, 0]),
            (2, 7, 26, 23, [170, 30, 190]),
            (4, 29, 20, 18, [255, 255, 255]),
        ] {
            let mut damaged = original.clone();
            paint(&mut damaged, x + dx, y + dy, w, h, rgb);
            assert!(
                recognise(&damaged, metadata()).is_err(),
                "{x},{y} {dx},{dy}"
            );
            assert!(playback_initial(&damaged).is_err(), "{x},{y} {dx},{dy}");
        }
        let mut duplicate = original.clone();
        // Explicit synthetic placement of this native corner onto a different physical card.
        for row in 0..48 {
            let from = (y as usize + row) * original.stride + x as usize * 4;
            let to = (331 + row) * duplicate.stride + 603 * 4;
            duplicate.pixels[to..to + 30 * 4]
                .copy_from_slice(&original.pixels[from..from + 30 * 4]);
        }
        assert_eq!(
            recognise(&duplicate, metadata()).unwrap_err().kind,
            FailureKind::DuplicateIdentity
        );
    }
}


#[test]
fn freecell_300_missing_unknown_mixed_duplicate_and_invalid_frame_refuse() {
    let original = decode_png(FC01).unwrap();
    for (x, y, w, h, rgb) in [
        (220, 338, 26, 23, [255, 255, 255]),
        (223, 361, 18, 16, [255, 255, 255]),
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


/// The genuine highlighted challenge acquires and freshly binds all52 printed identities.
#[test]
fn freecell_314_native_highlighted_challenge_reads_complete_physical_inventory() {
    let frame = decode_png(HIGHLIGHTED_CHALLENGE).unwrap();
    let deal = recognise(&frame, metadata()).unwrap();
    assert_labels(&deal, HIGHLIGHTED_LABELS);
    assert_eq!(
        objective::from_metadata(deal.metadata()),
        objective::Objective::Unresolved
    );
    assert!(objective::require_acquired(deal.metadata()).is_ok());
    let bytes = xsar::freecell_json::to_vec(&deal).unwrap();
    assert_eq!(xsar::freecell_json::from_slice(&bytes).unwrap(), deal);
    assert_eq!(playback_initial(&frame).unwrap().board(), deal.board());
}


/// Synthetic changes outside objective and all bounded glyph variants cannot veto RECON.
#[test]
fn freecell_314_all_other_pixels_are_irrelevant_to_identity_acquisition() {
    let mut frame = decode_png(FC01).unwrap();
    let expected = recognise(&frame, metadata()).unwrap();
    let mut preserved = vec![false; frame.width as usize * frame.height as usize];
    let rect = objective::BOUNDS;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            preserved[y as usize * frame.width as usize + x as usize] = true;
        }
    }
    for (&x, n) in PLAY_X.iter().zip(INITIAL_LENGTHS) {
        for &y in ROW_Y.iter().take(n) {
            // Full unions of the existing nine ±1 rank and suit recrops.
            for (dx, dy, w, h) in [(1, 6, 28, 25), (4, 29, 20, 18)] {
                for yy in y + dy..y + dy + h {
                    for xx in x + dx..x + dx + w {
                        preserved[yy as usize * frame.width as usize + xx as usize] = true;
                    }
                }
            }
        }
    }
    for (y, row) in preserved.chunks_exact(frame.width as usize).enumerate() {
        for (x, &keep) in row.iter().enumerate() {
            if !keep {
                let offset = y * frame.stride + x * 4;
                frame.pixels[offset..offset + 3].copy_from_slice(&[23, 41, 73]);
            }
        }
    }
    let acquired = recognise(&frame, metadata()).unwrap();
    assert_eq!(acquired, expected);
    assert!(playback_initial(&frame).is_err());
    assert!(
        position::verify(
            &frame,
            &xsar::games::freecell::rules::Position::initial(&expected),
            &mut || Ok(())
        )
        .is_err()
    );
}


/// Native ambiguous rank evidence remains refused after unrelated scene guards are removed.
#[test]
fn freecell_314_native_ambiguous_rank_is_not_completed_by_elimination() {
    let mut frame = decode_png(FC01).unwrap();
    let crop = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-ambiguous-rank-20261109.png"
    ))
    .unwrap();
    assert_eq!((crop.width, crop.height), (28, 25));
    // Explicit synthetic placement of the original native rank's complete ±1 recrop union.
    for y in 0..crop.height as usize {
        let from = y * crop.stride;
        let to = (331 + 6 + y) * frame.stride + (218 + 1) * 4;
        frame.pixels[to..to + 28 * 4].copy_from_slice(&crop.pixels[from..from + 28 * 4]);
    }
    let error = recognise(&frame, metadata()).unwrap_err();
    assert_eq!(error.kind, FailureKind::AmbiguousIdentity);
    assert_eq!(error.region, Region::Rank);
    assert_eq!(
        error.place,
        Place::Cascade {
            column: 0,
            depth: 0
        }
    );
    assert!(error.cause.contains("best 34, next 50"));
}


/// Readable non-white paper uses the same measured contrast for acquisition and live binding.
#[test]
fn freecell_3114_local_contrast_is_shared_by_recon_and_live_binding() {
    let original = decode_png(FC01).unwrap();
    let expected = recognise(&original, metadata()).unwrap();
    for rgb in [[233, 233, 233], [233, 250, 255], [255, 250, 233]] {
        let mut frame = original.clone();
        for y in 338..361 {
            for x in 220..246 {
                let offset = y * frame.stride + x * 4;
                if original.pixels[offset..offset + 3]
                    .iter()
                    .all(|&v| v >= 200)
                {
                    frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
                }
            }
        }
        assert_eq!(
            recognise(&frame, metadata()).unwrap().board(),
            expected.board()
        );
        assert_eq!(playback_initial(&frame).unwrap().board(), expected.board());
        position::verify(
            &frame,
            &xsar::games::freecell::rules::Position::initial(&expected),
            &mut || Ok(()),
        )
        .unwrap();
    }
}


/// Coherent red ink without sufficient relative glyph contrast cannot supply an identity.
#[test]
fn freecell_314_missing_contrast_refuses_without_identity_elimination() {
    let original = decode_png(FC01).unwrap();
    let mut frame = original.clone();
    for y in 338..361 {
        for x in 220..246 {
            let offset = y * frame.stride + x * 4;
            let paper = original.pixels[offset..offset + 3]
                .iter()
                .all(|&v| v >= 200);
            frame.pixels[offset..offset + 3].copy_from_slice(if paper {
                &[233, 233, 233]
            } else {
                &[255, 145, 145]
            });
        }
    }
    let error = recognise(&frame, metadata()).unwrap_err();
    assert_eq!(error.kind, FailureKind::UnknownIdentity);
    assert_eq!(error.region, Region::Rank);
    assert_eq!(
        error.place,
        Place::Cascade {
            column: 0,
            depth: 0
        }
    );
}


/// Native dimensions and storage remain mandatory even when all glyph origins are fixed.
#[test]
fn freecell_314_invalid_frame_storage_and_native_dimensions_refuse() {
    let original = decode_png(FC01).unwrap();
    for change in 0..4 {
        let mut frame = original.clone();
        match change {
            0 => frame.width = 1919,
            1 => frame.height = 1079,
            2 => frame.stride = 1,
            _ => {
                frame.pixels.pop();
            }
        }
        assert_eq!(
            recognise(&frame, metadata()).unwrap_err().kind,
            FailureKind::InvalidFrame
        );
    }
}


/// The retained later header is unresolved objective evidence, never full-board authority.
/// Its opaque objective never supplies card identity or semantic completion authority.
#[test]
fn freecell_313_challenge_header_is_retained_without_full_board_authority() {
    let challenge = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-challenge-20261107.png"
    ))
    .unwrap();
    let original = decode_png(FC01).unwrap();
    let reference = decode_png(REFERENCE).unwrap();
    assert_eq!(
        supported_scene(&challenge, &reference, &mut || Ok(())).unwrap(),
        objective::Objective::Unresolved
    );
    assert_eq!(
        playback_initial(&challenge).unwrap().board(),
        recognise(&challenge, metadata()).unwrap().board()
    );

    // Synthetic derivative: retain FC01's real inventory/geometry, replacing only objective pixels.
    let mut composed = original.clone();
    let rect = objective::BOUNDS;
    for y in rect.y..rect.y + rect.height {
        let begin = y as usize * composed.stride + rect.x as usize * 4;
        let source = y as usize * challenge.stride + rect.x as usize * 4;
        let bytes = rect.width as usize * 4;
        composed.pixels[begin..begin + bytes]
            .copy_from_slice(&challenge.pixels[source..source + bytes]);
    }
    let deal = recognise(&composed, metadata()).unwrap();
    let normal = recognise(&original, metadata()).unwrap();
    assert_eq!(deal.board(), normal.board());
    assert_eq!(
        objective::from_metadata(deal.metadata()),
        objective::Objective::Unresolved
    );
    assert!(objective::require_acquired(deal.metadata()).is_ok());
    let deal = objective::attach_evidence(&deal, "acquisition-313-request-1").unwrap();
    let bytes = xsar::freecell_json::to_vec(&deal).unwrap();
    let reopened = xsar::freecell_json::from_slice(&bytes).unwrap();
    assert_eq!(reopened, deal);
    assert!(
        reopened
            .metadata()
            .source()
            .ends_with(";evidence=acquisition-313-request-1")
    );
    assert!(objective::require_acquired(reopened.metadata()).is_ok());
    //3.1.5 permits opaque objectives while retaining all physical position evidence.
    position::verify(
        &composed,
        &xsar::games::freecell::rules::Position::initial(&normal),
        &mut || Ok(()),
    )
    .unwrap();
}


/// Live binding ignores arbitrary canvas regions while retaining real controls and Solver refusal.
#[test]
fn freecell_318_objective_variation_preserves_independent_control_refusals() {
    let original = decode_png(FC01).unwrap();
    let expected = playback_initial(&original).unwrap();
    for (x, y, w, h) in [
        (700, 36, 96, 50),
        (1124, 36, 96, 50),
        (80, 100, 24, 845),
        (1790, 100, 24, 845),
    ] {
        let mut frame = original.clone();
        paint(
            &mut frame,
            objective::BOUNDS.x,
            objective::BOUNDS.y,
            objective::BOUNDS.width,
            objective::BOUNDS.height,
            [160, 45, 45],
        );
        paint(&mut frame, x, y, w, h, [0, 0, 0]);
        assert_eq!(playback_initial(&frame).unwrap().board(), expected.board());
        position::verify(
            &frame,
            &xsar::games::freecell::rules::Position::initial(&expected),
            &mut || Ok(()),
        )
        .unwrap();
    }
    for rect in NATIVE_CONTROL_REGIONS {
        let mut frame = original.clone();
        paint(
            &mut frame,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            [0, 0, 0],
        );
        let error = playback_initial(&frame).unwrap_err();
        assert_eq!(error.kind, FailureKind::UnsupportedScene);
        assert!(error.cause.contains("game-control shape"));
    }
    let mut solver = original.clone();
    for y in [36, 85] {
        paint(&mut solver, 828, y, 263, 1, [220, 180, 20]);
    }
    assert!(crate::freecell::solver_active(&solver).unwrap());
    assert_eq!(
        recognise(&solver, metadata()).unwrap().board(),
        recognise(&original, metadata()).unwrap().board()
    );
    let refusal = playback_initial(&solver).unwrap_err();
    assert_eq!(refusal.kind, FailureKind::UnsupportedScene);
    assert!(refusal.cause.contains("Solver"));
    let native_solver =
        decode_png(include_bytes!("../../tests/fixtures/freecell-FC02.png")).unwrap();
    assert!(crate::freecell::solver_active(&native_solver).unwrap());
    let reference = decode_png(REFERENCE).unwrap();
    let refusal = supported_scene(&native_solver, &reference, &mut || Ok(())).unwrap_err();
    assert_eq!(refusal.kind, FailureKind::UnsupportedScene);
    // Synthetic derivative: transpose only FC02's real Solver header onto intact FC01.
    let mut native_header_only = original.clone();
    let rect = objective::BOUNDS;
    for y in rect.y..rect.y + rect.height {
        let begin = y as usize * native_header_only.stride + rect.x as usize * 4;
        let source = y as usize * native_solver.stride + rect.x as usize * 4;
        let bytes = rect.width as usize * 4;
        native_header_only.pixels[begin..begin + bytes]
            .copy_from_slice(&native_solver.pixels[source..source + bytes]);
    }
    let refusal = supported_scene(&native_header_only, &reference, &mut || Ok(())).unwrap_err();
    assert_eq!(refusal.kind, FailureKind::UnsupportedScene);
    assert!(refusal.cause.contains("Solver"));
    for png in [
        include_bytes!("../../tests/fixtures/freecell-FC09.png").as_slice(),
        include_bytes!("../../tests/fixtures/freecell-FC12.png").as_slice(),
    ] {
        assert!(playback_initial(&decode_png(png).unwrap()).is_err());
    }
}


/// RECON and live binding require all identities; live binding additionally proves empty slots.
#[test]
fn freecell_300_every_identity_and_empty_slot_is_required() {
    let original = decode_png(FC01).unwrap();
    let expected = recognise(&original, metadata()).unwrap();
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
        assert_eq!(
            recognise(&damaged, metadata()).unwrap().board(),
            expected.board()
        );
        assert!(
            playback_initial(&damaged).is_err(),
            "missing live top proof at {x}"
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
        assert_eq!(
            recognise(&damaged, metadata()).unwrap().board(),
            expected.board()
        );
        assert_eq!(
            playback_initial(&damaged).unwrap().board(),
            expected.board()
        );
    }
}

/// Independent complete native faces exercise typography beyond the exposed-strip training bank.
#[test]
fn freecell_300_native_full_faces_keep_rank_and_suit_competition() {
    let reference = decode_png(REFERENCE).unwrap();
    let mut bank = original_strip_bank(&reference);
    append_native_six_rank(&mut bank).unwrap();
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

/// Intact fixed corners bind independently of decorative final artwork and rails.
#[test]
fn freecell_3114_clipped_final_artwork_and_rails_do_not_veto_readable_identities() {
    let original = decode_png(FC01).unwrap();
    let expected = recognise(&original, metadata()).unwrap();
    for (&x, n) in PLAY_X.iter().zip(INITIAL_LENGTHS) {
        let top = ROW_Y[n - 1];
        let bottom = if n == 7 { 832 } else { 779 };
        for (xx, yy, w, h) in [
            (x, top + 50, CARD_WIDTH, bottom - top - 49),
            (x, top + 55, 1, 4),
            (x + CARD_WIDTH - 1, top + 55, 1, 4),
        ] {
            let mut cut = original.clone();
            paint(&mut cut, xx, yy, w, h, [20, 100, 20]);
            assert_eq!(
                recognise(&cut, metadata()).unwrap().board(),
                expected.board()
            );
            assert_eq!(playback_initial(&cut).unwrap().board(), expected.board());
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
    paint(&mut slot, CELL_X[2] + 30, 113, 4, 3, [0, 0, 0]);
    assert!(recognise(&slot, metadata()).is_ok());
    let error = playback_initial(&slot).unwrap_err();
    assert_eq!(error.kind, FailureKind::TopSlotNotEmpty);
    assert_eq!(error.place, Place::Cell(2));
    assert_eq!(error.region, Region::TopSlot);
}


/// Actual retained gold challenge plus normal originals bind exact complete physical inventories.
#[test]
fn freecell_315_playback_initial_reads_native_gold_and_preserves_normal_binding() {
    for (png, expected) in [
        (HIGHLIGHTED_CHALLENGE, HIGHLIGHTED_LABELS),
        (FC01, FC01_LABELS),
        (REFERENCE, LABELS),
    ] {
        let frame = decode_png(png).unwrap();
        let deal = playback_initial(&frame).unwrap();
        assert_labels(&deal, expected);
        let position = xsar::games::freecell::rules::Position::initial(&deal);
        position::verify(&frame, &position, &mut || Ok(())).unwrap();
    }
}


/// Current objective, gold edges and unrelated decoration do not change readable card identity.
#[test]
fn freecell_3114_gold_paper_does_not_require_objective_or_decorative_outline() {
    let original = decode_png(HIGHLIGHTED_CHALLENGE).unwrap();
    let reference = decode_png(REFERENCE).unwrap();
    let deal = recognise(&original, metadata()).unwrap();
    let expected = xsar::games::freecell::rules::Position::initial(&deal);
    let mut no_objective = original.clone();
    let rect = objective::BOUNDS;
    for y in rect.y..rect.y + rect.height {
        let offset = y as usize * no_objective.stride + rect.x as usize * 4;
        let source = y as usize * reference.stride + rect.x as usize * 4;
        let bytes = rect.width as usize * 4;
        no_objective.pixels[offset..offset + bytes]
            .copy_from_slice(&reference.pixels[source..source + bytes]);
    }
    assert_eq!(
        playback_initial(&no_objective).unwrap().board(),
        deal.board()
    );
    position::verify(&no_objective, &expected, &mut || Ok(())).unwrap();
    for (x, y) in [(450, 331), (410, 355), (546, 355)] {
        let mut interrupted = original.clone();
        paint(&mut interrupted, x, y, 1, 1, [20, 100, 20]);
        assert_eq!(
            playback_initial(&interrupted).unwrap().board(),
            deal.board()
        );
        position::verify(&interrupted, &expected, &mut || Ok(())).unwrap();
    }
    let mut unrelated = decode_png(FC01).unwrap();
    let normal = recognise(&unrelated, metadata()).unwrap();
    paint(&mut unrelated, 825, 45, 30, 30, [160, 45, 45]);
    paint(&mut unrelated, 900, 300, 121, 1, [238, 208, 94]);
    for y in 338..361 {
        for x in 220..246 {
            let offset = y * unrelated.stride + x * 4;
            if unrelated.pixels[offset..offset + 3]
                .iter()
                .all(|&v| v >= 200)
            {
                unrelated.pixels[offset..offset + 3].copy_from_slice(&[255, 250, 233]);
            }
        }
    }
    assert_eq!(
        playback_initial(&unrelated).unwrap().board(),
        normal.board()
    );
}


/// The authentic sevens incident binds all52 cards despite its halo extending below PLAY1.
#[test]
fn freecell_318_native_sevens_halo_binds_complete_physical_initial_position() {
    let frame = decode_png(SEVENS_CHALLENGE).unwrap();
    let acquired = recognise(&frame, metadata()).unwrap();
    assert_labels(&acquired, SEVENS_LABELS);
    let bound = playback_initial(&frame).unwrap();
    assert_labels(&bound, SEVENS_LABELS);
    assert_eq!(bound.board(), acquired.board());
    assert_eq!(
        objective::from_metadata(bound.metadata()),
        objective::Objective::Unresolved
    );
    let expected = xsar::games::freecell::rules::Position::initial(&bound);
    let geometry = position::verify(&frame, &expected, &mut || Ok(())).unwrap();
    assert_eq!(
        geometry
            .point(xsar::games::freecell::rules::Pile::Cascade(0))
            .unwrap(),
        crate::geometry::PixelPoint::new(233, 669)
    );
}


/// Live scene/position proofs ignore off-card canvas RGB and empty-slot interior texture.
#[test]
fn freecell_318_live_binding_is_invariant_to_advisory_background_pixels() {
    for png in [FC01, HIGHLIGHTED_CHALLENGE, SEVENS_CHALLENGE] {
        let original = decode_png(png).unwrap();
        let expected = playback_initial(&original).unwrap();
        let position = xsar::games::freecell::rules::Position::initial(&expected);
        let mut changed = original;
        for (x, y, w, h) in [
            (700, 36, 96, 50),
            (1124, 36, 96, 50),
            (80, 100, 24, 845),
            (1790, 100, 24, 845),
            (218, 840, 137, 95),
            (410, 840, 137, 95),
            (988, 790, 137, 145),
        ] {
            paint(&mut changed, x, y, w, h, [101, 27, 113]);
        }
        for x in CELL_X.into_iter().chain(FOUNDATION_X) {
            paint(&mut changed, x + 20, 145, 95, 110, [101, 27, 113]);
        }
        assert_eq!(
            playback_initial(&changed).unwrap().board(),
            expected.board()
        );
        position::verify(&changed, &position, &mut || Ok(())).unwrap();
    }
}


/// Native gold decoration never relaxes required visible identity evidence.
#[test]
fn freecell_3114_native_sevens_reject_unknown_mixed_and_occluded_identities() {
    let original = decode_png(SEVENS_CHALLENGE).unwrap();
    let initial = playback_initial(&original).unwrap();
    let expected = xsar::games::freecell::rules::Position::initial(&initial);
    for (x, y, w, h, rgb, exposed) in [
        (220, 338, 26, 23, [255, 255, 255], false),
        (220, 338, 26, 23, [170, 30, 190], false),
        (218, 650, 137, 183, [20, 100, 20], true),
    ] {
        let mut damaged = original.clone();
        paint(&mut damaged, x, y, w, h, rgb);
        assert!(playback_initial(&damaged).is_err(), "{x},{y}");
        let located = position::verify(&damaged, &expected, &mut || Ok(()));
        assert_eq!(located.is_err(), exposed, "{x},{y}");
    }
    for (x, y, w, h, rgb) in [
        (250, 650, 1, 1, [215, 215, 215]),
        (218, 740, 1, 1, [215, 215, 215]),
        (354, 740, 1, 1, [215, 215, 215]),
        (250, 831, 1, 3, [215, 215, 215]),
        (603, 355, 1, 1, [215, 215, 215]),
        (739, 462, 1, 1, [215, 215, 215]),
        (410, 667, 1, 1, [200, 218, 222]),
    ] {
        let mut decorated = original.clone();
        paint(&mut decorated, x, y, w, h, rgb);
        assert_eq!(
            playback_initial(&decorated).unwrap().board(),
            initial.board()
        );
        position::verify(&decorated, &expected, &mut || Ok(())).unwrap();
    }
    let mut mixed = original.clone();
    // Genuine red3H suit replaces the black4S suit, leaving the black rank unchanged.
    for dy in 0..16 {
        for dx in 0..18 {
            let from = (490 + 30 + dy) * original.stride + (218 + 5 + dx) * 4;
            let to = (331 + 30 + dy) * mixed.stride + (218 + 5 + dx) * 4;
            mixed.pixels[to..to + 4].copy_from_slice(&original.pixels[from..from + 4]);
        }
    }
    assert_eq!(
        playback_initial(&mixed).unwrap_err().kind,
        FailureKind::ColourDisagreement
    );
    // The damaged hidden prefix is not re-read during exposed-pile localisation.
    position::verify(&mixed, &expected, &mut || Ok(())).unwrap();
}


/// Initial binding reads the fixed52 cards; a wrong extra exposed card fails position matching.
#[test]
fn freecell_3114_extra_exposed_card_below_initial_face_refuses_position_match() {
    let original = decode_png(FC01).unwrap();
    let mut changed = original.clone();
    for dy in 0..53 {
        for dx in 0..137 {
            let from = (331 + dy) * original.stride + (218 + dx) * 4;
            let to = (850 + dy) * changed.stride + (218 + dx) * 4;
            changed.pixels[to..to + 4].copy_from_slice(&original.pixels[from..from + 4]);
        }
    }
    // Initial binding independently reads the fixed52 corners; effect geometry detects extras.
    assert_eq!(
        playback_initial(&changed).unwrap().board(),
        recognise(&original, metadata()).unwrap().board()
    );
    let expected =
        xsar::games::freecell::rules::Position::initial(&playback_initial(&original).unwrap());
    let error = position::verify(&changed, &expected, &mut || Ok(())).unwrap_err();
    assert!(!error.is_empty(), "{error}");
}


/// Gold paper never relaxes glyph colour coherence, confidence, ambiguity or cancellation.
#[test]
fn freecell_315_gold_playback_retains_glyph_competition_and_cancellation() {
    let original = decode_png(HIGHLIGHTED_CHALLENGE).unwrap();
    let mut mixed = original.clone();
    paint(&mut mixed, 412, 338, 4, 4, [0, 0, 0]);
    let refusal = playback_initial(&mixed).unwrap_err();
    assert_eq!(refusal.kind, FailureKind::UnknownIdentity);
    assert_eq!(refusal.region, Region::Rank);
    assert!(refusal.cause.contains("mixed-colour"));

    let crop = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-ambiguous-rank-20261109.png"
    ))
    .unwrap();
    let mut ambiguous = original.clone();
    for y in 0..crop.height as usize {
        let from = y * crop.stride;
        let to = (331 + 6 + y) * ambiguous.stride + (410 + 1) * 4;
        ambiguous.pixels[to..to + 28 * 4].copy_from_slice(&crop.pixels[from..from + 28 * 4]);
    }
    let refusal = playback_initial(&ambiguous).unwrap_err();
    assert_eq!(refusal.kind, FailureKind::AmbiguousIdentity);
    assert_eq!(refusal.region, Region::Rank);
    assert!(refusal.cause.contains("best 34, next 50"));

    let mut count = 0;
    recognise_playback_initial_checked(&original, metadata(), &mut || {
        count += 1;
        Ok(())
    })
    .unwrap();
    for stop_at in [1, 70, count - 1] {
        let mut visited = 0;
        let refusal = recognise_playback_initial_checked(&original, metadata(), &mut || {
            visited += 1;
            if visited == stop_at {
                Err("playback authority revoked".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(refusal.kind, FailureKind::Revoked);
        assert_eq!(refusal.cause, "playback authority revoked");
        assert_eq!(visited, stop_at);
    }
}


/// Supported scene, Solver exclusion and actual initial empty slots stay required.
#[test]
fn freecell_3114_live_binding_retains_scene_slots_solver_and_overlay_refusals() {
    let original = decode_png(HIGHLIGHTED_CHALLENGE).unwrap();
    for (x, y, w, h) in [
        (20, 46, 24, 22),
        (90, 46, 27, 25),
        (CELL_X[2] + 30, 113, 4, 3),
        (FOUNDATION_X[1] + 30, 113, 4, 3),
    ] {
        let mut interrupted = original.clone();
        paint(&mut interrupted, x, y, w, h, [20, 100, 20]);
        assert!(playback_initial(&interrupted).is_err(), "{x},{y}");
    }
    let mut solver = original.clone();
    for y in [36, 85] {
        paint(&mut solver, 828, y, 263, 1, [220, 180, 20]);
    }
    let error = playback_initial(&solver).unwrap_err();
    assert_eq!(error.kind, FailureKind::UnsupportedScene);
    assert!(error.cause.contains("Solver"));
    for png in [
        include_bytes!("../../tests/fixtures/freecell-FC09.png").as_slice(),
        include_bytes!("../../tests/fixtures/freecell-FC12.png").as_slice(),
    ] {
        assert!(playback_initial(&decode_png(png).unwrap()).is_err());
    }
}
