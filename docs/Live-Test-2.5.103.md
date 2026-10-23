# Live Test Brief

Date: 19 October 2026. Application: qmp-qemu-socket **2.5.103 host verified**;
xSAR 0.5.101 and frozen console 0.8.4 are unchanged. Required host verification
passed with 929 tests, zero failed/ignored, plus locked check, strict private
docs, Clippy, scoped formatting, native release and headless help. Exact evidence
and artifact identity are recorded in
[Verification-2.5.103.md](Verification-2.5.103.md).

M01–M09 remain accepted, **M10 Awaiting verification and M11 Planned**. The
retained daily-challenge attempt refused on its first capture without input.
This brief records no new live acquisition, computed route or full challenge.

## 1. What to test

- Start with an untouched supported Pyramid board at native 1920×1080, 100%
  display/text scaling and CLASSIC BLUE. Solver must not be engaged; a legitimate
  grey unavailable control is allowed. Solitaire must be foreground.
- Keep the bundled native-v7 profile and supplied **Next evidence JSON path** /
  **Next Deal ID**. Confirm
  the acquisition checkbox explicitly, then start reconnaissance. The app must
  recognise actual cards, complete the stock inventory and verify restoration
  before publishing a complete deal and retaining its session receipt.
- After success, check that the exact completed export path appears in the
  computed-route field. The next acquisition's untouched automatic output/ID may
  already have rotated; Calculate must continue to use the completed export.
- Select **Calculate route** and inspect its independent-replay success report.
  Calculation makes no guest connection or input. Then separately confirm the
  unchanged foreground board/no interference/Solver-not-engaged condition and
  select **Execute computed route** once.
- Watch the first MATCH. Selected-card tint remains unverified; a recognition
  refusal must retain its first diagnostics and stop without replaying a click.
  Retain the exact final report and attempted/acknowledged counts.

## 2. Configuration

### Normal setup — no manual profile or pathname transfer

Select the existing reachable QMP controller, Pyramid and Shortest Path. Keep
**Use bundled native v7 profile (recommended)** selected. The embedded profile
is exactly the retained native-v7 calibration, SHA-256
`9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`.
It needs no runtime fixture-file lookup. The custom-file option remains available
when deliberately needed.

The app supplies an unused-looking output pathname and Deal ID using an existing
eligible runtime `$HOME/tmp`, falling back to the runtime temporary directory.
Names contain UTC epoch milliseconds, PID and a checked session serial. They
create or reserve nothing; private no-overwrite publication still decides whether
the actual destination can be used. If naming fails, inspect the visible error
and enter a literal unused path in an existing writable directory plus a valid
Deal ID. Both fields remain editable, and manual overrides are preserved
independently. Do not reuse an existing export destination for a new acquisition.

Both consent checkboxes start unchecked. Bundled profile selection, generated
fields and a suggested route path do not authorise input. Calculate remains
disabled until a successful acquisition in the current context. A successful
export supplies its exact path automatically; moving that file later is allowed
if its bytes remain unchanged. Editing/reserialising it invalidates the receipt match.

### Daily-challenge evidence questions

The [retained daily original](../tests/fixtures/pyramid-recon/daily-challenge-solver-unavailable/README.md)
shows `Boards cleared 0/2`, `1 deal left`, LEFT King of clubs and an empty RIGHT
pile. Those labels do not establish unseen inventory, recycle allowances or
Undo All/restoration support. If a fresh live survey cannot prove advancement,
inventory, enabled Undo All, the specific confirmation or original-board
restoration, retain its first failure and stop. Do not alter the reference rules,
add presses or infer authority from the challenge caption.

One verified tableau clear is the execution boundary. The app sends no next-board
or terminal input. Automatic redeal, actual daily terminal behaviour and whole
challenge completion need separate evidence; they are not implied by accepting
an unavailable Solver icon.

## 3. Local install and executable identity

Local install: skipped. This is a desktop application using its compiler-selected
project release artifact, not a PATH system utility. The expected target path is:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
```

The compiler-selected artifact passed headless `--help`; its SHA-256 is:

```text
1b4a82cb55c2ca8415a49501e93f2286597a0e1e67c7db27d71b101eeb4ce733
```

At 2026-10-18T19:03:05.063955+00:00 (19 October in Brisbane), a read-only
preflight found no application process and zero files/bytes in the images
directory; it opened no QMP connection. This is an observation, not installation
or live acceptance. A rebuilt executable does not replace an existing process.
Launch and record the actual verified process for this session; it must acquire
its own restored deal before calculation/execution.

Application startup clears `/home/charlie/tmp/images` even without `--image`.
Preserve any existing evidence there before a later launch. `--image` saves GAME
WIN sequence PNGs only; ordinary route observations remain memory-only diagnostic
pixels. Preserve the complete session log and first mismatching preview.

## 4. CLI options

The existing launch flag is `--image`; Calculate and Execute remain GUI controls.
No new CLI option or PATH installation is introduced. The verified headless help
confirms `Usage: qmp-qemu-socket [--image]` and returns without starting the app.

## 5. Suggested live sequence

1. Preserve existing image evidence, prepare the untouched supported board and
   use the final verified release artifact when ready to operate the live guest:

   ```bash
   /home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket --image
   ```

2. Select the QMP controller, Pyramid and Shortest Path. Leave bundled v7 and the
   supplied **Next evidence JSON path** / **Next Deal ID** in place. Put Solitaire foreground, ensure Solver
   is not engaged, tick the untouched-board/no-interference consent and select
   **Start reconnaissance → JSON**. Wait for explicit complete inventory,
   restoration and successful export; do not supply guest input during the run.
3. Keep that restored board unchanged. Check the completed export path already in
   the computed-route panel, select **Calculate route** and inspect its report.
4. Put Solitaire foreground again, tick the separate restored-board/no-interference
   confirmation with Solver not engaged, and select **Execute computed route**
   once. Observe without supplying guest input.
5. Retain the exact result and diagnostic log. On refusal or STOP, preserve the
   first unmatched preview and delivery counts before further captures. Inspect
   the actual board and reacquire deliberately before another execution; there
   is no resume, automatic replay or cleanup input.

## 6. Special checks and acceptance

| Check | Required evidence or response |
| --- | --- |
| Fresh session and exact executable | Record actual running 2.5.103 process and final verified artifact identity; historical JSON alone grants no authority. |
| Default source and overrides | Bundled source is visible; no typing is required normally. Deliberate manual output/ID overrides remain literal and independent. |
| Successful path handover | Route field is the completed export, even after next-acquisition automatic fields rotate. Both consent steps remain explicit. |
| Daily stock and restoration | Observe actual advancement and complete inventory; prove Undo All/confirmation and two restored observations. An unavailable control or `1 deal left` cannot replace this evidence. |
| First MATCH and uncertain delivery | Retain first mismatch/counts; no second selection after STOP and no retry of an uncertain gesture. |
| Final result | Report one tableau clear separately from automatic redeal, a daily two-board objective or a regular three-board GAME WIN. |

M10 acceptance requires actual guest behaviour and Charlie's acceptance, in
addition to host checks. M11's broader completion and future-web packet remain
Planned. This brief authorises no automatic Codex GUI launch, installation,
commit, push, publication or unobserved whole-challenge claim.
