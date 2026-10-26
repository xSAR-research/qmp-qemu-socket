# Milestone Tracker & Baseline Receipt — `qmp-qemu-socket`

This file (`docs/milestones.md`) is the authoritative repository-local state receipt for the `Session-Start MCP, Memory & Baseline Gate` (`~/.codex/AGENTS.md`) and `qmp-qemu-socket/AGENTS.md`.

**Session-Start Evaluation Rule:**
1. Run the Destination Sentinel in `docs/handoff.md` via `ctx_execute`, and inspect `git rev-parse HEAD`, `git status --porcelain`, and sibling `../xsar` state.
2. Treat baseline compilation and the 1,522-test working-candidate suite as **PASSED (Cached)** when either:
- **Pre-Promotion Qualified Working Tree:** `HEAD` is `23016ba497f543a5aeee7bf97fe00e0a3c826b28` with the qualified, aligned153 release (manifest 153) and retained Track A acceptance (`../xsar` at `e50f7b3222fbd803d9eeecd8a2a37ba6fec856ca` with `10` porcelain lines) and `docs/handoff.md` reports `HANDOFF VERIFIED`, **OR**
- **Post-Promotion Clean Tree:** `git status --porcelain` is empty (`0` lines) and `Baseline_HEAD_SHA` matches `HEAD` or `HEAD~1`, **OR**
- **Approved Paired Promotion with Preserved Local Files:** the exact Destination Sentinel reports `HANDOFF VERIFIED` against the post-promotion state. It binds both published revisions, empty indices, complete porcelain state and all 782 unchanged compiler inputs. Unrelated `.gitignore` and local agent/memory/search files remain preserved outside the commits.
3. Do not run baseline `cargo check`, `cargo test`, `clippy`, `rustdoc`, or `release` builds before starting new work-product when `PASSED (Cached)` holds. Preserve all unrelated changes, calibration, evidence, and the Git index.

---

## 1. Active Baseline Receipt

| Field | Value | Notes |
| :--- | :--- | :--- |
| Package | qmp-qemu-socket 2.9.153 | Edition 2024 / MSRV 1.101.0 / native nightly |
| Current Manifest Version | 2.9.153 | Explicit approval m11412; alignment after qualification |
| Provisional target | None | No next increment authorised |
| Active state | 153 complete normal-header appearance correction host-qualified | Fresh153 trial pending; accepted Track A/Track B retained |
| Baseline_HEAD_SHA | 23016ba497f543a5aeee7bf97fe00e0a3c826b28 | Qualified pre-promotion base; exact published revision and preserved local state are bound by the Destination Sentinel |
| xSAR | 0.9.0 @ d20dca8ad2320dcbbf869e22b35a2583317d4569 | Published first with Good ED25519-SK signature; origin/main matches; all52 inputs unchanged;296 tests / 3 Miri cached |
| Host qualification | 1522 pass / 0 failed,ignored,filtered / 782 seals / six gates | Metadata-only aligned check/release/help then passed |
| Protected executable | /home/charlie/Documents/Pyramid/Validation/pyramid-normal-entry-20261026/native/qmp-qemu-socket-2.9.153 | SHA-256 8f7297cae0cec014fbaec6f116b867d5460ddca1a5bf88c148761b0010833cbc |
| Current state/sentinel | /home/charlie/Documents/Pyramid/Validation/pyramid-normal-entry-20261026/destination-state.json | Exact command in [handoff](handoff.md) |
| Acceptance | [Full25-case checklist](Acceptance-Tests-2.9.151.md) | All13 standing outcomes plus TracksA/B; no synthetic live passes |


### 2.9.153 approved normal-entry correction

2.9.153 is host qualified under Charlie's explicit approval m11412: 1522 tests passed, zero failed/ignored/filtered, 782 sealed inputs and six gates passed. Only the two own-package version fields then changed; aligned locked native check, release and help passed. The other 780 compiler inputs and all 52 xSAR inputs remain identical. xSAR 0.9.0 / 296 tests / 3 Miri remain cached. Protected executable: /home/charlie/Documents/Pyramid/Validation/pyramid-normal-entry-20261026/native/qmp-qemu-socket-2.9.153; SHA-256 8f7297cae0cec014fbaec6f116b867d5460ddca1a5bf88c148761b0010833cbc. Fresh 153 guest acceptance remains separate. At host-qualification closeout no agent GUI launch, START, guest input, installation or publication had occurred. The current promotion authority and published dependency are recorded below.

