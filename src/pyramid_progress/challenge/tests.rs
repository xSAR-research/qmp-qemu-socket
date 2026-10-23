//! Real native counter observations and explicitly labelled malformed derivatives.

use super::*;
use crate::capture::decode_png;


/// Fourteen exposed original scenes, including changing counts and four actual Score counters.
const SOURCES: [(&str, &[u8], ChallengeKind, u32, u32); 14] = [
    (
        "boards-two",
        include_bytes!("../../../tests/fixtures/pyramid-objectives/boards-zero-of-two.png"),
        ChallengeKind::Boards,
        0,
        2,
    ),
    (
        "boards-three",
        include_bytes!("../../../tests/fixtures/pyramid-progress/zero-of-three.png"),
        ChallengeKind::Boards,
        0,
        3,
    ),
    (
        "boards-one",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/restored.png"),
        ChallengeKind::Boards,
        1,
        3,
    ),
    (
        "boards-second",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"),
        ChallengeKind::Boards,
        2,
        3,
    ),
    (
        "boards-complete",
        include_bytes!("../../../tests/fixtures/pyramid-progress/negative-three-of-three.png"),
        ChallengeKind::Boards,
        3,
        3,
    ),
    (
        "twos-four",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v9/baseline.png"),
        ChallengeKind::Cards,
        4,
        12,
    ),
    (
        "twos-eight",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v11/baseline.png"),
        ChallengeKind::Cards,
        8,
        12,
    ),
    (
        "fours-four",
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png"),
        ChallengeKind::Cards,
        4,
        12,
    ),
    (
        "sevens-zero",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v16/gold-seven.png"),
        ChallengeKind::Cards,
        0,
        9,
    ),
    (
        "score-ten",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v12/ten-failure-baseline.png"),
        ChallengeKind::Score,
        2560,
        4200,
    ),
    (
        "score-fifth",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v12/fifth-failure-baseline.png"
        ),
        ChallengeKind::Score,
        1280,
        3000,
    ),
    (
        "score-final-right",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v17/final-clear-right-card.png"
        ),
        ChallengeKind::Score,
        1280,
        2900,
    ),
    (
        "score-final-empty",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v17/final-clear-empty-stock.png"
        ),
        ChallengeKind::Score,
        2570,
        2900,
    ),
    (
        "fours-zero-of-seven",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v21/fours-entry-last-observed.png"
        ),
        ChallengeKind::Cards,
        0,
        7,
    ),
];


/// Decode unchanged native bytes through the production capture codec.
fn native(bytes: &[u8]) -> CapturedFrame {
    decode_png(bytes).expect("retained native challenge PNG")
}


/// Erase a bounded RGB rectangle only; this is corruption, never additional native evidence.
fn erase(frame: &mut CapturedFrame, bounds: PixelRect) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let index = y as usize * frame.stride + x as usize * 4;
            frame.pixels[index..index + 3].fill(0);
        }
    }
}


/// Each real objective is classified independently of its changing numeric values.
#[test]
fn fourteen_native_counters_keep_exact_objective_and_values() {
    for (name, bytes, kind, current, target) in SOURCES {
        assert_eq!(
            observe(&native(bytes)),
            Ok(ChallengeProgress::new(kind, current, target)),
            "{name}"
        );
    }
}


/// Normal ordinals, the Solver banner, dimmed terminals and rewards are separate evidence.
#[test]
fn normal_solver_and_terminal_layouts_are_not_active_challenge_counters() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/pyramid-objectives/normal-one.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/normal-grandmaster-refused.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/solver-layout.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-progress/negative-aces-complete.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png")
            .as_slice(),
    ] {
        assert_eq!(observe(&native(bytes)), Ok(None));
    }
}


