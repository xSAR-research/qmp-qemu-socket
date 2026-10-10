# v2.0.11 — shared two-frame RGB measurement

## Status and scope

Gate 2 development against accepted app
`e59e06c49f7112b2dac1a7634999ae6f6fcd7344` and xSAR
`1a719b359a51d1e1e3113224193a779c76de82f8`. Package versions are 2.0.11 and
0.2.1 respectively. Charlie has promoted the crate and reports a successful
local-development run. Final Git-pinned consumer acceptance and application
promotion remain pending; see the separate evidence records below.

The worker's `materially_changed_pixels` and Pyramid's `changed_pile_interior`
now consume `xsar::image_matching::comparison::count_rgb_changes`. Both keep
their application-facing signatures and checked-convert exact u64 counts to
usize. Malformed evidence returns errors instead of a zero count.

The crate validates independent layouts through `FrameView`, equal dimensions,
non-empty in-frame half-open ROIs, and every exclusion before scanning. Valid
exclusions may extend outside the ROI but not outside the frame; their ROI
intersections form a union. RGB delta uses inclusive any-channel comparison;
alpha and row padding are ignored. Threshold zero counts all unexcluded pixels.
The new measurement allocates nothing, performs no I/O and grants no authority.

The worker retains its supplied mask, threshold and effect floor. Pyramid
retains the 12-pixel inset, threshold 20, effect floor 128 and three optional
cursor sources. Only Left and Right yield rectangles: Move is D, not a click.
Inset arithmetic is checked before constructing the ROI. Existing occupancy,
paired-pile, repeated fresh-pair, redeal and completion decisions are unchanged.

Five-game action policies, STOP/context checks, preview publication, capture and
transport are unchanged. No normal card-effect gate is added to Klondike,
Free Cell or Spider. TriPeaks retains its 30 acquisition/four confirmation
budgets and guarded terminal ordering. Spider XP, card/JSON models,
reconnaissance, the console and search are deferred.

## Promoted dependency and remaining boundary

Charlie committed and pushed xSAR at
`987856a0b5cb95aec2788551e050e18f7469b1ef`. Local inspection confirms that exact
HEAD and a clean crate checkout. Cargo subsequently fetched the revision over
HTTPS; every tracked fetched file matches the local promoted crate. The actual
package version remains 0.2.1. The user's commit subject begins `v0.03.0`; a
commit subject does not alter Cargo metadata, and the commit was not amended.

The application manifest and lockfile now select this exact Git revision with
defaults off and the existing qmp/png/image_matching features. The former
`path = "../xsar"` dependency is no longer the release specification. The
sibling-path and isolated paired builds below remain historical evidence.
No unrelated lockfile package changed during this source-only replacement.

Consumer validation has been repeated against the fetched Git source. Obtain
acceptance of that executable before promoting the application. No application
commit, push, registry publication, installation or live guest operation was
performed by the agent. Foundation implementation waits for RGB promotion.

## Initial paired validation evidence

Validated on 10 October 2026 on the native x86_64-unknown-linux-gnu host:

- rustc 1.101.0-nightly (`a30aa9064`, 2026-10-08), LLVM 23.1.3.
- Cargo 1.101.0-nightly (`29c5daa1a`, 2026-10-07).
- Clippy 0.1.101 and rustfmt 1.11.0-nightly from the same compiler toolchain.
- xSAR's unmodified host default is Rust/Cargo 1.99.0; checks explicitly selected
  installed nightly without changing that default or the declared MSRV.

Every Cargo validation command used `--locked --offline`. The initial paired
build used an unlocked, targeted xSAR patch update for dependency resolution.
Lockfile comparison confirms no unrelated dependency/version changes. Cargo's
native-platform metadata confirms the paired app consumes local xSAR 0.2.1
with exactly `image_matching`, `png` and `qmp` enabled.

