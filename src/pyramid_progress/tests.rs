//! Finite retained-frame and corruption regressions for literal board-count observations.
//!
//! Native challenge headers exercise literal values and rejection only. None of
//! these tests supplies standard-game classification, terminal or input authority.

use super::*;
use crate::capture::decode_png;


/// Exact first native zero-of-three frame retained for this reader.
const ZERO: &[u8] = include_bytes!("../../tests/fixtures/pyramid-progress/zero-of-three.png");


/// Existing restored native one-of-three frame, referenced without duplication.
const ONE: &[u8] = include_bytes!("../../tests/fixtures/pyramid-recon/native-v13/restored.png");


/// Existing partial selected one-of-three frame is an independent header observation.
const SELECTED_ONE: &[u8] =
    include_bytes!("../../tests/fixtures/pyramid-recon/native-v13/selected.png");


/// Existing native two-of-three frame, referenced without changing that fixture.
const TWO: &[u8] = include_bytes!("../../tests/fixtures/pyramid-recon/native-v13/recon-king.png");


/// Decode retained original pixels through the production PNG codec.
fn native(bytes: &[u8]) -> CapturedFrame {
    decode_png(bytes).expect("retained native PNG")
}


/// Replace checked native RGB bytes without changing alpha, layout or other pixels.
fn replace_rgb(frame: &mut CapturedFrame, bounds: PixelRect, rgb: &[u8]) {
    frame
        .as_view()
        .expect("valid native derivative")
        .checked_bounds(bounds)
        .expect("bounded native derivative");
    assert_eq!(
        rgb.len(),
        bounds.width as usize * bounds.height as usize * 3
    );
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
        let x = bounds.x as usize + index % bounds.width as usize;
        let y = bounds.y as usize + index / bounds.width as usize;
        let offset = y * frame.stride + x * 4;
        frame.pixels[offset..offset + 3].copy_from_slice(pixel);
    }
}


/// Change exactly one RGB channel by a representable positive or negative distance.
fn change_channel(frame: &mut CapturedFrame, x: u32, y: u32, channel: usize, difference: u8) {
    assert!(channel < RGB_CHANNELS);
    let offset = y as usize * frame.stride + x as usize * 4 + channel;
    let value = frame
        .pixels
        .get_mut(offset)
        .expect("checked derivative channel");
    *value = if *value <= u8::MAX - difference {
        *value + difference
    } else {
        *value - difference
    };
}


/// All three supported literals are read from complete retained native originals.
#[test]
fn retained_native_counts_are_literal_zero_one_two() {
    for (bytes, completed) in [(ZERO, 0), (ONE, 1), (TWO, 2)] {
        assert_eq!(
            observe(&native(bytes)),
            Ok(Some(BoardCount {
                completed,
                target: 3,
            }))
        );
    }
}


/// A partial selected board can carry the same literal header; the caller owns scene authority.
#[test]
fn independent_selected_header_is_literal_one_without_board_authority() {
    assert_eq!(
        observe(&native(SELECTED_ONE)),
        Ok(Some(BoardCount {
            completed: 1,
            target: 3,
        }))
    );
}


/// Different real objectives, target counts and terminal layouts remain unsupported.
#[test]
fn real_native_other_objectives_and_terminal_counts_are_unknown() {
    for bytes in [
        include_bytes!(
            "../../tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/baseline.png"
        )
        .as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-recon/native-v9/baseline.png").as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-recon/native-v11/baseline.png").as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-recon/native-v12/ten-failure-baseline.png")
            .as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-recon/native-v12/fifth-failure-baseline.png")
            .as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-progress/negative-aces-complete.png")
            .as_slice(),
        include_bytes!("../../tests/fixtures/pyramid-progress/negative-three-of-three.png")
            .as_slice(),
        include_bytes!("../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png")
            .as_slice(),
    ] {
        assert_eq!(observe(&native(bytes)), Ok(None));
    }
}


/// A supported fraction cannot replace either required objective word.
#[test]
fn another_objective_with_a_known_fraction_still_refuses() {
    let mut frame = native(include_bytes!(
        "../../tests/fixtures/pyramid-recon/native-v9/baseline.png"
    ));
    replace_rgb(&mut frame, FRACTION_PATCHES[0].1.bounds, ZERO_RGB);
    assert_eq!(observe(&frame), Ok(None));
}


/// Damage to either word, the completed digit or the target digit prevents acceptance.
#[test]
fn damaged_words_completed_digit_and_target_digit_refuse() {
    for (x, y) in [(840, 51), (936, 51), (1_050, 60), (1_082, 60)] {
        let mut frame = native(ZERO);
        change_channel(&mut frame, x, y, 0, CHANNEL_TOLERANCE + 1);
        assert_eq!(observe(&frame), Ok(None), "damaged pixel ({x},{y})");
    }
}


/// Every RGB channel uses the inclusive bound in both labels and the full fraction.
#[test]
fn every_rgb_channel_obeys_inclusive_tolerance_and_next_value_refuses() {
    for patch in [LABEL_PATCHES[0], LABEL_PATCHES[1], FRACTION_PATCHES[0].1] {
        for channel in 0..RGB_CHANNELS {
            let mut inside = native(ZERO);
            change_channel(
                &mut inside,
                patch.bounds.x,
                patch.bounds.y,
                channel,
                CHANNEL_TOLERANCE,
            );
            assert_eq!(
                observe(&inside),
                Ok(Some(BoardCount {
                    completed: 0,
                    target: 3,
                }))
            );
            let mut outside = native(ZERO);
            change_channel(
                &mut outside,
                patch.bounds.x,
                patch.bounds.y,
                channel,
                CHANNEL_TOLERANCE + 1,
            );
            assert_eq!(observe(&outside), Ok(None));
        }
    }
}


