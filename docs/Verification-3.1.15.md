# Verification — application 3.1.15 / xSAR 0.16.0

All 51 fresh paired host gates passed: 1,891 application, 365 library and 80 console tests, zero failed, ignored, measured or filtered.

This is host qualification for the approved [goal](FREE_CELL_goal_20261111F.md) and [contract](FreeCell-Quarter-3.1.15.md). It does not establish Windows cursor/button state, Solitaire acceptance or a solved drag defect. Fresh operator Windows acceptance remains pending. No application launch/replacement, new guest capture/input, installation or promotion occurred.

## Source and toolchain binding

Capsule: `Evidence/FreeCell/Validation/freecell-quarter-315-20261111/`. Prior 3.1.14 scoped sentinel passed before mutation with independently supplied closeout SHA-256 `1b9a5edfe81718b1a5a2d3e89f9472e08046785349ac3e3bb39964101596767c`, exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)`. The snapshot retains 1,050 baseline inputs, 1,076 preimages and four repository states. Prior evidence remains sealed.

All final gates used the same 1,054-input seal `b4e6a01e8640f3a902dfdd32c699018a01927674c2bf059e187e97afa0d35217`, with input and toolchain/configuration checks before and after every command. Exact command arguments, package/cwd, environment, toolchain seal, timings and original logs are retained in each gate receipt and `qualification-summary.json`. No cached gate is substituted.

Toolchain:

```text
rustc 1.102.0-nightly (ac8310ba6 2026-11-09)
cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)
rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)
```

Host target remains `x86_64-unknown-linux-gnu`, Rust edition 2024 and MSRV 1.101.0. Existing configuration, hardening, feature defaults, dependencies, normal drag, REL, source recognition/search, AUTO sequencing and cached recovery remain unchanged. App and console use path-only xSAR requirements; their lock resolutions align with 0.16.0. The public feature is additive; Cargo treats the pre-1.0 minor transition as a compatibility boundary. Existing diagnostic entry points keep their prior approach contract.

## Fresh checks and focused results

The paired schedule covers scoped formatting, locked/offline all-target checks, strict private rustdoc (`-D warnings`), all-target Clippy (`-D warnings`), complete tests including ignored cases and release builds. It also checks dependency-free defaults, all 16 isolated xSAR features and the real console consumer. The release uses Cargo JSON to select artifacts rather than assuming a filename.

Focused transport: 11 passed; focused application: six passed. Both receipts have unchanged inputs/toolchain. These local peers validate exact seven input packets, six selected real timing intervals, source-ACK gating, all six held replies after UP, routed release-only STOP/error recovery and native PLAY/CELL integration. Pure tests cover all occupied/empty CELL/SUIT centres, canonical suit mapping, PLAY destinations and invalid paths. Native fixtures/peers are host evidence only.

All qualifying attempts passed on their first run. No failures were discarded. Prior historical failure evidence remains in its sealed capsules.

| Gate | Exit | Seconds |
| --- | --- | ---: |
| `xsar-format` | 0 | 0.778 |
| `xsar-check` | 0 | 2.408 |
| `xsar-rustdoc` | 0 | 1.556 |
| `xsar-clippy` | 0 | 2.367 |
| `format` | 0 | 0.764 |
| `check` | 0 | 13.834 |
| `rustdoc` | 0 | 2.561 |
| `clippy` | 0 | 14.437 |
| `console-check` | 0 | 1.215 |
| `console-rustdoc` | 0 | 1.381 |
| `console-clippy` | 0 | 1.066 |
| `xsar-default-check` | 0 | 0.863 |
| `xsar-default-tests` | 0 | 1.052 |
| `xsar-feature-cards-check` | 0 | 0.856 |
| `xsar-feature-cards-tests` | 0 | 1.143 |
| `xsar-feature-execution-check` | 0 | 0.915 |
| `xsar-feature-execution-tests` | 0 | 1.224 |
| `xsar-feature-freecell-check` | 0 | 0.912 |
| `xsar-feature-freecell-tests` | 0 | 1.217 |
| `xsar-feature-freecell_json-check` | 0 | 1.0 |
| `xsar-feature-freecell_json-tests` | 0 | 1.63 |
| `xsar-feature-freecell_rules-check` | 0 | 0.981 |
| `xsar-feature-freecell_rules-tests` | 0 | 12.842 |
| `xsar-feature-freecell_search-check` | 0 | 1.381 |
| `xsar-feature-freecell_search-tests` | 0 | 13.249 |
| `xsar-feature-image_matching-check` | 0 | 1.001 |
| `xsar-feature-image_matching-tests` | 0 | 1.532 |
| `xsar-feature-pixel_format-check` | 0 | 0.898 |
| `xsar-feature-pixel_format-tests` | 0 | 1.314 |
| `xsar-feature-png-check` | 0 | 0.971 |
| `xsar-feature-png-tests` | 0 | 1.656 |
| `xsar-feature-pyramid-check` | 0 | 0.917 |
| `xsar-feature-pyramid-tests` | 0 | 1.305 |
| `xsar-feature-pyramid_json-check` | 0 | 1.205 |
| `xsar-feature-pyramid_json-tests` | 0 | 2.792 |
| `xsar-feature-pyramid_rules-check` | 0 | 0.997 |
| `xsar-feature-pyramid_rules-tests` | 0 | 1.481 |
| `xsar-feature-pyramid_search-check` | 0 | 1.161 |
| `xsar-feature-pyramid_search-tests` | 0 | 2.114 |
| `xsar-feature-qmp-check` | 0 | 1.213 |
| `xsar-feature-qmp-tests` | 0 | 26.423 |
| `xsar-feature-search-check` | 0 | 1.034 |
| `xsar-feature-search-tests` | 0 | 1.688 |
| `xsar-feature-tcp_capture-check` | 0 | 0.957 |
| `xsar-feature-tcp_capture-tests` | 0 | 2.171 |
| `xsar-tests` | 0 | 40.413 |
| `tests` | 0 | 103.53 |
| `console-tests` | 0 | 5.308 |
| `xsar-release` | 0 | 2.721 |
| `console-release` | 0 | 1.891 |
| `release` | 0 | 63.349 |

## Executable and dependency contexts

Qualified mutable Cargo artifact: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.15, 16830576 bytes, mode 0755, SHA-256 `4262909f4b68f3cabc8f51312e5089123c70b6559cc96886e55fad45090fe41c`. It is not an installed/protected copy and rebuilding does not replace any running application.

Standalone all-feature xSAR and the application-consumer xSAR are distinct compilation contexts. Their exact paths, features, profiles, package IDs and digests, plus the console artifact, are in `paired-artifacts.json`. All are freshly qualified by the corresponding final records.

## Preservation and evidence limits

| Repository | HEAD | Index/staged |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | Preserved |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | Preserved |
| qmp-qemu-socket | `c541e878440b19e3f7aaa289d7e3c92ed895fe2c` | Preserved |
| xsar | `59c5639eed1b57e5be51b97c3f35065f92c22b42` | Preserved |

All four histories, index bytes and staged diffs are preserved. The guest capture-server diff and unrelated material remain unchanged; the console changes only its xSAR lock entry. Source ownership and metadata differences are checked against the exact preimages. No new 3.1.15 sentinel pass is claimed; its expected closeout digest must come from the independent delivery/approval record. Default sentinel checks source/evidence/Git; optional flags separately validate mutable artifacts/current toolchain.

The new path keeps LEFT held through destination/up45/left34/exactdestination in application intent; only Windows instrumentation or operator behaviour can establish guest receipt/effect. Quarter movements and extra intermediate updates remain an experimental hypothesis. Route credit still requires two consecutive fresh exact final recorded endpoints within six observations; no uncertain move is replayed.

## Code Analysis

The new public path boundary requires exactly four native points, bounds-checks/maps source and every point before authority/I/O, rejects mapped source/final aliases and preserves intermediate aliases as distinct requests. Allocation is bounded to four movements. No unsafe code, dependencies, concurrency or listeners were added. Existing STOP/context and original-cause targeted safety-UP recovery remain; release may affect the guest and is not rollback. Pointer routing cannot be pinned through the gesture, waits are not instant cancellation, and session logs retain their existing durability limits.

Suggested commit subject: `feat(freecell): trial quarter-card ABS drag path`. No commit or promotion was performed.
