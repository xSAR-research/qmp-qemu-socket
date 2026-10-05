# Free Cell v1.3.0, candidate 3

## Identity and diagnosis

Exact base: pushed candidate 2 at
`656881ee3f906347bd6f5665c733892621b1cc98`. Main was copied once into a clean
checkout and verified before editing. No subsequent remote polling or public
write was performed. The Beast's untracked root AGENTS.md is not a delivery file.

Charlie's log `qmp-qemu-socket-1791208524884.log` confirms independent one-board
GAME WIN entry and one score-skip click. The controller then waited exclusively
for LEVEL UP OK. The native STOPPED screenshot instead shows Congratulations,
2000 XP and New Game. Charlie ended the wait with STOP; this was not a failed
gameplay action or a lost QMP connection. Earning a game win does not imply
earning a level.

## Correction

After score skip, every fresh frame checks the existing local OK and New Game
controls together. Exactly one ready control selects the next stage:

| Local readiness | Next action |
| --- | --- |
| OK only | Click OK once, then wait for New Game |
| New Game only | Click New Game once, bypassing LEVEL UP |
| Neither | Wait and capture input-free within the existing allowance |
| Both | Stop as uncertain, retaining the latest frame |

New Game → Play → fresh board → Solver activation if inactive → fresh source
remains ordered. Starting directly on the completed New Game panel also works.
Step Once and finite runs still send no terminal/restart input. Each input retains
the existing running-guest/tablet probe and STOP checks; uncertain input is never
replayed. No previous terminal click is repeated because a next control is late.

Readiness uses only the existing local caption and warm button body after
independent win entry. No level, rank, medal, fireworks, panel artwork, card or
SUIT matching is added. The source detector, shared capture/QMP code and earlier
game modes are unchanged. Timing defaults and editable bounds are unchanged:
750 ms action settle, 1000 ms recapture interval, 20 delayed observations
(editable 1–100), and Actions per Multi-Step 0 for continuous operation. The
inherited score-skip settle is 3 s; other terminal settles are 1 s. These are
existing settings, not durations inferred from screenshots.

The reported Clippy warning is corrected in a test loop by replacing constant
`chunks_exact_mut(4)` with `as_chunks_mut::<4>().0`. No warning suppression,
formatter, toolchain change or minimum-version change is included.

## Evidence and regressions

`tests/fixtures/freecell-FC15.png` is a byte-identical copy of the native
`qmp-qemu-socket 261005 235829 STOPPED.png`: 1920×1080 RGBA, 1,919,954 bytes,
SHA-256 `af7bef64ffea86f51a3977aeeeca642a8181e81628730a59022fed98b9a13891`.
It shows New Game ready and no LEVEL UP OK. FC14 remains reserved for the absent
toolbar-crossing source case; this correction does not enable that geometry.

Four focused Rust regressions were added: FC15 local-control isolation;
immediate/delayed no-level-up restart; direct FC15 restart and finite-run
isolation; contradictory OK/New Game control rejection. Existing level-up,
bounded waiting, STOP and uncertain-input tests remain in place.

Source review and a source-derived Python pixel rehearsal confirm FC15 selects
New Game and rejects Score, OK and Play. These are not Rust test results.
The delivery workspace has no rustc, Cargo or rustup. Rust check/test/doc/Clippy
and release build therefore remain unrun here and must be completed on the
Beast. The validation archive records actual checks and their limitations.

## Beast verification

Check the startup label is v1.3.0, candidate 3. Resume directly from the STOPPED
New Game panel with continuous Multi-Step and confirm New Game → Play → fresh
board → Solver → source play. Then exercise full wins both with and without a
level change. Verify a late next control does not repeat the prior click, STOP
remains responsive, and finite runs do not restart. Preserve the original first
failure frame and full log before Undo if any stage stops.

Additional reward offers, repeated LEVEL UP panels and changed control geometry
remain outside this evidenced correction. Unexpected controls receive bounded
input-free observations and a guarded stop.

## Code Analysis

- **Threats and authority:** the optional branch exists only after independent
  win entry and score skip. Each fresh frame supplies local control readiness;
  neither stage order nor missing HALO permits a speculative click. Conflicting
  controls stop before either input.
- **Memory safety:** safe Rust and the existing validated RGBA frame access are
  retained. The Clippy correction uses array chunks in test-only storage; it
  introduces no unsafe code or production allocation.
- **Bounds:** the observation allowance remains clamped to 1–100. Control reads
  validate native dimensions/storage and use existing bounded regions. Source
  detection remains above Y=947 and no extra frame history is retained.
- **Failures:** STOP, probe/capture/classification errors and uncertain delivery
  stop without replay, preserving the latest available frame and log. No level
  change now bypasses the absent OK stage. An unrecognised next control still
  stops when the existing allowance is exhausted.
