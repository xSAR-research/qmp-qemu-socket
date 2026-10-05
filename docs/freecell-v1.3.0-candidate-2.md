# Free Cell v1.3.0, candidate 2

## Identity and scope

Exact base: `44727193d4e620a749925e9cc46c9d9d4df1700f`, pushed v1.3.0
candidate 1. Main was copied once; the initial checkout was clean. The refreshed
companion AGENTS.md was read separately and remains outside the candidate.
No subsequent remote freshness check or public write was performed.

Charlie approved implementation after FC01–FC13 intake. This candidate enables
Free Cell source play and its single-board GAME WIN/restart sequence. It does
not introduce rank recognition, independent search or file-free capture.
TriPeaks, Pyramid and Klondike runtime policies remain unchanged.

## Gameplay process

1. Capture a fresh native frame at run start; the displayed preview is advisory.
2. Positively recognise the Free Cell board and Solver banner. Check CELL 1–4
   left-to-right, then PLAY 1–8 bottom-up with left-to-right tie-breaking.
3. Select only a complete solid source outline. Dashed destination guides are
   ignored; connected highlighted cards form one run.
4. Probe the running guest/current QEMU HID Tablet and check STOP. Click the
   bottom card once. No drag or destination click is sent.
5. Wait the editable action/automatic-transfer settle, then capture anew.
6. If there is no HALO, check independent GAME WIN entry. Otherwise recapture
   input-free within the editable allowance while automatic SUIT transfers finish.
   A recognised Solver-off board can activate Solver once, settle and recapture.
   An active Solver is not refreshed merely because a HALO is absent.

There is no source/destination card matching, changed-pixel threshold or effect
proof. The same source coordinates may recur legitimately. Acknowledgement
consumes one action slot but is not described as a proven card transfer.
SUIT cards never return to PLAY in this confirmed input contract: no SUIT source
scan is present. Free Cell has no Draw, Recycle or Solve operation.

Native dimensions and buffer layout are checked before pixel reads. Source
probes and clicks exclude the toolbar at Y=947. Optional FC14 was not supplied;
a source missing its complete visible lower outline above that boundary remains
unsupported. No Klondike clipped-source fallback was copied into Free Cell.

## GAME WIN and restart

One board is one game. Independent entry requires Congratulations lettering
plus the score-skip caption or completed-game New Game control. No HALO, an
isolated OK or the difficulty selector's Play control is not completion evidence.

After entry, continuous Multi-Step 0 follows the deterministic expected path:

| Expected stage | Authority | One native click |
| --- | --- | --- |
| Score counting | Local Click anywhere to skip caption | 960,550 |
| LEVEL UP | Local gold body and OK word | 959,812 |
| Congratulations | Local gold body and New Game words | 786,852 |
| New Game selector | Local gold body and Play word | 709,761 |
| Fresh board | Free Cell layout; activate Solver only if inactive | Solver 602,977 |

Expected button readiness uses broad warm body colour and coarse word contrast,
not fixed RGB samples from levels, titles, ranks, medals, fireworks, modal frames,
completed cards or SUIT piles. Small local caption alignment does not move the
fixed click point. Every input is sent once; unready next stages receive bounded
input-free captures rather than retries of the previous click.

The existing 500 ms terminal mouse hold is reused. Score skip waits the existing
3 s Level Up appearance interval; other terminal stages use the existing 1 s
post-game interval, then fresh capture. These are inherited settings, not measured
durations from still images. STOP remains effective during cooperative waits.
Restart must produce a recognised fresh board and a new supported source frame
before continuous source play resumes. Finite budgets and Step Once do not send
terminal/restart input. Starting directly on the completed New Game panel skips
already-finished score/OK stages.

FC09–FC13 supports one LEVEL UP in this sequence. A skipped/repeated LEVEL UP,
extra reward offer, changed control geometry or another unexpected dialog stops
input-free. The candidate does not claim these unobserved variants are supported.

## Parameters

