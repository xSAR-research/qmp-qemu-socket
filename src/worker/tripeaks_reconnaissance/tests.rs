//! Regression, local `AF_UNIX` QMP peer, and multi-frame state-machine tests for
//! [`super::LiveTriPeaksDomain`].

use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use xsar::{
    games::cards::Rank,
    search::{PartialProof, SearchLimits, SearchStop},
};

use super::*;
use crate::{
    QmpClient,
    capture::decode_png,
    freecell::PLAY_X,
    game_controls::FREECELL_ROW_Y,
    geometry::PixelRect,
    parameters::{
        NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, STOCK_CARD_BOUNDS, TARGET_STOCK,
        WASTE_CARD_BOUNDS,
    },
    recon::run_epistemic_recon,
    strategy::{ControllerContext, SolvingStrategy},
    tripeaks_model::{
        DestinationState, MAX_DOCUMENT_BYTES, TriPeaksPosition, apply_move, from_slice,
        verify_move_sequence, verify_objective_route,
    },
};

use super::super::{
    LatestFrameSlot, TestCaptureBackend, WorkerEventSink, game_operations::NativeOperations,
};

/// Monotonic per-process counter isolating temporary Unix sockets and output paths across threads.
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

/// Cached native `1920×1080` fixture frames and `52` card corner coordinates discovered from `FC01`.
struct BaseAssets {
    /// Source Free Cell frame containing all `52` native card corners.
    fc01: CapturedFrame,
    /// Untouched TriPeaks `T = 0` frame (`Undo All` disabled, `FaceDown` Stock, `AC` Waste).
    left_ace: CapturedFrame,
    /// TriPeaks gameplay frame after first `D` draw (`Undo All` enabled).
    draw_first: CapturedFrame,
    /// Calibrated Draw-23 frame showing dark-green empty Stock tray (`TARGET_STOCK`).
    stock_empty: CapturedFrame,
    /// Calibrated tall Normal `"No Moves Remaining"` modal frame with dimmed backdrop.
    no_moves: CapturedFrame,
    /// Calibrated compact Challenge `"No Moves Remaining"` modal frame (`5D` Waste, `Queens cleared 0/10`).
    no_moves_compact: CapturedFrame,
    /// Calibrated TriPeaks `Congratulations` board-win frame.
    tp01: CapturedFrame,
    /// Calibrated Challenge Complete `"Continue"` frame.
    challenge_complete: CapturedFrame,
    /// Calibrated Challenge XP-counting handoff frame (`fours-complete.png`).
    xp_counting: CapturedFrame,
    /// Native `Fours cleared 4/12` header strip source frame.
    fours_header: CapturedFrame,
    /// Discovered `(x, y)` corner origin inside `fc01` indexed by canonical card ID `0..52`.
    corner_origins: [(u32, u32); DEAL_SLOT_COUNT],
}

/// Global lazy asset bank decoded once across all test threads.
static BASE_ASSETS: OnceLock<BaseAssets> = OnceLock::new();

/// Borrow decoded shared native fixtures and discovered `52`-card corner lookup table.
fn assets() -> &'static BaseAssets {
    BASE_ASSETS.get_or_init(|| {
        let decode = |bytes: &[u8], label: &str| {
            decode_png(bytes).unwrap_or_else(|err| panic!("failed decoding {label}: {err}"))
        };
        let fc01 = decode(
            include_bytes!("../../../tests/fixtures/freecell-FC01.png"),
            "freecell-FC01",
        );
        let left_ace = decode(
            include_bytes!("../../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"),
            "left-ace",
        );
        let draw_first = decode(
            include_bytes!("../../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"),
            "draw-first",
        );
        let stock_empty = decode(
            include_bytes!("../../../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"),
            "stock-empty",
        );
        let no_moves = decode(
            include_bytes!(
                "../../../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
            ),
            "no-moves-remaining",
        );
        let no_moves_compact = decode(
            include_bytes!(
                "../../../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining-compact.png"
            ),
            "no-moves-remaining-compact",
        );
        let tp01 = decode(
            include_bytes!("../../../tests/fixtures/tripeaks-TP01.png"),
            "tripeaks-TP01",
        );
        let challenge_complete = decode(
            include_bytes!(
                "../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
            ),
            "boards-complete-continue",
        );
        let xp_counting = decode(
            include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-complete.png"),
            "fours-complete",
        );
        let fours_header = decode(
            include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png"),
            "fours-four-of-twelve",
        );

        let mut corner_origins = [None; DEAL_SLOT_COUNT];
        for (col_idx, &x) in PLAY_X.iter().enumerate() {
            let max_rows = if col_idx < 4 { 7 } else { 6 };
            for (row_idx, &y) in FREECELL_ROW_Y.iter().take(max_rows).enumerate() {
                let col_u8 = u8::try_from(col_idx + 1).expect("col in 1..=8");
                let row_u8 = u8::try_from(row_idx + 1).expect("row in 1..=7");
                let CardRead::Card(xc) =
                    read_card(&fc01, LocationRequest::freecell_play(col_u8, row_u8))
                        .expect("valid fc01 read")
                else {
                    panic!("FC01 ({col_u8}, {row_u8}) did not decode as Card");
                };
                let rc = Card::try_from(xc).expect("valid canonical card");
                corner_origins[usize::from(rc.0)] = Some((x, y));
            }
        }
        let resolved: [(u32, u32); DEAL_SLOT_COUNT] =
            corner_origins.map(|opt| opt.expect("FC01 must contain every canonical card 0..52"));

        BaseAssets {
            fc01,
            left_ace,
            draw_first,
            stock_empty,
            no_moves,
            no_moves_compact,
            tp01,
            challenge_complete,
            xp_counting,
            fours_header,
            corner_origins: resolved,
        }
    })
}

/// Build a canonical [`Card`] from zero-based `rank_idx` (`0..13`) and `suit_idx` (`0..4`).
const fn card(rank_idx: u8, suit_idx: u8) -> Card {
    Card(suit_idx * 13 + rank_idx)
}

/// Build an ordered `52`-card cyclic deal solvable with `ClearBoard` across `28` taps (`Waste = Ace`).
fn build_cyclic_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let order: [usize; 28] = [
        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 9, 10, 11, 12, 13, 14, 15, 16, 17, 3, 4, 5, 6, 7,
        8, 0, 1, 2,
    ];
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    out[usize::from(INITIAL_WASTE_SLOT.0)] = take_rank(0);
    for (step, &slot_idx) in order.iter().enumerate() {
        out[slot_idx] = take_rank(((step + 1) % 13) as u8);
    }
    let mut stock_idx = usize::from(STOCK_START_SLOT);
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            out[stock_idx] = Card(id as u8);
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    out
}

/// Build the stock-gated `52`-card deal requiring multi-probe discovery from `T = 0`.
fn build_stock_gated_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let main_chain: [usize; 25] = [
        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 9, 10, 11, 12, 13, 14, 15, 16, 17, 3, 4, 5, 6, 0, 1,
    ];
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    out[usize::from(INITIAL_WASTE_SLOT.0)] = take_rank(0);
    for (step, &slot_idx) in main_chain.iter().enumerate() {
        out[slot_idx] = take_rank(((step + 1) % 13) as u8);
    }
    out[usize::from(STOCK_START_SLOT)] = take_rank(4);
    out[7] = take_rank(5);
    out[8] = take_rank(6);
    out[2] = take_rank(7);

    let mut stock_idx = usize::from(STOCK_START_SLOT) + 1;
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            out[stock_idx] = Card(id as u8);
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    out
}

/// Build a deal with zero legal Row-4 moves across Waste and all 23 Stock draws, triggering a Draw-23 dead end and leaving 18 hidden cards unreachable.
fn build_draw_23_no_match_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    // Assign Rows 1..=3 (`0..18`) from ranks `{0, 1, 4, 11, 12}` (`4` Aces, `2` Twos, `4` Fives, `4` Queens, `4` Kings).
    let hidden_ranks = [
        0_u8, 0, 0, 0, 1, 1, 4, 4, 4, 4, 11, 11, 11, 11, 12, 12, 12, 12,
    ];
    for (slot, rank_idx) in out[0..18].iter_mut().zip(hidden_ranks) {
        *slot = take_rank(rank_idx);
    }
    // Assign Row 4 (`18..28`) from ranks `{1, 2, 3}` (`2` Twos, `4` Threes, `4` Fours).
    let row4_ranks = [1_u8, 1, 2, 2, 2, 2, 3, 3, 3, 3];
    for (slot, rank_idx) in out[18..TABLEAU_SLOT_COUNT].iter_mut().zip(row4_ranks) {
        *slot = take_rank(rank_idx);
    }
    // Assign Waste + Stock (`28..52`) from ranks `5..=10` (`Six..Jack`), keeping distance >= 2 from Row 4 `{1, 2, 3}`.
    for (idx, slot) in out[usize::from(INITIAL_WASTE_SLOT.0)..DEAL_SLOT_COUNT]
        .iter_mut()
        .enumerate()
    {
        let rank_idx = 5_u8 + (idx as u8 % 6);
        *slot = take_rank(rank_idx);
    }
    assert!(used.iter().all(|&taken| taken));
    out
}

/// Build an unsolvable deal whose Draw-23 card (`SlotId(51)`) is `5D` (matching
/// `no-moves-remaining-compact.png`) with zero legal Row-4 moves at Draw 23.
fn build_draw_23_compact_no_match_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let five_diamonds = card(4, 1);
    used[usize::from(five_diamonds.0)] = true;
    let mut take_rank = |rank_idx: u8| -> Card {
        loop {
            let s = suit_cursor[usize::from(rank_idx)];
            assert!(s < 4, "rank exhausted");
            suit_cursor[usize::from(rank_idx)] = s + 1;
            let c = card(rank_idx, s);
            if !used[usize::from(c.0)] {
                used[usize::from(c.0)] = true;
                return c;
            }
        }
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    // Hidden Rows 1..=3 (`0..18`): absorb adjacent ranks `{3 (4), 12 (K)}` plus remaining `{2, 4, 10, 11}`.
    let hidden_ranks = [
        2_u8, 2, 3, 3, 3, 3, 4, 4, 4, 10, 11, 11, 11, 11, 12, 12, 12, 12,
    ];
    for (slot, rank_idx) in out[0..18].iter_mut().zip(hidden_ranks) {
        *slot = take_rank(rank_idx);
    }
    // Row 4 (`18..28`): ranks `{0, 1, 2}` (`A, 2, 3`), keeping circular distance >= 2 from `5D (4)` and `{5..=10}`.
    let row4_ranks = [0_u8, 0, 0, 0, 1, 1, 1, 1, 2, 2];
    for (slot, rank_idx) in out[18..TABLEAU_SLOT_COUNT].iter_mut().zip(row4_ranks) {
        *slot = take_rank(rank_idx);
    }
    // Waste + Stock (`28..51`) only contain `{4..=10}` and `SlotId(51) = 5D` -> zero legal Row-4 matches.
    out[DEAL_SLOT_COUNT - 1] = five_diamonds;
    let mut next_slot = usize::from(INITIAL_WASTE_SLOT.0);
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            out[next_slot] = Card(id as u8);
            next_slot += 1;
        }
    }
    assert_eq!(next_slot, DEAL_SLOT_COUNT - 1);
    out
}

/// Build an unsolvable deal whose Draw-23 card (`SlotId(51)`) is `AH` (matching
/// `no-moves-remaining.png`) with zero legal Row-4 moves at Draw 23.
fn build_draw_23_tall_ah_no_match_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let ace_hearts = card(0, 2);
    used[usize::from(ace_hearts.0)] = true;
    let mut take_rank = |rank_idx: u8| -> Card {
        loop {
            let s = suit_cursor[usize::from(rank_idx)];
            assert!(s < 4, "rank exhausted");
            suit_cursor[usize::from(rank_idx)] = s + 1;
            let c = card(rank_idx, s);
            if !used[usize::from(c.0)] {
                used[usize::from(c.0)] = true;
                return c;
            }
        }
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    // Hidden Rows 1..=3 (`0..18`): absorb adjacent ranks `{1 (2), 5 (6)}` plus remaining `{0, 4, 11, 12}`.
    let hidden_ranks = [0_u8, 0, 0, 1, 1, 1, 1, 4, 4, 5, 5, 5, 5, 11, 12, 12, 12, 12];
    for (slot, rank_idx) in out[0..18].iter_mut().zip(hidden_ranks) {
        *slot = take_rank(rank_idx);
    }
    // Row 4 (`18..28`): ranks `{2, 3, 4}` (`3, 4, 5`), keeping circular distance >= 2 from `AH (0)` and `{6..=11}`.
    let row4_ranks = [2_u8, 2, 2, 2, 3, 3, 3, 3, 4, 4];
    for (slot, rank_idx) in out[18..TABLEAU_SLOT_COUNT].iter_mut().zip(row4_ranks) {
        *slot = take_rank(rank_idx);
    }
    out[DEAL_SLOT_COUNT - 1] = ace_hearts;
    let mut next_slot = usize::from(INITIAL_WASTE_SLOT.0);
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            out[next_slot] = Card(id as u8);
            next_slot += 1;
        }
    }
    assert_eq!(next_slot, DEAL_SLOT_COUNT - 1);
    out
}

/// Build a solvable `52`-card deal where Draw 23 (`SlotId(51) = 5D`) has zero Row-4 matches,
/// triggering the compact modal during initial Stock sweep before RECON probes and solves.
fn build_solvable_draw_23_compact_modal_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let five_diamonds = card(4, 1);
    used[usize::from(five_diamonds.0)] = true;
    let mut take_rank = |rank_idx: u8| -> Card {
        loop {
            let s = suit_cursor[usize::from(rank_idx)];
            assert!(s < 4, "rank exhausted");
            suit_cursor[usize::from(rank_idx)] = s + 1;
            let c = card(rank_idx, s);
            if !used[usize::from(c.0)] {
                used[usize::from(c.0)] = true;
                return c;
            }
        }
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    out[usize::from(INITIAL_WASTE_SLOT.0)] = take_rank(6); // Seven
    // Row 4 (`18..28`): zero `4 (3)` and zero `6 (5)` so Draw 23 (`5D`) has no Row-4 moves.
    for (slot_idx, rank_idx) in (18..28).zip([7_u8, 8, 9, 10, 11, 12, 0, 1, 2, 1]) {
        out[slot_idx] = take_rank(rank_idx);
    }
    // Row 3 (`9..18`): `3, 4, 5, 6, 7, 8, 9, 10, 11`.
    for (slot_idx, rank_idx) in (9..18).zip([2_u8, 3, 4, 5, 6, 7, 8, 9, 10]) {
        out[slot_idx] = take_rank(rank_idx);
    }
    // Row 2 (`3..9`): `Q, K, A, K, Q, J`.
    for (slot_idx, rank_idx) in (3..9).zip([11_u8, 12, 0, 12, 11, 10]) {
        out[slot_idx] = take_rank(rank_idx);
    }
    // Row 1 (`0..3`): `10, 9, 8`.
    for (slot_idx, rank_idx) in (0..3).zip([9_u8, 8, 7]) {
        out[slot_idx] = take_rank(rank_idx);
    }
    out[DEAL_SLOT_COUNT - 1] = five_diamonds;
    let mut stock_idx = usize::from(STOCK_START_SLOT);
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            out[stock_idx] = Card(id as u8);
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT - 1);
    out
}

/// Build an unsolvable deal trapping Peaks (`slots 0..9`) away from reachable ranks `{4..=11}`.
fn build_unsolvable_peaks_deal_cards() -> [Card; DEAL_SLOT_COUNT] {
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut out = [Card(0); DEAL_SLOT_COUNT];
    for (slot_idx, r) in [3_u8, 3, 3, 3, 12, 12, 12, 12, 0].into_iter().enumerate() {
        out[slot_idx] = take_rank(r);
    }
    for (dst, r) in (9..18)
        .chain(26..28)
        .zip([0_u8, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2])
    {
        out[dst] = take_rank(r);
    }
    let mut rest = Vec::new();
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            rest.push(Card(id as u8));
        }
    }
    let mut ri = rest.into_iter();
    out[usize::from(INITIAL_WASTE_SLOT.0)] = ri.next().expect("waste");
    for slot in &mut out[18..26] {
        *slot = ri.next().expect("row 4");
    }
    for slot in &mut out[usize::from(STOCK_START_SLOT)..DEAL_SLOT_COUNT] {
        *slot = ri.next().expect("stock");
    }
    assert!(ri.next().is_none());
    out
}

/// Convert raw canonical cards into all-`Known` `CardSlot`s and validate uniqueness.
fn cards_to_slots(cards: &[Card; DEAL_SLOT_COUNT]) -> [CardSlot; DEAL_SLOT_COUNT] {
    let slots = cards.map(CardSlot::Known);
    validate_slot_table(&slots).expect("valid 52-card deal");
    slots
}

/// Copy rectangle `src_rect` from `src` onto `dst` starting at `dst_origin`.
fn copy_rect(
    dst: &mut CapturedFrame,
    dst_origin: PixelPoint,
    src: &CapturedFrame,
    src_rect: PixelRect,
) {
    let dst_x = dst_origin.x as u32;
    let dst_y = dst_origin.y as u32;
    for dy in 0..src_rect.height {
        let sy = (src_rect.y + dy) as usize;
        let ty = (dst_y + dy) as usize;
        let src_start = sy * src.stride + src_rect.x as usize * 4;
        let src_end = src_start + src_rect.width as usize * 4;
        let dst_start = ty * dst.stride + dst_x as usize * 4;
        let dst_end = dst_start + src_rect.width as usize * 4;
        dst.pixels[dst_start..dst_end].copy_from_slice(&src.pixels[src_start..src_end]);
    }
}

