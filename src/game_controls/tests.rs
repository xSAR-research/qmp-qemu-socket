//! Unit and integration tests for shared `game_controls` card reading and Challenge Complete polling.

use std::{collections::VecDeque, time::Duration};

use super::*;
use crate::{
    capture::{PixelFormat, decode_png},
    freecell::{CELL_X, FOUNDATION_X, PLAY_X},
    freecell_recon::{objective, recognise_checked},
    game_operations::{CardRead, Input, OperationError, OperationIo},
    pyramid_recon::vision::{ReconScene, ReconVision, StockFace},
};
use xsar::games::{
    cards::{Card, Rank, Suit},
    freecell::{INITIAL_LENGTHS, Metadata},
};

/// Construct a genuine foreground-engine ambiguous corner (`6D` blended with `8D` from `FC13`) at `(x, y)`.
fn paint_foreground_ambiguous_6d_8d(frame: &mut CapturedFrame, x: u32, y: u32) {
    let reference =
        decode_png(include_bytes!("../../tests/fixtures/freecell-FC13.png")).expect("FC13 png");
    // Copy intact 6D suit + white rank background from `(795, 490)` onto `(x, y)`.
    for row in 4..53_u32 {
        for col in 3..34_u32 {
            let from = (490 + row) as usize * reference.stride + (795 + col) as usize * 4;
            let to = (y + row) as usize * frame.stride + (x + col) as usize * 4;
            frame.pixels[to..to + 4].copy_from_slice(&reference.pixels[from..from + 4]);
        }
    }
    // Average red ink channels across 6D `(795, 490)` and 8D `(1180, 543)` inside the rank box `(x+3..x+30, y+4..y+31)`.
    for row in 4..31_u32 {
        for col in 3..30_u32 {
            let off_a = (490 + row) as usize * reference.stride + (795 + col) as usize * 4;
            let off_b = (543 + row) as usize * reference.stride + (1180 + col) as usize * 4;
            let to = (y + row) as usize * frame.stride + (x + col) as usize * 4;
            for c in 0..3 {
                let avg = (u16::from(reference.pixels[off_a + c])
                    + u16::from(reference.pixels[off_b + c]))
                    / 2;
                frame.pixels[to + c] = avg as u8;
            }
            frame.pixels[to + 3] = 255;
        }
    }
}

/// Paint a solid RGB rectangle into an owned `CapturedFrame`.
fn paint_rect(frame: &mut CapturedFrame, rect: PixelRect, rgb: [u8; 3]) {
    for yy in rect.y..rect.y + rect.height {
        for xx in rect.x..rect.x + rect.width {
            let idx = yy as usize * frame.stride + xx as usize * 4;
            frame.pixels[idx..idx + 3].copy_from_slice(&rgb);
            frame.pixels[idx + 3] = 255;
        }
    }
}

/// Scripted `OperationIo` recording every lifecycle call, wait duration, and forbidden input attempt.
struct ScriptedIo {
    /// Queued outcomes returned by `capture` (either frame or `OperationError`).
    captures: VecDeque<Result<CapturedFrame, OperationError>>,
    /// Check number on which `check_active` returns `Err(Cancelled)`.
    fail_active_on: Option<usize>,
    /// Wait call number (1-based) on which `wait` fails.
    fail_wait_on: Option<usize>,
    /// Number of `check_active` calls observed.
    active_calls: usize,
    /// Number of `capture` calls observed.
    capture_calls: usize,
    /// Recorded wait durations in chronological order.
    waits: Vec<Duration>,
    /// Count of any input-delivery/probe/authorisation attempts (must remain zero).
    input_calls: usize,
}

impl ScriptedIo {
    /// Build a scripted `OperationIo` from queued capture results.
    fn new(captures: Vec<Result<CapturedFrame, OperationError>>) -> Self {
        Self {
            captures: captures.into(),
            fail_active_on: None,
            fail_wait_on: None,
            active_calls: 0,
            capture_calls: 0,
            waits: Vec::new(),
            input_calls: 0,
        }
    }
}

impl OperationIo for ScriptedIo {
    fn check_active(&mut self) -> Result<(), OperationError> {
        self.active_calls += 1;
        if self.fail_active_on == Some(self.active_calls) {
            return Err(OperationError::Cancelled);
        }
        Ok(())
    }

    fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
        self.capture_calls += 1;
        self.captures.pop_front().unwrap_or_else(|| {
            Err(OperationError::Transport(
                "no scripted frame left".to_owned(),
            ))
        })
    }

    fn authorise(&mut self, _frame: &CapturedFrame, _input: Input) -> Result<(), OperationError> {
        self.input_calls += 1;
        Err(OperationError::Unsupported(
            "unexpected authorise".to_owned(),
        ))
    }

    fn probe(&mut self) -> Result<(), OperationError> {
        self.input_calls += 1;
        Err(OperationError::Unsupported("unexpected probe".to_owned()))
    }

    fn deliver(&mut self, _input: Input) -> Result<(), OperationError> {
        self.input_calls += 1;
        Err(OperationError::Unsupported("unexpected deliver".to_owned()))
    }

    fn wait(&mut self, duration: Duration) -> Result<(), OperationError> {
        self.waits.push(duration);
        if self.fail_wait_on == Some(self.waits.len()) {
            return Err(OperationError::Cancelled);
        }
        Ok(())
    }
}