| xSAR feature configuration | Unit tests passed | Doc tests passed |
| --- | ---: | ---: |
| Default | 14 | 0 |
| No default features | 14 | 0 |
| image_matching | 29 | 1 |
| qmp | 23 | 0 |
| png | 31 | 1 |
| qmp + image_matching | 38 | 1 |
| All features | 40 | 1 |

All matrix runs had zero failures and zero ignored tests. The all-feature
check, strict private rustdoc, all-target Clippy with warnings denied and
release build passed. Default and image-only dependency trees contain only
xSAR itself. Ten new crate unit tests and one public API example cover the
comparison contract without application fixtures or input authority.

| Paired application check | Result |
| --- | --- |
| Focused worker comparison | 4 passed |
| Focused Pyramid comparison | 2 passed |
| Ordinary full suite | 586 passed, 0 failed, 26 ignored |
| Full suite with include-ignored | 612 passed, 0 failed, 0 ignored |
| Cargo check | Passed |
| Strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` |
| All-target Clippy | Passed with `-D warnings` |
| Release build | Passed |

Formatting checks passed for the changed application line ranges using its
existing formatter configuration, and for the new crate module with the existing
two-blank-line convention. This is scoped formatting evidence, not a claim that
a whole-repository formatter check passed. `git diff --check` passed. The
remaining legacy source was not reformatted.

The tested application Rust source files were byte-compared against the actual
checkout. The paired release is an 11,171,040-byte native x86-64 ELF executable:

```text
SHA-256 f50f0d2ded70c1cacafe36af393ad6c1ce4a6c3d9acc0b64f2e582114cc31c2d
```

This checksum identifies the paired development build only. Recompute it after
the final promoted dependency pin and rebuild. The executable was not launched
against the live guest, and no accepted installed executable was replaced.

Two tooling failures were retained and diagnosed before proceeding. An initial
all-platform offline metadata query requested uncached Android dependencies;
the native-platform query passed with the same locked/offline constraints.
Clippy's first invocation placed the patch configuration before the external
subcommand, so its Cargo subprocess did not receive it and refused the lockfile.
Passing `--config` after `clippy` passed without changing the lockfile, source or
warning policy. Neither failure was hidden by a dependency update or relaxed
validation flag. Full command logs and first-attempt evidence are retained in
the private local verification directory reported at handover.

## Historical sibling-checkout validation

After selecting `../xsar`, validation was repeated on 10 October 2026 directly
in `/home/charlie/repo/RUST/qmp-qemu-socket`, consuming
`/home/charlie/repo/RUST/xsar/Cargo.toml`. Native-platform Cargo metadata confirms
xSAR 0.2.1 from that path with exactly `image_matching`, `png` and `qmp` enabled.
No source override or copied workspace was used. Build output remained under
`/tmp`; changing the output directory does not replace the source dependency.

The targeted `cargo update --offline -p xsar` changed only xSAR's source and
version relative to the preceding development lockfile. All subsequent Cargo
validation commands used `--locked --offline` and the installed nightly above.

| Actual-checkout check | Result |
| --- | --- |
| Application check | Passed |
| Application full suite with include-ignored | 612 passed, 0 failed, 0 ignored |
| Application strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` |
| Application all-target Clippy | Passed with `-D warnings` |
| Application release build | Passed |
| xSAR all-feature tests | 40 unit tests and 1 doc test passed |

Both Git indexes, the unrelated workspace-file edit, the three affected
application Rust files and the crate comparison module retained their recorded
checksums throughout this dependency-wiring correction. No further game-policy
or comparison implementation changes were made. The earlier feature matrix and
scoped formatting evidence remain separate from these actual-checkout rechecks.

Backups, complete command logs, metadata, result records and a private release
copy are retained at `/tmp/xsar-local-integration.Yhru28i6`. The new native
x86-64 release executable is 11,171,040 bytes:

```text
SHA-256 f7e7407d37a7ad4e11a09c6faa2d6eec586f7cee2b5e394b26e6faff08b8c3c3
```