The automatic NEW GAME and manual START failures shared a complete normal-header appearance refusal. Two pixels differed by 13 against the existing per-channel limit of 12; the complete New footer passed. Fresh TCP captures and complete card observations were verified. The correction retains an additional complete 160 by 32 ordinal-one RGB appearance and every pixel/channel check at the original limits. Coherent appearances for the same count share one identity; conflicting counts still refuse. The 151 bounded Pending/Ready pair, strict normal footer, exclusive piles, fresh entry and fresh guarded Recon remain unchanged. No timing, control, threshold, crop, glyph, dependency, API, concurrency or xSAR policy changes are introduced.

Four new regressions cover both actual refusals, same-count aliases/conflicting counts, missing full patches, production automatic Pending then Ready/Ready with exactly three expected control presses, and input-free manual entry with STOP/context revocation. The 94 original test bodies remain unchanged. [Verification](Verification-2.9.153.md) and [live brief](Live-Test-2.9.153.md) record scope and limits. Complete source/log/review/metadata/promotion/sentinel records are under /home/charlie/Documents/Pyramid/Validation/pyramid-normal-entry-20261026. Charlie subsequently authorised signed-push and publishing xSAR first. The consumer retains its existing relative path dependency and seven features, with the exact xSAR commit recorded above; this promotion changes no source or version. No additional increment is authorised.

## 2. Retained 2.9.149 Release Qualification

All six gates passed with 1483 tests and 753 seals. The 52 xSAR inputs remain
unchanged; its 296-test qualification is PASSED (Cached). The approved version-only
update then passed locked check and native release/help. See
[the full results and limits](Verification-2.9.149.md).

The independent aged-absence finding was repaired and the full suite rerun;
the earlier 1482-test pass does not qualify 149. The approved protected GUI launched
as PID 3675217 and its actual bytes/session were verified. No agent START,
guest input, installation, commit or push occurred.

---

## 3. Retained Qualification & Revision Ledger (`2.9.x` Series)

Each row of the master implementation schedule is an agreed **minor** milestone (`major.minor.0`). Verified code revisions within `2.9` increment `patch` by `1`. Track A acceptance is now closed with the open fixes below. The 19 matrix identities and 14 wider schedule rows retain their recorded execution evidence; this acceptance does not turn an unobserved or failed case into a test pass.

| Version | xSAR | Host Tests / Seals | Status | Scope & Verification Reference (in `docs/`) |
| :--- | :--- | :--- | :--- | :--- |
| **2.9.152** |0.9.0 |1518 tests /775 seals | HOST QUALIFIED; VERSION ALIGNED | Coherent final absence and lower-material veto ([verification](Verification-2.9.152.md)). |
| **2.9.151** |0.9.0 |1512 tests /773 seals | RETAINED HOST QUALIFICATION | Bounded post-Play settling/fresh pair ([verification](Verification-2.9.151.md)). |
| **2.9.150** | 0.9.0 | 1504 tests / 770 seals | ACCEPTED; VERSION ALIGNED | Code fixes and Track B; post-approval locked check/release/help passed ([verification](Verification-Track-B-150.md)). Fresh 150 guest verification is separate. |
| **2.9.149** | 0.9.0 | 1483 tests / 753 seals | ACCEPTED WITH OPEN FIXES | Bounded Wild West material/glyphs and aged absence veto ([verification](Verification-2.9.149.md)); current Track A acceptance in section 6. |
| **`2.9.148`** | `0.9.0` | `1478` tests / `747` seals | ACCEPTED IN TRACK A | Top-left heading, wrapping options, right-hand **SOLVING** badge, full-width controls/status/preview ([Verification-2.9.148.md](Verification-2.9.148.md), [Live-Test-2.9.148.md](Live-Test-2.9.148.md)). |
| **`2.9.147`** | `0.9.0` | `1476` tests / `747` seals | ACCEPTED IN TRACK A WITH OPEN FIXES | Receipt-bound normal Pyramid `GAME WIN` reward/counting → optional `LevelUp OK` → `New Game` → `Play` chain; bound challenge kind/target/Kings policy; cooperative `Switch to Maximum card matches` reusing authenticated RECON; verified zero artificial post-RECON delay ([Verification-2.9.147.md](Verification-2.9.147.md), [Live-Test-2.9.147.md](Live-Test-2.9.147.md)). |
| **`2.9.146`** | `0.9.0` | `1449` tests / `743` seals | `RETAINED EVIDENCE` | Bundled `native-v23` private `native_corner_ink_v1` rank/suit ink reader on white/cream cards (`11` reader tests, `87` visible cards, `423` Classic observations, `13` Retro exemplars / `16` held-outs) + fixed `240x80` activity badge ([Verification-2.9.146.md](Verification-2.9.146.md), [Live-Test-2.9.146.md](Live-Test-2.9.146.md)). |
| **`2.9.145`** | `0.9.0` | `1436` tests / `731` seals | `RETAINED EVIDENCE` | `DL`/`DR` stock emptiness via light-paper coverage (`139x187` footprint, RGB `>= 180`, `>= 45%` nonempty, `8.41` percentage-point margin above `53.410534%` selected `K♥` minimum) ([Verification-2.9.145.md](Verification-2.9.145.md), [Live-Test-2.9.145.md](Live-Test-2.9.145.md)). |
| **`2.9.144`** | `0.9.0` | `1434` tests / `729` seals | `RETAINED EVIDENCE` | Occupied-RIGHT temporal `REDEAL` correction and protected launch ([Verification-2.9.144.md](Verification-2.9.144.md)). |
| **`2.9.143`** | `0.9.0` | `1432` tests / `726` seals | `RETAINED EVIDENCE` | Bounded mixed-legend correction; run 9 stock-position refusal preserved separately ([Verification-2.9.143.md](Verification-2.9.143.md)). |
| **`2.9.139`** | `0.9.0` | `1420` tests / `710` seals (`296` xSAR) | `RETAINED EVIDENCE` | Cooperative `PAUSE`/`CONTINUE` across A* `RECON`/`SEARCH`/`GAMEPLAY`, retained popup geometry, `Continuous`/`Shortest route`/`Challenge` defaults, idle `RESET`, and `REDEAL` `48x63` @ `(936,792)` delta check ([Verification-2.9.139.md](Verification-2.9.139.md), [Live-Test-2.9.139.md](Live-Test-2.9.139.md)). |

