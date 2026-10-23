# qmp-qemu-socket 2.5.103 verification — host verified

Updated: 19 October 2026. Required host verification for this changed revision
passed. Application 2.5.103 earns one correction build increment from host-verified
2.5.102, with no new minor milestone. xSAR
stays 0.5.101; accepted console 0.8.4 stays frozen. M01–M09 remain accepted,
**M10 Awaiting verification and M11 Planned**. No current live acquisition,
computed execution, whole-challenge completion or promotion is claimed.

## Authorised correction and source boundary

Charlie first authorised removing the visual Solver-state gate to support the
legitimate grey unavailable control in Daily Challenges. He then authorised
bundled calibration and automatic evidence fields, superseding the earlier UI
deferral. Explicit operator consent remains required; automatic setup supplies
neither consent nor proof of a supported or restored board.

The preceding [2.5.102 verification](Verification-2.5.102.md) remains unchanged
historical evidence: 907 application tests and 256 xSAR tests passed, with zero
failed/ignored, plus its recorded full feature/consumer checks. Those results
do not verify this revision. The new work directory is
`/home/charlie/tmp/pyramid-solver-consent-fo4irhas/`.

Its `baseline.json` records existing dirty work, HEADs, indices and per-file
hashes. It was captured after the new daily fixture and initial regression edits
were written, so it is an in-progress correction snapshot, not a pristine
2.5.102 baseline or the complete correction diff. Review the explicit paths below
and the preceding M10 records as well as that snapshot. Application HEAD remains
`842646e24b179fb4c4ee2cd04f1dc28b8cfed30b`; HEAD alone does not identify the
accepted dirty source content.

xSAR 0.5.101 at HEAD `00986b8d621803fc98da85616444fc76c14638a9` retains its
previously verified dirty M10 implementation. The console at HEAD
`22f52d7ec5fc867b542dd7ca4c20fe2b9fbe2e84` retains its accepted dirty 0.8.4
baseline. This correction authorises no sibling implementation or version change.
The relative application path dependency on `../xsar` remains a path, not a Git pin.

## Recognition and configuration contracts

`BoardProfile` no longer stores a Solver pattern. Schema 1 still requires and
fully validates `board.solver_off`, including its native bounds, samples, mask,
contrast and tolerance. The validated pattern is then discarded. Reconnaissance
`observe`, exact route `verify_position` and final-new-board recognition use the
same Solver-independent scene predicate. This is an explicitly authorised gate
removal, not a new grey/gold colour heuristic.

All scene masks, specific modal signatures, dimensions/stride/storage checks,
exact rank-and-suit readers, duplicate refusal, positive-empty pile evidence
and positive removed-slot evidence remain enforced. No glyph sample, mask,
distance cap, alignment, geometry or old profile/PNG is rewritten. The visual
classifier no longer establishes Solver state: the operator must confirm that
Solver is not engaged, Solitaire is foreground and there will be no interference.
An unavailable Solver is allowed, and these operations never activate it.

`ReconProfileSource::BundledNativeV7` embeds the exact native-v7 profile by default:
806,442 bytes, SHA-256
`9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`.
`ReconProfileSource::File(PathBuf)` retains a literal custom-file override.
Both use the production strict parser. Logs and diagnostic start metadata
distinguish bundled/file sources and record the profile identity; source metadata
does not grant authority. Bundled operation does not require the runtime fixture
file to be present beside the executable.

Automatic evidence names prefer an existing eligible real runtime `$HOME/tmp`,
then `std::env::temp_dir()`. Their Deal ID includes UTC epoch milliseconds,
PID and a checked session serial. At most 64 candidate metadata probes occur;
collisions consume serials, and overflow refuses. The generator creates no files,
directories or destination reservations. Write bits are preliminary evidence only:
ACLs, mounts, races and actual publication remain worker checks. Naming failures
are shown while manual fields remain available.

Output and Deal-ID overrides are preserved independently. Successful acquisition
passes the exact completed export pathname to the route field, then rotates only
untouched automatic fields for the next acquisition. The initial route field
can show its suggested pathname while awaiting acquisition; Calculate remains
disabled until successful acquisition in the current context. Both consent
checkboxes start unchecked. Existing worker receipts and exact JSON-byte checks
remain authoritative; a UI prerequisite alone cannot mint a plan or input permit.

## Retained daily-challenge evidence

The source diagnostic directory is `/home/charlie/tmp/.pyramid-recon-645935-1`.
The exact copied fixture is
[`daily-challenge-solver-unavailable`](../tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/README.md):

| Retained file | Bytes | SHA-256 |
| --- | ---: | --- |
| `baseline.png` | 3,770,751 | `bac4632eca0bd9b5b31cc4e3f94bfa7123b5d6e204875f3d56505a2aac7718ac` |
| `INCOMPLETE.json` | 867 | `343d049b54c4cd1c1eb83b08e7511eefbe71d407b74f4f9f69e223aee154d76e` |

