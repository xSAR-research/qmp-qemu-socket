# Native-v12 graded-reader corpus and profile

App 2.6.110 is host verified after the complete 1,078-test app sequence.
This fixture records unchanged native PNGs and direct visible labels, not full
stock/restoration or current input authority. Source diagnostics remain original
and incomplete; no new rank/suit RGB template is added. Native-v11 schema 1 banks
remain literal; native-v12 explicitly selects schema 2 `paper_ink_l1_v2`.

Native-v12 selects strict schema 2 `paper_ink_l1_v2` with unchanged native-v11
RGB banks. Each actual/template ROI independently requires nonzero raw-paper
support of at least area/3 (all RGB channels >=180). Its paper reference `p` is
the lower median of those pixels' minRGB values, index `(count-1)/2`, 180..255;
no default reference is allowed. Every ROI pixel contributes
`floor(255*max(p-minRGB,0)/p)` ink; pixels brighter than paper contribute zero.
Error remains `sum(abs(actual-template))/sum(max(actual,template))`. Inclusive
rank 220,000/suit 120,000 ppm maxima and different-label gap 80,000 ppm/scale 1,000,000
are unchanged. Checked exact ratios decide; floored score ppm is diagnostic
only. Both absolute quality and separation are required; nearest label alone
never suffices. Exclusive strong colour and paper are co-located with that
label's winning one of nine ±1 shifts; rank/suit colours must agree/match suit.
No RGB sample, deck inference or input authority is added.

The initial raw-ink policy used 104 historical+29 ten-failure examples and failed
its first Rust filter: 8 passed/4 failed/0 ignored. The independently labelled
fifth K♦ also failed its initial prospective test at 123,012 ppm versus 120,000.
That exact failed policy and outcome are preserved; the fifth is now
`exposed_regression_fifth`, with no renewed prospective/untouched claim.
The revised paper-relative policy was frozen at 2026-10-19T13:02:56.014505+00:00
using 250 development+29 ten-regression observations; thresholds/gap and RGB
banks remain unchanged. Corpus totals stay 423 observations/40 sources/14 positions:
250 development, 29 ten-regression, 115 prior-exposed validation and 29 exposed
fifth observations. Capture/deal lineage stays whole; daily/v8 share one lineage.
The 115 validation observations retain prior calibration/regression exposure;
they are not untouched evidence. Focused graded/native/corpus results passed as recorded below; full checks passed as recorded below. Counts
are observations, not unique frames, decks or a general accuracy guarantee.

`corpus.json` holds hashes, direct labels, fixed original positions, source
lineage and exposure/split role. File paths resolve relative to this directory;
existing originals are referenced rather than duplicated. Only the ten/fifth
failure PNGs are new retained source files. Pixel transformations are none.
The selected-v8 position keeps 26 remaining tableau identities and two original
positive-empty slots; do not compact/renumber. Both runtime banks must cover
all 423 observations and all 14 full/partial positions. The current production tests passed those finite checks; full verification passed; live acceptance remains pending.

| Item | Exact identity |
| --- | --- |
| `profile.json` | 817,977 bytes; SHA-256 `f8f1bc8b8a7e28d0ad3433494c47096f5ca5e0b546275912d845adf6afdd0813` |
| `frozen-policy.json` | 1,730 bytes; SHA-256 `5343651cea940c25add3ee79653b2c9c9da257b30d9874143f6b400cf0d81236` |
| `corpus.json` | 311,406 bytes; SHA-256 `51db58fbb292d262d43a81ce1c2491b1c99bae7e1c5effe4c26767f696c61eaa` |
| `ten-failure-baseline.png` | 3,794,493 bytes; SHA-256 `beb2753ea79180a2d8d872b9089e061859f208d6307de79e621cea8358884126` |
| `fifth-failure-baseline.png` | 3,772,290 bytes; SHA-256 `81519aaa7ae9f8d47136af0868834368ecf40f49ff1fd546e929ff37dcd2b2c3` |
| `initial-policy.json` | 1,194 bytes/SHA-256 `d6c8a647946ae2898cb27068a88c5314d5f2550f80048fa7976b47312871b3e1`; rejected raw-ink policy |
| `initial-policy-results.json` | First 8 passed/4 failed/0 ignored and initial prospective K♦ refusal preserved; no renewed prospective claim |
| `provenance.json` | Direct label references, original manifest, frozen policy and per-file hashes |
| `ten-failure-INCOMPLETE.json`/`fifth-failure-INCOMPLETE.json` | Original zero-input/no-publication diagnostics; no restored receipt |

Fifth K♦ at zero-based tableau slot 9/fixed 4|4 is independently labelled. Its
legacy v11 nominal suit colour has 3 black pixels; upward 1 px removes that colour
contradiction but old shape XOR 10 still exceeds cap 8. Do not describe the legacy
reader as accepting it, alter the old fixture, or claim the new frozen outcome
before actual Rust measurement; the initial raw policy already failed its prospective test. Full ROI negative space, exclusive colour/paper at
the winning shift and strict label-gap bounds remain mandatory.

[Verification](../../../../docs/Verification-2.6.110.md) records actual Rust/
corpus/frozen results and separate them from review/exposed/holdout provenance.
[Profile contract](../../../../docs/pyramid-recon-profile.md) defines versioned
semantics. Offline recognition never proves unseen stock, completed acquisition,
restoration, live authority or guest effects. Manual/Single board, STOP/receipt
policy and unavailable native Continuous remain unchanged; the larger goal is blocked on missing native standard-three-board evidence.

Focused production tests now pass: **14 graded unit tests, 0 failed/ignored**
(compile 8.26 s,test 0.00 s), and **12 native-v12 tests, 0 failed/ignored**,
1,066 filtered (compile 0.09 s,test 60.10 s). They verify **846 bank/observation
decisions** across 423 directly labelled cards, 14 full/partial positions and 40
scene checks (36 boards/4 modals). These are finite exposed/development/prior-
exposed validation regressions, not a new prospective result. Initial raw-policy
8/4 failure and first prospective K♦ refusal remain retained. **Full 1,078/0/0 app tests and required host checks passed in attempt 0002;
protected stable 110/native parity/help passed. Live acceptance is still pending.**

Full host verification passed **1,078 tests, 0 failed/ignored**, plus required
check/private docs/Clippy/format/release/help. Actual compiler is
`1.101.0-nightly (b6eeb93c9 2026-10-18)`. Protected candidate
`/home/charlie/tmp/pyramid-reader-v12-validation-8jiy32rw/release/qmp-qemu-socket-2.6.110` is 12,722,096 bytes/mode 0555,
SHA-256 `dcdbda38b1d16e9bd99b2608f01a8a19b3c5f66fc9fde7f6cbece7a2f832058d`, native parity/help 0.
Read-only preflight at 2026-10-19T13:16:29.450099+00:00 observed no controller
and images zero files/bytes, instant-only. One correction build/no minor is
earned; siblings/stable 107/108/109 remain frozen. Root separately validated the
metadata-only initial-policy pointer correction; executable/profile/corpus/
scoring inputs are unchanged. Final documentation/preservation audit passed: 9 documents, 177 local links and
29 review paths, with the exact metadata-pointer exception separately checked.
No guest acceptance or completed native Continuous/larger goal is claimed.
