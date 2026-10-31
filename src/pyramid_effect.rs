//! Shared checked RGB occurrence measurements for Pyramid action owners.
//!
//! Samples preserve the same native coordinates, RGB bytes and exclusion union.
//! They grant no input authority and do not establish the expected game state.

use xsar::image_matching::packed_rgb::{PackedRgbComparison, compare_packed_rgb};

use crate::{capture::CapturedFrame, geometry::PixelRect};


use crate::parameters::PyramidEffectSettings;


/// Top-left quarter of the common 133-by-179 card interior, rounded down.
/// The glyph reader and removal quietness retain their separate 31-by-49 corner.
pub(crate) const CARD_QUARTER: PixelRect = PixelRect::new(3, 4, 66, 89);


/// Translate the fixed effect quarter to one native card origin with checked offsets.
pub(crate) fn card_bounds(origin: (u32, u32)) -> Result<PixelRect, String> {
    Ok(PixelRect::new(
        origin
            .0
            .checked_add(CARD_QUARTER.x)
            .ok_or("Card effect X overflow")?,
        origin
            .1
            .checked_add(CARD_QUARTER.y)
            .ok_or("Card effect Y overflow")?,
        CARD_QUARTER.width,
        CARD_QUARTER.height,
    ))
}


/// Copy the complete effect quarter; preserve native RGB values and ignore alpha.
pub(crate) fn card_pixels(frame: &CapturedFrame, origin: (u32, u32)) -> Result<Vec<u32>, String> {
    if frame.width != 1920 || frame.height != 1080 {
        return Err("Card effect requires native dimensions".to_owned());
    }
    sample(frame, card_bounds(origin)?, &[])
}


/// Compare with a boundary-frozen pair, ignoring alpha and mean magnitude.
/// Early counts are lower bounds; a refusal always scans the whole sample.
pub(crate) fn compare_with_settings(
    before: &[u32],
    after: &[u32],
    settings: PyramidEffectSettings,
) -> Result<PackedRgbComparison, String> {
    if before.is_empty() {
        return Err("Pyramid effect region has no eligible pixels".to_owned());
    }
    let rounded_percent = u8::try_from(settings.coverage_tenths().div_ceil(10))
        .map_err(|_| "Invalid Pyramid coverage")?;
    let mut measured = compare_packed_rgb(before, after, settings.rgb_tolerance(), rounded_percent)
        .map_err(|error| error.to_string())?;
    // An early count already exceeds the rounded-up bound, and hence this exact bound.
    measured.allowed_changes = settings.allowed_changes(measured.total_pixels);
    Ok(measured)
}


/// Exercise the default policy without live worker state in isolated fixtures.
#[cfg(test)]
pub(crate) fn compare(before: &[u32], after: &[u32]) -> Result<PackedRgbComparison, String> {
    compare_with_settings(before, after, PyramidEffectSettings::default())
}


/// Describe joint coverage honestly when the library stopped after a sufficient prefix.
pub(crate) fn measurement_text(measured: PackedRgbComparison) -> String {
    let changed = 100.0 * measured.changed_pixels as f64 / measured.total_pixels as f64;
    if measured.compared_pixels == measured.total_pixels {
        format!(
            "jointly changed {changed:.2}%; other pixels {:.2}%",
            100.0 - changed
        )
    } else {
        format!(
            "jointly changed at least {changed:.2}%; other pixels at most {:.2}%",
            100.0 - changed
        )
    }
}


