#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Bounded (`<= 65,536` bytes), strict-schema TriPeaks deal JSON codec and atomic no-overwrite publisher.

use std::{
    collections::BTreeSet,
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};

use serde_json::{Map, Number, Value};
use xsar::games::cards::{Card as XsarCard, Rank, Suit};

use crate::recon::{Card, CardSlot, Objective, SlotId};

use super::{
    DEAL_SLOT_COUNT, INITIAL_WASTE_SLOT, MAX_ROUTE_MOVES, ModelError, TriPeaksMove,
    replay::{ReplayError, verify_move_sequence},
    rules::tableau_slot_id,
    validate_slot_table,
};

/// Active TriPeaks JSON deal schema version.
pub const SCHEMA_VERSION: u64 = 1;

/// Closed game identity string required inside exported TriPeaks JSON deals.
pub const GAME_IDENTIFIER: &str = "tripeaks";

/// Hard input and output JSON size cap (`65,536` bytes).
pub const MAX_DOCUMENT_BYTES: usize = 65_536;

/// Maximum JSON nesting depth permitted during duplicate-key pre-validation.
const MAX_JSON_DEPTH: usize = 16;

/// Maximum same-directory private temporary filename collision attempts.
const TEMPORARY_ATTEMPTS: usize = 64;

/// Process-unique non-wrapping counter used to name private same-directory temporary files.
static TEMPORARY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// Origin state captured alongside a TriPeaks 52-slot deal document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DealProvenance {
    /// Deal slots returned after restoring back to the initial `T = 0` root state.
    RestoredAtRoot,
    /// Challenge or board goal completed live during a probing sequence before `Undo All`.
    CompletedDuringRecon,
}

impl DealProvenance {
    /// Return the stable snake-case JSON wire string.
    const fn as_str(self) -> &'static str {
        match self {
            Self::RestoredAtRoot => "restored_at_root",
            Self::CompletedDuringRecon => "completed_during_recon",
        }
    }

    /// Parse the stable snake-case JSON wire string.
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "restored_at_root" => Some(Self::RestoredAtRoot),
            "completed_during_recon" => Some(Self::CompletedDuringRecon),
            _ => None,
        }
    }
}

/// Immutable 52-slot TriPeaks deal document for JSON persistence and offline route verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriPeaksDealDocument {
    /// Root vs mid-probe completion provenance.
    pub provenance: DealProvenance,
    /// Number of Undo-All probe cycles executed during reconnaissance.
    pub probes_used: usize,
    /// Bound Challenge or Classic solving objective.
    pub objective: Objective,
    /// Fixed `52`-slot deal table (`0..28` tableau, `28` initial waste, `29..52` stock `1..=23`).
    pub slots: [CardSlot; DEAL_SLOT_COUNT],
    /// Optional verified route executable from `T = 0`.
    pub planned_route: Option<Vec<TriPeaksMove>>,
}

impl TriPeaksDealDocument {
    /// Validate slot uniqueness, objective bounds, and any attached route against this document.
    ///
    /// # Errors
    ///
    /// Returns [`JsonError`] on invalid slots, invalid rank bounds, oversized route, or replay failure.
    pub fn validate(&self) -> Result<(), JsonError> {
        validate_slot_table(&self.slots)?;
        if let Objective::ClearRankQuota { rank, .. } = self.objective
            && rank >= 13
        {
            return Err(JsonError::InvalidObjective(
                "ClearRankQuota rank index must be in 0..13",
            ));
        }
        if let Some(route) = &self.planned_route {
            if route.len() > MAX_ROUTE_MOVES {
                return Err(JsonError::Replay(ReplayError::RouteTooLong {
                    actual: route.len(),
                }));
            }
            for &mv in route {
                if let TriPeaksMove::TapTableau { row, column } = mv
                    && tableau_slot_id(row, column).is_none()
                {
                    return Err(JsonError::Model(ModelError::InvalidTableauCoord {
                        row,
                        column,
                    }));
                }
            }
            if self.provenance == DealProvenance::RestoredAtRoot
                || matches!(
                    self.slots[usize::from(INITIAL_WASTE_SLOT.0)],
                    CardSlot::Known(_)
                )
            {
                verify_move_sequence(&self.slots, route)?;
            }
        }
        Ok(())
    }
}

