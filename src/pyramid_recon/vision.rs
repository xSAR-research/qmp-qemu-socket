//! Bounded, read-only native Pyramid recognition from an explicit calibration.
//!
//! The bundled profile retains measured native RGB evidence; custom files use the
//! same bounded parser. Neither source grants input authority. Solver appearance
//! is excluded: the caller owns explicit operator consent, freshness, cancellation,
//! context, inventory, restoration and publication.

use std::{collections::BTreeSet, fs::File, io::Read, path::Path};

use serde_json::{Map, Value};
use xsar::{
    games::cards::{Card, Rank, Suit},
    image_matching::FrameView,
};

use crate::{
    capture::CapturedFrame,
    geometry::{PixelPoint, PixelRect},
};


mod card_paper;
mod foreground_card;
mod graded_glyph;


#[cfg(test)]
mod execution_observability_tests;
mod final_clear;
mod native_colour;
mod native_execution;

#[cfg(test)]
mod native_colour_tests;
mod normal_clear;
mod redeal;
pub(crate) mod removal;
pub(crate) mod source_transition;


mod starting;


pub(crate) use starting::StartingReadiness;


#[cfg(test)]
mod native_v17_tests;


#[cfg(test)]
mod native_v18_tests;


#[cfg(test)]
mod native_v19_tests;


#[cfg(test)]
mod native_v20_tests;


#[cfg(test)]
mod native_v21_tests;


#[cfg(test)]
mod native_v22_tests;




/// Maximum encoded calibration size, before any JSON tree is allocated.
const MAX_PROFILE_BYTES: usize = 2 * 1024 * 1024;


/// Maximum pixels in any one template or signature patch.
const MAX_PATCH_PIXELS: usize = 4096;


/// Minimum selected RGB samples in each nonempty discriminating mask.
const MIN_SELECTED_PIXELS: usize = 32;


/// Maximum per-channel RGB tolerance; no mismatching selected pixels are allowed.
const MAX_TOLERANCE: u8 = 24;


/// Native geometry supported by this profile schema.
const NATIVE_BOUNDS: PixelRect = PixelRect::new(0, 0, 1920, 1080);


/// Original LEFT face, independent of the legacy controller's target ordering.
const LEFT_BOUNDS: PixelRect = PixelRect::new(759, 678, 139, 187);


/// Original RIGHT face, independent of the legacy controller's target ordering.
const RIGHT_BOUNDS: PixelRect = PixelRect::new(1022, 678, 139, 187);


/// Sample the effect quarter in the selected authoritative pile, independently of glyphs.
pub(crate) fn stock_corner(frame: &CapturedFrame, right: bool) -> Result<Vec<u32>, String> {
    if frame.width != NATIVE_BOUNDS.width || frame.height != NATIVE_BOUNDS.height {
        return Err("Stock effect requires native dimensions".to_owned());
    }
    let bounds = if right { RIGHT_BOUNDS } else { LEFT_BOUNDS };
    crate::pyramid_effect::card_pixels(frame, (bounds.x, bounds.y))
}


/// Measure the effect quarter without changing the independent glyph-reader corner.
#[cfg(test)]
pub(crate) fn card_corner(frame: &CapturedFrame, origin: (u32, u32)) -> Result<Vec<u32>, String> {
    if frame.width != NATIVE_BOUNDS.width || frame.height != NATIVE_BOUNDS.height {
        return Err("Card effect requires native dimensions".to_owned());
    }
    crate::pyramid_effect::card_pixels(frame, origin)
}


/// Positive REDEAL artwork corridor used only by an explicitly declared execution variant.
const EXECUTION_EMPTY_STOCK_SCENE_BOUNDS: PixelRect = PixelRect::new(920, 678, 80, 187);


/// Original dynamic stock-arrow scene role replaced only by a complete coupled proof.
const EXECUTION_EMPTY_STOCK_SCENE_INDEX: usize = 2;


/// Legacy schema-1 Solver metadata bounds; validated but never matched against frames.
const SOLVER_BOUNDS: PixelRect = PixelRect::new(585, 959, 35, 36);


/// Positive Undo All evidence and its fixed native click target.
const UNDO_BOUNDS: PixelRect = PixelRect::new(1301, 959, 38, 38);


/// Logical top-to-bottom, left-to-right card origins; not CV target indices.
const TABLEAU_ORIGINS: [(u32, u32); 28] = [
    (890, 112),
    (790, 165),
    (990, 165),
    (690, 218),
    (890, 218),
    (1091, 218),
    (589, 272),
    (790, 272),
    (990, 272),
    (1191, 272),
    (489, 325),
    (690, 325),
    (890, 325),
    (1091, 325),
    (1291, 325),
    (389, 378),
    (589, 378),
    (790, 378),
    (990, 378),
    (1191, 378),
    (1392, 378),
    (288, 432),
    (489, 432),
    (690, 432),
    (890, 432),
    (1091, 432),
    (1291, 432),
    (1492, 432),
];


/// A completely classified supported scene, or no supported signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconScene {
    /// All tableau/pile identities and toolbar evidence agree in this frame.
    Board(BoardObservation),
    /// All calibrated Undo All dialog components agree in this frame.
    UndoConfirmation {
        /// Native point contained in the positively matched OK patch.
        ok: PixelPoint,
    },
    /// Neither complete board nor complete dialog signature matches.
    Unknown,
}


/// One complete supported board observation, independent of Solver availability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardObservation {
    /// Actual recognised rank and suit, in logical top-to-bottom row order.
    pub tableau: [Card; 28],
    /// Recognised LEFT card or positively matched empty-LEFT appearance.
    pub left: StockFace,
    /// Recognised RIGHT card or positively matched empty-RIGHT appearance.
    pub right: StockFace,
    /// Fixed native point only when this frame positively matches enabled Undo All.
    pub undo_all: Option<PixelPoint>,
}


/// A pile is never declared empty merely because card recognition failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockFace {
    /// One uniquely matched, explicitly labelled rank-and-suit identity.
    Card(Card),
    /// Positive empty-pile evidence with no matching card template.
    Empty,
}


/// Validated private calibration; construction requires a complete strict profile.
#[derive(Clone, Debug)]
pub struct ReconVision {
    /// Bounded operator-supplied calibration identifier, not authenticated provenance.
    profile_id: String,
    /// Complete factorised or identity reader shared by all exposed tableau strips.
    tableau_reader: CardReader,
    /// Complete factorised or identity reader shared by LEFT and RIGHT native faces.
    stock_reader: CardReader,
    /// Complete supported board signature, independent of Solver and Undo availability.
    board: BoardProfile,
    /// Specific Undo All dialog components; generic OK/gold is insufficient.
    dialog: DialogProfile,
    /// Non-serialised continuation evidence installed only by the bundled native constructor.
    native_redeal: Option<redeal::NativeRedeal>,
    /// Private measured execution alternatives; only the bundled constructor opts in.
    native_execution: Option<native_execution::NativeExecution>,
    /// Private appearance policy; serialized/custom profiles cannot enable it.
    colour_independent: bool,
    /// Route-owned temporal absence; never installed by serialised calibration.
    temporal_removed: Option<removal::RemovalProof>,
}


/// A half-open native or card-relative mask with bounded decoded RGB samples.
#[derive(Clone, Debug)]
struct Pattern {
    /// Absolute bounds for scene patches, relative bounds for card templates.
    bounds: PixelRect,
    /// Row-major RGB constraints; `None` pixels are explicitly excluded.
    pixels: Vec<Option<[u8; 3]>>,
    /// Inclusive absolute difference accepted independently for every channel.
    tolerance: u8,
}


/// One complete factorised glyph reader or whole-identity bank for a card setting.
#[derive(Clone, Debug)]
struct CardReader {
    /// Private native corner capability; no serialized profile can enable it.
    foreground: Option<foreground_card::Reader>,
    /// Labelled, pairwise discriminating rank-and-suit templates.
    templates: Vec<(Card, Pattern)>,
    /// Independent rank/suit banks when real evidence uses factorised typography.
    glyphs: Option<GlyphReader>,
}


/// Independent native glyph labels; no whole-card pixels are composed or invented.
#[derive(Clone, Debug)]
struct GlyphReader {
    /// All thirteen rank labels, each with one to eight measured variants.
    ranks: Vec<(Rank, Vec<Pattern>)>,
    /// All four suit labels, each with one to sixteen measured variants.
    suits: Vec<(Suit, Vec<Pattern>)>,
    /// Explicit versioned comparison semantics; legacy profiles retain bounded XOR.
    policy: GlyphPolicy,
    /// Zero or one native pixel of integer alignment in each direction.
    alignment: i32,
}


/// Mutually exclusive comparison contracts selected by the profile schema.
#[derive(Clone, Copy, Debug)]
enum GlyphPolicy {
    /// Original schema-1 occupancy comparison with unchanged inclusive bounds.
    Xor {
        /// Maximum rank occupancy disagreements, at most 24 pixels.
        rank_max_distance: usize,
        /// Maximum suit occupancy disagreements, at most eight pixels.
        suit_max_distance: usize,
    },
    /// Schema-2 full-region graded ink comparison with fixed quality and margin bounds.
    Graded(graded_glyph::Policy),
}


/// Native evidence required together before any card observation is returned.
#[derive(Clone, Debug)]
struct BoardProfile {
    /// Three to eight disjoint, discriminating supported-scene patches.
    scene: Vec<Pattern>,
    /// Positive enabled Undo All icon evidence, independent of board recognition.
    undo_all: Pattern,
    /// Native positive empty-LEFT appearance, including its actual decoration.
    left_empty: Pattern,
    /// Native positive empty-RIGHT appearance.
    right_empty: Pattern,
    /// Optional execution-only coupled REDEAL and empty-LEFT evidence.
    execution_empty_stock: Option<ExecutionEmptyStock>,
    /// Explicit final-only Perfect marker and empty-circle evidence.
    final_clear: Option<final_clear::FinalClearProfile>,
    /// Explicit final-only normal header, unobscured absence and exact-pile evidence.
    normal_final_clear: Option<normal_clear::NormalClearProfile>,
    /// Explicit intermediate Boards-count advance coupled to exact final position proof.
    board_count_final_clear: bool,
    /// Explicit final-target Boards increment coupled to the same exact position proof.
    board_count_target_clear: bool,
    /// Explicit bound challenge clear with unobscured native absence bands.
    challenge_final_clear: bool,
}


/// A single measured stock-exhausted variant; its components never match independently.
#[derive(Clone, Debug)]
struct ExecutionEmptyStock {
    /// Positive central REDEAL icon, replacing the original arrow scene role.
    scene: Pattern,
    /// Positive empty-LEFT decoration measured in the same retained native state.
    left_empty: Pattern,
}


/// One measured dialog, including content, controls and dimmed native backdrop.
#[derive(Clone, Debug)]
struct DialogProfile {
    /// Panel, title, body, Cancel, OK and two separated backdrop patches.
    signature: Vec<Pattern>,
    /// Validated point within the OK patch; never inferred from gold pixels.
    ok: PixelPoint,
    /// Explicit post-confirmation appearance; normal observation never uses it.
    post_confirm_ok: Option<Pattern>,
}


