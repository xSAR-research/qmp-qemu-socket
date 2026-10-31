//! Quarter-card occurrence geometry, exact coverage boundaries and glyph independence.

use super::*;
use crate::capture::PixelFormat;


/// Synthetic native frame isolates coordinates from recognition or live input authority.
fn frame() -> CapturedFrame {
    CapturedFrame {
        width: 1920,
        height: 1080,
        stride: 1920 * 4,
        format: PixelFormat::Rgba8,
        pixels: [30, 40, 50, 255].repeat(1920 * 1080),
    }
}


/// Paint exactly one native RGB sample without interpreting alpha as effect evidence.
fn paint(frame: &mut CapturedFrame, x: usize, y: usize, rgb: [u8; 3]) {
    let offset = y * frame.stride + x * 4;
    frame.pixels[offset..offset + 3].copy_from_slice(&rgb);
}


/// Complete quarter traversal includes its last pixel and rejects malformed native inputs.
#[test]
fn quarter_170_geometry_and_complete_row_major_sampling() {
    let mut source = frame();
    let origin = (759, 678);
    assert_eq!(
        card_bounds(origin).unwrap(),
        PixelRect::new(762, 682, 66, 89)
    );
    paint(&mut source, 762, 682, [1, 2, 3]);
    paint(&mut source, 827, 770, [251, 252, 253]);
    let pixels = card_pixels(&source, origin).unwrap();
    assert_eq!(pixels.len(), 5874);
    assert_eq!(pixels[0], u32::from_le_bytes([1, 2, 3, 0]));
    assert_eq!(pixels[5873], u32::from_le_bytes([251, 252, 253, 0]));
    assert_eq!(
        crate::pyramid_recon::vision::stock_corner(&source, false).unwrap(),
        pixels
    );
    assert_eq!(
        crate::pyramid_recon::vision::card_corner(&source, origin).unwrap(),
        pixels
    );
    assert!(card_bounds((u32::MAX, 0)).is_err());
    assert!(card_bounds((0, u32::MAX)).is_err());
    assert!(card_pixels(&source, (1900, 1000)).is_err());
    source.width = 1919;
    assert!(card_pixels(&source, origin).is_err());
    source.width = 1920;
    source.pixels.pop();
    assert!(card_pixels(&source, origin).is_err());
}


/// Strict five-percent acceptance is 294 joint changes, with byte 5 still excluded.
#[test]
fn tolerance_171_strict_coverage_and_rgb_boundaries() {
    let before = vec![0; 5874];
    let mut after = before.clone();
    after[..293].fill(u32::from_le_bytes([6, 6, 6, 255]));
    let measured = compare_with_settings(&before, &after, Default::default()).unwrap();
    assert_eq!(
        (measured.changed_pixels, measured.allowed_changes),
        (293, 293)
    );
    assert!(!measured.exceeds_allowance());
    after[293] = u32::from_le_bytes([6, 6, 5, 255]);
    assert!(
        !compare_with_settings(&before, &after, Default::default())
            .unwrap()
            .exceeds_allowance()
    );
    after[293] = u32::from_le_bytes([6, 6, 6, 0]);
    assert!(
        compare_with_settings(&before, &after, Default::default())
            .unwrap()
            .exceeds_allowance()
    );
}


/// New evidence beyond the glyph corner contributes while excluded cursor pixels do not.
#[test]
fn quarter_170_added_area_and_cursor_exclusion_denominator() {
    let before = frame();
    let mut after = before.clone();
    let origin = (759, 678);
    for y in 682..771 {
        for x in 793..828 {
            paint(&mut after, x, y, [60, 70, 80]);
        }
    }
    let old_corner = PixelRect::new(762, 682, 31, 49);
    let old_before = sample(&before, old_corner, &[]).unwrap();
    let old_after = sample(&after, old_corner, &[]).unwrap();
    assert_eq!(old_before, old_after);
    let excluded = PixelRect::new(780, 723, 96, 96);
    let measured = region_counts(
        &before,
        &after,
        card_bounds(origin).unwrap(),
        &[excluded],
        Default::default(),
    )
    .unwrap();
    assert_eq!(measured.total_pixels, 5874 - 48 * 48);
    assert_eq!(measured.changed_pixels, 35 * 41);
    assert!(measured.exceeds_allowance());
}