/// Read every Free Cell initial cascade, cell, suit slot, and incident capture through `read_card`.
#[test]
fn read_card_reads_all_freecell_fixtures_and_new_incident_frames() {
    let fixtures: [(&[u8], &str); 7] = [
        (
            include_bytes!("../../tests/fixtures/freecell-FC13.png"),
            "FC13",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-FC01.png"),
            "FC01",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-challenge-highlight-20261109.png"),
            "highlighted",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-native-eight-20261112.png"),
            "8D-incident",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-native-nine-20261112.png"),
            "9D-incident",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-sixes-three-diamond-20261113.png"),
            "3D-incident",
        ),
        (
            include_bytes!("../../tests/fixtures/freecell-solve-eight-club-20261113.png"),
            "8C-incident",
        ),
    ];

    for (bytes, name) in fixtures {
        let frame = decode_png(bytes).expect("fixture png");
        let deal = recognise_checked(
            &frame,
            Metadata::new("gc", name, "unit").expect("valid metadata"),
            &mut || Ok(()),
        )
        .unwrap_or_else(|e| panic!("{name} deal failed: {e}"));
        let mut seen = Vec::new();
        for (col_idx, &len) in INITIAL_LENGTHS.iter().enumerate() {
            for row_idx in 0..len {
                let req = LocationRequest::freecell_play(
                    u8::try_from(col_idx + 1).expect("col fits u8"),
                    u8::try_from(row_idx + 1).expect("row fits u8"),
                );
                let got = read_card(&frame, req)
                    .unwrap_or_else(|e| panic!("{name} ({col_idx},{row_idx}) err: {e}"));
                let expected = deal.board().cascades()[col_idx]
                    .iter()
                    .nth(row_idx)
                    .expect("card at row");
                assert_eq!(
                    got,
                    CardRead::Card(expected),
                    "{name} col={col_idx} row={row_idx}"
                );
                seen.push(expected);
            }
        }
        assert_eq!(seen.len(), 52);
        for slot in 1..=4_u8 {
            assert_eq!(
                read_card(&frame, LocationRequest::freecell_cell(slot)).expect("cell read"),
                CardRead::Empty,
                "{name} CELL {slot}"
            );
            assert_eq!(
                read_card(&frame, LocationRequest::freecell_suit(slot)).expect("suit read"),
                CardRead::Empty,
                "{name} SUIT {slot}"
            );
        }
    }

    let three_d = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-sixes-three-diamond-20261113.png"
    ))
    .expect("three_d png");
    let deal_3d = recognise_checked(
        &three_d,
        Metadata::new("gc", "3d", "unit").expect("valid metadata"),
        &mut || Ok(()),
    )
    .expect("3d deal");
    assert_eq!(
        objective::from_metadata(deal_3d.metadata()),
        objective::Objective::Unresolved
    );
    assert!((0..8).any(|c| {
        deal_3d.board().cascades()[c]
            .iter()
            .any(|card| card == Card::new(Rank::Three, Suit::Diamonds))
    }));

    let eight_c = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-solve-eight-club-20261113.png"
    ))
    .expect("eight_c png");
    let deal_8c = recognise_checked(
        &eight_c,
        Metadata::new("gc", "8c", "unit").expect("valid metadata"),
        &mut || Ok(()),
    )
    .expect("8c deal");
    assert_eq!(
        objective::from_metadata(deal_8c.metadata()),
        objective::Objective::Unresolved
    );
    assert!((0..8).any(|c| {
        deal_8c.board().cascades()[c]
            .iter()
            .any(|card| card == Card::new(Rank::Eight, Suit::Clubs))
    }));
}

/// Exercise upper `CELL`/`SUIT` Free Cell reads and full Pyramid `PLAY`/`DRAW` compatibility.
#[test]
fn read_card_handles_freecell_upper_cards_and_pyramid_locations() {
    let fc19 = decode_png(include_bytes!("../../tests/fixtures/freecell-FC19.png")).expect("FC19");
    // Verify occupied/empty CELL and SUIT queries cleanly without requiring scene gates.
    for slot in 1..=4_u8 {
        let cell = read_card(&fc19, LocationRequest::freecell_cell(slot)).expect("FC19 cell");
        let suit = read_card(&fc19, LocationRequest::freecell_suit(slot)).expect("FC19 suit");
        assert!(matches!(
            cell,
            CardRead::Card(_) | CardRead::Empty | CardRead::Unreadable
        ));
        assert!(matches!(
            suit,
            CardRead::Card(_) | CardRead::Empty | CardRead::Unreadable
        ));
    }
    let fc13 = decode_png(include_bytes!("../../tests/fixtures/freecell-FC13.png")).expect("FC13");
    let mut upper = fc13.clone();
    // Blit FC13's `JC` (`PLAY 3 row 6`, (603, 597)) into `CELL 2` `(CELL_X[1], 112)` -> shifted `[x, x-1]`.
    for y in 0..53_u32 {
        for x in 0..38_u32 {
            let from = (597 + y) as usize * fc13.stride + (603 + x) as usize * 4;
            let to = (112 + y) as usize * upper.stride + (CELL_X[1] - 1 + x) as usize * 4;
            upper.pixels[to..to + 4].copy_from_slice(&fc13.pixels[from..from + 4]);
        }
    }
    assert_eq!(
        read_card(&upper, LocationRequest::freecell_cell(2)).expect("upper CELL 2"),
        CardRead::Card(Card::new(Rank::Jack, Suit::Clubs))
    );
    // Blit `AH` (`PLAY 1 row 5`, (218, 543)) into `SUIT 3` `(FOUNDATION_X[2], 112)`.
    for y in 0..53_u32 {
        for x in 0..38_u32 {
            let from = (543 + y) as usize * fc13.stride + (218 + x) as usize * 4;
            let to = (112 + y) as usize * upper.stride + (FOUNDATION_X[2] + x) as usize * 4;
            upper.pixels[to..to + 4].copy_from_slice(&fc13.pixels[from..from + 4]);
        }
    }
    assert_eq!(
        read_card(&upper, LocationRequest::freecell_suit(3)).expect("upper SUIT 3"),
        CardRead::Card(Card::new(Rank::Ace, Suit::Hearts))
    );

    let vision = ReconVision::bundled_native_v23().expect("bundled Pyramid vision");
    let pyr_start = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-recon/originals/current-start.png"
    ))
    .expect("Pyramid current-start");
    let ReconScene::Board(obs) = vision.observe(&pyr_start).expect("Pyramid board observe") else {
        panic!("expected Board scene for Pyramid current-start");
    };
    let mut idx = 0_usize;
    for row in 1..=7_u8 {
        for col in 1..=row {
            assert_eq!(
                read_card(&pyr_start, LocationRequest::pyramid_play(row, col))
                    .expect("Pyramid PLAY"),
                CardRead::Card(obs.tableau[idx]),
                "Pyramid PLAY {row}|{col}"
            );
            idx += 1;
        }
    }
    let stock_read = |face| match face {
        StockFace::Card(card) => CardRead::Card(card),
        StockFace::Empty => CardRead::Empty,
    };
    assert_eq!(
        read_card(&pyr_start, LocationRequest::pyramid_draw(1)).expect("Pyramid DRAW 1"),
        stock_read(obs.left)
    );
    assert_eq!(
        read_card(&pyr_start, LocationRequest::pyramid_draw(2)).expect("Pyramid DRAW 2"),
        stock_read(obs.right)
    );

    let empty_draw = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-recon/originals/issue1-06.png"
    ))
    .expect("Pyramid empty draw fixture");
    let left_draw =
        read_card(&empty_draw, LocationRequest::pyramid_draw(1)).expect("empty Pyramid DRAW 1");
    assert_eq!(left_draw, CardRead::Empty);
}