impl ReconVision {
    /// Load native-v22's single measured diamond appearance without changing v21 guards.
    ///
    /// Both factorised banks retain their original bounds, colour, paper, ambiguity
    /// and shape limits. This calibration grants no input or challenge authority.
    /// Returns an error if strict embedded calibration validation fails.
    pub(crate) fn bundled_native_v22() -> Result<Self, String> {
        let mut vision = Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v22/profile.json"
        ))?;
        vision.native_redeal = Some(redeal::NativeRedeal::bundled()?);
        vision.native_execution = Some(native_execution::NativeExecution::bundled()?);
        Ok(vision)
    }


    /// Load native-v21's bound challenge clear while retaining every v20 sample.
    ///
    /// The caller still proves final whole-action ACK, actual bound requirement,
    /// fresh exact position and a separate challenge completion before success.
    /// Returns an error if strict embedded calibration validation fails.
    #[cfg(test)]
    pub(crate) fn bundled_native_v21() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v21/profile.json"
        ))
    }


    /// Load the retained v20 target-only proof for explicit regression comparisons.
    ///
    /// Returns an error for invalid embedded calibration; grants no runtime authority.
    #[cfg(test)]
    pub(crate) fn bundled_native_v20() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v20/profile.json"
        ))
    }


    /// Load native-v19's bound intermediate Boards-clear capability without new glyph evidence.
    ///
    /// Parsing grants no freshness, final-ACK, challenge, input or restart authority.
    /// Returns an error if the embedded bounded calibration is inconsistent.
    #[cfg(test)]
    pub(crate) fn bundled_native_v19() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v19/profile.json"
        ))
    }


    /// Load the native-v18 normal final proof with all v17 calibration retained.
    ///
    /// Parsing grants no freshness, input, final-ACK or restart authority.
    /// Returns an error if the embedded bounded calibration is inconsistent.
    #[cfg(test)]
    pub(crate) fn bundled_native_v18() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v18/profile.json"
        ))
    }


    /// Load native-v17 final-clear evidence with unchanged native-v16 card readers.
    ///
    /// Parsing grants no freshness, input, final-ACK or restart authority.
    /// Returns an error if the embedded bounded calibration is inconsistent.
    #[cfg(test)]
    pub(crate) fn bundled_native_v17() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v17/profile.json"
        ))
    }


    /// Load the explicit native-v16 calibration with one measured gold-highlighted Seven.
    ///
    /// Retains native-v15's stock-exhaustion and post-confirmation evidence.
    /// Parsing establishes no capture, planning or guest-input authority.
    ///
    /// # Errors
    /// Returns an error if the embedded bounded calibration is inconsistent.
    #[cfg(test)]
    pub(crate) fn bundled_native_v16() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v16/profile.json"
        ))
    }


    /// Load the measured schema-2 execution-empty and post-confirmation calibration.
    ///
    /// Parsing grants no freshness, restoration, publication or input authority.
    ///
    /// # Errors
    /// Returns an error if the embedded profile violates its strict bounded contract.
    #[cfg(test)]
    pub(crate) fn bundled_native_v15() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v15/profile.json"
        ))
    }


    /// Parse the retained native-v11 calibration embedded in this executable.
    ///
    /// This performs no filesystem access and uses the same strict bounds as custom
    /// files. Embedded provenance is not consent, freshness or input authority.
    ///
    /// # Errors
    /// Returns an error if the embedded calibration violates the shared profile contract.
    #[cfg(test)]
    pub(crate) fn bundled_native_v11() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v11/profile.json"
        ))
    }


    /// Load native-v14 with one measured diamond crop at an allowed pixel offset.
    ///
    /// The complete parser retains the frozen graded, colour and geometry rules.
    /// Embedded calibration grants no acquisition, restoration or input authority.
    ///
    /// # Errors
    /// Returns an error if the embedded profile violates its strict contract.
    #[cfg(test)]
    pub(crate) fn bundled_native_v14() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v14/profile.json"
        ))
    }


    /// Loads the immutable native-v13 profile with measured diamond appearances.
    ///
    /// The strict parser validates the unchanged graded policy and native bounds.
    ///
    /// # Errors
    /// Returns an error if the embedded measured profile violates the strict contract.
    #[cfg(test)]
    pub(crate) fn bundled_native_v13() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v13/profile.json"
        ))
    }


    /// Parse the embedded native-v12 schema-2 graded calibration without I/O.
    ///
    /// The complete strict parser also serves custom files. Recognition grants no
    /// consent, freshness, restoration, publication or guest-input authority.
    ///
    /// # Errors
    /// Returns an error if the embedded calibration violates its versioned contract.
    #[cfg(test)]
    pub(crate) fn bundled_native_v12() -> Result<Self, String> {
        Self::from_bytes(include_bytes!(
            "../../tests/fixtures/pyramid-recon/native-v12/profile.json"
        ))
    }


    /// Load a strict schema-1 or schema-2 CLASSIC BLUE profile from a regular file.
    ///
    /// Reads at most 2 MiB plus one sentinel byte. Errors omit path/profile text.
    /// File checks are not a race-proof boundary and impose no I/O time limit.
    pub fn load(path: &Path) -> Result<Self, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|_| "recon profile is missing or inaccessible".to_owned())?;


        if !metadata.is_file() || metadata.len() > MAX_PROFILE_BYTES as u64 {
            return Err("recon profile must be a regular file of at most 2 MiB".to_owned());
        }

        let file = File::open(path).map_err(|_| "cannot open recon profile".to_owned())?;
        let opened = file
            .metadata()
            .map_err(|_| "cannot inspect recon profile".to_owned())?;


        if !opened.is_file() {
            return Err("opened recon profile is not a regular file".to_owned());
        }

        let mut bytes = Vec::new();
        file.take(MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "cannot read recon profile".to_owned())?;
        Self::from_bytes(&bytes)
    }


    /// Validate byte, structure, schema and calibration bounds before use.
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        require(
            bytes.len() <= MAX_PROFILE_BYTES,
            "recon profile exceeds 2 MiB",
        )?;
        JsonGuard::check(bytes)?;
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|_| "recon profile is not valid JSON".to_owned())?;
        let root = object(
            &value,
            &[
                "schema_version",
                "profile_id",
                "game",
                "artwork",
                "dimensions",
                "tableau_reader",
                "stock_reader",
                "board",
                "dialog",
            ],
        )?;
        let schema = integer(&root["schema_version"])?;
        require(matches!(schema, 1 | 2), "unsupported recon profile version")?;
        require(
            root["game"].as_str() == Some("pyramid"),
            "recon profile game must be pyramid",
        )?;
        require(
            root["artwork"].as_str() == Some("CLASSIC BLUE"),
            "unsupported recon artwork",
        )?;
        let profile_id = string(&root["profile_id"])?;
        require(
            !profile_id.is_empty()
                && profile_id.len() <= 64
                && profile_id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte)),
            "invalid recon profile identifier",
        )?;
        require(
            root["dimensions"] == serde_json::json!([1920, 1080]),
            "recon profile requires 1920x1080",
        )?;

        Ok(Self {
            profile_id: profile_id.to_owned(),
            tableau_reader: CardReader::parse_versioned(&root["tableau_reader"], 140, 53, schema)?,
            stock_reader: CardReader::parse_versioned(&root["stock_reader"], 139, 187, schema)?,
            board: BoardProfile::parse(&root["board"], schema)?,
            dialog: DialogProfile::parse(&root["dialog"], schema)?,
            native_redeal: None,
            native_execution: None,
            colour_independent: false,
            temporal_removed: None,
        })
    }


    /// Return the checked ASCII identifier for bounded diagnostic/provenance logging.
    ///
    /// This identifies an operator-supplied profile; it is not a digest or signature.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }


    /// Recognise one independently checked native frame without I/O or input.
    ///
    /// Alpha and row padding are ignored. Unknown/ambiguous card evidence on a
    /// supported board is an error, never an inferred card or an empty pile.
    pub fn observe(&self, frame: &CapturedFrame) -> Result<ReconScene, String> {
        self.observe_with_confirmation_state(frame, false)
    }


    /// Observe after the caller's sole acknowledged confirmation without granting input.
    ///
    /// The optional measured OK appearance requires all six other dialog components.
    /// The caller must already have acknowledged confirmation; this pure classifier
    /// neither proves restoration nor permits another confirmation or an unknown retry.
    ///
    /// # Errors
    /// Preserves normal frame, card and board/dialog ambiguity errors.
    pub(crate) fn observe_after_confirmation(
        &self,
        frame: &CapturedFrame,
    ) -> Result<ReconScene, String> {
        self.observe_with_confirmation_state(frame, true)
    }


    /// Share normal classification with an explicitly selected post-confirmation modal role.
    fn observe_with_confirmation_state(
        &self,
        frame: &CapturedFrame,
        after_confirmation: bool,
    ) -> Result<ReconScene, String> {
        require(
            frame.width == 1920 && frame.height == 1080,
            "recon requires a native 1920x1080 frame",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "invalid recon frame layout".to_owned())?;
        let board_matches = self.board_scene_matches(view)?;
        let dialog_matches = self.dialog_matches(view, after_confirmation)?;


        match (board_matches, dialog_matches) {
            (true, true) => return Err("ambiguous recon board/dialog signatures".to_owned()),
            (false, true) => return Ok(ReconScene::UndoConfirmation { ok: self.dialog.ok }),
            (false, false) => return Ok(ReconScene::Unknown),
            (true, false) => {}
        }

        let mut tableau = [Card::new(Rank::Ace, Suit::Clubs); 28];


        for (index, origin) in TABLEAU_ORIGINS.into_iter().enumerate() {
            tableau[index] = self
                .tableau_reader
                .recognise(view, origin)
                .map_err(|error| format!("tableau slot {index}: {error}"))?
                .ok_or_else(|| {
                    format!(
                        "unknown rank/suit at tableau slot {index}; accepted identities=0; {}",
                        self.tableau_reader.failure_details(view, origin)
                    )
                })?;
        }

        let left = self
            .stock_face(view, LEFT_BOUNDS, &self.board.left_empty)
            .map_err(|error| format!("LEFT: {error}"))?;
        let right = self
            .stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)
            .map_err(|error| format!("RIGHT: {error}"))?;
        let mut seen = Vec::with_capacity(30);


        for card in tableau
            .into_iter()
            .chain([left, right].into_iter().filter_map(|face| match face {
                StockFace::Card(card) => Some(card),
                StockFace::Empty => None,
            }))
        {
            require(
                !seen.contains(&card),
                "duplicate identity in native board observation",
            )?;
            seen.push(card);
        }

        let undo_all = self
            .native_undo(view)?
            .then_some(PixelPoint::new(1320, 978));
        Ok(ReconScene::Board(BoardObservation {
            tableau,
            left,
            right,
            undo_all,
        }))
    }


    /// Compare a partial position with independently recognised native evidence.
    ///
    /// Remaining cards require exact rank/suit; removed cards require positive lower-face
    /// felt evidence. Unknown recognition is never absence. This adds no input authority.
    ///
    /// # Errors
    /// Returns an error for invalid layouts, unsupported scenes, unknown/ambiguous
    /// evidence or any identity/occupancy/pile mismatch.
    pub(crate) fn verify_position(
        &self,
        frame: &CapturedFrame,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<(), String> {
        require(
            frame.width == 1920 && frame.height == 1080,
            "execution requires native 1920x1080",
        )?;
        let view = frame
            .as_view()
            .map_err(|_| "invalid execution frame layout".to_owned())?;
        // Diagnose a positive modal before its dimmed scene patches can mask it.
        require(
            !self.dialog_matches(view, true)?,
            "execution refused: matched complete Undo All dialog",
        )?;
        let coupled_empty_stock = self.matches_execution_empty_stock(view, left, right)?;
        let native_empty_stock = self.matches_native_empty_stock(view, left, right)?;
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if (coupled_empty_stock || native_empty_stock)
                && index == EXECUTION_EMPTY_STOCK_SCENE_INDEX
            {
                continue;
            }
            if !self.native_scene_anchor(view, index, pattern)? {
                let bounds = pattern.bounds;
                return Err(format!(
                    "execution board scene[{index}] mismatch at [{},{},{},{}]",
                    bounds.x, bounds.y, bounds.width, bounds.height,
                ));
            }
        }
        let left_empty = if coupled_empty_stock {
            &self
                .board
                .execution_empty_stock
                .as_ref()
                .ok_or("missing coupled execution stock evidence")?
                .left_empty
        } else {
            &self.board.left_empty
        };
        self.verify_cards(frame, view, expected, left, right, left_empty)
    }

    /// Verify all original card identities, positive removals and exclusive pile faces.
    ///
    /// Only the caller selects the positive LEFT empty signature; ambiguity, pixel
    /// quality, exact RIGHT identity and every tableau expectation stay unchanged.
    fn verify_cards(
        &self,
        frame: &CapturedFrame,
        view: FrameView<'_>,
        expected: &[Option<Card>; 28],
        left: Option<Card>,
        right: Option<Card>,
        left_empty: &Pattern,
    ) -> Result<(), String> {
        let occupancy = self
            .temporal_removed
            .is_none()
            .then(|| self.native_occupancy(frame))
            .transpose()?;
        for (index, origin) in TABLEAU_ORIGINS.into_iter().enumerate() {
            let expected_card = expected.get(index).ok_or("invalid tableau expectation")?;
            if let Some(card) = expected_card {
                let observed = self.tableau_reader.recognise(view, origin)?;
                if observed != Some(*card) {
                    let graded = self
                        .tableau_reader
                        .glyphs
                        .as_ref()
                        .is_some_and(|reader| matches!(reader.policy, GlyphPolicy::Graded(_)));
                    if graded {
                        return Err(format!(
                            "execution tableau identity mismatch at slot {index}; observed={observed:?}; {}",
                            self.tableau_reader.failure_details(view, origin)
                        ));
                    }
                    return Err(format!(
                        "execution tableau identity mismatch at slot {index}"
                    ));
                }
            } else {
                if let Some(proof) = self.temporal_removed {
                    require(proof.contains(index), &format!("execution slot {index} lacks temporal removal evidence"))?;
                    require(self.tableau_reader.recognise(view, origin)?.is_none(), &format!("execution removed slot {index} contains a recognised card"))?;
                    continue;
                }
                let (row, column) = xsar::games::pyramid::planning::tableau_coordinates(index)
                    .ok_or("invalid execution tableau coordinate")?;
                let kind = crate::pyramid::PyramidTargetKind::Card {
                    row: u8::try_from(row).map_err(|_| "invalid execution row")?,
                    column: u8::try_from(column).map_err(|_| "invalid execution column")?,
                };
                let slot = kind.slot_index().ok_or("invalid execution absence slot")?;
                require(
                    occupancy.as_ref().and_then(|occupancy| occupancy.get(slot)) == Some(&crate::pyramid::FaceEvidence::Absent),
                    &format!("execution slot {index} lacks positive absence"),
                )?;
            }
        }
        let face = |card: Option<Card>| card.map_or(StockFace::Empty, StockFace::Card);
        require(
            self.stock_face(view, LEFT_BOUNDS, left_empty)? == face(left),
            "execution LEFT mismatch",
        )?;
        require(
            self.stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)? == face(right),
            "execution RIGHT mismatch",
        )
    }


    /// Require one complete same-frame alternative solely for replayed empty stock.
    ///
    /// Both expected piles must be empty; positive scene, original anchors and both
    /// actual empty piles must agree. Card/empty ambiguity remains an error.
    fn matches_execution_empty_stock(
        &self,
        view: FrameView<'_>,
        left: Option<Card>,
        right: Option<Card>,
    ) -> Result<bool, String> {
        if left.is_some() || right.is_some() {
            return Ok(false);
        }
        let Some(variant) = &self.board.execution_empty_stock else {
            return Ok(false);
        };
        if !self.native_shape(view, &variant.scene, 1)? {
            return Ok(false);
        }
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if index != EXECUTION_EMPTY_STOCK_SCENE_INDEX
                && !self.native_scene_anchor(view, index, pattern)?
            {
                return Ok(false);
            }
        }
        Ok(
            self.stock_face(view, LEFT_BOUNDS, &variant.left_empty)? == StockFace::Empty
                && self.stock_face(view, RIGHT_BOUNDS, &self.board.right_empty)?
                    == StockFace::Empty,
        )
    }


    /// Require exactly one of positive-empty and unique-card evidence for a pile.
    fn stock_face(
        &self,
        view: FrameView<'_>,
        bounds: PixelRect,
        empty: &Pattern,
    ) -> Result<StockFace, String> {
        let card = self.stock_reader.recognise(view, (bounds.x, bounds.y))?;
        let empty_matches = self.native_empty(view, bounds, empty)?;


        match (card, empty_matches) {
            (Some(card), false) => Ok(StockFace::Card(card)),
            (None, true) => Ok(StockFace::Empty),
            (Some(_), true) => Err(
                "ambiguous card/empty pile evidence; card candidates=1; empty candidates=1"
                    .to_owned(),
            ),
            (None, false) => Err(format!(
                "unknown rank/suit or empty pile evidence; accepted identities=0; empty candidates=0; {}",
                self.stock_reader
                    .failure_details(view, (bounds.x, bounds.y))
            )),
        }
    }
}


impl Pattern {
    /// Parse one bounded RGB mask; card-relative bounds are supplied by the bank.
    fn parse(value: &Value, bounds: PixelRect) -> Result<Self, String> {
        Self::parse_with_contrast(value, bounds, 64)
    }


    /// Dimmed modal backdrop patches have a separately bounded contrast floor.
    fn parse_with_contrast(
        value: &Value,
        bounds: PixelRect,
        minimum_contrast: u8,
    ) -> Result<Self, String> {
        let fields = object(value, &["rgb", "mask", "tolerance", "source_sha256"])?;
        let tolerance = u8::try_from(integer(&fields["tolerance"])?)
            .map_err(|_| "invalid RGB tolerance".to_owned())?;
        require(tolerance <= MAX_TOLERANCE, "RGB tolerance exceeds 24")?;
        let hash = string(&fields["source_sha256"])?;
        require(
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "source_sha256 must contain 64 lower-case hexadecimal digits",
        )?;
        let count = (bounds.width as usize)
            .checked_mul(bounds.height as usize)
            .ok_or_else(|| "patch area overflow".to_owned())?;
        require(
            (MIN_SELECTED_PIXELS..=MAX_PATCH_PIXELS).contains(&count),
            "patch area must be 32..4096 pixels",
        )?;
        let rgb = string(&fields["rgb"])?.as_bytes();
        let mask = string(&fields["mask"])?.as_bytes();
        require(
            rgb.len() == count * 6 && mask.len() == count,
            "RGB/mask length differs from patch area",
        )?;
        let mut pixels = Vec::with_capacity(count);
        let mut selected = 0;
        let mut darkest = u8::MAX;
        let mut brightest = 0;


        for (encoded, enabled) in rgb.as_chunks::<6>().0.iter().zip(mask) {
            let mut colour = [0; 3];


            for (channel, pair) in colour.iter_mut().zip(encoded.as_chunks::<2>().0.iter()) {
                *channel = hex(pair[0])? * 16 + hex(pair[1])?;
            }

            require(
                *enabled == b'0' || *enabled == b'1',
                "mask contains a non-binary character",
            )?;


            if *enabled == b'1' {
                selected += 1;
                let luminance =
                    (u16::from(colour[0]) + u16::from(colour[1]) + u16::from(colour[2])) / 3;
                darkest = darkest.min(luminance as u8);
                brightest = brightest.max(luminance as u8);
                pixels.push(Some(colour));
            } else {
                pixels.push(None);
            }
        }

        require(
            selected >= MIN_SELECTED_PIXELS,
            "mask selects fewer than 32 pixels",
        )?;
        require(
            brightest.saturating_sub(darkest) >= minimum_contrast,
            "mask lacks discriminating RGB contrast",
        )?;
        Ok(Self {
            bounds,
            pixels,
            tolerance,
        })
    }


