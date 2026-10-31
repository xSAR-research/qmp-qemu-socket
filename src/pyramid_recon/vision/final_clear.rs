//! Explicit final-only clear evidence for the retained native Perfect overlay.
//!
//! A replayed empty tableau, all 28 complete felt bands, matching piles and a
//! coupled marker or explicitly bound actual challenge count are required together.
//! Each opt-in retains its own measured band geometry and requirement checks.
//! Neither recognises challenge completion or grants input/restart authority.
//! Ordinary position and acquisition checks retain their original evidence.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Three separated full-colour letter regions, each below the existing 4096-pixel cap.
const MARKER_BOUNDS: [PixelRect; 3] = [
    PixelRect::new(716, 468, 56, 64),
    PixelRect::new(929, 453, 48, 80),
    PixelRect::new(1165, 468, 42, 90),
];


/// Original DRAW scene role; only coupled positive empty-LEFT evidence replaces it.
const DRAW_SCENE: PixelRect = PixelRect::new(931, 695, 60, 50);


/// Independently measured empty-LEFT circle in both retained final-clear frames.
const FINAL_LEFT: PixelRect = PixelRect::new(767, 712, 60, 60);


/// Native signature channel tolerance, unchanged from the retained scene calibration.
const MAX_FINAL_TOLERANCE: u8 = 4;


/// At least four fifths of each fixed marker region survive the measured glow mask.
const MASK_FRACTION: (usize, usize) = (4, 5);


/// Per-slot positive felt band: x offset, y offset, width and height in native pixels.
const TOP_BAND: (u32, u32, u32, u32) = (40, 4, 60, 4);


/// Final-only native evidence; never part of ordinary board or input admission.
#[derive(Clone, Debug)]
pub(super) struct FinalClearProfile {
    /// All three measured letter patches must agree in the same fresh frame.
    marker: Vec<Pattern>,
    /// Full-region positive empty-LEFT circle, not inferred from missing card pixels.
    left_empty: Pattern,
}


impl FinalClearProfile {
    /// Validate fixed native roles, bounded masks and unchanged scene/pile geometry.
    ///
    /// Marker masks retain at least four fifths of each region and tolerance at most
    /// four per channel. The full empty-circle patch retains the same bound. Existing
    /// parser limits, contrast, storage and negative-space rules remain in force.
    pub(super) fn parse(value: &Value, scene: &[Pattern], left: &Pattern) -> Result<Self, String> {
        let fields = object(value, &["marker", "left_empty"])?;
        require(
            scene
                .iter()
                .filter(|pattern| pattern.bounds == DRAW_SCENE)
                .count()
                == 1
                && left.bounds == FINAL_LEFT,
            "final-clear calibration requires the original DRAW and LEFT roles",
        )?;
        let entries = fields["marker"]
            .as_array()
            .ok_or("final-clear marker must be an array")?;
        require(
            entries.len() == MARKER_BOUNDS.len(),
            "final-clear requires three marker patches",
        )?;
        let mut marker = Vec::with_capacity(MARKER_BOUNDS.len());
        for (entry, bounds) in entries.iter().zip(MARKER_BOUNDS) {
            let pattern = Pattern::patch(entry, bounds)?;
            require(
                pattern.bounds == bounds
                    && pattern.tolerance <= MAX_FINAL_TOLERANCE
                    && pattern
                        .pixels
                        .iter()
                        .filter(|pixel| pixel.is_some())
                        .count()
                        * MASK_FRACTION.1
                        >= pattern.pixels.len() * MASK_FRACTION.0,
                "final-clear marker requires fixed bounds, tolerance<=4 and at least80% mask",
            )?;
            marker.push(pattern);
        }
        let left_empty = Pattern::patch(&fields["left_empty"], FINAL_LEFT)?;
        require(
            left_empty.bounds == FINAL_LEFT
                && left_empty.tolerance <= MAX_FINAL_TOLERANCE
                && left_empty.pixels.iter().all(Option::is_some),
            "final-clear LEFT requires its full measured region and tolerance<=4",
        )?;
        Ok(Self { marker, left_empty })
    }
}


