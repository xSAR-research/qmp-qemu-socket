use thiserror::Error;

use crate::{
    capture::CapturedFrame,
    cards::CardRegionPixels,
    detector::{
        HaloDetectionError, find_gold_run_in_bounds, find_tableau_gold_halo_run,
        has_face_up_white_block, has_gameplay_felt_for_profile,
    },
    game::{GameMode, GameProfile, GuidedAction, TargetSelectionPolicy},
    geometry::PixelPoint,
    parameters::HALO_GOLD_LINE_OFFSET_Y,
};

/// Compact set of one-based tableau rows.
///
/// Bit zero represents the top row. A profile may use at most eight rows;
/// construction keeps every bit outside that profile clear.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowMask(u8);

impl RowMask {
    pub const EMPTY: Self = Self(0);

    pub const fn single_for(row: u8, row_count: u8) -> Option<Self> {
        if row == 0 || row > row_count || row > u8::BITS as u8 {
            None
        } else {
            Some(Self(1 << (row - 1)))
        }
    }

    pub const fn from_bits_for(bits: u8, row_count: u8) -> Option<Self> {
        let valid = Self::all_for(row_count).0;
        if bits & !valid == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn all_for(row_count: u8) -> Self {
        if row_count >= u8::BITS as u8 {
            Self(u8::MAX)
        } else if row_count == 0 {
            Self::EMPTY
        } else {
            Self((1 << row_count) - 1)
        }
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, row: u8) -> bool {
        if row == 0 || row > u8::BITS as u8 {
            false
        } else {
            self.0 & (1 << (row - 1)) != 0
        }
    }

    pub fn insert(&mut self, row: u8) -> bool {
        if row == 0 || row > u8::BITS as u8 {
            return false;
        }
        let row_mask = Self(1 << (row - 1));
        let previous = self.0;
        self.0 |= row_mask.0;
        self.0 != previous
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn row_upper_for(self, row_count: u8) -> Option<u8> {
        let mut row = 1;
        while row <= row_count {
            if self.contains(row) {
                return Some(row);
            }
            row += 1;
        }
        None
    }

    pub const fn row_lower_for(self, row_count: u8) -> Option<u8> {
        let mut row = row_count;
        while row > 0 {
            if self.contains(row) {
                return Some(row);
            }
            row -= 1;
        }
        None
    }

    fn rows_lower_to_upper(self, row_count: u8) -> impl Iterator<Item = u8> {
        (1..=row_count).rev().filter(move |row| self.contains(*row))
    }
}

/// Persistent row-window hint owned by the worker.
///
/// It is never click authority: fresh halo geometry and the worker's later
/// validation still decide whether an action may be sent. A complete bounded
/// halo fallback remains available whenever the row window finds no target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauScanState {
    mode: GameMode,
    active_rows: RowMask,
}

impl TableauScanState {
    /// TriPeaks starts with only the bottom row face-up.
    pub const fn initial() -> Self {
        Self::for_mode(GameMode::TriPeaks)
    }

    pub const fn for_mode(mode: GameMode) -> Self {
        let profile = mode.profile();
        Self {
            mode,
            active_rows: RowMask(profile.initial_active_rows),
        }
    }

    pub const fn from_active_rows(active_rows: RowMask) -> Option<Self> {
        Self::from_active_rows_for_mode(GameMode::TriPeaks, active_rows)
    }

    pub const fn from_active_rows_for_mode(mode: GameMode, active_rows: RowMask) -> Option<Self> {
        let profile = mode.profile();
        if active_rows.is_empty() || active_rows.bits() & !profile.valid_row_bits() != 0 {
            None
        } else {
            Some(Self { mode, active_rows })
        }
    }

    pub const fn mode(self) -> GameMode {
        self.mode
    }

    pub const fn active_rows(self) -> RowMask {
        self.active_rows
    }

    /// Upper bound derived from the non-empty active mask.
    pub const fn row_upper(self) -> u8 {
        match self
            .active_rows
            .row_upper_for(self.mode.profile().tableau_rows.len() as u8)
        {
            Some(row) => row,
            None => unreachable!(),
        }
    }

    /// Lower bound derived from the non-empty active mask.
    pub const fn row_lower(self) -> u8 {
        match self
            .active_rows
            .row_lower_for(self.mode.profile().tableau_rows.len() as u8)
        {
            Some(row) => row,
            None => unreachable!(),
        }
    }

