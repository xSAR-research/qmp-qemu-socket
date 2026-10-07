//! Checked pixel probes for Solver halos, card faces, gameplay and shared progress.
//! Detection reads immutable frames and never sends guest input.

use thiserror::Error;

use crate::{
    capture::CapturedFrame,
    cards::CardRegionPixels,
    game::{GameProfile, GameProgress, GameplaySceneProfile},
    geometry::{PixelPoint, PixelRect},
    parameters::{
        DIALOG_GOLD_BLUE_MAXIMUM, DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM, DIALOG_GOLD_GREEN_MINIMUM,
        DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM, DIALOG_GOLD_RED_MINIMUM,
        DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE, FACE_UP_WHITE_BLOCK_MIN_WIDTH,
        FACE_UP_WHITE_CHANNEL_MINIMUM, GOLD_CHANNEL_TOLERANCE, GOLD_RGB_CANDIDATES,
        HALO_GOLD_LINE_OFFSET_Y, HALO_GOLD_RUN_MIN, HALO_GOLD_SCAN_HEIGHT,
        TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM, TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM,
        TABLEAU_RELAXED_GOLD_GREEN_MINIMUM, TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM,
        TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM, TABLEAU_RELAXED_GOLD_RED_MINIMUM,
        TABLEAU_RELAXED_HALO_START_X_TOLERANCE, TABLEAU_RELAXED_HALO_Y_TOLERANCE,
    },
};


/// Classify the shared right-third progress probe using the selected profile.
/// Callers must establish a completion context before acting on this classification.
pub fn detect_game_progress_for_profile(
    frame: &CapturedFrame,
    profile: &GameProfile,
) -> Result<GameProgress, HaloDetectionError> {
    Ok(measure_game_progress_for_profile(frame, profile)?.classification)
}


/// Classification and measured pixel counts retained for transition diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameProgressEvidence {
    /// Progress classification; insufficient by itself to prove a board or game win.
    pub classification: GameProgress,
    /// Number of probe pixels at or below the black-channel threshold.
    pub black_pixels: u32,
    /// Total number of pixels in the checked probe rectangle.
    pub total_pixels: u32,
}


/// Preserve the measured probe counts for transition diagnostics. Classification
/// thresholds are shared with the existing TriPeaks progress detector.
pub fn measure_game_progress_for_profile(
    frame: &CapturedFrame,
    profile: &GameProfile,
) -> Result<GameProgressEvidence, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let progress = profile
        .game_progress
        .ok_or(HaloDetectionError::MissingProfileCalibration {
            target: "game progress",
        })?;
    let (right, bottom) = checked_bounds(progress.right_probe, frame)?;
    let mut black_pixels = 0_u32;
    let total_pixels = progress
        .right_probe
        .width
        .saturating_mul(progress.right_probe.height);


    for y in progress.right_probe.y..bottom {


        for x in progress.right_probe.x..right {
            let rgb = pixel_rgb(frame, x, y).ok_or(HaloDetectionError::InvalidFrameLayout)?;


            if rgb
                .into_iter()
                .all(|channel| channel <= progress.black_channel_maximum)
            {
                black_pixels = black_pixels.saturating_add(1);
            }
        }
    }


    let classification = if black_pixels.saturating_mul(1_000)
        >= total_pixels.saturating_mul(progress.black_fraction_per_mille)
    {
        GameProgress::AnotherBoard
    } else {
        GameProgress::GameComplete
    };
    Ok(GameProgressEvidence {
        classification,
        black_pixels,
        total_pixels,
    })
}


/// Horizontal gold evidence with the returned calibrated halo anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldRunHit {
    /// First pixel in the qualifying horizontal run.
    pub anchor: PixelPoint,
    /// Number of consecutive qualifying pixels on the matching row.
    pub length: u32,
}


/// Solid near-white rectangle found inside a face-up card probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhiteBlockHit {
    /// Top-left pixel of the qualifying near-white rectangle.
    pub anchor: PixelPoint,
    /// Number of consecutive fully white columns.
    pub width: u32,
    /// Full probe height covered by each accepted column.
    pub height: u32,
}


/// Invalid calibration or frame geometry that prevents a trustworthy pixel probe.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum HaloDetectionError {
    /// The selected game has no profile for the requested detector.
    #[error("{target} detector has no calibrated game profile")]
    MissingProfileCalibration {
        /// Human-readable detector name used in the error.
        target: &'static str,
    },
    /// Storage length, stride or pixel geometry is inconsistent.
    #[error("captured frame has an invalid pixel layout")]
    InvalidFrameLayout,
    /// A probe rectangle contains no pixels.
    #[error("detector bounds must be non-empty")]
    EmptyBounds,
    /// A probe overflows or extends beyond the frame.
    #[error("detector bounds lie outside the captured frame")]
    BoundsOutsideFrame,
    /// The declared halo rectangle does not enclose its card.
    #[error("card bounds must be contained by halo bounds")]
    CardOutsideHalo,
    /// The exterior scan lines do not fit the declared halo rectangle.
    #[error("the calibrated halo scan lines must be contained by halo bounds")]
    HaloLinesOutsideHalo,
}


