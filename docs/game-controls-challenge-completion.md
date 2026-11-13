# Shared `game_controls` and Challenge Completion (`3.2.0`)

`src/game_controls.rs` exposes the crate-wide borrowed-frame card reader (`read_card`) and input-free `Challenge Complete!` detectors (`is_challenge_complete_frame`, `challenge_complete`, `wait_for_challenge_complete`), while `src/freecell_terminal::classify_challenge_handoff` classifies Challenge XP-counting and settled `LEVEL-UP!` handoffs when the 5-frame **Continue** poll ends without **Continue**.

## `LocationRequest` and `read_card`

`game_controls::LocationRequest` uses 1-based physical coordinates:

- `game: GameMode`
- `kind: LocationKind` (`Play`, `Cell`, `Suit`, `Draw`)
- `row: u8`
- `column: u8`

Supported coordinate bounds at `1920×1080` native RGBA layout (`100%` display scale, `CLASSIC BLUE` card back):

| Game | Location | `row` | `column` | Empty handling |
| --- | --- | --- | --- | --- |
| `FreeCell` | `Play` | `1..=19` (`FREECELL_MAX_PLAY_ROW`) | `1..=8` | `row == 1` validates empty slot outline (`r >= 45`, interior paper check first) and returns `CardRead::Empty`; deeper unoccupied rows past the scanned bottom corner return `CardRead::Empty` without treating missing-corner/white-paper gaps as empty |
| `FreeCell` | `Cell` | `1` | `1..=4` | Checks `freecell_empty_top` / upper corner |
| `FreeCell` | `Suit` | `1` | `1..=4` | Physical left-to-right slots via shared upper-card corner classification |
| `Pyramid` | `Play` | `1..=7` | `1..=row` | Foreground card corner classification (Pyramid runtime pipeline untouched) |
| `Pyramid` | `Draw` | `1` | `1..=2` | Left = `1`, Right = `2` |

`game_controls::scan_freecell_play_corners` consolidates the vertical cascade corner sweep shared with `freecell_recon::position`, scanning downward up to `FREECELL_MAX_PLAY_ROW = 19` entries and recording `PlayCorner { y, corner, card }` with `Corner::matched_last` pickup anchoring.

## Zero-tolerance `Challenge Complete!` detection

`game_controls::is_challenge_complete_frame` validates:

1. White `ll` lower limbs in **Challenge**: `CHALLENGE_LL_RECTS` (`(766, 204, 8×15)` and `(785, 204, 7×15)`, RGB `[255, 255, 255]`).
2. Black **Continue** button glyph strokes: `CONTINUE_BLACK_RECTS` (8 exact segments, RGB `[0, 0, 0]`).

Matching is strictly zero-tolerance across all `225 + 149` exact pixels and independent of crimson or navy ribbon colour. `game_controls::challenge_complete` acquires one fresh `1920×1080` frame and checks cancellation and `is_challenge_complete_frame`; `game_controls::wait_for_challenge_complete` polls up to 5 captures separated by up to 4 cancellable `1 s` waits (`OperationIo`), sending zero QMP inputs.

## Fault-gated playback integration and Option 1A Challenge handoff

In `src/worker/freecell_route.rs`:

- Polling runs strictly when BOTH `challenge_mode` (`objective::from_metadata(..) == Objective::Unresolved`) AND a fresh board-recognition step faults (`transfer` pre-input proof fault, post-drag `finish_movement_probe` scene fault, or after 6 unmatched `stable` captures via `RouteIo::resolve_challenge_complete`). Ordinary `ClearBoard` deals never poll `Challenge Complete!`.
- On `wait_for_challenge_complete` -> `Ok(true)`, playback retires cleanly as `TransferOutcome::ChallengeComplete` (`"continue_visible"`) without clicking **Continue** or crediting unverified card transfers.
- On `wait_for_challenge_complete` -> `Ok(false)` (all 5 captures lacked settled **Continue**), `freecell_terminal::classify_challenge_handoff` inspects the 5th captured frame (`self.latest`) once:
  - `ChallengeTerminalHandoff::XpCounting`: white `ll` stems + non-white banner ribbon gap (`(776..=783, 204..=219)`) + `CHALLENGE_SKIP` (`Click anywhere to skip` at `(796, 864, 328×36)`). Retires cleanly as `ChallengeComplete` (`"xp_counting"`) with zero clicks so XP finishes tallying or the operator skips/continues.
  - `ChallengeTerminalHandoff::LevelUp`: `LEVEL-UP!` white banner lettering (`has_level_up_banner`) + settled-size gold `OK` button (`OkReadiness` `validate_settled_button` at `[190, 224]` around `ExpectedControlSpec::LevelUp`, with central black glyph ink). Retires cleanly as `ChallengeComplete` (`"level_up_ok"`) with zero clicks so the operator dismisses `LEVEL-UP!`.
  - `None`: preserves the original board-recognition fault and original `first` diagnostic frame (`first-unmatched`).

No QMP clicks, synthetic dismissals, or automatic post-modal `Next Challenge` inputs are issued.

