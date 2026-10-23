# Verification — app 2.8.121 operator-controlled Pyramid search

App **1272 passed, 0 failed, 0 ignored, 0 filtered** in the required full native include-ignored scope (328.903 seconds). Locked native check, strict private rustdoc, Clippy with warnings denied, release, six scoped Rust formatting checks and native/protected headless help passed. Unchanged xSAR **0.8.104** and console **0.8.4** retain their qualified 289/80 tests and six feature rows 14/55/25/67/57/99; they were not rerun.

Application Pyramid planning now has **no automatic state, expansion, memory-byte or time cutoff**. It continues until a route/proof, operator STOP, a context cancellation or an actual checked allocation/arithmetic failure. The shared-library defaults, guest execution guards and input authority are unchanged. Live status reports elapsed time and actual search counters and offers STOP. STOP withholds executable actions; interruption is never reported as proof that a winning route cannot exist.

The former failure took 2.105776793 seconds to hit 2,000,000 retained histories; 2.1 seconds was not a configured timer. The retained native deal is winnable: Maximum card matches returns 40 logical actions (25 MATCH/KING operations, 14 DRAW and one RECYCLE), independently replayed to all 28 tableau cards removed. Exact match-cost search has been tested continuing beyond the old history cap before an explicit test STOP. That test proves continuation and cancellation, not completion or minimum match cost for this deal.

## Scope and behaviour

Only the application policy, progress/diagnostic text, meaningful regressions, version metadata and review documents changed. Six Rust files are explicitly scoped. No dependencies, unsafe code, generic search APIs/defaults, recogniser calibration, draw pacing or guest execution bounds changed. Maximum card matches preserves logical-action win-first guidance and tableau-removal fallback. Shortest route preserves exact MATCH/KING cost with free DRAW/RECYCLE; its successful proof remains a minimum of that primary cost, not minimum total input count.

The application represents omitted allowances by the largest values accepted by the unchanged search API and emits JSON null for all four allowances. Checked allocation and arithmetic remain real failure modes. Finite allowances supplied explicitly to the policy diagnostics remain literal.

## Meaningful regressions and evidence

- An actual 2,000,001-edge search reaches its goal beyond both former state and expansion limits.
- Progress-triggered STOP returns cancellation without an executable route.
- The retained native 52-card deal yields a full 40-action Maximum card matches route and independent clear replay.
- The same native deal's exact search continues beyond two million histories before explicit test STOP; no exact solution is claimed.
- Interrupted and exhaustive-no-win diagnostics require different evidence.
- The complete former native cutoff packet and Charlie's last-match screenshot are retained byte-for-byte with provenance in [pyramid-search-stop](../tests/fixtures/pyramid-search-stop/README.md). Fixed manual coordinates remain one-based row|column; the screenshot shows DL 6C and apex 7D at 1|1.

The actual focused debug run was 6 passing policy/native-deal tests. Its measured debug timings are not substituted for release timing. The full scope above includes the new diagnostic regression. The original test compile failure is retained at /home/charlie/Documents/Pyramid/Validation/search-stop-only-20261021/focused-first-failure.log.

## Commands, preservation and protected artifact

Exact argv, UTC timestamps, input maps, durations and log hashes: /home/charlie/Documents/Pyramid/Validation/search-stop-only-20261021/qualification.json and runs/0001/commands.json. Commands use the owning package, locked/offline native target, all features/targets where applicable, full include-ignored tests with eight threads, private rustdoc with -D warnings -D missing_docs, Clippy -D warnings, release and scoped formatting. No unchanged Cargo gate was repeated after success.

Pre-edit hashes, modes, symlinks and all three Git identities/indices: /home/charlie/Documents/Pyramid/Validation/search-stop-only-20261021/baseline.json. The explicit review set and independently checked final preservation are review-paths.json and final-audit.json in that directory. Authentic zero-match Tree Ring preflight receipt/SQLite sidecars are separately enumerated in runtime-receipts.json; they are not implementation changes or durable memory captures. Real xSAR and console files, all older fixtures, unrelated dirty work and protected 107–120 releases remain preserved.

Protected executable: /home/charlie/Documents/Pyramid/Validation/search-stop-only-20261021/release/qmp-qemu-socket-2.8.121

Size 13459968 bytes; mode 0555; SHA-256 53c8c8b1c2a1484e0119a1460dd67c400646581cdb467f7e76ae9408e1df8eef. It is byte-identical to the selected native Cargo executable. protected-artifact.json records headless help and dated read-only process identity. A build does not replace a running process.

No 121 guest trial, controller replacement, installation, Git operation, upload or thread transfer was performed. Current live acceptance is [Live-Test-2.8.121.md](Live-Test-2.8.121.md); [120 verification](Verification-2.8.120.md) retains the unchanged native-v18 final-effect qualification. M10 remains Awaiting verification and M11 Planned. The separate TCP capture cancellation in the first 120 trials has no established cause and is not attributed to operator STOP.