/// Errors returned while decoding, validating, or serialising a [`TriPeaksDealDocument`].
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    /// Input exceeds the `65,536`-byte safety cap.
    #[error("TriPeaks JSON exceeds {MAX_DOCUMENT_BYTES}-byte input limit")]
    InputTooLarge,
    /// Serialised JSON output exceeds the `65,536`-byte safety cap.
    #[error("TriPeaks JSON exceeds {MAX_DOCUMENT_BYTES}-byte output limit")]
    OutputTooLarge,
    /// Malformed UTF-8 or JSON syntax error at a source line/column.
    #[error("malformed TriPeaks JSON at {line}:{column}")]
    Malformed {
        /// One-based line number.
        line: usize,
        /// One-based column number.
        column: usize,
    },
    /// Same object key appeared twice in the same JSON object.
    #[error("duplicate JSON object key {key:?}")]
    DuplicateKey {
        /// Duplicated key string.
        key: String,
    },
    /// Required JSON field missing from an object.
    #[error("missing required JSON field {field:?}")]
    MissingField {
        /// Missing key name.
        field: &'static str,
    },
    /// Unrecognised JSON field present on a strict schema object.
    #[error("unexpected JSON field {field:?}")]
    UnexpectedField {
        /// Unexpected key name.
        field: String,
    },
    /// Unsupported schema version, game identity, enum string, or field type.
    #[error("unsupported TriPeaks JSON schema or value: {0}")]
    Unsupported(&'static str),
    /// Invalid objective specification.
    #[error("invalid TriPeaks objective: {0}")]
    InvalidObjective(&'static str),
    /// Card code could not be parsed or converted cleanly.
    #[error("invalid TriPeaks card code {code:?}")]
    InvalidCardCode {
        /// Unrecognised card code string.
        code: String,
    },
    /// Slot-table model invariant violated.
    #[error(transparent)]
    Model(#[from] ModelError),
    /// Attached route replay invariant violated.
    #[error(transparent)]
    Replay(#[from] ReplayError),
    /// Underlying stream read failure.
    #[error("failed reading TriPeaks JSON: {0}")]
    Io(#[from] io::Error),
}

/// Observed state of a target file inspected after an uncertain or collided atomic link attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DestinationState {
    /// Destination path does not exist.
    Absent,
    /// Destination already holds byte-for-byte identical contents to what we tried to publish.
    MatchingBytesPresent,
    /// Destination holds different existing contents (preserved untouched by no-overwrite link).
    PreExistingDifferentBytes,
    /// Filesystem inspection after link failure itself failed.
    InspectionFailed(io::ErrorKind),
}

/// Stage inside atomic no-overwrite publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishStage {
    /// Open parent directory handle for durable sync.
    OpenDirectory,
    /// Create same-directory private `0600` temp file with `create_new`.
    CreateTemporary,
    /// Write complete JSON bytes to temp file.
    WriteTemporary,
    /// Flush temp file contents and metadata via `sync_all`.
    SyncTemporary,
    /// Atomically hard-link temp file to destination without overwriting.
    LinkDestination,
    /// Remove temp file or sync parent directory after successful link.
    FinalizeDirectory,
}

impl fmt::Display for PublishStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OpenDirectory => "open parent directory",
            Self::CreateTemporary => "create private temporary file",
            Self::WriteTemporary => "write private temporary file",
            Self::SyncTemporary => "sync private temporary file",
            Self::LinkDestination => "link destination without replacement",
            Self::FinalizeDirectory => "cleanup or sync directory after link",
        })
    }
}