---

## 4. Standing Required Outcomes Contract (preserved in 2.9.150)

Any subsequent edit must preserve these 13 baseline contracts from [Pyramid-Short-Term-Goal.md](Pyramid-Short-Term-Goal.md), with known exceptions carried in the FIX list below:
1. **Exhausted-LEFT Winning Route:** Exact usable `RIGHT`, both-empty pre-King state, final absence bands, and bound board/challenge handoff (`DR 5♥ + 8♠` at `2|2`, then `K♠` at `1|1`).
2. **Complete Preview Fit:** Fit in both dimensions with aspect ratio, native coordinate mapping, and overlays intact.
3. **Frozen Failure Diagnostics:** Retain failure-time error/config/run/version/profile/packet/log details, copy/selection, and reopening after `STOP`; stale receipts never overwrite newer failures.
4. **Live Independent LOG:** Keep `LOG` responsive and independent of worker I/O; preserve complete-log copy and idle clear gates.
5. **Panel Chrome & Controls:** Contrasting `PARAMS`/`LOG`/`ANALYSIS` chrome with collapse/maximise/restore/close; control order `START` before `RECON`, `LOG`/`ANALYSIS`, `PAUSE`/`CONTINUE`, `RESET`, `STOP`, and far-right `EXIT`.
6. **Defaults & Idle RESET:** Default to `Continuous` / `Shortest route` / `Challenge`. `RESET` is input-free and idle-only, retiring prepared/planning receipts, challenge binding, and queued challenge updates while preserving logs/diagnostics and `STOP`.
7. **Cooperative A* Pause/Continue:** Pause `RECON`, `SEARCH`, and `GAMEPLAY` only at safe checkpoints; exclude suspension time from deadlines/stats; reacquire evidence on resume; never park between `MATCH` clicks, between input and effect, or inside bounded restoration. `STOP`/context/`EXIT` always win.
8. **Native-v23 Identity & Stock Emptiness:** Stock uses below 45% supported material over 139×187 (bright plus bounded aged material). Aged material above the existing 5% allowance vetoes absence. Identity still requires unique full-family ink; custom/CV/HALO policy remains separate.
  9. **REDEAL Binding:** Bind to verified pre-input `48x63` region at `(936,792)` (`>= 512` pixels changing by inclusive RGB delta `>= 20`) before clicking `(960,823)` under full position/cycle/scene/probe/`STOP` guards.
  10. **Immutable Run Challenge Binding:** Lock challenge kind, target, and Kings policy at `START`/`RECON`; update progress only at safe boundaries via bound numeric reader with monotonic numerator and identical denominator.
  11. **Normal GAME WIN Handoff Chain:** Receipt-bound `reward/counting → optional LevelUp OK → New Game → Play` chain (including persistent reward after `OK`), requiring a fresh full-board pair before next A* `RECON` without activating guest Solver/HALO.
  12. **Cooperative Shortest-to-Maximum Switch:** Expose only during active Shortest calculation; reuse authenticated `RECON` deal/bytes without new capture; discard retired Shortest result; do not treat operator switch as exhaustive no-win.
  13. **Direct Completed RECON Handoff:** Preserve mandatory `1000 ms` `UndoAll`/`OK` settle and two restoration frames inside `RECON`; add zero artificial post-`RECON` delay.
  
  ---
  