    /// Parse an absolute signature patch wholly inside its prescribed region.
    fn patch(value: &Value, enclosing: PixelRect) -> Result<Self, String> {
        Self::patch_with_contrast(value, enclosing, 64)
    }


    /// Use reduced contrast only for explicit dimmed-backdrop signature components.
    fn patch_with_contrast(
        value: &Value,
        enclosing: PixelRect,
        minimum_contrast: u8,
    ) -> Result<Self, String> {
        let fields = object(value, &["bounds", "samples"])?;
        let bounds = rectangle(&fields["bounds"])?;
        require(
            contains(enclosing, bounds),
            "signature patch lies outside its allowed native region",
        )?;
        Self::parse_with_contrast(&fields["samples"], bounds, minimum_contrast)
    }


    /// Match every selected pixel after checked coordinate translation and ROI validation.
    fn matches(&self, view: FrameView<'_>, origin: (u32, u32)) -> Result<bool, String> {
        let bounds = PixelRect::new(
            origin
                .0
                .checked_add(self.bounds.x)
                .ok_or_else(|| "patch x overflow".to_owned())?,
            origin
                .1
                .checked_add(self.bounds.y)
                .ok_or_else(|| "patch y overflow".to_owned())?,
            self.bounds.width,
            self.bounds.height,
        );
        view.checked_bounds(bounds)
            .map_err(|_| "recognition patch is outside the frame".to_owned())?;


        for (index, expected) in self.pixels.iter().enumerate() {
            if let Some(expected) = expected {
                let column = index as u32 % bounds.width;
                let row = index as u32 / bounds.width;
                let actual = view
                    .pixel_rgb(bounds.x + column, bounds.y + row)
                    .ok_or_else(|| "recognition pixel is outside the frame".to_owned())?;


                if actual
                    .into_iter()
                    .zip(expected)
                    .any(|(channel, reference)| channel.abs_diff(*reference) > self.tolerance)
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}


impl CardReader {
    /// Summarise versioned deterministic bounds without claiming probability.
    fn policy(&self) -> String {
        if self.foreground.is_some() {
            return foreground_card::Reader::policy().to_owned();
        }
        match self
            .glyphs
            .as_ref()
            .map(|glyphs| (glyphs.policy, glyphs.alignment))
        {
            Some((
                GlyphPolicy::Xor {
                    rank_max_distance,
                    suit_max_distance,
                },
                alignment,
            )) => format!(
                "ink<128; XOR rank<={rank_max_distance}, suit<={suit_max_distance}; alignment={alignment}px; unique labels/paper/colour required"
            ),
            Some((GlyphPolicy::Graded(policy), alignment)) => format!(
                "paper_ink_l1_v2; error ppm rank<={}, suit<={}; different-label gap>={}ppm; alignment={alignment}px; paper/exclusive-colour required",
                policy.max_error_ppm(true),
                policy.max_error_ppm(false),
                policy.min_gap_ppm()
            ),
            None => "unique RGB identity; all selected samples within calibrated channel tolerance"
                .to_owned(),
        }
    }


    /// Explain a refusal from the same immutable pixels; this grants no authority.
    fn failure_details(&self, view: FrameView<'_>, origin: (u32, u32)) -> String {
        if let Some(reader) = &self.foreground {
            return reader.failure_details(view, origin);
        }
        let Some(glyphs) = &self.glyphs else {
            return self.policy();
        };
        let GlyphPolicy::Graded(policy) = glyphs.policy else {
            return self.policy();
        };
        let describe = || -> Result<String, String> {
            let rank = graded_glyph::evaluate(
                &glyphs.ranks,
                view,
                origin,
                policy.max_error_ppm(true),
                policy.min_gap_ppm(),
                glyphs.alignment,
            )?;
            let suit = graded_glyph::evaluate(
                &glyphs.suits,
                view,
                origin,
                policy.max_error_ppm(false),
                policy.min_gap_ppm(),
                glyphs.alignment,
            )?;
            let colour = match (rank.accepted(), suit.accepted()) {
                (Some(rank), Some(suit))
                    if rank.red != suit.red
                        || suit.red != matches!(suit.label, Suit::Diamonds | Suit::Hearts) =>
                {
                    "; rank/suit colour contradiction"
                }
                _ => "",
            };
            Ok(format!(
                "{}; rank: {}; suit: {}{colour}",
                self.policy(),
                rank.describe(Rank::as_str),
                suit.describe(Suit::as_str)
            ))
        };
        match describe() {
            Ok(details) => details,
            Err(error) => format!("{}; diagnostic evaluation failed: {error}", self.policy()),
        }
    }


    /// Dispatch explicitly by profile schema; legacy custom profiles never use graded matching.
    fn parse_versioned(
        value: &Value,
        width: u32,
        height: u32,
        schema: u32,
    ) -> Result<Self, String> {
        if schema == 1 {
            return Self::parse(value, width, height);
        }
        require(
            schema == 2 && value.get("ranks").is_some(),
            "schema-2 requires graded factorised readers",
        )?;
        Ok(Self {
            foreground: None,
            templates: Vec::new(),
            glyphs: Some(GlyphReader::parse_graded(value, width, height)?),
        })
    }


    /// Validate either complete rank/suit labels or all 52 disjoint RGB identities.
    fn parse(value: &Value, width: u32, height: u32) -> Result<Self, String> {
        if value.get("ranks").is_some() {
            return Ok(Self {
                foreground: None,
                templates: Vec::new(),
                glyphs: Some(GlyphReader::parse(value, width, height)?),
            });
        }

        let fields = object(value, &["roi", "templates"])?;
        let roi = rectangle(&fields["roi"])?;
        require(
            contains(PixelRect::new(0, 0, width, height), roi),
            "card ROI exceeds its visible native face",
        )?;
        let entries = fields["templates"]
            .as_array()
            .ok_or_else(|| "templates must be an array".to_owned())?;
        require(
            entries.len() == 52,
            "card reader requires exactly 52 labelled identities",
        )?;
        let mut templates: Vec<(Card, Pattern)> = Vec::with_capacity(52);


        for entry in entries {
            let fields = object(entry, &["rank", "suit", "samples"])?;
            let rank = Rank::ALL
                .into_iter()
                .find(|rank| Some(rank.as_str()) == fields["rank"].as_str())
                .ok_or_else(|| "invalid template rank".to_owned())?;
            let suit = Suit::ALL
                .into_iter()
                .find(|suit| Some(suit.as_str()) == fields["suit"].as_str())
                .ok_or_else(|| "invalid template suit".to_owned())?;
            let card = Card::new(rank, suit);
            require(
                !templates.iter().any(|(known, _)| *known == card),
                "duplicate template identity",
            )?;
            let pattern = Pattern::parse(&fields["samples"], roi)?;
            let paper = pattern
                .pixels
                .iter()
                .flatten()
                .filter(|rgb| rgb.iter().all(|channel| *channel >= 180))
                .count();
            let ink = pattern
                .pixels
                .iter()
                .flatten()
                .filter(|rgb| rgb.iter().any(|channel| *channel <= 100))
                .count();
            require(
                paper >= 8 && ink >= 8,
                "card mask needs at least eight paper and eight ink pixels",
            )?;


            for (_, other) in &templates {
                require(
                    disjoint(&pattern, other),
                    "card templates have overlapping or indistinguishable match regions",
                )?;
            }

            templates.push((card, pattern));
        }

        Ok(Self {
            foreground: None,
            templates,
            glyphs: None,
        })
    }


    /// Return only an independently matched identity; absent and ambiguous differ.
    fn recognise(&self, view: FrameView<'_>, origin: (u32, u32)) -> Result<Option<Card>, String> {
        if let Some(reader) = &self.foreground {
            return reader.recognise(view, origin);
        }
        if let Some(glyphs) = &self.glyphs {
            return glyphs.recognise(view, origin);
        }

        let mut recognised = None;


        for (card, template) in &self.templates {
            if template.matches(view, origin)? {
                require(
                    recognised.is_none(),
                    "ambiguous rank/suit template matches; candidate identities>=2; RGB mask policy",
                )?;
                recognised = Some(*card);
            }
        }

        Ok(recognised)
    }
}


impl GlyphReader {
    /// Parse exactly the original schema-1 XOR keys and their inclusive native bounds.
    fn parse(value: &Value, width: u32, height: u32) -> Result<Self, String> {
        let fields = object(
            value,
            &[
                "rank_roi",
                "suit_roi",
                "ranks",
                "suits",
                "rank_max_distance",
                "suit_max_distance",
                "alignment",
            ],
        )?;
        let rank_max_distance = integer(&fields["rank_max_distance"])?;
        let suit_max_distance = integer(&fields["suit_max_distance"])?;
        require(
            rank_max_distance <= 24 && suit_max_distance <= 8,
            "glyph mismatch/alignment limits exceed native calibration bounds",
        )?;
        Self::parse_parts(
            fields,
            width,
            height,
            GlyphPolicy::Xor {
                rank_max_distance: rank_max_distance as usize,
                suit_max_distance: suit_max_distance as usize,
            },
        )
    }


    /// Require the distinct schema-2 policy and reject legacy distance fields or mixed semantics.
    fn parse_graded(value: &Value, width: u32, height: u32) -> Result<Self, String> {
        let fields = object(
            value,
            &[
                "rank_roi",
                "suit_roi",
                "ranks",
                "suits",
                "alignment",
                "matching",
            ],
        )?;
        Self::parse_parts(
            fields,
            width,
            height,
            GlyphPolicy::Graded(graded_glyph::Policy::parse(&fields["matching"])?),
        )
    }


    /// Validate the shared measured templates and checked nine-alignment geometry.
    fn parse_parts(
        fields: &Map<String, Value>,
        width: u32,
        height: u32,
        policy: GlyphPolicy,
    ) -> Result<Self, String> {
        let rank_roi = rectangle(&fields["rank_roi"])?;
        let suit_roi = rectangle(&fields["suit_roi"])?;
        let face = PixelRect::new(0, 0, width, height);
        require(
            contains(face, rank_roi) && contains(face, suit_roi) && !overlaps(rank_roi, suit_roi),
            "rank/suit ROIs must be disjoint visible card regions",
        )?;
        let ranks = parse_glyph_bank(&fields["ranks"], rank_roi, &Rank::ALL, Rank::as_str, 8)?;
        let suits = parse_glyph_bank(&fields["suits"], suit_roi, &Suit::ALL, Suit::as_str, 16)?;
        if matches!(policy, GlyphPolicy::Graded(_)) {
            for pattern in ranks
                .iter()
                .flat_map(|(_, variants)| variants)
                .chain(suits.iter().flat_map(|(_, variants)| variants))
            {
                graded_glyph::validate_pattern(pattern)?;
            }
        }
        let alignment = integer(&fields["alignment"])?;
        require(
            alignment <= 1,
            "glyph mismatch/alignment limits exceed native calibration bounds",
        )?;
        for roi in [rank_roi, suit_roi] {
            require(
                roi.x >= alignment
                    && roi.y >= alignment
                    && roi
                        .right()
                        .checked_add(alignment)
                        .is_some_and(|edge| edge <= width)
                    && roi
                        .bottom()
                        .checked_add(alignment)
                        .is_some_and(|edge| edge <= height),
                "glyph alignment extends beyond its visible card face",
            )?;
        }
        Ok(Self {
            ranks,
            suits,
            policy,
            alignment: alignment as i32,
        })
    }


    /// Combine two actual independent glyph observations, never missing-card inference.
    fn recognise(&self, view: FrameView<'_>, origin: (u32, u32)) -> Result<Option<Card>, String> {
        let (rank_max_distance, suit_max_distance) = match self.policy {
            GlyphPolicy::Xor {
                rank_max_distance,
                suit_max_distance,
            } => (rank_max_distance, suit_max_distance),
            GlyphPolicy::Graded(policy) => return self.recognise_graded(view, origin, policy),
        };
        let rank = match_glyph_bank(&self.ranks, view, origin, rank_max_distance, self.alignment)
            .map_err(|error| format!("rank: {error}"))?;
        let suit = match_glyph_bank(&self.suits, view, origin, suit_max_distance, self.alignment)
            .map_err(|error| format!("suit: {error}"))?;


        match (rank, suit) {
            (Some(rank), Some(suit)) => {
                let red = matches!(suit, Suit::Diamonds | Suit::Hearts);
                let rank_matches = self
                    .ranks
                    .iter()
                    .find(|(label, _)| *label == rank)
                    .ok_or_else(|| "missing recognised rank bank".to_owned())?;
                let suit_matches = self
                    .suits
                    .iter()
                    .find(|(label, _)| *label == suit)
                    .ok_or_else(|| "missing recognised suit bank".to_owned())?;
                let mut rank_colour = false;
                let mut suit_colour = false;


                for variant in &rank_matches.1 {
                    rank_colour |= glyph_matches(
                        variant,
                        view,
                        origin,
                        rank_max_distance,
                        self.alignment,
                        Some(red),
                    )?;
                }


                for variant in &suit_matches.1 {
                    suit_colour |= glyph_matches(
                        variant,
                        view,
                        origin,
                        suit_max_distance,
                        self.alignment,
                        Some(red),
                    )?;
                }

                Ok((rank_colour && suit_colour).then_some(Card::new(rank, suit)))
            }
            _ => Ok(None),
        }
    }


    /// Join two graded witnesses only when each passes its own co-located guards and colours agree.
    fn recognise_graded(
        &self,
        view: FrameView<'_>,
        origin: (u32, u32),
        policy: graded_glyph::Policy,
    ) -> Result<Option<Card>, String> {
        let rank = graded_glyph::evaluate(
            &self.ranks,
            view,
            origin,
            policy.max_error_ppm(true),
            policy.min_gap_ppm(),
            self.alignment,
        )?;
        let suit = graded_glyph::evaluate(
            &self.suits,
            view,
            origin,
            policy.max_error_ppm(false),
            policy.min_gap_ppm(),
            self.alignment,
        )?;
        match (rank.accepted(), suit.accepted()) {
            (Some(rank), Some(suit))
                if rank.red == suit.red
                    && suit.red == matches!(suit.label, Suit::Diamonds | Suit::Hearts) =>
            {
                Ok(Some(Card::new(rank.label, suit.label)))
            }
            _ => Ok(None),
        }
    }
}


/// Validate native RGB masks as explicit ink/paper constraints without recolouring.
fn parse_glyph_bank<Label: Copy + PartialEq>(
    value: &Value,
    bounds: PixelRect,
    labels: &[Label],
    name: fn(Label) -> &'static str,
    maximum_variants: usize,
) -> Result<Vec<(Label, Vec<Pattern>)>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "glyph bank must be an array".to_owned())?;
    require(
        entries.len() == labels.len(),
        "glyph bank must cover every rank or suit",
    )?;
    let mut bank: Vec<(Label, Vec<Pattern>)> = Vec::with_capacity(labels.len());


    for entry in entries {
        let fields = object(entry, &["label", "variants"])?;
        let label = labels
            .iter()
            .copied()
            .find(|label| Some(name(*label)) == fields["label"].as_str())
            .ok_or_else(|| "invalid glyph label".to_owned())?;
        require(
            !bank.iter().any(|(known, _)| *known == label),
            "duplicate glyph label",
        )?;
        let variants = fields["variants"]
            .as_array()
            .ok_or_else(|| "glyph variants must be an array".to_owned())?;
        require(
            (1..=maximum_variants).contains(&variants.len()),
            "glyph variant count exceeds its bound",
        )?;
        let mut patterns = Vec::with_capacity(variants.len());


        for variant in variants {
            let pattern = Pattern::parse(variant, bounds)?;
            require(
                pattern.tolerance == 0,
                "normalised glyph tolerance must be zero",
            )?;
            require(
                pattern.pixels.iter().all(Option::is_some),
                "normalised glyph masks must include all negative space",
            )?;
            let ink = pattern
                .pixels
                .iter()
                .flatten()
                .filter(|rgb| ink_class(**rgb))
                .count();
            let paper = pattern.pixels.len() - ink;
            require(
                ink >= 8 && paper >= 8,
                "glyph mask needs eight ink and eight paper pixels",
            )?;


            for (_, other_patterns) in &bank {
                for other in other_patterns {
                    require(
                        glyphs_disjoint(&pattern, other),
                        "different glyph labels have identical occupancy",
                    )?;
                }
            }

            patterns.push(pattern);
        }

        bank.push((label, patterns));
    }

    Ok(bank)
}


/// Measured native binary occupancy; paper and colour guards remain independent.
fn ink_class(rgb: [u8; 3]) -> bool {
    rgb.iter().any(|channel| *channel < 128)
}


/// Reject identical cross-label occupancy; runtime rejects overlapping tolerance matches.
fn glyphs_disjoint(first: &Pattern, second: &Pattern) -> bool {
    first
        .pixels
        .iter()
        .zip(&second.pixels)
        .any(|(first, second)| match (first, second) {
            (Some(first), Some(second)) => ink_class(*first) != ink_class(*second),
            _ => false,
        })
}


/// Compare at most nine integer translations with bounded XOR, paper and colour checks.
fn glyph_matches(
    pattern: &Pattern,
    view: FrameView<'_>,
    origin: (u32, u32),
    maximum_distance: usize,
    alignment: i32,
    red: Option<bool>,
) -> Result<bool, String> {
    for shift_y in -alignment..=alignment {
        for shift_x in -alignment..=alignment {
            let column = i64::from(origin.0) + i64::from(pattern.bounds.x) + i64::from(shift_x);
            let row = i64::from(origin.1) + i64::from(pattern.bounds.y) + i64::from(shift_y);
            let column = u32::try_from(column).map_err(|_| "glyph x overflow".to_owned())?;
            let row = u32::try_from(row).map_err(|_| "glyph y overflow".to_owned())?;
            let bounds = PixelRect::new(column, row, pattern.bounds.width, pattern.bounds.height);
            view.checked_bounds(bounds)
                .map_err(|_| "glyph is outside the frame".to_owned())?;

            let mut distance = 0;
            let mut paper = 0;
            let mut red_ink = 0;
            let mut black_ink = 0;


            for (index, expected) in pattern.pixels.iter().enumerate() {
                let expected = expected
                    .ok_or_else(|| "glyph mask excludes required negative space".to_owned())?;
                let actual = view
                    .pixel_rgb(
                        column + index as u32 % bounds.width,
                        row + index as u32 / bounds.width,
                    )
                    .ok_or_else(|| "glyph pixel is outside the frame".to_owned())?;
                distance += usize::from(ink_class(actual) != ink_class(expected));
                paper += usize::from(actual.iter().all(|channel| *channel >= 180));
                red_ink += usize::from(
                    actual[0] >= 160
                        && actual[0].saturating_sub(actual[1]) >= 80
                        && actual[0].saturating_sub(actual[2]) >= 80,
                );
                black_ink += usize::from(actual.iter().all(|channel| *channel <= 100));


                if distance > maximum_distance {
                    break;
                }
            }

            let correct_colour = match red {
                Some(true) => red_ink >= 8 && black_ink == 0,
                Some(false) => black_ink >= 8 && red_ink == 0,
                None => red_ink + black_ink >= 8,
            };


            if distance <= maximum_distance && paper >= pattern.pixels.len() / 3 && correct_colour {
                return Ok(true);
            }
        }
    }

    Ok(false)
}


/// Multiple variants of one label are allowed; matching distinct labels is an error.
fn match_glyph_bank<Label: Copy + PartialEq>(
    bank: &[(Label, Vec<Pattern>)],
    view: FrameView<'_>,
    origin: (u32, u32),
    maximum_distance: usize,
    alignment: i32,
) -> Result<Option<Label>, String> {
    let mut recognised = None;


    for (label, variants) in bank {
        for pattern in variants {
            if glyph_matches(pattern, view, origin, maximum_distance, alignment, None)? {
                require(
                    recognised.is_none() || recognised == Some(*label),
                    "ambiguous glyph matches; candidate labels>=2; bounded XOR/paper/ink policy",
                )?;
                recognised = Some(*label);
            }
        }
    }

    Ok(recognised)
}


impl BoardProfile {
    /// Validate scene, toolbar and positive-empty evidence without inferring pixels.
    fn parse(value: &Value, schema: u32) -> Result<Self, String> {
        let mut names = vec![
            "scene",
            "solver_off",
            "undo_all",
            "left_empty",
            "right_empty",
        ];
        if schema == 2 {
            for optional in [
                "execution_empty_stock",
                "final_clear",
                "normal_final_clear",
                "board_count_final_clear",
                "board_count_target_clear",
                "challenge_final_clear",
            ] {
                if value.get(optional).is_some() {
                    names.push(optional);
                }
            }
        }
        let fields = object(value, &names)?;
        let entries = fields["scene"]
            .as_array()
            .ok_or_else(|| "scene signature must be an array".to_owned())?;
        require(
            (3..=8).contains(&entries.len()),
            "board signature requires three to eight patches",
        )?;
        let scene: Vec<_> = entries
            .iter()
            .map(|entry| Pattern::patch(entry, NATIVE_BOUNDS))
            .collect::<Result<_, _>>()?;
        require_disjoint_bounds(&scene)?;
        // Retain strict schema-1 compatibility without treating toolbar appearance
        // as scene or input authority. The operator confirms Solver is not engaged.
        let _ = Pattern::patch(&fields["solver_off"], SOLVER_BOUNDS)?;
        let undo_all = Pattern::patch(&fields["undo_all"], UNDO_BOUNDS)?;
        require(
            contains(undo_all.bounds, PixelRect::new(1320, 978, 1, 1)),
            "Undo All signature must cover its click point",
        )?;
        let left_empty = Pattern::patch(&fields["left_empty"], LEFT_BOUNDS)?;
        let execution_empty_stock = fields
            .get("execution_empty_stock")
            .map(|variant| ExecutionEmptyStock::parse(variant, &scene, &left_empty))
            .transpose()?;
        let final_clear = fields
            .get("final_clear")
            .map(|value| final_clear::FinalClearProfile::parse(value, &scene, &left_empty))
            .transpose()?;
        let normal_final_clear = fields
            .get("normal_final_clear")
            .map(|value| normal_clear::NormalClearProfile::parse(value, &scene, &left_empty))
            .transpose()?;
        let board_count_final_clear = if let Some(value) = fields.get("board_count_final_clear") {
            require(
                value.as_bool() == Some(true) && final_clear.is_some(),
                "board-count final-clear requires explicit true and original final-clear calibration",
            )?;
            true
        } else {
            false
        };
        let board_count_target_clear = if let Some(value) = fields.get("board_count_target_clear") {
            require(
                value.as_bool() == Some(true) && board_count_final_clear,
                "board-count target-clear requires explicit true and bound final-clear calibration",
            )?;
            true
        } else {
            false
        };
        let challenge_final_clear = if let Some(value) = fields.get("challenge_final_clear") {
            require(
                value.as_bool() == Some(true) && board_count_target_clear,
                "challenge final-clear requires explicit true and retained target calibration",
            )?;
            true
        } else {
            false
        };
        Ok(Self {
            scene,
            undo_all,
            left_empty,
            right_empty: Pattern::patch(&fields["right_empty"], RIGHT_BOUNDS)?,
            execution_empty_stock,
            final_clear,
            normal_final_clear,
            board_count_final_clear,
            board_count_target_clear,
            challenge_final_clear,
        })
    }