/// Errors produced by [`publish_to_path`], preserving destination inspection after I/O failure.
#[derive(Debug, thiserror::Error)]
pub enum PublishDealError {
    /// Document validation or bounded serialisation failed before any filesystem write.
    #[error("TriPeaks deal JSON validation/encoding failed: {0}")]
    Encode(#[from] JsonError),
    /// Target destination path is empty, contains NUL, or lacks a regular final filename.
    #[error("invalid destination path for atomic JSON publication")]
    InvalidPath,
    /// Atomic `0600` link publication is supported only on Unix targets.
    #[error("atomic JSON deal publication requires a Unix platform")]
    UnsupportedPlatform,
    /// I/O error before destination link attempt.
    #[error("JSON not published during {stage}: {source}")]
    IoBeforeLink {
        /// Failing pre-link stage.
        stage: PublishStage,
        /// Original filesystem error.
        source: io::Error,
        /// Optional error removing temporary file cleanly.
        cleanup_error: Option<io::Error>,
    },
    /// Destination `hard_link` returned an error; includes immediate destination inspection.
    #[error("JSON destination link failed ({destination_state:?}): {source}")]
    LinkFailed {
        /// Original `hard_link` failure (e.g. `AlreadyExists`).
        source: io::Error,
        /// Destination status inspected immediately after `hard_link` failed.
        destination_state: DestinationState,
        /// Optional error removing temporary file cleanly.
        cleanup_error: Option<io::Error>,
    },
    /// Destination link succeeded, but cleanup or parent directory sync reported an error.
    #[error("JSON published to destination, but post-link cleanup/sync failed: {detail}")]
    PostLinkWarning {
        /// Cleanup/sync diagnostic summary.
        detail: String,
    },
}

/// Format canonical [`Card`] as portable rank+suit shorthand (e.g. `"AS"`, `"10H"`).
pub(crate) fn format_card_code(card: Card) -> Result<String, JsonError> {
    let xc = XsarCard::try_from(card).map_err(|_| {
        JsonError::Model(ModelError::InvalidCardId {
            slot: SlotId(0),
            card_id: card.0,
        })
    })?;
    let suit_ch = match xc.suit() {
        Suit::Clubs => 'C',
        Suit::Diamonds => 'D',
        Suit::Hearts => 'H',
        Suit::Spades => 'S',
    };
    Ok(format!("{}{suit_ch}", xc.rank().as_str()))
}

/// Parse portable rank+suit shorthand (`"AS"`, `"10H"`, etc.) into canonical [`Card`].
pub(crate) fn parse_card_code(code: &str) -> Result<Card, JsonError> {
    if code.len() < 2 || code.len() > 3 || !code.is_ascii() {
        return Err(JsonError::InvalidCardCode {
            code: code.to_owned(),
        });
    }
    let (rank_str, suit_str) = code.split_at(code.len() - 1);
    let rank = parse_rank_str(rank_str).ok_or_else(|| JsonError::InvalidCardCode {
        code: code.to_owned(),
    })?;
    let suit = match suit_str {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => {
            return Err(JsonError::InvalidCardCode {
                code: code.to_owned(),
            });
        }
    };
    Card::try_from(XsarCard::new(rank, suit)).map_err(|_| JsonError::InvalidCardCode {
        code: code.to_owned(),
    })
}

/// Parse portable rank shorthand (`"A"`, `"2"`..`"10"`, `"J"`, `"Q"`, `"K"`).
fn parse_rank_str(rank_str: &str) -> Option<Rank> {
    Rank::ALL.into_iter().find(|rank| rank.as_str() == rank_str)
}

/// Compute 1-based `(line, column)` at byte offset `pos` inside valid UTF-8 `text`.
fn line_col_at(text: &str, pos: usize) -> (usize, usize) {
    let mut line = 1_usize;
    let mut col = 1_usize;
    for byte in text.as_bytes()[..pos.min(text.len())].iter() {
        if *byte == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Lightweight strict JSON pre-parser rejecting duplicate keys at any nesting depth before `Value`.
struct DuplicateKeyScanner<'a> {
    /// Full UTF-8 source slice.
    src: &'a str,
    /// Raw byte view for ASCII punctuation scanning.
    bytes: &'a [u8],
    /// Current zero-based byte cursor.
    pos: usize,
}

impl<'a> DuplicateKeyScanner<'a> {
    /// Create a scanner over UTF-8 source text.
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    /// Construct a `JsonError::Malformed` at `self.pos`.
    fn malformed_here(&self) -> JsonError {
        let (line, column) = line_col_at(self.src, self.pos);
        JsonError::Malformed { line, column }
    }

    /// Advance past ASCII JSON whitespace (`0x20`, `\t`, `\n`, `\r`).
    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Scan and return one unescaped string token starting at `"`.
    fn scan_string(&mut self) -> Result<String, JsonError> {
        let start = self.pos;
        if self.bytes.get(self.pos) != Some(&b'"') {
            return Err(self.malformed_here());
        }
        self.pos += 1;
        let mut escaped = false;
        while let Some(&b) = self.bytes.get(self.pos) {
            self.pos += 1;
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                let raw = &self.src[start..self.pos];
                return serde_json::from_str::<String>(raw).map_err(|err| JsonError::Malformed {
                    line: err.line(),
                    column: err.column(),
                });
            }
        }
        Err(self.malformed_here())
    }

    /// Scan one complete JSON value and reject duplicate keys in every nested object.
    fn scan_value(&mut self, depth: usize) -> Result<(), JsonError> {
        if depth > MAX_JSON_DEPTH {
            return Err(JsonError::Unsupported("JSON nesting depth exceeds limit"));
        }
        self.skip_ws();
        let Some(&b) = self.bytes.get(self.pos) else {
            return Err(self.malformed_here());
        };
        match b {
            b'"' => {
                let _ = self.scan_string()?;
                Ok(())
            }
            b'{' => {
                self.pos += 1;
                self.skip_ws();
                if self.bytes.get(self.pos) == Some(&b'}') {
                    self.pos += 1;
                    return Ok(());
                }
                let mut seen_keys = BTreeSet::new();
                loop {
                    self.skip_ws();
                    let key = self.scan_string()?;
                    if !seen_keys.insert(key.clone()) {
                        return Err(JsonError::DuplicateKey { key });
                    }
                    self.skip_ws();
                    if self.bytes.get(self.pos) != Some(&b':') {
                        return Err(self.malformed_here());
                    }
                    self.pos += 1;
                    self.scan_value(depth + 1)?;
                    self.skip_ws();
                    match self.bytes.get(self.pos) {
                        Some(&b',') => self.pos += 1,
                        Some(&b'}') => {
                            self.pos += 1;
                            return Ok(());
                        }
                        _ => return Err(self.malformed_here()),
                    }
                }
            }
            b'[' => {
                self.pos += 1;
                self.skip_ws();
                if self.bytes.get(self.pos) == Some(&b']') {
                    self.pos += 1;
                    return Ok(());
                }
                loop {
                    self.scan_value(depth + 1)?;
                    self.skip_ws();
                    match self.bytes.get(self.pos) {
                        Some(&b',') => self.pos += 1,
                        Some(&b']') => {
                            self.pos += 1;
                            return Ok(());
                        }
                        _ => return Err(self.malformed_here()),
                    }
                }
            }
            b't' | b'f' | b'n' | b'-' | b'0'..=b'9' => {
                while let Some(&ch) = self.bytes.get(self.pos) {
                    if matches!(ch, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r') {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(())
            }
            _ => Err(self.malformed_here()),
        }
    }
}

/// Extract an exact required field from a JSON object map.
fn required_field<'a>(
    map: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a Value, JsonError> {
    map.get(field).ok_or(JsonError::MissingField { field })
}

/// Ensure no unrecognised keys exist on a strict JSON object map.
fn reject_unknown_fields(map: &Map<String, Value>, allowed: &[&str]) -> Result<(), JsonError> {
    for key in map.keys() {
        if !allowed.iter().any(|&expected| expected == key) {
            return Err(JsonError::UnexpectedField { field: key.clone() });
        }
    }
    Ok(())
}

/// Require an exact unsigned `u64` integer from a JSON value.
fn as_strict_u64(value: &Value, context: &'static str) -> Result<u64, JsonError> {
    value.as_u64().ok_or(JsonError::Unsupported(context))
}

/// Parse one [`Objective`] from its strict JSON representation.
fn decode_objective(value: &Value) -> Result<Objective, JsonError> {
    let map = value
        .as_object()
        .ok_or(JsonError::Unsupported("objective must be a JSON object"))?;
    let kind = required_field(map, "kind")?
        .as_str()
        .ok_or(JsonError::Unsupported("objective.kind must be a string"))?;
    match kind {
        "clear_board" => {
            reject_unknown_fields(map, &["kind"])?;
            Ok(Objective::ClearBoard)
        }
        "clear_rank_quota" => {
            reject_unknown_fields(map, &["kind", "rank", "remaining"])?;
            let rank_str = required_field(map, "rank")?
                .as_str()
                .ok_or(JsonError::InvalidObjective("rank must be a string"))?;
            let rank_enum = parse_rank_str(rank_str)
                .ok_or(JsonError::InvalidObjective("unknown rank string"))?;
            let rank = rank_enum
                .value()
                .checked_sub(1)
                .ok_or(JsonError::InvalidObjective("rank index underflow"))?;
            let rem_u64 = as_strict_u64(
                required_field(map, "remaining")?,
                "objective.remaining must be an integer",
            )?;
            let remaining = u8::try_from(rem_u64)
                .map_err(|_| JsonError::InvalidObjective("remaining exceeds u8 range"))?;
            Ok(Objective::ClearRankQuota { rank, remaining })
        }
        "reach_score_delta" => {
            reject_unknown_fields(map, &["kind", "points_needed"])?;
            let pts_u64 = as_strict_u64(
                required_field(map, "points_needed")?,
                "objective.points_needed must be an integer",
            )?;
            let points_needed = u32::try_from(pts_u64)
                .map_err(|_| JsonError::InvalidObjective("points_needed exceeds u32 range"))?;
            Ok(Objective::ReachScoreDelta { points_needed })
        }
        _ => Err(JsonError::InvalidObjective("unsupported objective.kind")),
    }
}

/// Encode an [`Objective`] into strict canonical JSON.
fn encode_objective(objective: Objective) -> Result<Value, JsonError> {
    let mut map = Map::new();
    match objective {
        Objective::ClearBoard => {
            map.insert("kind".to_owned(), Value::String("clear_board".to_owned()));
        }
        Objective::ClearRankQuota { rank, remaining } => {
            let rank_enum = rank
                .checked_add(1)
                .and_then(Rank::from_value)
                .ok_or(JsonError::InvalidObjective("rank index outside 0..13"))?;
            map.insert(
                "kind".to_owned(),
                Value::String("clear_rank_quota".to_owned()),
            );
            map.insert(
                "rank".to_owned(),
                Value::String(rank_enum.as_str().to_owned()),
            );
            map.insert(
                "remaining".to_owned(),
                Value::Number(Number::from(remaining)),
            );
        }
        Objective::ReachScoreDelta { points_needed } => {
            map.insert(
                "kind".to_owned(),
                Value::String("reach_score_delta".to_owned()),
            );
            map.insert(
                "points_needed".to_owned(),
                Value::Number(Number::from(points_needed)),
            );
        }
    }
    Ok(Value::Object(map))
}

/// Decode one [`CardSlot`] entry (`"Unknown"`, `"Unreachable"`, or `{"Known": "<card>"}`).
fn decode_slot(value: &Value) -> Result<CardSlot, JsonError> {
    match value {
        Value::String(s) if s == "Unknown" => Ok(CardSlot::Unknown),
        Value::String(s) if s == "Unreachable" => Ok(CardSlot::Unreachable),
        Value::Object(map) => {
            reject_unknown_fields(map, &["Known"])?;
            let code = required_field(map, "Known")?
                .as_str()
                .ok_or(JsonError::Unsupported(
                    "Known slot card value must be a string",
                ))?;
            Ok(CardSlot::Known(parse_card_code(code)?))
        }
        _ => Err(JsonError::Unsupported(
            "slot entry must be \"Unknown\", \"Unreachable\", or {\"Known\": \"...\"}",
        )),
    }
}

/// Encode one [`CardSlot`] entry cleanly.
fn encode_slot(slot: CardSlot) -> Result<Value, JsonError> {
    match slot {
        CardSlot::Unknown => Ok(Value::String("Unknown".to_owned())),
        CardSlot::Unreachable => Ok(Value::String("Unreachable".to_owned())),
        CardSlot::Known(card) => {
            let mut map = Map::new();
            map.insert("Known".to_owned(), Value::String(format_card_code(card)?));
            Ok(Value::Object(map))
        }
    }
}

/// Decode one [`TriPeaksMove`] entry (`tap_tableau` or `draw`).
fn decode_move(value: &Value) -> Result<TriPeaksMove, JsonError> {
    let map = value
        .as_object()
        .ok_or(JsonError::Unsupported("route move entry must be an object"))?;
    let mv_kind = required_field(map, "move")?
        .as_str()
        .ok_or(JsonError::Unsupported("move field must be a string"))?;
    match mv_kind {
        "draw" => {
            reject_unknown_fields(map, &["move"])?;
            Ok(TriPeaksMove::Draw)
        }
        "tap_tableau" => {
            reject_unknown_fields(map, &["move", "row", "column"])?;
            let row_u64 = as_strict_u64(required_field(map, "row")?, "row must be an integer")?;
            let col_u64 =
                as_strict_u64(required_field(map, "column")?, "column must be an integer")?;
            let row = u8::try_from(row_u64)
                .map_err(|_| JsonError::Unsupported("row outside u8 range"))?;
            let column = u8::try_from(col_u64)
                .map_err(|_| JsonError::Unsupported("column outside u8 range"))?;
            if tableau_slot_id(row, column).is_none() {
                return Err(ModelError::InvalidTableauCoord { row, column }.into());
            }
            Ok(TriPeaksMove::TapTableau { row, column })
        }
        _ => Err(JsonError::Unsupported(
            "unsupported planned_route move kind",
        )),
    }
}

/// Encode one [`TriPeaksMove`] entry cleanly.
fn encode_move(mv: TriPeaksMove) -> Value {
    let mut map = Map::new();
    match mv {
        TriPeaksMove::Draw => {
            map.insert("move".to_owned(), Value::String("draw".to_owned()));
        }
        TriPeaksMove::TapTableau { row, column } => {
            map.insert("move".to_owned(), Value::String("tap_tableau".to_owned()));
            map.insert("row".to_owned(), Value::Number(Number::from(row)));
            map.insert("column".to_owned(), Value::Number(Number::from(column)));
        }
    }
    Value::Object(map)
}

/// Bounded writer tracking total written byte count up to [`MAX_DOCUMENT_BYTES`].
struct BoundedWriter {
    /// Accumulated UTF-8 JSON bytes.
    bytes: Vec<u8>,
    /// True once a write would cross `MAX_DOCUMENT_BYTES`.
    exceeded: bool,
}

impl Write for BoundedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(buf.len()) > MAX_DOCUMENT_BYTES {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "JSON exceeds output ceiling",
            ));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Decode and validate a complete [`TriPeaksDealDocument`] from JSON bytes (`<= 65,536` bytes).
///
/// Rejects duplicate keys anywhere in the JSON document, trailing bytes, unknown fields,
/// invalid slot tables, and invalid attached routes.
///
/// # Errors
///
/// Returns [`JsonError`] on size, syntax, duplicate-key, schema, or model failures.
pub fn from_slice(bytes: &[u8]) -> Result<TriPeaksDealDocument, JsonError> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(JsonError::InputTooLarge);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| JsonError::Malformed { line: 1, column: 1 })?;
    let mut scanner = DuplicateKeyScanner::new(text);
    scanner.scan_value(0)?;
    scanner.skip_ws();
    if scanner.pos != text.len() {
        return Err(scanner.malformed_here());
    }