## 5. Held Diagnostics & Open Items (Separate Scope)

Track A acceptance is closed by section 6. The bounded 149 aged-paper correction
is part of that accepted baseline; it does not establish arbitrary artwork support.
Charlie subsequently authorised Track B and the code fixes (m09727/m09728).
Football coverage, automatic clear-to-Recon handoff and the retained run 9 stock
boundary are covered by section 9; their original reports remain historical.
Boards/Perfect and the new 149 incidents are explicit FIX entries below.
The agreed implementation is host qualified and accepted in 150 (m10399).
Version alignment is complete; launch, installation, commit and push remain unauthorised.
  
## Approved149baseline reconciliation

Charlie approved149and baseline reconciliation (m08864,m08883). The additional
application goal and sibling milestone documents are preserved. All747qualified148
compiler inputs remain identical, so148qualification is retained as PASSED (Cached).
No baseline build or full test suite was repeated.

## Retained 149 receipt and separate improvement ideas

The protected repository-state-post-launch.json records exact HEAD/parent, porcelain bytes/hash,
index and diff hashes. The Destination Sentinel validates this preserved dirty
baseline. Before new work, 148 compilation/tests were PASSED (Cached), without
baseline reruns. The approval/reconciliation paragraph above is historical:
docs/goal.md subsequently became absent externally and has not been restored.

Future ideas remain separate: black Ten/additional singleton rank-colour held-outs,
and partial Western stroke negatives. Boards/Perfect is now covered by FIX-05.
The additional Western evidence ideas are not claimed complete or newly authorised.

## Actual GUI launch and logged trial

Charlie authorised the protected 149 GUI launch. Launched at 2026-10-25T13:42:43.037528+00:00,
PID 3675217; the stable process and running bytes were verified at
2026-10-25T13:44:58.091687+00:00. SHA-256: b1936a79c72c923023eacfc031858811179b5da9e6114c28ed18ec42700ef1d6.
Session: /home/charlie/tmp/qmp-qemu-socket-1792935763091.log.
No agent START or guest input was sent. Acceptance was pending at launch;
section 6 records Charlie's subsequent Track A acceptance.

Charlie then reported a separate background-theme entry refusal, explicitly
not Wild West, and requested logging only. The exact theme remains unclassified.
All three packet files and a session-prefix snapshot are preserved at
/home/charlie/Documents/Pyramid/Validation/background-theme-entry-149-20261025. No diagnosis or correction was performed for that report.

## 6. Track A acceptance and closeout

**CLOSED — ACCEPTED WITH OPEN FIXES.** Charlie accepted the outstanding Track A
items and directed newly discovered failures into the FIX list (m09592).
This closes the selected live-acceptance and promotion-readiness track in
[Codex-Goal.md](Codex-Goal.md). The decision covers the 147 functional checks,
148 presentation checks and verified provenance of the running 149 baseline.

The [acceptance record](Track-A-Acceptance-2.9.149.md) separates Charlie's decision
from saved log observations, cached host checks and unobserved live variants.
The failed normal run did not reach the complete GAME WIN chain. It remains a
known fix, rather than a newly claimed successful run. The old 22-item evidence
matrix is preserved; its pending/partial evidence labels are not rewritten as passes.

At the Track A closeout, versions were application **2.9.149** and xSAR **0.9.0**.
That closeout was documentation-only; current Track B qualification is in section 9.
M12 pacing and M13 naming remain separate.

## 7. FIX list — carried forward from accepted Track A

Implementation authority was subsequently supplied by m09727/m09728. The rows
record corrected or bounded host results accepted in 150 (m10399). Fresh 150
guest verification remains separate; original evidence is retained.