Source `latest.png` is identical to `baseline.png`. The old report remains
unchanged: one capture, zero D/Undo All/confirmation attempts, no uncertainty,
publication `not_attempted` and no restoration. All three board scene patches
matched; the former gold inactive-Solver gate failed against the legitimate grey
unavailable control before glyph recognition. Retaining or later recognising
that frame does not retroactively complete its acquisition.

The capture visibly contains `Boards cleared 0/2`, `1 deal left`, the labelled
28-card tableau, LEFT King of clubs and an empty RIGHT pile. This is 29 visible
identities, not a complete 52-card deal. The caption does not prove how the daily
stock/recycle rules correspond to the existing offline reference profile. Undo All
availability after the survey, restoration, selected-card tint, automatic redeal
and daily terminal behaviour remain live evidence questions.

## Current host verification

`verification.json` now records `passed-required-checks` for this revision.
`commands.json`, per-command logs and compiler-input hashes retain the exact
commands and source boundary. The first actual daily full-observe/exact-position
regression passed, followed by all eight daily regressions, including direct
embedded-profile/file-loader equivalence. These overlap; they are not nine
unique tests. The complete rerun then passed all 929 application tests, including
ignored tests, with zero failed or ignored. No result is inferred from historical
907/256 totals and no sibling test suite is claimed as newly run.

| Required scope | Current result |
| --- | --- |
| Focused actual daily-frame, bundled/file recognition and retained-gate regressions | 8 daily tests passed, zero failed/ignored; first one-test pass overlaps this set |
| UI defaults, independent overrides, route handover and explicit consent | Passed in the complete 929-test application suite |
| Worker bundled/file source, diagnostics and request authority | Passed in the complete 929-test application suite |
| Locked application check and complete tests including ignored tests | Check passed in 1.567 seconds; all 929 tests passed, zero failed/ignored, in 136.039 seconds |
| Strict private rustdoc and warnings-denied Clippy | Passed in 0.916 and 0.214 seconds respectively |
| Scoped formatting | Four new modules passed; edited parent nonempty lines match shadow formatting with six inherited two-blank-line contexts preserved |
| Native release, actual artifact selection and headless `--help` | Passed in 12.869 and 0.002 seconds respectively; exact artifact and checksum below |
| Source/evidence/index preservation; unchanged xSAR and frozen console | All indices, sibling files and compiler/fixture inputs unchanged; unrelated source/evidence preserved; automatic operational telemetry recorded separately |
| Read-only running-process/image-directory preflight | At 2026-10-18T19:03:05.063955+00:00: no application process; images empty, zero files/bytes; no QMP connection opened |

The new tests cover the actual unavailable-Solver frame through production full
observation and exact route comparison; toolbar mutations; missing scene, glyph,
empty-pile and modal evidence; the original failure report; bundled profile
provenance; bounded name generation/collisions/fallback; independent manual
overrides; successful-path handover; and unchecked, context-bound consent.
Existing M10 controller/wire, independent replay, STOP, partial-pair and
uncertain-delivery regressions passed again within the complete application suite.

The first complete suite also passed 929 tests, but its subsequent Clippy check
refused two test-only owned comparisons with `clippy::cmp_owned` at
`src/app.rs:4476:74` and `src/app.rs:4537:74`. The exact failed command, logs,
source and initial passing-suite evidence remain in `first-check-failure/`.
Comparisons now borrow `Path` values; the full required sequence above was rerun
after that correction. The earlier locked-check time of 6.326 seconds and first
suite time of 142.472 seconds belong to that retained first attempt, not the
final rerun. No lint was weakened.

Current formatting evidence is `format-blank-preservation.json` in the work
directory. Its six preserved contexts belong to this correction's three parent
files; they are distinct from the older M10 record's two formatting exceptions.
The initial formatter launcher omitted `--unstable-features` for `--file-lines`;
it failed before editing files. Adding the required flag allowed the scoped
check to complete. No package-wide formatting success is claimed.

