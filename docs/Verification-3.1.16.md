# Verification — application 3.1.16 / xSAR 0.16.1

All 51 fresh paired host gates passed with 1,893 application, 366 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests.

The approved [goal](FREE_CELL_goal_20261111G.md) and [contract](FreeCell-Source-Midpoint-3.1.16.md) define the source-up and midpoint diagnostic. Fresh operator Windows acceptance remains pending. The local peers measure outgoing requests and supplied capture responses. Windows held-button state and the game’s in-flight handling require guest evidence.

## Source and toolchain binding

Capsule `Evidence/FreeCell/Validation/freecell-source-midpoint-316-20261111/` retains 1,054 baseline compiler inputs, 1,080 preimages and four repository Git states. The previous 3.1.15 scoped Destination Sentinel passed before mutation using independently supplied closeout SHA-256 `d543b183ee3e5e2dc5bdc740b1f23adba1ae0c594df5c0d28c3962c4261ed9c0`, returning exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)`.

All 51 final gates bind the same 1,057 compiler inputs through seal `5979ab6e76503cf04bd821870d3274639001bf4175afaf9e6717aae48f1d425c`. Every command checks source, toolchain and Cargo configuration before and after execution. Exact arguments, working directories, environments, times and log hashes are retained in the gate receipts and `qualification-summary.json`.

```text
rustc 1.102.0-nightly (ac8310ba6 2026-11-09)
cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)
rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)
```

The host target is `x86_64-unknown-linux-gnu`, edition 2024 and MSRV 1.101.0. Application and console dependencies resolve xSAR through their existing path requirements. The approved metadata updates resolve application 3.1.16 and xSAR 0.16.1 across the three lockfiles.

## Checks and retained first failure

The paired schedule covers scoped formatting, locked/offline all-target checks, strict private rustdoc and Clippy with warnings denied, full tests including ignored cases, release builds, xSAR defaults and all 16 isolated features, and the real console consumer.

The initial focused runs passed 12 transport tests and eight application tests against the first sealed source. The first full application run passed 1,892 tests and failed one older AUTO native fixture. Its packet assertion expected the previous destination-first path, while the actual first held movement correctly used the source x coordinate. The mismatch was `3976` versus stale expected `22203` at `native.rs:369`. Only the fixture’s four held coordinates were corrected. Its seven-packet, one-drag and two-proof/cursor assertions remain. `tests-1.json` and `tests-1.log` preserve this first failure. The corrected fixture and complete final suites are bound by the final seal recorded above.

The public path fixtures cover source acknowledgement before DOWN, UP before held reply collection, both real timing profiles, exact seven input requests and six selected waits, independent mapped literals, odd midpoint sums, widened arithmetic, native y=45 boundaries and invalid coordinates. Existing STOP, command/write failure and captured-pointer release-only recovery cases also run. Application peers exercise PLAY and upper-slot integration with supplied native captures.

| Gate | Exit | Seconds |
| --- | --- | ---: |
| `xsar-format` | 0 | 0.793 |
| `xsar-check` | 0 | 0.801 |
| `xsar-rustdoc` | 0 | 0.79 |
| `xsar-clippy` | 0 | 0.844 |
| `format` | 0 | 0.808 |
| `check` | 0 | 4.124 |
| `rustdoc` | 0 | 2.888 |
| `clippy` | 0 | 4.739 |
| `console-check` | 0 | 0.805 |
| `console-rustdoc` | 0 | 0.782 |
| `console-clippy` | 0 | 0.808 |
| `xsar-default-check` | 0 | 0.788 |
| `xsar-default-tests` | 0 | 0.846 |
| `xsar-feature-cards-check` | 0 | 0.753 |
| `xsar-feature-cards-tests` | 0 | 0.83 |
| `xsar-feature-execution-check` | 0 | 0.768 |
| `xsar-feature-execution-tests` | 0 | 0.814 |
| `xsar-feature-freecell-check` | 0 | 0.754 |
| `xsar-feature-freecell-tests` | 0 | 0.834 |
| `xsar-feature-freecell_json-check` | 0 | 0.755 |
| `xsar-feature-freecell_json-tests` | 0 | 0.856 |
| `xsar-feature-freecell_rules-check` | 0 | 0.748 |
| `xsar-feature-freecell_rules-tests` | 0 | 12.19 |
| `xsar-feature-freecell_search-check` | 0 | 0.764 |
| `xsar-feature-freecell_search-tests` | 0 | 12.086 |
| `xsar-feature-image_matching-check` | 0 | 0.752 |
| `xsar-feature-image_matching-tests` | 0 | 0.979 |
| `xsar-feature-pixel_format-check` | 0 | 0.756 |
| `xsar-feature-pixel_format-tests` | 0 | 0.961 |
| `xsar-feature-png-check` | 0 | 0.781 |
| `xsar-feature-png-tests` | 0 | 1.007 |
| `xsar-feature-pyramid-check` | 0 | 0.819 |
| `xsar-feature-pyramid-tests` | 0 | 0.938 |
| `xsar-feature-pyramid_json-check` | 0 | 0.835 |
| `xsar-feature-pyramid_json-tests` | 0 | 1.781 |
| `xsar-feature-pyramid_rules-check` | 0 | 0.751 |
| `xsar-feature-pyramid_rules-tests` | 0 | 0.845 |
| `xsar-feature-pyramid_search-check` | 0 | 0.792 |
| `xsar-feature-pyramid_search-tests` | 0 | 0.854 |
| `xsar-feature-qmp-check` | 0 | 0.752 |
| `xsar-feature-qmp-tests` | 0 | 24.853 |
| `xsar-feature-search-check` | 0 | 0.757 |
| `xsar-feature-search-tests` | 0 | 0.819 |
| `xsar-feature-tcp_capture-check` | 0 | 0.753 |
| `xsar-feature-tcp_capture-tests` | 0 | 1.539 |
| `xsar-tests` | 0 | 40.171 |
| `tests` | 0 | 102.35 |
| `console-tests` | 0 | 5.235 |
| `xsar-release` | 0 | 2.503 |
| `console-release` | 0 | 1.884 |
| `release` | 0 | 65.42 |

## Executable and compilation contexts

The qualified Cargo executable is `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.16, 16831760 bytes, mode 0755, SHA-256 `c1f48045c41583b0a2646d1ec92edd82d59de7cdf4cf2f18bbcbcd0427ade699`. This path is mutable when Cargo rebuilds it. Process replacement requires a separate launch.