| ID | Failure and current finding | Evidence | Next scope |
| :--- | :--- | :--- | :--- |
| **FIX-01** | 149 normal Continuous board 2/3 stops after an acknowledged action: the unchanged Boards 2/3 header passes, but a flying-card shadow occludes the New footer, producing unknown progress and a fatal guard rejection. The saved run verified 20 actions and acknowledged 23 gestures. | [149 inspection](/home/charlie/Documents/Pyramid/Validation/normal-board2-header-149-20261026-wux200nf/inspection.json); related [145 footer evidence](Verification-2.9.146.md). | Implemented/host verified: typed Pending, no effect commitment or next input; same two-second window and strict source/contradiction/STOP/context/transport rules. |
| **FIX-02** | 149 Card challenge run 3 reaches the REDEAL stage but the measured region remains unchanged (0/3024 changed pixels); no click is authorised. The retained earlier slot-21 mismatch is not an established final cause. | [retained packet](/home/charlie/Documents/Pyramid/Validation/track-a-acceptance-20261026-z69tqa5a/evidence/pyramid-error-1792936148487-3675217-1); Track A saved session. | Bounded/host verified: actual no-widget/zero-deal endpoint is a partial stop only after independent exact exhausted-stock proof; otherwise the existing position refusal remains. No click, future-allowance or challenge-success claim. |
| **FIX-03** | 149 Score challenge run 4 expires its ordinary effect window after 56 verified actions and 75 acknowledged gestures. The saved first unmatched scene[0] reason is not a proven final root cause. | [retained packet](/home/charlie/Documents/Pyramid/Validation/track-a-acceptance-20261026-z69tqa5a/evidence/pyramid-error-1792937048206-3675217-2). | Implemented/host verified: saved frame is Challenge Complete/Loading at actual Score 2015/1900. A coherent bound appearance now retires card input without control authority. |
| **FIX-04** | Separate 149 entry refusals: the original non-Wild-West frame is Boards cleared 0/2 and its literal glyph fringe was misclassified as background. Two later normal frames have no ordinal and correctly withhold standard-game identity. They do not share an assumed theme or cause. | [original background report](/home/charlie/Documents/Pyramid/Validation/background-theme-entry-149-20261025); [later entry packets](/home/charlie/Documents/Pyramid/Validation/track-a-acceptance-20261026-z69tqa5a/followup-1/later-entry-refusals.json). | Implemented/bounded: original background Boards 0/2 uses existing literal foreground union with unchanged RGB/word limits. Two separate normal frames lack ordinal and remain unknown; no new input authority. |
| **FIX-05** | The retained 147 Boards/Perfect final-proof refusal: the bound counter advances 1/3 to 2/3 and final absence bands pass, but the exhausted LEFT-X exception invokes ordinary occupancy obscured by Perfect, then demands an absent DRAW scene. | [actual-reader confirmation](/home/charlie/Documents/Pyramid/Validation/boards-final-outcome-147-20261025/exact-reader-confirmation.json); [148 record](Verification-2.9.148.md). | Implemented/host verified: compose existing final absence proof with exact LEFT-X/RIGHT KD, without ordinary-tableau recheck or tolerance weakening. |
| **FIX-06** | Spider Uncertain stops reopened an old Klondike Failure Details record. | [saved report](/home/charlie/Documents/Pyramid/Validation/spider-game-win-149-20261025-l8afc0jx/inspection.json). | Implemented/host verified: contextual source-owned game/settings/error/worker times replace stale details. Original terminal-caption visual cause remains unproved; control refusal unchanged. |
| **FIX-07** |151 normal third-clear delivered 67 actions/86 ACK, verified 66 effects; Perfect crossed7|5 lower band. GAME WIN/XP skip never entered. | [saved originals](/home/charlie/Documents/Pyramid/Validation/normal-game-win-151-20261025-tzqc8voi); [reproduction/adapter proof](Verification-2.9.152.md). | Implemented/host verified152: coherent28upper bands plus original material veto; unchanged guards/piles/receipt. Fresh152 guest trial pending. |

## 8. Promotion readiness

Current review-only path lists and commit drafts are in
`/home/charlie/Documents/Pyramid/Validation/pyramid-track-b-fixes-20261025/promotion`.
They include the cumulative application work and the preserved nine xSAR paths.
The original Track A promotion drafts remain unchanged in its closeout bundle.
Promote xSAR before the application with the agreed consumer pin and required
revalidation. Acceptance and version alignment are complete; no signing, commit,
push, registry publication or installation is authorised here.

## 9. Code fixes and Track B — accepted and delivered in 2.9.150