/// Refuse ambiguous, unreadable, out-of-bounds, unsupported, and malformed-frame requests cleanly.
#[test]
fn read_card_rejects_ambiguous_unreadable_invalid_bounds_and_unsupported_requests() {
    let fc13 = decode_png(include_bytes!("../../tests/fixtures/freecell-FC13.png")).expect("FC13");
    let mut ambiguous = fc13.clone();
    paint_foreground_ambiguous_6d_8d(&mut ambiguous, PLAY_X[0], FREECELL_ROW_Y[0]);
    assert_eq!(
        read_card(&ambiguous, LocationRequest::freecell_play(1, 1)).expect("ambiguous read"),
        CardRead::Ambiguous
    );

    let mut unreadable = fc13.clone();
    paint_rect(
        &mut unreadable,
        PixelRect::new(PLAY_X[0] + 1, FREECELL_ROW_Y[0] + 4, 34, 49),
        [255, 255, 255],
    );
    assert_eq!(
        read_card(&unreadable, LocationRequest::freecell_play(1, 1)).expect("unreadable read"),
        CardRead::Unreadable
    );

    // Invalid 1-based coordinates -> OperationError::Invalid.
    for bad in [
        LocationRequest::freecell_play(0, 1),
        LocationRequest::freecell_play(9, 1),
        LocationRequest::freecell_play(1, 0),
        LocationRequest::freecell_play(1, FREECELL_MAX_PLAY_ROW + 1),
        LocationRequest::freecell_cell(0),
        LocationRequest::freecell_cell(5),
        LocationRequest {
            row: 2,
            ..LocationRequest::freecell_cell(1)
        },
        LocationRequest::freecell_suit(0),
        LocationRequest::freecell_suit(5),
        LocationRequest {
            row: 2,
            ..LocationRequest::freecell_suit(1)
        },
        LocationRequest::pyramid_play(0, 1),
        LocationRequest::pyramid_play(8, 1),
        LocationRequest::pyramid_play(3, 0),
        LocationRequest::pyramid_play(3, 4),
        LocationRequest::pyramid_draw(0),
        LocationRequest::pyramid_draw(3),
        LocationRequest {
            row: 2,
            ..LocationRequest::pyramid_draw(1)
        },
    ] {
        assert!(
            matches!(read_card(&fc13, bad), Err(OperationError::Invalid(_))),
            "expected Invalid for {bad:?}"
        );
    }

    // Unsupported game / location combinations -> OperationError::Unsupported.
    for unsupported in [
        LocationRequest {
            game: GameMode::FreeCell,
            kind: LocationKind::Draw,
            row: 1,
            column: 1,
        },
        LocationRequest {
            game: GameMode::Pyramid,
            kind: LocationKind::Cell,
            row: 1,
            column: 1,
        },
        LocationRequest {
            game: GameMode::Pyramid,
            kind: LocationKind::Suit,
            row: 1,
            column: 1,
        },
        LocationRequest {
            game: GameMode::Klondike,
            kind: LocationKind::Play,
            row: 1,
            column: 1,
        },
    ] {
        assert!(
            matches!(
                read_card(&fc13, unsupported),
                Err(OperationError::Unsupported(_))
            ),
            "expected Unsupported for {unsupported:?}"
        );
    }

    // Wrong-dimension or corrupted buffer -> OperationError::Invalid.
    let small = CapturedFrame {
        width: 1280,
        height: 720,
        stride: 1280 * 4,
        format: PixelFormat::Rgba8,
        pixels: vec![0; 1280 * 720 * 4],
    };
    assert!(matches!(
        read_card(&small, LocationRequest::freecell_play(1, 1)),
        Err(OperationError::Invalid(_))
    ));
}

