//! Retained native normal ordinals and explicitly synthetic corruption tests.
//!
//! Complete native references calibrate all three current ordinals. A
//! literal header/footer observation neither authenticates Pyramid/card scenes
//! nor grants mode, acquisition, terminal or input authority.

use super::super::observe_normal;
use super::*;
use crate::capture::{PixelFormat, decode_png};


/// Exact current-start native original; the external source PNG has the same bytes and hash.
const CURRENT_START: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/current-start.png");


/// Exact release-118 entry refusal, retained before its temporary packet can be cleaned.
const GRANDMASTER_REFUSAL: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-objectives/normal-grandmaster-refused.png");


/// Seven distinct original PNG files, all directly classified as literal normal ordinal one.
const POSITIVES: [&[u8]; 7] = [
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/current-exhausted.png"),
    CURRENT_START,
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/issue1-20.png"),
    include_bytes!("../../../tests/fixtures/pyramid-standard-progress/normal-one-variant.png"),
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/gate3-exhausted-refusal.png"),
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/gate3-start-refusal.png"),
    include_bytes!("../../../tests/fixtures/pyramid-recon/originals/gate3-undo-baseline.png"),
];


/// Existing challenge/objective, Perfect!, challenge-complete and gold reward originals.
const NEGATIVES: [(&str, &[u8]); 11] = [
    (
        "centred-zero-of-three",
        include_bytes!("../../../tests/fixtures/pyramid-progress/zero-of-three.png"),
    ),
    (
        "centred-one-of-three",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/restored.png"),
    ),
    (
        "centred-two-of-three",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png"),
    ),
    (
        "twos-four-of-twelve",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v9/baseline.png"),
    ),
    (
        "twos-eight-of-twelve",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v11/baseline.png"),
    ),
    (
        "score-objective-ten-failure",
        include_bytes!("../../../tests/fixtures/pyramid-recon/native-v12/ten-failure-baseline.png"),
    ),
    (
        "score-objective-fifth-failure",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v12/fifth-failure-baseline.png"
        ),
    ),
    (
        "daily-target-two",
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/baseline.png"
        ),
    ),
    (
        "aces-challenge-complete",
        include_bytes!("../../../tests/fixtures/pyramid-progress/negative-aces-complete.png"),
    ),
    (
        "three-of-three-perfect",
        include_bytes!("../../../tests/fixtures/pyramid-progress/negative-three-of-three.png"),
    ),
    (
        "pyramid-gold-reward",
        include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png"),
    ),
];


/// Decode exact retained originals through the production PNG codec.
fn native(bytes: &[u8]) -> CapturedFrame {
    decode_png(bytes).expect("retained normal-header native PNG")
}


/// Explicit result for the single calibrated normal ordinal, not a general ordinal conversion.
fn zero_completed() -> Option<BoardCount> {
    Some(BoardCount {
        completed: 0,
        target: 3,
    })
}


/// Mutate one representable RGB channel distance without changing alpha, row layout or other pixels.
fn change_channel(frame: &mut CapturedFrame, x: u32, y: u32, channel: usize, difference: u8) {
    assert!(channel < 3);
    let offset = y as usize * frame.stride + x as usize * 4 + channel;
    let value = frame
        .pixels
        .get_mut(offset)
        .expect("bounded derivative channel");
    *value = if *value <= u8::MAX - difference {
        *value + difference
    } else {
        *value - difference
    };
}


/// Erase only a checked measured patch; this is a labelled corruption, not new native evidence.
fn erase_patch(frame: &mut CapturedFrame, patch: HeaderPatch) {
    frame
        .as_view()
        .expect("valid derivative")
        .checked_bounds(patch.bounds)
        .expect("bounded patch");
    for y in patch.bounds.y..patch.bounds.bottom() {
        for x in patch.bounds.x..patch.bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].fill(0);
        }
    }
}


/// Every retained normal ordinal-one source reaches the public production observer.
#[test]
fn production_observer_accepts_seven_retained_normal_ordinal_one_originals() {
    for bytes in POSITIVES {
        assert_eq!(observe_normal(&native(bytes)), Ok(zero_completed()));
    }
}