    /// Require every supported scene patch, independently of Solver availability.
    fn matches(&self, view: FrameView<'_>) -> Result<bool, String> {
        matches_all(&self.scene, view)
    }
}


impl ExecutionEmptyStock {
    /// Validate one positive coupled variant within its original execution roles.
    fn parse(
        value: &Value,
        original_scene: &[Pattern],
        original_left: &Pattern,
    ) -> Result<Self, String> {
        let fields = object(value, &["scene", "left_empty"])?;
        let scene = Pattern::patch(&fields["scene"], EXECUTION_EMPTY_STOCK_SCENE_BOUNDS)?;
        let original_role = original_scene
            .get(EXECUTION_EMPTY_STOCK_SCENE_INDEX)
            .ok_or("missing original execution stock scene role")?;
        require(
            contains(EXECUTION_EMPTY_STOCK_SCENE_BOUNDS, original_role.bounds),
            "original execution stock scene role must lie within the central stock corridor",
        )?;
        require(
            scene.tolerance == original_role.tolerance && scene.pixels.iter().all(Option::is_some),
            "execution stock scene must retain original tolerance and select its complete positive artwork",
        )?;
        require(
            original_scene.iter().enumerate().all(|(index, pattern)| {
                index == EXECUTION_EMPTY_STOCK_SCENE_INDEX
                    || !overlaps(scene.bounds, pattern.bounds)
            }),
            "execution stock scene overlaps an unchanged scene anchor",
        )?;
        let left_empty = Pattern::patch(&fields["left_empty"], LEFT_BOUNDS)?;
        require(
            left_empty.bounds == original_left.bounds
                && left_empty.tolerance == original_left.tolerance
                && left_empty
                    .pixels
                    .iter()
                    .zip(&original_left.pixels)
                    .all(|(new, original)| new.is_some() == original.is_some()),
            "execution empty LEFT must retain original bounds, mask and tolerance",
        )?;
        Ok(Self { scene, left_empty })
    }
}


impl DialogProfile {
    /// Require distinct measured panel/content/control regions and two backdrop patches.
    fn parse(value: &Value, schema: u32) -> Result<Self, String> {
        let fields = if schema == 2 && value.get("post_confirm_ok").is_some() {
            object(
                value,
                &[
                    "bounds",
                    "panel",
                    "title",
                    "body",
                    "cancel",
                    "ok",
                    "ok_point",
                    "backdrop",
                    "post_confirm_ok",
                ],
            )?
        } else {
            object(
                value,
                &[
                    "bounds", "panel", "title", "body", "cancel", "ok", "ok_point", "backdrop",
                ],
            )?
        };
        let bounds = rectangle(&fields["bounds"])?;
        require(
            contains(PixelRect::new(300, 100, 1320, 880), bounds),
            "dialog bounds exceed the supported central region",
        )?;
        let mut signature = Vec::with_capacity(7);


        for label in ["panel", "title", "body", "cancel", "ok"] {
            signature.push(Pattern::patch(&fields[label], bounds)?);
        }

        require_disjoint_bounds(&signature)?;
        let point = fields["ok_point"]
            .as_array()
            .ok_or_else(|| "OK point must be an array".to_owned())?;
        require(point.len() == 2, "OK point must have two coordinates")?;
        let point_x = integer(&point[0])?;
        let point_y = integer(&point[1])?;
        require(
            contains(signature[4].bounds, PixelRect::new(point_x, point_y, 1, 1)),
            "OK point is outside its matched patch",
        )?;
        let backdrop = fields["backdrop"]
            .as_array()
            .ok_or_else(|| "dialog backdrop must be an array".to_owned())?;
        require(
            backdrop.len() == 2,
            "dialog requires two native backdrop patches",
        )?;


        for entry in backdrop {
            let pattern = Pattern::patch_with_contrast(entry, NATIVE_BOUNDS, 16)?;
            require(
                !overlaps(bounds, pattern.bounds),
                "dialog backdrop overlaps the dialog",
            )?;
            signature.push(pattern);
        }

        require_disjoint_bounds(&signature)?;
        let post_confirm_ok = fields
            .get("post_confirm_ok")
            .map(|value| -> Result<Pattern, String> {
                let alternate = Pattern::patch(value, signature[4].bounds)?;
                require(
                    alternate.bounds == signature[4].bounds
                        && alternate.tolerance == signature[4].tolerance
                        && alternate
                            .pixels
                            .iter()
                            .zip(&signature[4].pixels)
                            .all(|(new, original)| new.is_some() == original.is_some()),
                    "post-confirm OK must retain original bounds, mask and tolerance",
                )?;
                Ok(alternate)
            })
            .transpose()?;
        Ok(Self {
            signature,
            ok: PixelPoint::new(point_x as i32, point_y as i32),
            post_confirm_ok,
        })
    }


    /// Match the complete ordinary modal or the complete explicitly selected post-OK variant.
    fn matches(&self, view: FrameView<'_>, after_confirmation: bool) -> Result<bool, String> {
        if matches_all(&self.signature, view)? {
            return Ok(true);
        }
        if !after_confirmation {
            return Ok(false);
        }
        let Some(alternate) = &self.post_confirm_ok else {
            return Ok(false);
        };
        for (index, pattern) in self.signature.iter().enumerate() {
            let required = if index == 4 { alternate } else { pattern };
            if !required.matches(view, (0, 0))? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}


/// Find one jointly constrained channel proving the two tolerance boxes cannot overlap.
fn disjoint(first: &Pattern, second: &Pattern) -> bool {
    let separation = u16::from(first.tolerance) + u16::from(second.tolerance);
    first
        .pixels
        .iter()
        .zip(&second.pixels)
        .any(|(first, second)| match (first, second) {
            (Some(first), Some(second)) => first
                .iter()
                .zip(second)
                .any(|(first, second)| u16::from(first.abs_diff(*second)) > separation),
            _ => false,
        })
}


/// Match every bounded patch without retaining frames or returning partial evidence.
fn matches_all(patterns: &[Pattern], view: FrameView<'_>) -> Result<bool, String> {
    for pattern in patterns {
        if !pattern.matches(view, (0, 0))? {
            return Ok(false);
        }
    }

    Ok(true)
}


/// Enforce a fixed schema rather than silently accepting typo or extension fields.
fn object<'a>(value: &'a Value, expected: &[&str]) -> Result<&'a Map<String, Value>, String> {
    let fields = value
        .as_object()
        .ok_or_else(|| "profile requires a JSON object".to_owned())?;
    require(
        fields.len() == expected.len() && expected.iter().all(|key| fields.contains_key(*key)),
        "profile has missing or unknown fields",
    )?;
    Ok(fields)
}


/// Return a checked nonnegative integral u32; floats and numeric strings refuse.
fn integer(value: &Value) -> Result<u32, String> {
    value
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| "profile requires a bounded unsigned integer".to_owned())
}


/// Extract an actual JSON string without coercion or unchecked indexing.
fn string(value: &Value) -> Result<&str, String> {
    value
        .as_str()
        .ok_or_else(|| "profile requires a string".to_owned())
}


/// Decode lower-case native RGB hex; upper-case or malformed bytes refuse.
fn hex(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("RGB samples require lower-case hexadecimal".to_owned()),
    }
}


/// Parse positive half-open geometry with checked native edges before multiplication.
fn rectangle(value: &Value) -> Result<PixelRect, String> {
    let coordinates = value
        .as_array()
        .ok_or_else(|| "bounds must be an array".to_owned())?;
    require(
        coordinates.len() == 4,
        "bounds must contain x, y, width, height",
    )?;
    let bounds = PixelRect::new(
        integer(&coordinates[0])?,
        integer(&coordinates[1])?,
        integer(&coordinates[2])?,
        integer(&coordinates[3])?,
    );
    require(
        contains(NATIVE_BOUNDS, bounds),
        "bounds are empty, overflow or outside native geometry",
    )?;
    Ok(bounds)
}