/// Select only the final geometry admitted by the corresponding evidence gate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FinalPositionProof {
    /// Legacy complete top bands coupled to the measured Perfect letters.
    PerfectMarkers,
    /// Intermediate bound Boards count, retaining the legacy complete top bands.
    IntermediateBoardCount,
    /// Actual bound target, using unobscured lower edges for the last seven slots.
    TargetBoardCount,
    /// Bound actual challenge count using a separately measured lower row band.
    ChallengeCount,
    /// Bundled native alternative: every complete top band plus the same bound count.
    ChallengeTop,
}


impl ReconVision {
    /// Prove a replayed final empty tableau from one native post-ACK observation.
    ///
    /// The caller must establish the final whole-action ACK boundary. All 28 top
    /// bands must contain 240/240 pixels satisfying the unchanged felt predicate;
    /// this alternative is invalid for any remaining card. The optional empty-circle
    /// variant replaces DRAW only together with expected and recognised empty LEFT.
    /// Other scene anchors, exact pile faces, modal veto and bounded marker all apply.
    /// Returns an error for missing calibration, unsupported/ambiguous evidence,
    /// invalid frame storage or any mismatch. No frame, input or authority is retained.
    pub(crate) fn verify_final_clear(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<(), String> {
        self.verify_final_position(
            frame,
            expected,
            left,
            right,
            FinalPositionProof::PerfectMarkers,
        )
    }


    /// Prove only an intermediate Boards challenge clear after the final whole-action ACK.
    ///
    /// The immutable original matched requirement must be Boards, with one actual
    /// displayed increment and the same still-incomplete target. That coupled proof
    /// replaces the moving Perfect letters only; all 28 top bands, exact piles,
    /// original scene anchors and modal veto remain mandatory. The caller retains
    /// STOP/context/header guards and separately witnesses a fresh starting board.
    /// Returns an error for missing opt-in, unknown/changed/completed requirements
    /// or any position mismatch. This never proves target completion or input authority.
    pub(crate) fn verify_board_count_final_clear(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        initial: ChallengeProgress,
    ) -> Result<(), String> {
        require(
            self.board.board_count_final_clear,
            "no board-count final-clear calibration",
        )?;
        let current = crate::pyramid_progress::observe_bound_challenge_progress(frame, initial)?;
        require(
            initial.kind == ChallengeKind::Boards
                && !initial.target_reached()
                && current.is_some_and(|current| {
                    current.follows(initial)
                        && initial.current.checked_add(1) == Some(current.current)
                        && !current.target_reached()
                }),
            "board-count final-clear requires exactly one bound intermediate Boards increment",
        )?;
        self.verify_final_position(
            frame,
            expected,
            left,
            right,
            FinalPositionProof::IntermediateBoardCount,
        )
    }


    /// Prove only the final Boards clear at its actual bound target after whole-action ACK.
    ///
    /// Explicit target calibration, an initial incomplete Boards requirement and
    /// exactly one observed increment to that same target are mandatory. The
    /// 28 complete positive bands, original scene/modal and exact-pile checks
    /// prove the card effect only. The bottom seven bands use the existing normal
    /// final-clear lower-edge geometry, measured unobscured in this native frame.
    /// A separate completion event remains required.
    /// Returns an error for absent calibration, wrong counters or any position mismatch.
    pub(crate) fn verify_board_count_target_clear(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        initial: ChallengeProgress,
    ) -> Result<(), String> {
        require(
            self.board.board_count_target_clear,
            "no board-count target-clear calibration",
        )?;
        let current = crate::pyramid_progress::observe_bound_challenge_progress(frame, initial)?;
        require(
            initial.kind == ChallengeKind::Boards
                && !initial.target_reached()
                && current.is_some_and(|current| {
                    current.follows(initial)
                        && initial.current.checked_add(1) == Some(current.current)
                        && current.target_reached()
                }),
            "board-count target-clear requires exactly one bound increment to the actual target",
        )?;
        self.verify_final_position(
            frame,
            expected,
            left,
            right,
            FinalPositionProof::TargetBoardCount,
        )
    }


    /// Prove only a fully replayed clear under its immutable actual challenge requirement.
    ///
    /// Requires explicit v21 calibration, an initially incomplete requirement and
    /// a current reading of the same kind/target without regression. Boards must
    /// advance exactly once; Score and rank counts need not reach their target.
    /// All original scene/modal/pile checks and 28 full felt bands still apply.
    /// This proves the card effect only, never a terminal, transition or input authority.
    /// Returns an error for any missing binding, calibration or position evidence.
    pub(crate) fn verify_challenge_final_clear(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        initial: ChallengeProgress,
    ) -> Result<(), String> {
        require(
            self.board.challenge_final_clear,
            "no bound challenge final-clear calibration",
        )?;
        let current = crate::pyramid_progress::observe_bound_challenge_progress(frame, initial)?;
        require(
            !initial.target_reached()
                && current.is_some_and(|current| {
                    current.follows(initial)
                        && (initial.kind != ChallengeKind::Boards
                            || initial.current.checked_add(1) == Some(current.current))
                }),
            "challenge final-clear requires the same actual monotonic incomplete binding",
        )?;
        self.verify_final_position(
            frame,
            expected,
            left,
            right,
            FinalPositionProof::ChallengeCount,
        )
        .or_else(|lower_error| {
            require(
                self.native_execution.is_some(),
                "no native challenge top-band evidence",
            )?;
            self.verify_final_position(
                frame,
                expected,
                left,
                right,
                FinalPositionProof::ChallengeTop,
            )
            .map_err(|top_error| format!("{lower_error}; native top-band proof: {top_error}"))
        })
    }


    /// Require complete positive bands using only the geometry admitted by the evidence gate.
    ///
    /// Only the measured-marker entry or an explicitly bound challenge entry above
    /// calls this helper. Returns an error for any incomplete replay, malformed
    /// frame, scene/modal, absence-band, marker or exclusive-pile mismatch.
    fn verify_final_position(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        proof: FinalPositionProof,
    ) -> Result<(), String> {
        require(
            expected.iter().all(Option::is_none),
            "final-clear requires replayed empty tableau",
        )?;
        require(
            frame.width == NATIVE_BOUNDS.width && frame.height == NATIVE_BOUNDS.height,
            "final-clear requires native1920x1080",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "invalid final-clear frame layout".to_owned())?;
        let calibration = self
            .board
            .final_clear
            .as_ref()
            .ok_or("no final-clear calibration")?;
        require(
            !self.dialog_matches(view, true)?,
            "final-clear refused complete Undo All dialog",
        )?;
        if proof == FinalPositionProof::PerfectMarkers {
            require(
                calibration
                    .marker
                    .iter()
                    .try_fold(true, |matched, pattern| {
                        Ok::<_, String>(matched && self.native_marker(view, pattern)?)
                    })?,
                "final-clear marker mismatch",
            )?;
        }
        let circle = left.is_none()
            && self.native_stock_layout(view, LEFT_BOUNDS, &calibration.left_empty)?;
        // A positive exhausted LEFT-X composes with the final bands below. Ordinary
        // tableau occupancy can be obscured by Perfect and is not a second final
        // proof. Exact RIGHT, scene/modal and every selected absence band still apply.
        let exhausted = left.is_none() && self.native_exhausted_left(view)?;
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if (circle || exhausted) && pattern.bounds == DRAW_SCENE {
                continue;
            }
            require(
                self.native_scene_anchor(view, index, pattern)?,
                "final-clear scene anchor mismatch",
            )?;
        }
        if self.temporal_removed.is_some() {
            self.verify_temporal_clear(view)?;
        }
        for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
            if self.temporal_removed.is_some() {
                continue;
            }
            // Each explicit proof selects its measured unobscured band. Every
            // variant still requires all 240 pixels and leaves old geometry intact.
            let offset = match (proof, index >= 21) {
                (FinalPositionProof::ChallengeCount, true) => 168,
                (FinalPositionProof::TargetBoardCount, true) => 164,
                _ => TOP_BAND.1,
            };
            let band = PixelRect::new(x + TOP_BAND.0, y + offset, TOP_BAND.2, TOP_BAND.3);
            view.checked_bounds(band)
                .map_err(|_| "final-clear band outside frame".to_owned())?;
            require(
                self.native_absence_band(view, band)?,
                &format!("final-clear slot {index} lacks complete absence band"),
            )?;
        }
        let face = |card: Option<Card>| card.map_or(StockFace::Empty, StockFace::Card);
        let left_empty = if circle {
            &calibration.left_empty
        } else {
            &self.board.left_empty
        };
        require(
            exhausted || self.stock_face(view, LEFT_BOUNDS, left_empty)? == face(left),
            "final-clear LEFT mismatch",
        )?;
        require(
            self.stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)? == face(right),
            "final-clear RIGHT mismatch",
        )
    }
}
