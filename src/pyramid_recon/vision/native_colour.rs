//! Bundled A* appearance roles with no reference tableau hue.
//!
//! Classic card identities and calibrated geometry remain unchanged. Positive
//! legend/control shapes, exclusive piles and conservative paper/surface evidence
//! replace background RGB comparisons. Custom profiles and Computer Vision keep
//! their original readers. These methods retain no execution or UNDO authority.

use super::card_paper::STOCK_PAPER_PRESENT_PERCENT;
use super::*;
use crate::image_evidence::{self, PaperEvidence, PaperPolicy};


/// Measured Classic paper and empty-surface policy, in RGB units and percentages.
/// Real exposed faces include229/240 white pixels; retained empty apertures0/240.
/// The180 paper floor retains warm selection as uncertain. Genuine empty lower
/// footprints have maximum adjacent delta60;64 retains a small measurement margin.
const PAPER: PaperPolicy = PaperPolicy {
    white_minimum: 245,
    possible_paper_minimum: 180,
    present_percent: 90,
    absent_percent: 5,
    surface_delta: 64,
};


/// Complete native bare-coral clearance at fixed tableau location 7|1.
/// Every RGB channel is retained; no card pixels are masked or inferred empty.
const UNDERWATER_CORAL_RGB: &[u8; 14_688] = include_bytes!(
    "../../../tests/fixtures/pyramid-recognition-159/underwater-coral-clearance.rgb"
);


/// Second byte-original measured rendering of the same fixed bare-art footprint.
const UNDERWATER_CORAL_VARIANT_RGB: &[u8; 14_688] = include_bytes!(
    "../../../tests/fixtures/pyramid-recognition-160/underwater-coral-clearance-variant.rgb"
);


/// Third byte-original measured rendering from the stopped 160 request.
const UNDERWATER_CORAL_THIRD_RGB: &[u8; 14_688] = include_bytes!(
    "../../../tests/fixtures/pyramid-recognition-161/underwater-coral-clearance-third.rgb"
);


/// Separate exact bare-coral rendering from the stopped 163 first MATCH.
/// This complete crop adds no structural reference, tolerance or masked pixels.
const UNDERWATER_CORAL_EXACT_RGB: &[u8; 14_688] = include_bytes!(
    "../../../tests/fixtures/pyramid-recognition-164/underwater-coral-clearance-exact.rgb"
);


/// Fixed native clearance rectangle for original one-based tableau location 7|1.
const UNDERWATER_CORAL_EXACT_BOUNDS: PixelRect = PixelRect::new(286, 588, 144, 34);


/// Entire byte-original native bare-rock footprint from the stopped 162 MATCH.
/// All 4,896 RGB pixels participate, with no tolerance or material-pixel exclusion.
const UNDERWATER_ROCK_RGB: &[u8; 14_688] =
    include_bytes!("../../../tests/fixtures/pyramid-recognition-163/underwater-rock-clearance.rgb");


/// Fixed native clearance rectangle for original one-based tableau location 7|6.
const UNDERWATER_ROCK_BOUNDS: PixelRect = PixelRect::new(1289, 588, 144, 34);


/// Complete byte-original 7|1 Coral appearance from retained 164 Request 13.
/// This adds one exact positive proof without changing structural material roles.
const UNDERWATER_CORAL_CHALLENGE_RGB: &[u8; 14_688] =
    include_bytes!("../../../tests/fixtures/pyramid-recognition-165/request13/clearance.rgb");


/// Complete byte-original 7|6 rock appearance from retained 164 Request 2.
/// All channels remain exact, including the three changed rightmost columns.
const UNDERWATER_ROCK_VARIANT_RGB: &[u8; 14_688] =
    include_bytes!("../../../tests/fixtures/pyramid-recognition-165/request2/clearance.rgb");


/// Complete retained Request 13 background below the removed 3H at original 7|2.
const UNDERWATER_CORAL_SEVEN_TWO_RGB: &[u8; 14_688] = include_bytes!(
    "../../../tests/fixtures/pyramid-recognition-165/request13/clearance-seven-two.rgb"
);


/// Fixed native clearance rectangle for original one-based tableau location 7|2.
const UNDERWATER_CORAL_SEVEN_TWO_BOUNDS: PixelRect = PixelRect::new(487, 588, 144, 34);