/// Literal label crops include every source pixel and exclude their changing decimal bands.
#[test]
fn all_bundled_whole_labels_equal_their_own_native_sources() {
    let sources: Vec<_> = SOURCES
        .into_iter()
        .filter(|(_, _, kind, _, _)| *kind != ChallengeKind::Cards)
        .collect();
    assert_eq!(sources.len(), templates::LABELS.len());
    for ((name, bytes, kind, _, _), label) in sources.into_iter().zip(&templates::LABELS) {
        assert_eq!(kind, label.kind, "{name}");
        assert_eq!(
            matches_patch_with_tolerance(native(bytes).as_view().unwrap(), label.patch, 0),
            Ok(true),
            "{name}"
        );
    }
}


/// Neither valid numbers without their words nor a valid label without numbers is accepted.
#[test]
fn complete_words_and_complete_counter_are_both_required() {
    for (name, bytes, kind, _, _) in SOURCES {
        let mut missing_label = native(bytes);
        erase(&mut missing_label, PixelRect::new(814, 48, 132, 32));
        assert_eq!(observe(&missing_label), Ok(None), "{name}");
        let mut missing_count = native(bytes);
        let bounds = if kind == ChallengeKind::Score {
            PixelRect::new(958, 48, 142, 32)
        } else {
            PixelRect::new(1038, 48, 72, 32)
        };
        erase(&mut missing_count, bounds);
        assert_eq!(observe(&missing_count), Ok(None), "{name}");
    }
}


/// Removing the actual slash cannot be repaired from the expected objective or target.
#[test]
fn missing_slash_is_not_inferred_from_other_digits() {
    let mut frame = native(SOURCES[0].1);
    erase(&mut frame, PixelRect::new(1062, 56, 10, 19));
    assert_eq!(observe(&frame), Ok(None));
}


/// Extra unexplained ink at the counter edge refuses rather than being ignored as noise.
#[test]
fn edge_touching_unexplained_component_refuses() {
    let mut frame = native(SOURCES[0].1);
    let index = 48 * frame.stride + 1038 * 4;
    frame.pixels[index..index + 3].fill(255);
    assert_eq!(observe(&frame), Ok(None));
}


/// Overlong values, impossible targets and wrong objective units never become a local progress counter.
#[test]
fn literal_progress_keeps_positive_target_units_and_monotonicity() {
    assert_eq!(ChallengeProgress::new(ChallengeKind::Boards, 0, 0), None);
    assert_eq!(ChallengeProgress::new(ChallengeKind::Boards, 3, 2), None);
    assert_eq!(
        ChallengeProgress::new(ChallengeKind::Score, 1_000_000, 2),
        None
    );
    let cards = ChallengeProgress::new(ChallengeKind::Cards, 4, 12).unwrap();
    let boards = ChallengeProgress::new(ChallengeKind::Boards, 4, 12).unwrap();
    assert!(!cards.follows(boards));
    assert!(cards.follows(cards));
    assert!(!cards.target_reached());
}


/// Invalid storage fails before interpretation; valid unsupported geometry remains unknown.
#[test]
fn invalid_storage_and_unsupported_geometry_do_not_supply_progress() {
    let mut truncated = native(SOURCES[0].1);
    truncated.pixels.pop();
    assert!(observe(&truncated).is_err());
    let mut short = native(SOURCES[0].1);
    short.height -= 1;
    assert_eq!(observe(&short), Ok(None));
}


/// Both retained Kings scenes supply the same whole-word hint without changing generic Cards.
#[test]
fn kings_default_hint_is_separate_from_generic_card_progress() {
    for (png, current) in [
        (
            include_bytes!("../../../tests/fixtures/pyramid-kings-default/kings-one.png")
                .as_slice(),
            0,
        ),
        (
            include_bytes!("../../../tests/fixtures/pyramid-kings-default/kings-two.png")
                .as_slice(),
            4,
        ),
    ] {
        let mut frame = native(png);
        assert_eq!(
            observe(&frame),
            Ok(ChallengeProgress::new(ChallengeKind::Cards, current, 18))
        );
        assert_eq!(is_kings_header(&frame), Ok(true));
        erase(&mut frame, PixelRect::new(814, 48, 40, 32));
        assert_eq!(is_kings_header(&frame), Ok(false));
    }
    for (_, png, _, _, _) in SOURCES {
        assert_eq!(is_kings_header(&native(png)), Ok(false));
    }
}