/// Require both exact white `ll` stems and black `[0, 0, 0]` Continue segments in the same frame.
#[test]
fn challenge_complete_requires_joint_exact_ll_and_black_continue_independent_of_ribbon_colour() {
    let boards_continue = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .expect("boards-complete-continue");
    let freecell_continue = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-challenge-complete-continue-20261113.png"
    ))
    .expect("freecell-challenge-complete-continue");
    let fours_no_continue = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .expect("fours-complete");
    let xp_crimson = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-challenge-complete-xp-20261113.png"
    ))
    .expect("xp-crimson");
    let fc19 = decode_png(include_bytes!("../../tests/fixtures/freecell-FC19.png")).expect("FC19");

    assert!(is_challenge_complete_frame(&boards_continue).expect("boards_continue check"));
    assert!(is_challenge_complete_frame(&freecell_continue).expect("freecell_continue check"));
    assert!(!is_challenge_complete_frame(&fours_no_continue).expect("fours_no_continue check"));
    assert!(!is_challenge_complete_frame(&xp_crimson).expect("xp_crimson check"));
    assert!(
        !is_challenge_complete_frame(&fc19).expect("fc19 check"),
        "white card paper in CELL 4 covers ll stems, so Continue black check must keep this false"
    );
    for (name, bytes) in [
        (
            "FC09",
            include_bytes!("../../tests/fixtures/freecell-FC09.png").as_slice(),
        ),
        (
            "FC10",
            include_bytes!("../../tests/fixtures/freecell-FC10.png").as_slice(),
        ),
        (
            "FC11",
            include_bytes!("../../tests/fixtures/freecell-FC11.png").as_slice(),
        ),
        (
            "FC12",
            include_bytes!("../../tests/fixtures/freecell-FC12.png").as_slice(),
        ),
        (
            "FC15",
            include_bytes!("../../tests/fixtures/freecell-FC15.png").as_slice(),
        ),
    ] {
        let frame = decode_png(bytes).expect("negative terminal/board png");
        assert!(
            !is_challenge_complete_frame(&frame).expect("challenge complete check"),
            "{name} must not match strict ll + Continue"
        );
    }

    // Composite Continue from `boards_continue` onto live crimson Free Cell XP screen -> `Ok(true)`.
    let mut crimson_continue = xp_crimson.clone();
    for rect in CONTINUE_BLACK_RECTS {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                let off = y as usize * crimson_continue.stride + x as usize * 4;
                crimson_continue.pixels[off..off + 4]
                    .copy_from_slice(&boards_continue.pixels[off..off + 4]);
            }
        }
    }
    assert!(is_challenge_complete_frame(&crimson_continue).expect("crimson_continue check"));

    // Continue-only without white `ll` -> `Ok(false)`.
    let mut continue_only = fc19;
    paint_rect(&mut continue_only, CHALLENGE_LL_RECTS[0], [10, 80, 30]);
    for rect in CONTINUE_BLACK_RECTS {
        paint_rect(&mut continue_only, rect, [0, 0, 0]);
    }
    assert!(!is_challenge_complete_frame(&continue_only).expect("continue_only check"));

    // Single-pixel zero-tolerance check on both masks (`[255,255,254]` in `ll`, `[0,0,1]` in `Continue`).
    let mut one_px_ll = boards_continue.clone();
    let px_ll =
        CHALLENGE_LL_RECTS[0].y as usize * one_px_ll.stride + CHALLENGE_LL_RECTS[0].x as usize * 4;
    one_px_ll.pixels[px_ll + 2] = 254;
    assert!(!is_challenge_complete_frame(&one_px_ll).expect("one_px_ll check"));

    let mut one_px_continue = boards_continue.clone();
    let px_cont = CONTINUE_BLACK_RECTS[0].y as usize * one_px_continue.stride
        + CONTINUE_BLACK_RECTS[0].x as usize * 4;
    one_px_continue.pixels[px_cont + 2] = 1;
    assert!(!is_challenge_complete_frame(&one_px_continue).expect("one_px_continue check"));

    // Outside-mask invariance: invert every pixel outside both exact masks.
    let mut masked_inverted = boards_continue.clone();
    let in_masks = |x: u32, y: u32| -> bool {
        let contains =
            |r: PixelRect| x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height;
        CHALLENGE_LL_RECTS.into_iter().any(contains)
            || CONTINUE_BLACK_RECTS.into_iter().any(contains)
    };
    for y in 0..masked_inverted.height {
        for x in 0..masked_inverted.width {
            if !in_masks(x, y) {
                let off = y as usize * masked_inverted.stride + x as usize * 4;
                masked_inverted.pixels[off] ^= 0xff;
                masked_inverted.pixels[off + 1] ^= 0xff;
                masked_inverted.pixels[off + 2] ^= 0xff;
            }
        }
    }
    assert!(is_challenge_complete_frame(&masked_inverted).expect("masked_inverted check"));
}