/// Encode a `CapturedFrame` into valid uncompressed-DEFLATE PNG bytes in milliseconds for QMP tests.
fn encode_stored_png(frame: &CapturedFrame) -> Vec<u8> {
    /// Compute IEEE CRC-32 across a byte iterator.
    fn crc32(bytes: impl IntoIterator<Item = u8>) -> u32 {
        let mut table = [0_u32; 256];
        for (index, value) in table.iter_mut().enumerate() {
            let mut rem = index as u32;
            for _ in 0..8 {
                rem = if (rem & 1) != 0 {
                    (rem >> 1) ^ 0xedb8_8320
                } else {
                    rem >> 1
                };
            }
            *value = rem;
        }
        let mut checksum = u32::MAX;
        for byte in bytes {
            checksum = (checksum >> 8) ^ table[((checksum ^ u32::from(byte)) & 255) as usize];
        }
        !checksum
    }

    /// Append one length-prefixed PNG chunk with CRC.
    fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        png.extend_from_slice(kind);
        png.extend_from_slice(data);
        png.extend_from_slice(
            &crc32(kind.iter().copied().chain(data.iter().copied())).to_be_bytes(),
        );
    }

    let row_bytes = frame.width as usize * 4;
    let mut raw = Vec::with_capacity((row_bytes + 1) * frame.height as usize);
    for y in 0..frame.height as usize {
        raw.push(0);
        raw.extend_from_slice(&frame.pixels[y * frame.stride..y * frame.stride + row_bytes]);
    }
    let mut compressed = vec![0x78, 0x01];
    let count = raw.len().div_ceil(65_535);
    for (index, block) in raw.chunks(65_535).enumerate() {
        compressed.push(u8::from(index + 1 == count));
        let len = block.len() as u16;
        compressed.extend_from_slice(&len.to_le_bytes());
        compressed.extend_from_slice(&(!len).to_le_bytes());
        compressed.extend_from_slice(block);
    }
    let mut a = 1_u32;
    let mut b = 0_u32;
    for block in raw.chunks(5_552) {
        for &byte in block {
            a += u32::from(byte);
            b += a;
        }
        a %= 65_521;
        b %= 65_521;
    }
    compressed.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&frame.width.to_be_bytes());
    ihdr.extend_from_slice(&frame.height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &compressed);
    chunk(&mut png, b"IEND", &[]);
    png
}

/// Paint one synthetic `CLASSIC BLUE` face-down card back at `(dst_x, dst_y, width, 181)`.
fn paint_face_down_card(dst: &mut CapturedFrame, dst_x: u32, dst_y: u32, width: u32) {
    let base = assets();
    let copy_w = width.min(STOCK_CARD_BOUNDS.width);
    copy_rect(
        dst,
        PixelPoint::new(dst_x as i32, dst_y as i32),
        &base.left_ace,
        PixelRect::new(STOCK_CARD_BOUNDS.x, STOCK_CARD_BOUNDS.y, copy_w, 181),
    );
    if width > copy_w {
        copy_rect(
            dst,
            PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(
                STOCK_CARD_BOUNDS.x + copy_w - 1,
                STOCK_CARD_BOUNDS.y,
                1,
                181,
            ),
        );
    }
}

/// Paint one face-up synthetic TriPeaks card with verified `FC01` rank/suit corner and crisp paper.
fn paint_face_up_card(dst: &mut CapturedFrame, dst_x: u32, dst_y: u32, width: u32, target: Card) {
    let base = assets();
    let copy_w = width.min(WASTE_CARD_BOUNDS.width);
    copy_rect(
        dst,
        PixelPoint::new(dst_x as i32, dst_y as i32),
        &base.left_ace,
        PixelRect::new(WASTE_CARD_BOUNDS.x, WASTE_CARD_BOUNDS.y, copy_w, 181),
    );
    if width > copy_w {
        copy_rect(
            dst,
            PixelPoint::new((dst_x + copy_w) as i32, dst_y as i32),
            &base.left_ace,
            PixelRect::new(
                WASTE_CARD_BOUNDS.x + copy_w - 1,
                WASTE_CARD_BOUNDS.y,
                1,
                181,
            ),
        );
    }
    // Clear the underlying Waste Ace-of-Clubs central pip while preserving rounded card outline.
    for dy in 4..177_u32 {
        for dx in 4..width.saturating_sub(4) {
            let offset = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
            dst.pixels[offset..offset + 4].copy_from_slice(&[254, 254, 254, 255]);
        }
    }
    // Stamp genuine `34 × 52` Free Cell corner (preserving outer 3px rounded corner pixels).
    let (src_x, src_y) = base.corner_origins[usize::from(target.0)];
    for dy in 0..52_u32 {
        for dx in 0..34_u32 {
            if dx + dy < 3 {
                continue;
            }
            let so = (src_y + dy) as usize * base.fc01.stride + (src_x + dx) as usize * 4;
            let doff = (dst_y + dy) as usize * dst.stride + (dst_x + dx) as usize * 4;
            dst.pixels[doff..doff + 4].copy_from_slice(&base.fc01.pixels[so..so + 4]);
        }
    }
}

/// Optional Challenge top banner overlay pasted onto synthetic `1920×1080` frames.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BannerKind {
    /// Standard TriPeaks green top HUD (no Challenge ribbon).
    Normal,
    /// Calibrated `Fours cleared 4/12` Challenge ribbon (`ClearRankQuota { rank: 3, remaining: 8 }`).
    FoursFourOfTwelve,
}

/// Render a full `1920×1080` synthetic TriPeaks gameplay frame from `snapshot` and optional banner.
fn synthesize_gameplay_frame(snapshot: &BoardSnapshot, banner: BannerKind) -> CapturedFrame {
    let base = assets();
    let mut frame = base.left_ace.clone();

    if banner == BannerKind::FoursFourOfTwelve {
        copy_rect(
            &mut frame,
            PixelPoint::new(800, 30),
            &base.fours_header,
            PixelRect::new(800, 30, 320, 66),
        );
    }

    // Tile calibrated green gameplay felt across the tableau box `(115, 96, 1690, 545)`.
    for y in 96..(96 + 545_u32) {
        let src_y = 650 + (y - 96) % 180;
        let mut x = 115_u32;
        while x < 115 + 1_690 {
            let chunk = ((115 + 1_690) - x).min(520);
            copy_rect(
                &mut frame,
                PixelPoint::new(x as i32, y as i32),
                &base.left_ace,
                PixelRect::new(128, src_y, chunk, 1),
            );
            x += chunk;
        }
    }

    // Painter's order: Row 1 -> Row 2 -> Row 3 -> Row 4.
    for (idx, region) in TABLEAU_CARD_REGIONS.iter().enumerate() {
        let bounds = region.card_bounds;
        match snapshot.tableau[idx] {
            TableauSlotObs::Empty => {}
            TableauSlotObs::FaceDown => {
                paint_face_down_card(&mut frame, bounds.x, bounds.y, bounds.width);
            }
            TableauSlotObs::Card(card_val) => {
                paint_face_up_card(&mut frame, bounds.x, bounds.y, bounds.width, card_val);
            }
        }
    }

    match snapshot.stock {
        StockObs::FaceDown => {}
        StockObs::Empty => {
            copy_rect(
                &mut frame,
                PixelPoint::new(TARGET_STOCK.x as i32, TARGET_STOCK.y as i32),
                &base.stock_empty,
                TARGET_STOCK,
            );
        }
    }

    paint_face_up_card(
        &mut frame,
        WASTE_CARD_BOUNDS.x,
        WASTE_CARD_BOUNDS.y,
        WASTE_CARD_BOUNDS.width,
        snapshot.waste,
    );

    let undo_bounds = SHARED_UNDO_ALL_CONTROL.bounds;
    match snapshot.undo_all {
        ControlAvailability::Enabled(_) => {
            copy_rect(
                &mut frame,
                PixelPoint::new(undo_bounds.x as i32, undo_bounds.y as i32),
                &base.draw_first,
                undo_bounds,
            );
        }
        ControlAvailability::Disabled => {}
        _ => {
            for y in undo_bounds.y..undo_bounds.bottom() {
                for x in undo_bounds.x..undo_bounds.right() {
                    let off = y as usize * frame.stride + x as usize * 4;
                    frame.pixels[off..off + 4].copy_from_slice(&[20, 20, 20, 255]);
                }
            }
        }
    }

    frame
}

/// Convert a [`BoardSnapshot`] with unreadable slots to model a transient mid-flip `Settling` frame.
fn make_settling_variant(mut frame: CapturedFrame, unreadable_slot: usize) -> CapturedFrame {
    let bounds = TABLEAU_CARD_REGIONS[unreadable_slot].card_bounds;
    for y in bounds.y..bounds.y + 40 {
        for x in bounds.x..bounds.x + 40 {
            let off = y as usize * frame.stride + x as usize * 4;
            frame.pixels[off..off + 4].copy_from_slice(&[120, 120, 120, 255]);
        }
    }
    frame
}

/// Dim a synthetic live board (`0.20x`) and composite the calibrated compact Challenge modal panel
/// `(531, 393, 858, 278)` so uncovered upper cards remain dimmed behind the real dialog overlay.
fn synthesize_compact_modal_over_board(
    snapshot: &BoardSnapshot,
    banner: BannerKind,
) -> CapturedFrame {
    let mut frame = synthesize_gameplay_frame(snapshot, banner);
    let (chunks, _) = frame.pixels.as_chunks_mut::<4>();
    for px in chunks {
        px[0] /= 5;
        px[1] /= 5;
        px[2] /= 5;
    }
    copy_rect(
        &mut frame,
        PixelPoint::new(531, 393),
        &assets().no_moves_compact,
        PixelRect::new(531, 393, 858, 278),
    );
    frame
}

/// Compute `BoardSnapshot` from truth cards + [`TriPeaksPosition`] + `undo_all_enabled`.
fn snapshot_from_position(
    truth: &[Card; DEAL_SLOT_COUNT],
    pos: &TriPeaksPosition,
    undo_all_enabled: bool,
) -> BoardSnapshot {
    let mut tableau = [TableauSlotObs::Empty; TABLEAU_SLOT_COUNT];
    for (idx, dst) in tableau.iter_mut().enumerate() {
        let bit = 1_u32 << idx;
        if (pos.tableau_mask & bit) == 0 {
            *dst = TableauSlotObs::Empty;
        } else if (COVERING_CHILD_MASKS[idx] & pos.tableau_mask) == 0 {
            *dst = TableauSlotObs::Card(truth[idx]);
        } else {
            *dst = TableauSlotObs::FaceDown;
        }
    }
    let stock = if pos.stock_cursor < STOCK_CARD_COUNT {
        StockObs::FaceDown
    } else {
        StockObs::Empty
    };
    let undo_all = if undo_all_enabled {
        ControlAvailability::Enabled(SHARED_UNDO_ALL_CONTROL.click_point)
    } else {
        ControlAvailability::Disabled
    };
    BoardSnapshot {
        tableau,
        stock,
        waste: pos.waste_card,
        undo_all,
    }
}

/// RAII temporary directory removing all socket/JSON test files on drop.
struct TempTestDir(PathBuf);

impl TempTestDir {
    /// Allocate a unique subdirectory inside `std::env::temp_dir()`.
    fn create(prefix: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("{prefix}-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp test directory");
        Self(dir)
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Recorded QMP wire input decoded on the simulated `AF_UNIX` server peer.
#[derive(Clone, Debug, Eq, PartialEq)]
enum WireAction {
    /// Key press and release on QMP key `'d'`.
    DrawKey,
    /// Left click at `(qmp_x, qmp_y)` with measured QMP down-to-up duration.
    Click {
        /// Absolute QMP X coordinate (`0..=32767`).
        qmp_x: u32,
        /// Absolute QMP Y coordinate (`0..=32767`).
        qmp_y: u32,
        /// Measured elapsed duration from mouse-down to mouse-up packet.
        hold: Duration,
    },
}

/// Modal layout served by the simulated `AF_UNIX` QMP guest when `modal_open` is active.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum ModalKind {
    /// Tall Normal-mode modal (`no-moves-remaining.png`, click `(1_128, 728)`).
    #[default]
    TallNormal,
    /// Compact Challenge-mode modal (`no-moves-remaining-compact.png`, click `(1_133, 605)`).
    CompactChallenge,
}

/// Scripted guest scene modes and fault injections for the simulated `AF_UNIX` QMP peer.
#[derive(Clone, Default)]
struct PeerBehavior {
    /// Optional banner stamped onto every generated gameplay frame.
    banner: Option<BannerKind>,
    /// Which `"No Moves Remaining"` layout to serve when `modal_open` activates.
    modal_kind: ModalKind,
    /// Queue a transient `Settling` mid-flip frame on the first N tableau taps.
    inject_settling_on_tap: usize,
    /// Queue N `Animating` mid-unwind frames after `UndoAll`.
    inject_animating_on_undo_all: usize,
    /// Trigger `"No Moves Remaining"` immediately upon Draw 23 (`pos.stock_cursor == 23`), before any un-dimmed Draw-23 capture.
    immediate_modal_on_draw_23: bool,
    /// Trigger `"No Moves Remaining"` on the screendump immediately *following* the first Draw 23 read.
    modal_after_draw_23_read: bool,
    /// Open `"No Moves Remaining"` on the Nth `UndoAll` click (1-based) to exercise modal recovery on sweep (`1`) or probes (`>= 2`).
    modal_race_on_undo_all_index: Option<usize>,
    /// Open `"No Moves Remaining"` only *after* `UndoAll` arrives on the wire (narrow modal race).
    modal_race_on_undo_all: bool,
    /// Open `"No Moves Remaining"` immediately upon reaching the Nth tableau tap (`1`-based).
    modal_on_probe_tap: Option<usize>,
    /// Assert cooperative `cancel` (`STOP`) right after handling N screendumps.
    stop_after_captures: Option<(Arc<AtomicBool>, usize)>,
    /// Inject a QMP error reply on the Nth `input-send-event` down/key packet (triggers `UncertainDelivery`).
    fail_input_on_event_index: Option<usize>,
    /// Corrupt `Row 4 col 1` after the first `Undo All` unwind to force `RootRestoreMismatch`.
    drift_root_after_undo_all: bool,
    /// Intercept the first `TapTableau` with a non-gameplay terminal frame (e.g. `ChallengeComplete`).
    terminal_after_tap_count: Option<(usize, TerminalInject)>,
    /// Keep SlotId(2) (`Peak 3`) face-down when uncovered? (Used only when explicitly testing non-standard scenes.)
    force_slot_2_covered_visual: bool,
}

/// Live terminal overlay kind injected by the simulated QMP guest.
#[derive(Clone, Copy)]
enum TerminalInject {
    /// `boards-complete-continue.png` (`ChallengeComplete`).
    ChallengeComplete,
}

/// Cache key avoiding repeat `encode_png` compression on identical simulated board states.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PngCacheKey {
    /// Remaining tableau occupancy mask.
    tableau_mask: u32,
    /// Stock cursor (`0..=23`).
    stock_cursor: u8,
    /// Active waste card ID.
    waste: u8,
    /// Whether `Undo All` toolbar button is enabled.
    undo_enabled: bool,
    /// Optional settling slot corruption tag.
    settling_slot: Option<u8>,
    /// Top banner overlay kind.
    banner: BannerKind,
    /// Whether `root_drift` was applied.
    drifted: bool,
}

/// Write a single JSON packet terminated by newline and flush to QEMU peer socket.
/// Returns `false` if the client closed the socket before the reply landed.
fn write_qmp_packet(stream: &mut UnixStream, value: Value) -> bool {
    if serde_json::to_writer(&mut *stream, &value).is_err() {
        return false;
    }
    if stream.write_all(b"\n").is_err() {
        return false;
    }
    stream.flush().is_ok()
}

/// Accept one incoming QMP stream cleanly and configure blocking I/O for line reading.
fn accept_qmp_stream(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream
                    .set_nonblocking(false)
                    .expect("configure accepted QMP stream blocking");
                return stream;
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "mock QMP accept timed out");
                thread::sleep(Duration::from_millis(5));
            }
            Err(err) => panic!("mock QMP accept error: {err}"),
        }
    }
}

/// Telemetry returned by the simulated `AF_UNIX` QMP server thread upon shutdown.
#[derive(Debug)]
struct MockQmpTelemetry {
    /// Total `screendump` frames served.
    captures: usize,
    /// Complete wire clicks and keystrokes delivered over QMP.
    actions: Vec<WireAction>,
    /// Raw QMP command execution sequence (`"query-status"`, `"query-mice"`, `"input-send-event"`, ...).
    commands: Vec<String>,
}

