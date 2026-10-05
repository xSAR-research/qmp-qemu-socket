# v1.3.0, candidate 1 — Free Cell read-only foundation

Base: `146fe173f6e123a92e2eadf518523715d042cd66`, accepted and pushed
v1.2.6 candidate 11. Charlie reported that replaying the Undo All overlap case
and the complete Klondike GAME WIN sequence succeeded before promotion on
4 October 2026. That is user-reported Beast acceptance of those cases; it is
not a claim of exhaustive layout or timing validation.


## Candidate boundary

The game selector now includes **Free Cell**, with an independent profile and
native frame-layout validation. Read-only Capture Frame and Capture PNG use
the existing QMP/capture facilities. The preview reports calibration-only status.
No Free Cell gameplay, Solver activation, auto-finish or terminal input is enabled.
There are no dormant source-click constructors or card-effect comparators.

The UI disables Free Cell gameplay controls. The worker independently refuses
a Free Cell execution request before guest input, so UI state is not the sole
read-only guard. A valid full native frame can be inspected; invalid storage or
dimensions outside the 1920-by-1080 calibration are rejected. Read-only capture
does not enable the guest's Solver.

**Params** includes independent future Free Cell action/automatic-transfer settle
and input-free re-observation values. Defaults are **750/1000 ms**, editable from
0 to 5000 ms. **Actions per Multi-Step** starts at **0**; zero will mean continuous
when execution is enabled, and finite values are limited to 1–10000. These values
do not authorise an operation or establish an observed animation duration.


## Board names and intended next slice

| Area | Meaning | Numbering |
| --- | --- | --- |
| CELL | Four temporary upper-left card slots | 1–4, left to right |
| PLAY | Eight tableau columns with changing source-run geometry | 1–8, left to right |
| SUIT | Four upper-right foundation piles | 1–4, left to right |

Future gameplay follows the guest's Solver rather than an independent rank or
search solver. A fresh supported HALO on a CELL or PLAY source will authorise
one click on that source, including a connected multi-card run. After editable
settle, a new frame supplies the next recommendation. Automatic SUIT transfers
may follow the click or pause for a further HALO; a result need not contain only
one changed card. Source/recipient card matching and changed-pixel thresholds
will not gate that loop. Dashed destination guides supply no source-click authority.

There is no Draw, Recycle or Solve action to inherit. Source selection, the PLAY
boundary above the bottom toolbar, native click geometry and completion policy
remain Free Cell-owned calibration work. Shared QMP, capture, cancellation,
logging and UI facilities remain reusable. Independent one-board win evidence
must establish terminal context before an ordered local-button restart policy
can be enabled. Existing Klondike terminal coordinates are not Free Cell calibration.


## Evidence boundary

The three current Free Cell PNGs supplied on 4 October were available locally
and visually inspected for intake:

| Filename suffix | Observed intake state |
| --- | --- |
| 223950 FREE-CELL.png | Initial board with Solver off |
| 224004 FREE-CELL-SOLVER.png | Solver-on PLAY-to-SUIT recommendation |
| 224026 FREE-CELL-move-to-cell.png | PLAY-to-CELL recommendation |

They establish intake observations rather than runtime detector calibration.
This preparation adds no new screenshot fixture. The dated twelve-frame intake
in [Free Cell Gate 1](freecell-gate-1.md) remains a historical record; those earlier
raw files were not available for reinspection in this cycle. The newly requested
native **FC01–FC13** sequence and optional FC14 toolbar case have not yet arrived.

Charlie captures each requested state using the application **Capture PNG**
and **Save PNG**, attaching the original full 1920-by-1080 PNG rather than a
cropped preview or host screenshot. Full-frame capture remains QMP temporary
PNG → read original bytes → RGBA8 decode → normal cleanup. Manual Save writes
the previewed original bytes without a second screendump. This candidate does
not implement file-free capture or alter that identity contract.

Level-200 card-back offers with No Thanks remain a manual interruption. No
automatic handling of that screen is included.


## Preservation and verification

The accepted Klondike Y<947 live tableau detector, upper source click and
Solver-led action loop remain. Pyramid's highlighted MOVE/Recycle sends D;
card/pile targets retain clicks. TriPeaks keeps bounded input-free observations
before ordinary Solver recovery. Existing terminal policies, STOP, socket/mode
invalidation and uncertain-input non-replay remain distinct by mode.

Meaningful compact regressions cover Free Cell read-only request refusal,
malformed/non-native frames, independent bounded Params values and preservation
of the existing mode boundary. They do not attempt to calibrate unavailable
images or increase test counts for their own sake. Historical Klondike effect
tests remain diagnostic-only. Actual checks, tool versions, warnings and
environmental limits are recorded in the delivered review and validation archive;
no compiler or gameplay result is claimed from a prior candidate's run.

Do not run rustfmt or cargo fmt. Preserve the floating nightly selection,
Clippy/rustfmt components, edition 2024, minimum Rust 1.101.0 and Charlie's
double-blank-line source spacing. The channel name is not a dated nightly pin.

Beast verification is read-only Free Cell capture/status, disabled actions,
future Params editing and existing-mode regressions. Native source calibration,
guest input and timing acceptance are future work after the screenshot intake.


## Installation

The guarded installer pins this exact pushed base, complete affected contents
and modes. Preview changes no repository files. Apply backs up the actual
before-state and verifies the installed candidate. Unknown affected edits,
affected index changes, new-file collisions or a different HEAD stop before
replacement. Guarded rollback refuses to overwrite later local edits. Unrelated
work, calibration files and the untracked Beast root AGENTS.md remain preserved.
Download integrity, installed source identity and launched executable identity
are separate checks. Charlie builds, tests, commits and pushes after acceptance.


## Code Analysis

- Input surface: read-only capture uses the existing allow-listed QMP client.
  Free Cell execution is refused independently in the worker; no detector,
  gameplay, Solver or terminal action grants new guest-input authority.
- Memory safety: safe Rust and existing RGBA8 layout checks remain. Native frame
  validation rejects invalid storage and dimensions; capture and preview retain
  their existing bounded ownership and latest-frame mailbox.
- Bounds: editable future waits remain 0–5000 ms and finite action budgets
  1–10000, with zero reserved for continuous mode. Those settings cannot enable
  execution. No uncalibrated source region, click point or toolbar edge is used.
- Failures: capture/QMP errors retain their current stop behaviour. Unsupported
  Free Cell scenes publish calibration-only status and cannot fall through to
  another mode's input path. No uncertain non-idempotent operation is replayed.
- Installation: exact-base/content/index checks, bounded archive verification,
  backups and guarded rollback protect local edits. The package checksums
  establish agreement with the supplied files, not independent author identity.