    let value: Value = serde_json::from_slice(bytes).map_err(|err| JsonError::Malformed {
        line: err.line(),
        column: err.column(),
    })?;
    let map = value
        .as_object()
        .ok_or(JsonError::Unsupported("root JSON value must be an object"))?;
    reject_unknown_fields(
        map,
        &[
            "schema_version",
            "game",
            "provenance",
            "probes_used",
            "objective",
            "slots",
            "planned_route",
        ],
    )?;

    let schema_version = as_strict_u64(
        required_field(map, "schema_version")?,
        "schema_version must be integer 1",
    )?;
    if schema_version != SCHEMA_VERSION {
        return Err(JsonError::Unsupported("unsupported schema_version"));
    }
    let game = required_field(map, "game")?
        .as_str()
        .ok_or(JsonError::Unsupported("game must be a string"))?;
    if game != GAME_IDENTIFIER {
        return Err(JsonError::Unsupported("game must equal \"tripeaks\""));
    }

    let prov_str = required_field(map, "provenance")?
        .as_str()
        .ok_or(JsonError::Unsupported("provenance must be a string"))?;
    let provenance = DealProvenance::parse(prov_str)
        .ok_or(JsonError::Unsupported("unknown provenance variant"))?;

    let probes_u64 = as_strict_u64(
        required_field(map, "probes_used")?,
        "probes_used must be an integer",
    )?;
    let probes_used =
        usize::try_from(probes_u64).map_err(|_| JsonError::Unsupported("probes_used overflow"))?;