/// Check half-open containment without relying on saturating rectangle edges.
fn contains(outer: PixelRect, inner: PixelRect) -> bool {
    !inner.is_empty()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner
            .x
            .checked_add(inner.width)
            .zip(outer.x.checked_add(outer.width))
            .is_some_and(|(inner, outer)| inner <= outer)
        && inner
            .y
            .checked_add(inner.height)
            .zip(outer.y.checked_add(outer.height))
            .is_some_and(|(inner, outer)| inner <= outer)
}


/// Overlap is used only after native checked bounds validation.
fn overlaps(first: PixelRect, second: PixelRect) -> bool {
    first.x < second.right()
        && second.x < first.right()
        && first.y < second.bottom()
        && second.y < first.bottom()
}


/// Prevent several named signature components from being copies of one patch.
fn require_disjoint_bounds(patterns: &[Pattern]) -> Result<(), String> {
    for (index, pattern) in patterns.iter().enumerate() {
        require(
            patterns[..index]
                .iter()
                .all(|previous| !overlaps(previous.bounds, pattern.bounds)),
            "signature component bounds overlap",
        )?;
    }

    Ok(())
}


/// Return bounded static diagnostics without copying attacker-controlled values.
fn require(condition: bool, diagnostic: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(diagnostic.to_owned())
    }
}


/// Bounded structural preflight rejects duplicate keys before `Value` can lose them.
struct JsonGuard<'a> {
    /// Already byte-capped source; full JSON syntax is subsequently checked by serde.
    bytes: &'a [u8],
    /// Current offset into the capped source.
    cursor: usize,
    /// Global value count, independent of nesting and whitespace.
    values: usize,
}


impl JsonGuard<'_> {
    /// Bound nesting, collections, strings and decoded keys before tree allocation.
    fn check(bytes: &[u8]) -> Result<(), String> {
        let mut guard = JsonGuard {
            bytes,
            cursor: 0,
            values: 0,
        };
        guard.value(0)?;
        guard.whitespace();
        require(guard.cursor == bytes.len(), "trailing profile data")
    }


    /// Skip JSON whitespace only, never broader Unicode whitespace.
    fn whitespace(&mut self) {
        while self
            .bytes
            .get(self.cursor)
            .is_some_and(|byte| b" \n\r\t".contains(byte))
        {
            self.cursor += 1;
        }
    }


    /// Consume a punctuation token after optional JSON whitespace.
    fn token(&mut self, expected: u8) -> Result<(), String> {
        self.whitespace();
        require(
            self.bytes.get(self.cursor) == Some(&expected),
            "invalid profile JSON structure",
        )?;
        self.cursor += 1;
        Ok(())
    }


    /// Scan one quoted token; serde validates escapes, Unicode and control bytes.
    fn quoted(&mut self) -> Result<&[u8], String> {
        self.whitespace();
        let start = self.cursor;
        self.token(b'"')?;


        while let Some(byte) = self.bytes.get(self.cursor) {
            self.cursor += 1;


            match byte {
                b'"' => {
                    require(
                        self.cursor - start <= MAX_PATCH_PIXELS * 6 + 2,
                        "profile string exceeds its bound",
                    )?;
                    return Ok(&self.bytes[start..self.cursor]);
                }
                b'\\' => {
                    require(self.cursor < self.bytes.len(), "truncated profile escape")?;
                    self.cursor += 1;
                }
                _ => {}
            }
        }

        Err("unterminated profile string".to_owned())
    }


    /// Walk at most 16 levels and 16,384 values with bounded containers.
    fn value(&mut self, depth: usize) -> Result<(), String> {
        self.values += 1;
        require(
            depth <= 16 && self.values <= 16_384,
            "profile JSON complexity exceeds its bound",
        )?;
        self.whitespace();


        match self.bytes.get(self.cursor) {
            Some(b'{') => self.container(depth, true),
            Some(b'[') => self.container(depth, false),
            Some(b'"') => self.quoted().map(|_| ()),
            Some(_) => {
                let start = self.cursor;


                while self
                    .bytes
                    .get(self.cursor)
                    .is_some_and(|byte| !b" \n\r\t,]}".contains(byte))
                {
                    self.cursor += 1;
                }

                require(
                    (1..=32).contains(&(self.cursor - start)),
                    "invalid or excessive profile scalar",
                )
            }
            None => Err("truncated profile JSON".to_owned()),
        }
    }


    /// Decode and deduplicate at most 16 object keys, or accept at most 64 array values.
    fn container(&mut self, depth: usize, is_object: bool) -> Result<(), String> {
        self.cursor += 1;
        let closing = if is_object { b'}' } else { b']' };
        let mut keys = BTreeSet::new();
        let mut count = 0;
        self.whitespace();


        if self.bytes.get(self.cursor) == Some(&closing) {
            self.cursor += 1;
            return Ok(());
        }


        loop {
            count += 1;
            require(
                count <= if is_object { 16 } else { 64 },
                "profile JSON collection exceeds its bound",
            )?;


            if is_object {
                let quoted = self.quoted()?;
                require(quoted.len() <= 194, "profile JSON key exceeds its bound")?;
                let key: String = serde_json::from_slice(quoted)
                    .map_err(|_| "invalid profile JSON key".to_owned())?;
                require(
                    key.len() <= 32 && keys.insert(key),
                    "duplicate or excessive profile JSON key",
                )?;
                self.token(b':')?;
            }

            self.value(depth + 1)?;
            self.whitespace();


            if self.bytes.get(self.cursor) == Some(&closing) {
                self.cursor += 1;
                return Ok(());
            }

            self.token(b',')?;
        }
    }
}


#[cfg(test)]
mod tests {
    //! Separate synthetic mechanism tests and provenance-labelled native regressions.

    use super::*;
    use crate::capture::PixelFormat;
    use serde_json::json;


    /// Construct a labelled test deck with no implication of screen recognition.
    fn deck() -> Vec<Card> {
        Suit::ALL
            .into_iter()
            .flat_map(|suit| Rank::ALL.into_iter().map(move |rank| Card::new(rank, suit)))
            .collect()
    }


    /// Synthetic six-bit patterns uniquely label 52 cards and retain paper/ink.
    fn samples(code: usize) -> Value {
        let mut rgb = String::with_capacity(64 * 6);


        for pixel in 0..64 {
            let bright = if pixel < 6 {
                code & (1 << pixel) != 0
            } else {
                pixel % 2 == 0
            };
            rgb.push_str(if bright { "ffffff" } else { "000000" });
        }

        json!({
            "rgb": rgb, "mask": "1".repeat(64), "tolerance": 4,
            "source_sha256": "0".repeat(64),
        })
    }


    /// Describe a tiny synthetic native-coordinate patch, not a captured asset.
    fn patch(origin: (u32, u32)) -> Value {
        json!({"bounds": [origin.0, origin.1, 8, 8], "samples": samples(63)})
    }


    /// A complete, strict synthetic profile used only inside this unit-test module.
    fn profile_value() -> Value {
        let templates: Vec<_> = deck().iter().enumerate().map(|(index, card)| {
            json!({"rank": card.rank().as_str(), "suit": card.suit().as_str(), "samples": samples(index)})
        }).collect();
        let reader = json!({"roi": [8, 8, 8, 8], "templates": templates});
        json!({
            "schema_version": 1, "profile_id": "synthetic-unit-test-only",
            "game": "pyramid", "artwork": "CLASSIC BLUE", "dimensions": [1920, 1080],
            "tableau_reader": reader.clone(), "stock_reader": reader,
            "board": {
                "scene": [patch((100, 100)), patch((1800, 100)), patch((1800, 800))],
                "solver_off": patch((585, 959)), "undo_all": patch((1316, 974)),
                "left_empty": patch((800, 800)), "right_empty": patch((1100, 800)),
            },
            "dialog": {
                "bounds": [600, 250, 700, 450], "panel": patch((600, 250)),
                "title": patch((850, 290)), "body": patch((800, 400)),
                "cancel": patch((800, 600)), "ok": patch((1100, 600)),
                "ok_point": [1103, 603],
                "backdrop": [patch((100, 400)), patch((1800, 400))],
            },
        })
    }


    /// Exercise the real byte parser rather than constructing unchecked internals.
    fn parse(value: &Value) -> Result<ReconVision, String> {
        ReconVision::from_bytes(&serde_json::to_vec(value).expect("serialise test profile"))
    }


    /// Native-sized RGBA buffer with controllable row padding and no signatures.
    fn frame(padding: usize) -> CapturedFrame {
        CapturedFrame {
            width: 1920,
            height: 1080,
            stride: 1920 * 4 + padding,
            format: PixelFormat::Rgba8,
            pixels: vec![0; (1920 * 4 + padding) * 1080],
        }
    }


    /// Paint a test pattern into checked dimensions while leaving alpha arbitrary.
    fn paint(frame: &mut CapturedFrame, pattern: &Pattern, origin: (u32, u32)) {
        for (index, expected) in pattern.pixels.iter().enumerate() {
            if let Some(rgb) = expected {
                let column = origin.0 + pattern.bounds.x + index as u32 % pattern.bounds.width;
                let row = origin.1 + pattern.bounds.y + index as u32 / pattern.bounds.width;
                let offset = row as usize * frame.stride + column as usize * 4;
                frame.pixels[offset..offset + 3].copy_from_slice(rgb);
                frame.pixels[offset + 3] = (index % 256) as u8;
            }
        }
    }


    /// Paint exactly one test-supported complete board with optional Undo readiness.
    fn board_frame(vision: &ReconVision, enabled_undo: bool) -> CapturedFrame {
        let mut frame = frame(13);


        for pattern in &vision.board.scene {
            paint(&mut frame, pattern, (0, 0));
        }



        if enabled_undo {
            paint(&mut frame, &vision.board.undo_all, (0, 0));
        }


        for (index, origin) in TABLEAU_ORIGINS.into_iter().enumerate() {
            paint(
                &mut frame,
                &vision.tableau_reader.templates[index].1,
                origin,
            );
        }

        paint(
            &mut frame,
            &vision.stock_reader.templates[28].1,
            (LEFT_BOUNDS.x, LEFT_BOUNDS.y),
        );
        paint(&mut frame, &vision.board.right_empty, (0, 0));
        frame
    }