/// Spawn a live simulated TriPeaks Windows 11 guest speaking QMP over `socket_path`.
fn spawn_mock_qmp_guest(
    socket_path: PathBuf,
    truth: [Card; DEAL_SLOT_COUNT],
    behavior: PeerBehavior,
) -> thread::JoinHandle<MockQmpTelemetry> {
    // Warm shared fixture bank outside QMP `screendump` timeout window.
    let _ = assets();
    let listener = UnixListener::bind(&socket_path).expect("bind mock QMP socket");
    listener
        .set_nonblocking(true)
        .expect("set mock listener nonblocking");

    thread::spawn(move || {
        let mut stream = accept_qmp_stream(&listener);
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .expect("set QMP stream read timeout");
        let mut reader = BufReader::new(stream.try_clone().expect("clone QMP stream"));
        if !write_qmp_packet(
            &mut stream,
            json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}),
        ) {
            return MockQmpTelemetry {
                captures: 0,
                actions: Vec::new(),
                commands: Vec::new(),
            };
        }

        let full_slots = cards_to_slots(&truth);
        let root_pos = TriPeaksPosition::from_root(&full_slots).expect("valid root position");
        let mut current_truth = truth;
        let mut history: Vec<TriPeaksMove> = Vec::new();
        let mut pos = root_pos;

        let mut modal_open = false;
        let mut draw_23_seen_settled = false;
        let mut tap_count = 0_usize;
        let mut terminal_override: Option<TerminalInject> = None;
        let mut settling_left = 0_usize;
        let mut animating_left = 0_usize;
        let mut drifted = false;
        let mut undo_all_count = 0_usize;

        let mut captures = 0_usize;
        let mut input_packet_count = 0_usize;
        let mut actions = Vec::new();
        let mut commands = Vec::new();
        let mut last_abs: Option<(u32, u32)> = None;
        let mut btn_down_at: Option<(u32, u32, Instant)> = None;
        let mut png_cache: HashMap<PngCacheKey, Vec<u8>> = HashMap::new();
        let mut special_png_cache: HashMap<&'static str, Vec<u8>> = HashMap::new();

        let banner = behavior.banner.unwrap_or(BannerKind::Normal);
        let qmp_to_pixel = |qmp: u32, extent: u32| -> i32 {
            // Exact inverse check range for integer `px * 32767 / extent`.
            let approx = (u64::from(qmp) * u64::from(extent) + 16_383) / 32_767;
            approx as i32
        };

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(_) => break,
            }
            let Ok(request) = serde_json::from_str::<Value>(&line) else {
                break;
            };
            let id = request["id"].as_u64().expect("numeric QMP id");
            let cmd = request["execute"].as_str().expect("QMP command").to_owned();
            commands.push(cmd.clone());

            match cmd.as_str() {
                "qmp_capabilities" => {
                    if !write_qmp_packet(&mut stream, json!({"return": {}, "id": id})) {
                        break;
                    }
                }
                "query-status" => {
                    if !write_qmp_packet(
                        &mut stream,
                        json!({"return": {"running": true, "status": "running"}, "id": id}),
                    ) {
                        break;
                    }
                }
                "query-mice" => {
                    if !write_qmp_packet(
                        &mut stream,
                        json!({"return": [{"name": "QEMU HID Tablet", "index": 0, "current": true, "absolute": true}], "id": id}),
                    ) {
                        break;
                    }
                }
                "screendump" => {
                    captures += 1;
                    let filename = request["arguments"]["filename"]
                        .as_str()
                        .expect("screendump filename");

                    let png_bytes: Vec<u8> = if let Some(kind) = terminal_override {
                        let key = match kind {
                            TerminalInject::ChallengeComplete => "challenge_complete",
                        };
                        special_png_cache
                            .entry(key)
                            .or_insert_with(|| encode_stored_png(&assets().challenge_complete))
                            .clone()
                    } else if modal_open {
                        if behavior.modal_on_probe_tap.is_some()
                            && behavior.modal_kind == ModalKind::CompactChallenge
                        {
                            let snap = snapshot_from_position(&current_truth, &pos, true);
                            let frame = synthesize_compact_modal_over_board(&snap, banner);
                            encode_stored_png(&frame)
                        } else {
                            let (cache_key, modal_frame) = match behavior.modal_kind {
                                ModalKind::TallNormal => ("no_moves", &assets().no_moves),
                                ModalKind::CompactChallenge => {
                                    ("no_moves_compact", &assets().no_moves_compact)
                                }
                            };
                            special_png_cache
                                .entry(cache_key)
                                .or_insert_with(|| encode_stored_png(modal_frame))
                                .clone()
                        }
                    } else {
                        let mut effective_pos = pos;
                        let undo_enabled = if animating_left > 0 {
                            animating_left -= 1;
                            // Mid-unwind animating frame: keep Waste distinct and Undo All Enabled.
                            effective_pos.stock_cursor = STOCK_CARD_COUNT.saturating_sub(1);
                            true
                        } else {
                            !history.is_empty()
                        };
                        let settling_slot = if settling_left > 0 {
                            settling_left -= 1;
                            Some(18_u8)
                        } else {
                            None
                        };
                        let key = PngCacheKey {
                            tableau_mask: effective_pos.tableau_mask,
                            stock_cursor: effective_pos.stock_cursor,
                            waste: effective_pos.waste_card.0,
                            undo_enabled,
                            settling_slot,
                            banner,
                            drifted,
                        };
                        png_cache
                            .entry(key)
                            .or_insert_with(|| {
                                let mut snap = snapshot_from_position(
                                    &current_truth,
                                    &effective_pos,
                                    undo_enabled,
                                );
                                if behavior.force_slot_2_covered_visual
                                    && matches!(snap.tableau[2], TableauSlotObs::Card(_))
                                {
                                    snap.tableau[2] = TableauSlotObs::FaceDown;
                                }
                                let mut frame = synthesize_gameplay_frame(&snap, banner);
                                if let Some(slot_u8) = settling_slot {
                                    frame = make_settling_variant(frame, usize::from(slot_u8));
                                }
                                encode_stored_png(&frame)
                            })
                            .clone()
                    };

                    fs::write(filename, &png_bytes).expect("write PNG screendump");
                    if !write_qmp_packet(&mut stream, json!({"return": {}, "id": id})) {
                        break;
                    }

                    if behavior.modal_after_draw_23_read
                        && pos.stock_cursor == STOCK_CARD_COUNT
                        && !modal_open
                    {
                        if draw_23_seen_settled {
                            // Modal dims right after Draw 23 is captured.
                        }
                        draw_23_seen_settled = true;
                        modal_open = true;
                    }
                    if let Some((ref stop_flag, threshold)) = behavior.stop_after_captures
                        && captures == threshold
                    {
                        stop_flag.store(true, Ordering::Release);
                    }
                }
                "input-send-event" => {
                    input_packet_count += 1;
                    if behavior.fail_input_on_event_index == Some(input_packet_count) {
                        let _ = write_qmp_packet(
                            &mut stream,
                            json!({"error": {"class": "GenericError", "desc": "injected QMP transport failure"}, "id": id}),
                        );
                        continue;
                    }
                    let events = request["arguments"]["events"]
                        .as_array()
                        .expect("QMP events array");
                    let mut got_abs_x = None;
                    let mut got_abs_y = None;
                    for ev in events {
                        match ev["type"].as_str() {
                            Some("abs") => {
                                let val = ev["data"]["value"].as_u64().expect("abs value") as u32;
                                match ev["data"]["axis"].as_str() {
                                    Some("x") => got_abs_x = Some(val),
                                    Some("y") => got_abs_y = Some(val),
                                    _ => {}
                                }
                            }
                            Some("btn") if ev["data"]["button"] == "left" => {
                                let down = ev["data"]["down"].as_bool().expect("btn down bool");
                                if down {
                                    let (qx, qy) = last_abs.expect("btn down after abs move");
                                    btn_down_at = Some((qx, qy, Instant::now()));
                                } else if let Some((qx, qy, down_ts)) = btn_down_at.take() {
                                    let hold = Instant::now().duration_since(down_ts);
                                    actions.push(WireAction::Click {
                                        qmp_x: qx,
                                        qmp_y: qy,
                                        hold,
                                    });
                                    let px = PixelPoint::new(
                                        qmp_to_pixel(qx, NOMINAL_FRAME_WIDTH),
                                        qmp_to_pixel(qy, NOMINAL_FRAME_HEIGHT),
                                    );
                                    if modal_open {
                                        let expected_modal_click = match behavior.modal_kind {
                                            ModalKind::TallNormal => UNDO_LAST_MOVE_CLICK_POINT,
                                            ModalKind::CompactChallenge => {
                                                UNDO_LAST_MOVE_COMPACT_CLICK_POINT
                                            }
                                        };
                                        if px == expected_modal_click
                                            && hold >= Duration::from_millis(450)
                                        {
                                            modal_open = false;
                                            if let Some(_last) = history.pop() {
                                                // Replay remaining history cleanly from root.
                                                pos = root_pos;
                                                let cur_slots = cards_to_slots(&current_truth);
                                                for &mv in &history {
                                                    let (next, _) =
                                                        apply_move(&pos, &cur_slots, mv)
                                                            .expect("valid replay history");
                                                    pos = next;
                                                }
                                            }
                                        }
                                    } else if px == SHARED_UNDO_ALL_CONTROL.click_point {
                                        undo_all_count += 1;
                                        let race_target =
                                            behavior.modal_race_on_undo_all_index.or_else(|| {
                                                behavior.modal_race_on_undo_all.then_some(1)
                                            });
                                        if race_target == Some(undo_all_count) {
                                            modal_open = true;
                                        } else {
                                            history.clear();
                                            pos = root_pos;
                                            animating_left = behavior.inject_animating_on_undo_all;
                                            if behavior.drift_root_after_undo_all {
                                                // Swap Row-4 first and last cards cleanly while preserving 52 uniqueness.
                                                current_truth.swap(18, 27);
                                                drifted = true;
                                            }
                                        }
                                    } else if let Some((slot_idx, _)) =
                                        TABLEAU_CARD_REGIONS.iter().enumerate().find(|(_, r)| {
                                            r.click_point.x.abs_diff(px.x) <= 1
                                                && r.click_point.y.abs_diff(px.y) <= 1
                                        })
                                    {
                                        let slot_u8 = u8::try_from(slot_idx).expect("slot fits u8");
                                        let (row, col) = slot_to_tableau_coord(SlotId(slot_u8))
                                            .expect("valid slot coord");
                                        let mv = TriPeaksMove::TapTableau { row, column: col };
                                        let cur_slots = cards_to_slots(&current_truth);
                                        let (next, _) = apply_move(&pos, &cur_slots, mv)
                                            .expect("guest move legal");
                                        history.push(mv);
                                        pos = next;
                                        tap_count += 1;
                                        if tap_count <= behavior.inject_settling_on_tap {
                                            settling_left = 1;
                                        }
                                        if let Some((trigger_count, kind)) =
                                            behavior.terminal_after_tap_count
                                            && tap_count == trigger_count
                                        {
                                            terminal_override = Some(kind);
                                        }
                                        if behavior.modal_on_probe_tap == Some(tap_count) {
                                            modal_open = true;
                                        }
                                    }
                                }
                            }
                            Some("key") => {
                                let down = ev["data"]["down"].as_bool().expect("key down bool");
                                if !down && ev["data"]["key"]["data"] == "d" {
                                    actions.push(WireAction::DrawKey);
                                    let mv = TriPeaksMove::Draw;
                                    let cur_slots = cards_to_slots(&current_truth);
                                    let (next, _) =
                                        apply_move(&pos, &cur_slots, mv).expect("guest draw legal");
                                    history.push(mv);
                                    pos = next;
                                    if behavior.immediate_modal_on_draw_23
                                        && pos.stock_cursor == STOCK_CARD_COUNT
                                    {
                                        modal_open = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if let (Some(qx), Some(qy)) = (got_abs_x, got_abs_y) {
                        last_abs = Some((qx, qy));
                    }
                    if !write_qmp_packet(&mut stream, json!({"return": {}, "id": id})) {
                        break;
                    }
                }
                other => panic!("unexpected mock QMP command: {other}"),
            }
        }

        MockQmpTelemetry {
            captures,
            actions,
            commands,
        }
    })
}

/// Fast-timeout config exercising real QMP protocol packets cleanly within milliseconds between moves.
fn fast_qmp_config(max_probes: usize) -> TriPeaksReconConfig {
    TriPeaksReconConfig {
        max_probes,
        objective_override: None,
        force_challenge_mode: false,
        move_settle_delay: Duration::from_millis(2),
        reobserve_delay: Duration::from_millis(2),
        restore_settle_delay: Duration::from_millis(2),
        max_step_observations: 6,
        max_restore_observations: 8,
        search_limits: DEFAULT_SEARCH_LIMITS,
    }
}

/// Build a `WorkerEventSink` and `NativeOperations` QMP session for tests.
fn run_qmp_session_test<F>(
    socket_path: &Path,
    cancel: &Arc<AtomicBool>,
    config: TriPeaksReconConfig,
    destination: Option<&Path>,
    mut on_preview: F,
) -> Result<(TriPeaksReconRunReport, usize, bool), TriPeaksReconFailure>
where
    F: FnMut(usize, &CapturedFrame),
{
    let (tx, _rx) = std::sync::mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(None),
    };
    let mut qmp = QmpClient::connect(socket_path).expect("connect to local mock QMP peer");
    let context = ControllerContext::new(
        socket_path.to_path_buf(),
        GameMode::TriPeaks,
        SolvingStrategy::ShortestPath,
        1,
    );
    let io = NativeOperations {
        qmp: &mut qmp,
        socket: socket_path,
        events: &sink,
        cancel: cancel.as_ref(),
        active: || Ok(()),
        guard: authorise_tripeaks_input,
        last_timing: None,
    };
    let session = Session::new(io);
    let mut preview_count = 0_usize;
    let preview_context = context.clone();
    let preview_sink = &sink;
    let mut domain = LiveTriPeaksDomain::new(
        session,
        cancel.as_ref(),
        config,
        Box::new(|frame| {
            preview_count += 1;
            preview_sink.latest_frame.publish(
                frame.clone(),
                None,
                false,
                false,
                Some(preview_context.clone()),
            );
            on_preview(preview_count, frame);
        }),
    );

    let res = run_tripeaks_recon(&mut domain, destination);
    let uncertain = domain.is_uncertain_latched();
    drop(domain);
    let ui_has_preview = sink.latest_frame.take().is_some();
    assert!(
        ui_has_preview || preview_count == 0,
        "preview mailbox must retain latest published frame"
    );
    res.map(|report| (report, preview_count, uncertain))
}

#[test]
fn native_fixtures_and_52_card_synth_calibrate_cleanly() {
    let base = assets();
    assert_eq!(
        classify_tripeaks_undo_all(&base.left_ace).expect("classify T=0 Undo All"),
        ControlAvailability::Disabled
    );
    assert_eq!(
        classify_tripeaks_undo_all(&base.draw_first).expect("classify active Undo All"),
        ControlAvailability::Enabled(SHARED_UNDO_ALL_CONTROL.click_point)
    );
    assert_eq!(
        classify_tripeaks_undo_all(&base.no_moves).expect("classify modal Undo All"),
        ControlAvailability::NotFound
    );

    let FrameClassification::Settled(left_ace_snap) =
        observe_frame(&base.left_ace).expect("observe left-ace")
    else {
        panic!("left-ace must settle cleanly");
    };
    assert!(left_ace_snap.is_untouched_root_layout());
    assert_eq!(left_ace_snap.occupied_tableau_count(), 28);

    let left_eight = decode_png(include_bytes!(
        "../../../tests/fixtures/tripeaks-redeal-20261016/left-eight.png"
    ))
    .expect("decode left-eight");
    let FrameClassification::Settled(left_eight_snap) =
        observe_frame(&left_eight).expect("observe left-eight")
    else {
        panic!("left-eight must settle");
    };
    assert!(left_eight_snap.is_untouched_root_layout());
    assert_ne!(
        semantic_board_hash(&left_ace_snap),
        semantic_board_hash(&left_eight_snap)
    );

    let FrameClassification::Settled(draw_first_snap) =
        observe_frame(&base.draw_first).expect("observe draw-first")
    else {
        panic!("draw-first must settle");
    };
    assert!(!draw_first_snap.is_untouched_root_layout());
    assert_eq!(
        draw_first_snap.undo_all,
        ControlAvailability::Enabled(SHARED_UNDO_ALL_CONTROL.click_point)
    );

    let stock_last = decode_png(include_bytes!(
        "../../../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
    ))
    .expect("decode stock-last-card");
    let FrameClassification::Settled(last_snap) =
        observe_frame(&stock_last).expect("observe stock-last")
    else {
        panic!("stock-last must settle");
    };
    assert_eq!(last_snap.stock, StockObs::FaceDown);

    let FrameClassification::Settled(empty_snap) =
        observe_frame(&base.stock_empty).expect("observe stock-empty")
    else {
        panic!("stock-empty must settle");
    };
    assert_eq!(empty_snap.stock, StockObs::Empty);

    let row2_uncovered = decode_png(include_bytes!(
        "../../../tests/fixtures/tripeaks-calibration-20261116/tableau-row2-uncovered.png"
    ))
    .expect("decode row2 uncovered");
    assert!(matches!(
        observe_frame(&row2_uncovered).expect("observe row2-uncovered"),
        FrameClassification::Settled(_)
    ));

    let peak1_uncovered = decode_png(include_bytes!(
        "../../../tests/fixtures/tripeaks-calibration-20261116/peak1-uncovered.png"
    ))
    .expect("decode peak1 uncovered");
    assert!(matches!(
        observe_frame(&peak1_uncovered).expect("observe peak1-uncovered"),
        FrameClassification::Settled(_)
    ));

    assert_eq!(
        observe_frame(&base.no_moves).expect("observe no_moves"),
        FrameClassification::NoMovesModal(UNDO_LAST_MOVE_CLICK_POINT)
    );
    assert_eq!(
        observe_frame(&base.no_moves_compact).expect("observe compact no_moves"),
        FrameClassification::NoMovesModal(UNDO_LAST_MOVE_COMPACT_CLICK_POINT)
    );
    assert!(
        authorise_tripeaks_input(
            &base.no_moves_compact,
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
        )
        .is_ok()
    );
    assert!(
        authorise_tripeaks_input(
            &base.no_moves_compact,
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
        )
        .is_err()
    );
    assert!(
        authorise_tripeaks_input(
            &base.no_moves_compact,
            Input::MouseLeftHold {
                point: PixelPoint::new(791, 605),
                hold: POST_GAME_MOUSE_HOLD,
            },
        )
        .is_err(),
        "gold End game button must never be authorised"
    );
    assert!(
        authorise_tripeaks_input(
            &base.no_moves,
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
        )
        .is_err()
    );
    assert_eq!(
        observe_frame(&base.tp01).expect("observe TP01"),
        FrameClassification::BoardWinOverlay
    );
    assert_eq!(
        observe_frame(&base.challenge_complete).expect("observe challenge complete"),
        FrameClassification::ChallengeComplete
    );

    // Round-trip all 52 synthesized cards through `observe_frame` across two clean boards.
    let deal_a = build_cyclic_deal_cards();
    let slots_a = cards_to_slots(&deal_a);
    let pos_a = TriPeaksPosition::from_root(&slots_a).expect("root pos A");
    let full_mask_cleared_children = BoardSnapshot {
        tableau: [
            TableauSlotObs::Card(deal_a[0]),
            TableauSlotObs::Card(deal_a[1]),
            TableauSlotObs::Card(deal_a[2]),
            TableauSlotObs::Card(deal_a[3]),
            TableauSlotObs::Card(deal_a[4]),
            TableauSlotObs::Card(deal_a[5]),
            TableauSlotObs::Card(deal_a[6]),
            TableauSlotObs::Card(deal_a[7]),
            TableauSlotObs::Card(deal_a[8]),
            TableauSlotObs::Card(deal_a[9]),
            TableauSlotObs::Card(deal_a[10]),
            TableauSlotObs::Card(deal_a[11]),
            TableauSlotObs::Card(deal_a[12]),
            TableauSlotObs::Card(deal_a[13]),
            TableauSlotObs::Card(deal_a[14]),
            TableauSlotObs::Card(deal_a[15]),
            TableauSlotObs::Card(deal_a[16]),
            TableauSlotObs::Card(deal_a[17]),
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
            TableauSlotObs::Empty,
        ],
        stock: StockObs::FaceDown,
        waste: deal_a[28],
        undo_all: ControlAvailability::Enabled(SHARED_UNDO_ALL_CONTROL.click_point),
    };
    // Verify upper 18 slots + Waste synthesize cleanly when uncovered.
    let mut upper_only = full_mask_cleared_children;
    for slot in &mut upper_only.tableau[3..18] {
        *slot = TableauSlotObs::Empty;
    }
    let upper_frame = synthesize_gameplay_frame(&upper_only, BannerKind::Normal);
    assert!(matches!(
        observe_frame(&upper_frame).expect("observe uncovered Row 1"),
        FrameClassification::Settled(_)
    ));

    // Verify every single card `0..52` reads unambiguously on Row 4 & Waste via `read_card`.
    let base_snap = snapshot_from_position(&deal_a, &pos_a, false);
    for id in 0_u8..52 {
        let mut test_snap = base_snap.clone();
        test_snap.tableau[18] = TableauSlotObs::Card(Card(id));
        test_snap.waste = Card((id + 1) % 52);
        let synth = synthesize_gameplay_frame(&test_snap, BannerKind::Normal);
        let FrameClassification::Settled(observed) =
            observe_frame(&synth).expect("observe synthesized 52 card")
        else {
            panic!("card id {id} did not settle cleanly");
        };
        assert_eq!(observed.tableau[18], TableauSlotObs::Card(Card(id)));
        assert_eq!(observed.waste, Card((id + 1) % 52));
    }
}