The compiler-selected native artifact is:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
SHA-256 1b4a82cb55c2ca8415a49501e93f2286597a0e1e67c7db27d71b101eeb4ce733
```

Headless `--help` confirms `Usage: qmp-qemu-socket [--image]`: `--image` saves
GAME WIN sequence PNGs only, ordinary gameplay captures are not saved, every
application start removes `/home/charlie/tmp/images`, and `--image` recreates it.
`--help` returns without starting the application. No GUI launch, installation,
guest connection or live trial was performed. The runtime observation above
does not establish an installed executable or the identity of a future process.

The preservation comparison records automatic changes only in
`.trigger-tree/history.jsonl` and its active session JSON. Those operational
files were left untouched and remain outside review/promotion paths. No index,
console, xSAR, original fixture or unrelated source was changed by this correction.

`final-audit.json` records the final documentation/preservation audit: all ten
documents passed whitespace, fence and final-newline checks; all 111 relative
links resolved. All three HEADs and Git indices, both sibling file trees, the
verified compiler/fixture inputs and the release checksum remained unchanged.
The two historical-banner edits are included in this final documentation scope.

## Live boundary and operator handoff

Use [Live-Test-2.5.103.md](Live-Test-2.5.103.md) only with the final verified
artifact. The normal workflow keeps the bundled profile and automatic path/ID,
then explicitly confirms acquisition, waits for complete survey/restoration,
calculates from that receipt, separately confirms execution and runs once.
No manual pathname transfer is needed after successful acquisition.

One-tableau execution remains the M10 boundary. A daily counter of two boards
does not authorise a full challenge, change recycle budgets or prove terminal
compatibility. M11 remains a separate live completion/acceptance and future-web
handoff milestone. STOP or a refusal ends authority; inspect and reacquire
deliberately without replaying uncertain input or applying cleanup clicks.

## Explicit current correction paths

The following complete path list describes this correction, including regression
and fixture edits already present when the in-progress snapshot was captured:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/Verification-2.5.103.md
docs/Live-Test-2.5.103.md
docs/pyramid-recon-profile.md
docs/pyramid-reconnaissance.md
docs/pyramid-recon-live-test.md
docs/Pyramid-Implementation-Schedule.md
docs/Codex-Current-Status.md
docs/Pyramid-Evidence-Index.md
src/app.rs
src/app/pyramid_defaults.rs
src/pyramid_recon.rs
src/pyramid_recon/vision.rs
src/pyramid_recon/vision/native_v6_tests.rs
src/pyramid_recon/vision/native_v7_tests.rs
src/pyramid_recon/vision/route_tests.rs
src/pyramid_recon/vision/daily_challenge_tests.rs
src/worker/pyramid_reconnaissance.rs
tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/baseline.png
tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/INCOMPLETE.json
tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/README.md
```

Previously dirty M10 generic search/execution, pure Pyramid adapters, computed
controller/wire tests, native-v7 repair and historical verification/live records
remain separate. Their full review lists are retained in
[Verification-2.5.102.md](Verification-2.5.102.md) and the
[xSAR record](../../xsar/docs/Verification-0.5.101.md). Overlapping files must be
reviewed against the appropriate saved bytes, not attributed wholly to this
correction. This list grants no staging, commit, push or publication authority.
Local AGENTS.md, telemetry and private work directories stay outside promotion.

The ten documentation paths in this list are the complete current documentation
review set. The two older reconnaissance documents received historical banners
linking the current contract and live brief; their dated evidence is retained.
Historical `Verification-2.5.102.md` and `Live-Test-2.5.102.md` are unchanged.

Suggested commit title, if Charlie later authorises promotion:

```text
fix(pyramid): support unavailable Solver and default evidence fields
```

Suggested body:

```text
Recognise supported Pyramid boards independently of Solver toolbar pixels while
retaining explicit foreground, no-interference and Solver-not-engaged consent.
Validate and discard legacy schema-1 solver_off data, preserve exact recognition
guards, and retain the original zero-input Daily Challenge refusal as evidence.

Bundle exact native-v7 calibration, generate bounded unused-looking evidence
names, preserve independent manual overrides and hand successful export paths to
route calculation automatically. Keep consent unchecked and receipt authority
unchanged.

Verify app 2.5.103 with 929 passing tests, locked check, strict private rustdoc,
Clippy, scoped formatting, native release and headless help. Preserve the first
Clippy failure and complete successful rerun. xSAR 0.5.101 and console 0.8.4 are
unchanged. M10 Beast live verification remains outstanding; M11 remains Planned.
```

## Code Analysis

Removing the Solver visual gate deliberately transfers that condition to explicit
operator confirmation. A supported scene and exact cards remain necessary but
cannot prove focus, no interference or Solver state. Bundled/file selection and
automatic names grant no input authority; private same-session restored receipts,
exact JSON bytes, per-gesture evidence, STOP/context and transport checks remain
separate. No Solver-activation operation is introduced.

Automatic naming uses bounded metadata probes and checked counters without
creating destinations. Publication still uses the existing private atomic
no-overwrite path. Permissions, filesystem races or unresponsive storage can
refuse or block an operation; probe counts and byte limits are not wall-clock
deadlines. Manual names remain literal and are never silently rewritten.

Native pixel access remains checked, identities retain original fixed slots,
unknown and empty remain distinct, and no unsafe memory operation is introduced.
Current action/gesture/capture/deadline limits, independent replay, completed-pair
verification and release-only recovery remain unchanged. Selected-card rendering
and actual daily rules may still refuse a live run; no fallback, additional
input, assumed recycle allowance or whole-challenge success follows from the
new offline fixture.