    /// The fixed exported ordering is apex first and bottom-right last.
    #[test]
    fn logical_geometry_is_top_down_without_legacy_target_indices() {
        assert_eq!(TABLEAU_ORIGINS.len(), 28);
        assert_eq!(TABLEAU_ORIGINS[0], (890, 112));
        assert_eq!(TABLEAU_ORIGINS[1], (790, 165));
        assert_eq!(TABLEAU_ORIGINS[2], (990, 165));
        assert_eq!(TABLEAU_ORIGINS[21], (288, 432));
        assert_eq!(TABLEAU_ORIGINS[27], (1492, 432));


        for row in 0..7 {
            let start = row * (row + 1) / 2;
            let positions = &TABLEAU_ORIGINS[start..start + row + 1];
            assert!(
                positions
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 == pair[1].1)
            );
        }
    }


    /// All tableau identities, padded stride and optional enabled Undo are independent.
    #[test]
    fn complete_board_uses_actual_labels_and_optional_undo() {
        let vision = parse(&profile_value()).expect("complete test profile");
        assert_eq!(vision.profile_id(), "synthetic-unit-test-only");


        for enabled in [false, true] {
            let observed = vision
                .observe(&board_frame(&vision, enabled))
                .expect("board classification");
            let ReconScene::Board(board) = observed else {
                panic!("expected complete board")
            };
            assert_eq!(board.tableau.as_slice(), &deck()[..28]);
            assert_eq!(board.left, StockFace::Card(deck()[28]));
            assert_eq!(board.right, StockFace::Empty);
            assert_eq!(
                board.undo_all,
                enabled.then_some(PixelPoint::new(1320, 978))
            );
        }
    }


    /// Removing any supported scene component yields Unknown regardless of Solver.
    #[test]
    fn board_requires_every_scene_patch_without_solver_icon() {
        let vision = parse(&profile_value()).expect("profile");


        for pattern in &vision.board.scene {
            let mut native = board_frame(&vision, true);
            let offset = pattern.bounds.y as usize * native.stride + pattern.bounds.x as usize * 4;
            native.pixels[offset..offset + 3].fill(127);
            assert_eq!(
                vision.observe(&native).expect("unsupported board"),
                ReconScene::Unknown
            );
        }
    }


    /// Every labelled identity is independently recognisable with no deck elimination.
    #[test]
    fn both_readers_classify_all_52_and_reject_unknown() {
        let vision = parse(&profile_value()).expect("profile");


        for reader in [&vision.tableau_reader, &vision.stock_reader] {
            for (card, pattern) in &reader.templates {
                let mut native = frame(0);
                paint(&mut native, pattern, (100, 100));
                assert_eq!(
                    reader
                        .recognise(native.as_view().expect("view"), (100, 100))
                        .expect("match"),
                    Some(*card)
                );
            }

            let blank = frame(0);
            assert_eq!(
                reader
                    .recognise(blank.as_view().expect("view"), (100, 100))
                    .expect("unknown"),
                None
            );
        }
    }


    /// A duplicate physical observation or unreadable tableau card aborts acquisition.
    #[test]
    fn duplicate_and_unknown_tableau_are_errors() {
        let vision = parse(&profile_value()).expect("profile");
        let mut duplicate = board_frame(&vision, true);
        paint(
            &mut duplicate,
            &vision.tableau_reader.templates[0].1,
            TABLEAU_ORIGINS[1],
        );
        assert!(
            vision
                .observe(&duplicate)
                .expect_err("duplicate")
                .contains("duplicate identity")
        );
        let mut unknown = board_frame(&vision, false);
        let bounds = vision.tableau_reader.templates[0].1.bounds;
        let offset = (TABLEAU_ORIGINS[0].1 + bounds.y) as usize * unknown.stride
            + (TABLEAU_ORIGINS[0].0 + bounds.x) as usize * 4;
        unknown.pixels[offset..offset + 3].fill(127);
        assert!(
            vision
                .observe(&unknown)
                .expect_err("unknown")
                .contains("tableau slot 0")
        );
    }


    /// Empty evidence is positive and cannot coexist with a visible matching card.
    #[test]
    fn pile_empty_and_card_evidence_must_be_exclusive() {
        let vision = parse(&profile_value()).expect("profile");
        let mut native = frame(0);
        let view = native.as_view().expect("view");
        assert!(
            vision
                .stock_face(view, LEFT_BOUNDS, &vision.board.left_empty)
                .is_err()
        );
        paint(&mut native, &vision.board.left_empty, (0, 0));
        assert_eq!(
            vision
                .stock_face(
                    native.as_view().expect("view"),
                    LEFT_BOUNDS,
                    &vision.board.left_empty
                )
                .expect("positive empty"),
            StockFace::Empty
        );
        paint(
            &mut native,
            &vision.stock_reader.templates[28].1,
            (LEFT_BOUNDS.x, LEFT_BOUNDS.y),
        );
        assert!(
            vision
                .stock_face(
                    native.as_view().expect("view"),
                    LEFT_BOUNDS,
                    &vision.board.left_empty
                )
                .expect_err("ambiguous")
                .contains("ambiguous")
        );
    }


    /// All seven dialog components are required; an OK patch alone never authorises.
    #[test]
    fn dialog_requires_complete_fresh_native_signature() {
        let vision = parse(&profile_value()).expect("profile");
        let mut native = frame(0);
        paint(&mut native, &vision.dialog.signature[4], (0, 0));
        assert_eq!(
            vision.observe(&native).expect("OK only"),
            ReconScene::Unknown
        );


        for pattern in &vision.dialog.signature {
            paint(&mut native, pattern, (0, 0));
        }

        assert_eq!(
            vision.observe(&native).expect("dialog"),
            ReconScene::UndoConfirmation {
                ok: PixelPoint::new(1103, 603)
            }
        );


        for pattern in &vision.dialog.signature {
            let mut damaged = native.clone();
            let offset = pattern.bounds.y as usize * damaged.stride + pattern.bounds.x as usize * 4;
            damaged.pixels[offset..offset + 3].fill(127);
            assert_eq!(
                vision.observe(&damaged).expect("partial dialog"),
                ReconScene::Unknown
            );
        }
    }


    /// Even a contradictory profile/frame must not choose one of two complete scenes.
    #[test]
    fn simultaneous_board_and_dialog_are_ambiguous() {
        let vision = parse(&profile_value()).expect("profile");
        let mut native = board_frame(&vision, true);


        for pattern in &vision.dialog.signature {
            paint(&mut native, pattern, (0, 0));
        }

        assert!(
            vision
                .observe(&native)
                .expect_err("ambiguous scene")
                .contains("board/dialog")
        );
    }


    /// Invalid public frame storage, native dimensions and overflowing origins refuse.
    #[test]
    fn invalid_frames_and_patch_bounds_refuse() {
        let vision = parse(&profile_value()).expect("profile");
        let mut native = frame(0);
        native.width = 1919;
        assert!(vision.observe(&native).is_err());
        native.width = 1920;
        native.pixels.truncate(1);
        assert!(vision.observe(&native).is_err());
        let valid = frame(0);
        let view = valid.as_view().expect("view");
        assert!(
            vision
                .tableau_reader
                .recognise(view, (u32::MAX, 0))
                .is_err()
        );
        assert!(vision.tableau_reader.recognise(view, (1919, 1079)).is_err());
    }


    /// Inclusive RGB tolerance is exact and alpha does not participate.
    #[test]
    fn tolerance_boundary_ignores_alpha_and_padding() {
        let vision = parse(&profile_value()).expect("profile");
        let pattern = &vision.tableau_reader.templates[0].1;
        let mut native = frame(29);
        paint(&mut native, pattern, (0, 0));
        let offset = pattern.bounds.y as usize * native.stride + pattern.bounds.x as usize * 4;
        native.pixels[offset] = 4;
        native.pixels[offset + 3] = 255;
        assert!(
            pattern
                .matches(native.as_view().expect("view"), (0, 0))
                .expect("inclusive")
        );
        native.pixels[offset] = 5;
        assert!(
            !pattern
                .matches(native.as_view().expect("view"), (0, 0))
                .expect("outside")
        );
    }


    /// Missing/extra identities, duplicate labels and overlapping match boxes refuse.
    #[test]
    fn readers_require_complete_unique_discriminating_coverage() {
        let original = profile_value();


        for bank in ["tableau_reader", "stock_reader"] {
            let mut missing = original.clone();
            missing[bank]["templates"]
                .as_array_mut()
                .expect("array")
                .pop();
            assert!(parse(&missing).is_err());
            let mut duplicate = original.clone();
            duplicate[bank]["templates"][1] = duplicate[bank]["templates"][0].clone();
            assert!(
                parse(&duplicate)
                    .expect_err("duplicate label")
                    .contains("duplicate")
            );
            let mut indistinguishable = original.clone();
            indistinguishable[bank]["templates"][1]["samples"] =
                indistinguishable[bank]["templates"][0]["samples"].clone();
            assert!(
                parse(&indistinguishable)
                    .expect_err("overlap")
                    .contains("overlapping")
            );
            let mut invalid_label = original.clone();
            invalid_label[bank]["templates"][0]["suit"] = json!("red");
            assert!(parse(&invalid_label).is_err());
        }
    }


    /// Invalid profile values are not converted, inferred or silently defaulted.
    #[test]
    fn masks_colours_bounds_and_tolerances_are_checked() {
        let original = profile_value();


        for (field, invalid) in [
            ("mask", json!("0".repeat(64))),
            (
                "mask",
                json!(format!("{}{}", "1".repeat(31), "0".repeat(33))),
            ),
            ("mask", json!("x".repeat(64))),
            ("mask", json!("1".repeat(63))),
            ("rgb", json!("ffffff".repeat(64))),
            ("rgb", json!("zzzzzz".repeat(64))),
            ("rgb", json!("000000")),
            ("tolerance", json!(25)),
            ("tolerance", json!(-1)),
            ("tolerance", json!(4.0)),
            ("source_sha256", json!("not-a-hash")),
        ] {
            let mut invalid_profile = original.clone();
            invalid_profile["tableau_reader"]["templates"][0]["samples"][field] = invalid;
            assert!(parse(&invalid_profile).is_err(), "accepted invalid {field}");
        }


        for roi in [
            json!([0, 52, 8, 8]),
            json!([139, 0, 8, 8]),
            json!([0, 0, 0, 8]),
            json!([u32::MAX, 0, 8, 8]),
        ] {
            let mut invalid = original.clone();
            invalid["tableau_reader"]["roi"] = roi;
            assert!(parse(&invalid).is_err());
        }
    }


    /// Profile version, artwork, native dimensions and every object's fields are strict.
    #[test]
    fn strict_schema_rejects_unknown_missing_and_wrong_typed_fields() {
        let original = profile_value();


        for (field, invalid) in [
            ("schema_version", json!(2)),
            ("game", json!("tripeaks")),
            ("artwork", json!("OTHER")),
            ("dimensions", json!([1280, 720])),
            ("profile_id", json!("invalid\nidentifier")),
            ("profile_id", json!("")),
            ("board", Value::Null),
        ] {
            let mut modified = original.clone();
            modified[field] = invalid;
            assert!(parse(&modified).is_err(), "accepted invalid {field}");
        }


        for pointer in [
            "",
            "/board",
            "/dialog",
            "/tableau_reader",
            "/tableau_reader/templates/0",
            "/tableau_reader/templates/0/samples",
        ] {
            let mut modified = original.clone();
            modified
                .pointer_mut(pointer)
                .expect("object")
                .as_object_mut()
                .expect("map")
                .insert("unexpected".to_owned(), json!(true));
            assert!(parse(&modified).is_err());
        }

        let mut missing = original;
        missing.as_object_mut().expect("root").remove("dialog");
        assert!(parse(&missing).is_err());
    }


    /// Misplaced controls, reused regions and incomplete dialog signatures refuse.
    #[test]
    fn signature_geometry_prevents_generic_control_substitution() {
        let original = profile_value();


        for (pointer, invalid) in [
            ("/dialog/ok_point", json!([500, 500])),
            ("/dialog/backdrop", json!([patch((100, 400))])),
            ("/dialog/backdrop/0", patch((700, 300))),
            ("/dialog/body", original["dialog"]["title"].clone()),
            ("/board/scene/1", original["board"]["scene"][0].clone()),
            ("/board/undo_all/bounds", json!([1301, 959, 8, 8])),
            ("/board/solver_off/bounds", json!([100, 100, 8, 8])),
            ("/board/left_empty/bounds", json!([100, 100, 8, 8])),
        ] {
            let mut modified = original.clone();
            *modified.pointer_mut(pointer).expect("field") = invalid;
            assert!(parse(&modified).is_err(), "accepted invalid {pointer}");
        }
    }


    /// Key aliases using escapes cannot evade duplicate-key rejection at any depth.
    #[test]
    fn duplicate_keys_are_rejected_before_value_parsing() {
        for bytes in [
            br#"{"schema_version":1,"schema_version":1}"#.as_slice(),
            br#"{"outer":{"mask":"a","m\u0061sk":"b"}}"#.as_slice(),
            br#"[{"same":0,"same":1}]"#.as_slice(),
        ] {
            assert!(
                JsonGuard::check(bytes)
                    .expect_err("duplicate")
                    .contains("duplicate")
            );
        }

        JsonGuard::check(br#"{"left":{"same":1},"right":{"same":2}}"#)
            .expect("independent object keys");
    }


    /// All document and structural caps apply before materialising a JSON value tree.
    #[test]
    fn oversized_deep_and_excessive_profiles_refuse() {
        assert!(
            ReconVision::from_bytes(&vec![b' '; MAX_PROFILE_BYTES + 1])
                .expect_err("size")
                .contains("2 MiB")
        );
        let deep = format!("{}0{}", "[".repeat(18), "]".repeat(18));
        assert!(JsonGuard::check(deep.as_bytes()).is_err());
        let many = serde_json::to_vec(&vec![0; 65]).expect("array");
        assert!(JsonGuard::check(&many).is_err());
        let long_string =
            serde_json::to_vec(&"a".repeat(MAX_PATCH_PIXELS * 6 + 1)).expect("string");
        assert!(JsonGuard::check(&long_string).is_err());


        for malformed in [
            b"".as_slice(),
            b"{",
            b"[",
            b"{\"x\":1,}",
            b"{\"x\":\"\\",
            b"{} {}",
            b"[0,]",
            b"\xff",
        ] {
            assert!(ReconVision::from_bytes(malformed).is_err());
        }
    }


    /// Load refuses special-file mistakes without treating them as absent evidence.
    #[test]
    #[cfg(unix)]
    fn profile_loader_refuses_non_regular_files() {
        assert!(ReconVision::load(Path::new("/dev/null")).is_err());
        assert!(ReconVision::load(Path::new("/tmp")).is_err());
    }


    /// Read an unchanged retained original; no filler frame is represented as native.
    pub(super) fn native_original(source_id: &str) -> CapturedFrame {
        let manifest = native_manifest();
        let source = manifest["originals"]
            .as_array()
            .expect("originals")
            .iter()
            .find(|source| source["id"] == source_id)
            .expect("declared native original");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon")
            .join(source["file"].as_str().expect("retained relative file"));
        let bytes = std::fs::read(path).expect("retained native fixture original");
        crate::capture::decode_png(&bytes).expect("decode original native PNG")
    }


    /// Independently reviewed actual observations, not identities inferred by the reader.
    pub(super) fn native_manifest() -> Value {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/pyramid-recon/manifest.json"
        ))
        .expect("native evidence manifest")
    }


    /// Adapt only native training RGB to the strict factorised reader wire contract.
    fn native_reader_value() -> Value {
        let evidence: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/pyramid-recon/factorised-glyphs.json"
        ))
        .expect("native glyph evidence");
        let mut reader = json!({
            "rank_roi": evidence["banks"]["rank"]["roi"],
            "suit_roi": evidence["banks"]["suit"]["roi"],
            "rank_max_distance": 24, "suit_max_distance": 8, "alignment": 1,
            "ranks": [], "suits": [],
        });


        for (component, output) in [("rank", "ranks"), ("suit", "suits")] {
            let bank = &evidence["banks"][component];
            let grouped: Vec<_> = bank["labels"]
                .as_array()
                .expect("labels")
                .iter()
                .map(|label| {
                    let variants: Vec<_> = bank["variants"]
                        .as_array()
                        .expect("variants")
                        .iter()
                        .filter(|variant| variant["label"] == *label)
                        .map(|variant| variant["samples"].clone())
                        .collect();
                    json!({"label": label, "variants": variants})
                })
                .collect();
            reader[output] = json!(grouped);
        }

        reader
    }


    /// Label conversion for retained test evidence, independent of classifier output.
    pub(super) fn labelled_card(value: &Value) -> Card {
        Card::new(
            Rank::ALL
                .into_iter()
                .find(|rank| Some(rank.as_str()) == value["rank"].as_str())
                .expect("labelled rank"),
            Suit::ALL
                .into_iter()
                .find(|suit| Some(suit.as_str()) == value["suit"].as_str())
                .expect("labelled suit"),
        )
    }


    /// Extract unchanged RGB from an original into the exact bounded patch schema.
    pub(super) fn native_patch(
        frame: &CapturedFrame,
        bounds: PixelRect,
        source_sha256: &str,
    ) -> Value {
        use std::fmt::Write;

        let view = frame.as_view().expect("native layout");
        let (right, bottom) = view.checked_bounds(bounds).expect("native patch bounds");
        let mut rgb = String::new();


        for row in bounds.y..bottom {
            for column in bounds.x..right {
                let pixel = view.pixel_rgb(column, row).expect("native pixel");
                write!(&mut rgb, "{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
                    .expect("RGB encoding");
            }
        }

        json!({
            "bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
            "samples": {"rgb": rgb, "mask": "1".repeat((bounds.width * bounds.height) as usize),
                "tolerance": 24, "source_sha256": source_sha256},
        })
    }


    /// Find Charlie's exact verified modal original independently of its file name.
    fn native_dialog_original() -> CapturedFrame {
        native_original_with_hash(
            "78da9587035ee2707ffacf4adaf493a2a55e1ed5c28712bd5765dca23b30a9e0",
        )
    }


    /// Locate a newly supplied original through retained exact-byte provenance.
    fn native_original_with_hash(hash: &str) -> CapturedFrame {
        let manifest = native_manifest();
        let original = manifest["originals"]
            .as_array()
            .expect("originals")
            .iter()
            .find(|source| source["sha256"] == hash)
            .expect("verified native original");
        native_original(original["id"].as_str().expect("original ID"))
    }


    /// Build the specific native modal signature without using variable card identities.
    fn native_dialog_value() -> Value {
        let dialog = native_dialog_original();
        let hash = "78da9587035ee2707ffacf4adaf493a2a55e1ed5c28712bd5765dca23b30a9e0";
        json!({
            "bounds": [531, 367, 858, 330],
            "panel": native_patch(&dialog, PixelRect::new(535, 400, 64, 64), hash),
            "title": native_patch(&dialog, PixelRect::new(730, 410, 170, 23), hash),
            "body": native_patch(&dialog, PixelRect::new(730, 475, 420, 9), hash),
            "cancel": native_patch(&dialog, PixelRect::new(1080, 617, 110, 28), hash),
            "ok": native_patch(&dialog, PixelRect::new(762, 612, 60, 40), hash),
            "ok_point": [792, 630],
            "backdrop": [
                native_patch(&dialog, PixelRect::new(1812, 168, 72, 44), hash),
                native_patch(&dialog, PixelRect::new(9, 46, 45, 40), hash),
            ],
        })
    }


    /// Assemble only actual native component samples into the strict complete profile.
    fn native_profile_value() -> Value {
        let manifest = native_manifest();
        let originals = manifest["originals"].as_array().expect("original list");
        let hash = |index: usize| {
            originals[index - 1]["sha256"]
                .as_str()
                .expect("original hash")
        };
        let initial = native_original("issue1-20");
        let enabled = native_original("issue1-01");
        let empty = native_original("issue1-06");
        let board = json!({
            "scene": [
                native_patch(&initial, PixelRect::new(1832, 130, 44, 30), hash(20)),
                native_patch(&initial, PixelRect::new(1812, 250, 70, 24), hash(20)),
                native_patch(&initial, PixelRect::new(1660, 50, 150, 24), hash(20)),
            ],
            "solver_off": native_patch(&initial, SOLVER_BOUNDS, hash(20)),
            "undo_all": native_patch(&enabled, UNDO_BOUNDS, hash(1)),
            "left_empty": native_patch(&empty, PixelRect::new(790, 732, 64, 64), hash(6)),
            "right_empty": native_patch(&initial, PixelRect::new(1022, 678, 64, 24), hash(20)),
        });
        let reader_value = native_reader_value();
        json!({
            "schema_version": 1, "profile_id": "classic-blue-1920-native-v1",
            "game": "pyramid", "artwork": "CLASSIC BLUE", "dimensions": [1920, 1080],
            "tableau_reader": reader_value.clone(), "stock_reader": reader_value,
            "board": board, "dialog": native_dialog_value(),
        })
    }


    /// Exercise the complete parser with the real calibrated board and modal evidence.
    fn native_vision() -> ReconVision {
        parse(&native_profile_value()).expect("complete native calibration profile")
    }


    /// Load the exact operator-selectable profile, not only the test-built calibration.
    fn bundled_native_vision() -> ReconVision {
        ReconVision::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-profile.json"),
        )
        .expect("bundled native runtime profile")
    }


    /// All 46 real observations pass; all 16 LEFT observations are held out of training.
    #[test]
    fn native_factorised_reader_recognises_training_and_held_out_left_cards() {
        let reader = CardReader::parse(&native_reader_value(), 140, 53).expect("native components");
        let manifest = native_manifest();
        let observations = manifest["labelled_card_observations"]
            .as_array()
            .expect("observations");
        let mut recognised = 0;
        let mut held_out = 0;


        for source in manifest["originals"].as_array().expect("originals") {
            let source_id = source["id"].as_str().expect("source ID");


            if !source_id.starts_with("issue1-") {
                continue;
            }

            let native = native_original(source_id);
            let view = native.as_view().expect("native view");


            for observation in observations
                .iter()
                .filter(|observation| observation["source_id"] == source["id"])
            {
                let origin = (
                    integer(&observation["origin"][0]).expect("x"),
                    integer(&observation["origin"][1]).expect("y"),
                );
                assert_eq!(
                    reader
                        .recognise(view, origin)
                        .expect("unique native glyph labels"),
                    Some(labelled_card(observation)),
                    "{}",
                    observation["id"]
                );
                recognised += 1;
                held_out += usize::from(observation["template_training"] == json!(false));
            }
        }

        assert_eq!(recognised, 46);
        assert_eq!(held_out, 16);
    }


    /// The actual initial frame yields exactly the manually labelled 28+LEFT inventory.
    #[test]
    fn native_initial_board_has_28_cards_left_queen_clubs_and_disabled_undo() {
        let vision = native_vision();
        let initial = native_original("issue1-20");
        let observed = vision
            .observe(&initial)
            .expect("native initial observation");
        let ReconScene::Board(board) = observed else {
            panic!("expected native initial board")
        };
        let expected = native_manifest();


        for (index, card) in expected["initial_board"]["tableau"]
            .as_array()
            .expect("tableau")
            .iter()
            .enumerate()
        {
            assert_eq!(
                board.tableau[index],
                labelled_card(card),
                "logical index {index}"
            );
        }

        assert_eq!(
            board.left,
            StockFace::Card(Card::new(Rank::Queen, Suit::Clubs))
        );
        assert_eq!(board.right, StockFace::Empty);
        assert_eq!(board.undo_all, None);
    }


    /// The supplied modal is recognised only by its entire same-frame signature.
    #[test]
    fn native_undo_modal_matches_all_components_and_refuses_erasure() {
        let original = native_dialog_original();


        for vision in [native_vision(), bundled_native_vision()] {
            assert_eq!(
                vision.observe(&original).expect("native Undo All modal"),
                ReconScene::UndoConfirmation {
                    ok: PixelPoint::new(792, 630)
                }
            );


            for pattern in &vision.dialog.signature {
                let mut damaged = original.clone();


                for row in pattern.bounds.y..pattern.bounds.bottom() {
                    for column in pattern.bounds.x..pattern.bounds.right() {
                        let offset = row as usize * damaged.stride + column as usize * 4;
                        damaged.pixels[offset..offset + 3].fill(127);
                    }
                }

                assert_ne!(
                    vision.observe(&damaged).expect("missing modal component"),
                    ReconScene::UndoConfirmation {
                        ok: PixelPoint::new(792, 630)
                    }
                );
            }
        }
    }


    /// The actual worker and later operator captures require the complete native modal signature.
    #[test]
    fn bundled_native_profile_recognises_live_undo_modal_without_borrowing_board_state() {
        let vision = bundled_native_vision();


        for source in [
            "gate3-undo-modal",
            "gate3-undo-modal-operator",
            "gate3-undo-modal-repeat",
        ] {
            let original = native_original(source);
            assert_eq!(
                vision
                    .observe(&original)
                    .expect("live native Undo All modal"),
                ReconScene::UndoConfirmation {
                    ok: PixelPoint::new(792, 630)
                }
            );
        }
        let baseline = native_original("gate3-undo-baseline");
        assert_eq!(
            vision.observe(&baseline).expect("live baseline"),
            vision
                .observe(&native_original("current-start"))
                .expect("reference baseline")
        );
    }


    /// Every current native modal component remains necessary, with exact inclusive channel caps.
    #[test]
    fn bundled_live_modal_requires_all_components_and_measured_channel_bounds() {
        let vision = bundled_native_vision();
        let baseline = native_dialog_original();
        let expected = ReconScene::UndoConfirmation {
            ok: PixelPoint::new(792, 630),
        };


        for (index, pattern) in vision.dialog.signature.iter().enumerate() {
            assert_eq!(pattern.tolerance, [4, 9, 10, 9, 8, 4, 4][index]);
            let selected = pattern
                .pixels
                .iter()
                .position(Option::is_some)
                .expect("selected native evidence");
            let mut witnesses = vec![selected];


            if (1..=4).contains(&index) {
                let ink = if index == 4 {
                    [0, 0, 0]
                } else {
                    [255, 255, 255]
                };
                witnesses.push(
                    pattern
                        .pixels
                        .iter()
                        .position(|pixel| *pixel == Some(ink))
                        .expect("retained ink"),
                );
                witnesses.push(
                    pattern
                        .pixels
                        .iter()
                        .position(|pixel| {
                            pixel.is_some_and(|pixel| {
                                pixel
                                    .iter()
                                    .zip(ink)
                                    .any(|(actual, expected)| actual.abs_diff(expected) >= 128)
                            })
                        })
                        .expect("retained negative space"),
                );
            }


            for selected in witnesses {
                let row = pattern.bounds.y + selected as u32 / pattern.bounds.width;
                let column = pattern.bounds.x + selected as u32 % pattern.bounds.width;
                let offset = row as usize * baseline.stride + column as usize * 4;


                for channel in 0..3 {
                    let reference = baseline.pixels[offset + channel];


                    for delta in [pattern.tolerance, pattern.tolerance + 1] {
                        let mut changed = baseline.clone();
                        changed.pixels[offset + channel] = reference
                            .checked_add(delta)
                            .unwrap_or_else(|| reference - delta);
                        assert_eq!(
                            vision.observe(&changed).expect("selected-channel boundary"),
                            if delta == pattern.tolerance {
                                expected.clone()
                            } else {
                                ReconScene::Unknown
                            }
                        );
                    }
                }
            }


            for source in [
                "gate3-undo-modal",
                "gate3-undo-modal-operator",
                "gate3-undo-modal-repeat",
            ] {
                let mut damaged = native_original(source);


                for row in pattern.bounds.y..pattern.bounds.bottom() {
                    for column in pattern.bounds.x..pattern.bounds.right() {
                        let offset = row as usize * damaged.stride + column as usize * 4;
                        damaged.pixels[offset..offset + 3].fill(127);
                    }
                }
                assert_eq!(
                    vision
                        .observe(&damaged)
                        .expect("missing native modal component"),
                    ReconScene::Unknown
                );
            }
        }
    }


    /// Reproduce calibration selection from the original alone, including its one-pixel halo.
    fn native_modal_stable_selection(
        frame: &CapturedFrame,
        bounds: PixelRect,
        black_ink: bool,
    ) -> Vec<bool> {
        assert!(
            bounds.x > 0
                && bounds.y > 0
                && bounds.right() < frame.width
                && bounds.bottom() < frame.height
        );
        let ink = if black_ink {
            [0, 0, 0]
        } else {
            [255, 255, 255]
        };
        let mut selection = Vec::new();


        for row in bounds.y..bounds.bottom() {
            for column in bounds.x..bounds.right() {
                let mut minimum = [255u8; 3];
                let mut maximum = [0u8; 3];


                for neighbour_row in row - 1..=row + 1 {
                    for neighbour_column in column - 1..=column + 1 {
                        let offset =
                            neighbour_row as usize * frame.stride + neighbour_column as usize * 4;


                        for channel in 0..3 {
                            minimum[channel] = minimum[channel].min(frame.pixels[offset + channel]);
                            maximum[channel] = maximum[channel].max(frame.pixels[offset + channel]);
                        }
                    }
                }
                let offset = row as usize * frame.stride + column as usize * 4;
                selection.push(
                    frame.pixels[offset..offset + 3] == ink
                        || maximum
                            .iter()
                            .zip(minimum)
                            .all(|(high, low)| high - low <= 16),
                );
            }
        }
        selection
    }


    /// Source-only masks preserve solid ink and quiet background rather than fitting live mismatches.
    #[test]
    fn bundled_modal_masks_reproduce_reference_only_selection_and_excluded_boundaries() {
        let vision = bundled_native_vision();
        let original = native_dialog_original();
        let expected = vision.observe(&original).expect("original modal");


        for (index, pattern) in vision.dialog.signature.iter().enumerate() {
            if !(1..=4).contains(&index) {
                assert!(pattern.pixels.iter().all(Option::is_some));
                continue;
            }
            let selected = native_modal_stable_selection(&original, pattern.bounds, index == 4);
            assert_eq!(
                selected.iter().filter(|selected| **selected).count(),
                [2598, 2383, 2333, 1758][index - 1]
            );
            assert_eq!(
                selected,
                pattern
                    .pixels
                    .iter()
                    .map(Option::is_some)
                    .collect::<Vec<_>>()
            );
            let excluded = selected
                .iter()
                .position(|selected| !selected)
                .expect("unstable transition excluded");
            let row = pattern.bounds.y + excluded as u32 / pattern.bounds.width;
            let column = pattern.bounds.x + excluded as u32 % pattern.bounds.width;
            let offset = row as usize * original.stride + column as usize * 4;
            let mut changed = original.clone();
            changed.pixels[offset..offset + 3].copy_from_slice(&[255, 0, 255]);
            assert_eq!(
                vision
                    .observe(&changed)
                    .expect("excluded transition mutation"),
                expected
            );
        }
    }


    /// Every measured unmasked foreground component and nearby negative space remain constraining.
    #[test]
    fn bundled_modal_masks_reject_individual_glyph_erasure_and_added_ink() {
        let vision = bundled_native_vision();
        let original = native_dialog_original();
        let components: [&[(u32, u32)]; 4] = [
            &[
                (5, 10),
                (23, 28),
                (36, 55),
                (61, 82),
                (88, 110),
                (128, 151),
                (158, 162),
                (170, 173),
            ],
            &[
                (2, 14),
                (17, 26),
                (30, 42),
                (45, 52),
                (63, 71),
                (75, 86),
                (91, 103),
                (114, 124),
                (128, 140),
            ],
            &[(7, 25), (29, 43), (49, 64), (69, 82), (86, 101), (106, 109)],
            &[(6, 29), (34, 52)],
        ];


        for (index, pattern) in vision.dialog.signature[1..=4].iter().enumerate() {
            let ink = if index == 3 {
                [0, 0, 0]
            } else {
                [255, 255, 255]
            };
            let background = if index == 3 {
                [240, 191, 48]
            } else {
                [32, 86, 161]
            };
            let mut negative_checks = 0;


            for &(first, last) in components[index] {
                let mut erased = original.clone();
                let mut removed = 0;
                let mut negative = None;


                for row in pattern.bounds.y..pattern.bounds.bottom() {
                    for column in pattern.bounds.x + first..=pattern.bounds.x + last {
                        let sample = ((row - pattern.bounds.y) * pattern.bounds.width + column
                            - pattern.bounds.x) as usize;
                        let offset = row as usize * original.stride + column as usize * 4;


                        if pattern.pixels[sample] == Some(ink) {
                            erased.pixels[offset..offset + 3].copy_from_slice(&background);
                            removed += 1;
                        } else if pattern.pixels[sample].is_some_and(|pixel| {
                            pixel
                                .iter()
                                .zip(ink)
                                .any(|(actual, expected)| actual.abs_diff(expected) >= 128)
                        }) {
                            negative = Some(offset);
                        }
                    }
                }
                assert!(
                    removed > 0,
                    "each unmasked foreground component retains ink: component {first}..{last}"
                );
                assert_eq!(
                    vision.observe(&erased).expect("derived component erasure"),
                    ReconScene::Unknown
                );


                if let Some(offset) = negative {
                    let mut added = original.clone();
                    added.pixels[offset..offset + 3].copy_from_slice(&ink);
                    assert_eq!(
                        vision.observe(&added).expect("derived added stroke"),
                        ReconScene::Unknown
                    );
                    negative_checks += 1;
                }
            }
            assert!(
                negative_checks > 0,
                "nearby negative space is retained for every text patch"
            );
        }
    }


    /// Counter filling, an added glyph bridge and a removed thin stroke retain selected witnesses.
    #[test]
    fn bundled_modal_masks_reject_counter_bridge_and_thin_stroke_mutations() {
        let vision = bundled_native_vision();
        let original = native_dialog_original();
        let mutations = [
            (4, PixelRect::new(17, 10, 2, 15), false),
            (1, PixelRect::new(145, 10, 16, 2), false),
            (1, PixelRect::new(170, 4, 4, 15), true),
        ];


        for (component, region, erase) in mutations {
            let pattern = &vision.dialog.signature[component];
            let ink = if component == 4 {
                [0, 0, 0]
            } else {
                [255, 255, 255]
            };
            let mut changed = original.clone();
            let mut selected_witnesses = 0;
            assert!(
                region.right() <= pattern.bounds.width && region.bottom() <= pattern.bounds.height
            );


            for row in region.y..region.bottom() {
                for column in region.x..region.right() {
                    let sample = (row * pattern.bounds.width + column) as usize;
                    let offset = (pattern.bounds.y + row) as usize * original.stride
                        + (pattern.bounds.x + column) as usize * 4;
                    let pixel = &original.pixels[offset..offset + 3];
                    let is_ink = pixel == ink;
                    let is_negative = pixel
                        .iter()
                        .zip(ink)
                        .any(|(actual, expected)| actual.abs_diff(expected) >= 128);


                    if (erase && is_ink) || (!erase && is_negative) {
                        changed.pixels[offset..offset + 3].copy_from_slice(if erase {
                            &[23, 77, 149]
                        } else {
                            &ink
                        });
                        selected_witnesses += usize::from(pattern.pixels[sample].is_some());
                    }
                }
            }
            assert!(
                selected_witnesses > 0,
                "selected evidence must reject the intended stroke change"
            );
            assert_eq!(
                vision
                    .observe(&changed)
                    .expect("labelled synthetic stroke mutation"),
                ReconScene::Unknown
            );
        }
    }


    /// Wrong text, generic OK-only evidence, obscured panels and other native rewards refuse confirmation.
    #[test]
    fn bundled_live_modal_refuses_text_substitutions_and_other_native_dialogs() {
        let vision = bundled_native_vision();
        let original = native_original("gate3-undo-modal");


        for pattern in &vision.dialog.signature[1..=4] {
            let mut substituted = original.clone();


            for row in pattern.bounds.y..pattern.bounds.bottom() {
                for column in pattern.bounds.x..pattern.bounds.right() {
                    let source_column =
                        pattern.bounds.x + (column - pattern.bounds.x + 3) % pattern.bounds.width;
                    let source_offset = row as usize * original.stride + source_column as usize * 4;
                    let offset = row as usize * substituted.stride + column as usize * 4;
                    substituted.pixels[offset..offset + 3]
                        .copy_from_slice(&original.pixels[source_offset..source_offset + 3]);
                }
            }
            assert_eq!(
                vision
                    .observe(&substituted)
                    .expect("derived misaligned text"),
                ReconScene::Unknown
            );
        }
        let mut gold_only = native_original("current-start");
        let ok = &vision.dialog.signature[4];


        for row in ok.bounds.y..ok.bounds.bottom() {
            for column in ok.bounds.x..ok.bounds.right() {
                let offset = row as usize * gold_only.stride + column as usize * 4;
                gold_only.pixels[offset..offset + 3]
                    .copy_from_slice(&original.pixels[offset..offset + 3]);
            }
        }
        assert!(!matches!(
            vision.observe(&gold_only),
            Ok(ReconScene::UndoConfirmation { .. })
        ));


        for filename in [
            "freecell-FC22.png",
            "freecell-FC23.png",
            "spider-SP21.png",
            "spider-SP22.png",
            "tripeaks-TP01.png",
            "tripeaks-TP02.png",
            "tripeaks-TP03.png",
        ] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(filename);
            let bytes = std::fs::read(path).expect("existing native regression fixture");
            let frame = crate::capture::decode_png(&bytes).expect("native regression PNG");
            assert!(
                !matches!(
                    vision.observe(&frame),
                    Ok(ReconScene::UndoConfirmation { .. })
                ),
                "{filename}"
            );
        }
    }


    /// Changes to covered tableau/stock identity regions do not define this modal.
    #[test]
    fn native_undo_modal_does_not_depend_on_backdrop_card_identities() {
        let mut changed = native_dialog_original();


        for bounds in [
            PixelRect::new(880, 110, 150, 210),
            PixelRect::new(759, 700, 402, 164),
            PixelRect::new(531, 367, 4, 4),
        ] {
            for row in bounds.y..bounds.bottom() {
                for column in bounds.x..bounds.right() {
                    let offset = row as usize * changed.stride + column as usize * 4;
                    changed.pixels[offset..offset + 3].copy_from_slice(&[17, 19, 21]);
                }
            }
        }

        for vision in [native_vision(), bundled_native_vision()] {
            assert_eq!(
                vision
                    .observe(&changed)
                    .expect("changed background identities"),
                ReconScene::UndoConfirmation {
                    ok: PixelPoint::new(792, 630)
                }
            );
        }
    }


    /// The actual bundled runtime file handles the newly supplied start/end/modal states.
    ///
    /// These unordered images do not prove the 24 transitions or restoration.
    #[test]
    fn bundled_native_profile_recognises_current_start_exhaustion_and_modal() {
        let profile_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-profile.json");
        let vision = ReconVision::load(&profile_path).expect("load actual bundled native profile");
        let start = native_original_with_hash(
            "4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35",
        );
        let exhausted = native_original_with_hash(
            "b352d1645b15b2cb58d2a7a64e8b25a845ca91c955e3d66861cb76be1d352ef6",
        );
        let ReconScene::Board(start) = vision.observe(&start).expect("current native start") else {
            panic!("current native start must be a supported board");
        };
        let ReconScene::Board(exhausted) = vision
            .observe(&exhausted)
            .expect("current native exhaustion")
        else {
            panic!("current native exhaustion must remain a supported board");
        };
        assert_eq!(start.tableau, exhausted.tableau);
        let expected = [
            (Rank::Two, Suit::Diamonds),
            (Rank::Nine, Suit::Spades),
            (Rank::Ace, Suit::Clubs),
            (Rank::Queen, Suit::Clubs),
            (Rank::Four, Suit::Clubs),
            (Rank::Five, Suit::Hearts),
            (Rank::King, Suit::Clubs),
            (Rank::Ten, Suit::Hearts),
            (Rank::Two, Suit::Hearts),
            (Rank::King, Suit::Hearts),
            (Rank::Eight, Suit::Clubs),
            (Rank::King, Suit::Spades),
            (Rank::Ten, Suit::Diamonds),
            (Rank::Six, Suit::Clubs),
            (Rank::Four, Suit::Hearts),
            (Rank::Three, Suit::Diamonds),
            (Rank::Queen, Suit::Hearts),
            (Rank::Nine, Suit::Hearts),
            (Rank::Six, Suit::Hearts),
            (Rank::Three, Suit::Clubs),
            (Rank::Eight, Suit::Spades),
            (Rank::Six, Suit::Spades),
            (Rank::Ace, Suit::Spades),
            (Rank::Two, Suit::Clubs),
            (Rank::Queen, Suit::Spades),
            (Rank::Eight, Suit::Diamonds),
            (Rank::Four, Suit::Diamonds),
            (Rank::Four, Suit::Spades),
        ]
        .map(|(rank, suit)| Card::new(rank, suit));
        assert_eq!(start.tableau, expected);
        assert_eq!(
            start.left,
            StockFace::Card(Card::new(Rank::King, Suit::Diamonds))
        );
        assert_eq!(start.right, StockFace::Empty);
        assert_eq!(start.undo_all, None);
        assert_eq!(exhausted.left, StockFace::Empty);
        assert_eq!(
            exhausted.right,
            StockFace::Card(Card::new(Rank::Queen, Suit::Diamonds))
        );
        assert_eq!(exhausted.undo_all, Some(PixelPoint::new(1320, 978)));
        assert_eq!(
            vision
                .observe(&native_dialog_original())
                .expect("actual modal"),
            ReconScene::UndoConfirmation {
                ok: PixelPoint::new(792, 630)
            }
        );
    }


    /// Both actual Gate 3 captures retain their independently observed board and pile states.
    #[test]
    fn bundled_native_profile_recognises_gate3_start_and_exhausted_captures() {
        let vision = bundled_native_vision();
        let reference = vision
            .observe(&native_original("current-start"))
            .expect("calibration start");
        let observed = vision
            .observe(&native_original("gate3-start-refusal"))
            .expect("live starting frame");
        assert_eq!(observed, reference);
        let reference = vision
            .observe(&native_original("current-exhausted"))
            .expect("calibration exhaustion");
        let observed = vision
            .observe(&native_original("gate3-exhausted-refusal"))
            .expect("live exhausted frame");
        assert_eq!(observed, reference);
    }


    /// Each full rule-chart entry and the stock arrow remain mandatory.
    #[test]
    fn bundled_native_scene_refuses_erased_components_after_row_boundary_correction() {
        let vision = bundled_native_vision();
        let original = native_original("gate3-start-refusal");
        assert!(matches!(
            vision.observe(&original).expect("live starting frame"),
            ReconScene::Board(_)
        ));


        for bounds in [
            PixelRect::new(1838, 132, 23, 27),
            PixelRect::new(1820, 286, 23, 27),
            PixelRect::new(1858, 286, 23, 27),
            vision.board.scene[2].bounds,
        ] {
            let mut damaged = original.clone();


            for row in bounds.y..bounds.bottom() {
                for column in bounds.x..bounds.right() {
                    let offset = row as usize * damaged.stride + column as usize * 4;
                    damaged.pixels[offset..offset + 3].copy_from_slice(&[24, 80, 54]);
                }
            }
            assert_eq!(
                vision.observe(&damaged).expect("erased native component"),
                ReconScene::Unknown
            );
        }
    }


    /// Removed next-row fragments are independent; retained edge pixels and RGB caps are not.
    #[test]
    fn bundled_native_chart_boundaries_preserve_exact_selected_pixel_limits() {
        let vision = bundled_native_vision();
        let original = native_original("current-start");
        let expected = vision.observe(&original).expect("native baseline");
        let excluded = [
            PixelRect::new(1810, 172, 78, 10),
            PixelRect::new(1815, 323, 67, 12),
        ];


        for selection in [1, 2, 3] {
            let mut changed = original.clone();


            for (index, bounds) in excluded.iter().enumerate() {
                if selection & (1 << index) == 0 {
                    continue;
                }


                for row in bounds.y..bounds.bottom() {
                    for column in bounds.x..bounds.right() {
                        let offset = row as usize * changed.stride + column as usize * 4;
                        changed.pixels[offset..offset + 3].copy_from_slice(&[11, 13, 17]);
                    }
                }
            }
            assert_eq!(
                vision
                    .observe(&changed)
                    .expect("excluded neighbouring text"),
                expected
            );
        }


        for (index, pattern) in vision.board.scene[..2].iter().enumerate() {
            assert_eq!(pattern.tolerance, [6, 8][index]);
            assert!(pattern.pixels.iter().all(Option::is_some));
            let row = pattern.bounds.bottom() - 1;
            let offset = row as usize * original.stride + pattern.bounds.x as usize * 4;
            let reference = original.pixels[offset];


            for delta in [pattern.tolerance, pattern.tolerance + 1] {
                let mut changed = original.clone();
                changed.pixels[offset] = reference
                    .checked_add(delta)
                    .unwrap_or_else(|| reference - delta);
                let observed = vision.observe(&changed).expect("bounded RGB mutation");


                if delta == pattern.tolerance {
                    assert_eq!(observed, expected);
                } else {
                    assert_eq!(observed, ReconScene::Unknown);
                }
            }
        }
    }


    /// Native Undo readiness remains distinct across retained Solver toolbar appearances.
    #[test]
    fn native_undo_readiness_is_distinct_from_solver_appearance() {
        for vision in [native_vision(), bundled_native_vision()] {
            for number in 1..=19 {
                let native = native_original(&format!("issue1-{number:02}"));
                let view = native.as_view().expect("view");
                assert_eq!(
                    vision
                        .board
                        .undo_all
                        .matches(view, (0, 0))
                        .expect("native icon readiness"),
                    number != 19,
                    "Undo icon image {number}; image 19 has visibly disabled Undo All"
                );
            }

            let initial = native_original("issue1-20");
            assert!(
                !vision
                    .board
                    .undo_all
                    .matches(initial.as_view().expect("view"), (0, 0))
                    .expect("disabled icon")
            );
            assert!(matches!(
                vision.observe(&initial).expect("historical initial board"),
                ReconScene::Board(_)
            ));
        }
    }


    /// Real empty symbols do not become cards and require their measured positive patches.
    #[test]
    fn native_empty_pile_evidence_is_positive_without_granting_scene_authority() {
        let vision = native_vision();


        for number in 6..=9 {
            let native = native_original(&format!("issue1-{number:02}"));
            assert_eq!(
                vision
                    .stock_face(
                        native.as_view().expect("view"),
                        LEFT_BOUNDS,
                        &vision.board.left_empty
                    )
                    .expect("positive empty LEFT"),
                StockFace::Empty
            );
        }

        let initial = native_original("issue1-20");
        assert_eq!(
            vision
                .stock_face(
                    initial.as_view().expect("view"),
                    RIGHT_BOUNDS,
                    &vision.board.right_empty
                )
                .expect("positive empty RIGHT"),
            StockFace::Empty
        );
    }


    /// Erased rank or suit and wrong-colour glyphs refuse; native originals stay unchanged.
    #[test]
    fn native_glyph_mutations_do_not_infer_missing_or_mixed_cards() {
        let reader = CardReader::parse(&native_reader_value(), 140, 53).expect("native reader");
        let original = native_original("issue1-20");
        let origin = TABLEAU_ORIGINS[0];


        for roi in [PixelRect::new(1, 5, 30, 26), PixelRect::new(4, 29, 22, 20)] {
            let mut erased = original.clone();


            for row in origin.1 + roi.y..origin.1 + roi.bottom() {
                for column in origin.0 + roi.x..origin.0 + roi.right() {
                    let offset = row as usize * erased.stride + column as usize * 4;
                    erased.pixels[offset..offset + 3].fill(255);
                }
            }

            assert_eq!(
                reader
                    .recognise(erased.as_view().expect("mutated view"), origin)
                    .expect("unknown glyph"),
                None
            );
        }

        let mut mixed = original;


        for row in origin.1 + 5..origin.1 + 31 {
            for column in origin.0 + 1..origin.0 + 31 {
                let offset = row as usize * mixed.stride + column as usize * 4;
                let pixel = &mut mixed.pixels[offset..offset + 3];


                if pixel.iter().any(|channel| *channel < 128) {
                    pixel.copy_from_slice(&[220, 0, 0]);
                }
            }
        }

        assert_eq!(
            reader
                .recognise(mixed.as_view().expect("mutated view"), origin)
                .expect("mixed colour"),
            None
        );
    }


    /// Real glyph coverage cannot make an incomplete dialog/profile executable.
    #[test]
    fn native_evidence_without_confirmation_is_not_a_runtime_profile() {
        let evidence = include_bytes!("../../tests/fixtures/pyramid-recon/factorised-glyphs.json");
        assert!(ReconVision::from_bytes(evidence).is_err());
        let mut missing_dialog = profile_value();
        missing_dialog["tableau_reader"] = native_reader_value();
        missing_dialog["stock_reader"] = native_reader_value();
        missing_dialog["dialog"] = Value::Null;
        assert!(parse(&missing_dialog).is_err());
    }


    /// Factorised masks cannot remove negative space, omit labels or broaden the limits.
    #[test]
    fn factorised_profile_bounds_and_labels_are_strict() {
        let original = native_reader_value();


        for (field, invalid) in [
            ("alignment", json!(2)),
            ("rank_max_distance", json!(25)),
            ("suit_max_distance", json!(9)),
        ] {
            let mut modified = original.clone();
            modified[field] = invalid;
            assert!(CardReader::parse(&modified, 140, 53).is_err());
        }

        let mut missing = original.clone();
        missing["ranks"].as_array_mut().expect("ranks").pop();
        assert!(CardReader::parse(&missing, 140, 53).is_err());
        let mut duplicate = original.clone();
        duplicate["suits"][1]["label"] = duplicate["suits"][0]["label"].clone();
        assert!(CardReader::parse(&duplicate, 140, 53).is_err());
        let mut sparse = original.clone();
        sparse["ranks"][0]["variants"][0]["mask"] = json!("0".repeat(648));
        assert!(CardReader::parse(&sparse, 140, 53).is_err());
        let mut indistinguishable = original;
        indistinguishable["ranks"][1]["variants"] =
            indistinguishable["ranks"][0]["variants"].clone();
        assert!(CardReader::parse(&indistinguishable, 140, 53).is_err());
    }
}


#[cfg(test)]
#[path = "vision/native_v6_tests.rs"]
mod native_v6_tests;


#[cfg(test)]
#[path = "vision/native_v7_tests.rs"]
mod native_v7_tests;


#[cfg(test)]
mod route_tests;


#[cfg(test)]
mod daily_challenge_tests;


#[cfg(test)]
#[path = "vision/native_v8_tests.rs"]
mod native_v8_tests;


#[cfg(test)]
#[path = "vision/native_v9_tests.rs"]
mod native_v9_tests;


#[cfg(test)]
mod native_v10_tests;


#[cfg(test)]
mod native_v11_tests;


#[cfg(test)]
mod native_v12_tests;


#[cfg(test)]
#[path = "vision/native_v13_tests.rs"]
mod native_v13_tests;


#[cfg(test)]
#[path = "vision/native_v14_tests.rs"]
mod native_v14_tests;


#[cfg(test)]
mod native_v15_tests;


#[cfg(test)]
mod native_v16_tests;
