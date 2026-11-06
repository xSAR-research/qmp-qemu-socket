# Verification — 2.9.182 image-default CLI

GAME WIN image recording is now enabled by default. Run `cargo run --release` from the repository root; use `cargo run --release -- --no-image` to opt out. The former `--image` flag is rejected. Startup retains the existing fixed image-directory reset in both modes; help and invalid arguments return before that reset.

All six source gates passed, including 1725 tests with zero failures, ignored, measured or filtered tests. The approved version-only alignment then passed locked check, release and protected help. Of 961 sealed inputs, 959 remained identical after alignment; the 56 xSAR inputs are unchanged.

Charlie approved the milestone and version in message m10688. The package has one implicit `src/main.rs` executable, the repository nightly toolchain and default native target. From `/home/charlie/repo/RUST/qmp-qemu-socket`, run:

```sh
cargo run --release
```

To disable GAME WIN images:

```sh
cargo run --release -- --no-image
```

## Qualified executable

```sh
/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/native/qmp-qemu-socket-2.9.182
```

SHA-256 `b7c8d488029531c262d8be2c5f4bdcad90267c476c2c9a32cd0a941f3c76f01f`; 12587344 bytes; mode 0700. This protected release is distinct from the immutable preacceptance 2.9.181 image-default candidate.

## Validation and evidence boundary

Fifteen focused image-diagnostic tests passed. The full source suite included ignored tests and passed all 1725. Protected candidate checks verified `--help`, `--no-image --help`, and rejection of `--image` (exit 2), with unchanged `/home/charlie/tmp/images` metadata/content fingerprints after each bounded check. Aligned protected help also passed and preserved that tree. No GUI/application startup, guest input or new guest capture was performed. Fresh live acceptance remains pending. Existing worker tests preserve GAME WIN-only recording, original-frame identity, capture counts and STOP/error scope closure.

## Code Analysis

Only the existing startup image boolean default, opt-out argument, help and startup log changed. Image preparation, GAME WIN scope, transport, PNG caching, capture counts and failure-evidence paths retain their existing behaviour. No unsafe code, dependencies, threads, geometry or protocol changes were introduced. Default-on uses the existing enabled PNG/cache path and its existing filesystem failure handling. Help/invalid parsing precedes filesystem mutation. See [image policy](game-win-images.md) and [current QMP/capture guidance](qmp-and-capture.md).

## Reproducible records

- [Source qualification](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/qualification-app.json) and [source seal](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/qualification-inputs.json).
- [Approved alignment](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/aligned-qualification.json) and [approval](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/version-approval.json).
- [Implementation](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/IMPLEMENTATION.md) and [complete release-owned paths](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/release-owned-paths.txt).
- [Lifecycle audit](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/lifecycle-final-audit.json) and [destination state](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/image-default-182-_ipeg1nj/destination-state.json).

The old181 sentinel regression is retained. Known post-candidate automatic memory capture/recall artifacts are reconciled only by exact byte pins and paths; current main/sidecar payload copies are preserved. Historical logical database equivalence is not claimed. All four histories, indices, unrelated patches and prior evidence remain preserved. No installation, staging, commit or push occurred.

Suggested commit: `fix(images): enable GAME WIN recording by default`.
