# v2.1.0 development — Pyramid reconnaissance to JSON

Gate 2 implementation and automated validation completed on 11 October 2026,
Australia/Brisbane. Gate 3 live acceptance and Gate 4 promotion remain pending.
No controller was launched, installed, committed or pushed by these checks.

## Delivered scope

- Native, explicitly calibrated rank/suit reading for original logical tableau
  positions and LEFT/RIGHT piles, with unknown/ambiguous refusal.
- A distinct GUI/worker reconnaissance command. Ordinary independent preparation
  remains read-only; existing five-game CV controllers and the Spider XP quirk
  are unchanged.
- Guarded stock acquisition, unique standard-deck inventory, fresh advancement,
  positive exhaustion, specific Undo All confirmation and twice-verified baseline
  restoration before complete JSON publication.
- Actual xSAR 0.3.0 consumption: pure card/deal types, strict versioned JSON and
  the new atomic no-overwrite filesystem publisher. UI/QMP/native calibration
  remain application-owned. Console intake and search are deferred.
- Private incomplete diagnostics, original baseline/latest PNGs and the existing
  bounded latest-frame preview; no uncertain input or restoration replay.

The [controller contract](pyramid-reconnaissance.md),
[native profile](pyramid-recon-profile.md) and
[live-test brief](pyramid-recon-live-test.md) describe exact bounds and authority.

## Source and dependency boundary

Application base: `ba1a82beb3ea39242d91ead39ebdbd1198847f84`, package 2.0.11.
Current development package: 2.1.0, edition 2024, unchanged Rust 1.101.0 MSRV.
xSAR base: `987856a0b5cb95aec2788551e050e18f7469b1ef`; reviewed local foundation
and publisher are package 0.3.0, edition 2021, unchanged Rust 1.101.0 MSRV.

All application checks use the external
`/tmp/pyramid-recon-build.5qifGKcG/local-xsar.toml` source override. Cargo metadata
confirms the actual local 0.3.0 source with defaults disabled and qmp, png,
image_matching and pyramid_json, including the latter's cards/pyramid features.
No unrelated dependency version changed. The app's manifest still retains the
old promoted Git revision, which cannot independently supply the new APIs.

**Do not promote the current app manifest/paired lock state.** Charlie first
accepts and promotes xSAR, then supplies its exact commit for the application
repin, lock regeneration and locked revalidation. No host path or temporary
override belongs in promotion files. Both Git indexes remain unchanged.
Local AGENTS/HANDOFF and workspace files are ignored and not tracked in either
changed repository. The console source remains untouched.

## Automated validation

Tools: rustc `1.101.0-nightly (32dba69d6 2026-10-09)` and Cargo
`1.101.0-nightly (29c5daa1a 2026-10-07)` on x86_64-unknown-linux-gnu.
Commands use `--locked --offline`; xSAR uses explicit `+nightly`, an external
target directory and the host target. No formatter or global-toolchain change.

Evidence root: `/tmp/pyramid-recon-build.5qifGKcG/`.

