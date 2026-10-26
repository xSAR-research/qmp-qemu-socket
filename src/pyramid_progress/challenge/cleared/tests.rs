//! Native shared-word cases and explicitly synthetic corruption/prefix regressions.

use super::*;
use crate::{capture::decode_png, pyramid_progress::ChallengeKind};


/// Decode an unchanged exposed native frame, never a generated header image.
fn native(bytes: &[u8]) -> crate::capture::CapturedFrame {
    decode_png(bytes).unwrap()
}


/// Return the genuine Single-board Sixes acquisition source used for this calibration.
fn sixes() -> crate::capture::CapturedFrame {
    native(include_bytes!(
        "../../../../tests/fixtures/pyramid-card-cleared/sixes.png"
    ))
}


/// Paint a bounded test rectangle; these mutations are never described as native evidence.
fn paint(frame: &mut crate::capture::CapturedFrame, bounds: PixelRect, rgb: [u8; 3]) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
        }
    }
}


/// Actual previously unsupported words supply one generic Card kind and their literal counters.
#[test]
fn shared_cleared_native_sixes_and_nines_need_no_rank_identity() {
    for (bytes, target) in [
        (
            include_bytes!("../../../../tests/fixtures/pyramid-card-cleared/sixes.png").as_slice(),
            12,
        ),
        (
            include_bytes!("../../../../tests/fixtures/pyramid-card-cleared/nines.png").as_slice(),
            15,
        ),
    ] {
        assert_eq!(
            super::super::observe(&native(bytes)),
            Ok(super::super::ChallengeProgress::new(
                ChallengeKind::Cards,
                0,
                target
            ))
        );
    }
}


/// Uninterpreted prefix changes cannot require a bank of thirteen rank names.
#[test]
fn shared_cleared_arbitrary_rank_prefix_pixels_do_not_select_or_block_the_kind() {
    for seed in 0_u8..13 {
        let mut frame = sixes();
        for x in 814..910 {
            paint(
                &mut frame,
                PixelRect::new(x, 48, 1, 32),
                [seed.wrapping_mul(x as u8), 17, 6],
            );
        }
        assert_eq!(
            super::super::observe(&frame),
            Ok(super::super::ChallengeProgress::new(
                ChallengeKind::Cards,
                0,
                12
            )),
            "synthetic prefix {seed}"
        );
    }
}


/// Losing any one complete letter withholds the shared-word observation.
#[test]
fn shared_cleared_missing_each_of_the_seven_letters_refuses() {
    for (left, width) in [
        (915, 10),
        (927, 3),
        (932, 12),
        (945, 11),
        (958, 8),
        (966, 12),
        (979, 12),
    ] {
        let mut frame = sixes();
        paint(&mut frame, PixelRect::new(left, 48, width, 32), [0, 0, 6]);
        assert_eq!(
            super::super::observe(&frame),
            Ok(None),
            "missing letter at {left}"
        );
    }
}


/// Corrupting a whole Boards label never reclassifies its intact shared word as Card.
#[test]
fn shared_cleared_reserved_boards_word_vetoes_a_damaged_literal_label() {
    for (x, y) in [(814, 48), (845, 64)] {
        let mut frame = native(include_bytes!(
            "../../../../tests/fixtures/pyramid-objectives/boards-zero-of-two.png"
        ));
        let offset = y * frame.stride + x * 4;
        frame.pixels[offset] ^= 128;
        assert_eq!(
            super::super::observe(&frame),
            Ok(None),
            "corruption at {x},{y}"
        );
    }
}


/// Shared words supply no target when the actual changing counter or banner is absent.
#[test]
fn shared_cleared_requires_actual_counter_and_native_banner() {
    for bounds in [
        PixelRect::new(1038, 48, 72, 32),
        PixelRect::new(814, 41, 292, 7),
    ] {
        let mut frame = sixes();
        paint(&mut frame, bounds, [0, 0, 0]);
        assert_eq!(super::super::observe(&frame), Ok(None));
    }
}


/// Joining adjacent ink to the word cannot be mistaken for a complete isolated caption word.
#[test]
fn shared_cleared_requires_both_word_boundaries() {
    for bounds in [
        PixelRect::new(910, 48, 5, 32),
        PixelRect::new(990, 48, 8, 32),
    ] {
        let mut frame = sixes();
        paint(&mut frame, bounds, [255, 255, 255]);
        assert_eq!(super::super::observe(&frame), Ok(None));
    }
}


