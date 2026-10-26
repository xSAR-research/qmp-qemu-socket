//! Pure, checked image evidence shared by application recognition readers.
//!
//! These functions know no game rules, card identities, input coordinates or
//! execution state. Callers supply calibrated regions and appearance policy.
//! Unknown evidence remains distinct from absence; neither result authorises input.

use xsar::image_matching::FrameView;

use crate::geometry::PixelRect;


/// Appearance limits for a calibrated white-paper card family.
#[derive(Clone, Copy)]
pub(crate) struct PaperPolicy {
    /// Inclusive minimum channel for strong white paper, in 8-bit RGB units.
    pub white_minimum: u8,
    /// Inclusive minimum channel for possible paper, including warm selection tint.
    pub possible_paper_minimum: u8,
    /// Inclusive strong-white percentage required to establish presence.
    pub present_percent: u8,
    /// Maximum possible-paper percentage permitted in an absence candidate.
    pub absent_percent: u8,
    /// Maximum adjacent channel difference throughout the clearance region.
    pub surface_delta: u8,
}


/// Bounded image evidence, independent of identity and game accessibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaperEvidence {
    /// Strong white paper occupies the calibrated presence fraction.
    Present,
    /// Little possible paper and a continuous unobscured clearance region agree.
    Absent,
    /// Tinted paper, edges, occlusion, motion or insufficient evidence remains.
    Unknown,
}


/// Classify one card probe and its complete surrounding clearance region.
///
/// Warm or grey paper cannot become absent merely because it is not strong white.
/// A broad smooth occlusion or near-white artwork is not generally identifiable
/// from local pixels; callers must additionally establish their supported scene.
/// Returns an error for invalid policy, layout, bounds or containment.
pub(crate) fn paper_evidence(
    view: FrameView<'_>,
    probe: PixelRect,
    clearance: PixelRect,
    policy: PaperPolicy,
) -> Result<PaperEvidence, String> {
    if policy.present_percent > 100
        || policy.absent_percent >= policy.present_percent
        || policy.possible_paper_minimum > policy.white_minimum
    {
        return Err("invalid paper evidence policy".to_owned());
    }
    checked(view, probe)?;
    checked(view, clearance)?;
    if probe.x < clearance.x
        || probe.y < clearance.y
        || u64::from(probe.x) + u64::from(probe.width)
            > u64::from(clearance.x) + u64::from(clearance.width)
        || u64::from(probe.y) + u64::from(probe.height)
            > u64::from(clearance.y) + u64::from(clearance.height)
    {
        return Err("paper probe outside its clearance region".to_owned());
    }
    let total = u64::from(probe.width) * u64::from(probe.height);
    let white = count_paper(view, probe, policy.white_minimum)?;
    if u128::from(white) * 100 >= u128::from(total) * u128::from(policy.present_percent) {
        return Ok(PaperEvidence::Present);
    }
    let possible = count_paper(view, probe, policy.possible_paper_minimum)?;
    if u128::from(possible) * 100 <= u128::from(total) * u128::from(policy.absent_percent)
        && continuous_surface(view, clearance, policy.surface_delta)?
    {
        Ok(PaperEvidence::Absent)
    } else {
        Ok(PaperEvidence::Unknown)
    }
}


/// Count pixels whose three channels meet a paper threshold in checked bounds.
/// Returns an error for empty/outside regions or an invalid frame layout.
pub(crate) fn count_paper(
    view: FrameView<'_>,
    bounds: PixelRect,
    minimum: u8,
) -> Result<u64, String> {
    checked(view, bounds)?;
    let mut count = 0;
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            if pixel(view, x, y)?.iter().all(|channel| *channel >= minimum) {
                count += 1;
            }
        }
    }
    Ok(count)
}


