# Pyramid Gate 3 stable modal-text correction

Date: 11 October 2026, Australia/Brisbane. Package remains 2.1.0.

Subsequent acceptance: Charlie reported a successful complete export at
`/home/charlie/tmp/pyramid-recon-20261011-04.json`, with worker-verified original
restoration. The schema-1 file is 3253 bytes, SHA-256
`21a61672fa37840fc0b4c14dbe54aa437ecf6fa8d250dc8c8569985fe7c6d180`.
Its exact 28+24 standard-deck inventory/order matches the previous retained scan;
provenance records 31 capture attempts and two-frame restoration. The pending
statements below describe the earlier correction checkpoint, not this later
acceptance. The [schema-2 record](release-v2.2.0.md) preserves this original.

## Repeated live failure

Run `.pyramid-recon-2887595-1` loaded v3, not a stale profile. It recorded
the same 28 tableau identities and 24-card first-draw order as the previous
attempt, with exactly 52 distinct standard-deck identities. It acknowledged
24 D operations and one Undo All; zero OK operations were attempted. Restoration
is false and complete publication is `not_attempted`. These diagnostics cannot
be promoted into completed route input.

The final original PNG is retained without transformation as
`tests/fixtures/pyramid-recon/originals/gate3-undo-modal-repeat.png`, SHA-256
`0ff2e8c36f59a0eb13dd495ed9d859428bd7bdcefad963bdcf63b313463a9510`.
It is a 1,615,302-byte 1920x1080 RGB PNG. Original failure evidence and the
previous profile/manifests are backed up in `/tmp/pyramid-recon-stable.BxGtrrEm/`.
No assistant guest input, automatic resume, cleanup click or process restart
was performed. The user retains authority over the modal and manual recovery.

The title/body/panel/backdrop checks passed. Five Cancel pixels reached channel
delta 10 against tolerance 9; five OK pixels reached delta 9 against tolerance 8.
The preceding correction fitted bounds to observed maxima without margin for
another native rendering. The observed instability is concentrated around text
transitions; its precise guest rendering implementation has not been established.

## Reference-only selection

v4 changes only its profile ID and four existing text-patch masks. It retains
all RGB values, ROIs, targets, readers and tolerances. For each centre pixel:

1. Select it if its original RGB is exact ink: `(255,255,255)` for title/body/
   Cancel, or `(0,0,0)` for OK.
2. Otherwise select it only if each RGB channel has `maximum - minimum <= 16`
   across the original 3x3 neighbourhood centred there.
3. Read the neighbourhood from the full original frame, including a one-pixel
   halo outside the unchanged ROI. Ignore alpha; store unchanged source RGB.

Selection depends solely on the original `undo-all-confirmation.png`, SHA-256
`78da9587035ee2707ffacf4adaf493a2a55e1ed5c28712bd5765dca23b30a9e0`.
No failed-frame mismatch selects or exempts an individual pixel. Mask hashes,
counts and this rule are recorded in `profile-provenance.json` and reproduced
by a Rust regression test.

| Patch | ROI pixels | Selected | Pure ink retained | Unchanged tolerance | Largest observed selected delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| Title | 3916 | 2598 | 906 | 9 | 5 |
| Body | 4032 | 2383 | 337 | 10 | 7 |
| Cancel | 3689 | 2333 | 436 | 9 | 5 |
| OK | 2470 | 1758 | 250 | 8 | 6 |

Observed maxima cover the previous worker modal, operator screenshot and latest
failed modal. Repeated frames are not independent deployment coverage. The
remaining margins do not establish a bound on future rendering variation.

An initial flat-neighbourhood-only experiment removed all positive evidence
for two thin body components. It was rejected, not installed. The retained
pure-ink branch is essential; do not replace this rule with flatness alone.

**Recognition trade-off:** excluded pixels are entirely unconstrained, so these
masks widen acceptance despite unchanged channel tolerances. They retain pure
ink and locally flat samples, not necessarily only backgrounds or only pixels
away from all edges. The cropped signatures are not full-text OCR. Foreground
component, stroke and negative-space tests assess the retained distinctions;
they do not establish universal immunity to near-identical alternative dialogs.
No native near-identical non-Undo confirmation has been supplied.

Independent review also identifies two body `e` counters with no selected
negative samples: filling those small counters can leave this signature matching.
That is a demonstrated shape-level blind spot, not a supplied alternative native
dialog accepted as Undo All. Title, other body strokes, controls and backdrop
remain necessary. Do not describe these masks as checking every stroke/counter.
The body-relative bounding rectangles are `(5,9,7,4)` and `(33,9,7,4)`;
native rectangles are `(857,483,7,4)` and `(885,483,7,4)`. Independently filling
either rectangle white preserves all seven component matches, reproduced in
`counter-blind-spots.log`. No native PNG was modified or derived PNG saved.