/// The actual refused frame has a valid header and a distinct complete measured footer.
#[test]
fn retained_grandmaster_entry_refusal_has_a_separately_measured_footer() {
    let frame = native(GRANDMASTER_REFUSAL);
    let view = frame.as_view().expect("valid exact failure frame");
    assert_eq!(
        matches_patch_with_tolerance(view, NORMAL_HEADERS[0].1, HEADER_TOLERANCE),
        Ok(true)
    );
    assert_eq!(
        matches_patch_with_tolerance(view, NEW_FOOTER, FOOTER_TOLERANCE),
        Ok(false)
    );
    assert_eq!(
        matches_patch_with_tolerance(view, GRANDMASTER_NEW_FOOTER, 0),
        Ok(true)
    );
    assert_eq!(observe_normal(&frame), Ok(zero_completed()));
    assert_eq!(super::super::observe(&frame), Ok(None));

    for patch in [NORMAL_HEADERS[0].1, GRANDMASTER_NEW_FOOTER] {
        let mut damaged = native(GRANDMASTER_REFUSAL);
        erase_patch(&mut damaged, patch);
        assert_eq!(observe_normal(&damaged), Ok(None));
    }
}


/// Both bundled RGB rectangles equal their source crops exactly, before any tolerance is applied.
#[test]
fn raw_rgb_references_equal_the_current_start_original_crops() {
    let frame = native(CURRENT_START);
    let view = frame.as_view().expect("valid exact source");
    for patch in [NORMAL_HEADERS[0].1, NEW_FOOTER] {
        assert_eq!(matches_patch_with_tolerance(view, patch, 0), Ok(true));
    }
}


/// Tolerances are exactly the largest measured differences across the finite source corpus.
#[test]
fn complete_source_patches_reproduce_measured_twelve_and_ten_channel_bounds() {
    for (patch, expected_maximum) in [
        (NORMAL_HEADERS[0].1, HEADER_TOLERANCE),
        (NEW_FOOTER, FOOTER_TOLERANCE),
    ] {
        let mut maximum = 0;
        for bytes in POSITIVES {
            let frame = native(bytes);
            let view = frame.as_view().expect("valid measured native original");
            for (index, expected) in patch.rgb.as_chunks::<3>().0.iter().enumerate() {
                let x = patch.bounds.x + index as u32 % patch.bounds.width;
                let y = patch.bounds.y + index as u32 / patch.bounds.width;
                let actual = view.pixel_rgb(x, y).expect("bounded measured source pixel");
                for (actual, expected) in actual.into_iter().zip(expected) {
                    maximum = maximum.max(actual.abs_diff(*expected));
                }
            }
        }
        assert_eq!(maximum, expected_maximum);
    }
}


/// Other real target grammars and retained completion/reward scenes supply no normal ordinal.
#[test]
fn real_challenge_objective_and_terminal_artwork_remain_unknown() {
    for (label, bytes) in NEGATIVES {
        assert_eq!(observe_normal(&native(bytes)), Ok(None), "{label}");
    }
}


/// The older centred reader remains independent of the new normal grammar and mapping.
#[test]
fn centred_challenge_and_normal_grammars_do_not_substitute_for_each_other() {
    assert_eq!(super::super::observe(&native(CURRENT_START)), Ok(None));
    for (bytes, completed) in [
        (
            include_bytes!("../../../tests/fixtures/pyramid-progress/zero-of-three.png").as_slice(),
            0,
        ),
        (
            include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/restored.png")
                .as_slice(),
            1,
        ),
        (
            include_bytes!("../../../tests/fixtures/pyramid-recon/native-v13/recon-king.png")
                .as_slice(),
            2,
        ),
    ] {
        let frame = native(bytes);
        assert_eq!(
            super::super::observe(&frame),
            Ok(Some(BoardCount {
                completed,
                target: 3
            }))
        );
        assert_eq!(observe_normal(&frame), Ok(None));
    }
}


/// Losing either complete measured rectangle prevents an otherwise positive observation.
#[test]
fn complete_normal_header_and_new_footer_are_both_required() {
    for patch in [NORMAL_HEADERS[0].1, NEW_FOOTER] {
        let mut frame = native(CURRENT_START);
        erase_patch(&mut frame, patch);
        assert_eq!(observe_normal(&frame), Ok(None));
    }
}


