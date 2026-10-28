# Verification - qmp-qemu-socket 2.9.158

Charlie approved the recognition and diagnostic corrections and version in m02419.
m02442 explicitly excludes RESET. The exact 157 Destination Sentinel returned
HANDOFF VERIFIED before edits. The existing controller was left under operator
control throughout; no agent GUI launch, guest input, new guest capture,
installation, staging, commit or push occurred.

## Observed failures and correction

157 Request 8 delivered one acknowledged challenge REDEAL and then exhausted six
read-only transition observations. A subsequent explicit Recon on the same board
failed at the Five of spades in fixed location 5|4. The game pixels of both saved
frames are identical above the taskbar. The unchanged native reader's suit score
was 221119 ppm against its 200000 ppm limit, with an 11693 ppm same-colour lead
against the required 40000 ppm. No input or capture failure is inferred from this
recognition refusal.

The Classic bank gains one measured complete Spade appearance. The original
4557-byte, 31 by 49 RGB corner is compiled through the existing reader and checked
against its original Five rank. This extra suit requires an independently accepted Five rank and a separator
that does not split a connected ink component. Other ranks retain their original
suit bank. An earlier unconditional variant
made a clipped Ten of clubs ambiguous; that failed prototype was retained and the
separator guard resolves it. The first full suite then exposed a retained Seven
of clubs ambiguity when the sample was offered to other ranks: 1571 passed and
three retained-board checks failed. The final correction preserves the original
Five-rank context and includes that Seven in the focused Club regression. First
failed logs and source seal remain preserved. Quality, gap, colour-competition
and native geometry checks remain.

A separate 157 Underwater entry failed before any Recon input. Its brown printed
black suits and different complete font were unsupported by the existing banks.
The new Underwater family is compiled from thirteen untransformed native RGB
corners covering all ranks and four suits. Its brown policy applies only to that
family: ordered R >= G >= B, R <= 160, B <= 110, R-G <= 64 and 12 <= G-B <= 96,
with the existing ink, growth, seed-support and complete-glyph requirements.
Its measured card-relative separator is exactly row 30; its rank boundary permits
the full Ten at x < 34, and its native-size margin is two pixels. These settings
are confined to the new family. Classic, Retro and Western retain their own
policies. Matching remains at rank 220000 / gap 80000 ppm and suit 200000 / gap
40000 ppm, with translation only and no glyph scaling.

The native Underwater Queen and Heart touch across row 30. The fixed measured
separator retains both complete glyphs. Erasing a rank, suit or the zero of Ten,
and injecting conflicting printed ink, are tested refusals. Calibration does not
authorise an incomplete or unknown board.

## Native sources and evidence limits

[Fixture provenance and calibration](../tests/fixtures/pyramid-recognition-158/README.md)
and [the bounded recognition profile](pyramid-recon-profile.md) record the three
byte-original PNGs, raw crop hashes and all thirteen Underwater reference choices.
Charlie supplied the second Underwater frame with visible Three and Nine and
confirmed it came from Capture PNG at 100% scale, 1920 by 1080. The images and
calibration pixels were neither resized nor resampled.

Both Underwater tableaux and their three visible stock faces are manually labelled:
59 observed identities, including 13 training corners and 46 other corners in the
same two source frames. Those other corners are within-source held-outs, not
independent live runs. A black Three is absent from this retained corpus; arbitrary
theme, font, scale or rank/ink-colour coverage is not established. The second frame
has highlighted Nines, an existing draw and an occupied RIGHT pile. Its identities
are useful references but that state is explicitly refused as fresh entry.

The original source cases remain in:

- `/home/charlie/Documents/Pyramid/Validation/pyramid-challenge-redeal-157-20261027-uantoz26`
- `/home/charlie/Documents/Pyramid/Validation/pyramid-recon-suit-157-20261027-djbejwrm`
- `/home/charlie/Documents/Pyramid/Validation/recognition-157-to-158-20261028-3dxw5v0d/new-theme-entry`
- `/home/charlie/Documents/Pyramid/Validation/recognition-157-to-158-20261028-3dxw5v0d/underwater-missing-ranks`

No other temporary error packets were used to infer these failures. Original
filenames, packet boundaries, log snapshots and first failed prototypes are kept.

## Same-frame entry and transition diagnostics

The native classifier now returns its fact and optional reader refusal together.
Each successful capture starts with no retained refusal. Initial-entry failures
include both actual frame classifications; transition exhaustion includes its
last capture index and corresponding refusal. Reader errors remain pending
observations and do not become early fatal errors during dealing animation.