/// Verify 5-attempt bounded polling (`0`/`1`/`4` x 1s waits), error propagation, and zero input calls.
#[test]
fn wait_for_challenge_complete_polls_up_to_five_frames_and_propagates_errors_with_zero_input() {
    let win = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .expect("win png");
    let miss = decode_png(include_bytes!(
        "../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .expect("miss png");

    // 1. Immediate hit -> 1 capture, 0 waits, 0 inputs.
    let mut io = ScriptedIo::new(vec![Ok(win.clone())]);
    assert!(wait_for_challenge_complete(&mut io).expect("immediate hit"));
    assert_eq!(io.capture_calls, 1);
    assert!(io.waits.is_empty());
    assert_eq!(io.input_calls, 0);

    // 2. Hit on 2nd observation -> 2 captures, 1 x 1s wait, 0 inputs.
    let mut io = ScriptedIo::new(vec![Ok(miss.clone()), Ok(win.clone())]);
    assert!(wait_for_challenge_complete(&mut io).expect("2nd hit"));
    assert_eq!(io.capture_calls, 2);
    assert_eq!(io.waits, vec![Duration::from_secs(1)]);
    assert_eq!(io.input_calls, 0);

    // 3. Hit on 5th observation -> 5 captures, 4 x 1s waits, 0 inputs.
    let mut io = ScriptedIo::new(vec![
        Ok(miss.clone()),
        Ok(miss.clone()),
        Ok(miss.clone()),
        Ok(miss.clone()),
        Ok(win),
    ]);
    assert!(wait_for_challenge_complete(&mut io).expect("5th hit"));
    assert_eq!(io.capture_calls, 5);
    assert_eq!(io.waits, vec![Duration::from_secs(1); 4]);
    assert_eq!(io.input_calls, 0);

    // 4. Five misses -> Ok(false), 5 captures, 4 x 1s waits, 0 inputs.
    let mut io = ScriptedIo::new((0..6).map(|_| Ok(miss.clone())).collect());
    assert!(!wait_for_challenge_complete(&mut io).expect("five misses"));
    assert_eq!(io.capture_calls, 5);
    assert_eq!(io.waits, vec![Duration::from_secs(1); 4]);
    assert_eq!(io.input_calls, 0);

    // 5. Immediate cancellation / transport / wait / bad-frame propagation without extra polling.
    let mut cancelled = ScriptedIo::new(vec![Ok(miss.clone())]);
    cancelled.fail_active_on = Some(1);
    assert!(matches!(
        challenge_complete(&mut cancelled),
        Err(OperationError::Cancelled)
    ));
    assert_eq!(cancelled.capture_calls, 0);
    assert_eq!(cancelled.input_calls, 0);

    let mut cancelled_poll = ScriptedIo::new(vec![Ok(miss.clone()), Ok(miss.clone())]);
    cancelled_poll.fail_wait_on = Some(1);
    assert!(matches!(
        wait_for_challenge_complete(&mut cancelled_poll),
        Err(OperationError::Cancelled)
    ));
    assert_eq!(cancelled_poll.capture_calls, 1);
    assert_eq!(cancelled_poll.waits.len(), 1);

    let mut transport = ScriptedIo::new(vec![Err(OperationError::Transport("link down".into()))]);
    assert!(matches!(
        wait_for_challenge_complete(&mut transport),
        Err(OperationError::Transport(_))
    ));
    assert_eq!(transport.capture_calls, 1);
    assert!(transport.waits.is_empty());

    let bad = CapturedFrame {
        width: 800,
        height: 600,
        stride: 800 * 4,
        format: PixelFormat::Rgba8,
        pixels: vec![0; 800 * 600 * 4],
    };
    let mut invalid = ScriptedIo::new(vec![Ok(bad)]);
    assert!(matches!(
        wait_for_challenge_complete(&mut invalid),
        Err(OperationError::Invalid(_))
    ));
    assert_eq!(invalid.capture_calls, 1);
    assert!(invalid.waits.is_empty());
    assert_eq!(invalid.input_calls, 0);
}

/// Live Fours Challenge failure frame proves empty `PLAY 1` (`r >= 45`) and interior card paper on `CELL 1`.
#[test]
fn freecell_empty_slot_accepts_live_play1_and_diagnoses_occupied_cell1() {
    let frame = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-empty-play1-fours-20261113.png"
    ))
    .expect("retained live empty PLAY 1 frame");

    assert_eq!(
        freecell_empty_slot_error(&frame, PLAY_X[0], 331).expect("valid frame"),
        Ok(())
    );
    assert_eq!(
        freecell_empty_slot_error(&frame, CELL_X[0], FREECELL_UPPER_TOP).expect("valid frame"),
        Err("Card paper interrupts empty-slot interior".to_owned())
    );

    for (slot, code) in [(1_u8, "9D"), (2, "8S"), (3, "7H"), (4, "9C")] {
        let (rank_ch, suit_ch) = code.split_at(1);
        let expected_rank = Rank::ALL
            .into_iter()
            .find(|rank| rank.as_str() == rank_ch)
            .expect("valid rank");
        let expected_suit = match suit_ch {
            "C" => Suit::Clubs,
            "D" => Suit::Diamonds,
            "H" => Suit::Hearts,
            "S" => Suit::Spades,
            _ => unreachable!("known test suit"),
        };
        assert_eq!(
            read_card(&frame, LocationRequest::freecell_cell(slot)).expect("valid cell read"),
            CardRead::Card(Card::new(expected_rank, expected_suit))
        );
    }
    assert_eq!(
        read_card(&frame, LocationRequest::freecell_suit(1)).expect("SUIT 1"),
        CardRead::Card(Card::new(Rank::Three, Suit::Hearts))
    );
    assert_eq!(
        read_card(&frame, LocationRequest::freecell_suit(2)).expect("SUIT 2"),
        CardRead::Card(Card::new(Rank::Three, Suit::Clubs))
    );
    assert_eq!(
        read_card(&frame, LocationRequest::freecell_suit(3)).expect("SUIT 3"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&frame, LocationRequest::freecell_suit(4)).expect("SUIT 4"),
        CardRead::Card(Card::new(Rank::Ace, Suit::Spades))
    );

    let mut broken = frame.clone();
    paint_rect(
        &mut broken,
        PixelRect::new(PLAY_X[0] + 30, 332, 4, 3),
        [30, 80, 40],
    );
    assert_eq!(
        freecell_empty_slot_error(&broken, PLAY_X[0], 331).expect("valid frame"),
        Err("Positive native empty-slot outline unavailable".to_owned())
    );

    let mut felt_only = frame;
    paint_rect(
        &mut felt_only,
        PixelRect::new(PLAY_X[0] + 20, 332, 96, 3),
        [34, 132, 88],
    );
    assert_eq!(
        freecell_empty_slot_error(&felt_only, PLAY_X[0], 331).expect("valid frame"),
        Err("Positive native empty-slot outline unavailable".to_owned())
    );
}

/// Variable-depth Free Cell cascades (`PLAY 7` rows `1..=10` and `PLAY 1` row `8`..`9`) plus empty `PLAY 1`.
#[test]
fn read_card_handles_variable_depth_freecell_cascades_and_empty_play1() {
    let parse_card = |code: &str| {
        let (rank_str, suit_str) = code.split_at(code.len() - 1);
        let rank = Rank::ALL
            .into_iter()
            .find(|rank| rank.as_str() == rank_str)
            .expect("valid rank");
        let suit = match suit_str {
            "C" => Suit::Clubs,
            "D" => Suit::Diamonds,
            "H" => Suit::Hearts,
            "S" => Suit::Spades,
            _ => unreachable!("known suit"),
        };
        Card::new(rank, suit)
    };

    let empty_play1 = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-empty-play1-fours-20261113.png"
    ))
    .expect("empty PLAY 1 frame");
    assert_eq!(
        read_card(&empty_play1, LocationRequest::freecell_play(1, 1)).expect("empty PLAY 1 row 1"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&empty_play1, LocationRequest::freecell_play(1, 2)).expect("empty PLAY 1 row 2"),
        CardRead::Empty
    );
    for (idx, code) in ["7S", "10H", "7C", "9H", "QH", "JS", "10D", "9S", "8H"]
        .into_iter()
        .enumerate()
    {
        let row = u8::try_from(idx + 1).expect("row fits u8");
        assert_eq!(
            read_card(&empty_play1, LocationRequest::freecell_play(7, row)).expect("PLAY 7 read"),
            CardRead::Card(parse_card(code)),
            "PLAY 7 row {row}"
        );
    }
    assert_eq!(
        read_card(&empty_play1, LocationRequest::freecell_play(7, 10)).expect("PLAY 7 row 10"),
        CardRead::Empty
    );

    let jc_frame = decode_png(include_bytes!(
        "../../tests/fixtures/freecell-jc-cell4-returned-20261111.png"
    ))
    .expect("JC returned frame");
    assert_eq!(
        read_card(&jc_frame, LocationRequest::freecell_play(1, 8)).expect("PLAY 1 row 8"),
        CardRead::Card(Card::new(Rank::Jack, Suit::Clubs))
    );
    assert_eq!(
        read_card(&jc_frame, LocationRequest::freecell_play(1, 9)).expect("PLAY 1 row 9"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&jc_frame, LocationRequest::freecell_cell(4)).expect("CELL 4"),
        CardRead::Empty
    );
}