#[test]
fn qmp_peer_runs_full_stock_sweep_multi_probe_and_atomic_json() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-full");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("tripeaks-deal.json");

    let truth = build_stock_gated_deal_cards();
    let handle = spawn_mock_qmp_guest(
        socket.clone(),
        truth,
        PeerBehavior {
            inject_settling_on_tap: 1,
            inject_animating_on_undo_all: 1,
            ..PeerBehavior::default()
        },
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let mut saw_initial_preview = false;
    let (report, preview_count, uncertain) = run_qmp_session_test(
        &socket,
        &cancel,
        fast_qmp_config(10),
        Some(&dest),
        |idx, frame| {
            if idx == 1 {
                saw_initial_preview =
                    matches!(observe_frame(frame), Ok(FrameClassification::Settled(_)));
            }
        },
    )
    .expect("full stock-gated QMP RECON should succeed");

    let telemetry = handle.join().expect("join mock QMP server");
    assert!(saw_initial_preview);
    assert!(preview_count >= 26);
    assert!(!uncertain);
    assert!(telemetry.captures >= 26);

    let ReconOutcome::SolvedFromRoot { route, probes_used } = report.outcome else {
        panic!("expected SolvedFromRoot, got {:?}", report.outcome);
    };
    assert_eq!(probes_used, 1);

    let draw_count = telemetry
        .actions
        .iter()
        .filter(|act| matches!(act, WireAction::DrawKey))
        .count();
    assert!(draw_count >= usize::from(STOCK_CARD_COUNT));

    // Verify direct confirmation-less `Undo All` was used with normal `MOUSE_HOLD` (`< 200 ms`).
    let expected_undo_qmp_x = (SHARED_UNDO_ALL_CONTROL.click_point.x as u32 * 32_767) / 1_920;
    let expected_undo_qmp_y = (SHARED_UNDO_ALL_CONTROL.click_point.y as u32 * 32_767) / 1_080;
    let undo_clicks: Vec<_> = telemetry
        .actions
        .iter()
        .filter(|act| {
            matches!(
                act,
                WireAction::Click {
                    qmp_x,
                    qmp_y,
                    hold
                } if *qmp_x == expected_undo_qmp_x
                    && *qmp_y == expected_undo_qmp_y
                    && *hold < Duration::from_millis(200)
            )
        })
        .collect();
    assert_eq!(undo_clicks.len(), 1 + probes_used);

    // Confirm published JSON matches schema <= 65,536 bytes and replays to full Peak clear.
    let bytes = fs::read(&dest).expect("read published TriPeaks JSON");
    assert!(bytes.len() <= MAX_DOCUMENT_BYTES);
    let parsed = from_slice(&bytes).expect("decode published TriPeaks JSON");
    assert_eq!(parsed.provenance, DealProvenance::RestoredAtRoot);
    assert_eq!(parsed.probes_used, probes_used);
    assert_eq!(parsed.slots.len(), DEAL_SLOT_COUNT);
    assert!(
        parsed
            .slots
            .iter()
            .all(|slot| matches!(slot, CardSlot::Known(_)))
    );
    verify_objective_route(&parsed.slots, &route, Objective::ClearBoard)
        .expect("verified root route");
}