/// Match the audited gold palette within the configured per-channel tolerance.
fn is_goldish(rgb: [u8; 3]) -> bool {
    GOLD_RGB_CANDIDATES.iter().any(|candidate| {
        rgb[0].abs_diff(candidate[0]) <= GOLD_CHANNEL_TOLERANCE
            && rgb[1].abs_diff(candidate[1]) <= GOLD_CHANNEL_TOLERANCE
            && rgb[2].abs_diff(candidate[2]) <= GOLD_CHANNEL_TOLERANCE
    })
}


/// Recognise a darker gold-like colour by channel relationship.
///
/// This deliberately does not replace [`is_goldish`]. It is used only after
/// an exact tableau-halo miss, together with strict run length, exterior-strip
/// and calibrated-start constraints.
fn is_relational_tableau_gold(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    let red_green_delta = i16::from(red) - i16::from(green);
    let green_blue_delta = i16::from(green) - i16::from(blue);

    red >= TABLEAU_RELAXED_GOLD_RED_MINIMUM
        && green >= TABLEAU_RELAXED_GOLD_GREEN_MINIMUM
        && blue <= TABLEAU_RELAXED_GOLD_BLUE_MAXIMUM
        && red_green_delta >= i16::from(TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MINIMUM)
        && red_green_delta <= i16::from(TABLEAU_RELAXED_GOLD_RED_GREEN_DELTA_MAXIMUM)
        && green_blue_delta >= i16::from(TABLEAU_RELAXED_GOLD_GREEN_BLUE_DELTA_MINIMUM)
}


/// Require every RGB channel to meet the face-up card whiteness threshold.
fn is_near_white(rgb: [u8; 3]) -> bool {
    rgb.into_iter()
        .all(|channel| channel >= FACE_UP_WHITE_CHANNEL_MINIMUM)
}


/// Compare a pixel with the profile's felt brightness and colour separation limits.
fn is_gameplay_felt_green(rgb: [u8; 3], profile: GameplaySceneProfile) -> bool {
    let [red, green, blue] = rgb;

    green >= profile.green_minimum
        && i16::from(green) - i16::from(red) >= i16::from(profile.green_red_delta_minimum)
        && i16::from(green) - i16::from(blue) >= i16::from(profile.green_blue_delta_minimum)
}


/// Recognise the broad gold gradient used by post-game buttons.
fn is_dialog_gold(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;

    red >= DIALOG_GOLD_RED_MINIMUM
        && green >= DIALOG_GOLD_GREEN_MINIMUM
        && blue <= DIALOG_GOLD_BLUE_MAXIMUM
        && i16::from(red) - i16::from(green) >= i16::from(DIALOG_GOLD_RED_GREEN_DELTA_MINIMUM)
        && i16::from(green) - i16::from(blue) >= i16::from(DIALOG_GOLD_GREEN_BLUE_DELTA_MINIMUM)
}


/// Find the first long horizontal gold run on a card's calibrated halo lines.
///
/// Only the two audited lines below the card are examined, avoiding both card
/// artwork and the bulk of the padded halo rectangle. Either line may carry the
/// qualifying run. Its returned Y coordinate is normalised to the calibrated
/// baseline so the existing anchor-to-click invariant remains deterministic.
pub fn find_gold_halo_run(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (card_right, card_bottom) = checked_bounds(regions.card_bounds, frame)?;
    let (halo_right, halo_bottom) = checked_bounds(regions.halo_bounds, frame)?;


    if regions.card_bounds.x < regions.halo_bounds.x
        || regions.card_bounds.y < regions.halo_bounds.y
        || card_right > halo_right
        || card_bottom > halo_bottom
    {
        return Err(HaloDetectionError::CardOutsideHalo);
    }

    let scan_y = card_bottom
        .checked_add(HALO_GOLD_LINE_OFFSET_Y)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let scan_bottom = scan_y
        .checked_add(HALO_GOLD_SCAN_HEIGHT)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;


    if scan_y < regions.halo_bounds.y || scan_bottom > halo_bottom {
        return Err(HaloDetectionError::HaloLinesOutsideHalo);
    }


    for y in scan_y..scan_bottom {


        if let Some((run_start, run_length)) = find_horizontal_run(
            frame,
            regions.halo_bounds.x,
            halo_right,
            y,
            HALO_GOLD_RUN_MIN,
            is_goldish,
        ) {
            return Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(run_start as i32, scan_y as i32),
                length: run_length,
            }));
        }
    }

    Ok(None)
}