| Check | Actual result | Evidence |
| --- | --- | --- |
| Application full `test -- --include-ignored` | 684 passed, 0 failed, 0 ignored; 88.20 seconds | `app-full-tests-delivery.log` |
| Application check | Passed | `app-check-final.log` |
| Application strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` | `app-doc-delivery.log` |
| Application Clippy, all targets | Passed with `-D warnings` | `app-clippy-delivery.log` |
| Application explicit host release | Passed; not launched or installed | `app-release-delivery.log`, `app-release-artifacts-delivery.jsonl` |
| xSAR all features, include ignored | 123 unit/integration tests plus 2 doctests passed; 0 failed/ignored | `xsar-actual-all-tests-escalated.log` |
| xSAR dependency-free default tests | 14 passed | `xsar-default-tests.log` |
| xSAR default/cards/pyramid/pyramid_json/all feature checks | All passed | `xsar-feature-checks.log` |
| xSAR strict Clippy/private rustdoc/release | All passed | `xsar-clippy.log`, `xsar-doc.log`, `xsar-release.log` |
| Lean JSON dependency graph | serde/serde_json only; no image/QMP/GUI domain | `xsar-json-dependencies.log` |
| Source whitespace and local documentation links | Passed | Final working-tree audit |

The core integration test drives a scripted acquisition/restoration through the
real xSAR atomic writer and reader, compares the complete original order, and
checks refusal without altering an existing destination. Scripted observations
are not represented as live captures. STOP, phase order, bounded retries,
uncertain delivery, publication status, path refusal and GUI ownership are tested.

Read-only source hygiene found no high-signal credential matches in nonignored
text, staged changes and the last five commits. Expected author/name references
and documented local development paths remain for human review; binary images
were not OCR-scanned. See the two `*-secrets-report.md` evidence files. This is
pattern-based hygiene, not a guarantee that no sensitive data exists.

## Native evidence and resolved failures

The following counts describe the first delivery. Two subsequent failed live
captures and the profile-only runtime correction are recorded separately in
[Gate 3 scene correction](pyramid-recon-gate3-correction.md). Neither failed
attempt sent guest input or published a complete deal.

A later run acquired 28 tableau and 24 ordered stock identities, acknowledged
Undo All, then refused its modal without attempting OK or publication. The
[modal correction record](pyramid-recon-modal-correction.md) documents the
measured v3 profile update, additional original evidence and validation.

The repeat loaded v3 correctly and recorded the same complete inventory/order,
but ten Cancel/OK pixels exceeded their channel limits by one. The
[stable-text correction record](pyramid-recon-stable-text-correction.md)
documents v4's source-only masks with unchanged RGB/tolerances, the explicit
acceptance trade-off, additional native evidence and validation. No successful
live restoration or complete publication is claimed by either failed scan.

Twenty historical PNGs and all three newly supplied captures are retained without
resampling or recompression: 23 originals, 61,420,210 bytes (about 58.6 MiB).
The fixture audit verifies 68 source glyph samples and 14 scene/control patches.
The independent reference study records 104 native positives and 1,255 adverse
checks; the production application tests independently recognise the current
28-card start/exhausted tableaux, LEFT K diamonds, empty RIGHT, empty LEFT,
RIGHT Q diamonds and complete supplied modal.

First failures remain retained rather than overwritten:

1. xSAR filesystem unit tests initially met a read-only sibling-checkout sandbox.
   The scoped approved rerun passed; the first failure is environmental evidence,
   not an omitted test group.
2. One historical test incorrectly labelled image 19's Undo All as enabled.
   Exact native inspection corrected it: 01–18 enabled, 19/20 disabled.
3. The new board exposed missing native glyph variants. Measured examples now
   supply 33 rank and 35 suit variants; unknowns were never filled by elimination.
4. LEFT K diamonds exposed adjacent black King artwork in the old 18-high suit
   crop. The measured 16-high crop excludes that artwork. XOR caps remain 24/8,
   alignment remains one pixel and mixed-colour refusal remains strict.
5. The runtime modal panel patch now uses the opaque interior, not a rounded
   corner containing variable backdrop pixels. Both calibration and actual
   bundled-profile tests require every modal component and permit independent
   underlying-card changes without borrowing their identities.

Native source/provenance details and coverage gaps are in
`tests/fixtures/pyramid-recon/README.md` and its manifests. First-failure and
reference-study logs remain under `/tmp/pyramid-recon-native/`.

## Executable and profile identity

Host release executable:
`target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`

SHA-256:
`84c68c76e9ef518ef36a79e5e036163092394bc9b9ca92b07694ea4f1c51540d`

Explicit runtime profile:
`tests/fixtures/pyramid-recon/native-profile.json`

SHA-256 after the Gate 3 stable-text profile correction:
`796f3debfa098715fac781f4aa69de1dc4eaefb91af5ca996db7dc8e12474577`

Close the old controller normally before deliberately launching this test binary.
Use [the live-test brief](pyramid-recon-live-test.md), an untouched Solver-off
Pyramid board and a fresh output filename. No complete live deal JSON is claimed
yet. A native fixture endpoint does not establish all intervening stock identities
or restoration. Unknown future glyphs stop rather than guess.

## Promotion text and paths

These are proposed messages for Charlie after acceptance, not performed commits.
Promote xSAR first; hold the application until exact repinning and revalidation.

xSAR:

```text
Add checked Pyramid deals, JSON codec and atomic publication

Introduce optional card and Pyramid models with strict schema-1 validation.
Add bounded JSON reading and complete restored-deal serialisation.
Publish through private same-directory files without overwriting destinations.
Preserve publication uncertainty and cleanup errors; retain lean default features.
Cover model, codec, concurrent publication and malformed-input boundaries.
```

Application, after repinning:

```text
Add guarded Pyramid reconnaissance and restored-deal JSON export

Recognise native tableau and pile rank/suit evidence with explicit refusal.
Acquire ordered stock using fresh advancement and bounded input authority.
Verify Undo All confirmation and original-board restoration before JSON export.
Retain STOP/context revocation, original PNG diagnostics and live previews.
Integrate the promoted xSAR model/codec/publisher without changing CV policies.
```

Complete explicit changed-path lists and final file hashes are retained as
`promotion-xsar-paths.txt`, `promotion-app-paths.txt` and
`delivery-source-manifest.json` in the evidence root. They exclude ignored local
instructions, workspace files, binaries, session logs and the temporary override.
Review the listed paths before staging; never use broad `git add .` here. The
application lock/manifest in that list still require the promotion step above.

## Code Analysis

Untrusted inputs are bounded profile/JSON data, captured frames, QMP responses
and paths. Safe Rust and xSAR checked geometry/frame/model APIs validate these
boundaries; no unsafe Rust, host-wide input or duplicate QMP transport is added.
The scan permits at most 24 D, one Undo All, one OK and 180 captures, with six
observations per unresolved stage and a 300-second cooperative deadline. STOP
cannot retract delivered input or instantly cancel blocking I/O. Each retained
PNG has the existing 32 MiB cap; only baseline/latest originals are persisted.
Incomplete diagnostics cannot construct a CompleteDeal. Publication is atomic
and no-clobber on the supported Unix filesystem contract, not a hostile-path or
unconditional power-loss guarantee. Live acquisition and restoration remain the
human acceptance boundary.