This adds no capture, waiting allowance, input authority, recognition relaxation
or ordinary-gameplay PNG recording. Pair equality, exact header binding, the six
ordinary reads and existing conditional normal-only seventh confirmation are
preserved. The tests check that a later successful frame clears the earlier
reason and that the last failed frame reports its own slot.

## Regression and host qualification

Fifteen new regressions exercise the complete retained boards and stocks, byte
binding of references, damaged and conflicting glyph refusals, the native-v23
challenge REDEAL transition, fresh Underwater entry, occupied-RIGHT refusal,
STOP/context revocation, and current-frame diagnostic replacement. Native adapter
tests use private AF_UNIX mocks and explicitly synthetic repeated frames; they
do not constitute fresh guest observations or live input-effect acceptance.

Fourteen focused cases passed together; the additional bound-transition case
passed after the test adapter selected native-v23 rather than the old native-v17
profile. The full suite below includes all fifteen. Pre-correction failures,
prototype ambiguity, the first Underwater size/connected-glyph failures and a
corrected manually labelled Ace suit are retained with their original logs.

| Gate | Result | Elapsed |
| --- | --- | --- |
| scoped-format | PASS | 0.168 s |
| check | PASS | 3.007 s |
| rustdoc | PASS | 1.561 s |
| clippy | PASS | 3.544 s |
| full-tests | PASS | 572.689 s |
| release | PASS | 14.675 s |

Full tests: 1574 passed, zero failed, ignored, measured or filtered. The commands
use native nightly, `--package qmp-qemu-socket --locked --offline --target
x86_64-unknown-linux-gnu --all-features`; tests include `--include-ignored` with
eight threads. Rustdoc checks private items with `-D warnings -D missing_docs`;
Clippy checks all targets with `-D warnings`. Formatting checks the changed ranges
and preserves the repository's two-blank-line convention. Raw logs and exact argv
are recorded in `/home/charlie/Documents/Pyramid/Validation/recognition-157-to-158-20261028-3dxw5v0d/application-158/qualification.json`.

Qualification sealed 836 inputs. Only the approved own-package version fields in
Cargo.toml and Cargo.lock changed from 2.9.157 to 2.9.158 after all six gates passed.
The aligned locked check, native release and protected help passed; the other 834
inputs, including all 52 xSAR inputs, are identical. xSAR remains 0.9.0 at
`d20dca8ad2320dcbbf869e22b35a2583317d4569`; its retained 296 tests and three Miri
checks are cached evidence. The relative path dependency is not a Git pin.

Protected executable: `/home/charlie/Documents/Pyramid/Validation/recognition-157-to-158-20261028-3dxw5v0d/application-158/native/qmp-qemu-socket-2.9.158`.
SHA-256: `8a4ef553145dada1eeefe47c86d9596bbfb0c9f626eb2dc84857754295d7afc5`.

Baseline_HEAD_SHA: `58aa2882fb0c4ff3aa0d71ae49de0871545b86e3`; parent `b67d93942c68c55bfebe90e0f4e1e03f57300740`. The application changes are
uncommitted. The explicit complete promotion inventory, review and exact new
destination sentinel are in `/home/charlie/Documents/Pyramid/Validation/recognition-157-to-158-20261028-3dxw5v0d/application-158`. Older qualification/publication records and
protected executables remain unchanged.

## Operator acceptance and separate scope

Charlie reported more than an hour of successful 157 Continuous operation before
the theme-specific failures. This is operator-reported evidence; it does not
establish fresh acceptance of 158. Test the corrected Five of spades challenge
REDEAL and initial Underwater entry/Recon under the selected challenge method;
confirm actual restoration and route execution before accepting guest behaviour.
When a refusal is observed, preserve its packet and log before restarting or
repeating an operation.

RESET remains a possible future quality-of-life item and has no implementation in
this revision. Free Cell remains a saved plan with its separately approved future
3.0.0 classification; implementation is unapproved. Archive selection remains
deferred until 3.0.0 is ready after testing. STOP/context, fresh probes, no uncertain
replay, exact restoration and the GAME WIN image boundary remain authoritative.

## Code Analysis

The new policy is private, bounded by a fixed native corner and a finite complete
reference bank. Checked frame access, original component support, colour
competition, complete rank/suit proof and occupancy checks remain in force. Brown
subtractions follow ordered channel comparisons, so they cannot underflow. No
unsafe code, new dependency, transport, public API or input operation was added.
Diagnostic strings describe the same captured frame and cannot grant authority.
The principal residual risk is an unrepresented native glyph or theme appearance;
that must remain a refusal and is not converted into an inferred card.