    /// Commit one non-empty row observation.
    ///
    /// This state remains an optimisation hint rather than click authority:
    /// halo geometry, fresh-target matching and action-effect verification are
    /// still required before input. The one fresh observation may update this
    /// hint without becoming click authority.
    pub fn reconcile_observation(&mut self, observed: Option<RowMask>) -> bool {
        let Some(observed) = observed else {
            return false;
        };
        if observed.is_empty() || observed == self.active_rows {
            return false;
        }

        self.active_rows = observed;
        true
    }
}

impl Default for TableauScanState {
    fn default() -> Self {
        Self::initial()
    }
}

/// Prediction plus independently observed row occupancy for this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameAnalysis {
    pub prediction: PredictedAction,
    /// A non-empty proposed mask, or `None` when the frame provided no safe
    /// evidence with which to replace/extend the worker's current hint.
    pub observed_rows: Option<RowMask>,
    /// Number of individually exposed face-up cards on the three-card top row.
    /// This is observational evidence used to recognise the final tableau move;
    /// it is never sufficient on its own to authorise input.
    pub top_row_face_up_count: u8,
}

/// A read-only interpretation of Microsoft Solver's current gold highlight.
///
/// This is deliberately only a prediction. Callers must not treat a value as
/// authority to send guest input without the later input-safety milestone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictedAction {
    /// The frame is retained for positioning work, but this mode has no
    /// approved detector or guest-input geometry.
    CalibrationOnly {
        mode: GameMode,
    },
    NoHighlight,
    Action(GuidedAction),
    Ambiguous {
        highlight_count: usize,
    },
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TrackerError {
    #[error(
        "captured frame must be exactly {expected_width}x{expected_height}, got {actual_width}x{actual_height}"
    )]
    UnexpectedFrameDimensions {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
    #[error("captured frame has an invalid pixel layout")]
    InvalidFrameLayout,
    #[error("{target} halo detection failed: {source}")]
    HaloDetection {
        target: &'static str,
        #[source]
        source: HaloDetectionError,
    },
    #[error("bottom-target calibration index {index} has no valid action")]
    MissingBottomTarget { index: usize },
    #[error("tableau calibration index {index} has no valid position")]
    MissingTableauPosition { index: usize },
    #[error("{target} calibrated halo-line coordinate overflows")]
    HaloLineOverflow { target: &'static str },
    #[error(
        "{target} gold run begins at y={actual_y}, not the calibrated halo line y={expected_y}"
    )]
    UnexpectedHaloLine {
        target: &'static str,
        actual_y: i32,
        expected_y: i32,
    },
    #[error("tableau gold anchor plus the audited click offset overflows")]
    ClickOffsetOverflow,
    #[error(
        "tableau gold anchor maps to ({actual_x}, {actual_y}), not calibrated centre ({expected_x}, {expected_y})"
    )]
    ClickOffsetMismatch {
        actual_x: i32,
        actual_y: i32,
        expected_x: i32,
        expected_y: i32,
    },
}

/// Predict the action identified by the Solver halo in one calibrated frame.
///
/// The frame must match the audited 1920x1080 profile exactly. Every
/// qualifying highlight is collected before a result is selected: zero is an
/// observation, one is actionable metadata, and more than one is explicitly
/// ambiguous. No guest input is performed here.
pub fn analyse_frame(frame: &CapturedFrame) -> Result<PredictedAction, TrackerError> {
    Ok(analyse_frame_with_state(frame, &TableauScanState::initial())?.prediction)
}

/// Report whether a frame still shows the green gameplay board.
///
/// This gate is intentionally separate from [`analyse_frame_with_state`]. The
/// worker applies it before accepting a halo, while read-only analysis remains
/// available for diagnostics and detector tests.
pub fn is_gameplay_scene(frame: &CapturedFrame) -> Result<bool, TrackerError> {
    is_gameplay_scene_for_mode(frame, GameMode::TriPeaks)
}

pub fn is_gameplay_scene_for_mode(
    frame: &CapturedFrame,
    mode: GameMode,
) -> Result<bool, TrackerError> {
    let profile = mode.profile();
    validate_frame(frame, profile)?;
    has_gameplay_felt_for_profile(frame, profile).map_err(|source| TrackerError::HaloDetection {
        target: "gameplay felt probe",
        source,
    })
}

/// Analyse one immutable frame using a worker-owned tableau row hint.
///
/// Profile-ordered bottom targets are checked first, followed by active
/// tableau rows from lower to upper. An active row without a halo is checked
/// for face-up white, allowing an empty lower row to retire even when an upper
/// active row holds the target. If bottom targets plus active rows yield no candidate, inactive
/// white bands are reconciled and newly active rows are checked. If those
/// bounded checks still find no target, every as-yet-unscanned row receives a
/// fallback halo check so stale state cannot turn a real target into a miss.
pub fn analyse_frame_with_state(
    frame: &CapturedFrame,
    state: &TableauScanState,
) -> Result<FrameAnalysis, TrackerError> {
    let profile = state.mode().profile();
    analyse_frame_for_profile(frame, state, profile)
}