| Free Cell setting | Initial value | Bounds |
| --- | --- | --- |
| Action / automatic-transfer settle | 750 ms | 0–5000 ms |
| Input-free re-observation interval | 1000 ms | 0–5000 ms |
| Delayed observations per unresolved stage | 20 | 1–100 |
| Actions per Multi-Step | 0, continuous | 0 or 1–10000 |

The complete settings snapshot applies to one run. Terminal control clicks and
Solver activation do not consume source-action slots. Step Once authorises at
most one source action. Continuous play is bounded per unresolved stage and is
STOP-cancellable; it is not a promise of unlimited automatic recovery.

## Evidence and validation

The thirteen original native 1920×1080 RGBA PNGs are copied byte-identically to
flat `tests/fixtures/freecell-FC01.png` through `freecell-FC13.png`. Their fixture
manifest records names, dimensions, hashes and observations. FC06 and FC07 show
the same two-card source; timestamps are not settle measurements.

Focused Rust tests cover native sources, dashed-guide rejection, source priority,
toolbar bounds, terminal entry/control isolation, ordered restart, finite budgets,
automatic-transfer recapture, repeated source positions, STOP and uncertain
input non-replay. They use production classifiers on the supplied fixtures.

**Rust compilation and tests were not run in the delivery workspace:** rustc,
Cargo and rustup are absent, and the bounded official dated-toolchain download
attempt timed out. No global toolchain, proxy, stable compiler or formatting
setting was changed. Pixel rehearsals, manual source review, whitespace checks
and guarded-installer rehearsals are reported separately in the delivery review;
they do not substitute for Rust check/test/Clippy/build results. The Beast must
complete all supplied validation commands before launching or accepting this
candidate. No warning-free or gameplay-acceptance claim is made here.

## Beast checks

Check executable/startup identity v1.3.0, candidate 2. Start on FC01 with Solver
off, run continuously, and confirm a single activation then fresh HALO play.
Test CELL, single PLAY and multi-card PLAY clicks, automatic SUIT transfers and
consecutive recommendations at the same position. Verify Step Once, finite budgets,
independent settings, STOP and mode/socket invalidation. Capture PNG/Save PNG must
still save exactly the previewed original bytes without another screendump.

The acceptance focus is a full GAME WIN: score skip → OK → New Game → Play →
fresh board → Solver → new source action. Preserve the first failure's original
PNG and full log before Undo; a separate Undo capture is not its result frame.
Check current TriPeaks/Pyramid/Klondike behaviour without changing their timings.
Do not treat an absent FC14 or unobserved reward prompt as accepted coverage.

## Code Analysis

- **Threats and authority:** stale previews cannot authorise inputs; only fresh
  supported sources or the expected local control after independent win entry
  do. Read-only captures send no input. Forged Free Cell key actions fail canonical
  planning. SUIT, Draw, Recycle and Solve are outside the permitted action set.
- **Memory safety:** pixel reads use the shared checked accessor after native
  layout validation. No unsafe Rust or new raw buffer lifetime is introduced.
  Latest frames remain coalesced by shared worker output; complete logs use the
  existing private session file rather than an unbounded new in-memory history.
- **Bounds:** source coordinates are constrained to four CELL/eight PLAY lanes
  above Y=947. Percentage arithmetic is widened where necessary; caption masks
  fit u128. Observations clamp to 1–100 and finite action budgets to 10000.
  Continuous action counting checks overflow before input.
- **Failures:** STOP/probe/capture/classifier errors retain the latest available
  decoded frame. QMP input uncertainty exits without replay. Bounded waits do not
  turn missing HALOs into speculative clicks or a win. Unexpected terminal paths
  stop; an acknowledged click is not a proven UI transition.
- **Limits:** source/control recognition still depends on this native layout and
  local contrast. FC14, skipped/repeated LEVEL UP and live automatic-transfer
  timing are not validated. Native compiler and Beast gameplay checks remain
  outstanding; manual review cannot establish Rust compilation or absence of
  compiler warnings.
