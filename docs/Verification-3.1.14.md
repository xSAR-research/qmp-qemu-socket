# Verification — application 3.1.14 / unchanged xSAR 0.15.1

All six fresh application gates passed against the final input seal. Full tests on Beast: **1,887 passed, zero failed, ignored, measured or filtered**. xSAR remains 0.15.1 and the console remains 0.8.5. No fresh sibling gate or current sibling-artifact qualification is claimed.

## Source and toolchain binding

The capsule is `Evidence/FreeCell/Validation/freecell-identity-314-20261111/`. The starting snapshot preserves 1,046 baseline inputs, 1,072 exact preimages and four repository Git states. Final `qualified-final-inputs.json` covers 1,050 inputs with SHA-256 `39a575f501e87c6122d6bc844bebd80c05e979cab58afd324cbf3b802fa1b618`. Every final gate verified the full input inventory and selected toolchain/configuration before and after execution. The dirty xSAR path dependency is bound by these file fingerprints, not its HEAD alone.

Selected host: `x86_64-unknown-linux-gnu`. Toolchain: `rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`, `cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)`, `rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)`. Full literals and Cargo/environment hashes are in `toolchain-qualified-final-inputs.json` (SHA-256 `8ac2eefb080a42b580aa7d37189d2559e902d2d2220948c457e15e7c9c58c722`). Features, dependencies, edition, MSRV, target and existing compiler settings remain unchanged. Only the app manifest/lock version advances from 3.1.13 to 3.1.14.

## Required checks

| Gate | Result | Final log | Elapsed |
| --- | --- | --- | --- |
| format | PASS | `format-2.log` | 0.867 s |
| check | PASS | `check-2.log` | 4.443 s |
| rustdoc | PASS | `rustdoc-2.log` | 2.623 s |
| clippy | PASS | `clippy-2.log` | 4.865 s |
| tests | PASS | `tests-2.log` | 105.581 s |
| release | PASS | `release-1.log` | 62.865 s |

Exact final argv, also recorded with environment and hashes in `qualification-summary.json`:

- `rustup run nightly rustfmt --edition 2024 --config unstable_features=true,skip_children=true,blank_lines_lower_bound=0,blank_lines_upper_bound=2,style_edition=2024 --check src/freecell_recon.rs src/freecell_recon/tests.rs src/freecell_recon/position.rs src/freecell_recon/position/tests.rs src/worker/freecell_route.rs src/worker/freecell_route/tests.rs src/worker/freecell_route/tests/native.rs`
- `cargo +nightly check --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets`
- `cargo +nightly doc --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items`
- `cargo +nightly clippy --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings`
- `cargo +nightly test --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features -- --include-ignored`
- `cargo +nightly build --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --release --message-format=json-render-diagnostics`

Strict private rustdoc sets `RUSTDOCFLAGS=-D warnings` and its encoded equivalent with the U+001F argument separator. Each Cargo gate uses the host target, locked/offline operation and all features. The formatting gate covers only the seven approved owning Rust files. `QMP_SESSION_LOG_DIR` points to the private capsule session directory; the historical acceptance deal is pinned to this capsule. No helper command launched the application or contacted the running guest.

## Focused evidence and preserved failures

The first recognition focus retained 35 passes and eight wrong-exposed-corner failures in `focused-identity-first.log`. Investigation found a quiet-gap prescreen inside the actual rank crop: native queen tails were being vetoed. The correction removes that invalid gap and coarse-ink escape hatch, derives each label's own local contrast and retains independently qualified all-label shape support. Rank 48/gap 20 and suit 16/gap 12 never changed. Missing-rank/suit, mixed-colour, duplicate, ambiguous and partially readable lower-corner regressions remain.

The corrected recognition focus retained 43 passes and one obsolete point assertion: the observed sevens corner point was `(233,669)`, while the fixture expected `(233,670)`. Only that fixture expectation changed. `focused-worker-identity.log` then passed all nine correction tests, including the authentic sixes binding and relevant-pile local-peer transfer. This transfer test adds a labelled synthetic obstruction to unrelated PLAY3 after authentic all-52 binding; JH PLAY6→CELL1 still sends exactly one transfer. It does not claim a proved guest effect.

The first full run retained 1,886 passes and one old diagnostic assertion expecting `PLAY 1 measured 6 cards, expected 7`. The late-AUTO source change correctly refused before any VM probe/input under the new exposed-identity contract. That assertion now requires the exposed-identity mismatch and preserves all zero-input/retirement checks. Its focused retry passed. All six gates were run against the final test-file seal; all 1,887 tests passed. Earlier logs/receipts remain unchanged.

Retained native FC14 demonstrates compressed 29/30-pixel strips and an independently readable last corner around native y795, whose body is clipped by the toolbar. Hidden strips are not visually counted. The authentic sixes fixture is a byte-identical copy of the saved incident (SHA-256 `3621b103590b6d5b1c5c52fc7d2b9fba48ff6c1f0800f277e9d8336c01bd4d61`) with independently asserted 52 identities. The earlier real 9D ambiguity still refuses. Independent read-only source/contract review passed; its findings and limits are retained in `independent-review.json`.

## Executable and preservation

Qualified mutable executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`. Version 3.1.14; 16827632 bytes; mode 0755; SHA-256 `82f24c96eb91c1069c1873390d054a4eb192e30a3441b53a42fac4634a8eb65e`. It is the Cargo release artifact, without a protected copy or installation. Rebuilding does not replace a running process; no process was launched or replaced by the agent.

| Repository | HEAD | Index/staged state |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | preserved |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | preserved |
| qmp-qemu-socket | `c541e878440b19e3f7aaa289d7e3c92ed895fe2c` | preserved |
| xsar | `59c5639eed1b57e5be51b97c3f35065f92c22b42` | preserved |

All three sibling tracked diffs/manifests/locks remain unchanged. No public xSAR API, dependency, deal/route schema, QMP gesture, timing, normal-game or REL policy changes. Prior sibling build/test records are historical only; see `retained-prior-artifacts.json`.

The previous 3.1.13 Destination Sentinel passed before edits using independently supplied closeout SHA-256 `b847bcce3cb3b665a2d7d2396525003ddaacf7d0ac41c6a1b67b006632b3a5a7`, exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)`. The prior capsule remains immutable. No new 3.1.14 sentinel pass is claimed; its later validation needs the independently approved delivery digest. Default sentinel coverage is source/evidence/Git; add both artifact/toolchain checks for a current cached-build claim.

## Acceptance limits

Native saved pixels and synthetic TCP/QMP peers qualify the host recognition and sequencing boundaries. They do not prove fresh Windows gameplay or an authentic Free Cell challenge-completion appearance. Free Cell goals remain one-board card/rank-count goals, without score or multi-board continuation. Partial routes remain valid; no target OCR or target-specific search was added. No new guest capture/input, host/guest relaunch, installation, staging, commit, push or publication occurred. Operator Windows acceptance is pending.

## Code Analysis

The changed surface is private image evidence and worker proof selection. Native storage/bounds and cancellation checks remain; independent all-label competition and exact original-52 binding prevent identity completion by elimination. Requested occupied points come from measured printed corners and unrequested geometry refuses. Hidden counts/identities belong to authenticated original state and legal history. The locator is bounded to calibrated native x and y331..900; wholly unknown material and arbitrary overlays are not universally detected. Unknown relevant identity, STOP/context/source/run changes and uncertain effects stop without replay. Existing two-fresh-final-endpoint proof, passive terminal retirement and cached manual UNDO ALL restart remain. No unsafe code, dependency, listener or extra guest operation was introduced.
