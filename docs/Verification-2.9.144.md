# Verification — 2.9.144 / xSAR 0.9.0

Host qualification passed: **1434 full application tests**, zero failed/ignored/filtered, **729 sealed inputs**, all six required gates and native release/help. Fresh corrected guest behaviour and Charlie acceptance remain pending.

The changed runtime logic is confined to the temporal stock-position layout alternative in [redeal.rs](../src/pyramid_recon/vision/redeal.rs). The ordinary layout, native/modal/static anchors, full tableau identities and absence, empty LEFT and exact RIGHT identity remain required. The caller still derives appearance from this execution's verified pre-input region and owns legal exhaustion, freshness, STOP/context, probe and one-click delivery. The change removes an unintended RIGHT-empty prerequisite; it does not authorise a click from a boolean supplied by an operator or from an icon template.

## Retained failure evidence

The private archive is `/home/charlie/Documents/Pyramid/Validation/card-final-outcome-143-20261025/repeat-run-9`, including all seven original files, checksums and `inspection.json`. Run 9 independently verified 39 partial actions / 52 gestures, then attempted 15 legal DRAW operations and stopped with 67 gestures acknowledged. The last frame has 12 remaining tableau cards, LEFT X, RIGHT 3♦ and Sixes 8/10. The old LEFT-X RGB comparison fails; its paper-free inset also fails the old continuity limit. Compact MOVE evidence is absent. A RIGHT card then vetoed the temporal fallback regardless of measured appearance.

The saved `first_unmatched` slot 27 reason came from action 1 and recovered. It is not the established final blocker. Its region-to-last difference is 1353/3024 at delta 20, measured only as a diagnostic: that first-unmatched frame is not the original retained pre-input baseline, so this does not reconstruct the runtime baseline. The original consumed route is unavailable and is not invented.

## Regression scope

Both `native_temporal_right` regressions failed against unchanged 143 in `focused-before-fix.log` and pass in `focused-after-fix.log` after the correction. The actual-image test proves all 12 remaining original identities, 16 absences, empty LEFT and RIGHT 3♦; wrong cards, a nonempty LEFT, pale remnants and lost scene evidence refuse.

The native adapter test uses the actual textured LEFT/control crop with an older tableau, exact RIGHT 8♠ and a rules-derived synthetic exhausted stock state. It has zero remaining D operations. It proves the production exhausted-source observation, one centre click/down/up, one fresh probe and STOP/missing/unchanged-origin refusal; it does not replay run 9's 15 DRAWs or establish fresh guest acceptance. Existing full-suite stock-step tests remain independent coverage.

## Qualification

Full locked check, private Rustdoc with `-D warnings -D missing_docs`, Clippy with `-D warnings`, include-ignored full tests, scoped formatting and native release/help are required. The qualified input seal covers app sources/tests/manifests/config and unchanged xSAR path-dependency inputs. No sealed input may drift during the commands.

xSAR remains 0.9.0. Its previous 296 tests and exact pause-primitive Miri 3 results are retained and checked by source hash; they are not newly run 144 tests. No concurrency, unsafe, dependency, API or timing change is introduced.

## Code Analysis

The correction replaces `compact_move || exhausted || (right.is_none() && redeal_appeared)` with `compact_move || exhausted || redeal_appeared`. This admits only another stock layout proof before the unchanged `verify_cards` call. LEFT must still be replayed empty, and all 28 tableau states plus both exclusive pile states must agree in that same native frame. The region comparison retains its pre-input scope and thresholds. Missing appearance and other layout proofs still refuse; stopped/uncertain input is not replayed. Full checks and the native adapter are host evidence, while background/challenge behaviour remains a fresh manual acceptance task.

The earlier separate run 8 full-clear case remains open: its final saved frame is consistent with next-board dealing, but the retained image alone proves neither settling duration nor successful final confirmation. This patch leaves the two-second window unchanged.

See [manual checks](Live-Test-2.9.144.md). The authorised GUI launch is recorded below. No agent START, guest input, installation, commit or push occurred.

## Recorded qualification

| Gate | Exit | Seconds |
| --- | --- | ---: |
| scoped-format | 0 | 0.164 |
| check | 0 | 11.188 |
| rustdoc | 0 | 2.119 |
| clippy | 0 | 11.542 |
| full-tests | 0 | 435.771 |
| release | 0 | 18.940 |

Native release `--help`: exit 0. All 729 compiler/test inputs remained unchanged throughout the commands.

Protected executable: `/home/charlie/Documents/Pyramid/Validation/pyramid-temporal-right-20261025/native/qmp-qemu-socket-2.9.144`.

SHA-256: `f4e1ebd12ad6c4d09f047a48c9e26b4e2432a01f14c8e6dd8ef8f9f77e730436`. This is a new protected copy; earlier candidates, the protected 143 binary and all failure evidence remain available. The authorised 144 GUI launch is recorded below; no agent START or guest input occurred.

## Launch for Charlie

The protected qualified executable was launched at UTC 2026-10-24T16:53:40.462337+00:00 after Charlie confirmed 143 had exited. PID `1050463` resolved through `/proc/1050463/exe` to the protected 144 path and SHA-256 `f4e1ebd12ad6c4d09f047a48c9e26b4e2432a01f14c8e6dd8ef8f9f77e730436`.

Session log: `/home/charlie/tmp/qmp-qemu-socket-1792860820508.log`. Its first line was `[+0000.045s] qmp-qemu-socket v2.9.144 started; waiting for a solving strategy choice.`. No agent START or guest input occurred. Startup-image preservation found 0 files to retain. Fresh guest and Charlie acceptance remain pending. `launch.json` records current provenance; `candidate.json` remains the immutable prelaunch snapshot.