/// Recognise three additional complete retained Coral appearances at fixed slots.
/// Reject other locations or any changed RGB byte; checked bounds and pixel
/// access report malformed geometry. No old positive proof or material limit is
/// changed, and the caller must still exclude strong-white card presence.
fn measured_retained_underwater_surface(
    view: FrameView<'_>,
    bounds: PixelRect,
) -> Result<bool, &'static str> {
    let reference = if bounds == UNDERWATER_CORAL_EXACT_BOUNDS {
        UNDERWATER_CORAL_CHALLENGE_RGB
    } else if bounds == UNDERWATER_ROCK_BOUNDS {
        UNDERWATER_ROCK_VARIANT_RGB
    } else if bounds == UNDERWATER_CORAL_SEVEN_TWO_BOUNDS {
        UNDERWATER_CORAL_SEVEN_TWO_RGB
    } else {
        return Ok(false);
    };
    view.checked_bounds(bounds)
        .map_err(|_| "retained Coral clearance outside frame")?;
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view
            .pixel_rgb(x, y)
            .ok_or("retained Coral pixel outside frame")?;
        let expected = *reference
            .as_chunks::<3>()
            .0
            .get(index)
            .ok_or("retained Coral reference outside footprint")?;
        if actual != expected {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Only the three original calibration frames participate; 162 is evaluation only.
const CORAL_REFERENCES: [&[u8; 14_688]; 3] = [
    UNDERWATER_CORAL_RGB,
    UNDERWATER_CORAL_VARIANT_RGB,
    UNDERWATER_CORAL_THIRD_RGB,
];


/// Maximum corresponding-channel differences across pairs of the three originals.
const CORAL_ABSOLUTE_DELTA: [u8; 3] = [17, 22, 26];


/// Maximum signed spatial-edge differences across the same original pairs, in RGB.
const CORAL_EDGE_DELTA: [u16; 3] = [5, 7, 9];


/// Inclusive complete-footprint RGB sums measured only from those three originals.
const CORAL_SUM_RANGES: [[u32; 2]; 3] =
    [[458_826, 465_353], [511_146, 520_014], [491_004, 501_980]];


/// Complete byte-original bare Coral at original 7|5 from stopped 165 Request 2.
/// This adds one fixed exact proof without altering any earlier reference or limit.
const UNDERWATER_CORAL_SEVEN_FIVE_RGB: &[u8; 14_688] =
    include_bytes!("../../../tests/fixtures/pyramid-recognition-166/clearance.rgb");


/// Original native lower clearance below the removed Ace of clubs at 7|5.
const UNDERWATER_CORAL_SEVEN_FIVE_BOUNDS: PixelRect = PixelRect::new(1089, 588, 144, 34);


/// Prove the retained 7|5 background using every RGB byte at its fixed geometry.
/// Other locations or any changed byte refuse. Invalid frame bounds or missing
/// pixels return an error; the caller still excludes strong-white card presence.
/// This single calibration gives no tolerance or future-rendering guarantee.
pub(super) fn measured_coral_seven_five_surface(
    view: FrameView<'_>,
    bounds: PixelRect,
) -> Result<bool, &'static str> {
    if bounds != UNDERWATER_CORAL_SEVEN_FIVE_BOUNDS {
        return Ok(false);
    }
    view.checked_bounds(bounds)
        .map_err(|_| "7|5 clearance outside frame")?;
    let reference = UNDERWATER_CORAL_SEVEN_FIVE_RGB.as_chunks::<3>().0;
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view.pixel_rgb(x, y).ok_or("7|5 pixel outside frame")?;
        let expected = reference
            .get(index)
            .ok_or("7|5 reference outside footprint")?;
        if actual != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Classify aged, possible-paper and strong-white material using existing rules.
/// These are per-pixel veto roles, not card presence or permission to ignore pixels.
fn coral_material_roles([r, g, b]: [u8; 3]) -> [bool; 3] {
    let minimum = r.min(g).min(b);
    [
        super::card_paper::aged_pixel([r, g, b]),
        minimum >= PAPER.possible_paper_minimum,
        minimum >= PAPER.white_minimum,
    ]
}


/// Prove exact bare rock only at original 7|6 using the complete retained crop.
/// Every RGB channel must equal the single calibration frame; any changed pixel
/// refuses, including material overlays. Checked bounds/layout errors propagate.
/// This proof supplies positive absence before generic aged-colour heuristics,
/// while its caller retains strong-white presence and all scene/card/pile checks.
/// Calibration-frame success grants no future-rendering or live-input guarantee.
fn measured_rock_surface(view: FrameView<'_>, bounds: PixelRect) -> Result<bool, String> {
    if bounds != UNDERWATER_ROCK_BOUNDS {
        return Ok(false);
    }
    view.checked_bounds(bounds)
        .map_err(|_| "rock clearance outside frame")?;
    let reference = UNDERWATER_ROCK_RGB.as_chunks::<3>().0;
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view.pixel_rgb(x, y).ok_or("rock pixel outside frame")?;
        let expected = reference
            .get(index)
            .ok_or("rock reference outside footprint")?;
        if actual != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Prove the additional bare-coral rendering only at original 7|1 by exact RGB.
/// All 4,896 pixels must match; checked bounds and pixel errors propagate.
/// The caller retains strong-white presence and all scene, identity and pile checks.
/// This single-frame calibration provides no tolerance for future appearances.
fn measured_exact_coral_surface(view: FrameView<'_>, bounds: PixelRect) -> Result<bool, String> {
    if bounds != UNDERWATER_CORAL_EXACT_BOUNDS {
        return Ok(false);
    }
    view.checked_bounds(bounds)
        .map_err(|_| "exact coral clearance outside frame")?;
    let reference = UNDERWATER_CORAL_EXACT_RGB.as_chunks::<3>().0;
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view
            .pixel_rgb(x, y)
            .ok_or("exact coral pixel outside frame")?;
        let expected = reference
            .get(index)
            .ok_or("exact coral reference outside footprint")?;
        if actual != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Compare one signed edge in RGB; i16 differences stay within -255 through 255.
/// Their unsigned distance is at most 510, so no wrapping or saturation occurs.
fn coral_edge_matches(
    actual: [u8; 3],
    neighbour: [u8; 3],
    reference: [u8; 3],
    reference_neighbour: [u8; 3],
) -> bool {
    actual
        .into_iter()
        .zip(neighbour)
        .zip(reference.into_iter().zip(reference_neighbour))
        .zip(CORAL_EDGE_DELTA)
        .all(
            |(((actual, neighbour), (reference, reference_neighbour)), maximum)| {
                let observed = i16::from(actual) - i16::from(neighbour);
                let expected = i16::from(reference) - i16::from(reference_neighbour);
                observed.abs_diff(expected) <= maximum
            },
        )
}


/// Require all channels and all 9,614 edges to match one complete reference.
/// The caller fixes bounds to 144 by 34; row-major predecessors therefore exist
/// whenever their left/upper guards pass. Missing pixels or references are errors.
fn coral_reference_matches(
    view: FrameView<'_>,
    bounds: PixelRect,
    reference: &[u8; 14_688],
) -> Result<bool, String> {
    let pixels = reference.as_chunks::<3>().0;
    let width = usize::try_from(bounds.width).map_err(|_| "coral width overflow")?;
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view.pixel_rgb(x, y).ok_or("coral pixel outside frame")?;
        let expected = *pixels
            .get(index)
            .ok_or("coral reference outside footprint")?;
        if actual
            .into_iter()
            .zip(expected)
            .zip(CORAL_ABSOLUTE_DELTA)
            .any(|((actual, expected), maximum)| actual.abs_diff(expected) > maximum)
        {
            return Ok(false);
        }
        for (nx, ny, previous) in [
            (x > bounds.x).then(|| (x - 1, y, index - 1)),
            (y > bounds.y).then(|| (x, y - 1, index - width)),
        ]
        .into_iter()
        .flatten()
        {
            let neighbour = view
                .pixel_rgb(nx, ny)
                .ok_or("coral neighbour outside frame")?;
            let reference_neighbour = *pixels
                .get(previous)
                .ok_or("coral reference neighbour outside footprint")?;
            if !coral_edge_matches(actual, neighbour, expected, reference_neighbour) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}


/// Positively recognise the complete native bare-Coral footprint at original 7|1.
/// Every pixel participates in bounded RGB sums, material vetoes and one coherent
/// reference comparison. New material locations refuse even if structure agrees.
/// Only originals 159-161 establish limits; no fourth-reference fitting occurs.
/// The fixed 4,896 pixels sum to at most 1,248,480 per channel, fitting u32.
/// Invalid bounds/layout propagate errors; the caller retains its strong-white
/// presence veto, all other card/pile checks and sole input authority.
fn measured_coral_surface(view: FrameView<'_>, bounds: PixelRect) -> Result<bool, String> {
    if bounds != PixelRect::new(286, 588, 144, 34) {
        return Ok(false);
    }
    view.checked_bounds(bounds)
        .map_err(|_| "coral clearance outside frame")?;
    let mut sums = [0_u32; 3];
    let coordinates =
        (bounds.y..bounds.bottom()).flat_map(|y| (bounds.x..bounds.right()).map(move |x| (x, y)));
    for (index, (x, y)) in coordinates.enumerate() {
        let actual = view.pixel_rgb(x, y).ok_or("coral pixel outside frame")?;
        for (sum, value) in sums.iter_mut().zip(actual) {
            *sum += u32::from(value);
        }
        let mut allowed_roles = [false; 3];
        for reference in CORAL_REFERENCES {
            let pixel = *reference
                .as_chunks::<3>()
                .0
                .get(index)
                .ok_or("coral material reference outside footprint")?;
            for (allowed, role) in allowed_roles.iter_mut().zip(coral_material_roles(pixel)) {
                *allowed |= role;
            }
        }
        if coral_material_roles(actual)
            .into_iter()
            .zip(allowed_roles)
            .any(|(present, allowed)| present && !allowed)
        {
            return Ok(false);
        }
    }
    if sums
        .into_iter()
        .zip(CORAL_SUM_RANGES)
        .any(|(sum, [minimum, maximum])| sum < minimum || sum > maximum)
    {
        return Ok(false);
    }
    for reference in CORAL_REFERENCES {
        if coral_reference_matches(view, bounds, reference)? {
            return Ok(true);
        }
    }
    Ok(false)
}


/// Veto absence when aged material exceeds the existing possible-paper allowance.
/// Both the probe and complete clearance must stay within the unchanged percentage.
/// This only withholds absence; it grants neither card presence nor input authority.
/// Invalid rectangles and missing pixels propagate as errors before classification.
fn aged_material_blocks_absence(
    view: FrameView<'_>,
    probe: PixelRect,
    clearance: PixelRect,
) -> Result<bool, String> {
    for bounds in [probe, clearance] {
        let count = u128::from(super::card_paper::aged_pixels(view, bounds)?);
        let area = u128::from(bounds.width) * u128::from(bounds.height);
        if count * 100 > area * u128::from(PAPER.absent_percent) {
            return Ok(true);
        }
    }
    Ok(false)
}


/// Require the six complete native white legend separators, not a felt colour.
fn white_legend(view: FrameView<'_>) -> Result<bool, String> {
    for bounds in crate::pyramid::WHITE_SCENE_PROBES {
        let expected = u64::from(bounds.width) * u64::from(bounds.height);
        if image_evidence::count_paper(view, bounds, PAPER.white_minimum)? != expected {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Minimum per-channel lift above both neighbouring rows for a dim separator.
/// The actual mixed bars have minima 72..73; retained modal bars have 33..40.
/// Sixty-four preserves an eight-level margin below the measured supported sample.
const DIM_LEGEND_MINIMUM_LIFT: u8 = 64;


/// Maximum within-pixel channel spread for neutral dim separator ink.
/// The actual strips have spread at most 13; 32 permits native antialiasing.
const DIM_LEGEND_MAXIMUM_CHROMA: u8 = 32;


/// Require every dim-strip pixel to be neutral and above both complete surrounding rows.
/// Checked fixed native bounds and positive foreground lift reject missing strips,
/// coloured artwork and retained dimmed modal separators without inferring a scene.
fn dim_legend_strip(view: FrameView<'_>, bounds: PixelRect) -> Result<bool, String> {
    let above = PixelRect::new(
        bounds.x,
        bounds
            .y
            .checked_sub(3)
            .ok_or("legend upper row underflow")?,
        bounds.width,
        1,
    );
    let below = PixelRect::new(
        bounds.x,
        bounds
            .bottom()
            .checked_add(2)
            .ok_or("legend lower row overflow")?,
        bounds.width,
        1,
    );
    for checked in [bounds, above, below] {
        view.checked_bounds(checked)
            .map_err(|error| error.to_string())?;
    }
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let ink = view.pixel_rgb(x, y).ok_or("legend ink outside frame")?;
            let minimum = ink[0].min(ink[1]).min(ink[2]);
            let maximum = ink[0].max(ink[1]).max(ink[2]);
            if maximum - minimum > DIM_LEGEND_MAXIMUM_CHROMA {
                return Ok(false);
            }
            for surround_y in [above.y, below.y] {
                let surround = view
                    .pixel_rgb(x, surround_y)
                    .ok_or("legend surround outside frame")?;
                if ink.iter().zip(surround).any(|(foreground, background)| {
                    foreground.saturating_sub(background) < DIM_LEGEND_MINIMUM_LIFT
                }) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}


/// Preserve the all-white proof or require two adjacent complete bright strips and four dim ones.
/// Partial bright strips, any white speck in a dim strip, nonadjacent bright strips,
/// fully dimmed scenes and incomplete neutral/contrast evidence remain unknown.
/// This native A* scene alternative grants no card, control or input authority.
fn legend(view: FrameView<'_>) -> Result<bool, String> {
    if white_legend(view)? {
        return Ok(true);
    }
    let mut bright = 0;
    let mut previous_bright = false;
    let mut adjacent = false;
    for bounds in crate::pyramid::WHITE_SCENE_PROBES {
        let white_count = image_evidence::count_paper(view, bounds, PAPER.white_minimum)?;
        let complete = u64::from(bounds.width) * u64::from(bounds.height);
        if white_count == complete {
            bright += 1;
            adjacent |= previous_bright;
            previous_bright = true;
        } else {
            if white_count != 0 || !dim_legend_strip(view, bounds)? {
                return Ok(false);
            }
            previous_bright = false;
        }
    }
    Ok(bright == 2 && adjacent)
}


/// Identify strong white foreground using the unchanged Classic white threshold.
fn white(pixel: [u8; 3]) -> bool {
    pixel.iter().all(|channel| *channel >= PAPER.white_minimum)
}


/// Identify bright marker ink; full shape and negative space remain mandatory.
fn marker(pixel: [u8; 3]) -> bool {
    pixel[0] >= 160 && pixel[1] >= 120
}


/// Identify enabled orange Undo All cores in RGB channel units.
/// The measured red163, red-minus-green49 and green-minus-blue13 boundaries
/// preserve the complete retained and Xbox shapes despite antialiased fringes.
/// Disabled originals remain below the brightness bound; negative space is checked.
fn undo_ink(pixel: [u8; 3]) -> bool {
    pixel[0] >= 163
        && pixel[0].saturating_sub(pixel[1]) >= 49
        && pixel[1].saturating_sub(pixel[2]) >= 13
}


/// Minimum RGB spread at both endpoints of a sharp coloured-artwork edge.
/// Retained neutral/tinted corruption has spread at most21; the failing Xbox
/// edge endpoints have spread at least56.32 lies inside that measured gap;
/// it identifies no hue and does not change the existing64 continuity limit.
const ARTWORK_MINIMUM_CHROMA: u8 = 32;


/// Prove a paper-free clearance whose sharp edges remain chromatic at both ends.
/// Every clearance pixel must be below the possible-paper threshold. Neutral
/// occlusions/card fringes at any sharp edge refuse; smooth regions retain the
/// original policy. This bounded native appearance proof grants no input.
/// Returns invalid frame or rectangle errors without inferring absence.
fn artwork_surface(view: FrameView<'_>, bounds: PixelRect) -> Result<bool, String> {
    if image_evidence::count_paper(view, bounds, PAPER.possible_paper_minimum)? != 0 {
        return Ok(false);
    }
    let chroma = |pixel: [u8; 3]| {
        pixel.into_iter().max().unwrap_or(0) - pixel.into_iter().min().unwrap_or(0)
    };
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let current = view.pixel_rgb(x, y).ok_or("artwork pixel outside frame")?;
            for (nx, ny) in [
                (x > bounds.x).then(|| (x - 1, y)),
                (y > bounds.y).then(|| (x, y - 1)),
            ]
            .into_iter()
            .flatten()
            {
                let neighbour = view
                    .pixel_rgb(nx, ny)
                    .ok_or("artwork neighbour outside frame")?;
                if current
                    .iter()
                    .zip(neighbour)
                    .any(|(a, b)| a.abs_diff(b) > PAPER.surface_delta)
                    && (chroma(current) < ARTWORK_MINIMUM_CHROMA
                        || chroma(neighbour) < ARTWORK_MINIMUM_CHROMA)
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}


impl ReconVision {
    /// Load private complete corner-ink families and colour-independent A* roles.
    /// Classic calibration remains unchanged; custom versioned policies stay frozen.
    /// No serialized/custom identifier can opt in; no input or restoration is implied.
    /// Returns embedded calibration errors before any guest interaction.
    pub(crate) fn bundled_native_v23() -> Result<Self, String> {
        let mut vision = Self::bundled_native_v22()?;
        vision.profile_id = "pyramid-native-1920-classic-v23".to_owned();
        vision.colour_independent = true;
        for reader in [&mut vision.tableau_reader, &mut vision.stock_reader] {
            let glyphs = reader
                .glyphs
                .as_ref()
                .ok_or_else(|| "Bundled native glyph banks missing".to_owned())?;
            reader.foreground = Some(super::foreground_card::Reader::new(glyphs)?);
        }
        Ok(vision)
    }


    /// Conservatively veto a New Game caption over a coloured native gameplay surface.
    /// All legend bars and both static surface anchors are required; card identities
    /// and dynamic stock controls are deliberately irrelevant to this input-free veto.
    /// Legacy/custom readers return false and retain their existing shared veto.
    /// Returns invalid frame-layout errors without granting terminal or input authority.
    pub(crate) fn native_gameplay_veto(&self, frame: &CapturedFrame) -> Result<bool, String> {
        if !self.colour_independent
            || frame.width != NATIVE_BOUNDS.width
            || frame.height != NATIVE_BOUNDS.height
        {
            return Ok(false);
        }
        let view = frame
            .as_view()
            .map_err(|_| "invalid native gameplay-veto layout".to_owned())?;
        for (index, pattern) in self.board.scene.iter().take(2).enumerate() {
            if !self.native_scene_anchor(view, index, pattern)? {
                return Ok(false);
            }
        }
        Ok(self.board.scene.len() >= 2)
    }


    /// Match one dynamic white control, including its complete negative space.
    /// Zero-core rasters are explicitly negative evidence and grant nothing alone.
    pub(super) fn native_shape(
        &self,
        view: FrameView<'_>,
        pattern: &Pattern,
        minimum_positive: usize,
    ) -> Result<bool, String> {
        if pattern.matches(view, (0, 0))? {
            return Ok(true);
        }
        if !self.colour_independent {
            return Ok(false);
        }
        image_evidence::binary_shape_matches(
            view,
            pattern.bounds,
            &pattern.pixels,
            white,
            minimum_positive,
        )
    }


    /// Match a complete marker's bright ink and negative space without its background hue.
    pub(super) fn native_marker(
        &self,
        view: FrameView<'_>,
        pattern: &Pattern,
    ) -> Result<bool, String> {
        if pattern.matches(view, (0, 0))? {
            return Ok(true);
        }
        Ok(self.colour_independent
            && image_evidence::binary_shape_matches(
                view,
                pattern.bounds,
                &pattern.pixels,
                marker,
                1,
            )?)
    }


    /// Match one whole measured white-shape variant, never a mixture of variants.
    #[cfg(test)]
    pub(super) fn native_shapes(
        &self,
        view: FrameView<'_>,
        patterns: &[Pattern],
    ) -> Result<bool, String> {
        for pattern in patterns {
            if self.native_shape(view, pattern, 1)? {
                return Ok(true);
            }
        }
        Ok(false)
    }


    /// Verify one original scene role, coupled to every positive legend separator.
    /// Static patches reject paper/overlays; the dynamic stock role requires its
    /// full positive white shape. Unknown controls do not become empty stock.
    pub(super) fn native_scene_anchor(
        &self,
        view: FrameView<'_>,
        index: usize,
        pattern: &Pattern,
    ) -> Result<bool, String> {
        if !self.colour_independent {
            return pattern.matches(view, (0, 0));
        }
        if !legend(view)? {
            return Ok(false);
        }
        if index == EXECUTION_EMPTY_STOCK_SCENE_INDEX {
            return self.native_shape(view, pattern, 1);
        }
        let paper =
            image_evidence::count_paper(view, pattern.bounds, PAPER.possible_paper_minimum)?;
        let total = u64::from(pattern.bounds.width) * u64::from(pattern.bounds.height);
        Ok(paper * 100 <= total * u64::from(PAPER.absent_percent))
    }


    /// Require every scene role using only the explicitly selected bundled policy.
    pub(super) fn board_scene_matches(&self, view: FrameView<'_>) -> Result<bool, String> {
        if !self.colour_independent {
            return self.board.matches(view);
        }
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if !self.native_scene_anchor(view, index, pattern)? {
                return Ok(false);
            }
        }
        Ok(true)
    }


    /// Empty stock is the inverse of high white/cream or bounded aged-paper coverage in its full footprint.
    /// Original light paper or bounded aged material counts towards occupied stock;
    /// fewer than 45% means Empty, without reference background or continuity checks.
    /// Scene/modal and unique card/empty guards belong to the caller. High-paper
    /// unreadable cards remain Unknown; black-card themes are outside this policy.
    /// Cross-products use u128, covering every possible u32 rectangle without rounding.
    pub(super) fn native_empty(
        &self,
        view: FrameView<'_>,
        bounds: PixelRect,
        pattern: &Pattern,
    ) -> Result<bool, String> {
        if !self.colour_independent {
            return pattern.matches(view, (0, 0));
        }
        let paper = u128::from(super::card_paper::stock_pixels(view, bounds)?);
        let area = u128::from(bounds.width) * u128::from(bounds.height);
        if area == 0 {
            return Err("Stock paper footprint must be nonempty".to_owned());
        }
        Ok(paper * 100 < area * u128::from(STOCK_PAPER_PRESENT_PERCENT))
    }


    /// Preserve a measured exhausted/circle layout separately from ordinary pile absence.
    /// A paper-free stock footprint alone cannot assert a particular control layout.
    /// Legacy RGB and positive control shapes remain unchanged; the bundled surface
    /// alternative retains its measured twelve-pixel inset and continuity bound.
    pub(super) fn native_stock_layout(
        &self,
        view: FrameView<'_>,
        bounds: PixelRect,
        pattern: &Pattern,
    ) -> Result<bool, String> {
        if pattern.matches(view, (0, 0))? {
            return Ok(true);
        }
        if !self.colour_independent {
            return Ok(false);
        }
        if pattern.pixels.iter().flatten().any(|pixel| white(*pixel)) {
            return self.native_shape(view, pattern, 1);
        }
        let inset = PixelRect::new(
            bounds.x.checked_add(12).ok_or("stock inset x overflow")?,
            bounds.y.checked_add(12).ok_or("stock inset y overflow")?,
            bounds.width.checked_sub(24).ok_or("stock inset width")?,
            bounds.height.checked_sub(24).ok_or("stock inset height")?,
        );
        Ok(image_evidence::paper_evidence(view, inset, inset, PAPER)? == PaperEvidence::Absent)
    }

    /// Read tableau occupancy with the original fixed-slot/child accessibility rules.
    /// Only A* opts into hue-free paper/surface evidence; CV uses its original reader.
    pub(super) fn native_occupancy(
        &self,
        frame: &CapturedFrame,
    ) -> Result<[crate::pyramid::FaceEvidence; crate::pyramid::TARGET_COUNT], String> {
        if !self.colour_independent {
            return crate::pyramid::tableau_evidence(frame).map_err(|error| error.to_string());
        }
        crate::pyramid::tableau_evidence_with(frame, |frame, slot| {
            use crate::{detector::HaloDetectionError, pyramid::FaceEvidence};
            let view = frame
                .as_view()
                .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;
            let probe = PixelRect::new(slot.bounds.x + 40, slot.bounds.y + 178, 60, 4);
            let clearance = PixelRect::new(slot.bounds.x - 2, slot.bounds.y + 156, 144, 34);
            let evidence = image_evidence::paper_evidence(view, probe, clearance, PAPER)
                .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?;
            if evidence != PaperEvidence::Present
                && measured_coral_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Absent);
            }
            if evidence != PaperEvidence::Present
                && measured_exact_coral_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Absent);
            }
            if evidence != PaperEvidence::Present
                && measured_rock_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Absent);
            }
            if evidence != PaperEvidence::Present
                && measured_retained_underwater_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Absent);
            }
            if evidence != PaperEvidence::Present
                && measured_coral_seven_five_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Absent);
            }
            if evidence != PaperEvidence::Present
                && aged_material_blocks_absence(view, probe, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                return Ok(FaceEvidence::Unknown);
            }
            let evidence = if evidence == PaperEvidence::Unknown
                && artwork_surface(view, clearance)
                    .map_err(|_| HaloDetectionError::BoundsOutsideFrame)?
            {
                PaperEvidence::Absent
            } else {
                evidence
            };
            Ok(match evidence {
                PaperEvidence::Present => FaceEvidence::Present,
                PaperEvidence::Absent => FaceEvidence::Absent,
                PaperEvidence::Unknown => FaceEvidence::Unknown,
            })
        })
        .map_err(|error| error.to_string())
    }


    /// Check a final unobscured band with surrounding edges, not a named felt hue.
    /// Returns false for warm paper, remaining card edges or malformed/unknown surfaces.
    pub(super) fn native_absence_band(
        &self,
        view: FrameView<'_>,
        band: PixelRect,
    ) -> Result<bool, String> {
        if !self.colour_independent {
            for y in band.y..band.y + band.height {
                for x in band.x..band.x + band.width {
                    if !crate::pyramid::is_felt(
                        view.pixel_rgb(x, y).ok_or("missing clear-band pixel")?,
                    ) {
                        return Ok(false);
                    }
                }
            }
            return Ok(true);
        }
        let clearance = PixelRect::new(
            band.x.checked_sub(2).ok_or("clear band x underflow")?,
            band.y.checked_sub(2).ok_or("clear band y underflow")?,
            band.width
                .checked_add(4)
                .ok_or("clear band width overflow")?,
            band.height
                .checked_add(4)
                .ok_or("clear band height overflow")?,
        );
        Ok(
            image_evidence::paper_evidence(view, band, clearance, PAPER)? == PaperEvidence::Absent
                && !aged_material_blocks_absence(view, band, clearance)?,
        )
    }


    /// Veto alternate final bands when an original band or its clearance contains card material.
    ///
    /// Uses the unchanged possible-paper percentage and aged-material veto. This
    /// material-only check never proves absence: a caller must independently prove
    /// an entire alternate layout, including its original continuity checks.
    /// Unsupported policies always veto; malformed bounds return an error.
    pub(super) fn native_clear_band_material_veto(
        &self,
        view: FrameView<'_>,
        band: PixelRect,
    ) -> Result<bool, String> {
        if !self.colour_independent {
            return Ok(true);
        }
        let clearance = PixelRect::new(
            band.x.checked_sub(2).ok_or("clear band x underflow")?,
            band.y.checked_sub(2).ok_or("clear band y underflow")?,
            band.width
                .checked_add(4)
                .ok_or("clear band width overflow")?,
            band.height
                .checked_add(4)
                .ok_or("clear band height overflow")?,
        );
        for bounds in [band, clearance] {
            let paper = u128::from(image_evidence::count_paper(
                view,
                bounds,
                PAPER.possible_paper_minimum,
            )?);
            let area = u128::from(bounds.width) * u128::from(bounds.height);
            if paper * 100 > area * u128::from(PAPER.absent_percent) {
                return Ok(true);
            }
        }
        aged_material_blocks_absence(view, band, clearance)
    }


    /// Recognise the complete enabled Undo All orange shape; disabled ink grants nothing.
    pub(super) fn native_undo(&self, view: FrameView<'_>) -> Result<bool, String> {
        if self.board.undo_all.matches(view, (0, 0))? {
            return Ok(true);
        }
        Ok(self.colour_independent
            && legend(view)?
            && image_evidence::binary_shape_matches(
                view,
                self.board.undo_all.bounds,
                &self.board.undo_all.pixels,
                undo_ink,
                1,
            )?)
    }


    /// Match all five specific modal components while treating backdrop hue separately.
    /// Two dimmed toolbar patches cannot stand in for title/body/Cancel/OK evidence.
    /// Normal and post-confirmation OK alternatives remain distinct complete proofs.
    pub(super) fn dialog_matches(
        &self,
        view: FrameView<'_>,
        after_confirmation: bool,
    ) -> Result<bool, String> {
        if self.dialog.matches(view, after_confirmation)? {
            return Ok(true);
        }
        if !self.colour_independent || white_legend(view)? {
            return Ok(false);
        }
        for (index, pattern) in self.dialog.signature.iter().take(5).enumerate() {
            if pattern.matches(view, (0, 0))? {
                continue;
            }
            if index == 4
                && after_confirmation
                && let Some(alternate) = self.dialog.post_confirm_ok.as_ref()
                && alternate.matches(view, (0, 0))?
            {
                continue;
            }
            return Ok(false);
        }
        Ok(true)
    }
}