Cargo JSON selected the release artifacts. `paired-artifacts.json` records separate standalone all-feature and application-consumer xSAR contexts, together with application and console artifacts. Their package IDs, profiles, features, paths and hashes are recorded with the corresponding fresh qualification commands.

## Preservation

| Repository | HEAD | Index and staged changes |
| --- | --- | --- |
| basic-route-planning | `98ac0a7bc02b799829dfddfa251298890c1c1b9f` | Preserved |
| qemu-screen-grab-server | `3e4f76ed4f9c2e0dea4edd91646f6fd87dc78345` | Preserved |
| qmp-qemu-socket | `c541e878440b19e3f7aaa289d7e3c92ed895fe2c` | Preserved |
| xsar | `59c5639eed1b57e5be51b97c3f35065f92c22b42` | Preserved |

All four repository histories, index bytes and staged diffs match the starting records. Source ownership and approved version/lock differences are checked against preimages. The capture-server changes and unrelated material retain their prior bytes. Historical release bodies and previous capsules remain sealed.

The new 3.1.16 Destination Sentinel awaits an independently supplied expected closeout digest. Its default mode verifies source, evidence and Git state. The optional artifact/toolchain flags also compare mutable build outputs and current tools.

## Code Analysis

The application precomputes and validates source-up45, the floor midpoint of the original native source/effective destination, destination-up45 and exact destination. Midpoint sums widen to i64. Upper targets use canonical slot centres, and the last two held points must fit the same destination slot. Effective endpoint consumers use the fourth held point. Source y or destination y below 45 refuses before operation input.

The bounded xSAR four-point transport uses seven input packets and six selected minimum waits of two seconds or 500 ms. Source acknowledgement and a fresh routing check precede DOWN. UP precedes collection of six held replies. Each uncertain post-DOWN failure retains its cause and uses the existing captured-pointer release path. A safety release can affect the guest. Original-pointer restoration requires acknowledged UP and continuing authority.

Ordinary effects still require two consecutive fresh exact recorded endpoints within six observations. Relevant identity proofs, source/run/context ownership and STOP remain the input conditions. Pointer routing can change concurrently, synchronous waits have bounded cancellation latency, and existing session logs have best-effort persistence. The diagnostic remains subject to fresh Windows trial.
