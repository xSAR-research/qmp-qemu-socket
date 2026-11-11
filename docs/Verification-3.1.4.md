# Free Cell RECON 3.1.4 verification

## Approved scope and behaviour

[The current goal](FREE_CELL_goal_20261109B.md) supersedes the earlier acquisition
scene restrictions. Read-only RECON validates native frame storage and crop
bounds, retains the challenge information and reads all 52 fixed rank/suit
positions. Each identity must be readable and unambiguous, and the resulting
inventory must be a complete unique deck. Matching unrelated scene pixels,
felt, empty-slot decoration and unhighlighted borders is not an acceptance rule.

The retained native incident contains a readable gold-highlighted 9♦ with warm
paper. RECON uses the existing local-paper contrast and coherent print evidence
without a separate white-paper brightness veto. Rank and suit shape limits,
distinct-label separation, native alignment variants and the FC13 template bank
remain unchanged. Blank, unreadable, mixed-colour, ambiguous or duplicate card
evidence still refuses; deck completeness never supplies a missing identity.

## Boundaries and compatibility

The application is 3.1.4; xSAR remains 0.11.1. Public crate APIs, JSON schema,
dependencies, transport, UI ownership and concurrency are unchanged. The expanded
RECON acceptance is intentional. Descriptive acquisition provenance grants no
input authority. Nonblank challenge information is retained as unresolved pixels,
not OCR text or a recognised completion objective; planning and playback still
refuse it.

Live initial binding uses a separate strict entry point retaining the existing
scene, top-slot, border, lower-face, rail and white-paper checks. Subsequent live
position verification, source identity, STOP/context/VM/pointer probes, guarded
drag, release-only recovery and two-observation effect proof remain unchanged.
No capture, input, retry, fallback or authority is added by this correction. Strict geometry now preflights before identity matching, so a live frame with several defects can report a different first refusal while retaining the same acceptance boundary.

## Evidence and qualification

All six required host gates passed on the same 1,024 compiler inputs, seal SHA256
`8b3adc9ce4a242c489ff412fdd77d1395e7b6806b341f06a417e4a01637c4a71`.
Source and toolchain stability were checked before and after each gate. Rustc is
1.102.0-nightly `f5cedb83a788ea9581097abc36a2a8503a7f541c` (2026-11-07);
Cargo is 1.102.0-nightly `c07ac2c4d12237effc4b94f4543769b421790045`.
Target is `x86_64-unknown-linux-gnu`, with the existing CPU/linker configuration.

| Gate | Actual result |
| --- | --- |
| Nightly rustfmt, exact five owning files | PASS |
| Locked offline all-target check, all features | PASS |
| Strict private rustdoc, all features | PASS |
| Strict all-target Clippy, all features | PASS |
| Complete tests, all features, including ignored tests | 1,810 passed; zero failed/ignored/filtered |
| Release build, all features | PASS |

Cargo uses `+nightly --locked --offline --package qmp-qemu-socket --target
x86_64-unknown-linux-gnu --all-features`. Check and Clippy add `--all-targets`;
Clippy adds `-- -D warnings`. Rustdoc uses `--no-deps --document-private-items`,
`RUSTDOCFLAGS=-D warnings` and encoded `-D` / `warnings`. Tests add
`-- --include-ignored`; release adds `--release --message-format=json-render-diagnostics`.
Formatter uses edition 2024 and `skip_children=true` on the exact owning files.
Complete commands, environments, logs and receipts are retained under
`Evidence/FreeCell/Validation/freecell-recon-314-20261109/`.

The first focused compilation overlapped the operator's nightly update from
1.101.0 to 1.102.0 and ran no tests. Its metadata-mismatch output remains in
`focused-first.log`. The retry passed all seven new tests; the full six-gate run
used settled nightly 1.102.0 without a source correction.

| Requirement | Direct verification |
| --- | --- |
| Frame dimensions, storage and bounded crops | `freecell_314_invalid_frame_storage_and_native_dimensions_refuse`; checked crop extraction |
| Challenge retained and all 52 identities read | Native highlighted-board test compares every physical identity with manual labels; strict JSON round trip |
| Unrelated pixels and borders irrelevant | Invariance test changes every pixel outside objective and full glyph-crop unions, preserving the exact acquired deal |
| Readable paper; live policy separate | Isolated grey/blue/warm paper test keeps borders intact: RECON accepts; strict initial and position readers refuse |
| Unreadable, ambiguous or duplicate cards refuse | Low-contrast, mixed-colour and duplicate regressions; original native rank 34 versus 50 still fails gap 20; no elimination |
| One capture, exact evidence and export | Local TCP native-highlight/pop-up test exports the same board on two explicit requests, retains exact per-request pixels and uses no QMP peer |
| Playback authority preserved | Independent source review, preserved strict negatives and entire unchanged position verifier; full suite passed |

The new retained native board is byte-identical to its incident image, SHA256
`35dbe33e2e9f05ab593dc0334094cd5acae026e1c5d619d8ec46f3b65adf5636`.
It informed this correction and is not claimed as a blind evaluation. Synthetic
derivatives are labelled; original incident packets are unchanged. The earlier
ambiguous native 9♦ still scores 34 versus 50 and is refused without weakening
confidence or using elimination. Independent review found no authority widening.

## Executable and preservation

Normal mutable Cargo target: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.4, 16706832 bytes, mode `0755`, SHA256 `512b0983c15ef84e0f3b2c25ce8b68c525b075684b11c94132449a21132a7c31`. No protected secondary copy was made. The Cargo artifact record is retained as `artifact.json`.

The app base remains published `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`; qualification binds the changed tree, not HEAD alone. All four Git histories/indices and the independent sibling staged/unstaged changes match the starting receipt. Source remains uncommitted and unstaged. Original incidents and sealed 3.1.3 evidence are preserved. No installation, application launch, guest input or promotion occurred; fresh guest/operator acceptance remains unperformed.

Fresh guest acquisition, UI acceptance and live playback were not performed. Unresolved challenge information remains unavailable for CALCULATE/playback.

## Code Analysis

The changed boundary is read-only acquisition acceptance. Native frame storage
and every sampled crop remain checked, cancellation is retained, and existing
bounded matching and strict unique-deck construction control failure. Successful
inventory acquisition is not scene or playback authentication. No new unsafe
code, dependency, shared-state owner or input operation is introduced. Publication
and diagnostic failures retain their existing no-overwrite and causal-error
semantics.