/// The intensity template is precisely the documented native crop, not synthesised text.
#[test]
fn shared_cleared_template_preserves_the_exact_native_blue_intensities() {
    let frame = sixes();
    let view = frame.as_view().unwrap();
    let expected: Vec<u8> = (48..80)
        .flat_map(|y| {
            (912..996).map(move |x| view.pixel_rgb(x, y).unwrap()[2].saturating_sub(BLUE_FLOOR))
        })
        .collect();
    assert_eq!(CLEARED.ink, expected);
}


/// The actual Xbox Fours words/counter survive only the background-blended corner exclusion.
#[test]
fn native_141_xbox_header_keeps_words_counter_and_noncorner_border_positive() {
    let frame = native(include_bytes!(
        "../../../../tests/fixtures/pyramid-native-141/xbox-fours-entry.png"
    ));
    let expected = super::super::ChallengeProgress::new(ChallengeKind::Cards, 0, 9);
    assert_eq!(super::super::observe(&frame), Ok(expected));
    for bounds in [
        PixelRect::new(900, 41, 1, 1),
        PixelRect::new(900, 86, 1, 1),
        PixelRect::new(915, 48, 84, 32),
        PixelRect::new(1038, 48, 72, 32),
    ] {
        let mut bad = frame.clone();
        paint(&mut bad, bounds, [0; 3]);
        assert_eq!(super::super::observe(&bad), Ok(None), "{bounds:?}");
    }
}


/// The retained background case must read its whole Boards-cleared requirement.
#[test]
fn native_150_entry_background_boards_cleared() {
    let frame = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-entry-150/background-boards.png"
    ))
    .unwrap();
    let view = frame.as_view().unwrap();
    assert!(banner_matches(view).unwrap(), "actual native banner");
    assert!(
        origin(view, &COMPLETE_BOARDS).flatten().is_some(),
        "all six Boards letter cores and whole word"
    );
    assert!(
        origin(view, &CLEARED).flatten().is_some(),
        "all seven cleared letter cores and whole word"
    );
    assert!(
        observe_boards(view).unwrap(),
        "whole ordered words and their negative space"
    );
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&frame).unwrap(),
        crate::pyramid_progress::ChallengeProgress::new(
            crate::pyramid_progress::ChallengeKind::Boards,
            0,
            2
        )
    );
}


/// New/Grandmaster alone cannot invent a missing standard three-board ordinal.
#[test]
fn native_150_entry_missing_normal_ordinal_remains_unknown() {
    for bytes in [
        include_bytes!("../../../../tests/fixtures/pyramid-entry-150/normal-no-counter-4.png")
            .as_slice(),
        include_bytes!("../../../../tests/fixtures/pyramid-entry-150/normal-no-counter-5.png")
            .as_slice(),
    ] {
        let frame = crate::capture::decode_png(bytes).unwrap();
        assert_eq!(crate::pyramid_progress::observe_normal(&frame), Ok(None));
        assert_eq!(crate::pyramid_progress::observe_challenge(&frame), Ok(None));
    }
}


/// Genuine weak letter fringes do not turn red-only corruption in true background into ink.
#[test]
fn native_150_entry_background_keeps_negative_space_and_complete_words() {
    let source = native(include_bytes!(
        "../../../../tests/fixtures/pyramid-entry-150/background-boards.png"
    ));
    for x in [814, 945] {
        let mut corrupted = source.clone();
        let mut rgb = corrupted.as_view().unwrap().pixel_rgb(x, 48).unwrap();
        rgb[0] = rgb[0].saturating_add(40);
        paint(&mut corrupted, PixelRect::new(x, 48, 1, 1), rgb);
        assert!(!observe_boards(corrupted.as_view().unwrap()).unwrap());
        assert_eq!(
            crate::pyramid_progress::observe_challenge(&corrupted),
            Ok(None)
        );
    }
    for (word, x) in [(&COMPLETE_BOARDS, 841), (&CLEARED, 922)] {
        for &(dx, dy) in word.anchors {
            let mut missing = source.clone();
            paint(
                &mut missing,
                PixelRect::new(x + dx - 3, SEARCH.y + dy - 3, 7, 7),
                [161, 102, 6],
            );
            assert!(
                !observe_boards(missing.as_view().unwrap()).unwrap(),
                "missing complete letter at {},{}",
                x + dx,
                SEARCH.y + dy
            );
        }
    }
}
