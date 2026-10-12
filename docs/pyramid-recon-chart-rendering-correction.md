# Pyramid Gate 3 chart-rendering correction

Date: 11 October 2026, Australia/Brisbane. Package remains 2.2.0; this is a
bounded native-profile correction, not a new input operation or JSON schema.

## Failure and evidence

Run `.pyramid-recon-2974057-2` refused its first native observation. It attempted
one capture and zero D, Undo All or OK inputs. No input delivery was uncertain;
complete JSON publication was `not_attempted`. The `restored=false` diagnostic
means restoration was not verified by this failed run, not that it changed the
board. The original diagnostics remain untouched under `/home/charlie/tmp/`.

The identical baseline/latest PNG is retained without transformation as
`tests/fixtures/pyramid-recon/originals/gate3-chart-rendering-refusal.png`:
2,789,817 bytes, 1920x1080 RGB, SHA-256
`07b8cf87aeb2a5ad8b6c3afd5f2e1fcef87c83855a65846ff1c92ad0651173cb`.
The captured game labels the deal **Grandmaster**, not Easy. Its visible
tableau and initial LEFT King of diamonds match the calibration arrangement;
this image alone cannot establish hidden stock order or a new Easy deal.

The actual running PID 2974057 was inspected through `/proc/2974057/exe`.
Its native release path and SHA-256
`37314eeb509667231dbf48ec36324bc4829b91595778ba0cd029611c0f83953c`
match the intended v2.2.0 build. The reported profile is v4, matching the local
file hash `796f3debfa098715fac781f4aa69de1dc4eaefb91af5ca996db7dc8e12474577`.
This was not diagnosed as a stale executable, Undo All failure or JSON failure.

Independent RGB evaluation reproduces the production matching condition:
ten selected samples in `board.scene[1]`, the 9/4 rule-chart patch
`[1815,281,67,42]`, exceed its channel tolerance 8; the maximum delta is 10.
The K chart passes at its limit 6. The stock arrow, Solver-OFF and empty-RIGHT
patches match exactly. Disabled Undo and occupied LEFT correctly fail their
enabled-Undo/empty-LEFT patterns; those are not initial-board prerequisites.
The variation is concentrated around glyph transitions. Its precise guest
rendering mechanism is unverified; anti-aliasing-like variation is an image-level
description, not a proven implementation cause.

## Source-only correction

Profile `pyramid-native-1920-classic-blue-v5` changes only its ID and the masks
of the two chart patches. Every RGB value, ROI, channel tolerance, card reader,
modal mask and target remains unchanged. The reference remains the original
`current-start.png`, SHA-256
`4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35`.
No live mismatch determines an individual exemption.

For each original chart sample, select it if any of these conditions holds:

1. Its whole original ROI row qualifies as a separator: each bright sample has
   `min(RGB) >= 192`, and `5 * bright_count >= 4 * row_width`. Retain the entire
   qualifying row, including its margins. This preserves the off-white rules.
2. Its centre has `min(RGB) >= 249`, retaining bright stroke cores that are not
   necessarily exact white. This includes the thin 4 crossbar's RGB254 centres.
3. Every RGB channel's maximum-minus-minimum is at most 16 over the original
   full-frame 3x3 neighbourhood, including a one-pixel halo outside the ROI.

The exact-white/modal rule alone was rejected: it would omit the off-white
separator lines and the central 4 crossbar. The final rule is not that candidate.
These are measured calibration constants, not universal font-rendering rules.

| Patch | Selected / ROI | Separator rows, relative | Unchanged tolerance |
| --- | --- | --- | --- |
| K | 2355 / 3120 | 37, 38 | 6 |
| 9/4 | 1728 / 2814 | 2, 3, 40, 41 | 8 |

Mask hashes and the exact source rule are recorded in `profile-provenance.json`.
The final profile SHA-256 is
`b48e0b1a20aec2439422a1b967de1468d5553cc9a12077784356c3b68bcfd3e7`.

**Acceptance trade-off:** excluded transition samples become unconstrained, so
masking widens recognition even with unchanged tolerances. The retained ink,
negative space and separators are bounded sampled evidence, not full OCR or a
universal false-positive guarantee. All other board signatures and separate
Solver-OFF, card uniqueness, exhaustion, fresh modal, restoration, STOP/context
and publication requirements remain mandatory. Other games and Spider XP are
untouched. Observed headroom is not a guarantee about future rendering.

## Regression and validation boundary

The regression freezes the actual first-failure PNG. Reinstating the old dense
chart masks must reproduce `Unknown`; the bundled v5 profile must recognise the
same full typed board as the independently labelled source, including LEFT and
empty RIGHT. Source-only masks and exact threshold boundaries are reproduced.