This identifies the sibling-checkout development build. The agent did not launch
it or replace an installed executable. Charlie subsequently reported running it
successfully and supplied process-verification output: PID 2699501, the private
executable path above, and `/proc/2699501/exe: OK` for this SHA-256. This is
user-supplied host evidence, not an independently observed five-game test matrix.

## Promoted Git consumer validation

On 10 October 2026, Cargo metadata confirms xSAR 0.2.1 from
`git+https://github.com/xsar-research/xsar?rev=987856a0b5cb95aec2788551e050e18f7469b1ef#987856a0b5cb95aec2788551e050e18f7469b1ef`
with exactly image_matching/png/qmp enabled. All tracked crate files match the
promoted source, including the previously feature-tested comparison module.
The application Rust files, both Git indexes and unrelated workspace edit
retained their pre-pin checksums. No functional source changed in this step.

Using the same installed nightly and native host as above, all validation
commands used `--locked --offline` after the one required HTTPS resolution:

| Git-pinned application check | Result |
| --- | --- |
| Cargo check | Passed |
| Full suite with include-ignored | 612 passed, 0 failed, 0 ignored |
| Strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` |
| All-target Clippy | Passed with `-D warnings` |
| Release build | Passed |

The crate feature matrix is the earlier recorded result, not a new matrix run.
Cargo's JSON compiler-artifact output selected the release executable. A private
copy, metadata, backups and complete logs are retained at
`/tmp/xsar-promoted-consumer.FCoG2QWw`. The executable is 11,171,088 bytes:

```text
SHA-256 d067071d52e59b26bb3801ebfe633a8f25ed05beb1e62715c9c35fb78b7841c9
```

Its path is `/tmp/xsar-promoted-consumer.FCoG2QWw/qmp-qemu-socket`.
It has not been launched by the agent or installed over the accepted release.
The different hash identifies a different build; the earlier process check does
not verify this executable or replace an already-running process.

First-attempt evidence is retained. `cargo update -p xsar` could not match the
old path package after the manifest source change. A native-platform metadata
resolution then reached the new Git source but was denied writing Cargo's
read-only cache in the sandbox. The same resolution with explicit approval
fetched the exact revision and added only that one lockfile source. No broad
dependency update or relaxed validation was used.

## Human verification

Charlie's successful sibling-build run and executable verification are recorded
above. Detailed per-game, STOP/context and unexercised-branch results were not
supplied; do not invent them. Check the final Git-pinned release's version/checksum
and the actual launched executable before acceptance. Verify affected
Pyramid/TriPeaks paths, all five accepted game
policies, STOP and context invalidation. Retain first-failure captures and logs;
never replay uncertain input. Automated mock-socket tests do not establish
live guest timing or gameplay acceptance.

## Promotion preparation

Completed xSAR promotion: `987856a0b5cb95aec2788551e050e18f7469b1ef`.
Actual user commit title:
`v0.03.0 Stage 1 of implementation of geometry analysis and telemetry processing`.
This promotes the RGB slice, not the separately approved card/JSON foundation.

Complete xSAR paths: `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`,
`src/image_matching.rs`, `src/image_matching/comparison.rs`.

Proposed application commit title: `Reuse xSAR RGB measurements in worker and Pyramid`.
Body: Replace both local counting loops with the shared comparison primitive,
retaining calibrated masks, thresholds and all five game policies. Add adapter
regressions, pin promoted xSAR 0.2.1 at 987856a0b5cb95aec2788551e050e18f7469b1ef,
and document locked consumer validation and human verification boundaries.

Complete application paths: `Cargo.toml`, `Cargo.lock`, `README.md`,
`docs/development.md`, `docs/release-v2.0.11.md`, `src/worker.rs`,
`src/worker/tests.rs`, `src/pyramid.rs`.

These are review lists, not authorisation to stage or commit. The exact pin and
lock update are complete; final consumer acceptance is still required. Keep
AGENTS files, the handoff packet, workspace files, temporary configuration,
pairing lockfiles and `docs/pyramid-foundation-gate1.md` outside RGB staging.
No source files are deleted. Charlie performs the application commit/push unless
he explicitly delegates them.
