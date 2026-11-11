# Verification — 3.1.8 / xSAR 0.14.1

Status: all 51 fresh host qualification gates passed. The approved scope is the
[held-motion and halo contract](QMP-Drag-Diagnostics-3.1.8.md).

## Source, toolchain and versions

- Application3.1.8; xSAR 0.14.1; consumer0.8.5 with lock-only alignment.
- Source/configuration/test inputs1033; seal SHA256 `36052160e609c451152bcb9344228740e8016ce746bd1c8b1611d94491420ba6`.
- rustc 1.102.0-nightly `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` (2026-11-08);
  Cargo 1.102.0-nightly `c07ac2c4d12237effc4b94f4543769b421790045`.
- Native target x86_64-unknown-linux-gnu; existing owning CPU/hardening flags,
  edition 2024 / MSRV 1.101.0, dependencies/defaults/features/schemas unchanged.
- Old 3.1.7 source continuity was 1,032 matching inputs; its failed sentinel and
  changed mutable artifacts/compiler remain separately recorded. No false cached
  qualification or old-sentinel pass is claimed.

## Recorded checks

Every gate uses `--locked --offline --package <owner> --target x86_64-unknown-linux-gnu`.
Owning gates: scoped nightly rustfmt, all-target check, strict `-D warnings` private
rustdoc, all-target Clippy `-D warnings`, tests `-- --include-ignored`, release JSON
artifact build. xSAR defaults and all 16 isolated features each have check/tests;
the actual console consumer has check/rustdoc/Clippy/tests/release. Exact argv,
environment, toolchain/configuration and stable-input receipts are in
`Evidence/FreeCell/Validation/qmp-step-318-20261109/qualification-summary.json`.

Full results: **1862 application / 347 library / 80 console**, zero failed,
ignored, measured or filtered. All51 gates are fresh against the same seal.

First focused transport run passed 14 cases. First focused application run compiled,
passed 183 and failed five geometry cases; it is retained unchanged. Corrections use
measured native rail spread 21 (one TH pixel RGB200,218,221;22negative retained),
legitimate181/182/183 lower-edge offsets and independent extra-card refusals.
The recognition rerun passed 39 cases. Final full suites cover corrected sources.

## Evidence and limitations

The exact saved sevens image is an authentic byte copy with SHA256
`fef4cdd0a498b4cefb968b07980535d6f7dc6fd5b77052211332dfa196b8face`.
Tests manually assert all 52 printed identities, initial binding and evolving
geometry; mutated-background and occupied/occluded/ambiguous/broken-edge cases
separate advisory canvas from required evidence. Existing native gold-card
fixtures and synthetic generalisations stay labelled separately.

Local peer tests verify queried pointer indices, ten movements, actual two-second
gaps, source/endpoint mapping, release-before-ACK collection, STOP/recovery and
no replay. REL probe tests preserve cursor/history, retire authority before one
after-frame, retain original capture failures and secondary persistence errors.
Those peers and frames do not establish actual Windows held state or Solitaire
acceptance. **Fresh live drag acceptance remains pending operator trial.**

## Delivered executable

`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`

Version 3.1.8; 16,822,312 bytes; mode 0755; SHA256
`a5998ebd55c29804f06f21def72c54e7589972d6969e41d1fbdcdbcec346cef2`.

This is a mutable Cargo artifact, not a protected installation. The executing
PID 672085 remains operator 3.1.7 at old executable digest 6262002e…64476. No agent
launch, new guest capture/input, install, stage, commit, push or publication.
The original 3.1.7 receipt, first failures, four Git histories/index bytes/staged
changes and unrelated capture-server edits are preserved.

## Code Analysis

No new unsafe code or public API. Checked native geometry and fixedten-step
payloads bound input construction; every identity competes independently.
STOP/context/run/source checks and release-only recovery remain. REL probe ACK
cannot advance a route or restore authority; after-frame failure retains that
retirement. Off-card overlays and unsupported artwork may still block actual
guest interaction; stale/unknown required evidence refuses and uncertain input
is never replayed. Logging retains existing best-effort durability limits.
