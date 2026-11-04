# Verification — 2.9.178

## Problem and resulting behaviour

The saved Coral Five of Diamonds at fixed 7|5 (zero-based slot25) had an accepted
Five rank but suit error16292/80964=201225.236ppm, above the unchanged200000 limit.
The same board failed initial entry and later input-free transition observation.
Restoring the removed pre176 Four of Diamonds would not repair this refusal.

One bounded Underwater Diamonds suit supplement is decoded from the byte-original retained Five of Diamonds at 6|5. All thirteen original rank corners and every previous suit variant remain unchanged. The supplement must already match the complete original family as Five and Diamonds before only its suit glyph is appended. Quality, gap, colour, geometry and ambiguity rules remain unchanged.

The optional suit_variants list permits zero or one supplement. Incorrect types,
over-budget entries, wrong metadata/raster rank or suit, incomplete/black/mixed
corners, invalid RGB and duplicate suit glyphs are rejected before append.
There is no global fallback, per-pixel reference mixing or tolerance increase.

## Exact retained provenance

The supplement is all4557RGB bytes from
[original158 underwater.png](../tests/fixtures/pyramid-recognition-158/underwater.png),
fixed6|5/native origin1191,378, crop[1194,1225) x [382,431).
Original PNG SHA256013bce0fe1a82109aeb38edb30f6d1fb739ef0c33f5c4dd447093e9a9a972821;
crop SHA25614c195be7c32e4fe7a4b576379cb54f9c14fd252fc06b3e525b72cbff4718721.
All original asset fields and thirteen corners are unchanged. Tests assert exact
rank-bank equality and each previous suit vector as an unchanged prefix.

[Fixture metadata](../tests/fixtures/pyramid-recognition-178/calibration.json)
records both byte-original saved frames and all manual labels. Both complete
boards pass actual v23 recognition and production Native/paired_entry as Boards1/2,
LEFT King of Spades and RIGHT empty. The failed suit now scores8416/76288=110318.792ppm.
The initial two new whole-board tests failed on the original calibration before
this correction; red output is retained. Six new tests and31 retained focused
tests pass, including constructor/provenance and damaged/mixed/clipped negatives.

## Host qualification

| Gate | Result | Elapsed |
| --- | --- | --- |
| scoped-format | PASS | 0.15 s |
| check | PASS | 4.29 s |
| rustdoc | PASS | 2.05 s |
| clippy | PASS | 12.44 s |
| full-tests | PASS | 151.28 s |
| release | PASS | 53.20 s |

1703 tests passed; zero failed, ignored, measured or filtered. Commands explicitly
use nightly, package qmp-qemu-socket, locked/offline native x86_64, all features,
and include ignored tests with eight threads. Strict private rustdoc and Clippy
warnings are errors. [Source receipts](../Evidence/Pyramid/Validation/pyramid-diamond-178-20261104-hu88ojh8/qualification-app.json) bind
exact argv, cwd, environment, log hashes and952input seal.

[Aligned receipts](../Evidence/Pyramid/Validation/pyramid-diamond-178-20261104-hu88ojh8/aligned-qualification.json) prove locked check,
release and protected help after only own manifest/lock177-to178 changes;
950 other inputs remain identical. User profiles, Wayland/znver4/hardening and
xSAR0.9.1/56inputs remain unchanged. Cached library qualification is16gates,
303full/14default tests and two Miri checks, not a new library run.

[Independent review](../Evidence/Pyramid/Validation/pyramid-diamond-178-20261104-hu88ojh8/independent-review.json) found no correctness
issue in the exact frozen nine files and calibration provenance. That static
review supplements root-owned Rust validation and is not fresh guest evidence.

## Preservation and limits

Published app HEAD390310066c6b394ac329b6372c11b189a0404517, parent8ea6d5c559c2b4d6d1c96c1dcf83ed128e778e53, and all four histories/indices/unrelated
changes are preserved. The existing server Cargo.lock change is preserved.
All1574prior evidence pins and136document preimages are guarded before writes.
The old published-HEAD sentinel failure stays historical; bounded automatic
receipt/sidecar reconciliation is recorded in the baseline. No broad metadata
ignore or archive rehash was introduced. The editable work-list is untouched.
[Owned paths](../Evidence/Pyramid/Validation/pyramid-diamond-178-20261104-hu88ojh8/owned-paths.txt) lists all18 promotion paths.

No new guest capture/input, installation, staging, commit, push or sibling edit
was performed. Fresh full RECON, restoration, continuous play and guest acceptance
remain unverified. Unsupported scenes still revoke input authority.

## Executable

    /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-diamond-178-20261104-hu88ojh8/native/qmp-qemu-socket-2.9.178 --image

SHA256 81a7cd4ce47ad5be2698eb7a98aff05e89bf5402810e92d3065680850b887cab; 12581648 bytes; mode0700.
