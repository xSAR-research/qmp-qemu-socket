# Live Test Brief

Date: 19 October 2026. Application: qmp-qemu-socket 2.5.102 with xSAR 0.5.101.
Both repositories remain on `main` with uncommitted work. The application's
accepted 2.4.101 baseline was already dirty; xSAR's 0.4.100 baseline was clean.
Project kind: desktop GUI application and reusable library, rather than a PATH
system utility. The accepted basic-route-planning console 0.8.4 is frozen.

M01–M09 are accepted. M10 is **Awaiting verification** because host checks pass
but automated guest execution has not been exercised. M11 remains Planned.
The [application verification record](Verification-2.5.102.md) and
[xSAR record](../../xsar/docs/Verification-0.5.101.md) retain exact host evidence.

## 1. What to test

- Acquire and restore a fresh supported Pyramid board in this application
  session. Confirm that the successful export populates the computed-route
  pathname. A saved historical JSON alone must not authorise execution.
- Calculate the route and inspect the report. Calculation uses the restored
  acquisition and matching JSON bytes without connecting to or controlling
  the guest. The calculated route must pass independent replay.
- Confirm the unchanged board, Solitaire foreground, Solver OFF and no guest
  interference, then execute once. Check that actions agree with the visible
  game and that the worker positively verifies each complete action's effect.
- Observe the first MATCH carefully: selected-card rendering is unverified.
  A strict recognition refusal must stop the run and retain diagnostic pixels;
  it is not permission to repeat a click or relax recognition.
- Check the reported final result. One verified tableau clear is separate from
  a three-board GAME WIN; the application sends no next-board or terminal input.
  Any separate STOP trial requires inspection and a new restored acquisition
  before another execution; there is no resume.

## 2. Configuration

### Changes in this work

The GUI adds separate Calculate route and Execute computed route controls under
Pyramid Shortest Path. Existing runtime configuration schemas and the native-v7
calibration are unchanged. Search and execution use fixed bounded defaults,
shown in the UI and verification records; zero does not mean unlimited.

### Live configuration review

Use native 1920×1080 CLASSIC BLUE with 100% display/text scaling, an untouched
Pyramid tableau, visible initial LEFT card, empty RIGHT pile and Solver OFF.
For this trial, use the regular Grandmaster Pyramid scene matching the accepted
native baseline: New game, score display, and an available gold Solver key left
inactive. A disabled grey Solver control is a different state. The first live
refusal captured a Medium board with Retry and grey Solver/Hint; all three scene
patches matched, but the mandatory gold inactive-Solver patch did not. That run
captured once and attempted no guest input or output publication; it requires no
restoration due to the failed scan. Keep those diagnostics and prepare the
supported scene before confirming another scan.

Select the existing reachable QMP controller in the GUI. The operator must put
Solitaire in the foreground; a QMP acknowledgement cannot establish focus.

Suggested reconnaissance values were checked locally: profile exists, output
parent exists and the proposed output is absent. They are literal paths without
surrounding quotes or leading whitespace in the GUI fields.

| Field | Value |
| --- | --- |
| Native profile | `/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v7/profile.json` |
| Output JSON | `/home/charlie/tmp/pyramid-recon-20261019-v7-m10-01.json` |
| Deal ID | `pyramid-recon-20261019-v7-m10-01` |

The worker must complete all 52 identities, verified Undo All/confirmation and
two restored observations before publishing and retaining the session receipt.
Keep the board unchanged afterwards. A moved or extensionless export is valid
when its bytes match; editing or reserialising the JSON invalidates the match.

## 3. Local install

Local install: skipped. This desktop application uses the compiler-selected
project release artifact; this work authorises no PATH installation or GUI
launch by Codex. Native release build and the headless `--help` check passed.

Artifact:
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`

SHA-256:
`5bd527f9398df87ebd1f9b10a03d71f5388b6f94135e72960aa3a300c136dda3`

Read-only preflight found no running application process and no files in
`/home/charlie/tmp/images`. This is a checkpoint, not a future process claim.
Application startup clears that image directory even without `--image`.
Preserve any evidence added there after this checkpoint before a later launch.

## 4. CLI options

CLI options table: n/a for this desktop GUI. The verified launch below uses the
existing `--image` flag; Calculate and Execute are GUI controls, not new flags.

## 5. Special checks

| Risk | Symptom | Check and response |
| --- | --- | --- |
| Historical executable or missing session receipt | Old controls, or calculation refuses the selected file | Launch the exact verified artifact and perform a fresh acquisition/restoration in that process. |
| Changed board or focus | Origin mismatch or guest effect is refused | Inspect the retained preview/log. Prepare and reacquire deliberately; do not continue the failed route. |
| Selected-card rendering | Refusal between MATCH selections | Retain the first unmatched preview and log before more captures or input. Record the action/gesture and attempted/acknowledged counts for review. |
| Uncertain delivery or STOP | Partial-action report; candidate revoked | Inspect the board. No click/D replay, cleanup input or resume follows automatically. A later execution needs a new successful acquisition. |
| Unsupported geometry or recognition | Profile, scene, identity or positive-empty refusal | Retain the first diagnostics. Do not infer unseen identities or loosen masks/caps to finish the trial. |
| Path or filesystem | Export/read refusal, or blocked external storage | Use literal regular local files and an existing writable parent. Size limits do not give filesystem operations a hard deadline. |

## Suggested live command sequence

1. Prepare a fresh untouched supported board with Solver OFF. From any directory,
   launch this verified project artifact when ready to operate the live guest:

   ```bash
   /home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket --image
   ```

2. Select the QMP controller, Pyramid and Shortest Path. Use the profile/output/
   deal-ID fields above and the existing reconnaissance controls. Wait for the
   explicit successful inventory/restoration/export result.
3. Keep that exact restored board unchanged. Select **Calculate route** in the
   computed-route panel and inspect its success report.
4. Put Solitaire in the foreground, leave Solver OFF, tick the explicit
   no-interference confirmation and select **Execute computed route**. Observe
   the game without supplying guest input during the run.
5. Retain the exact final report. On any refusal or STOP, retain the first
   unmatched preview and session log before more captures, then report what
   happened. No automatic retry or continuation is authorised by a failure.

## Out of scope / not installed

This brief records no performed live trial, installation, promotion or deployed
version. No remote sync, commit, push, guest input or application launch was
performed by Codex. Acceptance of one tableau does not complete M11's broader
GAME WIN and future-web handoff requirements.