Derived adverse tests alter only original foreground for stroke/separator
erasure, rather than repainting their backgrounds. They require distinguishing
selected witnesses for K stem/arms, the 9 tail, the off-white 4 crossbar/diagonal,
9/4 counter filling, an actual K stroke bridge and every separator. Selected
ink/background/separator samples exercise all RGB channels at their unchanged
cap and one beyond. Excluded-pixel mutations explicitly demonstrate the blind
spot instead of claiming complete shape checking. Retained other-game,
Solver-ON, incomplete-scene, STOP and exhausted-start refusals remain required.

Local validation completed successfully through job `job-mv3jl0co-68c83ac4`
in 124.4 seconds. The exact failing adverse test passed on its own; the focused
vision suite passed 41 tests, and the full host suite passed 702 tests with
`--include-ignored` (zero failed or ignored). `cargo check`, strict private
rustdoc, Clippy with warnings denied and the native release build all passed
with `--locked --target host-tuple`. Focused nightly rustfmt checking and
`git diff --check` also passed; unrelated formatting was preserved.

The independent audit verified all 30 original image hashes/dimensions and all
14 runtime RGB crops. Only the profile ID and two chart masks differ from v4.
The production Rust prefix is byte-identical to the promoted baseline; all
Rust additions are tests. The Git index remains unchanged.

The first validation attempt is retained honestly: 40 focused tests passed and
one newly added negative test failed because its bridge edit reached no retained
sample differing above tolerance. The corrected bridge connects the original K
stem and upper arm through two distinguishing negative-space witnesses. The
witness assertion was preserved, and the positive regression now explicitly
requires a Board result before comparing its complete typed arrangement.

Actual tools were rustc 1.101.0-nightly (32dba69d6, 2026-10-09), Cargo
1.101.0-nightly (29c5daa1a, 2026-10-07) and rustfmt 1.11.0-nightly. The release
checksum remains
`37314eeb509667231dbf48ec36324bc4829b91595778ba0cd029611c0f83953c`.
Evidence and first-failure logs are retained in
`/home/charlie/tmp/pyramid-recon-failure-2974057.cjxsCM2W/`; successful rerun logs
are in its `attempt-2/` directory. These offline results are distinct from the
subsequent operator acceptance below.

## Live acceptance and console input

Charlie reported that the new scan worked perfectly and supplied the worker's
complete-inventory/original-board-restoration success message. The exact file is
`/home/charlie/tmp/pyramid-recon-20261011-v5-012json`, 4,741 bytes, SHA-256
`29e4768874e87203d69a3881c2f16e06661ba1a6673a660a32d71cfda5a6ced9`.
The filename suffix is literal, not a missing file to repair automatically.
Its internal deal ID is `pyramid-recon-20261011-v5-02`.

A separate bounded structural audit confirmed schema 2, complete/restored
declarations, 28 tableau and 24 stock entries, every standard identity exactly
once and agreeing canonical labels/numeric values/suits. Provenance records
`qmp-qemu-socket/2.2.0`, screen recognition, 31 capture attempts and the two-frame
restoration check. This audit did not independently observe the guest or execute
the shared codec. The next console milestone must load the file through xSAR.
Difficulty and a winning route are not recorded; do not infer either. Producer
correction promotion remains Charlie's separate Gate 4 decision.

## Operator recovery

No assistant guest input, controller launch, process restart or cleanup click
was performed. The report records zero input attempts, so there is no failed
reconnaissance action to undo. Inspect the actual guest before choosing another
operation; the saved frame is not proof of its present state.

The profile loads afresh for each new operation. An idle, verified v2.2.0
controller need not restart solely for this data-only correction. A failed run
cannot resume. Local validation passed; now deliberately prepare
the intended untouched deal with Solver off, fresh confirmation and a new output
path/deal ID. Select Easy manually if that is the intended fixture; the retained
Grandmaster label must not be reinterpreted as Easy. Preserve all failure evidence.

Older launcher scripts pin the v4 profile hash and will intentionally refuse
v5. Do not weaken those checks or confuse that refusal with a binary mismatch.
The [live-test brief](pyramid-recon-live-test.md) now points to v5-pinned launch
and running-process verification scripts. Both were syntax-checked without
launching the controller. At the post-build process check, no controller was
running; Charlie must deliberately launch and verify it before a new scan.

## Code Analysis

Only calibration data and tests change recognition behaviour; production Rust
and its input whitelist/state machine are unchanged. No unsafe code, new
dependencies, mutable globals or unbounded runtime allocation is introduced.
The masks reduce constraints on selected chart edges, with explicit retained
stroke/separator evidence and documented blind spots. Unknown scenes still
revoke authority before input, and a recognised start alone never authorises
publication without the full inventory and verified restoration sequence.