/// Parse compact rank+suit notation like `"10S"` or `"AC"` for TriPeaks fixture assertions.
fn parse_test_card(code: &str) -> Card {
    let (rank_str, suit_str) = code.split_at(code.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|rank| rank.as_str() == rank_str)
        .expect("valid rank");
    let suit = match suit_str {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => unreachable!("known test suit"),
    };
    Card::new(rank, suit)
}

/// Verify `read_card` across all 28 TriPeaks tableau slots plus Stock and Waste on retained Beast PNGs.
#[test]
fn read_card_recognises_tripeaks_redeal_fixtures_face_up_face_down_and_draw() {
    let fixtures: [(&str, &[u8], &str, [&str; 10]); 3] = [
        (
            "draw-first",
            include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"),
            "2S",
            ["5D", "JC", "5H", "7H", "8D", "2D", "JD", "7D", "2H", "10S"],
        ),
        (
            "left-ace",
            include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"),
            "2D",
            ["2H", "AC", "QS", "5H", "KC", "6D", "4D", "8H", "JH", "9S"],
        ),
        (
            "left-eight",
            include_bytes!("../../tests/fixtures/tripeaks-redeal-20261016/left-eight.png"),
            "9D",
            ["JD", "8C", "2C", "8D", "JH", "8H", "8S", "QC", "QD", "9S"],
        ),
    ];

    for (name, bytes, expected_waste, expected_row4) in fixtures {
        let frame = decode_png(bytes).expect("tripeaks fixture png");

        for (row_idx, &row_len) in [3_u8, 6, 9].iter().enumerate() {
            let row = u8::try_from(row_idx + 1).expect("row fits");
            for col in 1..=row_len {
                assert_eq!(
                    read_card(&frame, LocationRequest::tripeaks_play(row, col))
                        .expect("covered tripeaks slot read"),
                    CardRead::FaceDown,
                    "{name} covered slot ({row}, {col})"
                );
            }
        }

        for (idx, code) in expected_row4.into_iter().enumerate() {
            let col = u8::try_from(idx + 1).expect("col fits");
            assert_eq!(
                read_card(&frame, LocationRequest::tripeaks_play(4, col))
                    .expect("row-4 tripeaks slot read"),
                CardRead::Card(parse_test_card(code)),
                "{name} row 4 col {col}"
            );
        }

        assert_eq!(
            read_card(&frame, LocationRequest::tripeaks_draw(1)).expect("tripeaks DRAW 1"),
            CardRead::FaceDown,
            "{name} stock"
        );
        assert_eq!(
            read_card(&frame, LocationRequest::tripeaks_draw(2)).expect("tripeaks DRAW 2"),
            CardRead::Card(parse_test_card(expected_waste)),
            "{name} waste"
        );
    }
}

