# Pyramid reconnaissance live-test brief

> Historical operator brief for the dated 2.2.0 delivery. Its versions,
> dependency/profile selection, manual-field instructions and positive Solver-OFF
> wording are superseded by the [2.5.103 live brief](Live-Test-2.5.103.md) and
> [current profile contract](pyramid-recon-profile.md). Current setup uses bundled
> exact v7, automatic editable evidence names and successful-export handover to
> Calculate. Consent remains explicit, Solver must not be engaged, and an
> unavailable control is allowed. Retain the dated evidence below; use the
> current brief for new live work.

Date: 11 October 2026, Australia/Brisbane.
Project: qmp-qemu-socket, native GUI application, development 2.2.0.
Charlie accepted a complete 2.1.0 live scan and restored schema-1 export.
This brief covers the subsequent schema-2 integration; a prior running process
does not acquire that writer merely because source or its on-disk binary changed.

## 1. What to test

1. Ordinary independent Pyramid Capture remains read-only. The separate
   **Guarded Pyramid reconnaissance → JSON** panel requires its own profile,
   output path, deal ID and untouched/Solver-OFF confirmation.
2. The original 28 tableau identities use seven logical rows, top-to-bottom and
   left-to-right. DRAW ordinal 0 is the initially visible LEFT card before D.
3. A successful scan records 24 unique stock identities in first-draw order,
   proves exhaustion twice, recognises Undo All and its specific modal, then
   verifies the original tableau/LEFT/empty RIGHT twice before publishing JSON.
4. Every successful intermediate capture refreshes the advisory preview.
   The log distinguishes recognition, input acknowledgements and restoration.
5. STOP prevents further input, including automatic restoration. A failed or
   stopped run preserves explicitly incomplete diagnostics, not route input.
6. Recheck ordinary CV operation and STOP on all five game types. No controller
   policy or Spider XP fix belongs to this slice.

## 2. Configuration

Use the existing QMP socket and native 1920x1080 guest display, 100% display/text
scaling and CLASSIC BLUE. The app neither launches QEMU nor creates its listener.
Do not operate the guest manually during a scan or run another controller against
the same guest. Before the successful-run test, manually establish an untouched
Pyramid board with Solver off, 28 tableau cards, visible LEFT and empty RIGHT.

The supplied candidate profile is
`tests/fixtures/pyramid-recon/native-profile.json`. Select it explicitly in the
panel; there is no implicit or fabricated fallback. Paths are literal, with no
tilde or environment expansion. The output must name a new file in an existing,
trusted writable directory. Use a fresh deal ID and output filename per attempt.
Do not change the profile's recognition tolerances merely to bypass a refusal.

The model/codec/publisher come from xSAR 0.4.0 at promoted Git revision
`ab226a65d3bbc2b4914a5cecfb2d3247364c7536`, recorded in the application manifest
and lockfile. Normal builds require no paired-development source override.

## 3. Local install and executable identity

Local installation is skipped: this GUI development build must not silently
replace the accepted application or an already-running process. No live launch
or guest input is performed by the implementation checks. Use the exact release
executable and SHA-256 recorded in the validation evidence, close the old
controller normally, then launch the test executable deliberately.

Verify the window/session version, executable path and `/proc/<pid>/exe`
checksum before testing. Updating source or a file on disk does not update an
existing process. There are no reconnaissance CLI options; use the GUI panel.

