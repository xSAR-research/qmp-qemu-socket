# Pyramid reconnaissance to JSON

> Historical implementation record. The version, dependency, profile, manual
> setup and positive Solver-OFF descriptions below record the dated 2.2.0
> delivery; they do not describe current 2.5.103 operation. Use the
> [current profile contract](pyramid-recon-profile.md),
> [2.5.103 verification](Verification-2.5.103.md) and
> [current live brief](Live-Test-2.5.103.md). Current operation bundles exact v7
> calibration, supplies editable evidence names and hands a successful export
> to Calculate automatically. Legacy `solver_off` data is validated and discarded;
> explicit foreground/no-interference/Solver-not-engaged consent remains required,
> and an unavailable Solver is allowed. The dated evidence below is preserved.

Implementation scope authorised by Charlie on 11 October 2026. This operation
is separate from ordinary read-only Pyramid preparation and all five existing
Computer Vision controllers. Console intake is deferred; no search is included.

## Baseline and delivery boundary

Current package 2.2.0 consumes xSAR 0.4.0's strict schema-1/2 reader and schema-2
writer. Every exported card has canonical `card`, integer `value` (A=1 through
K=13) and `suit`. Suit remains essential to acquisition identity and complete-deck
validation even when a later game's matching rule ignores it. Mapping and codec
live in xSAR; no local serializer or rank-value table is duplicated here.
The native calibration profile remains schema 1/v4; it is not the deal schema.
See [the schema-2 delivery record](release-v2.2.0.md).

The following paragraphs retain the initial reconnaissance implementation base:

Application base: ba1a82beb3ea39242d91ead39ebdbd1198847f84, version 2.0.11.
New application slice: 2.1.0. xSAR base remains
987856a0b5cb95aec2788551e050e18f7469b1ef, with its reviewed local 0.3.0 foundation.
This slice adds optional JSON filesystem publication to that unpromoted crate.
Keep both repositories' editions, declared MSRV and existing dependency versions.

The current application manifest and lockfile pin the complete xSAR 0.4.0
implementation at `ab226a65d3bbc2b4914a5cecfb2d3247364c7536`. Normal builds and
validation use this Git source without a local checkout or source override.
Earlier paired validation remains historical evidence; its temporary configs
and host paths stay outside promotion files. Application promotion follows
Git-backed checks and consumer acceptance. An automated test result does not
establish installation or live guest behaviour.

## Operator contract

Select Pyramid independent preparation, then use the separately labelled
reconnaissance operation. Supply a native calibration profile, a new JSON output
path and a bounded deal identifier. Explicitly confirm an untouched supported
CLASSIC BLUE deal, Solver off, at 1920x1080 and 100% guest scaling. Confirmation
belongs to the selected context and is consumed on dispatch; changing selection
requires a new confirmation. Preview pixels do not grant input authority.

No built-in dialog signature is invented. A missing or incomplete calibration
refuses the operation before guest input. The native profile format and its
provenance requirements are documented in
[the profile contract](pyramid-recon-profile.md).

Normal Capture remains read-only, and ShortestPath.permits_input remains false.
The dedicated command grants only stock D, one calibrated Undo All attempt after
the complete confirmed acquisition history, and the supported confirmation's
freshly recognised OK point. It cannot click
tableau cards, activate Solver, recycle, start another game or execute a route.

## Acquisition and restoration

1. Capture and recognise the original 28 tableau cards in logical top-down,
   left-to-right order. Read rank and suit. Recognise initial LEFT as stock
   ordinal 0 before D; require positive empty RIGHT and two stable fresh board
   observations. Reject duplicate identities before input.
2. Probe VM running state and the active absolute pointer before each input,
   with STOP/context checks after probes and immediately before delivery.
3. Send one D and wait. Only a newly recognised LEFT identity, the previous LEFT
   identity positively recognised in RIGHT, and an unchanged tableau establish
   an advance. Record the next stock ordinal once. Repeated supported frames
   cause input-free observations, never another blind D.
4. After all 24 unique stock identities, require positive empty LEFT and the
   last identity in RIGHT on two consecutive fresh observations. Early emptiness,
   unknown/ambiguous cards, duplicate identity or a changed tableau stops safely.
5. After 24 acknowledged D operations, all 52 unique identities and two positive
   empty-LEFT observations, attempt Undo All once at the shared calibrated point
   `(1320,978)`. Confirmed operation history supplies availability; the toolbar
   icon match is advisory across background themes. Fresh running-VM and absolute
   pointer probes, STOP and selected-context checks still precede the attempt.
   Observe its full specific
   confirmation, then obtain another complete fresh matching signature before
   its single OK click. Generic gold/OK decoration is never sufficient.
6. Verify the original tableau, initial LEFT identity and positive empty RIGHT
   twice on consecutive fresh observations. An input acknowledgement alone does
   not establish restoration. Original deal knowledge is retained independently
   of the live scan cursor.
7. Construct the shared complete-deal model, recheck authority and publish only
   after restoration. Unknown identities are never supplied by elimination.