#[test]
fn qmp_peer_backs_out_draw_23_no_moves_modal_and_single_move_dead_end() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let truth = build_draw_23_no_match_deal_cards();

    // 1. Subcase A: Modal appears right after Draw 23 Waste is read cleanly, backed out before `Undo All`.
    let dir_a = TempTestDir::create("tripeaks-qmp-modal-a");
    let sock_a = dir_a.0.join("qmp.sock");
    let handle_a = spawn_mock_qmp_guest(
        sock_a.clone(),
        truth,
        PeerBehavior {
            modal_after_draw_23_read: true,
            ..PeerBehavior::default()
        },
    );
    let cancel_a = Arc::new(AtomicBool::new(false));
    let dest_a = dir_a.0.join("dead-deal.json");
    let (report_a, _, _) = run_qmp_session_test(
        &sock_a,
        &cancel_a,
        fast_qmp_config(4),
        Some(&dest_a),
        |_, _| {},
    )
    .expect("Draw-23 modal back-out should recover and conclude UnsolvableBestPartial");
    let tel_a = handle_a.join().expect("join modal A peer");
    assert!(matches!(
        report_a.outcome,
        ReconOutcome::UnsolvableBestPartial { .. }
    ));

    let expected_modal_x = (UNDO_LAST_MOVE_CLICK_POINT.x as u32 * 32_767) / 1_920;
    let expected_modal_y = (UNDO_LAST_MOVE_CLICK_POINT.y as u32 * 32_767) / 1_080;
    let long_hold_clicks = tel_a
        .actions
        .iter()
        .filter(|act| {
            matches!(
                act,
                WireAction::Click { qmp_x, qmp_y, hold }
                if *qmp_x == expected_modal_x
                    && *qmp_y == expected_modal_y
                    && *hold >= Duration::from_millis(450)
            )
        })
        .count();
    assert_eq!(
        long_hold_clicks, 1,
        "must dismiss No Moves Remaining using POST_GAME_MOUSE_HOLD once"
    );

    // 2. Subcase B: Narrow race where No Moves modal opens only *immediately after* `Undo All` click arrives.
    let dir_b = TempTestDir::create("tripeaks-qmp-modal-b");
    let sock_b = dir_b.0.join("qmp.sock");
    let handle_b = spawn_mock_qmp_guest(
        sock_b.clone(),
        truth,
        PeerBehavior {
            modal_race_on_undo_all: true,
            ..PeerBehavior::default()
        },
    );
    let cancel_b = Arc::new(AtomicBool::new(false));
    let (report_b, _, _) =
        run_qmp_session_test(&sock_b, &cancel_b, fast_qmp_config(4), None, |_, _| {})
            .expect("post-UndoAll modal race should back out cleanly");
    let tel_b = handle_b.join().expect("join modal B peer");
    assert!(matches!(
        report_b.outcome,
        ReconOutcome::UnsolvableBestPartial { .. }
    ));
    assert!(tel_b.actions.iter().any(|act| matches!(
        act,
        WireAction::Click { qmp_x, qmp_y, hold }
        if *qmp_x == expected_modal_x && *qmp_y == expected_modal_y && *hold >= Duration::from_millis(450)
    )));

    // 3. Subcase C: 1-move dead-end probe returns directly to T=0 upon `"Undo last move"`, skipping disabled `Undo All`.
    struct DeadEndModalIo {
        frames: VecDeque<CapturedFrame>,
        inputs: Vec<Input>,
    }
    impl OperationIo for DeadEndModalIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.frames
                .pop_front()
                .ok_or_else(|| OperationError::Transport("out of frames".to_owned()))
        }
        fn authorise(&mut self, f: &CapturedFrame, i: Input) -> Result<(), OperationError> {
            authorise_tripeaks_input(f, i)
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.inputs.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    let deal = build_cyclic_deal_cards();
    let slots = cards_to_slots(&deal);
    let root_snap = snapshot_from_position(
        &deal,
        &TriPeaksPosition::from_root(&slots).expect("root"),
        false,
    );
    let root_frame = synthesize_gameplay_frame(&root_snap, BannerKind::Normal);
    let modal_frame = assets().no_moves.clone();

    let mut frames = VecDeque::new();
    // undo_all_and_hash: observe NoMovesModal twice -> click Undo last move (POST_GAME_MOUSE_HOLD) -> observe root twice!
    frames.push_back(modal_frame.clone());
    frames.push_back(modal_frame);
    frames.push_back(root_frame.clone());
    frames.push_back(root_frame.clone());

    let cancel_c = AtomicBool::new(false);
    let mut dom_c = LiveTriPeaksDomain::new(
        Session::new(DeadEndModalIo {
            frames,
            inputs: Vec::new(),
        }),
        &cancel_c,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_c.root_snapshot = Some(root_snap.clone());
    dom_c.root_hash = Some(semantic_board_hash(&root_snap));

    let restored_hash = dom_c
        .undo_all_and_hash()
        .expect("direct modal return to T=0 must succeed without disabled Undo All click");
    assert_eq!(restored_hash, semantic_board_hash(&root_snap));
    let session_io = dom_c.into_session();
    assert_eq!(
        session_io.io().inputs,
        vec![Input::MouseLeftHold {
            point: UNDO_LAST_MOVE_CLICK_POINT,
            hold: POST_GAME_MOUSE_HOLD,
        }]
    );
}

#[test]
fn qmp_peer_recovers_compact_challenge_draw_23_modal_and_probe_modals() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let expected_compact_x = (UNDO_LAST_MOVE_COMPACT_CLICK_POINT.x as u32 * 32_767) / 1_920;
    let expected_compact_y = (UNDO_LAST_MOVE_COMPACT_CLICK_POINT.y as u32 * 32_767) / 1_080;
    let expected_tall_x = (UNDO_LAST_MOVE_CLICK_POINT.x as u32 * 32_767) / 1_920;
    let expected_tall_y = (UNDO_LAST_MOVE_CLICK_POINT.y as u32 * 32_767) / 1_080;
    let expected_undo_x = (SHARED_UNDO_ALL_CONTROL.click_point.x as u32 * 32_767) / 1_920;
    let expected_undo_y = (SHARED_UNDO_ALL_CONTROL.click_point.y as u32 * 32_767) / 1_080;
    let five_diamonds = Card::try_from(xsar::games::cards::Card::new(
        Rank::Five,
        xsar::games::cards::Suit::Diamonds,
    ))
    .expect("5D canonical");
    let ace_hearts = Card::try_from(xsar::games::cards::Card::new(
        Rank::Ace,
        xsar::games::cards::Suit::Hearts,
    ))
    .expect("AH canonical");

    // 1. Subcase A: Solvable deal where Draw 23 (`SlotId(51) = 5D`) IMMEDIATELY opens the real
    //    `no-moves-remaining-compact.png` Challenge modal -> RECON reads `5D` from darkened
    //    Waste + confirms darkened empty Stock -> clicks compact `(1_133, 605)` with 500ms hold
    //    -> confirms `Stock == FaceDown` & `Waste == SlotId(50)` -> clicks `Undo All` -> restores
    //    `T = 0` -> continues probing and solves to `SolvedFromRoot`!
    let dir_a = TempTestDir::create("tripeaks-qmp-compact-solvable");
    let sock_a = dir_a.0.join("qmp.sock");
    let dest_a = dir_a.0.join("compact-solved.json");
    let truth_a = build_solvable_draw_23_compact_modal_deal_cards();
    assert_eq!(truth_a[DEAL_SLOT_COUNT - 1], five_diamonds);
    let handle_a = spawn_mock_qmp_guest(
        sock_a.clone(),
        truth_a,
        PeerBehavior {
            banner: Some(BannerKind::FoursFourOfTwelve),
            modal_kind: ModalKind::CompactChallenge,
            immediate_modal_on_draw_23: true,
            ..PeerBehavior::default()
        },
    );
    let cancel_a = Arc::new(AtomicBool::new(false));
    let mut cfg_a = fast_qmp_config(6);
    cfg_a.objective_override = Some(Objective::ClearBoard);
    let (report_a, _, uncertain_a) =
        run_qmp_session_test(&sock_a, &cancel_a, cfg_a, Some(&dest_a), |_, _| {})
            .expect("immediate compact Draw-23 modal must recover and solve deal");
    let tel_a = handle_a.join().expect("join compact solvable peer");
    assert!(!uncertain_a);

    let ReconOutcome::SolvedFromRoot { route, probes_used } = report_a.outcome else {
        panic!("expected SolvedFromRoot, got {:?}", report_a.outcome);
    };
    assert!(probes_used >= 1);
    assert_eq!(report_a.document.slots[51], CardSlot::Known(five_diamonds));
    assert!(
        report_a
            .document
            .slots
            .iter()
            .all(|s| matches!(s, CardSlot::Known(_)))
    );

    let compact_clicks = tel_a
        .actions
        .iter()
        .filter(|act| {
            matches!(
                act,
                WireAction::Click { qmp_x, qmp_y, hold }
                if *qmp_x == expected_compact_x
                    && *qmp_y == expected_compact_y
                    && *hold >= Duration::from_millis(450)
            )
        })
        .count();
    assert_eq!(compact_clicks, 1);
    assert!(!tel_a.actions.iter().any(|act| {
        matches!(act, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == expected_tall_x && *qmp_y == expected_tall_y)
    }));
    let undo_all_idx = tel_a
        .actions
        .iter()
        .position(|act| {
            matches!(act, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == expected_undo_x && *qmp_y == expected_undo_y)
        })
        .expect("Undo All click must follow Undo last move");
    let modal_click_idx = tel_a
        .actions
        .iter()
        .position(|act| {
            matches!(act, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == expected_compact_x && *qmp_y == expected_compact_y)
        })
        .expect("compact modal Undo last move click");
    assert!(modal_click_idx < undo_all_idx);

    let decoded_a = from_slice(&fs::read(&dest_a).expect("read solved JSON")).expect("valid JSON");
    verify_objective_route(&decoded_a.slots, &route, Objective::ClearBoard)
        .expect("verified root route after compact modal recovery");

    // 2. Subcase B: Dead-end deal (`Row 4` unreachable, `SlotId(51) = 5D`) with immediate compact
    //    modal concludes `UnsolvableBestPartial` with all 34 initial/stock cards `Known` (including `5D`).
    let dir_b = TempTestDir::create("tripeaks-qmp-compact-dead");
    let sock_b = dir_b.0.join("qmp.sock");
    let truth_b = build_draw_23_compact_no_match_deal_cards();
    let handle_b = spawn_mock_qmp_guest(
        sock_b.clone(),
        truth_b,
        PeerBehavior {
            modal_kind: ModalKind::CompactChallenge,
            immediate_modal_on_draw_23: true,
            ..PeerBehavior::default()
        },
    );
    let cancel_b = Arc::new(AtomicBool::new(false));
    let (report_b, _, _) =
        run_qmp_session_test(&sock_b, &cancel_b, fast_qmp_config(4), None, |_, _| {})
            .expect("dead-end immediate compact Draw-23 modal recovers cleanly");
    let tel_b = handle_b.join().expect("join compact dead-end peer");
    assert!(matches!(
        report_b.outcome,
        ReconOutcome::UnsolvableBestPartial {
            unreachable_slots: 18,
            ..
        }
    ));
    assert_eq!(report_b.document.slots[51], CardSlot::Known(five_diamonds));
    assert!(
        report_b.document.slots[18..]
            .iter()
            .all(|s| matches!(s, CardSlot::Known(_)))
    );
    assert!(tel_b.actions.iter().any(|act| {
        matches!(
            act,
            WireAction::Click { qmp_x, qmp_y, hold }
            if *qmp_x == expected_compact_x && *qmp_y == expected_compact_y && *hold >= Duration::from_millis(450)
        )
    }));

    // 3. Subcase C: Immediate tall Normal modal (`no-moves-remaining.png`, `SlotId(51) = AH`) on
    //    Draw 23 recovers `AH` via bottom-right rotated fallback and clicks `(1_128, 728)`.
    let dir_c = TempTestDir::create("tripeaks-qmp-tall-ah");
    let sock_c = dir_c.0.join("qmp.sock");
    let truth_c = build_draw_23_tall_ah_no_match_deal_cards();
    let handle_c = spawn_mock_qmp_guest(
        sock_c.clone(),
        truth_c,
        PeerBehavior {
            modal_kind: ModalKind::TallNormal,
            immediate_modal_on_draw_23: true,
            ..PeerBehavior::default()
        },
    );
    let cancel_c = Arc::new(AtomicBool::new(false));
    let (report_c, _, _) =
        run_qmp_session_test(&sock_c, &cancel_c, fast_qmp_config(4), None, |_, _| {})
            .expect("immediate tall Draw-23 modal recovers AH cleanly");
    let tel_c = handle_c.join().expect("join tall AH peer");
    assert_eq!(report_c.document.slots[51], CardSlot::Known(ace_hearts));
    assert!(tel_c.actions.iter().any(|act| {
        matches!(
            act,
            WireAction::Click { qmp_x, qmp_y, hold }
            if *qmp_x == expected_tall_x && *qmp_y == expected_tall_y && *hold >= Duration::from_millis(450)
        )
    }));

    // 4. Subcase D: Mid-RECON probe (`Undo All` #2) triggers compact Challenge modal over QMP ->
    //    two-step `(1_133, 605)` `POST_GAME_MOUSE_HOLD` -> `(1_320, 978)` `Undo All` -> `T = 0` -> solves!
    let dir_d = TempTestDir::create("tripeaks-qmp-probe-compact");
    let sock_d = dir_d.0.join("qmp.sock");
    let truth_d = build_stock_gated_deal_cards();
    let handle_d = spawn_mock_qmp_guest(
        sock_d.clone(),
        truth_d,
        PeerBehavior {
            modal_kind: ModalKind::CompactChallenge,
            modal_race_on_undo_all_index: Some(2),
            ..PeerBehavior::default()
        },
    );
    let cancel_d = Arc::new(AtomicBool::new(false));
    let (report_d, _, _) =
        run_qmp_session_test(&sock_d, &cancel_d, fast_qmp_config(8), None, |_, _| {})
            .expect("mid-RECON probe compact modal recovers cleanly");
    let tel_d = handle_d.join().expect("join probe compact peer");
    assert!(matches!(
        report_d.outcome,
        ReconOutcome::SolvedFromRoot { .. }
    ));
    assert!(tel_d.actions.iter().any(|act| {
        matches!(
            act,
            WireAction::Click { qmp_x, qmp_y, hold }
            if *qmp_x == expected_compact_x && *qmp_y == expected_compact_y && *hold >= Duration::from_millis(450)
        )
    }));

    // 5. Subcase E: Two-step `acquire_non_settling_scene` mid-RECON probe modal recovery
    //    (compact Challenge modal -> `(1_133, 605)` hold -> un-dimmed mid-probe board -> `(1_320, 978)` `Undo All` -> `T = 0`).
    struct ScriptedProbeModalIo {
        frames: VecDeque<CapturedFrame>,
        inputs: Vec<Input>,
    }
    impl OperationIo for ScriptedProbeModalIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.frames
                .pop_front()
                .ok_or_else(|| OperationError::Transport("out of scripted probe frames".to_owned()))
        }
        fn authorise(&mut self, f: &CapturedFrame, i: Input) -> Result<(), OperationError> {
            authorise_tripeaks_input(f, i)
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.inputs.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    let deal_e = build_cyclic_deal_cards();
    let slots_e = cards_to_slots(&deal_e);
    let root_pos_e = TriPeaksPosition::from_root(&slots_e).expect("cyclic root");
    let root_snap_e = snapshot_from_position(&deal_e, &root_pos_e, false);
    let root_frame_e = synthesize_gameplay_frame(&root_snap_e, BannerKind::Normal);
    let (mid_pos_e, _) = apply_move(&root_pos_e, &slots_e, TriPeaksMove::Draw).expect("first draw");
    let mid_snap_e = snapshot_from_position(&deal_e, &mid_pos_e, true);
    let mid_frame_e = synthesize_gameplay_frame(&mid_snap_e, BannerKind::Normal);

    let compact_modal = assets().no_moves_compact.clone();
    let frames_e = VecDeque::from([
        compact_modal.clone(),
        compact_modal,
        mid_frame_e.clone(),
        mid_frame_e,
        root_frame_e.clone(),
        root_frame_e,
    ]);
    let cancel_e = AtomicBool::new(false);
    let mut dom_e = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: frames_e,
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_e.root_snapshot = Some(root_snap_e.clone());
    dom_e.root_hash = Some(semantic_board_hash(&root_snap_e));
    let restored_e = dom_e
        .undo_all_and_hash()
        .expect("mid-probe compact modal recovers cleanly in two steps");
    assert_eq!(restored_e, semantic_board_hash(&root_snap_e));
    assert_eq!(
        dom_e.into_session().io().inputs,
        vec![
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
            Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point),
        ]
    );

    // 6. Subcase F: Fail-closed checks when darkened Waste is unreadable or back-out restores wrong card.
    let mut broken_waste_modal = assets().no_moves_compact.clone();
    for y in WASTE_CARD_BOUNDS.y..(WASTE_CARD_BOUNDS.y + 60) {
        for x in WASTE_CARD_BOUNDS.x..(WASTE_CARD_BOUNDS.x + 40) {
            let off = y as usize * broken_waste_modal.stride + x as usize * 4;
            broken_waste_modal.pixels[off..off + 3].copy_from_slice(&[51, 51, 52]);
        }
    }
    let mut dom_bad_waste = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([broken_waste_modal]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    let mut slots_bad = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let bad_err = dom_bad_waste
        .poll_stock_draw_step(
            STOCK_CARD_COUNT - 1,
            &root_snap_e,
            &mid_snap_e,
            &mut slots_bad,
        )
        .expect_err("unreadable darkened Waste must fail closed before clicking");
    assert!(matches!(
        bad_err,
        ReconError::VisionClassificationFailed { slot: SlotId(51) }
    ));
    assert!(dom_bad_waste.into_session().io().inputs.is_empty());

    // Wrong post-`Undo last move` card (e.g., restored to `T = 0` or wrong Waste instead of `mid_snap_e.waste`).
    let mut wrong_prev_snap = mid_snap_e.clone();
    wrong_prev_snap.waste = deal_e[usize::from(STOCK_START_SLOT) + 1];
    let wrong_prev_frame = synthesize_gameplay_frame(&wrong_prev_snap, BannerKind::Normal);
    let mut dom_wrong_back_out = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                wrong_prev_frame.clone(),
                wrong_prev_frame,
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    let mut slots_wrong = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let wrong_err = dom_wrong_back_out
        .poll_stock_draw_step(
            STOCK_CARD_COUNT - 1,
            &root_snap_e,
            &mid_snap_e,
            &mut slots_wrong,
        )
        .expect_err("wrong Waste after Draw-23 Undo last move must fail before Undo All");
    assert!(matches!(wrong_err, ReconError::SceneVerification { .. }));
    assert_eq!(
        dom_wrong_back_out.into_session().io().inputs,
        vec![Input::MouseLeftHold {
            point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
            hold: POST_GAME_MOUSE_HOLD,
        }]
    );

    // 7. Subcase G: End-to-end `AF_UNIX` QMP peer where a mid-RECON probe tableau tap immediately
    //    triggers the compact Challenge `"No Moves Remaining"` modal (`modal_on_probe_tap: Some(2)`):
    //    RECON confirms the modal twice, reads the dimmed newly uncovered Row-3 parent card, executes
    //    Step 1 `(1_133, 605)` `POST_GAME_MOUSE_HOLD`, executes Step 2 `(1_320, 978)` `Undo All` -> `T = 0`,
    //    and continues probing to `SolvedFromRoot`!
    let dir_g = TempTestDir::create("tripeaks-qmp-probe-immediate-compact");
    let sock_g = dir_g.0.join("qmp.sock");
    let dest_g = dir_g.0.join("probe-immediate-solved.json");
    let truth_g = build_stock_gated_deal_cards();
    let handle_g = spawn_mock_qmp_guest(
        sock_g.clone(),
        truth_g,
        PeerBehavior {
            modal_kind: ModalKind::CompactChallenge,
            modal_on_probe_tap: Some(2),
            ..PeerBehavior::default()
        },
    );
    let cancel_g = Arc::new(AtomicBool::new(false));
    let (report_g, _, uncertain_g) = run_qmp_session_test(
        &sock_g,
        &cancel_g,
        fast_qmp_config(10),
        Some(&dest_g),
        |_, _| {},
    )
    .expect("immediate mid-RECON probe compact modal must recover and solve");
    let tel_g = handle_g.join().expect("join immediate probe modal peer");
    assert!(!uncertain_g);
    let ReconOutcome::SolvedFromRoot { route: route_g, .. } = report_g.outcome else {
        panic!(
            "expected SolvedFromRoot after probe modal recovery, got {:?}",
            report_g.outcome
        );
    };
    let modal_click_g = tel_g
        .actions
        .iter()
        .position(|act| {
            matches!(
                act,
                WireAction::Click { qmp_x, qmp_y, hold }
                if *qmp_x == expected_compact_x && *qmp_y == expected_compact_y && *hold >= Duration::from_millis(450)
            )
        })
        .expect("probe modal must click compact Undo last move");
    let post_modal_undo_all = tel_g
        .actions
        .iter()
        .skip(modal_click_g + 1)
        .any(|act| {
            matches!(
                act,
                WireAction::Click { qmp_x, qmp_y, hold }
                if *qmp_x == expected_undo_x && *qmp_y == expected_undo_y && *hold < Duration::from_millis(200)
            )
        });
    assert!(
        post_modal_undo_all,
        "Step 2 Undo All must follow Step 1 Undo last move after multi-step probe modal"
    );
    let decoded_g = from_slice(&fs::read(&dest_g).expect("read solved JSON")).expect("valid JSON");
    verify_objective_route(&decoded_g.slots, &route_g, Objective::ClearBoard)
        .expect("verified root route after immediate probe modal recovery");

    // 8. Subcase H: Scripted state-machine edge coverage:
    //    (a) 2-step probe (`[Tap(4, 1), Tap(4, 2)]` from untouched `T = 0`) where tap #2 triggers
    //        immediate compact modal: reads dimmed uncovered Row-3 parent `slot 9` across two frames,
    //        waits through delayed XAML dismiss (`NoMovesModal` + dimmed fade-out -> `Settled` x 2)
    //        without re-clicking, restores pre-step `after_tap1`, then Step 2 `Undo All` -> `T = 0`;
    //    (b) Draw-23 fade-in (dimmed fade-in frame -> `NoMovesModal` x 2) recovers cleanly with 1 hold click;
    //    (c) Stuck modal refusing to clear after 1 `Undo last move` click fails closed without re-clicking.
    let deal_h = build_cyclic_deal_cards();
    let slots_h = cards_to_slots(&deal_h);
    let root_pos_h = TriPeaksPosition::from_root(&slots_h).expect("root pos H");
    let root_snap_h = snapshot_from_position(&deal_h, &root_pos_h, false);
    let root_frame_h = synthesize_gameplay_frame(&root_snap_h, BannerKind::Normal);

    let (tap1_pos_h, _) = apply_move(
        &root_pos_h,
        &slots_h,
        TriPeaksMove::TapTableau { row: 4, column: 1 },
    )
    .expect("tap1 legal");
    let tap1_snap_h = snapshot_from_position(&deal_h, &tap1_pos_h, true);
    let tap1_frame_h = synthesize_gameplay_frame(&tap1_snap_h, BannerKind::Normal);

    let (tap2_pos_h, _) = apply_move(
        &tap1_pos_h,
        &slots_h,
        TriPeaksMove::TapTableau { row: 4, column: 2 },
    )
    .expect("tap2 legal");
    let tap2_snap_h = snapshot_from_position(&deal_h, &tap2_pos_h, true);
    let modal_after_tap2 = synthesize_compact_modal_over_board(&tap2_snap_h, BannerKind::Normal);

    let mut fade_only_frame = tap1_frame_h.clone();
    for px in fade_only_frame.pixels.as_chunks_mut::<4>().0 {
        px[0] /= 3;
        px[1] /= 3;
        px[2] /= 3;
    }

    let mut dom_2_step = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                root_frame_h.clone(),     // probe_entry_verify
                tap1_frame_h.clone(),     // after tap1 settles
                modal_after_tap2.clone(), // after tap2 immediate modal #1
                modal_after_tap2.clone(), // modal confirm #2
                modal_after_tap2,         // delayed XAML dismiss poll #0 (still modal)
                fade_only_frame.clone(),  // delayed XAML dismiss poll #1 (dimmed fade-out)
                tap1_frame_h.clone(),     // un-dimmed tap1 settled #1
                tap1_frame_h.clone(),     // un-dimmed tap1 settled #2
                tap1_frame_h,             // undo_all_and_hash acquire_non_settling_scene
                root_frame_h.clone(),     // Undo All -> T = 0 #1
                root_frame_h,             // Undo All -> T = 0 #2
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_2_step.root_snapshot = Some(root_snap_h.clone());
    dom_2_step.root_hash = Some(semantic_board_hash(&root_snap_h));
    let observed_2_step = dom_2_step
        .execute_probe_and_observe(&[
            TriPeaksMove::TapTableau { row: 4, column: 1 },
            TriPeaksMove::TapTableau { row: 4, column: 2 },
        ])
        .expect("2-step immediate probe modal must recover cleanly")
        .expect("must return revealed Row-3 parent observation");
    assert_eq!(observed_2_step, vec![(SlotId(9), deal_h[9])]);
    assert_eq!(
        dom_2_step
            .undo_all_and_hash()
            .expect("Step 2 Undo All must restore T=0 cleanly after Step 1 back-out"),
        semantic_board_hash(&root_snap_h)
    );
    assert_eq!(
        dom_2_step.into_session().io().inputs,
        vec![
            Input::MouseLeft(TABLEAU_CARD_REGIONS[18].click_point),
            Input::MouseLeft(TABLEAU_CARD_REGIONS[19].click_point),
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
            Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point),
        ]
    );

    // Draw-23 fade-in (dimmed fade-in frame before `NoMovesModal` x 2) recovers cleanly with 1 hold click.
    let mid_frame_h = synthesize_gameplay_frame(&mid_snap_e, BannerKind::Normal);
    let mut dom_fade_in = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                fade_only_frame,
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                mid_frame_h.clone(),
                mid_frame_h,
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    let mut slots_fade = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let un_dimmed_fade = dom_fade_in
        .poll_stock_draw_step(
            STOCK_CARD_COUNT - 1,
            &root_snap_e,
            &mid_snap_e,
            &mut slots_fade,
        )
        .expect("Draw-23 fade-in frame before modal must recover cleanly");
    assert_eq!(un_dimmed_fade, mid_snap_e);
    assert_eq!(slots_fade[51], CardSlot::Known(five_diamonds));
    assert_eq!(
        dom_fade_in.into_session().io().inputs,
        vec![Input::MouseLeftHold {
            point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
            hold: POST_GAME_MOUSE_HOLD,
        }]
    );

    // Stuck modal that never clears after 1 `Undo last move` click must fail closed without re-clicking.
    let mut cfg_stuck = fast_qmp_config(2);
    cfg_stuck.max_restore_observations = 3;
    let mut dom_stuck = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        cfg_stuck,
        Box::new(|_| {}),
    );
    let stuck_err = dom_stuck
        .driver()
        .back_out_no_moves_modal(
            "stuck_modal_test",
            UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
            false,
        )
        .expect_err("stuck modal must fail closed after budget expiry");
    assert!(matches!(stuck_err, ReconError::SceneVerification { .. }));
    assert_eq!(
        dom_stuck.into_session().io().inputs.len(),
        1,
        "must never send a second click when modal refuses to clear"
    );

    // 9. Subcase I: Dimmed XAML scrim fade-in resilience & fail-closed bounds across all three
    //    reconnaissance entry points:
    //    (a) `acquire_non_settling_scene` / `restore_to_untouched_root` (`Draw23Sweep`) tolerates
    //        bounded dimmed fade frames before `NoMovesModal`, then executes two-step recovery;
    let root_frame_i = synthesize_gameplay_frame(&root_snap_e, BannerKind::Normal);
    let mid_frame_i = synthesize_gameplay_frame(&mid_snap_e, BannerKind::Normal);
    let root_frame_ih = synthesize_gameplay_frame(&root_snap_h, BannerKind::Normal);
    let tap1_frame_ih = synthesize_gameplay_frame(&tap1_snap_h, BannerKind::Normal);
    let modal_after_tap2_i = synthesize_compact_modal_over_board(&tap2_snap_h, BannerKind::Normal);
    let mut fade_only_i = tap1_frame_ih.clone();
    for px in fade_only_i.pixels.as_chunks_mut::<4>().0 {
        px[0] /= 3;
        px[1] /= 3;
        px[2] /= 3;
    }

    let mut dom_draw23_fade = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                fade_only_i.clone(),
                fade_only_i.clone(),
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                mid_frame_i.clone(),
                mid_frame_i.clone(),
                root_frame_i.clone(),
                root_frame_i.clone(),
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    let draw23_fade_restored = dom_draw23_fade
        .restore_to_untouched_root(
            "draw23_fade_in_recovery",
            RootRestoreTarget::Draw23Sweep {
                root: &root_snap_e,
                slot_50: mid_snap_e.waste,
            },
        )
        .expect("Draw23Sweep must wait through dimmed fade-in frames before compact modal");
    assert_eq!(draw23_fade_restored, root_snap_e);
    assert_eq!(
        dom_draw23_fade.into_session().io().inputs,
        vec![
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
            Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point),
        ]
    );

    //    (b) `DriverUndoAllIo::observe` treats bounded dimmed fade-in frames after `Undo All` as
    //        `UndoScene::Animating` until the delayed `NoMovesModal` settles, without extra clicks;
    let mut dom_undo_race_fade = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                mid_frame_i.clone(),
                fade_only_i.clone(),
                fade_only_i.clone(),
                assets().no_moves_compact.clone(),
                assets().no_moves_compact.clone(),
                mid_frame_i.clone(),
                mid_frame_i.clone(),
                root_frame_i.clone(),
                root_frame_i.clone(),
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_undo_race_fade.root_snapshot = Some(root_snap_e.clone());
    dom_undo_race_fade.root_hash = Some(semantic_board_hash(&root_snap_e));
    let undo_race_hash = dom_undo_race_fade
        .undo_all_and_hash()
        .expect("Undo All race with dimmed fade-in into delayed modal must recover cleanly");
    assert_eq!(undo_race_hash, semantic_board_hash(&root_snap_e));
    assert_eq!(
        dom_undo_race_fade.into_session().io().inputs,
        vec![
            Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point),
            Input::MouseLeftHold {
                point: UNDO_LAST_MOVE_COMPACT_CLICK_POINT,
                hold: POST_GAME_MOUSE_HOLD,
            },
            Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point),
        ]
    );

    //    (c) Persistent dimmed fade exceeding `MAX_DIMMED_FADE_OBSERVATIONS` fails closed cleanly,
    //        while genuine pure-black non-gameplay frames abort on the very first observation;
    let mut black_frame = root_frame_i.clone();
    black_frame.pixels.fill(0);
    for px in black_frame.pixels.as_chunks_mut::<4>().0 {
        px[3] = 255;
    }
    let mut cfg_fade_cap = fast_qmp_config(2);
    cfg_fade_cap.max_restore_observations = 12;
    let mut dom_persist_fade = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from(vec![fade_only_i.clone(); 10]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        cfg_fade_cap,
        Box::new(|_| {}),
    );
    let persist_err = dom_persist_fade
        .driver()
        .acquire_non_settling_scene("persist_dimmed_fade_fail_closed")
        .expect("bounded fade retries must return Unsupported rather than loop");
    assert!(matches!(persist_err, FrameClassification::Unsupported(_)));
    assert_eq!(
        dom_persist_fade.into_session().io().frames.len(),
        10 - (MAX_DIMMED_FADE_OBSERVATIONS + 1),
        "must stop immediately once MAX_DIMMED_FADE_OBSERVATIONS is reached"
    );

    let mut dom_black = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([mid_frame_i, black_frame, root_frame_i]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_black.root_snapshot = Some(root_snap_e.clone());
    dom_black.root_hash = Some(semantic_board_hash(&root_snap_e));
    let black_err = dom_black
        .undo_all_and_hash()
        .expect_err("black screen after Undo All must abort immediately on observation #1");
    assert!(matches!(black_err, ReconError::SceneVerification { .. }));
    let black_session = dom_black.into_session();
    let black_io = black_session.io();
    assert_eq!(black_io.frames.len(), 1);
    assert_eq!(
        black_io.inputs,
        vec![Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point)]
    );

    //    (d) Challenge-mode mid-RECON probe encountering dimmed fade-in before compact modal skips the
    //        5-capture Challenge Complete poll and recovers cleanly to `tap1_frame_ih`.
    let mut dom_challenge_fade = LiveTriPeaksDomain::new(
        Session::new(ScriptedProbeModalIo {
            frames: VecDeque::from([
                root_frame_ih,
                tap1_frame_ih.clone(),
                fade_only_i,
                modal_after_tap2_i.clone(),
                modal_after_tap2_i,
                tap1_frame_ih.clone(),
                tap1_frame_ih,
            ]),
            inputs: Vec::new(),
        }),
        &cancel_e,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_challenge_fade.challenge_mode = true;
    dom_challenge_fade.root_snapshot = Some(root_snap_h.clone());
    dom_challenge_fade.root_hash = Some(semantic_board_hash(&root_snap_h));
    let obs_challenge_fade = dom_challenge_fade
        .execute_probe_and_observe(&[
            TriPeaksMove::TapTableau { row: 4, column: 1 },
            TriPeaksMove::TapTableau { row: 4, column: 2 },
        ])
        .expect("Challenge probe encountering dimmed fade-in before modal must recover cleanly")
        .expect("must return revealed Row-3 parent observation");
    assert_eq!(obs_challenge_fade, vec![(SlotId(9), deal_h[9])]);
    assert!(dom_challenge_fade.into_session().io().frames.is_empty());
}