All seven same-frame components remain mandatory: panel, title, body, Cancel,
OK and two dimmed non-card backdrop patches. Panel/backdrop masks remain dense
and their tolerances remain 4. The controller still requires a fresh second
complete confirmation before OK, then two fresh restored-board observations
before publication. STOP/context revocation and uncertain-input non-replay are
unchanged. Other games and the deferred Spider XP behaviour are untouched.

## Runtime identity and recovery

Profile: `tests/fixtures/pyramid-recon/native-profile.json`.
ID: `pyramid-native-1920-classic-blue-v4`; schema 1.
SHA-256: `796f3debfa098715fac781f4aa69de1dc4eaefb91af5ca996db7dc8e12474577`.
Runtime Rust is unchanged; the Rust additions are tests. The profile loads
before every new operation, so an idle verified 2.1.0 controller needs no restart
solely for this data correction. A failed operation cannot resume.

When ready, Charlie must manually confirm the specific Undo All dialog once,
without selecting its suppression checkbox. Verify the original 28 cards,
initial LEFT King of diamonds, empty RIGHT and Solver off before a new scan.
Use a new output pathname/deal ID and renew untouched-board confirmation after
editing fields. Preserve prior diagnostics. Only a successful new scan can prove
automated restoration and publish the complete JSON.

## Validation

Evidence root: `/tmp/pyramid-recon-stable.BxGtrrEm/`. `native-first-failure.log`
reproduces v3 refusal on the latest native frame. `profile-audit.log` records
29 unchanged original hashes/sizes/dimensions and all 14 unchanged RGB crops.
The final focused suite passes 53 tests. It recognises the original modal and
all three later native observations. Tests reproduce each mask exactly, require
each of the seven components on every later observation, check selected ink and
negative-space samples at the inclusive tolerance and one beyond in every RGB
channel, and explicitly demonstrate that excluded-pixel changes are ignored.
Twenty-five source foreground-component x-bands retain pure-ink evidence whose
erasure alone refuses. These are not asserted to be 25 complete letters: crops
can split a letter or group strokes. Targeted synthetic O-counter filling,
A/l bridging and thin-l erasure require selected mismatches and all refuse.
The independent eight-connected component audit uses `min(RGB) >= 192` for
white text and `max(RGB) <= 64` for black. It includes small components: the
only additional component is one body ROI-edge pixel at relative `(0,8)`, RGB
`(193,208,228)`, which is excluded. This edge fragment is an explicit blind spot,
not a claimed retained letter. All 25 components of at least eight pixels retain
positive evidence. See `foreground-components-final.log`; the initial overly
strong all-components assertion is preserved in `foreground-components.log`.
Wrong-text shifts, generic OK-only evidence, contradictory scenes and seven
other-game native scenes remain negative cases. They are not a native corpus
of near-identical competing confirmation wording.

| Check | Actual result | Evidence |
| --- | --- | --- |
| Final focused reconnaissance suite | 53 passed, 0 failed, 0 ignored | `focused-final.log` |
| Full application suite after review, including ignored host tests | 695 passed, 0 failed, 0 ignored; 82.67 seconds | `full-tests-final.log` |
| Cargo check | Passed | `check.log` |
| Final all-target Clippy with `-D warnings` | Passed | `clippy-reviewed.log` |
| Strict private rustdoc, `RUSTDOCFLAGS=-D warnings` | Passed | `doc.log` |
| Native-host release build | Passed; executable hash unchanged | `release.log` |

Commands use `--locked --offline` and the external paired-development config
`/tmp/pyramid-recon-build.5qifGKcG/local-xsar.toml`. Compiler is
`rustc 1.101.0-nightly (32dba69d6 2026-10-09)`; Cargo is
`1.101.0-nightly (29c5daa1a 2026-10-07)`. The native release executable retains
SHA-256 `84c68c76e9ef518ef36a79e5e036163092394bc9b9ca92b07694ea4f1c51540d`.
The previous guarded launcher is backed up here; its updated profile hash is
syntax-checked without launching. No formatter, installation, xSAR code change,
dependency/version change or Git mutation occurred. Live restoration/export
remain the next human acceptance boundary.

The read-only subagent review independently reproduced the masks and native
positives, identified the counter blind spots, and corrected a bridge-test
rectangle to exercise an actual joined stroke rather than only gap intrusion.
The final full suite includes that correction. Its report is retained as
`independent-review.md`; no blocking implementation concern remained within the
bounded signature contract. This review does not establish live acceptance.

## Code Analysis

This is a bounded calibration correction using the existing safe-Rust matcher,
not a new recognition API or input path. Memory, checked frame/ROI arithmetic,
capture budgets and publication bounds are unchanged. Reducing pixel constraints
trades sensitivity to unstable samples against discrimination; retained positive
and negative evidence is tested explicitly. Unknown/ambiguous scenes still revoke
authority. Successful offline recognition does not prove live OK delivery,
restoration, file publication or console consumption.