/// Verify synthetic TriPeaks Empty, shifted Stock, Unreadable, Ambiguous, and coordinate guards.
#[test]
fn read_card_tripeaks_handles_empty_unreadable_ambiguous_and_bounds_edges() {
    let base = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"
    ))
    .expect("draw-first png");

    // 1. Overlapped Empty: when Row 4 col 2 (298, 443) is removed while parent Row 3 col 1
    //    (213, 332..513) stays FaceDown, Row 3 col 1's CLASSIC BLUE back sits at (298, 443..513)
    //    while Row 4 col 2's own non-overlapping band (y >= 514) shows felt -> read_card reads Empty.
    let mut cleared_r4_c2 = base.clone();
    paint_rect(
        &mut cleared_r4_c2,
        PixelRect::new(298, 443, 136, 70),
        [21, 87, 173],
    );
    paint_rect(
        &mut cleared_r4_c2,
        PixelRect::new(298, 514, 136, 110),
        [20, 130, 75],
    );
    assert_eq!(
        read_card(&cleared_r4_c2, LocationRequest::tripeaks_play(4, 2)).expect("cleared R4C2"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&cleared_r4_c2, LocationRequest::tripeaks_play(3, 1)).expect("R3C1 still down"),
        CardRead::FaceDown
    );

    // 2. Empty Stock and Waste.
    let mut empty_draw = base.clone();
    paint_rect(&mut empty_draw, TARGET_STOCK, [20, 130, 75]);
    paint_rect(&mut empty_draw, WASTE_CARD_BOUNDS, [20, 130, 75]);
    assert_eq!(
        read_card(&empty_draw, LocationRequest::tripeaks_draw(1)).expect("empty stock"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&empty_draw, LocationRequest::tripeaks_draw(2)).expect("empty waste"),
        CardRead::Empty
    );

    // 3. Shifted Stock stack contracted horizontally to the left inside TARGET_STOCK.
    let mut contracted_stock = empty_draw.clone();
    for row in 0..STOCK_CARD_BOUNDS.height {
        for col in 0..STOCK_CARD_BOUNDS.width {
            let from = (STOCK_CARD_BOUNDS.y + row) as usize * base.stride
                + (STOCK_CARD_BOUNDS.x + col) as usize * 4;
            let to = (STOCK_CARD_BOUNDS.y + row) as usize * contracted_stock.stride
                + (708 + col) as usize * 4;
            contracted_stock.pixels[to..to + 4].copy_from_slice(&base.pixels[from..from + 4]);
        }
    }
    assert_eq!(
        read_card(&contracted_stock, LocationRequest::tripeaks_draw(1)).expect("shifted stock"),
        CardRead::FaceDown
    );

    // 4. Unreadable corner / dark damage / invalid blue-backed Waste / partial Stock occlusion.
    let mut unreadable_play = base.clone();
    paint_rect(
        &mut unreadable_play,
        PixelRect::new(131, 447, 31, 49),
        [18, 18, 18],
    );
    paint_rect(
        &mut unreadable_play,
        PixelRect::new(389, 117, 28, 44),
        [18, 18, 18],
    );
    assert_eq!(
        read_card(&unreadable_play, LocationRequest::tripeaks_play(4, 1)).expect("dark R4C1"),
        CardRead::Unreadable
    );
    assert_eq!(
        read_card(&unreadable_play, LocationRequest::tripeaks_play(1, 1)).expect("dark R1C1"),
        CardRead::Unreadable
    );

    let mut unreadable_waste_blue = base.clone();
    paint_rect(&mut unreadable_waste_blue, WASTE_CARD_BOUNDS, [21, 87, 173]);
    assert_eq!(
        read_card(&unreadable_waste_blue, LocationRequest::tripeaks_draw(2))
            .expect("blue back waste"),
        CardRead::Unreadable
    );

    let mut broken_stock = base.clone();
    paint_rect(&mut broken_stock, STOCK_CARD_BOUNDS, [18, 18, 18]);
    assert_eq!(
        read_card(&broken_stock, LocationRequest::tripeaks_draw(1)).expect("broken stock"),
        CardRead::Unreadable
    );

    // 5. Ambiguous conflicting evidence on Tableau, Stock, and Waste.
    let mut ambiguous_fg = base.clone();
    paint_foreground_ambiguous_6d_8d(&mut ambiguous_fg, 128, 443);
    paint_foreground_ambiguous_6d_8d(&mut ambiguous_fg, WASTE_CARD_BOUNDS.x, WASTE_CARD_BOUNDS.y);
    assert_eq!(
        read_card(&ambiguous_fg, LocationRequest::tripeaks_play(4, 1)).expect("fg ambig tableau"),
        CardRead::Ambiguous
    );
    assert_eq!(
        read_card(&ambiguous_fg, LocationRequest::tripeaks_draw(2)).expect("fg ambig waste"),
        CardRead::Ambiguous
    );

    let mut ambiguous_band_felt = base.clone();
    paint_rect(
        &mut ambiguous_band_felt,
        PixelRect::new(134, 528, 124, 8),
        [20, 130, 75],
    );
    assert_eq!(
        read_card(&ambiguous_band_felt, LocationRequest::tripeaks_play(4, 1))
            .expect("corner + felt ambig"),
        CardRead::Ambiguous
    );

    let mut ambiguous_band_blue = base.clone();
    paint_rect(
        &mut ambiguous_band_blue,
        PixelRect::new(134, 528, 124, 8),
        [21, 87, 173],
    );
    assert_eq!(
        read_card(&ambiguous_band_blue, LocationRequest::tripeaks_play(4, 1))
            .expect("corner + blue ambig"),
        CardRead::Ambiguous
    );

    let mut ambiguous_stock = base;
    paint_rect(
        &mut ambiguous_stock,
        PixelRect::new(832, 682, 68, 182),
        [20, 130, 75],
    );
    assert_eq!(
        read_card(&ambiguous_stock, LocationRequest::tripeaks_draw(1)).expect("half felt stock"),
        CardRead::Ambiguous
    );

    // 6. Out-of-range coordinates -> Invalid; Cell/Suit -> Unsupported.
    for invalid in [
        LocationRequest::tripeaks_play(0, 1),
        LocationRequest::tripeaks_play(5, 1),
        LocationRequest::tripeaks_play(1, 0),
        LocationRequest::tripeaks_play(1, 4),
        LocationRequest::tripeaks_play(2, 7),
        LocationRequest::tripeaks_play(3, 10),
        LocationRequest::tripeaks_play(4, 11),
        LocationRequest::tripeaks_draw(0),
        LocationRequest::tripeaks_draw(3),
        LocationRequest {
            game: GameMode::TriPeaks,
            kind: LocationKind::Draw,
            row: 2,
            column: 1,
        },
    ] {
        assert!(
            matches!(
                read_card(&empty_draw, invalid),
                Err(OperationError::Invalid(_))
            ),
            "expected Invalid for {invalid:?}"
        );
    }

    for unsupported in [
        LocationRequest {
            game: GameMode::TriPeaks,
            kind: LocationKind::Cell,
            row: 1,
            column: 1,
        },
        LocationRequest {
            game: GameMode::TriPeaks,
            kind: LocationKind::Suit,
            row: 1,
            column: 1,
        },
    ] {
        assert!(
            matches!(
                read_card(&empty_draw, unsupported),
                Err(OperationError::Unsupported(_))
            ),
            "expected Unsupported for {unsupported:?}"
        );
    }
}