/// Find the first exact-colour horizontal halo run inside one tight profile
/// region. The profile controls ordering; this primitive performs no action
/// selection and sends no guest input.
pub fn find_gold_run_in_bounds(
    frame: &CapturedFrame,
    bounds: PixelRect,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(bounds, frame)?;


    for y in bounds.y..bottom {


        if let Some((run_start, run_length)) =
            find_horizontal_run(frame, bounds.x, right, y, HALO_GOLD_RUN_MIN, is_goldish)
        {
            return Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(run_start as i32, y as i32),
                length: run_length,
            }));
        }
    }

    Ok(None)
}


/// Require the calibrated fraction of the scene probe to contain green felt.
/// Reject missing calibration, invalid frame storage and out-of-frame bounds.
pub fn has_gameplay_felt_for_profile(
    frame: &CapturedFrame,
    profile: &GameProfile,
) -> Result<bool, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let scene = profile
        .gameplay_scene
        .ok_or(HaloDetectionError::MissingProfileCalibration {
            target: "gameplay scene",
        })?;
    let (right, bottom) = checked_bounds(scene.probe_bounds, frame)?;
    let total_pixels = u64::from(scene.probe_bounds.width) * u64::from(scene.probe_bounds.height);
    let mut matching_pixels = 0_u64;


    for y in scene.probe_bounds.y..bottom {


        for x in scene.probe_bounds.x..right {


            if pixel_rgb(frame, x, y).is_some_and(|rgb| is_gameplay_felt_green(rgb, scene)) {
                matching_pixels += 1;
            }
        }
    }

    Ok(matching_pixels * 1_000 >= total_pixels * u64::from(scene.required_fraction_per_mille))
}


/// Report whether a calibrated dialog band contains enough gold-gradient pixels.
///
/// This detector is intentionally broader than halo matching. Callers must
/// also enforce the ordered post-game stage and reject gameplay scenes.
pub fn has_dialog_gold_button(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<bool, HaloDetectionError> {
    Ok(dialog_gold_fraction_per_mille(frame, probe_bounds)?
        >= DIALOG_GOLD_REQUIRED_FRACTION_PER_MILLE)
}


/// Fraction of dialog-gold pixels inside an in-bounds region. The post-game
/// controller uses a stronger threshold when two layout probes overlap the
/// same button, while the individual variant probes keep their calibration.
pub fn dialog_gold_fraction_per_mille(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<u32, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(probe_bounds, frame)?;
    let total_pixels = u64::from(probe_bounds.width) * u64::from(probe_bounds.height);
    let mut matching_pixels = 0_u64;


    for y in probe_bounds.y..bottom {


        for x in probe_bounds.x..right {


            if pixel_rgb(frame, x, y).is_some_and(is_dialog_gold) {
                matching_pixels = matching_pixels.saturating_add(1);
            }
        }
    }

    Ok((matching_pixels.saturating_mul(1_000) / total_pixels) as u32)
}


/// Find a tableau halo without allowing rendering drift to move the click.
///
/// The audited exact-colour/two-line detector remains the fast path. If it
/// misses, a narrow exterior strip is searched for a darker gold-shaped run.
/// A fallback run must begin close to the slot's calibrated X anchor, and its
/// hit always returns the immutable calibrated anchor rather than an observed
/// fringe pixel. Exact hits retain their observed anchor so the tracker's
/// existing offset invariant continues to reject shifted exact evidence.
pub fn find_tableau_gold_halo_run(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
    click_offset_x: i32,
) -> Result<Option<GoldRunHit>, HaloDetectionError> {


    if let Some(exact_hit) = find_gold_halo_run(frame, regions)? {
        return Ok(Some(exact_hit));
    }

    let canonical_anchor = canonical_tableau_halo_anchor(regions, click_offset_x)?;
    let canonical_x =
        u32::try_from(canonical_anchor.x).map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;

    let (_, card_bottom) = checked_bounds(regions.card_bounds, frame)?;
    let (halo_right, halo_bottom) = checked_bounds(regions.halo_bounds, frame)?;
    let canonical_y =
        u32::try_from(canonical_anchor.y).map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;

    // Do not admit card artwork: even the upward side of the tolerance window
    // is clamped to the calibrated exterior at the card's bottom edge.
    let strip_top = canonical_y
        .saturating_sub(TABLEAU_RELAXED_HALO_Y_TOLERANCE)
        .max(card_bottom)
        .max(regions.halo_bounds.y);
    let strip_bottom = canonical_y
        .checked_add(HALO_GOLD_SCAN_HEIGHT)
        .and_then(|bottom| bottom.checked_add(TABLEAU_RELAXED_HALO_Y_TOLERANCE))
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?
        .min(halo_bottom);


    for y in strip_top..strip_bottom {


        if let Some((_, run_length)) = find_horizontal_run_near_anchor(
            frame,
            regions.halo_bounds.x,
            halo_right,
            y,
            HALO_GOLD_RUN_MIN,
            canonical_x,
            TABLEAU_RELAXED_HALO_START_X_TOLERANCE,
            is_relational_tableau_gold,
        ) {
            return Ok(Some(GoldRunHit {
                anchor: canonical_anchor,
                length: run_length,
            }));
        }
    }

    Ok(None)
}


/// Find a solid near-white rectangle in a calibrated face-up-card probe band.
///
/// Every pixel in a qualifying column must be near-white across the full band
/// height. Consecutive columns form the block, so isolated bright pixels and
/// the speckled snow on face-down card backs cannot qualify.
pub fn find_face_up_white_block(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<Option<WhiteBlockHit>, HaloDetectionError> {


    if !frame.is_layout_valid() {
        return Err(HaloDetectionError::InvalidFrameLayout);
    }

    let (right, bottom) = checked_bounds(probe_bounds, frame)?;
    let mut run_start = 0;
    let mut run_width = 0;


    for x in probe_bounds.x..right {
        let column_is_white =
            (probe_bounds.y..bottom).all(|y| pixel_rgb(frame, x, y).is_some_and(is_near_white));


        if column_is_white {


            if run_width == 0 {
                run_start = x;
            }
            run_width += 1;
        } else if run_width >= FACE_UP_WHITE_BLOCK_MIN_WIDTH {
            return Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(run_start as i32, probe_bounds.y as i32),
                width: run_width,
                height: probe_bounds.height,
            }));
        } else {
            run_width = 0;
        }
    }

    Ok(
        (run_width >= FACE_UP_WHITE_BLOCK_MIN_WIDTH).then_some(WhiteBlockHit {
            anchor: PixelPoint::new(run_start as i32, probe_bounds.y as i32),
            width: run_width,
            height: probe_bounds.height,
        }),
    )
}


