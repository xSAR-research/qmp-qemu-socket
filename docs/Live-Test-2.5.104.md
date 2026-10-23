# Live Test Brief

Date: 19 October 2026. Repo: qmp-qemu-socket, local `main` with existing dirty
M10/2.5.103 work plus the scoped native-v8 correction. Application **2.5.104 is
host verified** after all required checks passed. xSAR 0.5.101 and accepted
console 0.8.4 remain unchanged. Project kind: desktop GUI / other, using the
compiler-selected native artifact, with no PATH installation.

**M01–M09 accepted; M10 Awaiting verification; M11 Planned.** The prior 2.5.103
run acquired/restored and calculated its deal, verified ten actions, then refused
before action 11's second gesture. Its plan is consumed. This brief does not
claim corrected-build live success. Completed host checks, exact native
release identity and scope are in
[Verification-2.5.104.md](Verification-2.5.104.md).

Thirteen focused v8 regressions passed, but the first full suite reported
941 passed/one failed/zero ignored on an obsolete generic-error assertion.
The exact scene-index/bounds assertion is corrected without production changes;
the complete required rerun passed **942 tests, zero failed/ignored**, locked
check, strict private rustdoc, Clippy, scoped formatting, release and headless help.
The first failure is retained separately and earns no separate build increment.
All three indices, frozen siblings and unrelated source/evidence were preserved;
automatic `.trigger-tree` telemetry was classified and left untouched.

## 1. What to test

- Prepare a fresh untouched supported Pyramid board at native 1920×1080, 100%
  display/text scale and CLASSIC BLUE. Do not resume the interrupted route.
  Keep Solitaire foreground and Solver not engaged; a legitimate unavailable
  Solver remains allowed.
- Keep **Use bundled native v8 profile (recommended)** and automatic **Next
  evidence JSON path** / **Next Deal ID**. Explicitly confirm acquisition and
  wait for full stock survey, original-board restoration and successful export.
- Confirm that the completed export path is handed to Calculate. The next
  acquisition fields may already have rotated; they are not the completed deal
  selected for route calculation. Leave the restored guest board unchanged.
- Calculate in Rust, inspect the independently replayed route report, then
  separately confirm unchanged foreground board/no interference/Solver not
  engaged and Execute once. Calculation itself makes no guest connection/input.
- Observe MATCH's intermediate selection and later removals. The new static
  scene patches must tolerate legitimate chart changes while exact cards,
  positive-empty/removed slots, modal refusal and all input guards still apply.
  On mismatch retain the exact scene index/bounds and first evidence before
  another capture. No successful acknowledgement alone proves a logical action.

## 2. Configuration

### Changes in this correction

The bundled default is now `ReconProfileSource::BundledNativeV8`, profile ID
`pyramid-native-1920-classic-blue-v8`, 793,900 bytes, SHA-256
`0b459aa78701da2f07f6cf911ce6f78f7190b1432b66706f4f0135ed21f48a3e`.
Only the ID and scene samples 0/1 differ from v7: static right-border rectangles
`[1796,116,109,18]` and `[1796,388,109,20]`, complete masks and tolerance 4.
The arrow, card readers/caps, modal signatures, scaling and geometry are unchanged.
The [fixture record](../tests/fixtures/pyramid-recon/native-v8/README.md)
identifies the evidence without turning it into a current receipt.

### Live setup and overrides

Select the existing reachable QMP controller, Pyramid and Shortest Path. Normal
setup requires no profile typing or manual pathname transfer. Bundled bytes use
the strict parser without a runtime fixture-file lookup. A deliberately selected
custom file remains literal; an old v7 custom file still contains the old chart
anchors and does not receive the bundled-v8 correction automatically.

Automatic output/Deal ID generation creates or reserves nothing. An existing
eligible runtime `$HOME/tmp` is preferred, then the runtime temporary directory;
worker atomic no-overwrite publication remains authoritative. Naming failures
remain visible and independent manual overrides are preserved. Use an unused
output path in an existing writable directory if manual setup is needed.

Both consent checkboxes start unchecked. Generated names and a suggested route
path grant no authority. Calculate requires successful current-context acquisition
and its private restored receipt; Execute separately consumes the retained plan.
Moving the completed JSON is allowed if exact bytes remain unchanged; editing
or reserialising it invalidates the receipt match. A historical fixture cannot
replace a fresh session receipt.

### Daily-challenge boundary

The earlier daily counter `Boards cleared 0/2` / `1 deal left` does not define
stock/recycle rules or authorise more than one tableau. The previous successful
survey/restoration is evidence for that prior run; it does not prove every daily
scene, current board or corrected session. Any missing advancement, inventory,
enabled Undo All, specific confirmation or restoration evidence must refuse.
The app sends no next-board/terminal input; automatic redeal and full-challenge
completion require separate evidence and acceptance.