    let objective = decode_objective(required_field(map, "objective")?)?;

    let slot_values = required_field(map, "slots")?
        .as_array()
        .ok_or(JsonError::Unsupported("slots must be an array"))?;
    if slot_values.len() != DEAL_SLOT_COUNT {
        return Err(ModelError::InvalidSlotTableLength {
            actual: slot_values.len(),
        }
        .into());
    }
    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    for (dst, raw_slot) in slots.iter_mut().zip(slot_values) {
        *dst = decode_slot(raw_slot)?;
    }

    let planned_route = match map.get("planned_route") {
        None | Some(Value::Null) => None,
        Some(Value::Array(items)) => {
            if items.len() > MAX_ROUTE_MOVES {
                return Err(ReplayError::RouteTooLong {
                    actual: items.len(),
                }
                .into());
            }
            let mut route = Vec::with_capacity(items.len());
            for item in items {
                route.push(decode_move(item)?);
            }
            Some(route)
        }
        Some(_) => {
            return Err(JsonError::Unsupported(
                "planned_route must be an array or null",
            ));
        }
    };

    let doc = TriPeaksDealDocument {
        provenance,
        probes_used,
        objective,
        slots,
        planned_route,
    };
    doc.validate()?;
    Ok(doc)
}

/// Read and decode up to `65,536` bytes from `reader` without trusting external metadata.
///
/// # Errors
///
/// Returns [`JsonError::InputTooLarge`] if `reader` yields more than `65,536` bytes, or any
/// underlying read/validation failure from [`from_slice`].
pub fn read_from(reader: impl Read) -> Result<TriPeaksDealDocument, JsonError> {
    let mut bytes = Vec::with_capacity(MAX_DOCUMENT_BYTES + 1);
    reader
        .take(u64::try_from(MAX_DOCUMENT_BYTES).unwrap_or(65_536) + 1)
        .read_to_end(&mut bytes)?;
    from_slice(&bytes)
}

