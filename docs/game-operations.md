# Reusable game operations

This private application module implements the operation boundary for future
game workflows. `src/game_operations.rs` owns the contracts; its `session`,
`option` and `restoration` children own input, pure option detection and physical
restoration. `src/worker/game_operations.rs` adapts existing worker resources.
Pyramid reconnaissance uses the shared Undo All protocol. Existing Draw callers
also use the typed key wrapper. Other solvers retain their existing controllers.

## Existing facilities and ownership

| Facility | Existing owner | Reuse in this implementation |
| --- | --- | --- |
| Native TCP capture and RGBA conversion | xSAR client and `worker/tcp_capture.rs` | Native adapter calls existing `capture_screen`; selected backend, retained connection, cancellation, transport timing and diagnostics remain there. |
| Legacy QMP screendump | `worker.rs` | Explicit existing backend remains available; used by local transport tests. |
| QMP probe, pointer, button and key press/release | xSAR transport through `qmp.rs` | Existing configured transport and timing; application `SupportedKey::Draw` maps to QEMU qcode `d`. |
| Checked pixels, rectangles and coordinate conversion | xSAR through `geometry.rs` | Full display coordinates and checked conversion; no scaling. |
| Horizontal run matching | xSAR `find_horizontal_run` | Pure unique maximal-run wrapper; existing terminal first-hit searches retain their semantics. |
| Rank/suit and positive pile-empty recognition | `pyramid_recon/vision.rs` | Narrow borrowed-frame read of one Pyramid location, using existing calibrated readers and strict source-scene guards. |
| Complete Undo All confirmation and restoration | `pyramid_recon.rs` and native Recon I/O | Shared bounded protocol; adapter delegates existing capture, input, settle, deadline, context and delivery ledger. |
| Single Undo enabled appearance and predecessor | No measured production adapter | Generic protocol implemented and tested; a live adapter must supply calibration and semantic history. |

Transport, checked geometry and image processing stay in xSAR. Game locations,
scene authority, expected transitions and restoration remain application owned.
The capture server and wire protocol require no changes. No dependencies,
threads, public library API or general solver framework are introduced.

## Types and evidence

`CapturedFrame` remains the owned RGBA buffer. `validate_native_frame` requires
1920×1080, stride 7680, exactly 8,294,400 bytes and a valid checked view. It
borrows storage and never resizes or copies it. Pure option detection can also
inspect a valid padded frame; a live `Session` requires the packed native form.

`CardLocation::Pyramid(Location)` reuses xSAR's checked original one-based
tableau locations and DL/DR. Removed cards never compact those locations.
Additional games require explicit location variants and calibrated readers.

`CardRead` distinguishes `Card(Card)`, positive `Empty`, `Unreadable` and
`Ambiguous`. The latter currently identifies contradictory card/empty pile
evidence. Existing recognisers may conservatively return unreadable for
non-unique glyph evidence, or a typed `Recognition` error for structural/template
refusal; this layer does not classify diagnostic strings or invent an identity.
Unreadable tableau evidence never becomes Empty.

`OperationError` separates invalid configuration/storage/coordinates,
unsupported capability, not found, disabled, ambiguous, cancelled, read-only
transport failure, recognition failure, semantic verification failure and
retained adapter errors. `Uncertain` means delivery was attempted and must never
be replayed. `InterruptedAfterDelivery` preserves an acknowledged delivery even
when STOP or context checking subsequently fails.

Three facts remain separate: a recogniser found a target; QMP acknowledged one
input; a semantic observer proved its effect. `Delivery { frame_serial, input }`
records only the second fact. Recognition results and detected option points
grant no independent input authority.

## Function catalogue

