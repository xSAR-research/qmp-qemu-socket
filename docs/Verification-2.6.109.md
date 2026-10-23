# Verification — app 2.6.109 native-v11 rank-8 correction

Status: **host verified; complete app sequence passed on attempt 0001**.
Updated 19 October 2026. App 2.6.109 earns exactly one correction build,
with no additional minor milestone.
xSAR 0.6.102 and console 0.8.4 are frozen byte-for-byte; no sibling suite rerun is
claimed. M01–M09 accepted; **M10 Awaiting verification; M11 Planned**.

## Approved change and evidence boundary

The approved app-only rank-8 correction appends one exact native RGB sample to
both reader banks and selects native-v11. Rank-8 variants increase 3→4 within
the existing cap 8; XOR 24/8, alignment ±1, paper/ink/colour, uniqueness, geometry,
scene/dialog and positive-empty guards are unchanged. Native-v10 remains intact.
The source refusal retained one capture, zero input attempts and no export;
its diagnostics prove neither a complete deal nor restoration.

Manual/Fast startup, separate Recon/Plan/Solve, Single board sessions, paired
500 ms RECON, final ACK-relative two-second observation and search budgets are
unchanged. Native Continuous remains disabled/unavailable: positive standard
three-board target/progress evidence is still absent. This correction does not
complete the larger goal or authorise Daily Challenge automation. STOP/context,
receipt consumption, independent replay and no uncertain-input replay remain.

Source: `/home/charlie/tmp/.pyramid-recon-745174-1/baseline.png`,
3,751,479 bytes, SHA-256 `28e1127ea9d4663ebcca61061779093ebc7b571ec0ebde13a4973906b9913409`.
Exact rank crop `[792,171,27,24]`: 1,944 unchanged RGB bytes, full 648-pixel mask,
tolerance zero, raw SHA-256
`fa288ca41690ca02f9a3fff2ba1e2dc90e8cedf2123865827e219e0ccd5946a5`.
Profile: `pyramid-native-1920-classic-blue-v11`, 817,701 bytes, SHA-256
`7269769d0c1373edd431e7606f6cdad3ed1239dc5d70839257b5ca89ed408124`.
Only ID and one appended 8 variant in each bank differ from literal native-v10;
removing those additions and restoring the ID reproduces its complete JSON.

Original `/home/charlie/tmp/.pyramid-recon-745174-1/INCOMPLETE.json` reports
one capture, zero D/Undo All/confirmation attempts, zero recorded stock/tableau,
restored false, complete false, publication not_attempted and no delivery
uncertainty. Its exact refusal is:

```text
unknown rank/suit at tableau slot 1; accepted identities=0; ink<128; XOR rank<=24, suit<=8; alignment=1px; unique labels/paper/colour required
```

The private proposal `/home/charlie/tmp/pyramid-eight-review-FxDVLWPw/REVIEW.md` and `measurements.json`
report Python-mirror visible-card recognition 28/29→29/29 in each bank and
104 historical labelled crops retained in both profiles/banks. All 29 historical
source PNG identities and scene/dialog equality were checked. These are review
measurements, not Rust parser/live/restoration acceptance. No additional guest
capture/input was requested for this correction. [Profile provenance](pyramid-recon-profile.md)
and [fixture record](../tests/fixtures/pyramid-recon/native-v11/README.md) preserve
that distinction.

## Required host checks — passed

Work root: `/home/charlie/tmp/pyramid-v11-validation-x_1cfs_r`. `verification.json` is
`passed-required-checks`, complete attempt `runs/0001`; there was no first
compile/test/check failure in this correction. The baseline precedes this
correction but contains earlier dirty M10 work. Literal baseline hashes and
paths define this goal; the accumulated HEAD diff does not.

Actual rustc **1.101.0-nightly (8006091eb 2026-10-17)**, full commit
`8006091eb71ae7d9e9bccb66c6122c9d68af4466`, LLVM 23.1.4; Cargo
**1.101.0-nightly (3960fa69b 2026-10-12)**; native host/target
`x86_64-unknown-linux-gnu`. `commands.json` retains exact argv/cwd/timings/logs.
Cargo used explicit `+nightly --locked --offline` with the native target;
check/Clippy selected all targets/features, full tests all features including
ignored, and strict private docs all features/no dependencies with
`RUSTDOCFLAGS="-D warnings -D missing_docs"`. App release selected package
defaults; the executable was selected from Cargo JSON rather than guessed.