/// Require bounded spatial continuity without comparing a reference hue.
/// Every horizontal and vertical edge in the region participates, including its
/// full width and height. Returns an error for invalid/empty bounds or layout.
pub(crate) fn continuous_surface(
    view: FrameView<'_>,
    bounds: PixelRect,
    maximum_delta: u8,
) -> Result<bool, String> {
    checked(view, bounds)?;
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let current = pixel(view, x, y)?;
            for neighbour in [
                (x > bounds.x).then(|| pixel(view, x - 1, y)),
                (y > bounds.y).then(|| pixel(view, x, y - 1)),
            ]
            .into_iter()
            .flatten()
            {
                if current
                    .iter()
                    .zip(neighbour?)
                    .any(|(first, second)| first.abs_diff(second) > maximum_delta)
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}


/// Compare a complete selected binary foreground shape and its negative space.
///
/// The caller's classifier identifies foreground, rather than an underlying hue.
/// A positive control must request at least one positive core. A zero-core shape
/// is only negative-space evidence and must be coupled to independent positives.
/// Returns an error for malformed masks, unchecked bounds or missing pixels.
pub(crate) fn binary_shape_matches(
    view: FrameView<'_>,
    bounds: PixelRect,
    expected: &[Option<[u8; 3]>],
    classify: fn([u8; 3]) -> bool,
    minimum_positive: usize,
) -> Result<bool, String> {
    checked(view, bounds)?;
    let area = u64::from(bounds.width) * u64::from(bounds.height);
    if usize::try_from(area).ok() != Some(expected.len()) {
        return Err("binary shape mask length differs from its bounds".to_owned());
    }
    let mut positive = 0_usize;
    let width = usize::try_from(bounds.width).map_err(|_| "shape width overflow")?;
    for (index, reference) in expected.iter().enumerate() {
        let Some(reference) = reference else { continue };
        let x = bounds.x + u32::try_from(index % width).map_err(|_| "shape x overflow")?;
        let y = bounds.y + u32::try_from(index / width).map_err(|_| "shape y overflow")?;
        let wanted = classify(*reference);
        positive += usize::from(wanted);
        if classify(pixel(view, x, y)?) != wanted {
            return Ok(false);
        }
    }
    Ok(positive >= minimum_positive)
}


/// Validate a nonempty rectangle before iteration; checked layout bounds prevent overflow.
fn checked(view: FrameView<'_>, bounds: PixelRect) -> Result<(), String> {
    if bounds.width == 0 || bounds.height == 0 {
        return Err("empty image evidence region".to_owned());
    }
    view.checked_bounds(bounds)
        .map(|_| ())
        .map_err(|_| "image evidence region outside frame".to_owned())
}


/// Read one checked RGB pixel, preserving invalid-layout errors instead of inventing colour.
fn pixel(view: FrameView<'_>, x: u32, y: u32) -> Result<[u8; 3], String> {
    view.pixel_rgb(x, y)
        .ok_or_else(|| "missing image evidence pixel".to_owned())
}


#[cfg(test)]
mod tests {
    //! Synthetic boundary checks for game-independent observation primitives.

    use super::*;


    /// Return a synthetic policy with exact, easily inspected percentage limits.
    fn policy() -> PaperPolicy {
        PaperPolicy {
            white_minimum: 200,
            possible_paper_minimum: 100,
            present_percent: 75,
            absent_percent: 25,
            surface_delta: 10,
        }
    }


    /// Classify arbitrary foreground without card identities or a white-ink model.
    fn foreground(rgb: [u8; 3]) -> bool {
        rgb[0] > rgb[1]
    }


    /// Invalid regions, containment, policies and raster sizes must return errors.
    #[test]
    fn malformed_evidence_requests_fail_closed() {
        let bytes = [0_u8; 16];
        let view = FrameView::from_rgba(2, 2, 8, &bytes).unwrap();
        let whole = PixelRect::new(0, 0, 2, 2);
        let unit = PixelRect::new(0, 0, 1, 1);
        for invalid in [
            PixelRect::new(0, 0, 0, 1),
            PixelRect::new(0, 0, 1, 0),
            PixelRect::new(2, 0, 1, 1),
            PixelRect::new(0, 2, 1, 1),
            PixelRect::new(u32::MAX, 0, 2, 1),
            PixelRect::new(0, u32::MAX, 1, 2),
        ] {
            assert!(count_paper(view, invalid, 100).is_err());
            assert!(continuous_surface(view, invalid, 10).is_err());
            assert!(paper_evidence(view, invalid, whole, policy()).is_err());
            assert!(paper_evidence(view, unit, invalid, policy()).is_err());
            assert!(binary_shape_matches(view, invalid, &[], foreground, 1).is_err());
        }
        assert!(paper_evidence(view, whole, unit, policy()).is_err());
        assert!(binary_shape_matches(view, whole, &[Some([0; 3])], foreground, 1).is_err());
        for invalid in [
            PaperPolicy {
                present_percent: 101,
                ..policy()
            },
            PaperPolicy {
                absent_percent: 75,
                ..policy()
            },
            PaperPolicy {
                present_percent: 0,
                absent_percent: 0,
                ..policy()
            },
            PaperPolicy {
                possible_paper_minimum: 201,
                ..policy()
            },
        ] {
            assert!(paper_evidence(view, unit, whole, invalid).is_err());
        }
    }


    /// Ratios use inclusive exact comparisons, without rounding thirds to percentages.
    #[test]
    fn paper_percentage_boundaries_preserve_unknown() {
        let strong = [200_u8, 200, 200, 255];
        let dark = [0_u8, 0, 0, 255];
        let cases = [
            (vec![strong, strong, dark], 66, 25, PaperEvidence::Present),
            (vec![strong, strong, dark], 67, 25, PaperEvidence::Unknown),
            (vec![strong, dark, dark], 75, 33, PaperEvidence::Unknown),
            (vec![strong, dark, dark], 75, 34, PaperEvidence::Absent),
            (
                vec![strong, strong, dark, dark],
                50,
                25,
                PaperEvidence::Present,
            ),
            (
                vec![strong, dark, dark, dark],
                75,
                25,
                PaperEvidence::Absent,
            ),
        ];
        for (pixels, present_percent, absent_percent, expected) in cases {
            let width = u32::try_from(pixels.len()).unwrap();
            let bytes = pixels.concat();
            let view = FrameView::from_rgba(width, 1, bytes.len(), &bytes).unwrap();
            let bounds = PixelRect::new(0, 0, width, 1);
            let limits = PaperPolicy {
                present_percent,
                absent_percent,
                surface_delta: 255,
                ..policy()
            };
            assert_eq!(
                paper_evidence(view, bounds, bounds, limits).unwrap(),
                expected
            );
        }
    }


    /// Possible paper and final-row/column discontinuities remain unknown.
    #[test]
    fn absence_requires_below_threshold_paper_and_complete_clearance() {
        let probe = PixelRect::new(1, 1, 1, 1);
        let clearance = PixelRect::new(0, 0, 3, 3);
        for (edge, channel, expected) in [
            ([6_usize, 7, 8], 109_u8, PaperEvidence::Absent),
            ([6, 7, 8], 110, PaperEvidence::Unknown),
            ([2, 5, 8], 110, PaperEvidence::Unknown),
        ] {
            let mut pixels = [[99_u8, 99, 99, 255]; 9];
            for index in edge {
                pixels[index] = [channel, channel, channel, 255];
            }
            let bytes = pixels.concat();
            let view = FrameView::from_rgba(3, 3, 12, &bytes).unwrap();
            assert_eq!(
                paper_evidence(view, probe, clearance, policy()).unwrap(),
                expected
            );
        }
        let possible_paper = [[100_u8, 100, 100, 255]; 9].concat();
        let view = FrameView::from_rgba(3, 3, 12, &possible_paper).unwrap();
        assert_eq!(
            paper_evidence(view, probe, clearance, policy()).unwrap(),
            PaperEvidence::Unknown
        );
    }


    /// Selected negative pixels and a requested positive core are both mandatory.
    #[test]
    fn binary_shapes_use_caller_classes_and_all_selected_negative_space() {
        let bounds = PixelRect::new(0, 0, 2, 2);
        let expected = [Some([2, 1, 0]), None, Some([1, 2, 0]), Some([1, 2, 0])];
        let mut pixels = [
            [210_u8, 20, 40, 255],
            [5, 150, 80, 255],
            [30, 160, 90, 255],
            [10, 120, 30, 255],
        ];
        for ignored_pixel in [[5, 150, 80, 255], [200, 5, 80, 255]] {
            pixels[1] = ignored_pixel;
            let bytes = pixels.concat();
            let view = FrameView::from_rgba(2, 2, 8, &bytes).unwrap();
            assert!(binary_shape_matches(view, bounds, &expected, foreground, 1).unwrap());
        }
        pixels[3] = [200, 5, 30, 255];
        let bytes = pixels.concat();
        let view = FrameView::from_rgba(2, 2, 8, &bytes).unwrap();
        assert!(!binary_shape_matches(view, bounds, &expected, foreground, 1).unwrap());

        let negative_pixels = [[1_u8, 2, 0, 255]; 4].concat();
        let view = FrameView::from_rgba(2, 2, 8, &negative_pixels).unwrap();
        let negative_only = [Some([1, 2, 0]); 4];
        assert!(!binary_shape_matches(view, bounds, &negative_only, foreground, 1).unwrap());
        assert!(binary_shape_matches(view, bounds, &negative_only, foreground, 0).unwrap());
    }
}