| Operation | Inputs and result | Completion, side effects and recovery |
| --- | --- | --- |
| `Session::screen_capture` | Existing adapter; returns a borrowed fresh native frame | Retires any older frame before capture, checks active context before and after acquisition, validates layout and advances a checked serial. No input. A failed capture leaves no usable frame. |
| `Session::mouse_left_click` | Checked guest `PixelPoint`; returns `Delivery` | Consumes one captured frame, checks selected-game/action evidence, probes VM and absolute pointer, checks STOP/context, then performs one complete move/left press/release. ACK is delivery only. Obtain a new capture for another action. |
| `Session::send_key_stroke` | `SupportedKey::Draw`; returns `Delivery` | Same single-use authority sequence; existing QMP qcode and key hold timing. No arbitrary integer keycodes. More keys require explicit supported variants. |
| `read_card` | Borrowed frame, existing `ReconVision`, typed location; returns `CardRead` | Pure. Strict source scene, layout and modal guards, followed by only the selected card/pile reader. Does not require unrelated tableau identities. No capture, input, cache or identity inference. |
| `detect_option` / `Session::click_option` | Frame, inclusive `InclusiveBounds`, minimum black-run length; detection or `Delivery` | Pure detector returns NotFound, Ambiguous or Found. Session detects again on its current frame, then uses the same guarded one-click boundary. Black pixels alone never select game policy or authorise input. |
| `undo_last` | Calibrated `UndoLastIo`, distinct previous/current physical snapshots, known control, semantic equality and finite budget | Fresh current-position and enabled-control checks precede one Undo click. One settle then two consecutive semantic predecessor observations prove restoration. Disabled/not-found/ambiguous/wrong control or indistinguishable history refuses before input. No measured live adapter exists yet. |
| `undo_all` | `UndoAllIo`, original/current physical snapshots, known control, semantic equality and finite budget | Existing complete acquisition authorises Undo All; complete confirmation is observed twice before one OK. One post-ACK settle and two consecutive original-position observations prove restoration. Caller publishes only after success. |

The generic restoration traits require the adapter to capture freshly, enforce
STOP/context/deadline, guard every input with a fresh probe and retain an
uncertainty latch. They do not bypass these rules by accepting a raw QMP client.
The existing Recon adapter supplies them through `Runner` and its delivery
ledger. Native single Undo currently remains unsupported rather than treating
shared toolbar geometry as proof of an enabled command.

## Option bounds and grouping

Bounds include both endpoints, including (0,0) and (1919,1079). Width and height
use checked `maximum - minimum + 1`; reversed, overflowing and out-of-frame
bounds fail. Minimum length zero fails. A minimum longer than the region finds
no target. Frame views handle stride, and padding is never searched.

A candidate is one maximal horizontal run of exact RGB black; alpha is ignored.
A long run counts once. Two qualifying runs, even on adjacent rows, are
Ambiguous. This deliberately does not group thick lettering into a two
dimensional connected component. Detection uses at most three disjoint existing
run scans: the region, the first row's remaining suffix, and later rows. The
checked midpoint lies inside the recognised run. A caller must choose a region
and length appropriate to its known option and supply its own action guard.

## Timing, cancellation and delivery

The native adapter reuses existing capture timeouts and cancellation, QMP
timeouts, `POINTER_SETTLE`, `MOUSE_HOLD`, `KEY_HOLD` and the existing sliced
`cancellable_wait`. No new timing value is introduced. Callers choose an existing
named settle duration; `Session::wait` retires input authority before waiting.

Session input ordering is uncertainty check → consume frame → active check → checked coordinates →
same-frame game/action guard → active check → fresh probe → active check → one
attempt. The latch is set before attempted delivery and cleared only by ACK.
Any delivery error keeps it set permanently, including after subsequent
captures. An ACK is recorded before the post-delivery active check. STOP never
clears the latch or authorises cleanup/restoration. Each input needs a new
capture; the selected game's guard remains responsible for recognising a
supported current scene and expected action.

Restoration owns finite observation budgets, not wall-clock guesses. Its adapter
also retains existing deadlines. A transport or cancellation error propagates
immediately. No generic pixel-change percentage establishes completion.

## Preserved Pyramid Undo All protocol