/// Background and final patch pixels are required; no negative-space mask exists.
#[test]
fn background_and_last_pixels_of_each_patch_are_required() {
    for patch in [LABEL_PATCHES[0], LABEL_PATCHES[1], FRACTION_PATCHES[0].1] {
        for (x, y) in [
            (patch.bounds.x, patch.bounds.y),
            (patch.bounds.right() - 1, patch.bounds.bottom() - 1),
        ] {
            let mut frame = native(ZERO);
            change_channel(&mut frame, x, y, 2, CHANNEL_TOLERANCE + 1);
            assert_eq!(observe(&frame), Ok(None));
        }
    }
}


/// Each required pixel rejects a one-channel difference of ten; no background pixel is skipped.
#[test]
fn every_required_pixel_rejects_a_channel_difference_of_ten() {
    let mut frame = native(ZERO);
    for patch in [LABEL_PATCHES[0], LABEL_PATCHES[1], FRACTION_PATCHES[0].1] {
        for index in 0..patch.rgb.len() / RGB_CHANNELS {
            let x = patch.bounds.x as usize + index % patch.bounds.width as usize;
            let y = patch.bounds.y as usize + index / patch.bounds.width as usize;
            let channel = index % RGB_CHANNELS;
            let offset = y * frame.stride + x * 4 + channel;
            let original = frame.pixels[offset];
            change_channel(
                &mut frame,
                x as u32,
                y as u32,
                channel,
                CHANNEL_TOLERANCE + 1,
            );
            assert_eq!(
                matches_patch(frame.as_view().expect("valid mutation"), patch),
                Ok(false),
                "required patch pixel {index}, channel {channel}"
            );
            frame.pixels[offset] = original;
        }
    }
    assert_eq!(
        observe(&frame),
        Ok(Some(BoardCount {
            completed: 0,
            target: 3
        }))
    );
}


/// Known literal references cannot overlap within two inclusive tolerance intervals.
#[test]
fn calibrated_fractions_have_disjoint_tolerance_neighbourhoods() {
    for (index, (_, left)) in FRACTION_PATCHES.iter().enumerate() {
        for (_, right) in FRACTION_PATCHES.iter().skip(index + 1) {
            assert!(left.rgb.iter().zip(right.rgb).any(|(left, right)| {
                u16::from(left.abs_diff(*right)) > u16::from(CHANNEL_TOLERANCE) * 2
            }));
        }
    }
}


/// Missing and partly mixed fraction artwork does not become a known count.
#[test]
fn erased_or_mixed_fraction_is_unknown() {
    let mut erased = native(ZERO);
    let bounds = FRACTION_PATCHES[0].1.bounds;
    replace_rgb(&mut erased, bounds, &[0; 3_864]);
    assert_eq!(observe(&erased), Ok(None));
    let mut mixed = *ZERO_RGB;
    let middle = mixed.len() / 2;
    mixed[..middle].copy_from_slice(&ONE_RGB[..middle]);
    let mut frame = native(ZERO);
    replace_rgb(&mut frame, bounds, &mixed);
    assert_eq!(observe(&frame), Ok(None));
}


/// Invalid storage and overflowed row arithmetic fail before any pixel interpretation.
#[test]
fn invalid_layout_stride_and_truncation_return_errors() {
    let mut truncated = native(ZERO);
    truncated.pixels.pop();
    assert!(observe(&truncated).is_err());
    let mut short_stride = native(ZERO);
    short_stride.stride -= 1;
    assert!(observe(&short_stride).is_err());
    let mut overflow_stride = native(ZERO);
    overflow_stride.stride = usize::MAX;
    assert!(observe(&overflow_stride).is_err());
}


/// Valid non-native dimensions remain unknown, including an empty valid frame view.
#[test]
fn valid_unsupported_dimensions_are_unknown() {
    let mut shorter = native(ZERO);
    shorter.height -= 1;
    assert_eq!(observe(&shorter), Ok(None));
    let empty = CapturedFrame {
        width: 0,
        height: 0,
        stride: 0,
        format: crate::capture::PixelFormat::Rgba8,
        pixels: Vec::new(),
    };
    assert_eq!(observe(&empty), Ok(None));
}


/// Legitimate padded rows and alpha changes preserve the same RGB header evidence.
#[test]
fn padded_stride_and_alpha_are_ignored_without_repacking_the_observer() {
    let source = native(ZERO);
    let stride = source.stride + 37;
    let mut pixels = vec![193; stride * source.height as usize];
    for y in 0..source.height as usize {
        pixels[y * stride..y * stride + source.stride]
            .copy_from_slice(&source.pixels[y * source.stride..(y + 1) * source.stride]);
    }
    for pixel in pixels.chunks_mut(stride) {
        for rgba in pixel[..source.stride].as_chunks_mut::<4>().0 {
            rgba[3] = 0;
        }
    }
    let padded = CapturedFrame {
        width: source.width,
        height: source.height,
        stride,
        format: source.format,
        pixels,
    };
    assert_eq!(observe(&padded), observe(&source));
}


/// Invalid static patch metadata is rejected rather than allowing an incomplete match.
#[test]
fn malformed_reference_and_outside_bounds_are_errors() {
    let frame = native(ZERO);
    let view = frame.as_view().expect("valid retained frame");
    assert!(
        matches_patch(
            view,
            HeaderPatch {
                bounds: PixelRect::new(839, 50, 96, 28),
                rgb: &[0; 3],
            }
        )
        .is_err()
    );
    assert!(
        matches_patch(
            view,
            HeaderPatch {
                bounds: PixelRect::new(u32::MAX, 50, 96, 28),
                rgb: LEFT_LABEL_RGB,
            }
        )
        .is_err()
    );
}
