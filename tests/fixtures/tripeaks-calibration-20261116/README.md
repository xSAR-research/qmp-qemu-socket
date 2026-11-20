# TriPeaks Live Calibration Fixtures (`2026-11-16`)

Curated `1920×1080` captures from live Windows 11 *Microsoft Solitaire & Casual Games* TriPeaks on Beast (`XSCP v3` TCP capture + guarded QMP input), plus compact ink-channel templates extracted for `src/game_controls.rs` and `src/tripeaks_progress.rs`.

## Key Calibrated Findings

1. **Stock horizontal contraction & empty-Stock dark tray:**
   - Full `23`-card Stock begins at blue span `x = 836..966` (`130 px` wide) and contracts uniformly by `-3 px` per `D` draw down to `x = 770..900` (`129 px` wide) at Draw 22 (`stock-last-card.png`).
   - Empty Stock (`stock-empty.png`, Draw 23) renders inside a darkened translucent green tray (`R = 0..81`, `G = 57..178`, `B = 28..140`) with a light-green empty-slot outline at `x = 767..900, y = 682..860`.
2. **Draw-23 zero-match `"No Moves Remaining"` modal & `Undo All` restoration:**
   - When `0` Stock cards and `0` playable tableau `±1` moves remain in Normal mode (`no-moves-remaining.png`), Microsoft Solitaire dims the full board (including `SHARED_UNDO_ALL_CONTROL` at `(1320, 978)`) after ~`1.2–2.0 s` and shows `"No Moves Remaining"` (`x = 572..1347, y = 268..795`) with `"Undo last move"` at `PixelRect::new(976, 692, 305, 72)` (click centre `(1128, 728)` using `POST_GAME_MOUSE_HOLD`).
   - In Challenge mode (`no-moves-remaining-compact.png`, `Queens cleared 0/10`), Draw 23 with zero legal moves immediately dims the board and displays the compact modal (`x = 531..1388, y = 393..670`, without a Solver prompt) with `"No Moves Remaining"` at `x = 727..1197, y = 457..505` and `"Undo last move"` at `PixelRect::new(977, 568, 312, 74)` (click centre `(1133, 605)`), leaving `WASTE_CARD_BOUNDS` (`5D`) and the dimmed empty Stock outline completely unobstructed below the panel.
   - Clicking `"Undo last move"` dismisses the modal cleanly so `Undo All` (`(1320, 978)`) can restore `T=0` (`0` differing pixels across `y = 40..1025`). On this guest, `Undo All` executed directly without a secondary `OK / Cancel` dialog.
3. **Empirically verified TriPeaks scoring:**
   - Tableau clears on consecutive streaks `k = 1..6` increment `Score` by `100 * (2 * k - 1)` (`+100, +300, +500, +700, +900, +1,100` -> `100 * streak^2`).
   - `Draw` (`D`) adds `0` points and resets `streak = 0`.
   - Clearing a `Row 1` Peak (`peak1-uncovered.png` -> `peak1-cleared`) awards a `+500` Peak clear bonus (`+100 + 500 = +600` at streak `1`).
4. **Banner & deals-left template provenance:**
   - `12` of `13` plural rank words (`aces`..`kings` except `threes`), `boards`, `cleared`, `score`, `solve`, `the-deck`, `clear-word`, `deals-zero`, `deals-one`, `deals-two`, `no-moves-title`, and `no-moves-title-compact` are extracted directly from native Windows 11 screenshots.
   - `threes-spaced.ink` and `threes-compact.ink` are labelled `synthetic-calibrated-from-native-glyphs` in `manifest.json`, assembled strictly from native Segoe UI Semibold `T`, `h`, `r`, `ee`, `s` glyphs on the identical `y = 48..80` DirectWrite baseline.
5. **Normal HUD `Score:` label & `h = 17` digit bank provenance:**
   - `normal-score-label.ink` (`69×32`), `normal-score-label-alt.ink` (`68×32`), `normal-digits.ink` (`28` native `20×26` glyph canvases = `14,560` bytes), and `normal-labels.txt` are extracted strictly from native `h = 17` Windows 11 top-HUD captures (`0..=9`, `','`) across TriPeaks live calibration and retained Normal Pyramid fixtures.