fn analyse_frame_for_profile(
    frame: &CapturedFrame,
    state: &TableauScanState,
    profile: &GameProfile,
) -> Result<FrameAnalysis, TrackerError> {
    validate_frame(frame, profile)?;

    let mut candidates = Vec::with_capacity(
        profile
            .tableau_cards
            .len()
            .saturating_add(profile.bottom_targets.len()),
    );
    let mut scanned_rows = RowMask::EMPTY;
    let mut halo_rows = RowMask::EMPTY;
    let mut observed_rows = RowMask::EMPTY;
    let row_count = profile.tableau_rows.len() as u8;
    let all_rows = RowMask::all_for(row_count);
    let top_row_face_up_count = count_face_up_cards_in_row(frame, profile, 1)?;

    for (index, target) in profile.bottom_targets.iter().enumerate() {
        let hit = find_gold_run_in_bounds(frame, target.halo_scan_bounds).map_err(|source| {
            TrackerError::HaloDetection {
                target: target.label,
                source,
            }
        })?;
        let Some(hit) = hit else {
            continue;
        };
        let action = profile
            .bottom_action(index, hit.anchor)
            .ok_or(TrackerError::MissingBottomTarget { index })?;
        if profile.target_selection == TargetSelectionPolicy::FirstByPriority {
            return Ok(FrameAnalysis {
                prediction: PredictedAction::Action(action),
                observed_rows: None,
                top_row_face_up_count,
            });
        }
        candidates.push(action);
    }

    for row in state.active_rows().rows_lower_to_upper(row_count) {
        scan_tableau_row(
            frame,
            profile,
            row,
            &mut scanned_rows,
            &mut halo_rows,
            &mut candidates,
        )?;

        if halo_rows.contains(row) || row_has_face_up_card(frame, profile, row)? {
            observed_rows.insert(row);
        }
        if profile.target_selection == TargetSelectionPolicy::FirstByPriority
            && !candidates.is_empty()
        {
            break;
        }
    }

    if profile.target_selection == TargetSelectionPolicy::FirstByPriority && !candidates.is_empty()
    {
        return Ok(FrameAnalysis {
            prediction: PredictedAction::Action(candidates[0]),
            observed_rows: (!observed_rows.is_empty()).then_some(observed_rows),
            top_row_face_up_count,
        });
    }

    // White probes are the cheap widening step: even when the current window
    // already contains a halo, exposed cards on another row extend the window
    // and receive a bounded halo scan in this same frame.
    let inactive_rows = all_rows.difference(state.active_rows());
    for row in inactive_rows.rows_lower_to_upper(row_count) {
        if row_has_face_up_card(frame, profile, row)? {
            observed_rows.insert(row);
        }
    }

    for row in observed_rows
        .difference(scanned_rows)
        .rows_lower_to_upper(row_count)
    {
        scan_tableau_row(
            frame,
            profile,
            row,
            &mut scanned_rows,
            &mut halo_rows,
            &mut candidates,
        )?;
        if profile.target_selection == TargetSelectionPolicy::FirstByPriority
            && !candidates.is_empty()
        {
            break;
        }
    }

    // If the bounded row window and white probes still find no target, perform
    // one full calibrated fallback. Successful ordinary frames therefore scan
    // only plausible rows, while stale hints and imperfect white evidence fail
    // safe rather than hiding a target.
    if candidates.is_empty() {
        for row in all_rows
            .difference(scanned_rows)
            .rows_lower_to_upper(row_count)
        {
            scan_tableau_row(
                frame,
                profile,
                row,
                &mut scanned_rows,
                &mut halo_rows,
                &mut candidates,
            )?;
            if profile.target_selection == TargetSelectionPolicy::FirstByPriority
                && !candidates.is_empty()
            {
                break;
            }
        }
    }

    // A fallback halo is stronger evidence than a failed white probe. This is
    // also what lets stale state recover without becoming detection authority.
    observed_rows = observed_rows.union(halo_rows);

    let prediction = match candidates.as_slice() {
        [] => PredictedAction::NoHighlight,
        [action] => PredictedAction::Action(*action),
        [first, ..] if profile.target_selection == TargetSelectionPolicy::FirstByPriority => {
            PredictedAction::Action(*first)
        }
        _ => PredictedAction::Ambiguous {
            highlight_count: candidates.len(),
        },
    };

    Ok(FrameAnalysis {
        prediction,
        observed_rows: (!observed_rows.is_empty()).then_some(observed_rows),
        top_row_face_up_count,
    })
}