| Check | Actual result |
| --- | --- |
| Locked/offline all-target/all-feature Cargo check | Passed, 5.775 s |
| Full app tests including ignored | **1,052 passed, 0 failed, 0 ignored**, 150.403 s; test report 150.25 s |
| Strict private rustdoc, warnings/missing docs denied | Passed, 1.217 s |
| Clippy, warnings denied | Passed, 5.925 s |
| New-module scoped formatter | 1 module passed, 0.032 s |
| Inherited parent formatter shadows | 10 modules passed, 1.067 s; nonempty lines match |
| Default-feature release/Cargo JSON artifact | Passed, 12.583 s |
| Native executable headless help | Passed, exit 0, 0.004 s |
| All three HEADs/indices/closed compiler and fixture inputs | Passed |
| All frozen xSAR/console files and stable 107/108 artifacts | Passed; no sibling rebuild/test/matrix rerun |
| Unrelated source/calibration/retained evidence | Preserved |
| Final documentation/link/scope audit | Passed: 9 documents, 165 local links and 26 review paths; all three HEADs/indices, frozen siblings and verified artifacts preserved |
| Fresh Beast/operator acceptance | Pending; no agent launch/input/install/promotion |

Native-v11 focused filter passed **9 tests, 0 failed, 0 ignored**, 1,043 filtered;
compile 10.56 s, tests 8.67 s, from `focused-result.json`. These 9 tests overlap
the full 1,052 suite. The 9 new production parser/recognition regressions and
complete full suite establish host behaviour; they do not prove restoration or
guest effects. The private Python review above remains separately attributed.

Standing blank spacing is preserved in formatter-shadow exceptions
`src/app.rs`, `src/pyramid_recon.rs`, `src/pyramid_recon/vision.rs` and
`src/worker/pyramid_reconnaissance.rs`; only blank lines differ. No blanket
formatting occurred. Required preservation also classified the automatic
`.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` changes as
operational telemetry and left them untouched. No unchanged shared suite was
rerun; retained 108 app/xSAR/compatibility counts belong to their historical
verification record and do not qualify this correction.

## Stable candidate — verified host identity

The compiler-selected native executable is `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`,
SHA-256 `0d6dc51bb7a292606ff81ef6c534898aef4ca7129c18a9d070cb7d88727a58ed`.
The stable copy `/home/charlie/tmp/pyramid-v11-validation-x_1cfs_r/release/qmp-qemu-socket-2.6.109` is
**12,685,240 bytes, mode 0555**, matches that native SHA-256 and passed headless
help with exit 0. `stable-artifact.json` records it at
2026-10-19T05:10:21.673114+00:00. The delivered stable 108 remains
`/home/charlie/tmp/pyramid-improvements-2k8a917g/release/qmp-qemu-socket-2.6.108`,
SHA-256 `badd90d6f75d78600066b749d1582a4ec681ab3f89bae8e774d0f23a7f35e573`;
stable 107 is likewise preserved.

Read-only `runtime-preflight.json` observed **no application-controller process**
at 2026-10-19T05:10:21.704548+00:00; `/home/charlie/tmp/images` contained
zero files/bytes at that instant. These facts do not establish a later running
identity, continuing emptiness or an installation. No QMP connection, GUI launch,
guest input, process replacement, installation, staging/commit/push or
publication was performed. The [live brief](Live-Test-2.6.109.md) gives the exact
stable launch and warns that startup clears that image directory. Fresh Beast
acceptance remains pending; native Continuous is still unavailable and the
larger goal remains incomplete.

## Explicit correction review paths

These 26 paths are the complete approved 109 correction delta, **not** the complete
promotion set for accumulated dirty M10 work. No staging/commit/push authority is
implied. `review-paths.json` remains the authoritative literal list.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/daily_challenge_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/post_game/pyramid_independent_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v10_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v11_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v11/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v11/baseline.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v11/INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v11/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v11/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.109.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.109.md
```

Proposed review-only Conventional Commit title:

```text
fix(pyramid): recognise the retained native rank-8 variant
```

Proposed body:

```text
Append one exact native rank-8 RGB crop to both reader banks and select v11.
Preserve v10, XOR 24/8, alignment ±1, full negative-space and authority guards.
Retain the one-capture/zero-input refusal and production recognition regressions.
Record the complete 1,052-test app-only pass and verified stable 109 identity.
Keep xSAR/console and delivered 107/108 artifacts unchanged; live acceptance and
native Continuous remain pending.
```

## Code Analysis

The added sample identifies only its rank; colour/suit/unique-label/scene guards
still determine acceptance. Full648-pixel negative space and exact RGB preserve
refusal of unsupported glyphs; no cap/alignment relaxation is included. Checked
native coordinates, stock/tableau ownership, STOP and receipt gates remain in
the owning modules. Diagnostic JSON is incomplete evidence, not route authority.
No file overwrite, retry/resume, new input, extra capture or broader mode support
is introduced. Host tests and successful release do not prove guest effects.