STOP, socket/game/strategy/generation changes or uncertain input delivery revoke
further input, including restoration. There is no automatic cleanup clicking
after failure. Inspect the guest manually before starting another operation.

## Bounds and evidence

- At most 24 D attempts, one Undo All attempt and one confirmation attempt.
- At most 180 capture attempts across the entire operation.
- At most six fresh input-free observations per unresolved advance, dialog
  appearance or restoration phase. Initial/exhaustion/restoration evidence is
  explicitly counted and cannot append duplicate physical cards.
- One-second reconnaissance waits, separate from unchanged CV timing controls.
- A 300-second cooperative elapsed deadline checked between operations. Existing
  per-I/O timeouts remain; this is not instantaneous cancellation or a hard
  wall-clock guarantee for filesystem/socket operations.
- Every successful capture updates the existing bounded latest-frame preview.
- Private diagnostics retain original baseline and latest/failure PNG bytes,
  partial recognised identities and input-attempt/acknowledgement counts.
Partial diagnostic JSON is explicitly separate from the planning-input schema.

Freshness means separate successful capture operations; identical pixel content
is not itself stale, nor proof that a previous key was consumed. A new stock
ordinal needs the full identity transition above. Diagnostics do not grant
resume or replay authority.

The expected universe is the closed set of 52 standard rank/suit identities.
At run start no identity has been observed: absence from the recognised tableau
and stock records means unobserved, not an inferred card. The completed model
requires 28+24 unique identities from that universe; initialisation alone cannot
establish completeness. The original arrangement, accumulated stock knowledge
and current live pile observation are separate values.

## Publication

The optional xSAR JSON writer validates and serialises before filesystem
publication. It uses a private same-directory temporary file, synchronises its
contents and publishes without replacing any existing destination. Existing
files and symlinks are refused. Publication/cleanup/durability failures must
distinguish a definitely created output from uncertain publication, preventing
misleading retry instructions. UI path selection remains application-owned.

The complete JSON contains the immutable original arrangement and ordered stock,
not current board coordinates. Native diagnostics remain separate provenance.
An independently supplied JSON restoration declaration is not authenticated proof
of screen activity; only the local reconnaissance evidence supports that claim.

## Native evidence and acceptance boundary

The twenty originals indexed in [the earlier calibration](pyramid-calibration.md)
were recovered and their recorded SHA-256 values verified. Image 20 supplies an
initial-looking Solver-off board with all 28 tableau cards, LEFT Q of clubs and
empty RIGHT. The set includes all ranks and suits in tableau corners, selected
LEFT identities, empty-LEFT artwork and enabled/disabled Undo All toolbar states.

Charlie additionally supplied the original 1920x1080 Undo All confirmation PNG,
`qmp-qemu-socket-PYRAMID-Undo-All-OK.png`, SHA-256
`78da9587035ee2707ffacf4adaf493a2a55e1ed5c28712bd5765dca23b30a9e0`.
The modal is independent of the underlying board state; no separate initial,
exhausted or restored dialog variants are required. Its unchecked optional
checkbox is not an input target. Calibration requires the complete modal, not
only the gold OK button. This image establishes the dialog appearance, not
the full scan, initial deal or restoration.

Charlie also supplied native Solver-off starting and exhausted-LEFT captures:

- `qmp-qemu-socket-PYRAMID-start-no-SOLVER.png`, SHA-256
  `4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35`:
  full original tableau, LEFT K of diamonds, positive empty RIGHT.
- `qmp-qemu-socket-PYRAMID-exhausted-left-DRAW-pile.png`, SHA-256
  `b352d1645b15b2cb58d2a7a64e8b25a845ca91c955e3d66861cb76be1d352ef6`:
  the same tableau, positive empty/recycle-artwork LEFT, RIGHT Q of diamonds,
  and enabled Undo All. The recycle artwork never grants another D operation.

These observed endpoints do not identify the 22 intervening stock cards or prove
every advancement/restoration transition. Live end-to-end and STOP evidence remain explicit
acceptance requirements; synthetic state-machine tests do not satisfy them.

Automated acceptance covers ordinal ordering, duplicate/conflicting identities,
unknown scenes, repeated frames, input uncertainty, cancellation/context changes,
positive exhaustion/restoration, budgets and no-clobber publication. Native
reader fixtures must retain their provenance and mark coverage gaps honestly.
Record actual test/build results in the delivery report; this contract is not
itself a claim that those checks or live reconnaissance succeeded.

Use [the live-test brief](pyramid-recon-live-test.md) for Charlie's Gate 3 checks.

## Code Analysis

Untrusted boundaries include calibration JSON, captured pixels, QMP replies,
filesystem paths and terminal/UI diagnostics. Checked frame/ROI operations and
closed card identities prevent unchecked indexing and partial planning input.
No new transport, host-wide input or unsafe code is required. Memory/disk bounds
are tied to profile, PNG, card-count and diagnostic limits, not exact RSS.
The safest response to unknown/ambiguous native evidence is to stop without
replay or unapproved cleanup. Atomic no-clobber publication does not guarantee
directory durability on every filesystem; publication status must stay explicit.