Charlie selected implementation (m09727/m09728). B1 remains delivered in 149;
B2 football is covered by actual-reader regressions; B3 normal/challenge handoff and
FIX-01 through FIX-06 are implemented or bounded above. B4 retains run 9 occupied-RIGHT
and stock refusal contracts. See [qualification](Verification-Track-B-150.md) and
[live brief](Live-Test-Track-B-150.md). Charlie accepted the results and approved
2.9.150 (m10399). Manifest, lock entry, changelog and protected native release are
aligned. The post-alignment locked check, release and help passed; the implementation
retains the 1504-test qualification with 768 unchanged compiler inputs and only two
version-field changes. Track A stays closed. Host checks do not promote M12/M13
or rewrite the original case/schedule evidence. No launch/input/install/commit/push.


## 10. Normal GAME WIN readiness — 2.9.151

After GAME WIN / Play, supported dealing can remain Pending within the existing six-observation bound and stage interval. Pending clears pair history. Only two consecutive complete equal28-card tableaus, exclusive initial DL, positively empty DR and the complete normal-one header/footer permit the existing fresh entry and fresh guarded Recon. Every subsequent observation is newly captured; the caller may supply its latest actual first capture. The bundled pending-only surface branch keeps every static scene role and legend positive while the stock control is still absent. It cannot produce Ready or input authority. Recognition caps, full Ready scene, contradictions, malformed/unsupported evidence, STOP/context and no retired-click replay remain strict.

2.9.151 is host qualified under the explicit151 goal (m10618/m10619/m10655): 1512 tests, zero failed/ignored/filtered,773 sealed inputs and six gates passed. Only the two own-package version fields then changed; aligned locked native check, release and help passed. xSAR0.9.0/all 52 inputs are unchanged;296 tests / 3 Miri remain cached. Protected binary SHA-256: `3fb544a8a56f98395c89c974c189e33c8e8bdb4c02f52535c1eebc1fa8dd8ab6`. Fresh151 guest acceptance is pending. No agent GUI launch or guest input occurred.

[Verification](Verification-2.9.151.md), [live brief](Live-Test-2.9.151.md), [complete25-case checklist](Acceptance-Tests-2.9.151.md).

Both150 incidents used fresh TCP captures during dealing: session9/request479/frame7360 at+779.685s after Play+778.586s; session10/request488/frame7848 at+1277.125s after Play+1276.011s. The first readiness check aborted before Recon, with unknown rank/suit or incomplete-board rejection. Both unchanged original PNGs are retained; complete follow-ups in host tests are labelled compositions, not original animation traces.

The earlier150 Queens12/12 Challenge Complete timeout remains a separate undiagnosed case at `/home/charlie/Documents/Pyramid/Validation/normal-entry-150-20261025-h6hq6uan/earlier-route-packet`; checklistC08 includes it without claiming repair. Charlie corrected the earlier normal-entry attempt made while guest Solver was still selected; it is not another normal-entry defect.

## 11. Normal third-clear follow-up —2.9.152

2.9.152 is host qualified under the explicit 2.9.152 goal (m10915/m10916):1518 tests, zero failed/ignored/filtered, 775 sealed inputs and six gates passed. Only the two own-package version fields then changed; aligned locked native check, release and help passed. xSAR0.9.0/all 52 inputs are unchanged;296 tests /3 Miri remain cached. Protected binary SHA-256: `09de4002975c37d64f367f58a168b8eb05e80a8a490c3372462db2df9475bd50`. Fresh152 guest acceptance is pending. No agent GUI launch, guest input, installation or publication occurred.

The retained normal third-board clear was refused because moving Perfect glow crossed the lower absence band at 7|5. Bundled colour-independent normal recognition retains the original complete layout first, then may prove one coherent layout of all 28 existing upper bands. The alternative also vetoes card material in every original band and full clearance, preserving the three retained flying-card refusals. Replayed empty tableau, complete normal header/footer, scene/modal/DRAW roles, exact exclusive piles, final-action phase, whole-action ACK boundary, STOP/context and the same two-second window remain required. No per-slot mixing, threshold/crop/bank changes or ACK-only restart is admitted.

[Verification](Verification-2.9.152.md), [live brief](Live-Test-2.9.152.md), retained [complete 25-case checklist](Acceptance-Tests-2.9.151.md).

Original failed reproduction, exact source/review seals and protected151 are retained. Current sentinel/audit/promotion are linked from handoff.md; no original67-action route is invented.