/// Verify `read_card` against the Goal 2 live Beast TriPeaks captures (contracted/empty Stock,
/// uncovered Row 1–3 cards, cleared Empty slots above/beside remaining cards, and dimmed modal).
#[test]
fn read_card_recognises_live_tripeaks_calibration_fixtures() {
    let stock_last = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
    ))
    .expect("stock-last-card png");
    assert_eq!(
        read_card(&stock_last, LocationRequest::tripeaks_draw(1)).expect("Draw 22 Stock"),
        CardRead::FaceDown
    );
    assert_eq!(
        read_card(&stock_last, LocationRequest::tripeaks_draw(2)).expect("Draw 22 Waste"),
        CardRead::Card(parse_test_card("5S"))
    );

    let stock_empty = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png"
    ))
    .expect("stock-empty png");
    assert_eq!(
        read_card(&stock_empty, LocationRequest::tripeaks_draw(1)).expect("Draw 23 empty Stock"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&stock_empty, LocationRequest::tripeaks_draw(2)).expect("Draw 23 Waste"),
        CardRead::Card(parse_test_card("4D"))
    );

    let row2_uncovered = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-calibration-20261116/tableau-row2-uncovered.png"
    ))
    .expect("tableau-row2-uncovered png");
    assert_eq!(
        read_card(&row2_uncovered, LocationRequest::tripeaks_play(2, 1)).expect("R2C1"),
        CardRead::Card(parse_test_card("5C"))
    );
    assert_eq!(
        read_card(&row2_uncovered, LocationRequest::tripeaks_play(2, 2)).expect("R2C2"),
        CardRead::Card(parse_test_card("KC"))
    );
    assert_eq!(
        read_card(&row2_uncovered, LocationRequest::tripeaks_play(3, 6)).expect("R3C6"),
        CardRead::Card(parse_test_card("AC"))
    );
    for (row, cols) in [(3_u8, &[1_u8, 2, 3][..]), (4, &[1, 2, 3, 4, 6, 7, 9][..])] {
        for &col in cols {
            assert_eq!(
                read_card(&row2_uncovered, LocationRequest::tripeaks_play(row, col))
                    .expect("cleared slot"),
                CardRead::Empty,
                "row {row} col {col} should be Empty"
            );
        }
    }
    for col in 1..=3 {
        assert_eq!(
            read_card(&row2_uncovered, LocationRequest::tripeaks_play(1, col))
                .expect("covered Row 1"),
            CardRead::FaceDown
        );
    }
    assert_eq!(
        read_card(&row2_uncovered, LocationRequest::tripeaks_draw(1)).expect("contracted Stock"),
        CardRead::FaceDown
    );
    assert_eq!(
        read_card(&row2_uncovered, LocationRequest::tripeaks_draw(2)).expect("Waste QC"),
        CardRead::Card(parse_test_card("QC"))
    );

    let peak1 = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-calibration-20261116/peak1-uncovered.png"
    ))
    .expect("peak1-uncovered png");
    assert_eq!(
        read_card(&peak1, LocationRequest::tripeaks_play(1, 1)).expect("Peak 1 R1C1"),
        CardRead::Card(parse_test_card("7D"))
    );
    assert_eq!(
        read_card(&peak1, LocationRequest::tripeaks_play(2, 1)).expect("cleared R2C1"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&peak1, LocationRequest::tripeaks_play(2, 2)).expect("cleared R2C2"),
        CardRead::Empty
    );
    assert_eq!(
        read_card(&peak1, LocationRequest::tripeaks_play(1, 2)).expect("covered Peak 2"),
        CardRead::FaceDown
    );
    assert_eq!(
        read_card(&peak1, LocationRequest::tripeaks_draw(2)).expect("Waste 5C"),
        CardRead::Card(parse_test_card("5C"))
    );

    let no_moves = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
    ))
    .expect("no-moves-remaining png");
    assert_eq!(
        read_card(&no_moves, LocationRequest::tripeaks_draw(1)).expect("dimmed modal Stock"),
        CardRead::Unreadable
    );
    assert_eq!(
        read_card(&no_moves, LocationRequest::tripeaks_play(1, 1)).expect("dimmed modal R1C1"),
        CardRead::Unreadable
    );
}

#[test]
fn full_root_stock_accepts_settled_t0_and_rejects_contracted_or_mid_rewind() {
    let draw_first = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-redeal-20261016/draw-first.png"
    ))
    .expect("draw-first png");
    let left_ace = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-redeal-20261016/left-ace.png"
    ))
    .expect("left-ace png");
    let left_eight = decode_png(include_bytes!(
        "../../tests/fixtures/tripeaks-redeal-20261016/left-eight.png"
    ))
    .expect("left-eight png");
    assert!(is_tripeaks_full_root_stock(&draw_first).expect("draw-first full stock"));
    assert!(is_tripeaks_full_root_stock(&left_ace).expect("left-ace full stock"));
    assert!(is_tripeaks_full_root_stock(&left_eight).expect("left-eight full stock"));

    for (name, bytes) in [
        (
            "tableau-row2-uncovered",
            include_bytes!(
                "../../tests/fixtures/tripeaks-calibration-20261116/tableau-row2-uncovered.png"
            )
            .as_slice(),
        ),
        (
            "peak1-uncovered",
            include_bytes!(
                "../../tests/fixtures/tripeaks-calibration-20261116/peak1-uncovered.png"
            )
            .as_slice(),
        ),
        (
            "stock-last-card",
            include_bytes!(
                "../../tests/fixtures/tripeaks-calibration-20261116/stock-last-card.png"
            )
            .as_slice(),
        ),
        (
            "stock-empty",
            include_bytes!("../../tests/fixtures/tripeaks-calibration-20261116/stock-empty.png")
                .as_slice(),
        ),
        (
            "no-moves-remaining",
            include_bytes!(
                "../../tests/fixtures/tripeaks-calibration-20261116/no-moves-remaining.png"
            )
            .as_slice(),
        ),
    ] {
        let frame = decode_png(bytes).expect("calibration png");
        assert!(
            !is_tripeaks_full_root_stock(&frame).expect("check non-root calibration fixture"),
            "{name} must not satisfy full T=0 Stock settlement"
        );
    }

    // 1. Even a single drawn card (-3 px shift exposing x = 964..968 green tray) fails full T=0 check.
    let mut one_draw_contracted = left_ace.clone();
    for row in 0..STOCK_CARD_BOUNDS.height {
        for col in 0..STOCK_CARD_BOUNDS.width {
            let from = (STOCK_CARD_BOUNDS.y + row) as usize * left_ace.stride
                + (STOCK_CARD_BOUNDS.x + col) as usize * 4;
            let to = (STOCK_CARD_BOUNDS.y + row) as usize * one_draw_contracted.stride
                + (STOCK_CARD_BOUNDS.x - 3 + col) as usize * 4;
            one_draw_contracted.pixels[to..to + 4]
                .copy_from_slice(&left_ace.pixels[from..from + 4]);
        }
    }
    paint_rect(
        &mut one_draw_contracted,
        PixelRect::new(
            STOCK_CARD_BOUNDS.right() - 3,
            STOCK_CARD_BOUNDS.y,
            4,
            STOCK_CARD_BOUNDS.height,
        ),
        [18, 83, 52],
    );
    assert!(!is_tripeaks_full_root_stock(&one_draw_contracted).expect("1-draw contracted stock"));

    // 2. Card still sliding left across x = 968..974 right overhang fails full T=0 check.
    let mut transit_overhang = left_ace.clone();
    paint_rect(
        &mut transit_overhang,
        PixelRect::new(STOCK_CARD_BOUNDS.right(), 700, 6, 150),
        [21, 87, 173],
    );
    assert!(!is_tripeaks_full_root_stock(&transit_overhang).expect("overhang blue card"));

    // 3. Rewinding card showing CLASSIC BLUE stripe inside left white deck stack (x = 810..817) fails.
    let mut mid_slide_stack = left_ace;
    paint_rect(
        &mut mid_slide_stack,
        PixelRect::new(810, 700, 8, 150),
        [21, 87, 173],
    );
    assert!(!is_tripeaks_full_root_stock(&mid_slide_stack).expect("mid-slide white stack"));
}
