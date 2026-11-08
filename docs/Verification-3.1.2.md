# Verification — app 3.1.2 / xSAR 0.11.1

Free Cell playback now uses one guarded source-to-destination drag per recorded transfer. The source movement is acknowledged before button-down; held movement and release are sent before collecting their replies. Interrupted or uncertain delivery permits release-only recovery and never replays the transfer. Two fresh semantic endpoint observations, rather than delivery acknowledgements, advance the cursor. CALCULATE remains offline; autoplay policy, Single Step scope and permanent revocation rules remain unchanged.

All A* games use the same activity header, game selector, STOP/EXIT and preview shell. GAME TYPE lists Klondike, Spider, Free Cell, Pyramid and TriPeaks in that order. Unimplemented A* games show an explicit unavailable message and selecting them starts no capture or input. The physical-route inspector opens maximised with LOG-style chrome, Restore and local 10–24 pt text. It borrows the accepted immutable report; source changes/new CALCULATE close it. Failures automatically open FAILURE DETAILS with frozen worker action, phase, cursor, observation, actual candidate refusal and diagnostic paths. Historical first rejection is distinguished from terminal evidence, and persistence/retirement problems do not erase the original cause.

All 38 source commands passed: dependency-free defaults and eleven isolated xSAR features, both owning-package six-gate suites and console check/tests. Full tests: **323 xSAR / 1,789 application / 80 console**, zero failed/ignored/filtered. Fourteen focused application tests include local TCPv3/Unix-QMP peers, immutable failure presentation and actual headless Restore clicks at wide/narrow workspace sizes. Seven aligned commands passed after exactly five approved Cargo version edits; **1,013 of 1,018** compiler inputs stayed identical. xSAR is **0.11.1**, application **3.1.2**, console package unchanged.

## Evidence and lineage

The evidence directory is `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42`. [Source command plan](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/commands.json), [qualification report](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/qualification-paired.json), [source seal](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/qualification-inputs.json), [source patch](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/source-diff.patch), [aligned report](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/aligned-qualification.json), [aligned inputs](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/aligned-inputs.json) and [approval](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/version-approval.json) bind the actual source, flags, logs, configuration and toolchain. The app source-gate result is 1,789/0/0/0/0; library323 and console80 likewise have zero failed/ignored/measured/filtered. Required ignored tests were included. No unchanged suite was rerun merely to change version labels.

Source seal SHA256 `ff1de40cf3ca101f9a45a953e2ff182c2aac1d8bd2e050694e5bb4383651e041`; patch `60298bfea5a7ac336ff90c58a76fc3fa6a54e452a9fc8105fc1405b4ec20e61d`. Aligned seal SHA256 `e9187f965cbcb539caaf41ceeee01edd3d3c3f4a9914217068d214d801511055`. Only the two owning Cargo manifest/lock pairs and the console's xSAR lock entry changed after source qualification. Console's own version is unchanged.

## Delivery

Normal mutable Cargo artifact: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, SHA256 `754f681c88fcd8a790bd5627297b07bc9112d6d03e46680c1bc3113274a136e9`, 16524728 bytes, mode `0o755`. Cargo identifies package3.1.2; direct `--help` passed without changing the image directory. No protected secondary copy was created.

```sh
cargo run --release
```

To disable GAME WIN recording:

```sh
cargo run --release -- --no-image
```

Startup resets `/home/charlie/tmp/images` in either mode. Recording concerns GAME WIN only; failure diagnostics remain independent. Help/invalid CLI arguments return before startup side effects.

## Preservation and acceptance

The finalizer guards 1,018 aligned compiler inputs, all155 existing document preimages and 3,777 prior evidence pins, except the exact previously recorded memory-store delta. Its lifecycle audit pins the unchanged current store plus exactly ten recorded startup receipts and literal sidecar absences; it claims no logical database equivalence. Git histories/indices and unowned tracked patch/status are guarded across all four repositories, including independent server staged/unstaged work. Unrecorded pre-existing untracked file contents are outside these byte-preservation claims. Restricted editable task documents are hash-only. The prior311 failed-sentinel/reconciliation history is retained without rerunning or relabelling it.

There are16 changed/new Rust owners, five aligned Cargo paths and11 release documents (32 owned paths). The complete list is [owned-paths.txt](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/owned-paths.txt). Current document seal has156 members, including this new verification. Prior release narratives remain historical.

Host/retained-frame/local-peer evidence does not establish fresh guest drag effects, autoplay timing or full-route acceptance. Native support remains 1920×1080 CLASSIC BLUE with Solver off and complete supported geometry; toolbar-clipped ten-card columns may refuse. A mismatched or uncertain position stops playback and requires CALCULATE again. No guest actions, installation, executable copy, commit or push were performed for this release.

Operator acceptance: use the original calculated deal, Solver off, matching frozen autoplay policy. Check explicit-target first drag and observed endpoint, repeated Single Step, SOLVE, safe PAUSE/CONTINUE and STOP/uncertainty without replay. Inspect a forced refusal's automatically opened FAILURE DETAILS and retained first/latest evidence. Verify all five A* selections keep the shell, unavailable games have disabled capabilities, and the route inspector opens/restores/resizes with font control. Host tests include headless wide/narrow Restore, not a fresh desktop/UI or guest acceptance session.

[GitHub publication instructions](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/github-push-instructions.md) are preparation only; no commit/tag/push is performed.

## Code Analysis

Coordinates are checked before transport, including mapped aliases. The source-movement ACK fences down; release is queued before waiting on held-input responses. Guard loss or uncertainty never retries the transfer; release-only recovery can change physical state, so the candidate is retired. Private immutable plans remain separate from mutable verified physical cursors and QMP delivery counts. Observation loops are bounded; existing pause, causal STOP and ownership gates remain active. Diagnostics reuse acquired bounded first/latest frames, freeze per-rejection stage, preserve the primary cause over retirement/persistence failures and grant no input authority. Shared UI routing exposes availability without widening Pyramid/CV authority. Remaining risks are the explicitly unverified native drag/autoplay behaviour, supported visual-layout limits and normal Cargo artifact mutability.