/// Copy exact native RGB samples within one checked region and outside exclusions.
/// Exclusions form a union and never contribute either numerator or denominator.
pub(crate) fn sample(
    frame: &CapturedFrame,
    bounds: PixelRect,
    exclusions: &[PixelRect],
) -> Result<Vec<u32>, String> {
    if frame.width != 1920 || frame.height != 1080 {
        return Err("Pyramid effect requires native 1920x1080".to_owned());
    }
    let view = frame.as_view().map_err(|error| error.to_string())?;
    let (right, bottom) = view
        .checked_bounds(bounds)
        .map_err(|error| error.to_string())?;
    let count = usize::try_from(bounds.width)
        .ok()
        .and_then(|width| {
            usize::try_from(bounds.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or("Pyramid effect region size overflow")?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| "Cannot reserve Pyramid effect samples")?;
    for y in bounds.y..bottom {
        for x in bounds.x..right {
            if exclusions
                .iter()
                .any(|r| x >= r.x && y >= r.y && x - r.x < r.width && y - r.y < r.height)
            {
                continue;
            }
            let [r, g, b] = view
                .pixel_rgb(x, y)
                .ok_or("Unreadable Pyramid effect pixel")?;
            pixels.push(u32::from_le_bytes([r, g, b, 0]));
        }
    }
    if pixels.is_empty() {
        return Err("Pyramid effect region has no eligible pixels".to_owned());
    }
    Ok(pixels)
}


/// Compare the same eligible native region in two frames, with exact full counts.
/// This form serves existing count-based callers without exposing partial counts.
#[cfg(test)]
pub(crate) fn region_counts(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    exclusions: &[PixelRect],
    settings: PyramidEffectSettings,
) -> Result<PackedRgbComparison, String> {
    let before = sample(before, bounds, exclusions)?;
    let after = sample(after, bounds, exclusions)?;
    let mut measured = compare_packed_rgb(&before, &after, settings.rgb_tolerance(), 100)
        .map_err(|error| error.to_string())?;
    measured.allowed_changes = settings.allowed_changes(measured.total_pixels);
    Ok(measured)
}


/// Immutable bounded source region belonging to one action or appearance proof.
pub(crate) struct RegionBefore {
    /// Authoritative native rectangle, reused exactly after acknowledgement.
    bounds: PixelRect,
    /// Original RGB0 words; no alpha or source-frame mutation is retained.
    pixels: Vec<u32>,
}


impl RegionBefore {
    /// Capture one complete checked region before the owning action begins.
    pub(crate) fn capture(frame: &CapturedFrame, bounds: PixelRect) -> Result<Self, String> {
        Ok(Self {
            bounds,
            pixels: sample(frame, bounds, &[])?,
        })
    }


    /// Return the shared strict joint RGB occurrence decision on a fresh observation.
    pub(crate) fn changed(
        &self,
        after: &CapturedFrame,
        settings: PyramidEffectSettings,
    ) -> Result<bool, String> {
        Ok(
            compare_with_settings(&self.pixels, &sample(after, self.bounds, &[])?, settings)?
                .exceeds_allowance(),
        )
    }


    /// Measure every original sample for diagnostics without exposing a prefix as a full count.
    pub(crate) fn measure(
        &self,
        after: &CapturedFrame,
        settings: PyramidEffectSettings,
    ) -> Result<PackedRgbComparison, String> {
        let after = sample(after, self.bounds, &[])?;
        let mut measured = compare_packed_rgb(&self.pixels, &after, settings.rgb_tolerance(), 100)
            .map_err(|error| error.to_string())?;
        measured.allowed_changes = settings.allowed_changes(measured.total_pixels);
        Ok(measured)
    }
}


/// Native pile quarters from one checked pre-action source or pre-paired-D source.
pub(crate) struct StockBefore {
    /// Authoritative LEFT then RIGHT quarter, including a positively known empty pile.
    pixels: [Vec<u32>; 2],
}


impl StockBefore {
    /// Retain both checked quarters without another capture or guest input.
    #[cfg(test)]
    pub(crate) fn capture(frame: &CapturedFrame) -> Result<Self, String> {
        Ok(Self {
            pixels: [
                crate::pyramid_recon::vision::stock_corner(frame, false)?,
                crate::pyramid_recon::vision::stock_corner(frame, true)?,
            ],
        })
    }


    /// Require joint change only in piles whose expected or recognised identities changed.
    /// Identity, stock ordering, recycling and settling remain the owning controller's checks.
    #[cfg(test)]
    pub(crate) fn verify(&self, frame: &CapturedFrame, changed: [bool; 2]) -> Result<(), String> {
        self.verify_with_settings(frame, changed, PyramidEffectSettings::default())
    }


    /// Apply one frozen batch policy without retaining UI/shared state in pixel samples.
    pub(crate) fn verify_with_settings(
        &self,
        frame: &CapturedFrame,
        changed: [bool; 2],
        settings: PyramidEffectSettings,
    ) -> Result<(), String> {
        for (right, required) in changed.into_iter().enumerate() {
            if required {
                let current = crate::pyramid_recon::vision::stock_corner(frame, right == 1)?;
                let measured = compare_with_settings(&self.pixels[right], &current, settings)?;
                if !measured.exceeds_allowance() {
                    return Err(format!(
                        "{} stock quarter lacks joint RGB change: {}/{}; allowed={}; {}; {}",
                        if right == 0 { "LEFT" } else { "RIGHT" },
                        measured.changed_pixels,
                        measured.total_pixels,
                        measured.allowed_changes,
                        settings.description(),
                        measurement_text(measured)
                    ));
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod quarter_tests;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::PixelFormat;


    /// Synthetic native storage isolates occurrence arithmetic from scene recognition.
    fn frame(rgb: [u8; 3]) -> CapturedFrame {
        CapturedFrame {
            width: 1920,
            height: 1080,
            stride: 1920 * 4,
            format: PixelFormat::Rgba8,
            pixels: [rgb[0], rgb[1], rgb[2], 255].repeat(1920 * 1080),
        }
    }


    /// Same coordinates, padded rows, alpha exclusion and overlapping masks remain exact.
    #[test]
    fn joint_168_region_layout_and_exclusion_union() {
        let before = frame([100, 120, 140]);
        let mut after = frame([127, 147, 167]);
        after.stride += 16;
        after.pixels = [127, 147, 167, 0].repeat(after.stride / 4 * 1080);
        let bounds = PixelRect::new(10, 20, 10, 10);
        let excluded = [PixelRect::new(10, 20, 3, 10), PixelRect::new(12, 20, 3, 10)];
        let m = region_counts(
            &before,
            &after,
            bounds,
            &excluded,
            PyramidEffectSettings::default(),
        )
        .unwrap();
        assert_eq!(
            (m.changed_pixels, m.total_pixels, m.compared_pixels),
            (50, 50, 50)
        );
        assert!(m.exceeds_allowance());
        assert!(sample(&before, bounds, &[bounds]).is_err());
        assert!(sample(&before, PixelRect::new(1919, 1079, 2, 2), &[]).is_err());
        let mut invalid = before.clone();
        invalid.pixels.clear();
        assert!(sample(&invalid, bounds, &[]).is_err());
        assert!(
            region_counts(
                &before,
                &frame([127, 147, 145]),
                bounds,
                &[],
                PyramidEffectSettings::default()
            )
            .unwrap()
            .changed_pixels
                == 0
        );
    }


    /// Real retained stock replacement uses the shared predicate only on changed sides.
    #[test]
    fn joint_168_retained_stock_change_and_unchanged_refusal() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-168/green");
        let before =
            crate::capture::decode_png(&std::fs::read(base.join("first-unmatched.png")).unwrap())
                .unwrap();
        let after =
            crate::capture::decode_png(&std::fs::read(base.join("last-observed.png")).unwrap())
                .unwrap();
        let source = StockBefore::capture(&before).unwrap();
        assert!(source.verify(&before, [true, false]).is_err());
        source.verify(&after, [true, false]).unwrap();
        assert!(source.verify(&after, [false, true]).is_err());
    }


    /// A stock proof cannot borrow alpha or unrelated per-channel changes.
    #[test]
    fn joint_168_stock_rejects_alpha_single_channel_and_disjoint_changes() {
        let before = frame([100, 100, 100]);
        let source = StockBefore::capture(&before).unwrap();
        let mut alpha = before.clone();
        for pixel in alpha.pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 0;
        }
        assert!(source.verify(&alpha, [true, true]).is_err());
        assert!(
            source
                .verify(&frame([255, 100, 100]), [true, true])
                .is_err()
        );
        let mut disjoint = before.clone();
        for (index, pixel) in disjoint
            .pixels
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
        {
            pixel[..3].copy_from_slice(match index % 3 {
                0 => &[255, 255, 100],
                1 => &[100, 255, 255],
                _ => &[255, 100, 255],
            });
        }
        assert!(source.verify(&disjoint, [true, true]).is_err());
        source
            .verify(&frame([127, 127, 127]), [true, true])
            .unwrap();
        source.verify(&frame([73, 73, 73]), [true, false]).unwrap();
    }
}


/// Fractional coverage and immutable pair tests exercise application policy only.
#[cfg(test)]
mod live_settings_tests {
    use super::*;


    /// Strict bounds, full counts, tail handling and huge allowance arithmetic stay exact.
    #[test]
    fn live_168_fractional_settings_preserve_strict_joint_decision() {
        let defaults = PyramidEffectSettings::default();
        assert_eq!(
            (defaults.rgb_tolerance(), defaults.coverage_tenths()),
            (5, 50)
        );
        assert!(PyramidEffectSettings::new(13, 1001).is_err());
        assert_eq!(
            PyramidEffectSettings::new(0, 1000)
                .unwrap()
                .allowed_changes(usize::MAX),
            usize::MAX
        );
        let before = vec![0; 1519];
        let mut after = before.clone();
        after[..75].fill(u32::from_le_bytes([14, 14, 14, 255]));
        assert!(
            !compare_with_settings(&before, &after, defaults)
                .unwrap()
                .exceeds_allowance()
        );
        after[75] = u32::from_le_bytes([14, 14, 14, 255]);
        assert!(
            compare_with_settings(&before, &after, defaults)
                .unwrap()
                .exceeds_allowance()
        );
        let ten_percent = PyramidEffectSettings::new(13, 100).unwrap();
        after[..151].fill(u32::from_le_bytes([14, 14, 14, 255]));
        assert!(
            !compare_with_settings(&before, &after, ten_percent)
                .unwrap()
                .exceeds_allowance()
        );
        after[151] = u32::from_le_bytes([14, 14, 14, 255]);
        assert!(
            compare_with_settings(&before, &after, ten_percent)
                .unwrap()
                .exceeds_allowance()
        );
        let fractional = PyramidEffectSettings::new(13, 101).unwrap();
        assert_eq!(fractional.allowed_changes(1519), 153);
        assert!(
            !compare_with_settings(&before, &after, fractional)
                .unwrap()
                .exceeds_allowance()
        );
        after[152..154].fill(u32::from_le_bytes([14, 14, 14, 0]));
        assert!(
            compare_with_settings(&before, &after, fractional)
                .unwrap()
                .exceeds_allowance()
        );
        after.fill(u32::from_le_bytes([255, 255, 5, 255]));
        assert!(
            !compare_with_settings(&before, &after, defaults)
                .unwrap()
                .exceeds_allowance()
        );
        for length in [1, 63, 64, 65, 127, 128, 129, 1519] {
            let mut tail = vec![0; length];
            tail[length - 1] = u32::from_le_bytes([14, 14, 14, 0]);
            assert!(
                compare_with_settings(
                    &vec![0; length],
                    &tail,
                    PyramidEffectSettings::new(13, 0).unwrap()
                )
                .unwrap()
                .exceeds_allowance()
            );
            assert!(
                !compare_with_settings(
                    &vec![0; length],
                    &tail,
                    PyramidEffectSettings::new(13, 1000).unwrap()
                )
                .unwrap()
                .exceeds_allowance()
            );
        }
    }


    /// Synthetic reported stock-change counts isolate coverage, not saved prebatch frames.
    #[test]
    fn coverage_169_reported_recon_count_passes_default_but_not_old_policy() {
        let before = vec![0; 1519];
        for count in [102, 131] {
            let mut after = before.clone();
            after[..count].fill(u32::from_le_bytes([14, 14, 14, 255]));
            let defaults = PyramidEffectSettings::default();
            assert_eq!(defaults.allowed_changes(before.len()), 75);
            assert!(
                compare_with_settings(&before, &after, defaults)
                    .unwrap()
                    .exceeds_allowance()
            );
            let old = compare_with_settings(
                &before,
                &after,
                PyramidEffectSettings::new(13, 100).unwrap(),
            )
            .unwrap();
            assert_eq!(old.changed_pixels, count);
            assert_eq!(old.allowed_changes, 151);
            assert!(!old.exceeds_allowance());
        }
    }
}