/// Validate and serialise `document` into canonical pretty JSON (`<= 65,536` bytes + `\n`).
///
/// # Errors
///
/// Returns [`JsonError`] if validation fails or pretty output exceeds [`MAX_DOCUMENT_BYTES`].
pub fn to_vec(document: &TriPeaksDealDocument) -> Result<Vec<u8>, JsonError> {
    document.validate()?;

    let mut slots_vec = Vec::with_capacity(DEAL_SLOT_COUNT);
    for &slot in &document.slots {
        slots_vec.push(encode_slot(slot)?);
    }

    let mut root = Map::new();
    root.insert(
        "schema_version".to_owned(),
        Value::Number(Number::from(SCHEMA_VERSION)),
    );
    root.insert("game".to_owned(), Value::String(GAME_IDENTIFIER.to_owned()));
    root.insert(
        "provenance".to_owned(),
        Value::String(document.provenance.as_str().to_owned()),
    );
    root.insert(
        "probes_used".to_owned(),
        Value::Number(Number::from(
            u64::try_from(document.probes_used).map_err(|_| JsonError::OutputTooLarge)?,
        )),
    );
    root.insert(
        "objective".to_owned(),
        encode_objective(document.objective)?,
    );
    root.insert("slots".to_owned(), Value::Array(slots_vec));
    if let Some(route) = &document.planned_route {
        let moves = route.iter().copied().map(encode_move).collect();
        root.insert("planned_route".to_owned(), Value::Array(moves));
    }

    let mut writer = BoundedWriter {
        bytes: Vec::with_capacity(MAX_DOCUMENT_BYTES.min(4_096)),
        exceeded: false,
    };
    if serde_json::to_writer_pretty(&mut writer, &Value::Object(root)).is_err()
        || writer.write_all(b"\n").is_err()
    {
        return Err(if writer.exceeded {
            JsonError::OutputTooLarge
        } else {
            JsonError::Unsupported("JSON serialisation failed")
        });
    }
    Ok(writer.bytes)
}