#[test]
fn stop_mid_sweep_search_and_probe_aborts_without_undo_all_or_json() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-stop");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("should-not-exist.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_stock_gated_deal_cards();

    // STOP fires after screendump #12 (mid-Stock sweep).
    let handle = spawn_mock_qmp_guest(
        socket.clone(),
        truth,
        PeerBehavior {
            stop_after_captures: Some((Arc::clone(&cancel), 12)),
            ..PeerBehavior::default()
        },
    );

    let err = run_qmp_session_test(&socket, &cancel, fast_qmp_config(6), Some(&dest), |_, _| {})
        .expect_err("STOP mid-sweep must cancel cleanly");
    let tel = handle.join().expect("join STOP server");
    assert!(matches!(
        err,
        TriPeaksReconFailure::Recon(ReconError::Cancelled)
    ));
    assert!(!dest.exists(), "STOP must never write destination JSON");

    let undo_x = (SHARED_UNDO_ALL_CONTROL.click_point.x as u32 * 32_767) / 1_920;
    let undo_y = (SHARED_UNDO_ALL_CONTROL.click_point.y as u32 * 32_767) / 1_080;
    assert!(
        !tel.actions
            .iter()
            .any(|a| matches!(a, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == undo_x && *qmp_y == undo_y)),
        "STOP must send zero Undo All cleanup clicks"
    );

    // Also test STOP during `plan_info_gain_probe` + `probe_entry` (`run_epistemic_recon` returns Cancelled, never UnsolvableBestPartial).
    struct NeverCalledIo;
    impl OperationIo for NeverCalledIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            unreachable!("cancelled search must never capture")
        }
        fn authorise(&mut self, _: &CapturedFrame, _: Input) -> Result<(), OperationError> {
            unreachable!()
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            unreachable!()
        }
        fn deliver(&mut self, _: Input) -> Result<(), OperationError> {
            unreachable!("cancelled search must never click")
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    let cancel_search = AtomicBool::new(true);
    let mut dom_search = LiveTriPeaksDomain::new(
        Session::new(NeverCalledIo),
        &cancel_search,
        fast_qmp_config(4),
        Box::new(|_| {}),
    );
    let mut partial_slots = cards_to_slots(&truth);
    for slot in &mut partial_slots[0..18] {
        slot_set_unknown(slot);
    }
    let plan_err = dom_search
        .plan_info_gain_probe(&partial_slots)
        .expect_err("cancelled planner must immediately return Cancelled");
    assert!(matches!(plan_err, ReconError::Cancelled));
    let probe_err = dom_search
        .execute_probe_and_observe(&[TriPeaksMove::Draw])
        .expect_err("cancelled probe must abort immediately");
    assert!(matches!(probe_err, ReconError::Cancelled));
}

/// Set one `CardSlot` reference cleanly without cluttering closures.
fn slot_set_unknown(slot: &mut CardSlot) {
    *slot = CardSlot::Unknown;
}

#[test]
fn uncertain_qmp_delivery_latches_permanently_without_replay_or_undo() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-uncertain");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("never-published.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_stock_gated_deal_cards();

    // Inject transport error on QMP `input-send-event` #5 (key packet during Stock Draw 3).
    let handle = spawn_mock_qmp_guest(
        socket.clone(),
        truth,
        PeerBehavior {
            fail_input_on_event_index: Some(5),
            ..PeerBehavior::default()
        },
    );

    let mut qmp = QmpClient::connect(&socket).expect("connect uncertain QMP socket");
    let (tx, _) = std::sync::mpsc::channel();
    let sink = WorkerEventSink {
        events: tx,
        latest_frame: LatestFrameSlot::default(),
        capture_context: Mutex::new(None),
    };
    let io = NativeOperations {
        qmp: &mut qmp,
        socket: &socket,
        events: &sink,
        cancel: cancel.as_ref(),
        active: || Ok(()),
        guard: authorise_tripeaks_input,
        last_timing: None,
    };
    let mut dom = LiveTriPeaksDomain::new(
        Session::new(io),
        cancel.as_ref(),
        fast_qmp_config(4),
        Box::new(|_| {}),
    );
    let err = run_tripeaks_recon(&mut dom, Some(&dest)).expect_err("must latch UncertainDelivery");
    let tel = handle.join().expect("join uncertain peer");
    assert!(matches!(
        err,
        TriPeaksReconFailure::Recon(ReconError::UncertainDelivery { .. })
    ));
    assert!(dom.is_uncertain_latched());
    assert!(!dest.exists());

    // Subsequent domain calls must remain permanently refused.
    assert!(matches!(
        dom.initialize_and_sweep_stock(),
        Err(ReconError::UncertainDelivery { .. })
    ));
    assert!(matches!(
        dom.undo_all_and_hash(),
        Err(ReconError::UncertainDelivery { .. })
    ));
    let input_events = tel
        .commands
        .iter()
        .filter(|c| c.as_str() == "input-send-event")
        .count();
    assert!(
        (6..=7).contains(&input_events),
        "expected 6..=7 input-send-event packets (2 clean draws=4, failed #5 + pipelined up #6, optional #7 release read before EPIPE on #6 shutdown), got {input_events}"
    );
    assert!(
        !tel.actions
            .iter()
            .any(|action| matches!(action, WireAction::Click { .. })),
        "must issue zero Undo All clicks after uncertain packet"
    );
}

#[test]
fn root_restore_mismatch_dirty_start_and_duplicate_refusals() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-mismatch");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("mismatch-no-json.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_stock_gated_deal_cards();

    // 1. Subcase A: Guest drifts untouched T=0 layout after `Undo All` -> `RootRestoreMismatch`.
    let handle = spawn_mock_qmp_guest(
        socket.clone(),
        truth,
        PeerBehavior {
            drift_root_after_undo_all: true,
            ..PeerBehavior::default()
        },
    );
    let err = run_qmp_session_test(&socket, &cancel, fast_qmp_config(4), Some(&dest), |_, _| {})
        .expect_err("drifted Undo All must fail with RootRestoreMismatch");
    let _ = handle.join().expect("join mismatch server");
    match err {
        TriPeaksReconFailure::Recon(ReconError::RootRestoreMismatch { expected, actual }) => {
            assert_ne!(expected, actual);
        }
        other => panic!("expected RootRestoreMismatch, got {other:?}"),
    }
    assert!(!dest.exists());

    // 2. Subcase B: Starting on `draw-first.png` (Undo All enabled) fails before any QMP input.
    struct SingleFrameIo(VecDeque<CapturedFrame>, Vec<Input>);
    impl OperationIo for SingleFrameIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.0
                .pop_front()
                .ok_or_else(|| OperationError::Transport("exhausted".to_owned()))
        }
        fn authorise(&mut self, _: &CapturedFrame, _: Input) -> Result<(), OperationError> {
            Ok(())
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.1.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    let cancel_b = AtomicBool::new(false);
    let mut dom_dirty = LiveTriPeaksDomain::new(
        Session::new(SingleFrameIo(
            VecDeque::from([assets().draw_first.clone()]),
            Vec::new(),
        )),
        &cancel_b,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    let dirty_err = run_tripeaks_recon(&mut dom_dirty, None).expect_err("must reject dirty start");
    assert!(matches!(
        dirty_err,
        TriPeaksReconFailure::Recon(ReconError::SceneVerification { .. })
    ));
    assert!(dom_dirty.into_session().io().1.is_empty());

    // 3. Subcase C: `check_single_deck_slot_binding` rejects duplicate canonical cards (`DuplicateCard`) and 51->52 complement triggers via `run_epistemic_recon`.
    let mut slots_51 = cards_to_slots(&truth);
    let hidden_peak = slots_51[2];
    slots_51[0] = CardSlot::Unknown;
    slots_51[2] = CardSlot::Unknown;

    let CardSlot::Known(already_waste) = slots_51[usize::from(INITIAL_WASTE_SLOT.0)] else {
        panic!("known waste");
    };
    let dup_err = check_single_deck_slot_binding(&slots_51, SlotId(0), already_waste)
        .expect_err("duplicate single-deck binding must refuse");
    assert!(matches!(
        dup_err,
        ReconError::DuplicateCard {
            first_slot: INITIAL_WASTE_SLOT,
            second_slot: SlotId(0),
            ..
        }
    ));

    // Also test real `LiveTriPeaksDomain` 51->52 complement deduction where guest keeps Peak 3 (`slot 2`) visually hidden while Peak 1 (`slot 0`) uncovers cleanly.
    let CardSlot::Known(peak1_card) = hidden_peak else {
        panic!("known peak");
    };
    let _ = peak1_card;
    let cyclic = build_cyclic_deal_cards();
    let cyclic_slots = cards_to_slots(&cyclic);
    let cyclic_pos = TriPeaksPosition::from_root(&cyclic_slots).expect("cyclic root");
    let cyclic_root_snap = snapshot_from_position(&cyclic, &cyclic_pos, false);
    let dir_ded = TempTestDir::create("tripeaks-51-52");
    let dest_ded = dir_ded.0.join("deduced-52.json");

    struct Deduct51Io {
        truth: [Card; DEAL_SLOT_COUNT],
        pos: TriPeaksPosition,
        history: Vec<TriPeaksMove>,
    }
    impl OperationIo for Deduct51Io {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            let snap = snapshot_from_position(&self.truth, &self.pos, !self.history.is_empty());
            Ok(synthesize_gameplay_frame(&snap, BannerKind::Normal))
        }
        fn authorise(&mut self, f: &CapturedFrame, i: Input) -> Result<(), OperationError> {
            authorise_tripeaks_input(f, i)
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, input: Input) -> Result<(), OperationError> {
            let slots = cards_to_slots(&self.truth);
            match input {
                Input::Key(SupportedKey::Draw) => {
                    let (n, _) =
                        apply_move(&self.pos, &slots, TriPeaksMove::Draw).expect("legal mock draw");
                    self.history.push(TriPeaksMove::Draw);
                    self.pos = n;
                }
                Input::MouseLeft(pt) if pt == SHARED_UNDO_ALL_CONTROL.click_point => {
                    self.history.clear();
                    self.pos = TriPeaksPosition::from_root(&slots).expect("root");
                }
                Input::MouseLeft(pt) => {
                    let (slot_idx, _) = TABLEAU_CARD_REGIONS
                        .iter()
                        .enumerate()
                        .find(|(_, r)| r.click_point == pt)
                        .expect("tableau click");
                    let (row, col) =
                        slot_to_tableau_coord(SlotId(slot_idx as u8)).expect("valid coord");
                    let mv = TriPeaksMove::TapTableau { row, column: col };
                    let (n, _) = apply_move(&self.pos, &slots, mv).expect("legal tap");
                    self.history.push(mv);
                    self.pos = n;
                }
                Input::MouseLeftHold { .. } => {}
            }
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    // Wrap `LiveTriPeaksDomain` in a thin `ReconDomain` proxy hiding Row 1 (`slot 0`) & Row 1 (`slot 2`) at start so 1 probe (exposing `0`, leaving `2` unprobed) deduces the 52nd card!
    struct Deduct51Domain<'a>(LiveTriPeaksDomain<'a, Deduct51Io>, [Card; DEAL_SLOT_COUNT]);
    impl ReconDomain for Deduct51Domain<'_> {
        type Move = TriPeaksMove;
        fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
            let (mut slots, hash) = self.0.initialize_and_sweep_stock()?;
            // Pre-populate 50 Known cards (`Row 2..4` + `Waste` + `Stock` + `slot 1`), leaving ONLY `slots[0]` & `slots[2]` Unknown!
            for (idx, dst) in slots
                .iter_mut()
                .enumerate()
                .take(TABLEAU_SLOT_COUNT)
                .skip(1)
            {
                if idx != 2 {
                    *dst = CardSlot::Known(self.1[idx]);
                }
            }
            slots[0] = CardSlot::Unknown;
            slots[2] = CardSlot::Unknown;
            Ok((slots, hash))
        }
        fn solve_known_subgraph(
            &self,
            slots: &[CardSlot],
            objective: Objective,
        ) -> Option<Vec<Self::Move>> {
            self.0.solve_known_subgraph(slots, objective)
        }
        fn plan_info_gain_probe(
            &self,
            _slots: &[CardSlot],
        ) -> Result<ProbePlan<Self::Move>, ReconError> {
            // Plan prefix touching Peak 1's subtree (`18..=27 -> 9..=17 -> 3, 4`) leaving Peak 3 (`slot 2`) covered:
            Ok(ProbePlan::Found(vec![
                TriPeaksMove::TapTableau { row: 4, column: 1 },
                TriPeaksMove::TapTableau { row: 4, column: 2 },
                TriPeaksMove::TapTableau { row: 4, column: 3 },
                TriPeaksMove::TapTableau { row: 4, column: 4 },
                TriPeaksMove::TapTableau { row: 4, column: 5 },
                TriPeaksMove::TapTableau { row: 4, column: 6 },
                TriPeaksMove::TapTableau { row: 4, column: 7 },
                TriPeaksMove::TapTableau { row: 4, column: 8 },
                TriPeaksMove::TapTableau { row: 4, column: 9 },
                TriPeaksMove::TapTableau { row: 4, column: 10 },
                TriPeaksMove::TapTableau { row: 3, column: 1 },
                TriPeaksMove::TapTableau { row: 3, column: 2 },
                TriPeaksMove::TapTableau { row: 3, column: 3 },
                TriPeaksMove::TapTableau { row: 3, column: 4 },
                TriPeaksMove::TapTableau { row: 3, column: 5 },
                TriPeaksMove::TapTableau { row: 3, column: 6 },
                TriPeaksMove::TapTableau { row: 3, column: 7 },
                TriPeaksMove::TapTableau { row: 3, column: 8 },
                TriPeaksMove::TapTableau { row: 3, column: 9 },
                TriPeaksMove::TapTableau { row: 2, column: 1 },
                TriPeaksMove::TapTableau { row: 2, column: 2 },
            ]))
        }
        fn execute_probe_and_observe(
            &mut self,
            probe: &[Self::Move],
        ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
            self.0.execute_probe_and_observe(probe)
        }
        fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
            self.0.undo_all_and_hash()
        }
        fn best_partial_route(
            &self,
            slots: &[CardSlot],
        ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
            self.0.best_partial_route(slots)
        }
    }

    let _ = cyclic_root_snap;
    let cancel_ded = AtomicBool::new(false);
    let mut wrapped = Deduct51Domain(
        LiveTriPeaksDomain::new(
            Session::new(Deduct51Io {
                truth: cyclic,
                pos: cyclic_pos,
                history: Vec::new(),
            }),
            &cancel_ded,
            fast_qmp_config(4),
            Box::new(|_| {}),
        ),
        cyclic,
    );
    let ded_outcome =
        run_epistemic_recon(&mut wrapped, Objective::ClearBoard, true, 4).expect("51->52 RECON");
    assert!(matches!(
        ded_outcome,
        ReconOutcome::SolvedFromRoot { probes_used: 1, .. }
    ));
    assert_eq!(wrapped.0.latest_slots()[2], CardSlot::Known(cyclic[2]));
    let doc_ded = TriPeaksDealDocument {
        provenance: DealProvenance::RestoredAtRoot,
        probes_used: 1,
        objective: Objective::ClearBoard,
        slots: wrapped.0.latest_slots(),
        planned_route: match ded_outcome {
            ReconOutcome::SolvedFromRoot { route, .. } => Some(route),
            _ => None,
        },
    };
    publish_to_path(&doc_ded, &dest_ded).expect("publish 51->52 deal");
}