fn count_face_up_cards_in_row(
    frame: &CapturedFrame,
    game: &GameProfile,
    row: u8,
) -> Result<u8, TrackerError> {
    let profile = &game.tableau_rows[row as usize - 1];
    let mut count = 0_u8;
    for regions in &game.tableau_cards[profile.first_card_index..profile.card_end_index()] {
        let bounds = crate::geometry::PixelRect::new(
            regions.card_bounds.x,
            profile.face_probe_bounds.y,
            regions.card_bounds.width,
            profile.face_probe_bounds.height,
        );
        if has_face_up_white_block(frame, bounds).map_err(|source| TrackerError::HaloDetection {
            target: "tableau per-card face probe",
            source,
        })? {
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}

fn row_has_face_up_card(
    frame: &CapturedFrame,
    game: &GameProfile,
    row: u8,
) -> Result<bool, TrackerError> {
    let profile = &game.tableau_rows[row as usize - 1];
    has_face_up_white_block(frame, profile.face_probe_bounds).map_err(|source| {
        TrackerError::HaloDetection {
            target: "tableau face probe",
            source,
        }
    })
}

fn scan_tableau_row(
    frame: &CapturedFrame,
    game: &GameProfile,
    row: u8,
    scanned_rows: &mut RowMask,
    halo_rows: &mut RowMask,
    candidates: &mut Vec<GuidedAction>,
) -> Result<(), TrackerError> {
    if scanned_rows.contains(row) {
        return Ok(());
    }

    let profile = &game.tableau_rows[row as usize - 1];
    scanned_rows.insert(row);

    for index in profile.first_card_index..profile.card_end_index() {
        let regions = game.tableau_cards[index];
        let Some(anchor) =
            calibrated_tableau_halo_anchor(frame, regions, game.tableau_click_offset.x)?
        else {
            continue;
        };

        halo_rows.insert(row);
        candidates.push(tableau_candidate(game, index, regions, anchor)?);
    }

    Ok(())
}

fn tableau_candidate(
    game: &GameProfile,
    index: usize,
    regions: CardRegionPixels,
    anchor: PixelPoint,
) -> Result<GuidedAction, TrackerError> {
    // The calibrated halo-to-centre offset is an independent geometry check. The
    // emitted point is the canonical per-slot centre, so a shifted or
    // malformed halo fails closed instead of moving the proposed click away
    // from the known card centre.
    let offset_point = PixelPoint::new(
        anchor
            .x
            .checked_add(game.tableau_click_offset.x)
            .ok_or(TrackerError::ClickOffsetOverflow)?,
        anchor
            .y
            .checked_add(game.tableau_click_offset.y)
            .ok_or(TrackerError::ClickOffsetOverflow)?,
    );
    if offset_point != regions.click_point {
        return Err(TrackerError::ClickOffsetMismatch {
            actual_x: offset_point.x,
            actual_y: offset_point.y,
            expected_x: regions.click_point.x,
            expected_y: regions.click_point.y,
        });
    }

    game.tableau_action(index, anchor)
        .ok_or(TrackerError::MissingTableauPosition { index })
}

fn validate_frame(frame: &CapturedFrame, profile: &GameProfile) -> Result<(), TrackerError> {
    if (frame.width, frame.height) != (profile.frame_width, profile.frame_height) {
        return Err(TrackerError::UnexpectedFrameDimensions {
            expected_width: profile.frame_width,
            expected_height: profile.frame_height,
            actual_width: frame.width,
            actual_height: frame.height,
        });
    }
    if !frame.is_layout_valid() {
        return Err(TrackerError::InvalidFrameLayout);
    }
    Ok(())
}

fn calibrated_tableau_halo_anchor(
    frame: &CapturedFrame,
    regions: CardRegionPixels,
    click_offset_x: i32,
) -> Result<Option<PixelPoint>, TrackerError> {
    let Some(hit) =
        find_tableau_gold_halo_run(frame, regions, click_offset_x).map_err(|source| {
            TrackerError::HaloDetection {
                target: "tableau",
                source,
            }
        })?
    else {
        return Ok(None);
    };

    validate_calibrated_halo_y(regions, "tableau", hit.anchor)?;
    Ok(Some(hit.anchor))
}

fn validate_calibrated_halo_y(
    regions: CardRegionPixels,
    target: &'static str,
    anchor: PixelPoint,
) -> Result<(), TrackerError> {
    let expected_y = regions
        .card_bounds
        .y
        .checked_add(regions.card_bounds.height)
        .and_then(|bottom| bottom.checked_add(HALO_GOLD_LINE_OFFSET_Y))
        .and_then(|y| i32::try_from(y).ok())
        .ok_or(TrackerError::HaloLineOverflow { target })?;
    if anchor.y != expected_y {
        return Err(TrackerError::UnexpectedHaloLine {
            target,
            actual_y: anchor.y,
            expected_y,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        capture::PixelFormat,
        parameters::{
            CLICK_OFFSET_X, GAMEPLAY_FELT_PROBE_BOUNDS, GOLD_RGB_CANDIDATES, HALO_GOLD_RUN_MIN,
            NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, STOCK_CARD_REGION, STOCK_HALO_SCAN_BOUNDS,
            TABLEAU_CARD_REGIONS, TABLEAU_ROW_SCAN_PROFILES,
        },
    };

    fn blank_frame(width: u32, height: u32) -> CapturedFrame {
        CapturedFrame {
            width,
            height,
            stride: width as usize * 4,
            format: PixelFormat::Rgba8,
            pixels: vec![0; width as usize * height as usize * 4],
            cursor: None,
        }
    }

    fn set_pixel(frame: &mut CapturedFrame, x: u32, y: u32, rgb: [u8; 3]) {
        let offset = y as usize * frame.stride + x as usize * 4;
        frame.pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }

    fn fill_rect(frame: &mut CapturedFrame, bounds: crate::geometry::PixelRect, rgb: [u8; 3]) {
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                set_pixel(frame, x, y, rgb);
            }
        }
    }

    fn set_calibrated_halo_run(
        frame: &mut CapturedFrame,
        regions: CardRegionPixels,
        start_x: u32,
    ) -> PixelPoint {
        let y = regions.card_bounds.bottom() + HALO_GOLD_LINE_OFFSET_Y;
        for x in start_x..start_x + HALO_GOLD_RUN_MIN + 1 {
            set_pixel(frame, x, y, GOLD_RGB_CANDIDATES[0]);
        }
        PixelPoint::new(start_x as i32, y as i32)
    }

    fn set_slot_halo(frame: &mut CapturedFrame, index: usize) -> PixelPoint {
        let regions = TABLEAU_CARD_REGIONS[index];
        let start_x = regions.click_point.x - CLICK_OFFSET_X;
        set_calibrated_halo_run(frame, regions, start_x as u32)
    }

    fn set_relaxed_slot_halo(
        frame: &mut CapturedFrame,
        index: usize,
        x_shift: i32,
        y_shift: i32,
    ) -> PixelPoint {
        let regions = TABLEAU_CARD_REGIONS[index];
        let canonical_x = regions.click_point.x - CLICK_OFFSET_X;
        let canonical_y = regions.card_bounds.bottom() as i32 + HALO_GOLD_LINE_OFFSET_Y as i32;
        let observed_x = (canonical_x + x_shift) as u32;
        let observed_y = (canonical_y + y_shift) as u32;

        for x in observed_x..observed_x + HALO_GOLD_RUN_MIN + 20 {
            set_pixel(frame, x, observed_y, [225, 194, 100]);
        }

        PixelPoint::new(canonical_x, canonical_y)
    }

    fn set_face_up_row(frame: &mut CapturedFrame, row: u8) {
        let bounds = TABLEAU_ROW_SCAN_PROFILES[row as usize - 1].face_probe_bounds;
        let start_x = bounds.x + 5;
        for y in bounds.y..bounds.bottom() {
            for x in start_x..start_x + crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH {
                set_pixel(frame, x, y, [255; 3]);
            }
        }
    }

    fn set_top_row_face_up_card(frame: &mut CapturedFrame, index: usize) {
        assert!(index < TABLEAU_ROW_SCAN_PROFILES[0].card_count);
        let profile = &TABLEAU_ROW_SCAN_PROFILES[0];
        let start_x = TABLEAU_CARD_REGIONS[index].card_bounds.x + 5;
        for y in profile.face_probe_bounds.y..profile.face_probe_bounds.bottom() {
            for x in start_x..start_x + crate::parameters::FACE_UP_WHITE_BLOCK_MIN_WIDTH {
                set_pixel(frame, x, y, [255; 3]);
            }
        }
    }

    fn row_mask(rows: &[u8]) -> RowMask {
        let mut mask = RowMask::EMPTY;
        for row in rows {
            assert!(mask.insert(*row));
        }
        mask
    }

    fn draw_prediction(anchor: PixelPoint) -> PredictedAction {
        PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .bottom_action(0, anchor)
                .unwrap(),
        )
    }

    fn tableau_prediction(index: usize, anchor: PixelPoint) -> PredictedAction {
        PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .tableau_action(index, anchor)
                .unwrap(),
        )
    }

    #[test]
    fn reports_no_highlight_for_a_blank_calibrated_frame() {
        let frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);

        assert_eq!(analyse_frame(&frame), Ok(PredictedAction::NoHighlight));
    }

    #[test]
    fn reports_draw_for_the_stock_halo() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let anchor = set_calibrated_halo_run(&mut frame, STOCK_CARD_REGION, 842);

        assert_eq!(analyse_frame(&frame), Ok(draw_prediction(anchor)));
    }

    #[test]
    fn reports_draw_after_the_stock_pile_moves_left() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let start_x = 760;
        let y = STOCK_HALO_SCAN_BOUNDS.y;
        assert!(start_x >= STOCK_HALO_SCAN_BOUNDS.x);
        assert!(start_x + HALO_GOLD_RUN_MIN <= STOCK_HALO_SCAN_BOUNDS.right());

        for x in start_x..start_x + HALO_GOLD_RUN_MIN + 1 {
            set_pixel(&mut frame, x, y, GOLD_RGB_CANDIDATES[0]);
        }
        let anchor = PixelPoint::new(start_x as i32, y as i32);

        assert_eq!(analyse_frame(&frame), Ok(draw_prediction(anchor)));
    }

    #[test]
    fn does_not_treat_a_gold_run_outside_the_stock_lane_as_draw() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let start_x = STOCK_HALO_SCAN_BOUNDS.right();
        let y = STOCK_HALO_SCAN_BOUNDS.y;
        assert!(start_x + HALO_GOLD_RUN_MIN < NOMINAL_FRAME_WIDTH);

        for x in start_x..start_x + HALO_GOLD_RUN_MIN + 1 {
            set_pixel(&mut frame, x, y, GOLD_RGB_CANDIDATES[0]);
        }

        assert_eq!(analyse_frame(&frame), Ok(PredictedAction::NoHighlight));
    }

    #[test]
    fn accepts_a_green_felt_gameplay_scene() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        fill_rect(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [20, 140, 60]);

        assert_eq!(is_gameplay_scene(&frame), Ok(true));
    }

    #[test]
    fn rejects_a_blue_dialog_scene() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        fill_rect(&mut frame, GAMEPLAY_FELT_PROBE_BOUNDS, [25, 70, 150]);

        assert_eq!(is_gameplay_scene(&frame), Ok(false));
    }

    #[test]
    fn reports_row_four_column_ten_and_its_calibrated_centre() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let regions = TABLEAU_CARD_REGIONS[27];
        let anchor = set_calibrated_halo_run(&mut frame, regions, 1_662);
        assert_eq!(analyse_frame(&frame), Ok(tableau_prediction(27, anchor)));
    }

    #[test]
    fn rejects_a_shifted_exact_halo_anchor_instead_of_shifting_the_click() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let regions = TABLEAU_CARD_REGIONS[27];
        set_calibrated_halo_run(&mut frame, regions, 1_663);

        assert_eq!(
            analyse_frame(&frame),
            Err(TrackerError::ClickOffsetMismatch {
                actual_x: 1_724,
                actual_y: 533,
                expected_x: 1_723,
                expected_y: 533,
            })
        );
    }

    #[test]
    fn reports_row_three_card_from_a_dark_halo_shifted_four_pixels() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        // Row 3, column 7 is the slot observed around the highlighted Ace.
        let index = 15;
        let anchor = set_relaxed_slot_halo(&mut frame, index, -4, -4);

        assert_eq!(
            analyse_frame_with_state(&frame, &TableauScanState::initial()),
            Ok(FrameAnalysis {
                prediction: tableau_prediction(index, anchor),
                observed_rows: Some(row_mask(&[3])),
                top_row_face_up_count: 0,
            })
        );
    }

    #[test]
    fn counts_individual_face_up_cards_on_the_top_row() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let state = TableauScanState::initial();

        assert_eq!(
            analyse_frame_with_state(&frame, &state)
                .unwrap()
                .top_row_face_up_count,
            0
        );

        set_top_row_face_up_card(&mut frame, 0);
        assert_eq!(
            analyse_frame_with_state(&frame, &state)
                .unwrap()
                .top_row_face_up_count,
            1
        );

        set_top_row_face_up_card(&mut frame, 2);
        assert_eq!(
            analyse_frame_with_state(&frame, &state)
                .unwrap()
                .top_row_face_up_count,
            2
        );
    }

    #[test]
    fn reports_multiple_valid_highlights_as_ambiguous() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_calibrated_halo_run(&mut frame, STOCK_CARD_REGION, 842);
        set_calibrated_halo_run(&mut frame, TABLEAU_CARD_REGIONS[27], 1_662);

        assert_eq!(
            analyse_frame(&frame),
            Ok(PredictedAction::Ambiguous { highlight_count: 2 })
        );
    }

    #[test]
    fn priority_policy_selects_the_first_bottom_target_without_scanning_tableau() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let bottom_anchor = set_calibrated_halo_run(&mut frame, STOCK_CARD_REGION, 842);
        set_slot_halo(&mut frame, 18);
        set_slot_halo(&mut frame, 27);
        let mut profile = *GameMode::TriPeaks.profile();
        profile.target_selection = TargetSelectionPolicy::FirstByPriority;

        let analysis = analyse_frame_for_profile(&frame, &TableauScanState::initial(), &profile)
            .expect("priority analysis");

        assert_eq!(analysis.prediction, draw_prediction(bottom_anchor));
    }

    #[test]
    fn priority_policy_selects_the_first_card_in_lower_to_upper_scan_order() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let first_anchor = set_slot_halo(&mut frame, 18);
        set_slot_halo(&mut frame, 27);
        let mut profile = *GameMode::TriPeaks.profile();
        profile.target_selection = TargetSelectionPolicy::FirstByPriority;

        let analysis = analyse_frame_for_profile(&frame, &TableauScanState::initial(), &profile)
            .expect("priority analysis");

        assert_eq!(analysis.prediction, tableau_prediction(18, first_anchor));
    }

    #[test]
    fn row_masks_preserve_gaps_and_derive_valid_bounds() {
        let mask = row_mask(&[2, 4]);
        let row_count = GameMode::TriPeaks.profile().tableau_rows.len() as u8;

        assert_eq!(mask.bits(), 0b1010);
        assert_eq!(mask.row_upper_for(row_count), Some(2));
        assert_eq!(mask.row_lower_for(row_count), Some(4));
        assert!(mask.contains(2));
        assert!(!mask.contains(3));
        assert_eq!(RowMask::from_bits_for(0b1_0000, row_count), None);
        assert_eq!(TableauScanState::from_active_rows(RowMask::EMPTY), None);
    }

    #[test]
    fn finds_an_upper_row_after_reconciling_white_bands() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 2);
        let anchor = set_slot_halo(&mut frame, 5);

        assert_eq!(
            analyse_frame_with_state(&frame, &TableauScanState::initial()),
            Ok(FrameAnalysis {
                prediction: tableau_prediction(5, anchor),
                observed_rows: Some(row_mask(&[2])),
                top_row_face_up_count: 0,
            })
        );
    }

    #[test]
    fn white_reconciliation_keeps_non_contiguous_active_rows() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 2);
        set_face_up_row(&mut frame, 4);
        let anchor = set_slot_halo(&mut frame, 3);

        let analysis = analyse_frame_with_state(&frame, &TableauScanState::initial()).unwrap();
        assert_eq!(analysis.prediction, tableau_prediction(3, anchor));
        let observed = analysis.observed_rows.unwrap();
        assert_eq!(observed, row_mask(&[2, 4]));
        let row_count = GameMode::TriPeaks.profile().tableau_rows.len() as u8;
        assert_eq!(observed.row_upper_for(row_count), Some(2));
        assert_eq!(observed.row_lower_for(row_count), Some(4));
        assert!(!observed.contains(3));
    }

    #[test]
    fn stale_row_one_state_recovers_a_row_four_highlight() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 4);
        let anchor = set_slot_halo(&mut frame, 27);
        let state = TableauScanState::from_active_rows(row_mask(&[1])).unwrap();

        let analysis = analyse_frame_with_state(&frame, &state).unwrap();
        assert_eq!(analysis.prediction, tableau_prediction(27, anchor));
        assert_eq!(analysis.observed_rows, Some(row_mask(&[4])));
    }

    #[test]
    fn fallback_halo_recovers_stale_state_without_white_evidence() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let anchor = set_slot_halo(&mut frame, 18);
        let state = TableauScanState::from_active_rows(row_mask(&[1])).unwrap();

        let analysis = analyse_frame_with_state(&frame, &state).unwrap();
        assert_eq!(analysis.prediction, tableau_prediction(18, anchor));
        assert_eq!(analysis.observed_rows, Some(row_mask(&[4])));
    }

    #[test]
    fn retires_an_empty_lower_row_when_an_upper_active_row_has_the_halo() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 3);
        let anchor = set_slot_halo(&mut frame, 9);
        let mut state = TableauScanState::from_active_rows(row_mask(&[3, 4])).unwrap();

        let observation = analyse_frame_with_state(&frame, &state).unwrap();
        assert_eq!(observation.prediction, tableau_prediction(9, anchor));
        assert_eq!(observation.observed_rows, Some(row_mask(&[3])));

        assert!(state.reconcile_observation(observation.observed_rows));
        assert_eq!(state.row_upper(), 3);
        assert_eq!(state.row_lower(), 3);
    }

    #[test]
    fn full_fallback_preserves_cross_row_ambiguity() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 2);
        set_slot_halo(&mut frame, 3);
        set_slot_halo(&mut frame, 18);

        let analysis = analyse_frame_with_state(&frame, &TableauScanState::initial()).unwrap();
        assert_eq!(
            analysis.prediction,
            PredictedAction::Ambiguous { highlight_count: 2 }
        );
        assert_eq!(analysis.observed_rows, Some(row_mask(&[2, 4])));
    }

    #[test]
    fn relaxed_fallback_preserves_cross_row_ambiguity() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 3);
        set_relaxed_slot_halo(&mut frame, 9, -4, -4);
        set_relaxed_slot_halo(&mut frame, 18, 4, 4);

        let analysis = analyse_frame_with_state(&frame, &TableauScanState::initial()).unwrap();
        assert_eq!(
            analysis.prediction,
            PredictedAction::Ambiguous { highlight_count: 2 }
        );
        assert_eq!(analysis.observed_rows, Some(row_mask(&[3, 4])));
    }

    #[test]
    fn stock_and_two_tableau_halos_report_three_way_ambiguity() {
        let mut frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        set_face_up_row(&mut frame, 2);
        set_calibrated_halo_run(&mut frame, STOCK_CARD_REGION, 842);
        set_slot_halo(&mut frame, 3);
        set_slot_halo(&mut frame, 18);

        let analysis = analyse_frame_with_state(&frame, &TableauScanState::initial()).unwrap();
        assert_eq!(
            analysis.prediction,
            PredictedAction::Ambiguous { highlight_count: 3 }
        );
        assert_eq!(analysis.observed_rows, Some(row_mask(&[2, 4])));
    }

    #[test]
    fn observed_mask_transitions_change_state_but_empty_evidence_does_not() {
        let mut state = TableauScanState::initial();
        assert_eq!(state.row_upper(), 4);
        assert_eq!(state.row_lower(), 4);

        let split = row_mask(&[2, 4]);
        assert!(state.reconcile_observation(Some(split)));
        assert_eq!(state.active_rows(), split);
        assert_eq!(state.row_upper(), 2);
        assert_eq!(state.row_lower(), 4);

        assert!(!state.reconcile_observation(None));
        assert_eq!(state.active_rows(), split);

        let narrowed = row_mask(&[1]);
        assert!(state.reconcile_observation(Some(narrowed)));
        assert_eq!(state.active_rows(), narrowed);
        assert_eq!((state.row_upper(), state.row_lower()), (1, 1));
    }

    #[test]
    fn single_observation_updates_the_non_authoritative_row_hint() {
        let mut state = TableauScanState::initial();
        let split = row_mask(&[2, 4]);

        assert!(state.reconcile_observation(Some(split)));
        assert_eq!(state.active_rows(), split);
        assert!(!state.reconcile_observation(Some(split)));
        assert!(!state.reconcile_observation(None));
    }

    #[test]
    fn blank_frame_cannot_collapse_a_multi_row_state() {
        let frame = blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT);
        let split = row_mask(&[2, 4]);
        let state = TableauScanState::from_active_rows(split).unwrap();

        let analysis = analyse_frame_with_state(&frame, &state).unwrap();
        assert_eq!(analysis.prediction, PredictedAction::NoHighlight);
        assert_eq!(analysis.observed_rows, None);
    }

    #[test]
    fn rejects_a_frame_with_the_wrong_dimensions_before_detection() {
        let frame = blank_frame(NOMINAL_FRAME_WIDTH - 1, NOMINAL_FRAME_HEIGHT);

        assert_eq!(
            analyse_frame(&frame),
            Err(TrackerError::UnexpectedFrameDimensions {
                expected_width: NOMINAL_FRAME_WIDTH,
                expected_height: NOMINAL_FRAME_HEIGHT,
                actual_width: NOMINAL_FRAME_WIDTH - 1,
                actual_height: NOMINAL_FRAME_HEIGHT,
            })
        );
    }
}