/// Inspect `destination` after a `hard_link` failure against expected `published_bytes`.
#[cfg(unix)]
fn inspect_destination(destination: &Path, published_bytes: &[u8]) -> DestinationState {
    match fs::File::open(destination) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => DestinationState::Absent,
        Err(err) => DestinationState::InspectionFailed(err.kind()),
        Ok(file) => {
            let mut existing = Vec::with_capacity(MAX_DOCUMENT_BYTES + 1);
            match file
                .take(u64::try_from(MAX_DOCUMENT_BYTES).unwrap_or(65_536) + 1)
                .read_to_end(&mut existing)
            {
                Ok(_) if existing.as_slice() == published_bytes => {
                    DestinationState::MatchingBytesPresent
                }
                Ok(_) => DestinationState::PreExistingDifferentBytes,
                Err(err) => DestinationState::InspectionFailed(err.kind()),
            }
        }
    }
}

/// Validate destination filename safety and extract parent directory on Unix.
#[cfg(unix)]
fn checked_parent(destination: &Path) -> Result<&Path, PublishDealError> {
    let raw = destination.as_os_str().as_bytes();
    let filename = raw.rsplit(|&b| b == b'/').next().unwrap_or_default();
    if filename.is_empty() || filename == b"." || filename == b".." || raw.contains(&0) {
        return Err(PublishDealError::InvalidPath);
    }
    Ok(destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".")))
}