#[test]
fn completed_during_recon_intercepts_challenge_and_normal_wins_without_extra_clicks() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-live-win");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("completed-live.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_stock_gated_deal_cards();

    // 1. Subcase A (QMP peer): T=0 has native `Fours cleared 4/12` ribbon -> probe triggers `Challenge Complete!` -> zero Continue or Undo All clicks!
    let handle = spawn_mock_qmp_guest(
        socket.clone(),
        truth,
        PeerBehavior {
            banner: Some(BannerKind::FoursFourOfTwelve),
            terminal_after_tap_count: Some((2, TerminalInject::ChallengeComplete)),
            ..PeerBehavior::default()
        },
    );
    let (report, _, _) =
        run_qmp_session_test(&socket, &cancel, fast_qmp_config(4), Some(&dest), |_, _| {})
            .expect("live Challenge Complete should return CompletedDuringRecon");
    let tel = handle.join().expect("join Challenge server");
    assert_eq!(
        report.objective,
        Objective::ClearRankQuota {
            rank: Rank::Four.value() - 1,
            remaining: 8,
        }
    );
    assert_eq!(
        report.detected_challenge,
        Some(TriPeaksChallenge::ClearRank {
            rank: Rank::Four,
            current: 4,
            target: 12,
        })
    );
    assert_eq!(
        report.outcome,
        ReconOutcome::CompletedDuringRecon { probes_used: 1 }
    );
    assert_eq!(
        report.document.provenance,
        DealProvenance::CompletedDuringRecon
    );
    assert!(
        report
            .document
            .planned_route
            .as_ref()
            .is_some_and(|r| r.len() == 2)
    );

    let cont_x = (959_u32 * 32_767) / 1_920;
    let cont_y = (880_u32 * 32_767) / 1_080;
    assert!(
        !tel.actions
            .iter()
            .any(|a| matches!(a, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == cont_x && *qmp_y == cont_y)),
        "Continue must NEVER be clicked on Challenge Complete"
    );
    let saved = from_slice(&fs::read(&dest).expect("read completed JSON")).expect("valid JSON");
    assert_eq!(saved.provenance, DealProvenance::CompletedDuringRecon);

    // 2. Subcase B & C (fast state-machine `OperationIo`): `XpCounting` handoff (`fours-complete.png`) and final-card `BoardWinOverlay` (`tripeaks-TP01.png`).
    struct ScriptedStepIo {
        frames: VecDeque<CapturedFrame>,
        inputs: Vec<Input>,
    }
    impl OperationIo for ScriptedStepIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.frames
                .pop_front()
                .ok_or_else(|| OperationError::Transport("out of scripted frames".to_owned()))
        }
        fn authorise(&mut self, _: &CapturedFrame, _: Input) -> Result<(), OperationError> {
            Ok(())
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.inputs.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    let deal = build_cyclic_deal_cards();
    let slots = cards_to_slots(&deal);
    let root_snap = snapshot_from_position(
        &deal,
        &TriPeaksPosition::from_root(&slots).expect("cyclic"),
        false,
    );
    let root_frame = synthesize_gameplay_frame(&root_snap, BannerKind::FoursFourOfTwelve);
    let xp_frame = assets().xp_counting.clone();

    let mut frames_xp = VecDeque::new();
    frames_xp.push_back(root_frame.clone()); // pre-check inside execute_probe_and_observe
    frames_xp.push_back(xp_frame.clone()); // immediate step observe sees Unsupported in challenge mode -> starts 5-frame poll
    for _ in 0..5 {
        frames_xp.push_back(xp_frame.clone());
    }
    let cancel_xp = AtomicBool::new(false);
    let mut cfg_xp = fast_qmp_config(2);
    cfg_xp.force_challenge_mode = true;
    let mut dom_xp = LiveTriPeaksDomain::new(
        Session::new(ScriptedStepIo {
            frames: frames_xp,
            inputs: Vec::new(),
        }),
        &cancel_xp,
        cfg_xp,
        Box::new(|_| {}),
    );
    dom_xp.root_snapshot = Some(root_snap.clone());
    dom_xp.root_hash = Some(semantic_board_hash(&root_snap));
    let xp_res = dom_xp
        .execute_probe_and_observe(&[TriPeaksMove::TapTableau { row: 4, column: 1 }])
        .expect("XpCounting handoff succeeds cleanly");
    assert_eq!(xp_res, None);
    assert_eq!(dom_xp.into_session().io().inputs.len(), 1);

    // Subcase C: Normal final-card (`prev_occupied == 1`) transitions to `BoardWinOverlay` (`TP01`).
    let mut one_left_snap = BoardSnapshot {
        tableau: [TableauSlotObs::Empty; TABLEAU_SLOT_COUNT],
        stock: StockObs::FaceDown,
        waste: deal[28],
        undo_all: ControlAvailability::Disabled,
    };
    one_left_snap.tableau[0] = TableauSlotObs::Card(deal[18]);
    let one_left_frame = synthesize_gameplay_frame(&one_left_snap, BannerKind::Normal);
    let mut frames_tp = VecDeque::new();
    frames_tp.push_back(one_left_frame.clone());
    frames_tp.push_back(assets().tp01.clone());
    frames_tp.push_back(assets().tp01.clone());

    let cancel_tp = AtomicBool::new(false);
    let mut dom_tp = LiveTriPeaksDomain::new(
        Session::new(ScriptedStepIo {
            frames: frames_tp,
            inputs: Vec::new(),
        }),
        &cancel_tp,
        fast_qmp_config(2),
        Box::new(|_| {}),
    );
    dom_tp.root_snapshot = Some(one_left_snap.clone());
    dom_tp.root_hash = Some(semantic_board_hash(&one_left_snap));
    let tp_res = dom_tp
        .execute_probe_and_observe(&[TriPeaksMove::TapTableau { row: 1, column: 1 }])
        .expect("final-card TP01 win overlay returns Ok(None)");
    assert_eq!(tp_res, None);
    assert_eq!(dom_tp.into_session().io().inputs.len(), 1);

    // Subcase D: Active Challenge refuses Normal `BoardWinOverlay` (`TP01`) cleanly.
    let mut cfg_tp_challenge = fast_qmp_config(2);
    cfg_tp_challenge.force_challenge_mode = true;
    let mut dom_tp_challenge = LiveTriPeaksDomain::new(
        Session::new(ScriptedStepIo {
            frames: VecDeque::from([one_left_frame, assets().tp01.clone(), assets().tp01.clone()]),
            inputs: Vec::new(),
        }),
        &cancel_tp,
        cfg_tp_challenge,
        Box::new(|_| {}),
    );
    dom_tp_challenge.root_snapshot = Some(one_left_snap);
    dom_tp_challenge.root_hash = Some(semantic_board_hash(&BoardSnapshot {
        tableau: {
            let mut t = [TableauSlotObs::Empty; TABLEAU_SLOT_COUNT];
            t[0] = TableauSlotObs::Card(deal[18]);
            t
        },
        stock: StockObs::FaceDown,
        waste: deal[28],
        undo_all: ControlAvailability::Disabled,
    }));
    let tp_challenge_err = dom_tp_challenge
        .execute_probe_and_observe(&[TriPeaksMove::TapTableau { row: 1, column: 1 }])
        .expect_err("Challenge mode must refuse Normal BoardWinOverlay");
    assert!(matches!(
        tp_challenge_err,
        ReconError::SceneVerification { .. }
    ));
}

#[test]
fn unsolvable_best_partial_and_destination_collision_preservation() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-unsolvable");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("unsolvable.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_unsolvable_peaks_deal_cards();

    let handle = spawn_mock_qmp_guest(socket.clone(), truth, PeerBehavior::default());
    let (report, _, _) =
        run_qmp_session_test(&socket, &cancel, fast_qmp_config(8), Some(&dest), |_, _| {})
            .expect("unsolvable deal completes cleanly with BestPartial JSON");
    let _ = handle.join().expect("join unsolvable peer");

    let ReconOutcome::UnsolvableBestPartial {
        best_partial_route,
        unreachable_slots,
        proof,
    } = report.outcome
    else {
        panic!("expected UnsolvableBestPartial, got {:?}", report.outcome);
    };
    assert!(unreachable_slots >= 3);
    assert_eq!(proof, PartialProof::Exhaustive);
    verify_move_sequence(&report.document.slots, &best_partial_route)
        .expect("best_partial_route valid on published slots");

    let original_bytes = fs::read(&dest).expect("read published unsolvable JSON");
    let decoded = from_slice(&original_bytes).expect("decode published unsolvable JSON");
    assert_eq!(
        decoded
            .slots
            .iter()
            .filter(|&&s| s == CardSlot::Unreachable)
            .count(),
        unreachable_slots
    );

    // Re-running targeting existing file must fail closed (`PreExistingDifferentBytes` or `MatchingBytesPresent` -> LinkFailed) and never overwrite `dest`!
    fs::write(&dest, b"{\"preexisting\":true}").expect("seed differing pre-existing destination");
    let preserved = fs::read(&dest).expect("read seeded destination");
    let sock_dup = dir.0.join("qmp-collision.sock");
    let handle2 = spawn_mock_qmp_guest(sock_dup.clone(), truth, PeerBehavior::default());
    let collision_err = run_qmp_session_test(
        &sock_dup,
        &cancel,
        fast_qmp_config(8),
        Some(&dest),
        |_, _| {},
    )
    .expect_err("no-overwrite collision must fail");
    let _ = handle2.join().expect("join collision peer");
    assert!(matches!(
        collision_err,
        TriPeaksReconFailure::Publication(PublishDealError::LinkFailed {
            destination_state: DestinationState::PreExistingDifferentBytes,
            ..
        })
    ));
    assert_eq!(
        fs::read(&dest).expect("read destination after collision"),
        preserved
    );
}