The following scripts are retained in the validation evidence directory. They
have been syntax-checked, not executed against the live application. They run
as separate zsh processes, so `exit`/`exec` cannot replace the interactive shell;
see [zsh command execution](https://zsh.sourceforge.io/Doc/Release/Shell-Grammar.html).
The launcher refuses an existing controller and mismatched executable/profile
hashes. It does not install anything or start reconnaissance automatically.

1. Close the old controller normally, then deliberately launch the verified build.
   Run as the normal desktop user, without sudo.

```zsh
printf 'Launch the verified Pyramid reconnaissance test build\n'
/usr/bin/zsh /tmp/pyramid-git-pin.KWcKZAfa/launch-verified.zsh
```

2. In a second terminal, verify the one running process before guest input.

```zsh
printf 'Verify the actual running controller, not just its file on disk\n'
/usr/bin/zsh /tmp/pyramid-git-pin.KWcKZAfa/verify-running.zsh
```

In the panel, use the absolute profile path ending in
`qmp-qemu-socket/tests/fixtures/pyramid-recon/native-profile.json`, a fresh output
such as `/home/charlie/tmp/pyramid-recon-20261011-05.json`, and deal ID
`pyramid-recon-20261011-05`. Choose an unused filename; this example is not a
reservation. These fields do not create output until the guarded operation
succeeds, and the application refuses an existing destination.

## 4. Successful scan

Use the corrected native profile `pyramid-native-1920-classic-blue-v4` described
in the [stable-text correction record](pyramid-recon-stable-text-correction.md), retaining
the earlier [chart correction](pyramid-recon-gate3-correction.md). The same
profile pathname is loaded anew before each reconnaissance operation; no restart
is needed solely for this data correction. After editing any form field, tick
the untouched-board confirmation again. Failed-attempt diagnostic directories
are not completed deal output, and must be retained rather than overwritten.

A run that already failed at the Undo All modal cannot resume. After retaining
the evidence, Charlie may manually confirm that specific Undo All dialog once
to restore the board. Verify the original tableau, initial LEFT and empty RIGHT,
then begin a new scan with fresh consent and output path. Manual restoration
does not retrospectively qualify the failed diagnostic as completed route input.
Leave the optional dialog-suppression checkbox unchecked. On the retained deal,
the initial LEFT card is King of diamonds; RIGHT must be empty and Solver off.

1. Retain an original initial PNG and record its tableau plus initial LEFT card.
   Select Shortest Path/Pyramid, wait for idle and close any PNG review dialog.
2. Enter the native profile path, a new JSON output path and bounded deal ID.
   Confirm the actual untouched Solver-off board and start reconnaissance once.
3. Watch the native preview and ordinal messages. Do not press D or click cards.
   Expect at most 24 D attempts, one Undo All and one specific OK. The optional
   modal checkbox and Cancel button are not targets.
4. Confirm the guest visibly returns to the same tableau, initial LEFT and empty
   RIGHT. Confirm the final log reports restoration and the exact published path.
5. Retain the JSON, complete session log and adjacent private diagnostic directory.
   Compare every exported tableau identity with the initial image and stock
   ordinals with acquisition messages; check 28+24 unique standard-deck identities.
   The shared writer validates the completed model; the console reader is deferred.

Only this actual observed run can establish live acquisition/restoration. The
three supplied native screenshots establish endpoint/modal appearances, not the
22 intervening stock identities or a successful automated restoration sequence.

## 5. Refusal and STOP checks

1. Choose an existing output file: refusal must happen before guest input and
   leave its bytes untouched. Missing/invalid profiles must also refuse before input.
2. On a separate manually prepared untouched board, start with a fresh output
   path, then request STOP during stock acquisition. Wait for the worker to finish;
   an in-flight QMP operation may already have been delivered.
3. Confirm no later D, Undo All or OK was attempted after cancellation was
   observed. No complete JSON should be reported for an interrupted acquisition.
   Inspect the retained incomplete inventory and failure PNG before any recovery.
4. Restore the guest manually if necessary. Never blindly retry uncertain input
   or assume STOP restores the board. A new operation needs fresh confirmation.

A STOP arriving after successful publication cannot retract the exported file;
the UI reports the completed export honestly without granting further authority.
Publication errors distinguish absent, uncertain and already-published outcomes.
Inspect the destination before any new operation; no automatic replacement occurs.

## 6. Special checks

| Risk | Symptom | Check and response |
| --- | --- | --- |
| Wrong executable | Old version or no reconnaissance panel | Verify the launched process path and checksum; close/relaunch deliberately. |
| Wrong display/profile | Unknown scene or unreadable card | Retain the original failure PNG and profile; check native geometry/artwork, do not weaken guards. |
| Capture permissions | QMP screendump or PNG read fails | Verify app and QEMU share access to the capture directory beside the socket. |
| Output permissions/space | Preflight or diagnostic persistence failure | Use a trusted writable parent with sufficient space; inspect retained partial diagnostics. |
| Unknown animation/input effect | Scan stops without another D | Preserve first-failure evidence; acknowledgement alone is not advancement. |
| Context/STOP | Scan refuses further input | Wait for completion, inspect the guest, then recover manually if needed. |

New deal exports use schema 2. Check every tableau/stock record for exactly
`card`, integer `value` and `suit`; verify A=1, J=11, Q=12, K=13 and unchanged
rank/suit identity/order. An offline schema-1-to-2 conversion is sufficient to
check representation, but does not constitute a fresh 2.2.0 live scan. Use the
new executable identity from the v2.2.0 report for any deliberate live retry.

No solver/search, redeal, Solver activation, card removal, New Game operation,
console installation or promotion is part of this test. Charlie performs Git
promotion after acceptance. xSAR is promoted and pinned; application validation
and live acceptance retain their separate evidence requirements.

## Code Analysis

The scan authorises only three typed operations under fresh native evidence and
cooperative STOP/context checks. Malformed profiles, unreadable or ambiguous cards,
duplicate identities and uncertain delivery stop without replay. Capture/input/
observation counts are bounded; blocking I/O can outlast the cooperative deadline.
No unsafe Rust or host-wide input is added. Complete JSON uses checked xSAR models
and atomic no-overwrite publication; diagnostics remain separate and incomplete.