/// Owned temporary file handle removed automatically on drop unless already cleaned.
#[cfg(unix)]
struct TempGuard {
    /// Active temporary file path until unlinked.
    path: Option<PathBuf>,
}

#[cfg(unix)]
impl TempGuard {
    /// Create a fresh `0600` file inside `parent` using `create_new(true)`.
    fn create(destination: &Path, parent: &Path) -> io::Result<(Self, fs::File)> {
        for _ in 0..TEMPORARY_ATTEMPTS {
            let seq = TEMPORARY_SEQUENCE
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .map_err(|_| io::Error::other("temporary file sequence exhausted"))?;
            let candidate_path =
                parent.join(format!(".tripeaks-deal-{}-{seq}.tmp", std::process::id()));
            if candidate_path.file_name() == destination.file_name() {
                continue;
            }
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&candidate_path)
            {
                Ok(file) => {
                    return Ok((
                        Self {
                            path: Some(candidate_path),
                        },
                        file,
                    ));
                }
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "temporary filename collision attempts exhausted",
        ))
    }

    /// Attempt to remove the temporary file once and return any unlink error.
    fn cleanup(&mut self) -> Option<io::Error> {
        let path = self.path.take()?;
        fs::remove_file(path).err()
    }
}

#[cfg(unix)]
impl Drop for TempGuard {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Publish `document` to `destination` via same-directory temp file, `sync_all`, and `no-overwrite` `hard_link`.
///
/// Validates and encodes completely before creating any file. Whenever linking fails,
/// removes the private temporary file and inspects `destination` to distinguish
/// absent destinations, pre-existing collisions, and identical completed writes.
///
/// # Errors
///
/// Returns [`PublishDealError`] on validation/serialisation refusal, invalid path, pre-link I/O error,
/// link collision/failure (with [`DestinationState`]), or post-link cleanup/directory-sync warning.
#[cfg(unix)]
pub fn publish_to_path(
    document: &TriPeaksDealDocument,
    destination: &Path,
) -> Result<(), PublishDealError> {
    let bytes = to_vec(document)?;
    let parent = checked_parent(destination)?;
    let dir_handle =
        fs::File::open(parent.join(".")).map_err(|source| PublishDealError::IoBeforeLink {
            stage: PublishStage::OpenDirectory,
            source,
            cleanup_error: None,
        })?;
    let (mut temp, mut file) = TempGuard::create(destination, parent).map_err(|source| {
        PublishDealError::IoBeforeLink {
            stage: PublishStage::CreateTemporary,
            source,
            cleanup_error: None,
        }
    })?;
    if let Err(source) = file.write_all(&bytes) {
        drop(file);
        return Err(PublishDealError::IoBeforeLink {
            stage: PublishStage::WriteTemporary,
            source,
            cleanup_error: temp.cleanup(),
        });
    }
    if let Err(source) = file.sync_all() {
        drop(file);
        return Err(PublishDealError::IoBeforeLink {
            stage: PublishStage::SyncTemporary,
            source,
            cleanup_error: temp.cleanup(),
        });
    }
    drop(file);
    let temp_path = temp.path.clone().ok_or(PublishDealError::InvalidPath)?;
    if let Err(source) = fs::hard_link(&temp_path, destination) {
        let cleanup_error = temp.cleanup();
        let destination_state = inspect_destination(destination, &bytes);
        return Err(PublishDealError::LinkFailed {
            source,
            destination_state,
            cleanup_error,
        });
    }
    let cleanup_error = temp.cleanup();
    let dir_sync_error = dir_handle.sync_all().err();
    if cleanup_error.is_some() || dir_sync_error.is_some() {
        return Err(PublishDealError::PostLinkWarning {
            detail: format!("cleanup_error={cleanup_error:?}, dir_sync_error={dir_sync_error:?}"),
        });
    }
    Ok(())
}

/// Refuse atomic Unix publication on non-Unix platforms.
#[cfg(not(unix))]
pub fn publish_to_path(
    _document: &TriPeaksDealDocument,
    _destination: &Path,
) -> Result<(), PublishDealError> {
    Err(PublishDealError::UnsupportedPlatform)
}
