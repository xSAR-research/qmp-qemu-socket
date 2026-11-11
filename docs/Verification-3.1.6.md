# Verification — 3.1.6 / xSAR 0.13.0

Application **3.1.6** with xSAR **0.13.0** is host-qualified for the approved QMP diagnostic. Complete outgoing JSON packets, including `execute`, `arguments` and `id`, are recorded under `QMP_WIRE` in the existing private session log. Capability negotiation and primary/fresh release-only recovery are covered. `detail.json` on each `write-attempt` contains the exact compact packet string; separate flushed/failed records distinguish write outcomes from acknowledgement and guest effect.

A* Free Cell SINGLE STEP uses one-second source/down/held-move/up intervals, then a cancellable three-second wait, one fresh terminal/physical-authority frame and a separately guarded current-pointer LEFT click. Normal route playback retains its existing gesture. STOP after confirmed UP produces no later capture, probe, click or recovery connection. The usual two fresh physical endpoint proofs within six observations remain necessary for cursor advancement.

All **51 required gate outcomes passed**: **47 fresh gates** bind the final **1,031 inputs** at seal SHA256 `34b5cce8ddb9dadc28d1806e322af7146488d2eac418d126a12e11620e81aa65`; four library format/check/private-doc/Clippy gates are retained against identical xSAR source, configuration and toolchain. The intervening source difference was a consumer-private Clippy simplification; `retained-library-gates.json` records the comparison. The matrix covers dependency-free defaults, all sixteen isolated xSAR features, both owning packages and the real console consumer. Full tests passed **1,845 application, 338 library, 80 console**, zero failed, ignored or filtered. All eleven new 3.1.6 tests passed in the full suites.

Application base is `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`; xSAR base is `59c5639eed1b57e5be51b97c3f35065f92c22b42`. Exact dirty-file pins bind qualification. All four Git histories, indices and staged changes, plus independent capture-server edits, are preserved. Console stays at 0.8.5 with approved lock alignment. No installation, application launch, guest input or promotion was performed.

Windows/Solitaire drag acceptance and a QEMU root cause remain unverified. Supplied xHCI/tablet, listener and actual `/run/user/1000/qmp-qemu-socket.sock` configuration show no discrepancy requiring a setup change. Logging remains best effort if the existing session log is disabled, fails or the process terminates abruptly.

The [approved diagnostic contract](QMP-Drag-Diagnostics-3.1.6.md) governs this build.

| Requirement | Actual evidence |
| --- | --- |
| Full JSON, IDs, time and connection | Observer peers compare every request against received JSON, including capabilities, probes, input and tagged fresh recovery; genuine write failure keeps its original error class |
| Single Step sequence | Real one-second phase timing and separate current-pointer down/up without extra ABS; ordinary route retains four input packets |
| STOP, terminal and physical guards | Native local peers prove fresh pre-click frame and six-packet sequence; terminal, unknown frame and post-UP STOP prevent extra click/probe/recovery |
| All-game private session log | Scope tests cover frozen owners, all games, exact packet text, recovery and disabled-scope restoration; independent review confirms current/stale handling and join-error final drain |
| Existing transport | Legacy full suites pass; normal drag body and QmpConfig unchanged; unwinding observer failures retain causal errors |
| Compatibility | Additive observer/types/constructors/diagnostic methods; existing signatures, defaults, schema, dependencies and requirement strings retained; consumer locks resolve 0.13.0 |
| Delivery | Exact compiler/toolchain receipts, 51 outcomes, four-repository preservation and Cargo-selected artifacts |

## Qualification

Common Cargo flags: `+nightly --locked --offline --package <owner> --target x86_64-unknown-linux-gnu --all-features`. Check/Clippy include `--all-targets`; Clippy uses `-D warnings`. Private rustdoc uses `--no-deps --document-private-items` and both ordinary and encoded `-D warnings`. Full tests use `-- --include-ignored`; isolated rows use `--no-default-features` and one named feature. Release uses `--release --message-format=json-render-diagnostics`. Format checks cover only seven owning app and two owning library Rust files.

Inputs, rustc/Cargo/rustfmt, inherited compiler environment and Cargo configuration were checked before/after gates. First failures remain: initial app test compilation lacked a test-only QmpError import and used newly deprecated fetch_update; initial strict app Clippy requested `?` for an Option early return. Narrow fixes preserve behaviour, use MSRV-compatible try_update and add no lint exceptions. Final app checks/tests cover these corrections.

Exact commands, environments, timing, logs and review are retained in `Evidence/FreeCell/Validation/qmp-wire-316-20261109`. Toolchain: rustc 1.102.0-nightly (f5cedb83a 2026-11-07); cargo 1.102.0-nightly (c07ac2c4d 2026-11-06). Tests used host/local Unix-QMP/TCP peers; the Windows guest was not driven.

Qualified artifact: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, **3.1.6**, 16800728 bytes, mode 0755, SHA256 `24ed172a208848d5931f694a3add43065881ab1255f7b4d9dffb42ef17f48dac`. This is mutable Cargo output, without a protected copy. Rebuilding does not replace any running process.

## Reading the packets

The existing application session log announces its actual filename. Search it for `QMP_WIRE`. Every `write-attempt` contains the full compact packet string at `detail.json`, including execute/arguments/id. Connection ID, primary/release-recovery kind and QMP ID disambiguate reset IDs; microsecond/source timestamps preserve phase timing. `write-flushed` and `write-failed` distinguish outcomes. Flush does not prove acknowledgement or effect. Logging introduces no capture or input.

## Code Analysis

The observer borrows immutable packet/error evidence and owns no files or game policy. The app queues existing private SessionTelemetry without filesystem I/O in the serializer. Checked mapping, STOP/context/run/source fences, fresh pre-click proof and release-only recovery retain authority. Uncertain input is never replayed. No new unsafe code or dependencies. Unwinding callbacks are contained; existing release panic=abort means process-aborting failures cannot be caught. Disabled/failed logs or abrupt exit may lose records; no durable-before-input guarantee. The extra diagnostic click may change a card and cannot waive endpoint proof.