Pyramid still requires complete stock acquisition, positive empty LEFT and
exactly 24 acknowledged Draws before its known Undo All control can be used.
The toolbar icon remains advisory. Requiring enabled-icon pixels would change
the accepted existing reconnaissance protocol.

The shared routine preserves this observable order:

1. One Undo All input, then an immediate observation. The same current board may
   wait within the existing six-observation budget, settling only between
   remaining misses. Unexpected scenes fail.
2. A complete confirmation signature, followed immediately by a second complete
   confirmation with the same OK point. One guarded OK input follows.
3. Exactly one settle after OK ACK, then bounded restoration observations.
   Two consecutive original tableau/LEFT/RIGHT arrangements succeed with no
   added wait between the matching observations. The old current arrangement
   or same confirmation resets the consecutive count and waits only when a
   further attempt remains. Other scenes fail.

Only helper success sets `progress.restored`. Complete-deal validation,
cancellation checks and atomic JSON publication remain with the Recon caller.
The original diagnostic messages and uncertainty ledger are retained. The
shared helper neither deletes learned stock identities nor resets progress.

## Exploration and retained knowledge

A future workflow can observe, select one deliberate move, capture its result
and add independently recognised revealed identities to a knowledge store. It
should keep three distinct records: current physical position, accumulated
card knowledge, and acknowledged/verified move history. Operation frame serials
and delivery records support correlation; existing worker diagnostics retain
the relevant pixels and first failure.

Before an Undo, the adapter must know its semantic predecessor and the actual
current physical snapshot. A compound action or automatic foundation transfer
must be included in that transition; one UI Undo is not assumed to equal one
planner move. Only verified restoration changes the physical/history cursor.
Previously observed identities remain known. Conflicting new observations
require reconciliation rather than overwriting knowledge silently.

Undo All uses a separately retained original snapshot and preserves learned
knowledge while returning physical position. Failed or uncertain delivery
stops exploration and requires operator recovery; it does not trigger another
Undo, another OK or route replay. This revision supplies the operations and
restoration protocol, not a new exploration solver or a universal history model.

## Verification and remaining adapters

Focused tests exercise inclusive edges, stride/padding, maximal runs,
ambiguity, exact black, frame retirement, guard/probe/STOP ordering, permanent
uncertainty, ACK followed by interruption, partial Pyramid reads and semantic
single Undo. Local AF_UNIX QMP tests exercise the native bridge's actual capture,
mapped mouse press/release, typed D down/up and STOP after probe. They use a
simulated peer and retained PNGs; they do not capture or control the guest.

Existing Recon tests remain the integration evidence for modal signatures,
capture/settle ordering, observation bounds, STOP/context sweeps, delivery
uncertainty and publication after restored evidence. Required whole-package
qualification follows the source freeze. Fresh live acceptance remains a
separate check.

Next adapters should add measured single-Undo enabled/disabled appearances and
physical predecessors, then per-game locations/card reading where useful.
Option callers must define their known region and linear-group policy. Each
adapter needs positive and damaged/ambiguous evidence before enabling input.
Further keys, multi-card transfers and exploration policy need their own
contracts; none requires changing this revision's capture protocol.

## Code Analysis

Threat surfaces are decoded capture storage, selected socket/context, option
search bounds, recognised input targets and delivery responses. Existing
transport/decoder limits remain authoritative. Native validation and checked
frame/rectangle access reject malformed layouts; signed/unsigned conversions,
inclusive extents, run suffixes, midpoints and frame serial increments are
checked. No unsafe code, unchecked production indexing, new threads or raw
buffer ownership scheme is introduced.

Wrong or incomplete evidence cannot itself authorise an action: the caller's
scene/action guard and a fresh running/absolute-pointer probe are still
required. A partial mouse/key delivery is permanently uncertain, and ACK is
never promoted into a visible effect. Restoration requires supported complete
semantic snapshots. Unavailable single-Undo calibration remains a capability
gap, and the option detector conservatively refuses multiple horizontal groups.