/// Report whether a calibrated row probe contains a face-up white block.
pub fn has_face_up_white_block(
    frame: &CapturedFrame,
    probe_bounds: PixelRect,
) -> Result<bool, HaloDetectionError> {
    Ok(find_face_up_white_block(frame, probe_bounds)?.is_some())
}


/// Return the first sufficiently long contiguous matching run in a checked row.
fn find_horizontal_run(
    frame: &CapturedFrame,
    left: u32,
    right: u32,
    y: u32,
    minimum_length: u32,
    predicate: fn([u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    let width = right.checked_sub(left)?;
    let view = frame.as_view().ok()?;
    let hit = xsar::image_matching::find_horizontal_run(
        view,
        PixelRect::new(left, y, width, 1),
        minimum_length,
        None,
        predicate,
    ).ok()??;
    Some((u32::try_from(hit.anchor.x).ok()?, hit.length))
}


/// Find a matching run whose first pixel lies within the calibrated X tolerance.
#[allow(clippy::too_many_arguments)]
fn find_horizontal_run_near_anchor(
    frame: &CapturedFrame,
    left: u32,
    right: u32,
    y: u32,
    minimum_length: u32,
    canonical_start: u32,
    start_tolerance: u32,
    predicate: fn([u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    let width = right.checked_sub(left)?;
    let view = frame.as_view().ok()?;
    let constraint = xsar::image_matching::RunStartConstraint {
        x: canonical_start,
        tolerance: start_tolerance,
    };
    let hit = xsar::image_matching::find_horizontal_run(
        view,
        PixelRect::new(left, y, width, 1),
        minimum_length,
        Some(constraint),
        predicate,
    ).ok()??;
    Some((u32::try_from(hit.anchor.x).ok()?, hit.length))
}


/// Derive the fixed TriPeaks halo baseline and X anchor with checked arithmetic.
fn canonical_tableau_halo_anchor(
    regions: CardRegionPixels,
    click_offset_x: i32,
) -> Result<PixelPoint, HaloDetectionError> {
    let x = regions
        .click_point
        .x
        .checked_sub(click_offset_x)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let y = regions
        .card_bounds
        .y
        .checked_add(regions.card_bounds.height)
        .and_then(|bottom| bottom.checked_add(HALO_GOLD_LINE_OFFSET_Y))
        .and_then(|value| i32::try_from(value).ok())
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;

    Ok(PixelPoint::new(x, y))
}


/// Reject empty, overflowing or out-of-frame rectangles and return exclusive edges.
fn checked_bounds(
    bounds: PixelRect,
    frame: &CapturedFrame,
) -> Result<(u32, u32), HaloDetectionError> {


    if bounds.is_empty() {
        return Err(HaloDetectionError::EmptyBounds);
    }

    let right = bounds
        .x
        .checked_add(bounds.width)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;
    let bottom = bounds
        .y
        .checked_add(bounds.height)
        .ok_or(HaloDetectionError::BoundsOutsideFrame)?;


    if right > frame.width || bottom > frame.height {
        return Err(HaloDetectionError::BoundsOutsideFrame);
    }

    Ok((right, bottom))
}


/// Read the RGB channels of one RGBA pixel in a caller-validated frame.
pub(crate) fn pixel_rgb(frame: &CapturedFrame, x: u32, y: u32) -> Option<[u8; 3]> {
    xsar::image_matching::pixel_rgb(frame, x, y)
}


#[cfg(test)]
mod tests {
    //! Synthetic detector boundaries and recorded progress-probe regression evidence.
    use super::*;
    use crate::capture::PixelFormat;
    use crate::parameters::{
        CLICK_OFFSET_X, FACE_UP_WHITE_SCAN_HEIGHT, SOLVER_PROGRESS_RIGHT_PROBE,
    };


    /// Use the TriPeaks click offset when exercising the active tableau halo detector.
    fn find_tableau_halo(
        frame: &CapturedFrame,
        regions: CardRegionPixels,
    ) -> Result<Option<GoldRunHit>, HaloDetectionError> {
        find_tableau_gold_halo_run(frame, regions, CLICK_OFFSET_X)
    }


    /// Build a black RGBA test fixture with tightly packed rows.
    fn rgba_frame(width: u32, height: u32) -> CapturedFrame {
        CapturedFrame {
            width,
            height,
            stride: width as usize * 4,
            format: PixelFormat::Rgba8,
            pixels: vec![0; width as usize * height as usize * 4],
        }
    }


    /// Paint one RGBA fixture pixel with opaque alpha.
    fn set_pixel(frame: &mut CapturedFrame, x: u32, y: u32, rgb: [u8; 3]) {
        let offset = y as usize * frame.stride + x as usize * 4;
        let pixel = [rgb[0], rgb[1], rgb[2], 255];
        frame.pixels[offset..offset + 4].copy_from_slice(&pixel);
    }


    /// Return a small synthetic card with space for the calibrated exterior halo lines.
    fn card_regions() -> CardRegionPixels {
        CardRegionPixels::new(
            PixelRect::new(40, 40, 120, 140),
            PixelRect::new(28, 26, 144, 168),
            PixelRect::new(43, 47, 25, 22),
            PixelPoint::new(100, 110),
        )
    }


    /// Paint a test halo using the first audited exact gold colour.
    fn set_horizontal_run(frame: &mut CapturedFrame, start_x: u32, y: u32, length: u32) {
        set_coloured_horizontal_run(frame, start_x, y, length, GOLD_RGB_CANDIDATES[0]);
    }


    /// Paint a horizontal fixture run for exact and relaxed colour checks.
    fn set_coloured_horizontal_run(
        frame: &mut CapturedFrame,
        start_x: u32,
        y: u32,
        length: u32,
        colour: [u8; 3],
    ) {


        for x in start_x..start_x + length {
            set_pixel(frame, x, y, colour);
        }
    }


    /// Paint a solid greyscale rectangle for face-up detection boundary tests.
    fn set_white_block(
        frame: &mut CapturedFrame,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        value: u8,
    ) {


        for y in start_y..start_y + height {


            for x in start_x..start_x + width {
                set_pixel(frame, x, y, [value; 3]);
            }
        }
    }


    /// Check the detected anchor and full run length on the first exterior line.

    #[test]
    fn finds_a_long_halo_run_below_the_card() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, 119);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
        assert_eq!(
            find_gold_halo_run(&frame, card_regions()).map(|hit| hit.is_some()),
            Ok(true)
        );
    }


    /// Keep the minimum accepted halo length inclusive.

    #[test]
    fn accepts_a_halo_run_exactly_at_the_minimum_length() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, HALO_GOLD_RUN_MIN);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: HALO_GOLD_RUN_MIN,
            }))
        );
    }


    /// Keep second-line detections anchored to the audited baseline.

    #[test]
    fn canonicalises_a_run_on_the_second_halo_line_to_the_baseline() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 187, 119);

        assert_eq!(
            find_gold_halo_run(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
    }


    /// Ensure an exact TriPeaks halo hit keeps its observed anchor.

    #[test]
    fn tableau_fast_path_preserves_the_audited_anchor() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, 119);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(41, 186),
                length: 119,
            }))
        );
    }


    /// Accept calibrated dark-gold rendering drift without moving the click anchor.

    #[test]
    fn tableau_fallback_finds_a_dark_halo_on_a_shifted_exterior_line() {
        let mut frame = rgba_frame(220, 220);
        set_coloured_horizontal_run(&mut frame, 35, 182, 119, [225, 194, 100]);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(39, 186),
                length: 119,
            }))
        );
    }


    /// Accept the inclusive X tolerance while returning the fixed calibrated anchor.

    #[test]
    fn tableau_fallback_canonicalises_a_tolerated_fringe_start() {
        let mut frame = rgba_frame(220, 220);
        let observed_start = 39 - TABLEAU_RELAXED_HALO_START_X_TOLERANCE;
        set_coloured_horizontal_run(&mut frame, observed_start, 190, 100, [225, 194, 100]);

        assert_eq!(
            find_tableau_halo(&frame, card_regions()),
            Ok(Some(GoldRunHit {
                anchor: PixelPoint::new(39, 186),
                length: 100,
            }))
        );
    }


    /// Reject white, felt, red and other colours outside the relaxed gold relationship.

    #[test]
    fn tableau_fallback_rejects_non_gold_channel_relationships() {


        for colour in [
            [250, 250, 250],
            [90, 160, 100],
            [220, 30, 30],
            [220, 210, 100],
            [170, 140, 80],
        ] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, 182, 119, colour);

            assert_eq!(
                find_tableau_halo(&frame, card_regions()),
                Ok(None),
                "unexpected fallback match for {colour:?}"
            );
        }
    }


    /// Reject a long gold run just beyond the permitted start displacement.

    #[test]
    fn tableau_fallback_rejects_a_gold_run_starting_outside_x_tolerance() {
        let mut frame = rgba_frame(220, 220);
        let wrong_start = 39 + TABLEAU_RELAXED_HALO_START_X_TOLERANCE + 1;
        set_coloured_horizontal_run(&mut frame, wrong_start, 182, 100, [225, 194, 100]);

        assert_eq!(find_tableau_halo(&frame, card_regions()), Ok(None));
    }


    /// Accept both exterior strip edges and reject adjacent card or out-of-strip pixels.

    #[test]
    fn tableau_fallback_is_bounded_to_the_exterior_strip() {


        for accepted_y in [180, 193] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, accepted_y, 100, [225, 194, 100]);
            assert!(
                find_tableau_halo(&frame, card_regions()).unwrap().is_some(),
                "expected exterior line y={accepted_y} to be accepted"
            );
        }


        for rejected_y in [179, 194] {
            let mut frame = rgba_frame(220, 220);
            set_coloured_horizontal_run(&mut frame, 39, rejected_y, 100, [225, 194, 100]);
            assert_eq!(
                find_tableau_halo(&frame, card_regions()),
                Ok(None),
                "unexpected fallback match outside exterior strip at y={rejected_y}"
            );
        }
    }


    /// Prevent the exact detector from widening its two-line search.

    #[test]
    fn ignores_gold_runs_outside_the_two_calibrated_halo_lines() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 185, 119);
        set_horizontal_run(&mut frame, 41, 188, 119);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
    }


    /// Ensure gold card artwork cannot qualify as an exterior halo.

    #[test]
    fn ignores_a_long_gold_run_inside_card_artwork() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 40, 150, 120);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
        assert_eq!(
            find_gold_halo_run(&frame, card_regions()).map(|hit| hit.is_some()),
            Ok(false)
        );
    }


    /// Keep a run one pixel below the minimum ineligible.

    #[test]
    fn rejects_an_exterior_run_shorter_than_the_threshold() {
        let mut frame = rgba_frame(220, 220);
        set_horizontal_run(&mut frame, 41, 186, HALO_GOLD_RUN_MIN - 1);

        assert_eq!(find_gold_halo_run(&frame, card_regions()), Ok(None));
    }


    /// Check the accepted face-up rectangle geometry and presence result.

    #[test]
    fn detects_a_solid_near_white_face_block() {
        let mut frame = rgba_frame(80, 40);
        let probe = PixelRect::new(10, 12, 60, 4);
        set_white_block(
            &mut frame,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
            4,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );

        assert_eq!(
            find_face_up_white_block(&frame, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(23, 12),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
                height: 4,
            }))
        );
        assert_eq!(has_face_up_white_block(&frame, probe), Ok(true));
    }


    /// Cover inclusive white/width thresholds and a block touching the right edge.

    #[test]
    fn white_block_threshold_and_size_boundaries_are_exact() {
        let probe = PixelRect::new(10, 12, 60, FACE_UP_WHITE_SCAN_HEIGHT);

        let mut exact = rgba_frame(80, 40);
        set_white_block(
            &mut exact,
            probe.x,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );
        assert_eq!(
            find_face_up_white_block(&exact, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(probe.x as i32, probe.y as i32),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH,
                height: FACE_UP_WHITE_SCAN_HEIGHT,
            }))
        );

        let mut below_threshold = rgba_frame(80, 40);
        set_white_block(
            &mut below_threshold,
            probe.right() - FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM - 1,
        );
        assert_eq!(find_face_up_white_block(&below_threshold, probe), Ok(None));

        let mut at_right_edge = rgba_frame(80, 40);
        let right_edge_start = probe.right() - FACE_UP_WHITE_BLOCK_MIN_WIDTH;
        set_white_block(
            &mut at_right_edge,
            right_edge_start,
            probe.y,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH,
            FACE_UP_WHITE_SCAN_HEIGHT,
            FACE_UP_WHITE_CHANNEL_MINIMUM,
        );
        assert_eq!(
            find_face_up_white_block(&at_right_edge, probe),
            Ok(Some(WhiteBlockHit {
                anchor: PixelPoint::new(right_edge_start as i32, probe.y as i32),
                width: FACE_UP_WHITE_BLOCK_MIN_WIDTH,
                height: FACE_UP_WHITE_SCAN_HEIGHT,
            }))
        );
    }


    /// Reject insufficient width, incomplete height and scattered bright pixels.

    #[test]
    fn rejects_short_partial_and_speckled_white_patterns() {
        let probe = PixelRect::new(10, 12, 60, 4);

        let mut short = rgba_frame(80, 40);
        set_white_block(
            &mut short,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH - 1,
            4,
            255,
        );
        assert_eq!(find_face_up_white_block(&short, probe), Ok(None));

        let mut partial_height = rgba_frame(80, 40);
        set_white_block(
            &mut partial_height,
            23,
            12,
            FACE_UP_WHITE_BLOCK_MIN_WIDTH + 5,
            3,
            255,
        );
        assert_eq!(find_face_up_white_block(&partial_height, probe), Ok(None));

        let mut speckled = rgba_frame(80, 40);


        for x in probe.x..probe.right() {
            set_pixel(&mut speckled, x, probe.y + x % probe.height, [255; 3]);
        }
        assert_eq!(find_face_up_white_block(&speckled, probe), Ok(None));
    }


    /// Distinguish a qualifying gold button area from similarly bright neutral pixels.

    #[test]
    fn dialog_gold_requires_a_meaningful_gradient_area() {
        let mut frame = rgba_frame(20, 20);
        let probe = PixelRect::new(5, 5, 10, 10);


        for x in 5..13 {
            set_pixel(&mut frame, x, 5, [220, 170, 80]);
        }
        assert_eq!(has_dialog_gold_button(&frame, probe), Ok(true));

        let mut false_colour = rgba_frame(20, 20);


        for x in 5..13 {
            set_pixel(&mut false_colour, x, 5, [220, 215, 210]);
        }
        assert_eq!(has_dialog_gold_button(&false_colour, probe), Ok(false));
    }


    /// Check the black-versus-filled progress classification on the calibrated probe.

    #[test]
    fn solver_progress_uses_the_black_right_third_as_another_board() {
        let mut frame = rgba_frame(1_920, 1_080);
        assert_eq!(
            detect_game_progress_for_profile(&frame, &crate::tripeaks::PROFILE),
            Ok(GameProgress::AnotherBoard)
        );


        for y in SOLVER_PROGRESS_RIGHT_PROBE.y..SOLVER_PROGRESS_RIGHT_PROBE.bottom() {


            for x in SOLVER_PROGRESS_RIGHT_PROBE.x..SOLVER_PROGRESS_RIGHT_PROBE.right() {
                set_pixel(&mut frame, x, y, [172, 142, 84]);
            }
        }

        assert_eq!(
            detect_game_progress_for_profile(&frame, &crate::tripeaks::PROFILE),
            Ok(GameProgress::GameComplete)
        );
    }


    /// Keep both game modes on the same inclusive 65-black-pixel boundary.

    #[test]
    fn both_modes_share_the_exact_65_of_76_progress_threshold() {
        let mut frame = rgba_frame(1_920, 1_080);
        let probe = SOLVER_PROGRESS_RIGHT_PROBE;


        for coloured_pixels in [11_u32, 12] {


            for offset in 0..coloured_pixels {
                set_pixel(&mut frame, probe.x + offset, probe.y, [172, 142, 84]);
            }


            let expected = if coloured_pixels == 11 {
                GameProgress::AnotherBoard
            } else {
                GameProgress::GameComplete
            };


            for mode in [crate::game::GameMode::TriPeaks, crate::game::GameMode::Pyramid] {
                assert_eq!(
                    detect_game_progress_for_profile(&frame, mode.profile()),
                    Ok(expected)
                );
                assert_eq!(
                    measure_game_progress_for_profile(&frame, mode.profile()),
                    Ok(GameProgressEvidence {
                        classification: expected,
                        black_pixels: 76 - coloured_pixels,
                        total_pixels: 76,
                    })
                );
            }
        }
    }


    /// Replay recorded progress pixels to protect the interior probe from border regressions.

    #[test]
    fn captured_new_board_progress_samples_the_interior_instead_of_the_gold_border() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/solver-progress-new-board.json"
        ))
        .expect("recorded progress pixels must be valid JSON");
        let mut frame = rgba_frame(1_920, 1_080);
        let patches = fixture["patches"].as_array().expect("recorded patches");


        for patch in patches {
            let bounds = patch["bounds"].as_array().expect("patch bounds");
            let x = bounds[0].as_u64().unwrap() as u32;
            let y = bounds[1].as_u64().unwrap() as u32;
            let width = bounds[2].as_u64().unwrap() as u32;
            let height = bounds[3].as_u64().unwrap() as u32;
            let pixels = patch["rgb_hex"].as_str().expect("recorded RGB pixels");
            assert_eq!(pixels.len(), (width * height * 6) as usize);


            for offset in 0..width * height {
                let start = offset as usize * 6;
                let rgb = std::array::from_fn(|channel| {
                    let start = start + channel * 2;
                    u8::from_str_radix(&pixels[start..start + 2], 16).unwrap()
                });
                set_pixel(&mut frame, x + offset % width, y + offset / width, rgb);
            }
        }


        // The historical coordinates land on gold even on this fresh board.
        for y in 84..86 {


            for x in 1_050..1_088 {
                assert!(pixel_rgb(&frame, x, y).unwrap()[0] > 170);
            }
        }


        for mode in [crate::game::GameMode::TriPeaks, crate::game::GameMode::Pyramid] {
            assert_eq!(
                measure_game_progress_for_profile(&frame, mode.profile()),
                Ok(GameProgressEvidence {
                    classification: GameProgress::AnotherBoard,
                    black_pixels: 76,
                    total_pixels: 76,
                })
            );
        }


        // Reuse the captured gold interior from the filled left third to model
        // a fully filled bar. This is synthetic completion evidence, not a
        // claim that the source screenshot captured a completed game.
        for y in 87..89 {


            for offset in 0..38 {
                let filled_rgb = pixel_rgb(&frame, 830 + offset, y).unwrap();
                set_pixel(&mut frame, 1_050 + offset, y, filled_rgb);
            }
        }


        for mode in [crate::game::GameMode::TriPeaks, crate::game::GameMode::Pyramid] {
            assert_eq!(
                measure_game_progress_for_profile(&frame, mode.profile()),
                Ok(GameProgressEvidence {
                    classification: GameProgress::GameComplete,
                    black_pixels: 0,
                    total_pixels: 76,
                })
            );
        }
    }


    /// Reject truncated storage and a frame too short to contain the progress probe.

    #[test]
    fn progress_rejects_incomplete_frames_instead_of_classifying_them() {
        let mut truncated = rgba_frame(1_920, 1_080);
        truncated.pixels.truncate(4);


        for mode in [crate::game::GameMode::TriPeaks, crate::game::GameMode::Pyramid] {
            assert_eq!(
                measure_game_progress_for_profile(&truncated, mode.profile()),
                Err(HaloDetectionError::InvalidFrameLayout)
            );
            assert_eq!(
                measure_game_progress_for_profile(&rgba_frame(1_920, 88), mode.profile()),
                Err(HaloDetectionError::BoundsOutsideFrame)
            );
        }
    }


    /// Exercise invalid storage, out-of-frame probes and improperly nested card geometry.

    #[test]
    fn rejects_invalid_layout_and_geometry() {
        let mut invalid_layout = rgba_frame(220, 220);
        invalid_layout.pixels.truncate(4);
        assert_eq!(
            find_gold_halo_run(&invalid_layout, card_regions()),
            Err(HaloDetectionError::InvalidFrameLayout)
        );

        let mut outside = card_regions();
        outside.halo_bounds = PixelRect::new(28, 26, 300, 168);
        assert_eq!(
            find_gold_halo_run(&rgba_frame(220, 220), outside),
            Err(HaloDetectionError::BoundsOutsideFrame)
        );

        let mut not_nested = card_regions();
        not_nested.halo_bounds = PixelRect::new(50, 50, 100, 100);
        assert_eq!(
            find_gold_halo_run(&rgba_frame(220, 220), not_nested),
            Err(HaloDetectionError::CardOutsideHalo)
        );

        assert_eq!(
            has_face_up_white_block(&invalid_layout, PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
        assert_eq!(
            has_face_up_white_block(&rgba_frame(20, 20), PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::BoundsOutsideFrame)
        );
        assert_eq!(
            has_dialog_gold_button(&invalid_layout, PixelRect::new(10, 10, 20, 4)),
            Err(HaloDetectionError::InvalidFrameLayout)
        );
    }
}