/// Restore byte-original quarter crops into a synthetic native surround to test the shared sampler.
fn picture_frame(rgb: &[u8]) -> CapturedFrame {
    assert_eq!(rgb.len(), 5874 * 3);
    let mut result = frame();
    for (i, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
        paint(
            &mut result,
            762 + i % 66,
            682 + i / 66,
            [pixel[0], pixel[1], pixel[2]],
        );
    }
    result
}


/// Exposed same-theme J/Q/K suit controls use complete measured RGB quarter crops.
/// These aligned face comparisons establish no natural guest transition or live acceptance.
#[test]
fn quarter_170_retained_picture_suits_remain_distinct() {
    let pairs: [(&str, &[u8], &[u8], usize); 7] = [
        (
            "coral-jc-js",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/coral-jc-js-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/coral-jc-js-b.rgb"
            )),
            1025,
        ),
        (
            "coral-kh-kd",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/coral-kh-kd-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/coral-kh-kd-b.rgb"
            )),
            564,
        ),
        (
            "white-qc-qs",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-qc-qs-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-qc-qs-b.rgb"
            )),
            2229,
        ),
        (
            "white-kc-ks",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-kc-ks-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-kc-ks-b.rgb"
            )),
            2258,
        ),
        (
            "white-qh-qd",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-qh-qd-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/white-qh-qd-b.rgb"
            )),
            2095,
        ),
        (
            "cream-jc-js",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/cream-jc-js-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/cream-jc-js-b.rgb"
            )),
            1940,
        ),
        (
            "cream-jh-jd",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/cream-jh-jd-a.rgb"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pyramid-quarter-170/cream-jh-jd-b.rgb"
            )),
            2003,
        ),
    ];
    for (label, before, after, expected) in pairs {
        let a = picture_frame(before);
        let b = picture_frame(after);
        let measured = region_counts(
            &a,
            &b,
            card_bounds((759, 678)).unwrap(),
            &[],
            PyramidEffectSettings::new(13, 50).unwrap(),
        )
        .unwrap();
        assert_eq!(measured.total_pixels, 5874, "{label}");
        assert_eq!(measured.compared_pixels, 5874, "{label}");
        assert_eq!(measured.changed_pixels, expected, "{label}");
        assert_eq!(measured.allowed_changes, 293, "{label}");
        assert!(measured.exceeds_allowance(), "{label}");
        let tightened = region_counts(
            &a, &b, card_bounds((759, 678)).unwrap(), &[], Default::default(),
        ).unwrap();
        assert_eq!(tightened.compared_pixels, 5874, "{label}");
        assert!(tightened.changed_pixels >= measured.changed_pixels, "{label}");
        assert!(tightened.exceeds_allowance(), "{label}");
        assert!(
            !compare_with_settings(
                &card_pixels(&a, (759, 678)).unwrap(),
                &card_pixels(&a, (759, 678)).unwrap(),
                Default::default()
            )
            .unwrap()
            .exceeds_allowance()
        );
    }
}


/// A synthetic colour change with one delta of six passes five, but still refuses thirteen.
/// These constructed pixels isolate the policy and do not reproduce the missing runtime source.
#[test]
fn tolerance_171_six_level_channel_change_is_distinct_from_thirteen() {
    let before = vec![u32::from_le_bytes([126, 91, 24, 0]); 5874];
    let after = vec![u32::from_le_bytes([207, 85, 49, 255]); 5874];
    let tightened = compare_with_settings(&before, &after, Default::default()).unwrap();
    assert_eq!(tightened.allowed_changes, 293);
    assert!(tightened.exceeds_allowance());
    let historical =
        compare_with_settings(&before, &after, PyramidEffectSettings::new(13, 50).unwrap())
            .unwrap();
    assert_eq!(historical.changed_pixels, 0);
    assert!(!historical.exceeds_allowance());
}