#[test]
fn undo_all_waits_through_mid_rewind_disabled_toolbar_and_compositor_stall() {
    assert_eq!(TriPeaksReconConfig::default().max_restore_observations, 40);

    let deal = build_cyclic_deal_cards();
    let slots = cards_to_slots(&deal);
    let root_pos = TriPeaksPosition::from_root(&slots).expect("valid root");
    let root_snap = snapshot_from_position(&deal, &root_pos, false);
    let expected_hash = semantic_board_hash(&root_snap);
    let root_frame = synthesize_gameplay_frame(&root_snap, BannerKind::Normal);

    // Mid-play snapshot (`Undo All` enabled) after drawing to the first Stock card.
    let (drawn_pos, _) =
        apply_move(&root_pos, &slots, TriPeaksMove::Draw).expect("initial draw succeeds");
    let drawn_snap = snapshot_from_position(&deal, &drawn_pos, true);
    let drawn_frame = synthesize_gameplay_frame(&drawn_snap, BannerKind::Normal);

    // Simulated mid-rewind capture matching Beast incident anatomy: `Undo All` is already `Disabled`,
    // Stock top card has only rewound to `x = 821..952` (exposing green tray `x = 953..968` and
    // flying blue stripes in the left white deck stack), and early Stock card still sits on Waste.
    let wrong_waste = deal[usize::from(STOCK_START_SLOT)];
    let mut mid_rewind_snap = root_snap.clone();
    mid_rewind_snap.waste = wrong_waste;
    let mut mid_rewind_frame = synthesize_gameplay_frame(&mid_rewind_snap, BannerKind::Normal);
    copy_rect(
        &mut mid_rewind_frame,
        PixelPoint::new(TARGET_STOCK.x as i32, TARGET_STOCK.y as i32),
        &assets().stock_empty,
        TARGET_STOCK,
    );
    paint_face_down_card(&mut mid_rewind_frame, 767, STOCK_CARD_BOUNDS.y, 186);
    for y in 686..860_u32 {
        for x in 769..806_u32 {
            let off = y as usize * mid_rewind_frame.stride + x as usize * 4;
            mid_rewind_frame.pixels[off..off + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    paint_face_down_card(&mut mid_rewind_frame, 821, STOCK_CARD_BOUNDS.y, 132);
    assert_eq!(
        observe_frame(&mid_rewind_frame).expect("mid-rewind classifies without error"),
        FrameClassification::Settling
    );

    // Full settled `23`-card `T = 0` frame carrying a non-root Waste card (`wrong_waste`).
    let settled_wrong_root_frame = synthesize_gameplay_frame(&mid_rewind_snap, BannerKind::Normal);
    assert!(matches!(
        observe_frame(&settled_wrong_root_frame).expect("settled wrong root frame"),
        FrameClassification::Settled(_)
    ));

    struct ScriptedUndoIo {
        frames: VecDeque<CapturedFrame>,
        inputs: Vec<Input>,
    }
    impl OperationIo for ScriptedUndoIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.frames
                .pop_front()
                .ok_or_else(|| OperationError::Transport("out of scripted undo frames".to_owned()))
        }
        fn authorise(&mut self, _: &CapturedFrame, _: Input) -> Result<(), OperationError> {
            Ok(())
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.inputs.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    // 1. Rewind sequence: 3 mid-rewind captures (`Settling`) + 3 stalled wrong-Waste full-root
    // frames (< MIN_CONFIRMED_DRIFT_STREAK = 4) + 2 matching `T = 0` frames -> succeeds on 1 click.
    let mut cfg = fast_qmp_config(4);
    cfg.max_restore_observations = 16;
    let frames = VecDeque::from([
        drawn_frame.clone(),
        mid_rewind_frame.clone(),
        mid_rewind_frame.clone(),
        mid_rewind_frame,
        settled_wrong_root_frame.clone(),
        settled_wrong_root_frame.clone(),
        settled_wrong_root_frame.clone(),
        root_frame.clone(),
        root_frame,
    ]);
    let cancel = AtomicBool::new(false);
    let mut domain = LiveTriPeaksDomain::new(
        Session::new(ScriptedUndoIo {
            frames,
            inputs: Vec::new(),
        }),
        &cancel,
        cfg,
        Box::new(|_| {}),
    );
    domain.root_snapshot = Some(root_snap.clone());
    domain.root_hash = Some(expected_hash);
    let restored_hash = domain
        .undo_all_and_hash()
        .expect("undo_all waits through mid-rewind Stock and 3-frame stall");
    assert_eq!(restored_hash, expected_hash);
    assert_eq!(
        domain.into_session().io().inputs,
        vec![Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point)]
    );

    // 2. Genuine drift: `4` consecutive settled mismatched full-root captures (`>= 4`) short-circuit
    // without exhausting `max_restore_observations` and expose the drifted semantic hash.
    let mut cfg_drift = fast_qmp_config(4);
    cfg_drift.max_restore_observations = 16;
    let drift_frames = VecDeque::from([
        drawn_frame,
        settled_wrong_root_frame.clone(),
        settled_wrong_root_frame.clone(),
        settled_wrong_root_frame.clone(),
        settled_wrong_root_frame,
    ]);
    let mut drift_domain = LiveTriPeaksDomain::new(
        Session::new(ScriptedUndoIo {
            frames: drift_frames,
            inputs: Vec::new(),
        }),
        &cancel,
        cfg_drift,
        Box::new(|_| {}),
    );
    drift_domain.root_snapshot = Some(root_snap);
    drift_domain.root_hash = Some(expected_hash);
    let actual_drift_hash = drift_domain
        .undo_all_and_hash()
        .expect("confirmed drift returns drifted semantic hash for RootRestoreMismatch check");
    assert_ne!(actual_drift_hash, expected_hash);
    assert!(drift_domain.into_session().io().frames.is_empty());
}

#[test]
fn bounded_probe_limits_during_live_recon_preserve_unknown_slots_in_published_json() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let dir = TempTestDir::create("tripeaks-qmp-bounded-probe");
    let socket = dir.0.join("qmp.sock");
    let dest = dir.0.join("bounded-probe.json");
    let cancel = Arc::new(AtomicBool::new(false));
    let truth = build_cyclic_deal_cards();

    let mut cfg = fast_qmp_config(4);
    cfg.search_limits = SearchLimits {
        max_states: 1_000,
        max_expansions: 1,
        ..DEFAULT_SEARCH_LIMITS
    };

    let handle = spawn_mock_qmp_guest(socket.clone(), truth, PeerBehavior::default());
    let (report, _, _) = run_qmp_session_test(&socket, &cancel, cfg, Some(&dest), |_, _| {})
        .expect("tight probe limit completes with BoundedBestPartial JSON");
    let _ = handle.join().expect("join mock QMP guest");

    assert!(matches!(
        report.outcome,
        ReconOutcome::BoundedBestPartial {
            unknown_slots: 18,
            unreachable_slots: 0,
            probe_stop: SearchStop::ExpansionLimit,
            proof: PartialProof::Bounded(SearchStop::ExpansionLimit),
            ..
        }
    ));

    let bytes = fs::read(&dest).expect("read bounded JSON artifact");
    let decoded = from_slice(&bytes).expect("decode bounded JSON artifact");
    assert_eq!(
        decoded
            .slots
            .iter()
            .filter(|&&slot| slot == CardSlot::Unknown)
            .count(),
        18
    );
    assert_eq!(
        decoded
            .slots
            .iter()
            .filter(|&&slot| slot == CardSlot::Unreachable)
            .count(),
        0
    );
}

/// Verify `observe_frame` classifies untouched `T = 0` boards with warm-ivory Challenge target
/// cards as `Settled` rather than looping `Settling`.
#[test]
fn observe_frame_settles_on_warm_ivory_challenge_target_tableau_cards() {
    let mut frame = assets().left_ace.clone();
    // Tint multiple playable Row 4 slots (Row 4 Col 1 = 2H, Row 4 Col 5 = KC) and Waste (2D) to
    // calibrated warm-ivory Challenge target paper with 2 px outer gold glow borders.
    for bounds in [
        TABLEAU_CARD_REGIONS[18].card_bounds,
        TABLEAU_CARD_REGIONS[22].card_bounds,
        WASTE_CARD_BOUNDS,
    ] {
        for dy in 0..bounds.height {
            for dx in 0..bounds.width {
                let idx = (bounds.y + dy) as usize * frame.stride + (bounds.x + dx) as usize * 4;
                if dx == 0 || dx + 1 == bounds.width {
                    frame.pixels[idx..idx + 4].copy_from_slice(&[254, 209, 4, 255]);
                } else if dx == 1 || dx + 2 == bounds.width {
                    frame.pixels[idx..idx + 4].copy_from_slice(&[221, 211, 172, 255]);
                } else if frame.pixels[idx] >= 245
                    && frame.pixels[idx + 1] >= 245
                    && frame.pixels[idx + 2] >= 245
                {
                    let rgb = if dx <= 3 || dx + 4 >= bounds.width {
                        [249, 245, 228, 255]
                    } else {
                        [252, 246, 230, 255]
                    };
                    frame.pixels[idx..idx + 4].copy_from_slice(&rgb);
                }
            }
        }
    }

    let classification = observe_frame(&frame).expect("observe warm-ivory Challenge T=0 frame");
    let FrameClassification::Settled(snapshot) = classification else {
        panic!("expected Settled for warm-ivory T=0 frame, got {classification:?}");
    };
    assert!(snapshot.is_untouched_root_layout());
    assert_eq!(snapshot.stock, StockObs::FaceDown);
    assert_eq!(snapshot.undo_all, ControlAvailability::Disabled);
    assert_eq!(
        snapshot.tableau[18],
        TableauSlotObs::Card(
            Card::try_from(xsar::games::cards::Card::new(
                Rank::Two,
                xsar::games::cards::Suit::Hearts,
            ))
            .expect("2H canonical")
        )
    );
    assert_eq!(
        snapshot.tableau[22],
        TableauSlotObs::Card(
            Card::try_from(xsar::games::cards::Card::new(
                Rank::King,
                xsar::games::cards::Suit::Clubs,
            ))
            .expect("KC canonical")
        )
    );
    assert_eq!(
        snapshot.waste,
        Card::try_from(xsar::games::cards::Card::new(
            Rank::Two,
            xsar::games::cards::Suit::Diamonds,
        ))
        .expect("2D canonical")
    );
}

/// Verify `observe_frame` and `undo_all_and_hash` treat transient mid-flight card transit,
/// mid-flip Stock/Waste transitions, and child-before-parent cover inversions on gameplay
/// frames as `Settling` / `Animating`, while aborting immediately on non-gameplay frames and
/// failing closed if gameplay ambiguity persists across the restoration budget.
#[test]
fn undo_all_waits_through_in_flight_card_transit_and_cover_inversion_and_fails_closed_on_persistent_ambiguity()
 {
    let deal = build_cyclic_deal_cards();
    let slots = cards_to_slots(&deal);
    let root_pos = TriPeaksPosition::from_root(&slots).expect("valid root");
    let root_snap = snapshot_from_position(&deal, &root_pos, false);
    let expected_hash = semantic_board_hash(&root_snap);
    let root_frame = synthesize_gameplay_frame(&root_snap, BannerKind::Normal);

    let (drawn_pos, _) =
        apply_move(&root_pos, &slots, TriPeaksMove::Draw).expect("initial draw succeeds");
    let drawn_snap = snapshot_from_position(&deal, &drawn_pos, true);
    let drawn_frame = synthesize_gameplay_frame(&drawn_snap, BannerKind::Normal);

    // 1. Mid-flight tableau transit matching live incident 2397959-0: A♣ flying diagonally back
    // from Waste toward Row 2 Col 3 across cleared Row 3 Cols 4 & 5 produces `band_mixed_conflict`
    // (`CardRead::Ambiguous`). `read_card` stays strict while `observe_frame` reports `Settling`.
    let mut mid_flight_snap = root_snap.clone();
    mid_flight_snap.tableau[5] = TableauSlotObs::Empty;
    mid_flight_snap.tableau[12] = TableauSlotObs::Empty;
    mid_flight_snap.tableau[13] = TableauSlotObs::Empty;
    mid_flight_snap.tableau[21] = TableauSlotObs::Empty;
    mid_flight_snap.tableau[22] = TableauSlotObs::Empty;
    mid_flight_snap.tableau[23] = TableauSlotObs::Empty;
    let mut in_flight_tableau_frame =
        synthesize_gameplay_frame(&mid_flight_snap, BannerKind::Normal);
    paint_face_up_card(&mut in_flight_tableau_frame, 826, 250, 136, Card(0));
    assert_eq!(
        read_card(
            &in_flight_tableau_frame,
            LocationRequest::tripeaks_play(3, 4)
        )
        .expect("R3C4 mixed transit"),
        CardRead::Ambiguous
    );
    assert_eq!(
        read_card(
            &in_flight_tableau_frame,
            LocationRequest::tripeaks_play(3, 5)
        )
        .expect("R3C5 mixed transit"),
        CardRead::Ambiguous
    );
    assert_eq!(
        observe_frame(&in_flight_tableau_frame).expect("observe mid-flight tableau"),
        FrameClassification::Settling
    );

    // 2. Mid-flip Stock & Waste transit (`CardRead::Ambiguous` on DRAW 1 & DRAW 2).
    let mut mid_flip_stock_waste_frame = root_frame.clone();
    for y in STOCK_CARD_BOUNDS.y..STOCK_CARD_BOUNDS.bottom() {
        for x in STOCK_CARD_BOUNDS.x..(STOCK_CARD_BOUNDS.x + 68) {
            let off = y as usize * mid_flip_stock_waste_frame.stride + x as usize * 4;
            mid_flip_stock_waste_frame.pixels[off..off + 4].copy_from_slice(&[20, 130, 75, 255]);
        }
    }
    for y in 767..779_u32 {
        for x in WASTE_CARD_BOUNDS.x..(WASTE_CARD_BOUNDS.x + 68) {
            let off = y as usize * mid_flip_stock_waste_frame.stride + x as usize * 4;
            mid_flip_stock_waste_frame.pixels[off..off + 4].copy_from_slice(&[20, 130, 75, 255]);
        }
    }
    assert_eq!(
        read_card(
            &mid_flip_stock_waste_frame,
            LocationRequest::tripeaks_draw(1)
        )
        .expect("Stock ambiguous mid-flip"),
        CardRead::Ambiguous
    );
    assert_eq!(
        read_card(
            &mid_flip_stock_waste_frame,
            LocationRequest::tripeaks_draw(2)
        )
        .expect("Waste ambiguous mid-flip"),
        CardRead::Ambiguous
    );
    assert_eq!(
        observe_frame(&mid_flip_stock_waste_frame).expect("observe Stock/Waste mid-flip"),
        FrameClassification::Settling
    );

    // 3. Cover-order animation lag: Row 4 children landed before parent Row 3 Col 1 flipped back
    // to FaceDown (`covered == true && obs != FaceDown`).
    let mut cover_inversion_snap = root_snap.clone();
    cover_inversion_snap.tableau[9] = TableauSlotObs::Card(deal[9]);
    let cover_inversion_frame =
        synthesize_gameplay_frame(&cover_inversion_snap, BannerKind::Normal);
    assert_eq!(
        observe_frame(&cover_inversion_frame).expect("observe cover-order transit"),
        FrameClassification::Settling
    );

    struct ScriptedTransitIo {
        frames: VecDeque<CapturedFrame>,
        inputs: Vec<Input>,
    }
    impl OperationIo for ScriptedTransitIo {
        fn check_active(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
            self.frames.pop_front().ok_or_else(|| {
                OperationError::Transport("out of scripted transit frames".to_owned())
            })
        }
        fn authorise(&mut self, _: &CapturedFrame, _: Input) -> Result<(), OperationError> {
            Ok(())
        }
        fn probe(&mut self) -> Result<(), OperationError> {
            Ok(())
        }
        fn deliver(&mut self, i: Input) -> Result<(), OperationError> {
            self.inputs.push(i);
            Ok(())
        }
        fn wait(&mut self, _: Duration) -> Result<(), OperationError> {
            Ok(())
        }
    }

    // A. Recovers cleanly in one Undo All click across all three transient rewind frames.
    let cancel = AtomicBool::new(false);
    let mut domain = LiveTriPeaksDomain::new(
        Session::new(ScriptedTransitIo {
            frames: VecDeque::from([
                drawn_frame.clone(),
                in_flight_tableau_frame.clone(),
                mid_flip_stock_waste_frame,
                cover_inversion_frame,
                root_frame.clone(),
                root_frame,
            ]),
            inputs: Vec::new(),
        }),
        &cancel,
        fast_qmp_config(4),
        Box::new(|_| {}),
    );
    domain.root_snapshot = Some(root_snap.clone());
    domain.root_hash = Some(expected_hash);
    let restored = domain
        .undo_all_and_hash()
        .expect("undo_all settles across in-flight card transit and cover inversion");
    assert_eq!(restored, expected_hash);
    assert_eq!(
        domain.into_session().io().inputs,
        vec![Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point)]
    );

    // B. Non-gameplay overlay during Undo All still aborts immediately (`UndoScene::Unknown`).
    let mut non_gameplay_domain = LiveTriPeaksDomain::new(
        Session::new(ScriptedTransitIo {
            frames: VecDeque::from([
                drawn_frame.clone(),
                assets().xp_counting.clone(),
                in_flight_tableau_frame.clone(),
            ]),
            inputs: Vec::new(),
        }),
        &cancel,
        fast_qmp_config(4),
        Box::new(|_| {}),
    );
    non_gameplay_domain.root_snapshot = Some(root_snap.clone());
    non_gameplay_domain.root_hash = Some(expected_hash);
    let non_gameplay_err = non_gameplay_domain
        .undo_all_and_hash()
        .expect_err("non-gameplay scene must abort Undo All immediately");
    assert!(
        matches!(
            &non_gameplay_err,
            ReconError::SceneVerification { detail }
                if detail.contains("Unexpected scene after Undo All; restoration aborted")
        ),
        "unexpected non-gameplay error: {non_gameplay_err:?}"
    );
    assert_eq!(non_gameplay_domain.into_session().io().frames.len(), 1);

    // C. Persistent stationary gameplay ambiguity fails closed at budget expiry with 1 click.
    let mut persistent_cfg = fast_qmp_config(4);
    persistent_cfg.max_restore_observations = 4;
    let mut persistent_domain = LiveTriPeaksDomain::new(
        Session::new(ScriptedTransitIo {
            frames: VecDeque::from([
                drawn_frame,
                in_flight_tableau_frame.clone(),
                in_flight_tableau_frame.clone(),
                in_flight_tableau_frame.clone(),
                in_flight_tableau_frame.clone(),
                in_flight_tableau_frame,
            ]),
            inputs: Vec::new(),
        }),
        &cancel,
        persistent_cfg,
        Box::new(|_| {}),
    );
    persistent_domain.root_snapshot = Some(root_snap);
    persistent_domain.root_hash = Some(expected_hash);
    let persistent_err = persistent_domain
        .undo_all_and_hash()
        .expect_err("persistent gameplay ambiguity must fail closed once budget expires");
    assert!(
        matches!(
            &persistent_err,
            ReconError::SceneVerification { detail }
                if detail.contains("Undo All did not prove original-board restoration twice")
        ),
        "unexpected persistent ambiguity error: {persistent_err:?}"
    );
    assert_eq!(
        persistent_domain.into_session().io().inputs,
        vec![Input::MouseLeft(SHARED_UNDO_ALL_CONTROL.click_point)]
    );
}

#[test]
fn qmp_peer_reactive_continuation_chains_newly_flipped_cards_and_forks_across_probes() {
    let _backend = TestCaptureBackend::legacy_qmp();
    let expected_undo_qmp_x = (SHARED_UNDO_ALL_CONTROL.click_point.x as u32 * 32_767) / 1_920;
    let expected_undo_qmp_y = (SHARED_UNDO_ALL_CONTROL.click_point.y as u32 * 32_767) / 1_080;

    // 1. Subcase A: Forked 52-card deal where Fork A (`18=2 -> 19=3 -> [9=4] -> 20=5 -> [10=4] -> [3=4]`)
    //    chains across Row 3 & Row 2 inside a single reactive probe, while mutually isolated Fork B
    //    (`26=K -> 27=Q -> [17=K]`) forces a separate probe from `T = 0`.
    let mut used = [false; DEAL_SLOT_COUNT];
    let mut suit_cursor = [0_u8; 13];
    let mut take_card = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank {rank_idx} exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };
    let mut truth_a = [Card(0); DEAL_SLOT_COUNT];
    truth_a[usize::from(INITIAL_WASTE_SLOT.0)] = take_card(0); // Ace
    // Fork A: 18(2) -> 19(3) -> [9=4] -> 20(5) -> [10=4] -> [uncovers 3=4]
    truth_a[18] = take_card(1);
    truth_a[19] = take_card(2);
    truth_a[9] = take_card(3);
    truth_a[20] = take_card(4);
    truth_a[10] = take_card(3);
    truth_a[3] = take_card(3);
    // Fork B: 26(K) -> 27(Q) -> [uncovers 17=K]
    truth_a[26] = take_card(12);
    truth_a[27] = take_card(11);
    truth_a[17] = take_card(12);
    // Alternating [Q, 2, Q, 2, Q] barrier across Row 4 [21..=25] prevents bridging between Fork A and Fork B.
    for (slot_idx, rank_idx) in [(21, 11), (22, 1), (23, 11), (24, 1), (25, 11)] {
        truth_a[slot_idx] = take_card(rank_idx);
    }
    // Stock [29..52] holds all 23 remaining cards of ranks 4..=9 (Five..Ten), non-adjacent to Ace/King/Queen/Two.
    let mut stock_idx = usize::from(STOCK_START_SLOT);
    for _ in 0..3 {
        truth_a[stock_idx] = take_card(4);
        stock_idx += 1;
    }
    for rank_idx in 5_u8..=9 {
        for _ in 0..4 {
            truth_a[stock_idx] = take_card(rank_idx);
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    let rem_covered_slots = [0, 1, 2, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16];
    let mut slot_iter = rem_covered_slots.into_iter();
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            let dst = slot_iter.next().expect("exact 14 remaining covered slots");
            truth_a[dst] = Card(id as u8);
        }
    }
    assert!(slot_iter.next().is_none());

    let dir_a = TempTestDir::create("tripeaks-qmp-reactive-forks");
    let sock_a = dir_a.0.join("qmp.sock");
    let handle_a = spawn_mock_qmp_guest(sock_a.clone(), truth_a, PeerBehavior::default());
    let cancel_a = Arc::new(AtomicBool::new(false));
    let (report_a, _, uncertain_a) =
        run_qmp_session_test(&sock_a, &cancel_a, fast_qmp_config(6), None, |_, _| {})
            .expect("forked reactive QMP RECON finishes cleanly");
    let tel_a = handle_a.join().expect("join forked peer");
    assert!(!uncertain_a);
    let ReconOutcome::UnsolvableBestPartial { .. } = report_a.outcome else {
        panic!(
            "expected UnsolvableBestPartial after exploring both isolated forks, got {:?}",
            report_a.outcome
        );
    };
    assert_eq!(report_a.document.slots[9], CardSlot::Known(truth_a[9]));
    assert_eq!(report_a.document.slots[10], CardSlot::Known(truth_a[10]));
    assert_eq!(report_a.document.slots[3], CardSlot::Known(truth_a[3]));
    assert_eq!(report_a.document.slots[17], CardSlot::Known(truth_a[17]));
    assert!(report_a.document.probes_used >= 2);

    // Fork A clicks (`18, 19, 9, 20, 10`) execute together without an intermediate `Undo All` click between `19` and `9` or `20` and `10`.
    let qmp_coord = |slot_idx: usize| -> (u32, u32) {
        let pt = TABLEAU_CARD_REGIONS[slot_idx].click_point;
        (
            (pt.x as u32 * 32_767) / 1_920,
            (pt.y as u32 * 32_767) / 1_080,
        )
    };
    let (q19_x, q19_y) = qmp_coord(19);
    let (q10_x, q10_y) = qmp_coord(10);
    let pos_19 = tel_a
        .actions
        .iter()
        .position(|a| matches!(a, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == q19_x && *qmp_y == q19_y))
        .expect("Fork A clicks slot 19");
    let pos_10 = tel_a
        .actions
        .iter()
        .position(|a| matches!(a, WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == q10_x && *qmp_y == q10_y))
        .expect("Fork A chains online through newly revealed slot 9 to tap slot 10");
    assert!(pos_19 < pos_10);
    assert!(
        !tel_a.actions[pos_19..pos_10].iter().any(|a| matches!(
            a,
            WireAction::Click { qmp_x, qmp_y, .. } if *qmp_x == expected_undo_qmp_x && *qmp_y == expected_undo_qmp_y
        )),
        "must not reset via Undo All between discovering slot 9 and chaining through slot 10"
    );

    // 2. Subcase B: Multi-board Challenge (`Fours cleared 4/12`, `ClearRankQuota { rank: 3, remaining: 8 }`)
    //    over solvable `build_stock_gated_deal_cards()` escalates to `ClearBoard` inside `solve_known_subgraph`.
    let dir_b = TempTestDir::create("tripeaks-qmp-challenge-clear-board");
    let sock_b = dir_b.0.join("qmp.sock");
    let truth_b = build_stock_gated_deal_cards();
    let handle_b = spawn_mock_qmp_guest(
        sock_b.clone(),
        truth_b,
        PeerBehavior {
            banner: Some(BannerKind::FoursFourOfTwelve),
            ..PeerBehavior::default()
        },
    );
    let cancel_b = Arc::new(AtomicBool::new(false));
    let (report_b, _, uncertain_b) =
        run_qmp_session_test(&sock_b, &cancel_b, fast_qmp_config(4), None, |_, _| {})
            .expect("Challenge ClearBoard escalation solves inside RECON");
    let _tel_b = handle_b.join().expect("join Challenge escalation peer");
    assert!(!uncertain_b);
    assert_eq!(
        report_b.objective,
        Objective::ClearRankQuota {
            rank: 3,
            remaining: 8,
        }
    );
    let ReconOutcome::SolvedFromRoot { route, probes_used } = report_b.outcome else {
        panic!(
            "expected Challenge ClearBoard escalation to return SolvedFromRoot, got {:?}",
            report_b.outcome
        );
    };
    assert_eq!(probes_used, 1);
    verify_objective_route(&report_b.document.slots, &route, Objective::ClearBoard)
        .expect("escalated ClearBoard Challenge route replays cleanly");
}