## 3. Local install and executable identity

Local install: skipped (desktop application, not a PATH system utility).
Required release build, headless help and checksum passed with Rust
`1.101.0-nightly (8006091eb 2026-10-17)`, Cargo
`1.101.0-nightly (3960fa69b 2026-10-12)` and native target
`x86_64-unknown-linux-gnu`. Use this compiler-selected executable:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
SHA-256: fc719c5f1e9b6e0dd7cee8e369b05d741048f37546f81b76764b8f566a30c25e
```

At `2026-10-18T21:25:43.496975+00:00`, read-only preflight observed the still-running
older **PID 667842**, executable
`/home/charlie/repo/RUST/qmp-qemu-socket/target/release/qmp-qemu-socket`, command
`target/release/qmp-qemu-socket --image`, SHA-256
`42c14a670be3e613c9da893f75f49687d47dd011a7686f68bd69f0c6631d4333`.
It differs from the verified 2.5.104 native artifact; rebuilding did not replace
its process or retained preview. The same observation found zero image files/bytes.
Record actual later process/executable state before testing, preserve the old log
and retained preview, then deliberately close the old application through its UI
before launching the new candidate. Avoid two controllers operating together.
No GUI launch, guest input, installation or process stop was performed by this
correction workflow. The observation is not a guarantee of later image contents.

Startup removes `/home/charlie/tmp/images`, even without `--image`; preserve any
existing evidence there before a later launch. `--image` saves GAME WIN sequence
PNGs only. Ordinary computed-route observations remain memory-only diagnostic
frames. Keep the complete session log and first unmatched preview. **Capture
PNG requests a fresh frame; it does not save the exact retained mismatch.**

## 4. CLI options

CLI table: n/a (desktop GUI, not a host system utility). The existing `--image`
launch flag is unchanged; Calculate and Execute remain GUI controls. Current
headless `--help` verification passed; no new CLI/config field is introduced.

## 5. Suggested live sequence — host checks passed, operator trial pending

1. Confirm the current verified artifact/checksum and actual process situation,
   preserve the older application's session log/retained preview and any current
   startup image evidence. Deliberately close that old application through its
   UI, prepare a fresh untouched supported guest board, then launch this exact
   verified native candidate when ready:

   ```bash
   /home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket --image
   ```

2. Select QMP controller, Pyramid and Shortest Path; keep bundled v8 and the
   supplied next evidence fields. Put Solitaire foreground, ensure Solver is
   not engaged, tick acquisition consent and select **Start reconnaissance →
   JSON**. Wait for explicit complete inventory, restoration and export. Supply
   no guest input during the run.
3. Keep the restored board unchanged. Check the completed export path in the
   computed-route panel, Calculate and inspect the independent replay report.
4. Restore Solitaire foreground, explicitly confirm no interference/unchanged
   board/Solver not engaged, then select **Execute computed route** once.
   Observe without guest input. Retain exact action/gesture/report counts.
5. On STOP, mismatch or uncertain delivery, retain the first report and preview
   before further capture. Inspect the board and obtain a fresh successful
   acquisition before another execution. No resume, uncertain-input replay or
   cleanup clicks follow from the new profile.

## 6. Special checks and acceptance

| Risk or check | Symptom / evidence | Response |
| --- | --- | --- |
| Wrong executable or old running process | UI/version/source provenance remains 2.5.103 or bundled v7 | Record actual process/executable and use only the verified candidate; a build alone is insufficient. |
| Custom old profile | Old K-chart mismatch remains | Deliberately select bundled v8 or exact approved v8 custom bytes; custom files are not silently rewritten. |
| Wrong output assumption | Next acquisition path is absent after successful export | Use the completed export already handed to Calculate; next automatic fields are separate future destinations. |
| Filesystem permission/collision | Export refuses or naming reports an error | Preserve first diagnostics; inspect directory/destination and choose a new literal unused output if needed. Publication never overwrites. |
| Scene or modal mismatch | Exact `scene[index]` native bounds or complete Undo All dialog report | Stop and retain the first evidence. Do not relax thresholds or add input as a workaround. |
| Partial MATCH / STOP / uncertain delivery | Only one selection acknowledged or effect not verified | No further input/replay/cleanup; retain counts and reacquire deliberately. |
| Final result | One positively verified tableau clear | Report separately from automatic redeal, daily objective or regular full GAME WIN. |

M10 acceptance requires observed Beast behaviour and Charlie's acceptance in
addition to host checks. M11 remains Planned. This brief grants no automatic
Codex GUI launch, guest operation, installation, commit, push or publication.