/// All channels honour each rectangle's inclusive bound and refuse its next integer distance.
#[test]
fn all_rgb_channels_use_inclusive_measured_bounds_and_reject_the_next_value() {
    for (patch, tolerance) in [
        (NORMAL_HEADERS[0].1, HEADER_TOLERANCE),
        (NEW_FOOTER, FOOTER_TOLERANCE),
    ] {
        for channel in 0..3 {
            let mut inside = native(CURRENT_START);
            change_channel(
                &mut inside,
                patch.bounds.x,
                patch.bounds.y,
                channel,
                tolerance,
            );
            assert_eq!(observe_normal(&inside), Ok(zero_completed()));
            let mut outside = native(CURRENT_START);
            change_channel(
                &mut outside,
                patch.bounds.x,
                patch.bounds.y,
                channel,
                tolerance + 1,
            );
            assert_eq!(observe_normal(&outside), Ok(None));
        }
    }
}


/// No background, final pixel or RGB channel is masked out of a required rectangle.
#[test]
fn every_required_pixel_rejects_one_channel_beyond_its_measured_bound() {
    let mut frame = native(CURRENT_START);
    for (patch, tolerance) in [
        (NORMAL_HEADERS[0].1, HEADER_TOLERANCE),
        (NEW_FOOTER, FOOTER_TOLERANCE),
    ] {
        for index in 0..patch.rgb.len() / 3 {
            let x = patch.bounds.x + index as u32 % patch.bounds.width;
            let y = patch.bounds.y + index as u32 / patch.bounds.width;
            let channel = index % 3;
            let offset = y as usize * frame.stride + x as usize * 4 + channel;
            let original = frame.pixels[offset];
            change_channel(&mut frame, x, y, channel, tolerance + 1);
            assert_eq!(
                matches_patch_with_tolerance(
                    frame.as_view().expect("valid derivative"),
                    patch,
                    tolerance
                ),
                Ok(false),
                "required pixel {index}, channel {channel}"
            );
            frame.pixels[offset] = original;
        }
    }
    assert_eq!(observe_normal(&frame), Ok(zero_completed()));
}


/// Header words, the ordinal and target digits, footer text and negative space are all required.
#[test]
fn damaged_words_ordinal_target_footer_and_background_are_unknown() {
    for (x, y, tolerance) in [
        (1_670, 60, HEADER_TOLERANCE),
        (1_756, 60, HEADER_TOLERANCE),
        (1_804, 60, HEADER_TOLERANCE),
        (1_817, 77, HEADER_TOLERANCE),
        (240, 1_012, FOOTER_TOLERANCE),
        (269, 1_028, FOOTER_TOLERANCE),
    ] {
        let mut frame = native(CURRENT_START);
        change_channel(&mut frame, x, y, 1, tolerance + 1);
        assert_eq!(
            observe_normal(&frame),
            Ok(None),
            "damaged native pixel ({x},{y})"
        );
    }
}


/// Metadata-only overlapping labels cannot choose a count; no new ordinal artwork is generated.
#[test]
fn ambiguous_different_labels_refuse_without_a_nearest_label_fallback() {
    let frame = native(CURRENT_START);
    let ambiguous = [
        NORMAL_HEADERS[0],
        (
            BoardCount {
                completed: 1,
                target: 3,
            },
            NORMAL_HEADERS[0].1,
        ),
    ];
    assert_eq!(
        match_header(frame.as_view().expect("valid source"), &ambiguous),
        Ok(None)
    );
    assert_eq!(NORMAL_HEADERS.len(), 3);
}


/// All new native originals use literal complete patches and explicit current-to-completed mapping.
#[test]
fn three_original_ordinals_are_distinct_and_keep_challenge_counts_separate() {
    for (completed, bytes) in [
        include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-1.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-2.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-standard-series/normal-3.png").as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let frame = native(bytes);
        assert_eq!(
            observe_normal(&frame),
            Ok(Some(BoardCount {
                completed: completed as u8,
                target: 3
            }))
        );
        assert_eq!(super::super::observe(&frame), Ok(None));
        let view = frame.as_view().expect("valid original ordinal");
        for (index, (_, patch)) in NORMAL_HEADERS.into_iter().enumerate() {
            assert_eq!(
                matches_patch_with_tolerance(view, patch, HEADER_TOLERANCE),
                Ok(index == completed)
            );
        }
        for patch in [NORMAL_HEADERS[completed].1, NEW_FOOTER] {
            let mut damaged = native(bytes);
            erase_patch(&mut damaged, patch);
            assert_eq!(observe_normal(&damaged), Ok(None));
        }
    }
}


