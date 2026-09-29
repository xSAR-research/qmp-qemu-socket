# Klondike v1.2.1, candidate 1

Date: 30 September 2026, Australia/Brisbane.
Base: qmp-qemu-socket v1.2.0, commit
`db506d11b46ce7180cd3be7fe05a66cfa056999e`.

## Approved input contract

Charlie confirmed Draw 1, automatic reveal of newly exposed cards, the `d`
shortcut for ordinary draws, one click anywhere inside a highlighted source
block to move a card or complete run, up to three visible RIGHT fan positions,
and possible moves from SUIT piles back to the tableau. The empty-stock recycle
HALO is shown in the additional 30 September capture. Recycle uses a click;
`d` is not assumed to recycle. No drag or destination click is generated.

The 30 September instruction replaces the earlier no-HALO restriction for
Klondike only: a recognised gameplay scene with no eligible source may receive
one Solver activation, immediately followed by capture. The existing TriPeaks
and Pyramid recovery policies remain in force for those modes.

## Source and scene detection

One immutable 1920x1080 RGBA frame is scanned in this order:

1. DRAW, classifying a blue stock back separately from green empty-stock recycle.
2. RIGHT, allowing front-card origins at x=558, 586 or 614.
3. Tableau, selecting the lowest eligible source bottom, with left-to-right ties.
4. SUIT piles, left to right.

Normal card origins have a 168-pixel pitch, beginning at x=390. Upper piles
start at y=112 and tableau cards at y=342. Measured card width is 132 pixels.
The lower scan bound is conservatively inside the playable area, above the
observed toolbar. Different scaling, themes and displaced layouts are not
calibrated merely because the frame still measures 1920x1080.

The reusable solid-source primitive requires a continuous lower edge,
connected exterior side rails and bright card paper above the edge. This
rejects a dark dashed landing guide without relying on one potentially black
rank/suit pixel. Connected rails group a highlighted run into one action.
Stock uses positive card-back/recycle interior tests instead of card-paper tests.
The centred Solver banner and board anchors qualify source recommendations.
Scene recognition also works with Solver off so that explicit recovery can
be offered; sparse or unrecognised layouts stop.

The supplied examples establish highlighted tableau sources in columns 3, 5
and 7, ordinary stock highlights and recycle. RIGHT and SUIT source selection
use Charlie's input contract and measured shared card geometry; their full live
transfer effects are not recorded in the original pairs. Test fixtures label
synthetic arrangements separately from original evidence.

## Execution and bounds

The worker owns the QMP connection and input. The initial frame must reproduce
the approved mode/socket prediction before gameplay. It freshly requires a
running VM and the current absolute `QEMU HID Tablet` before each operation.
Click movement, down and up remain separately acknowledged; draw uses key down
and up without pointer movement. An uncertain operation is never replayed.
STOP is cooperative, including intentional waits, and cannot interrupt an
already in-flight QMP request instantly.

Single Step sends at most one gameplay operation. Multi-Step defaults to 10
and requires a finite limit from 1 through 10000. Each action settles using
Klondike's independent editable value, initially 2000 ms. A fresh result is
then verified and reused as the next planning frame. Parameters are snapshotted
at run start; editing Params affects the next run.

A valid no-HALO scene permits one Solver click per unresolved context, followed
by immediate capture with no added post-click sleep. Pointer positioning,
button hold and protocol acknowledgements still take time. At most three
additional input-free captures follow, separated by the editable recapture
interval, initially 1000 ms. The run also bounds total Solver refreshes by the
gameplay limit plus one. These timing defaults are user-approved starting
values, not measurements from still images.

If the initial planning frame has no eligible target, Solver recovery displays
its result and ends the request with zero gameplay actions. Charlie reviews
that preview and starts another explicit step/run. If a gameplay result loses
its HALO, refreshing Solver does not establish that the preceding action worked:
its independent effect still has to pass before another gameplay operation.

## Effect and failure policy

Draw requires material waste-content change. Recycle requires a stock back to
reappear, waste to become empty felt, and waste-content change. Transfers need
positive source-content replacement/removal evidence and content change in
another tableau/foundation region. Gold edges, cursor neighbourhoods and source
darkening into a destination guide cannot by themselves prove a transfer.
No rank identity or legal-move search is performed; this remains a conservative
visual check rather than symbolic proof of a card move.

Every fresh observation updates the bounded diagnostic preview mailbox. Failure
frames remain diagnostic and cannot authorise further input. Unknown scenes,
invalid geometry, changed initial targets, exhausted observations and uncertain
input stop with the latest frame and logs. A fresh explicit capture/review is
required to resume after an unverified action.

Completion, autocomplete, no-solution dialogs and restart are unsupported.
Klondike never enters the shared three-board or post-game controller. Lack of a
HALO does not count as a win. No independent rank recognition, path search or
file-free capture backend is included.

## Capture and source formatting

Capture still uses QMP temporary PNG, read, RGBA8 decode and normal best-effort
cleanup. Detectors then read that frame in memory. Capture PNG/Save PNG retain
and save the original PNG bytes without recapture or preview scaling.

All Rust source, including pre-existing modules, receives two blank lines before
definitions and statement blocks, above attached comments/attributes. No
`rustfmt` or `cargo fmt` is run. The spacing pass checks identical Rust leaf
tokens/comments and valid syntax before and after insertion. Nightly and
rustfmt configuration remain unchanged; the application dependency lock graph
is unchanged apart from the local package version.

## Verification and live acceptance

The separate candidate review records exact compiler versions, checks executed
and installer rehearsals. Supplied PNG pixels provide detector/effect regression
fixtures; controlled mocks exercise the real Klondike execution state machine.
Neither establishes live QEMU/Windows behaviour.

On the Beast, begin with Single Step, then a small finite Multi-Step budget.
Exercise ordinary draw, a single tableau transfer, a full highlighted run,
RIGHT fan positions, recycle and SUIT return when encountered. Check editable
timings, Solver recovery, STOP, latest-frame retention, mode/socket invalidation
and exact-byte manual PNG saving. Retain the first failing frame/session log.
Regress TriPeaks and Pyramid, including their established transitions, before
promoting this candidate. No new screenshots are needed for already-covered
cases unless the candidate behaves differently.

## Code Analysis

No dependencies or unsafe Rust are added. Captured pixels and typed targets use
checked native geometry; detector loops and execution budgets are finite.
QMP remains restricted to the selected Unix socket, with no host-wide input.
The main threat surface is incorrectly classifying a visual state: independent
scene/source/effect checks, finite budgets, fresh probes, diagnostic-only failure
frames and no replay limit the resulting input authority. Visual verification
can still have false negatives or false positives and needs Beast acceptance.
Existing PNG decoding, temporary-file cleanup and trusted dependency build-code
risks remain. Completion and uncalibrated end scenes stop rather than granting
terminal input authority.
