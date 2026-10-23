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