/// Frame storage is checked before interpretation, including arithmetic overflow and truncation.
#[test]
fn invalid_layout_stride_and_storage_return_errors() {
    let mut truncated = native(CURRENT_START);
    truncated.pixels.pop();
    assert!(observe_normal(&truncated).is_err());
    let mut short_stride = native(CURRENT_START);
    short_stride.stride -= 1;
    assert!(observe_normal(&short_stride).is_err());
    let mut overflow_stride = native(CURRENT_START);
    overflow_stride.stride = usize::MAX;
    assert!(observe_normal(&overflow_stride).is_err());
}


/// Valid unsupported geometry never becomes a scaled or partially matching normal header.
#[test]
fn valid_unsupported_geometry_returns_unknown() {
    let mut shorter = native(CURRENT_START);
    shorter.height -= 1;
    assert_eq!(observe_normal(&shorter), Ok(None));
    let empty = CapturedFrame {
        width: 0,
        height: 0,
        stride: 0,
        format: PixelFormat::Rgba8,
        pixels: Vec::new(),
    };
    assert_eq!(observe_normal(&empty), Ok(None));
}


/// Row padding and alpha carry no evidence; the observer reads the original RGB layout directly.
#[test]
fn padded_rows_and_alpha_changes_preserve_exact_rgb_observation() {
    let source = native(CURRENT_START);
    let stride = source.stride + 37;
    let mut pixels = vec![193; stride * source.height as usize];
    for y in 0..source.height as usize {
        pixels[y * stride..y * stride + source.stride]
            .copy_from_slice(&source.pixels[y * source.stride..(y + 1) * source.stride]);
        for rgba in pixels[y * stride..y * stride + source.stride]
            .as_chunks_mut::<4>()
            .0
        {
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
    assert_eq!(observe_normal(&padded), Ok(zero_completed()));
}


/// Header observations cannot authenticate untouched card/deck pixels or a supported board scene.
#[test]
fn unmeasured_card_pixels_do_not_supply_scene_or_deck_authority() {
    let mut frame = native(CURRENT_START);
    change_channel(&mut frame, 960, 600, 0, 100);
    assert_eq!(observe_normal(&frame), Ok(zero_completed()));
}


/// Both retained actual failures supply complete card evidence and the same literal normal count.
#[test]
fn native_153_reported_entry_frames_have_complete_shared_identity() {
    use crate::pyramid_recon::vision::{ReconScene, ReconVision, StartingReadiness};
    let reader = ReconVision::bundled_native_v23().unwrap();
    let images: [&[u8]; 2] = [
        include_bytes!("../../../tests/fixtures/pyramid-normal-entry-153/automatic-entry.png"),
        include_bytes!("../../../tests/fixtures/pyramid-normal-entry-153/manual-entry.png"),
    ];
    for bytes in images {
        let frame = native(bytes);
        assert_eq!(observe_normal(&frame).unwrap(), zero_completed());
        assert_eq!(observe_header(&frame).unwrap(), zero_completed());
        let ReconScene::Board(board) = reader.observe(&frame).unwrap() else {
            panic!("complete actual board")
        };
        assert_eq!(board.tableau.len(), 28);
        assert!(matches!(
            reader.starting_readiness(&frame).unwrap(),
            StartingReadiness::Ready(_)
        ));
    }
}


/// Complete appearances may share one count, while contradictory counts and missing patches refuse.
#[test]
fn native_153_header_appearance_keeps_identity_and_footer_guards() {
    let original = native(include_bytes!(
        "../../../tests/fixtures/pyramid-normal-entry-153/automatic-entry.png"
    ));
    let patch = NORMAL_APPEARANCES[3].1;
    let count = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(
        match_header(
            original.as_view().unwrap(),
            &[(count, patch), (count, patch)]
        )
        .unwrap(),
        Some(count)
    );
    assert_eq!(
        match_header(
            original.as_view().unwrap(),
            &[
                (count, patch),
                (
                    BoardCount {
                        completed: 1,
                        target: 3
                    },
                    patch
                )
            ]
        )
        .unwrap(),
        None
    );
    let mut footer = original.clone();
    erase_patch(&mut footer, NEW_FOOTER);
    assert_eq!(observe_normal(&footer).unwrap(), None);
    let mut header = original;
    erase_patch(&mut header, NORMAL_HEADERS[0].1);
    assert_eq!(observe_normal(&header).unwrap(), None);
}


mod testing_159;
