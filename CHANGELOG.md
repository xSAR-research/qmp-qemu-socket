# Changelog

## 2.9.134 — timing constants, 23 October 2026

- Centralise Recon, planned-route, stock-drain, final-confirmation, handoff, partial-REDEAL, repaint and reporting timings in `src/parameters.rs`. Alias shared xSAR QMP defaults and reuse animation defaults where purpose and value match. Add the [timing catalogue](docs/Timing-Constants.md).
- Preserve every timing value, fresh-effect proof and STOP/context rule. No tuning, new dependencies or unsafe code.
- Required host checks passed 1359 tests, with zero failures, ignored tests or filtered tests. No major/minor version change, installation or publication; native Windows/Beast acceptance remains separate.

## 2.9.133 — publication summary, 23 October 2026

- Deliver the accumulated native Pyramid workflow: restored 28-card tableau and
  24-card stock acquisition, paired DRAW feedback, Manual, Single board and
  Continuous operation, and separate normal-game, challenge-target and
  no-challenge stop policies.
- Add Shortest route using exact card-match costs and Maximum card matches using
  weighted search with full-clear preference and independently replayed tableau
  removal fallback. An exhaustive no-win result can switch objectives on the
  same authenticated Recon. Application search continues until completion,
  operator STOP or checked failure; shared-library defaults remain unchanged.
  Maximum scores tableau removals and does not promise stock-only King collection.
- Guard execution with single-use acquisition ownership, exact export/context
  checks, independent route replay, observed action effects and actual stock-pass
  accounting. Add native stock, REDEAL, final-clear and next-board evidence while
  preserving STOP, refusal and no uncertain-input replay.
- Show observed game/challenge progress separately from the selected policy.
  Record A* parameters, costs, limits, per-method statistics, fallback and results
  in both the visible log and session file. Include the measured empty-stock apex
  King and below-target Score board-clear corrections in this release.
- Qualification: 1,359 native tests passed with zero failed, ignored or filtered;
  locked check, strict private rustdoc, Clippy, release and scoped formatting
  passed. M01–M10 and Continuous — no challenge are accepted. M11, complete
  target-aware gameplay and M14 acceptance remain open; this commit is a host
  qualified development checkpoint while fresh Beast testing continues.
- Retain the dated release records, immutable native fixtures, fixture provenance,
  testing matrix and upstream attribution. Git publication does not imply crate
  publication, installation or acceptance of the remaining live cases.

## Current delivery — 2.9.133, host qualified; fresh Beast verification pending

App **2.9.133** passed **1359 tests, 0 failed, 0 ignored, 0 filtered** in the full native include-ignored suite (404.801s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer gates are retained by exact input/log hashes (289 shared, 80 console; feature rows14/55/25/67/57/99).

**Accepted:** M01–M10, including M10 guarded execution (m13464), and **Continuous — no challenge** (m13393). **M11, full target-aware completion and M14 acceptance remain open.** Host qualification is separate from fresh Beast acceptance.

The retained 2.9.132 session exposed two execution proof gaps. The apex K clubs at fixed **1|1** could not be clicked because the stock control had disappeared after both piles became empty. The bundled reader now recognises that measured whole absent-control/zero-deal appearance alongside positive empty piles, exact tableau/absence and all original static/modal/layout checks. This private execution alternative does not enable ordinary Recon, custom profiles or REDEAL.

The Score final frame shows an empty tableau and **2510/2900**, with a moving Perfect overlay. A final-only bundled alternative checks every pixel in all 28 upper felt bands, exact piles, static/modal/layout and the same bound incomplete challenge after replayed goal/final whole-action delivery. The older proof remains. This establishes a board clear; it does not establish a challenge win.

The UI now displays actual observed game/progress separately from selected mode and stop condition. Advisory observations come from existing captures or the authenticated restored Recon, with context/run/board filtering; they grant no input or completion authority. Generic Card ranks remain unidentified except positive Kings advice. The ordinary on-screen log is always available. Both screen and file logs receive actual A* parameters, costs, profile/rules, configured limits, per-method progress, fallback and proof/result. Progress is throttled to one ordinary row per second plus the final row; no search cutoff was added.

The reported target-aware Boards 2/3 failure could not be correlated to a failed request. Two fresh actual native 2/3 frames pass the entry adapter offline with no gameplay input; this is a regression check, not a claim that the unidentified report is fixed. Future trials can record both selected policy and observed facts. The retained session is safe to leave behind; no recreation, file move or old-board preservation is needed. Idle CPU and themes remain separate backlog.

[Verification](docs/Verification-2.9.133.md) · [Live test](docs/Live-Test-2.9.133.md) · [Selected goal](docs/Pyramid-Current-Goal.md) · [Testing matrix](docs/Pyramid-Testing-Matrix.md).

## Retained history — previous deliveries and reports

Earlier failures and pending statements below are historical. Use the current qualification and acceptance boundaries above.

## Current delivery — 2.9.132, host qualified; challenge REDEAL trial pending

App **2.9.132** passed **1351 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored suite (419.651s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer checks remain valid by exact input/log hashes (289 shared, 80 console; feature rows14/55/25/67/57/99).

**Accepted:** M10 guarded independent execution (Charlie, m13464) and **Continuous — no challenge** (m13393). **M11, the full target-aware goal and M14 acceptance remain open.** This correction is host qualified and requires a fresh Beast trial.

Two actual 2.9.131 target-aware Card runs verified every planned route action and reached two acknowledged RECYCLE operations, then refused the one-deal REDEAL. The Jacks 4/12 final frame has ten remaining tableau cards and RIGHT 9S; Eights 3/12 has six and RIGHT 4S. Both have the measured positive LEFT-X. The old complete icon and caption patterns failed small raster differences at unchanged tolerance 4. The slot 20/slot 1 absence messages were earlier retained mismatches, not the final failed control predicate.

The bundled continuation capability now retains three complete enabled-icon variants and four complete literal one-deal captions, each matched independently with all pixels at the existing bounds/tolerance. It never mixes pixels between templates. Exact tableau/positive absence/RIGHT, LEFT-X, dimensions/layout, static scene, modal refusal and zero-caption exclusion remain required. Intermediate empty-circle/MOVE proof still grants no REDEAL. Unknown, zero and unmeasured two-deal variants refuse. Custom profile identifiers cannot enable this private capability.

Final REDEAL refusal now reports same-frame LEFT-X, enabled-icon, one-caption and zero-caption results. The original first mismatch remains separately preserved. No extra capture, input, retry, cleanup, timing, search-policy, scoring or default change was added. Actual RECYCLE accounting, single click, post-probe STOP/context checks, fresh next-board Recon, target precedence and all existing modes remain.

Charlie can continue gameplay; the exact packets, matching Recon, exports and log prefixes are retained. Screenshots and tmp stay in place per m13480. Variable backgrounds and idle CPU remain separate backlog.

[Verification](docs/Verification-2.9.132.md) · [Live test](docs/Live-Test-2.9.132.md) · [Selected goal](docs/Pyramid-Current-Goal.md) · [Testing matrix](docs/Pyramid-Testing-Matrix.md).

## Retained history — previous deliveries and reports

Earlier failure/unfixed statements below are historical. Current host qualification does not replace fresh Beast acceptance.

## Current delivery — 2.9.131, host qualified; recovery verification pending

App **2.9.131** passed **1347 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored suite (411.666s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer checks are retained by exact input/log hashes (289 shared, 80 console; features 14/55/25/67/57/99).

**Accepted working:** Charlie confirmed the stop condition **Continuous — no challenge** in m13393. The [operator acceptance record](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/operator-acceptance.md>) is separate from host qualification. The new recovery change and the remaining detailed cases still require fresh verification; the entire activated goal is incomplete.

Optional Recon was displayed in Single board and Continuous, but its handler silently returned for every non-Manual mode. The corrected handler permits Recon in every idle Pyramid workflow mode. Manual Plan/Solve remain mode-restricted, and busy, STOP, context and ownership checks remain.

Each explicit Start single board or target-agnostic Start continuous now requests a fresh read-only frame within the existing worker reservation, before inspecting a retained acquisition. Other automatic entry paths already capture fresh frames. No separate UI capture command is chained to Start. The new frame supplies current pixels; it cannot authenticate an old route, grant input, or bypass supported-scene, restoration or effect verification. A valid same-board Recon can still be reused across settings/planning changes.

No separate stale-preview Start latch was demonstrated. Matching failed completion already retires the run, route and receipt, and Uncertain is idle. The concrete handler defect and the requested explicit-start capture are covered by regressions. Unsupported dialogs/scenes continue to refuse; fresh pixels do not dismiss them.

Existing stock-boundary proofs, Kings-only advisory defaults, target-aware completion, NoChallenge target opt-out, same-inventory fallback, three-pass stock accounting, guarded single REDEAL, bounded next-board settling, DL-first paired feedback and operator-controlled search are preserved. Variable backgrounds and idle CPU remain separate backlog.

[Verification](docs/Verification-2.9.131.md) · [Live test](docs/Live-Test-2.9.131.md) · [Selected goal](docs/Pyramid-Current-Goal.md) · [Testing matrix](docs/Pyramid-Testing-Matrix.md).

## Retained history — earlier deliveries

Earlier pending-status entries below are historical. The current operator acceptance above supersedes blanket pending wording for the no-challenge stop condition; other requirements retain their evidence boundaries.

## Current delivery — 2.9.130, host qualified; fresh Beast pending

App **2.9.130** passed **1344 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored suite (386.48s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer gates remain valid by exact input/log hashes (289 shared, 80 console; features 14/55/25/67/57/99).

The actual v2.9.129 failure followed all 32 verified partial-route actions. One RECYCLE had been acknowledged; the subsequent stock continuation stopped with DL positively empty and DR8S. The compact MOVE scene differed from ordinary MOVE, while the existing continuation alternative required the final X. The original first failure and settled frame are retained.

The new continuation-only alternative requires the measured compact MOVE in two adjacent complete patches, the existing positive empty circle, exact remaining tableau/absence/DR, supported static scene and modal/layout checks. It proves a position only and returns no REDEAL point. The final X/control/positive allowance gate remains separate. Two whole measured one-deal caption variants are accepted at the unchanged tolerance; zero/unknown captions grant no REDEAL. Stock passes remain initial plus two total RECYCLEs, accounting for those already acknowledged. A stale post-D effect stops without repeating input.

The challenge default is **Shortest route**, with **Maximum card matches** only for a positively identified **Kings Card challenge**. The exact restored PNG must provide both generic Card progress and the complete measured Kings label. Two retained native examples show 0/18 and 4/18. This advice is stored beside the private receipt and authenticated against context, authority and exact export bytes before use. It does not alter generic challenge identity, progress equality, completion, search scoring or route replay. Existing explicit Manual preferences are preserved within the same controller context; changing context retires the old override. Automatic first-board explicit choices are preserved; later boards retire that override before applying their independently observed default. Continuous — no challenge keeps its classification opt-out and Shortest reset.

The original full no-challenge / same-Recon fallback / remaining-stock / guarded-REDEAL objective remains incomplete pending fresh Beast acceptance. Full clear handoffs, bounded settling, two fresh starting witnesses, paired Recon with DL-first feedback, restoration and single-use execution remain. Target-aware Boards bypass partial matching; Card/Score targets retire input pending independent completion. Star Club Continue remains operator-owned. Search remains operator-controlled with checked allocation/STOP. Variable backgrounds and idle CPU remain separate backlog.

[Verification](docs/Verification-2.9.130.md) · [Live test](docs/Live-Test-2.9.130.md) · [Selected goal](docs/Pyramid-Current-Goal.md) · [Testing matrix](docs/Pyramid-Testing-Matrix.md).

## Retained history — earlier deliveries

Earlier states below are historical. The current delivery and full activated goal govern remaining acceptance; milestone status is unchanged.

## Current delivery — 2.9.129, host qualified; fresh Beast pending

App **2.9.129** passed **1336 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (383.928s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer gates are retained by exact source and log hashes (289 shared, 80 console; feature rows 14/55/25/67/57/99).

The fresh v2.9.128 Continuous/no-challenge failure occurred on its first observation of the next board, after the prior 56-action route and its three-second handoff. The retained Nines7/15 frame has displaced cards, including fixed slot26 (7|6). All recognised scene anchors remained supported; no eligible glyph existed at that slot's normal origin. This failed Recon sent zero DRAW/Undo/Confirm input and published nothing. A later Single board scan recognised the same settled board. The earlier Single board run's 44-action clear is controller-log evidence, not blanket acceptance of the full loop.

Only **Continuous + Continuous — no challenge + board generation greater than one** gains the new readiness stage. It uses at most six fresh observations with existing one-second transition pacing, sends no input and interprets no challenge header. Ordinary recognition must yield all28 unique tableau cards, a recognised initial LEFT and positively empty RIGHT. Two consecutive positions must match exactly; incomplete or changed positions reset that pair. Unsupported scene, dialog, dimensions/layout, transport, STOP and context changes remain fatal.

The successful pair is passed to the existing Recon starting guard. Recon still independently requires its own two fresh matching starting observations before the first D, then unchanged paired scanning, restoration, no-overwrite publication and private receipt binding. No glyph sample, threshold, native-v22 profile or timing of the existing three-second/1.5-second handoffs changed. First-board/manual/Single acquisition and target-aware/normal-series transition semantics are preserved.

Shortest route still has no application time, state, expansion or byte cutoff. Exhaustive no-win alone triggers automatic same-inventory Maximum fallback; interruption cannot do so. The displayed final Weighted statistics describe that fallback pass, not the earlier Exact pass or cumulative planning time. The operator's repeated 33-action plan proved maximum removal13/28, used19 DRAW and1 RECYCLE, and stopped at an exact partial endpoint in Single board mode. New-board REDEAL allowance and in-board RECYCLEs remain separate; Single board does not automatically REDEAL.

Recon feedback now follows Charlie’s m12898 clarification: DL: card details | DR: card details, with DL first. This supersedes the earlier DR-first presentation clause only; paired physical D and logical stock order remain unchanged.

The original full no-challenge / same-Recon fallback / finite-stock / guarded-REDEAL objective remains incomplete pending fresh Beast acceptance. Each new board resets Shortest. Target-aware Boards bypass partial matching; target-aware Card/Score input retires at the actual target pending independent completion. No-challenge deliberately ignores those targets. Star Club Continue remains operator-owned. Only the measured one-deal enabled REDEAL control grants a click; zero/unknown/unmeasured allowances refuse. Variable backgrounds and idle CPU remain separate backlog.

[Verification](docs/Verification-2.9.129.md) · [Live test](docs/Live-Test-2.9.129.md) · [Selected goal](docs/Pyramid-Current-Goal.md) · [Testing matrix](docs/Pyramid-Testing-Matrix.md).

## Retained history — earlier deliveries

Earlier states below are dated evidence. The current delivery and immutable full goal govern remaining work; milestone acceptance is unchanged.

## 2.9.128 — restored acquisition receipt and A* Route Solution

App **2.9.128** passed **1330 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (370.116s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting and native/protected headless help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; the same 18 shared/feature/private-consumer gates are retained by exact source and log hashes (289 shared tests, 80 console tests; feature rows 14/55/25/67/57/99), not rerun.

Charlie’s three real v2.9.127 acquisitions completed their 24 DRAWs, restoration and atomic JSON publication, then failed private receipt creation with `Missing restored acquisition frame`. Publication had consumed the latest PNG buffer before the restored-frame binding read it. This was a host lifetime error after successful restoration, not evidence that the published JSON granted execution authority.

Persistence now borrows the same bounded PNG; the exact final pixels remain available for receipt binding, then diagnostics finalisation releases the buffer. The complete final image is still written at most once, and persistence errors are retained without retry. The production post-confirmation adapter regression now includes atomic publication, final-header extraction, private receipt creation and exact-byte/context reuse validation. Its five actual captures and single Confirm are unchanged; its pre-confirmation inventory prefix remains explicitly synthetic.

The strategy is now labelled **A* Route Solution**. Startup route objective is **Shortest route** / Exact A* weight 1. Maximum card matches remains selectable and remains the automatic fallback after exhaustive no-win. This correction changes no calibration, thresholds, transport gestures, scan timing, search limits, dependency or shared API.

Fresh v2.9.127 trials supersede the older missing-trial audit. They demonstrate the reported failure, not successful route execution or completion. Fresh v2.9.128 acceptance is pending. The full activated no-challenge / same-Recon fallback / finite stock / guarded REDEAL goal remains incomplete and unchanged; all previous nine live cases carry forward. Target-aware Card/Score stopping, Boards bypass, operator-owned Continue, STOP/context revocation and no uncertain-input replay remain required. The measured one-deal REDEAL capability and safe refusal for zero/unknown/unmeasured variants are unchanged. Backgrounds and idle CPU remain separate backlog.

Charlie relocated the historical tmp validation folders and screenshots into `/home/charlie/tmp/_HOLDER_/`. All twelve moved protected releases retain their original SHA-256 and mode 0555; nothing was restored or moved by the agent. [Verified release relocation map](</home/charlie/Documents/Pyramid/Validation/restored-acquisition-20261022/verified-relocations.json>) and [updated historical documentation targets](</home/charlie/Documents/Pyramid/Validation/restored-acquisition-20261022/documentation-relocations.json>) preserve the old and current paths. Historical immutable packets retain their original path text.

[Verification](docs/Verification-2.9.128.md) · [Fresh Beast brief](docs/Live-Test-2.9.128.md).

## Retained earlier delivery notes

## 2.9.127 — no-challenge continuation, host qualified

App **2.9.127** passed **1330 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (369.45s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting and native/protected headless help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; their 18 shared/feature/private-consumer gates are retained by exact input and log hashes (289 shared tests, 80 console tests; feature rows 14/55/25/67/57/99), not rerun.

The host implementation is qualified. The full selected goal is blocked pending fresh v2.9.127 Beast testing after three consecutive checks found no candidate run; all nine acceptance cases remain pending. Select Mode **Continuous** and Stop condition **Continuous — no challenge** for the new target-agnostic loop. It reuses the Single board acquisition/planning machinery without classifying a challenge or checking targets.

Same-board setting changes discard the plan and retain authenticated Recon. Recon is an optional visible rescan in automatic modes too. An initial automatic start may use that exact current acquisition once; changed bytes, STOP or context revoke it. Each subsequent board resets the worker, status and route dropdown to Shortest route. Paired Recon reports actual RIGHT/DR and LEFT/DL together without changing its physical 24 D operations, pacing or capture policy.

Exhaustive Shortest no-win switches automatically to Maximum card matches within the same owned inventory. Interruption never proves no-win. Target-aware Boards no-win bypasses partial matching, including when Maximum was selected. Score/Card target stopping remains separate, with independent completion confirmation and operator-owned Star Club Continue. No-challenge deliberately ignores those targets.

Only the no-challenge loop uses independently replayed empty-tableau completion plus the final whole-action ACK to schedule a cancellable three-second wait and fresh Recon. ACK does not prove a win. Fresh Recon must independently reject an old partial, exhausted, unsupported or unsettled board. Existing Manual/Single and normal/target-aware final-effect semantics are preserved.

Automatic partial/no-card continuations count actual acknowledged RECYCLEs against the replayed state, derive only the legal remaining D operations (at most 74 from untouched stock: 72 DRAW + 2 RECYCLE), verify every effect, and require a positive native enabled REDEAL and allowance before one click. The 1.5-second post-click wait is a configured policy, not a timing measurement from the three stills. A fresh Recon must prove the next board. Planned RECYCLEs, executed RECYCLEs and new-board REDEAL allowance are separate.

The new bundled-only native capability recognises the measured **1 deal left** and **0 deals left** captions. Only the complete enabled one-deal control grants a click. Two-deal and absent-caption variants remain unsupported and stop safely. File/custom profiles cannot opt in through an identifier. Ordinary native-v22 recognition and glyph limits are unchanged. Variable backgrounds and idle CPU remain outside this goal.

[Verification](docs/Verification-2.9.127.md) · [Live test](docs/Live-Test-2.9.127.md)

## Retained earlier delivery notes

## 2.8.126 — rank-independent Card challenge progress

App **2.8.126 / native-v22** passed **1318 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (360.435s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting of 19 Rust files and native/protected headless help passed. xSAR **0.8.104** and console **0.8.4** are unchanged: 18 shared/feature/private-consumer gates are retained by exact input/log hashes (289 shared tests, 80 console tests; feature rows 14/55/25/67/57/99), not rerun.

The active Card reader identifies the complete top-centre **cleared** word and
the actual `current/target` counter. `ChallengeKind::Cards` contains no card rank.
All thirteen possible rank names use this one path; no per-rank runtime label
bank is required. Boards and Score retain their nine original literal labels,
their separate units and the unchanged numeric-reader policy.

The shared word uses the full 84×32 measured blue-minus-6 intensity template,
exact L1/max ratio ≤210000 ppm, one strong-white core in each of seven letters,
two blank columns at both boundaries, and unchanged tolerance-9 native banner
borders. Search is horizontal within the existing 224×32 objective band, without
resizing or rotating. A separate measured Boards word is a veto only; it grants
no Boards authority and tolerates one damaged core to prevent a corrupted whole
Boards label from becoming a Card challenge. No card-glyph/profile, scene,
modal, empty-position, final-effect, numeric tolerance or timing was relaxed.

Counter kind, target, monotonic progress and session context still bind the
lifetime. An unmet target needs a positively verified board effect and witnessed
fresh board before automatic new Recon/restoration/planning/execution. A numeric
target alone cannot finish the challenge. Independent real completion can stop
remaining actions at the existing guarded action boundary; Star Club Continue
remains operator-owned. Manual, Single board, normal Continuous/restart/mid-game
entry, both route objectives and STOP/context/no-uncertain-retry guards remain.

Native Sixes 0/12 and Nines 0/15 now pass the actual paired worker entry adapter.
Synthetic prefix and missing-letter/boundary/banner/counter cases are explicitly
host regressions. They do not establish fresh guest acceptance of 126.

**The selected generic Card goal is blocked pending fresh Beast verification
and remains incomplete.** Fresh 126 Beast
verification starts with the available Sixes challenge. It must establish real
unmet-target board continuation and verified target stopping, with preserved
modes/guards checked at their appropriate evidence layer. The old missing-125-
trial blocker is historical: actual 125 Single-board Sixes acquisitions and
planning were observed, but did not prove Continuous completion. Its recorded
Continuous refusal used the normal-repeat stop choice. Select **Stop at challenge
target** for this goal. M14 DRAW/REDEAL, variable backgrounds and idle CPU stay
outside this goal. M01–M09 remain accepted; M10 Awaiting verification, M11 Planned,
M12 In progress and M13/M14 Planned. No milestone acceptance is inferred here.

## 2.8.125 — challenge final effects, captions and measured diamond

The retained 123 failures include the following cases. Boards reached 3/3
after 57 actions / 75 acknowledged gestures but the final effect was unconfirmed.
Score cleared its tableau at 1260/1400 after 60 actions / 80 gestures but could
not continue. Charlie accepted Fours gameplay and automatic Recon/planning across
the required boards; its actual 12/12 Challenge Complete/Continue frame was still
refused after 46 verified actions / 61 gestures. Twos 0/10 refused before Recon. A later Fours 0/7 caption also refused entry.
A separate Manual/Single-board acquisition refused JD at 4|1: suit shape error
120414 ppm exceeded the unchanged 120000 ppm cap. Its native frame shows Sixes
0/12; this card-reader correction does not add Sixes challenge-target support.

Native-v21 adds a final-only challenge proof bound to the original incomplete
requirement. It requires the same kind/target and monotonic actual progress;
Boards must advance exactly once. Only a replayed empty tableau after the final
whole-action acknowledgement may use it. All 28 full 60×4 felt bands, scene/modal
checks and exact exclusive pile identities remain required; the new bottom-row
bands are at y + 168. Legacy v20 geometry remains intact. This proves a card
effect only: an unmet Score target proceeds through the existing witnessed board
transition, while a reached target still requires independent real completion.

The actual Twos caption and Fours title/Continue appearances are added as finite
native samples with unchanged tolerances. Fours completion may retire remaining
card actions through the existing guarded terminal path. Continue stays operator
owned. A measured full Fours 0/7 word crop is added without raising tolerance 9.
Native-v22 retains v21's complete profile and appends one exact measured diamond
appearance to each bank (13 to 14, capacity 16). Rank/suit caps, the 80000 ppm gap,
paper and exclusive-colour gates remain unchanged. The original JD frame now
reads all 28 directly labelled tableau cards and LEFT 8C. The new profile also
passes every one of the 423 prior direct labels independently in both banks.
No arbitrary named rank labels or alternate themes are introduced.

App **2.8.125 / native-v22** passed **1309 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (360.816s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting of 14 Rust files and native/protected headless help passed. Unchanged xSAR **0.8.104** and console **0.8.4** retain 18 hash-verified shared/feature/private-consumer gates: 289/80 tests and feature rows 14/55/25/67/57/99. Those gates were retained, not rerun.

**The selected goal is blocked pending fresh Beast verification and remains
incomplete.** Fresh 125 Beast verification
is pending for Boards, Score, Twos, Fours, Sevens and preserved modes/guards.
Charlie's accepted 123 Fours gameplay/transitions are retained as that specific
evidence; its failed endpoint and the new offline regression pass are not fresh
125 controller acceptance. M01–M09 remain accepted, M10 Awaiting verification,
M11 Planned, M12 In progress, M13/M14 Planned. Additional active rank labels,
variable backgrounds and idle CPU remain separate backlog.

## Retained 123 and earlier evidence

## 2.8.123 — guarded challenge target handover

The retained 2.8.122 packet delivered 39 actions / acknowledged 57 gestures but verified only 38. The final original shows actual Boards cleared 2/2 and Perfect, before Challenge Complete. The intermediate v19 proof excludes reached targets; its upper absence bands also overlap Perfect in slots 23–25 (687 non-felt pixels). The normal alternative requires a normal caption. The first scene[2] diagnostic belongs to the earlier flying frame, not the complete final diagnosis.

A verified whole-action effect at a reached count retires card input through a typed pending result; the count does not prove completion. The input-free transition independently witnesses the real terminal before success. Actual Continue and XP skip-instruction crops govern the existing shared XP/optional-LevelUp flow. Star Club Continue is left to the operator, including when it appears after XP. Missing or corrupted Continue is never skip authority. Verified terminal completion can retire a route with actions remaining; ordinary and partial card effects still require exact verification.

App **2.8.123 / native-v20** passed **1297 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (339.941s). Locked check, strict private rustdoc, Clippy with warnings denied, native release, scoped formatting of 23 Rust files and native/protected headless help passed. Unchanged xSAR **0.8.104** and console **0.8.4** retain their 18 hash-verified shared/feature/private-consumer gates: 289/80 tests and feature rows 14/55/25/67/57/99. They were retained, not rerun.

**The selected goal is incomplete; automatic continuation is blocked pending fresh Beast verification.** Fresh protected 2.8.123 Beast verification remains pending for Boards, Score, Twos, Fours, Sevens and the unmet-target board transition. Old native PNGs, offline transports and synthetic compositions are not fresh acceptance. M10 remains Awaiting verification; M11 Planned; M12 In progress; M13 and M14 Planned. Additional ranks, partial DRAW/REDEAL, variable backgrounds and idle CPU remain separate backlog.

## [2.8.122] - 2026-10-21

- Recognise a positively cleared intermediate Boards challenge after the final ACK using native-v19 and an exact bound counter increment.
- Preserve independent empty-tableau/pile/scene/modal proof, all input/STOP/guard checks and existing transition/reward authority.
- Retain five original 121 failure files and eight new reader/production-adapter regressions; 1280 full tests passed. Fresh live acceptance pending.

## [2.8.121] - 2026-10-21

- Remove application Pyramid state/expansion/time/memory-byte search cutoffs; retain checked growth, arithmetic and cooperative operator STOP.
- Show elapsed time and ongoing counters while searching; distinguish interrupted search from exhaustive no-win proof.
- Retain the native winnable-deal counterexample, full-clear and beyond-old-limit/STOP regressions. Full application qualification: 1272 passed, zero failed/ignored/filtered. No 121 live acceptance inferred.

<!-- qualification120:start -->
## v2.8.120 — native normal final-clear correction, host verified

- Retain the three actual normal finals and their first failures/deals with exact provenance.
- Add explicit native-v18 final-only normal header/scene/absence/pile proof.
- Preserve literal v17 rollback, ordinary/partial checks and all input authority.
- Independently replay the recorded 53/47/44-action routes through NativeIo.

App **1268 passed, 0 failed, 0 ignored, 0 filtered** in the full native
include-ignored scope. Locked native check, strict private rustdoc, Clippy with
warnings denied, release build, nine scoped Rust formatting checks and both
headless help checks passed. xSAR **0.8.104** and console **0.8.4** are unchanged;
their qualified 289/80 tests and six feature rows 14/55/25/67/57/99 are retained,
not rerun. Fresh Beast acceptance remains pending. M01–M09 remain accepted,
M10 is Awaiting verification and M11 is Planned.

The first focused compile error and first strict Clippy failure are retained in
the permanent validation directory; no failed full test run occurred for this revision.
See [verification](docs/Verification-2.8.120.md).
<!-- qualification120:end -->

<!-- qualification119:start -->
## v2.8.119 — Continuous challenge objectives and shared rewards, host verified

- Correct the real normal-board refusal with a complete measured second New
  footer; preserve the existing pixel tolerances and native-v17 card profile.
- Separate Shortest route/Maximum card matches from normal-repeat/challenge-target
  lifetime. Use additive checked xSAR 0.8.104 objective costs; keep old APIs.
- Bind actual Score/Boards/rank targets and monotonic progress; interrupt only
  after whole-action ACKs on confirmed challenge success. Reuse shared GAME WIN
  XP skip/optional OK timing and stop before NewGame for challenge mode.
- Preserve exact partial stopping, STOP/context/no-replay and permanent native
  provenance; defer DRAW-until-REDEAL continuation.

The scoped Continuous/challenge implementation is host qualified: app **1260**, xSAR **289**, private console **80**; zero failed, ignored or filtered.
The six overlapping shared feature configurations passed **14/55/25/67/57/99**.
Locked native checks, strict private rustdoc, warnings-denied Clippy, both release
builds, 35 explicitly scoped Rust formatting checks and native/protected help passed.
Fresh Beast gameplay acceptance remains pending. M01-M09 remain accepted,
M10 is Awaiting verification and M11 is Planned.

The first 1,257-pass/3-failure and next 1,259-pass/1-failure application runs are
retained. The latter exposed a costly default match-cost search; Maximum card
matches retains logical-action win-first guidance, while Shortest route uses
match costs. Corrected full qualification passed without raising bounds or
weakening recognition/input guards.
No fresh guest trial, controller replacement, installation or promotion occurred.
See [verification](docs/Verification-2.8.119.md).
<!-- qualification119:end -->

<!-- qualification118:start -->
## v2.7.118 — Continuous and removal fallback, host verified

- Enable Continuous using all three complete measured normal ordinal patches,
  real same-frame progress, guarded series transitions and Solver-free restart.
- Use the additive xSAR 0.7.103 scored search API under one aggregate budget.
  Keep weighted/exact route proof and actual planning statistics visible; distinguish
  exhaustive maximum removals from bounded best-found prefixes.
- Independently replay beneficial partial routes and require exact final ordinary
  position proof before releasing automatic control. A partial result cannot clear,
  advance or restart, including on board three. Preserve STOP and no-replay rules.
- Retain native-v17 card/final-clear calibration and every earlier fixture. The
  console 0.8.4 source and lock remain unchanged.

Required paired checks and protected artifact passed: app 1,218/shared 278/private
console 80, zero failed/ignored/filtered; all six shared feature configurations
passed. The first app 1,215/3/0 full failure and its three obsolete expectations
remain retained; corrected app scope passed without repeating unchanged shared
work. Final documentation/preservation audit passed: 12 documents, 309 local links,
40 review paths and exercised inputs preserved. Versions are qualified.
No guest trial, process replacement, installation or Git promotion was performed.
See [Verification-2.7.118](docs/Verification-2.7.118.md).
<!-- qualification118:end -->

<!-- qualification117:start -->
## v2.6.117 — host verified; live acceptance pending

Final-only verification recognises the retained Perfect overlay together with all
28 completely felt top bands, exact replayed piles and preserved Pyramid scene
anchors. A positive measured empty-LEFT circle can replace the DRAW anchor only
for modelled empty LEFT. All three marker patches and the normal-series frame
guard remain required. The final whole-action ACK gate, two-second window,
STOP/context checks, ordinary recognition and input budgets are unchanged.
Perfect proves tableau clear; challenge completion and restart retain separate
proof requirements. All previous card/profile data roll back exactly to v16.

**All 1,211 tests passed, zero failed/ignored/filtered**, including ignored tests
with eight test threads. Locked check, strict private docs, Clippy, scoped nightly
formatting, release, native/protected help and sealed-input preservation passed.
The protected 117 executable is 12,999,472 bytes, mode 0555, SHA-256
`c67037c7b61f075db35f4af9539c7257b05c2a915f0f48c173fdf5ce879357d8` at `/home/charlie/tmp/pyramid-final-clear-validation-99xkhzgy/release/qmp-qemu-socket-2.6.117`.
No guest trial, installation, process replacement or Git promotion was performed.
Live acceptance remains pending. Final documentation/preservation audit record:
`/home/charlie/tmp/pyramid-final-clear-validation-99xkhzgy/final-audit.json`.
Continuous remains disabled: actual normal Boards: 2 of 3 / 3 of 3 references are
missing. Challenge extension, alternate-theme support and idle-CPU diagnosis remain
open. See the [verification record](docs/Verification-2.6.117.md).
<!-- qualification117:end -->

## v2.6.116 — host verified; live acceptance pending

Manual Recon and automatic Start now share acquisition-name preparation before
consent/request creation. Switching mode still invalidates the old receipt; a
fresh Recon and new Plan route are required. Literal/mixed overrides and the
worker's strict no-overwrite boundary remain unchanged.

**Runs/0002 passed all 1,203 tests, zero failed/ignored/filtered**, locked check,
strict private docs, Clippy, three-parent scoped formatting, release, native and
protected help, and sealed-input preservation. The verified protected 116 executable
is 12,882,768 bytes, mode 0555, SHA-256
`2b3efc9c64ab9edd39868969b63d684d3808a4fe9ed6d1ec94c166f370ab8fb0`.
The initial E0616 and first full run's 1,202/1/0 local-fixture failure remain
retained; the complete unchanged-source retry uses eight test threads and includes
all tests. Load sensitivity is inferred rather than a measured failure timeline.

Root's final documentation/link/preservation audit passed (seven documents, 243 local links, 12 review paths). Fresh live
acceptance has not been performed. Native-v16, fixtures, siblings and protected
107–115 remain unchanged; Continuous is disabled and broader challenge/theme/
idle-CPU objectives remain open. See the [116 verification](docs/Verification-2.6.116.md)
and [116 operator brief](docs/Live-Test-2.6.116.md).

## Retained qualified 2.6.115 evidence

Required host checks, the native release and protected 115 executable have
passed. The current suite reports **1,199 passed, zero failed/ignored**; sealed
preservation passed through `runs/0001/audit-resumptions/0001` after the exact
telemetry refusal `Reviewed operational bytes changed: .trigger-tree/history.jsonl`.
Original failure evidence remains retained; no Cargo command was repeated.
Root's final documentation/link/preservation audit passed. Fresh live
acceptance has not been performed; Continuous remains disabled and the full goal
remains incomplete. See the [verification record](docs/Verification-2.6.115.md).

## v2.6.115 — host verified; live acceptance pending

- Select bundled native-v16. It inherits native-v15's two explicit schema-2
  calibration extensions and adds one exact full-mask gold objective-highlighted
  7♣ rank sample at fixed `3|3` to both banks, rank-seven variants 1→2.
  Frozen glyph policy, caps, suit variants, ROIs and alignment are preserved.
- Recognise the coupled depleted-stock execution state only when replay expects
  both piles empty, with measured REDEAL/LEFT-X, original other scene anchors,
  positive RIGHT-empty and exact remaining/absent tableau evidence. An apex King
  still present is not a clear; no REDEAL operation is added.
- Recognise the measured alternate complete Undo All modal only after the sole
  confirmation ACK. Wait one cooperative second before the first restoration
  capture, then retain six observations with at most five further one-second
  known-stale waits. Require two consecutive original-board proofs without
  re-clicking. STOP/context still revoke waiting and further I/O; ordinary
  observation, unknown refusal and restoration before publication remain strict.
- Bind the supported normal first-board identity/count through Recon, restoration
  and matched execution frames. Reuse independent bounded restart controls only
  from a supported specific Pyramid terminal or typed actual final-board evidence.
  ACKs, planned goals and DeliveredUnconfirmed grant no such authority.
- Keep Continuous disabled: actual normal second/third ordinal evidence is missing.
  Challenge objectives, alternate-theme support, idle CPU and the whole goal remain
  unfinished. Preserve Manual/Single board and unrelated post-game policy.
- **Current run 0001 passed 1,199 tests, zero failed/ignored**, locked check,
  strict private docs, Clippy and scoped formatting. Release, protected-artifact and sealed preservation checks passed; the final documentation audit passed. The earlier
  1,179-test provisional-115 pass
  at 18:12:54–18:16:37 UTC on 19 October 2026 predates these corrections. It is retained
  evidence, not current release qualification. The protected 115 artifact is ready for a separately accepted operator trial;
  protected 107–114, xSAR 0.6.102 and console 0.8.4 remain frozen. No installation,
  process replacement, promotion or new live acceptance is claimed.
- Correct stock-tail regression fixture setup by encoding its explicitly
  synthetic PNG before starting the bounded local QMP peer. This addresses the
  test's handshake timing failure without relaxing production timeouts or policy;
  the corrected fixture is covered by the current passing suite. Retain the
  original `WouldBlock` attempt and test-only E0599/E0308/E0425 correction history
  in the verification record.

See [verification record](docs/Verification-2.6.115.md) and
[operator brief](docs/Live-Test-2.6.115.md).

## v2.6.114 — observed Pyramid series progress, host verified

- Add bounded literal native 0/3, 1/3 and 2/3 header recognition without inferring
  game mode from the denominator.
- Require observed next progress after every positive Continuous execution result
  and observed 0/3 after restart; stop on unknown, stale or conflicting facts.
  Route delivery, action totals and elapsed time do not supply a displayed count.
- Preserve Single board, native-v14 card matching and the disabled Continuous UI
  until standard-mode and native terminal/restart evidence is implemented and
  verified independently of the count reader.
- Keep xSAR 0.6.102, console 0.8.4 and protected stable executables 107–113 frozen.
- Add 14 reader tests, 10 production native-adapter tests (eight private AF_UNIX
  fixture cases) and expanded observed-progress driver coverage, including
  first-capture STOP/context revocation and retained evidence.
- Required host checks passed: **1,132 tests, zero failed/ignored**, strict private
  docs, Clippy, scoped format, release and native/protected help. Final
  documentation/preservation audit passed.
  The full Continuous goal remains incomplete; no live acceptance, process
  replacement, installation or promotion is claimed.

## v2.6.113 — exact aligned King diamond sample, host verified

The app-only native-v14 correction appends **one measured full 20×16 RGB
diamond sample to both reader banks, 12→13**, and selects bundled native-v14.
The source is fixed `5|3` K♦ from the original failed RECON capture. Its nominal
shape already fits at 69,137 ppm, but one strong-black face-art pixel makes
nominal colour nonexclusive; the best eligible old witness is 130,980 ppm,
above the unchanged 120,000 ppm cap. The measured clean crop uses native
`[896,354,20,16]`, the existing `(+1,-1)` alignment. This is a bounded calibration
addition for an exposed case, not a general artwork or colour-policy repair.

Schema 2 `paper_ink_l1_v2`, rank/suit caps 220,000/120,000 ppm, different-label
gap 80,000 ppm, full masks, nine ±1 shifts, paper support and exclusive colour
remain unchanged. Native-v13, its templates and explicit custom File behaviour
are retained. The host-verified 2.6.112 stock-only Undo All/OK pacing correction
is preserved; no wait, search, input-authority or other-game change is added.

Focused native-v14 production tests passed **7/0/0**, 1,093 filtered;
compile 13.83 s/tests 80.22 s. These overlap the later full suite.
Required Cargo commands passed, including **1,100 tests, zero failed/ignored**,
strict private rustdoc, Clippy, scoped formatting, native release and headless
help. The protected 2.6.113 copy matches the native artifact and passes help.
Attempt 0001's final preservation check stopped only on updated automatic
trigger-tree telemetry after every command passed. Audit-only resumption
validated the sealed command evidence/frozen inputs and refreshed only the two
already declared telemetry hashes; no workload or help was repeated. Required
verification now records `passed-required-checks`. App 2.6.113 earns one correction
build/no additional minor. The final root documentation/preservation audit passed for nine documents,
221 local links and 21 review paths; no full test failure occurred. Read-only preflight still found
protected 2.6.111 running; the operator must preserve evidence and close it
deliberately before later 2.6.113 launch.

The larger goal remains **ACTIVE and incomplete**. Native Continuous is still
disabled pending implemented/validated progress, objective, terminal/redeal and
restart handling. Charlie reports Single board works; that report is separate
from a new 2.6.113 operator trial. M10 remains Awaiting verification, M11 Planned,
M12 In progress and M13 Planned. The intermittent idle-CPU report has not recurred
under SysProf and remains unresolved; this correction contains no CPU fix.
xSAR 0.6.102, console 0.8.4, indices and protected stable 107–112 remain frozen.
No guest/GUI/input, installation, process replacement or promotion is performed.

See [verification](docs/Verification-2.6.113.md) and
[operator brief](docs/Live-Test-2.6.113.md).

## v2.6.112 — RECON reset timing, host verified

The approved app-only correction removes four configured one-second waits from
the normal stock-only RECON Undo All/OK reset. It retains one guarded Undo All,
two fresh matching confirmation-modal captures, one guarded OK and two fresh
consecutive original-board/pile captures before complete JSON publication.
Recognised unchanged exhausted-board or matching-modal scenes delay the next
observation by one second only when another bounded attempt remains. Initial
board confirmation still waits one second; D pacing remains at least 500 ms.
The six-observation budgets, 300-second deadline, STOP/context checks, input
uncertainty refusal, complete inventory and restoration/export contracts remain.
Other delays, recognition profiles, search behaviour and other games are unchanged.

Required host checks passed: **1,093 tests, zero failed/ignored**, locked check,
strict private rustdoc, Clippy, changed-range formatting shadow, native release
and headless help. Protected executable parity/mode/hash/help passed. Focused
core checks passed 25 overlapping tests. Retained first-failure evidence covers
the SQLite sidecar preflight stop and the 1,092-pass/one-failure obsolete paired
expectation, whose four removed waits were corrected without changing other
asserted events. After the successful commands, a new automatic zero-result
Tree Ring recall receipt stopped preservation; a narrow audit-only resumption
classified it and reused unchanged command evidence. No Cargo/formatting repeat
was needed. Final documentation/preservation audit passed: 6 documents, 134 local links and 10 review paths. App 2.6.112 earns
one correction build and no additional minor. No measured live speed-up or
reliability result is claimed. xSAR 0.6.102, console 0.8.4 and protected stable
107–111 remain frozen; no guest/GUI/input/install/process replacement/promotion.

Charlie reports Single board works. Native Continuous remains disabled and the
larger goal remains ACTIVE/incomplete. Native 0/3, 1/3 and 2/3 sources are now
available but uncalibrated and do not establish mode/objective/restart authority.
The master schedule records challenge target-count requirements; challenge
automation and validated progress/redeal handling remain pending. M10 Awaiting
verification; M11 Planned. M12 covers immediate measured pacing and optional
bounded timing sliders; M13 covers longer-term professional, generic xSAR naming.
This correction alone does not complete those milestones. See the
[verification](docs/Verification-2.6.112.md) and
[operator brief](docs/Live-Test-2.6.112.md). Historical records below retain their
original evidence and status at delivery.

Charlie reports high idle CPU after route completion. Its cause is under
read-only investigation; this correction includes no CPU fix.

## v2.6.111 — exact native diamond variants, host verified

The approved app-only correction appends measured native 20×16 diamond RGB
variants to both reader banks and selects bundled native-v13. Schema 2
`paper_ink_l1_v2`, rank/suit maxima 220,000/120,000 ppm, different-label gap
80,000 ppm, full ROI, nine ±1 shifts, paper support and exclusive colour remain
unchanged. No matcher or input-authority change is included. Literal custom
File profiles retain their selected schema, strict parsing and no fallback.

The priority evidence is the PLAY refusal for covered K♦ at fixed `3|2`, beside
cyan-highlighted 6♦ at `4|2`, after 27 of 42 route actions and 42 acknowledgements.
Separate RECON refusals expose K♦ at fixed `5|2` beside lower face artwork and
7♦ at fixed `3|2`. These original failures remain failures; finite labelled
regressions do not establish fresh acquisition, restoration or live success.

Manual/Fast and Single board keep their existing timing and search budgets.
The prior 45- and 42-action routes used **Fast**, with heuristic weight two;
they do not establish minimum action counts. **Fewest actions** selects exact
unit-cost A* with weight one, subject to its existing resource bounds. Select
it before a fresh RECON: changing method invalidates the acquisition receipt.
Never replay the interrupted route or use an old export as current authority.

**Native Continuous remains unavailable** pending an implemented and validated
standard-three progress/terminal contract. Retained source headers show 1/3
(PLAY selected/restored) and 2/3 (RECON king); these sources remain uncalibrated
and unverified for automatic progress. Positive 0/3, GAME WIN and restart-target
evidence are still needed. The larger goal is blocked; **M10 Awaiting
verification; M11 Planned**. xSAR 0.6.102 and console 0.8.4 remain frozen,
together with prior stable executables.

The exact profile appends **two diamond variants per bank, 10→12**, and is
827,721 bytes, SHA-256
`507fbe1499cc6359b117022fb2cadb79f0737d4f3f77687bc763992acfc3edd8`.
The clean K♦ crop uses the existing upward one-pixel alignment; the other crop
retains the legitimate neighbouring selection halo. The exposed 7♦ capture
adds no template. Reversing only the ID and both appends exactly restores v12.

First-attempt focused production verification passed **9 tests, 0 failed/ignored**,
1,078 filtered, compile 11.70 s/test 60.24 s. It includes 846 retained-v12,
116 new RECON and 26 partial-PLAY bank decisions: **988 counted decisions**,
with whole-position checks additionally overlapping those observations. These
finite exposed regressions do not measure independent general accuracy or prove
fresh acquisition, restoration or guest success.

Required app host checks passed: **1,087 tests, 0 failed/ignored**, including
the nine focused tests, plus locked check, strict private rustdoc, Clippy,
scoped formatting, release and headless help. All Cargo commands exited zero
in attempt 0001; its final preservation audit stopped on automatic trigger-tree
read telemetry. Audit-only finalisation 0002 revalidated the retained command
logs and frozen compiled inputs without repeating Cargo or tests. The final
documentation/preservation audit passed for 9 documents, 200 local links and 27 review paths. One app correction build/no
additional minor is earned; xSAR/console and stable 107/108/109/110 stay frozen.

Protected **2.6.111 is ready for an operator trial** at
`/home/charlie/tmp/pyramid-diamond-v13-validation-ugnvhgv9/release/qmp-qemu-socket-2.6.111`:
12,733,440 bytes, mode 0555, exact native parity/headless help exit zero, SHA-256
`32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`.
Read-only preflight at 2026-10-19T14:08:38.312919+00:00 still observed PID 27073
running protected **2.6.110** with `--image`; images were zero files/bytes at
that instant only. Preserve evidence before startup clears images, then close
the old app deliberately before the operator launches 2.6.111. No guest/GUI/
input/install/promotion or process control is performed by this correction.

See [verification](docs/Verification-2.6.111.md),
[fixture provenance](tests/fixtures/pyramid-recon/native-v13/README.md) and
[operator brief](docs/Live-Test-2.6.111.md).

## v2.6.110 — versioned graded glyph reader, host verified

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

Schema 1 keeps its exact previous XOR 24/8 matching and strict keys. Literal
custom File profiles receive no automatic migration or fallback. Native
1920×1080 geometry, scene/dialog, positive-empty, complete-inventory,
restoration, receipt/context/STOP and input-effect contracts remain. Manual/Fast
and Single board keep their existing workflow/timing/search budgets. **Native
Continuous remains disabled/unavailable** without positive standard-three target
and progress evidence; Daily Challenge automation is not added. The resumed
larger objective is blocked on native standard-three-board evidence; M10 Awaiting verification/M11 Planned.

Initial raw policy calibrated 104 historical+29 ten-failure cards failed 8/4;
revised paper-relative policy uses 250 development+29 ten-regression, fixed bounds; 423 retained observations split
250 development/29 development-regression/115 prior-exposed validation/29 exposed
fifth observations. The original prospective raw-ink failure remains retained. Full 1,078 tests and required host checks passed; no new
RGB template or universal accuracy claim. xSAR 0.6.102/console 0.8.4 and delivered
107/108/109 stay frozen, one earned correction build/no minor after full pass. See
[verification](docs/Verification-2.6.110.md) and [live brief](docs/Live-Test-2.6.110.md).

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

## v2.6.109 — exact native rank-8 variant, host verified

The approved app-only rank-8 correction appends one exact native RGB sample to
both reader banks and selects native-v11. Rank-8 variants increase 3→4 within
the existing cap 8; XOR 24/8, alignment ±1, paper/ink/colour, uniqueness, geometry,
scene/dialog and positive-empty guards are unchanged. Native-v10 remains intact.
The source refusal retained one capture, zero input attempts and no export;
its diagnostics prove neither a complete deal nor restoration.

Full 1,052 tests passed, zero failed/ignored, plus locked check, strict private
docs, Clippy, scoped formatting, release and help; attempt 0001 passed without
failure. Focused 9 cases overlap full scope. Stable 109 12,685,240 bytes/mode 0555
SHA-256 `0d6dc51bb7a292606ff81ef6c534898aef4ca7129c18a9d070cb7d88727a58ed`
passed native parity/help 0. No controller/images zero at 05:10:21 UTC only. Final
documentation/preservation audit passed: 9 documents, 165 local links and 26 review paths. xSAR 0.6.102/console 0.8.4 and
stable 107/108 remain frozen; one earned correction build/no minor after full pass.
Manual/Single board, paired RECON, final observation, Fast/Fewest actions,
statistics, receipt/STOP rules and unavailable Continuous are unchanged.
M10 live Awaiting verification; M11 Planned; larger goal incomplete.
[Verification](docs/Verification-2.6.109.md) retains actual check results; [live brief](docs/Live-Test-2.6.109.md) does not authorise agent launch.

## v2.6.108 — independent Pyramid workflow extensions, host verified

- Add Manual/Single board/Continuous preferences and Fast/Fewest actions methods;
  keep Manual/Fast startup and selection input-free. Single board holds one
  worker lease across fresh acquisition, input-free planning and one execution.
- Pair bounded D operations with at least 500 ms after each ACK, recognising
  both newly exposed endpoint cards while preserving complete-inventory,
  exhaustion, restoration and no-replay requirements. The deterministic success
  requires 20 captures/24 D operations.
- Retire card dispatch after final delivery. Observe until positive clear,
  complete new-board or terminal proof, or the two-second ACK-relative window;
  DeliveredUnconfirmed stops without another card input or continuation.
- Consume additive xSAR weighted/options/progress APIs directly. Fast uses weight
  two, Fewest actions uses exact A*, both independently replay. Display latest-only
  actual planning work/time and retain matching context/run/board ownership.
- Append the approved exact native highlighted-clubs sample to both banks as
  native-v10; preserve v9 and 24/8 XOR, ±1 alignment, full masks and strict guards.

**Native Continuous remains unavailable:** no positive native 0/1/2 progress or
standard-three-board target evidence exists. Terminal/New Game/Play/full-board
facts do not prove it; refuse before restart/input. Generic driver and independent
restart coverage uses synthetic facts only. The larger goal remains incomplete;
no Daily Challenge automation or live acceptance is claimed.

App 1,043/xSAR 266 full tests passed, zero failed/ignored; the six feature
configurations and private-console-copy 80 tests passed and overlap their stated
scopes. Required locked checks, strict private docs, Clippy, scoped formatting,
releases and app help passed in complete attempt 0002. Earlier attempt 0001 was
intentionally interrupted for decoded-frame retention review, not a test failure;
full rerun passed after retaining raw pixels before fallible analysis. Both
focused first failures remain recorded. One agreed minor/build in each changed
repository is earned. The separate stable 108 candidate is mode 0555, SHA-256
`badd90d6f75d78600066b749d1582a4ec681ab3f89bae8e774d0f23a7f35e573`; delivered 107 stays unchanged.
M10 live acceptance remains pending; M11 Planned; native Continuous unavailable.
No GUI, guest input, installation or promotion was performed. Final documentation
audit follows ownership return.
[Verification](docs/Verification-2.6.108.md),
[live brief](docs/Live-Test-2.6.108.md),
[exact v10 provenance](tests/fixtures/pyramid-recon/native-v10/README.md).

## v2.5.107 — exact native King variant, host verified

- Append one exact King-rank crop from the retained failed native board to both
  tableau and stock reader banks; select the separately named native-v9 by
  default and preserve every native-v8 byte.
- Retain rank/suit XOR caps 24/8, alignment ±1, all existing variants, scene/modal
  checks, geometry, identity validation and guest-input authority. The K♠ rank
  at `3|2` previously measured 28; suit distance 2 already passed. All other 27
  tableau cards and DL 4♣ passed. This is a calibrated variant, not elimination
  or a widened threshold.
- Preserve the original one-capture, zero-input, unpublished refusal at
  `/home/charlie/tmp/.pyramid-recon-710918-3`. Nine focused regressions passed
  and overlap the full 977-test suite, zero failed/ignored. The initial test-only
  E0689 compile failure is retained; explicit `u32` coordinates corrected it
  without a production change.

Required locked check, strict private docs, Clippy, scoped formatting, release
and headless help passed. The separately copied mode-0555 stable executable
passes help and matches native SHA-256
`b2460b0dc15327346349425cb45a9e9f285fc840a75593d42c6949f12b1cf6cb`.
One correction build/no minor is earned; fresh Beast acceptance remains pending.
No GUI launch, process replacement, installation or guest input was performed.
xSAR 0.5.101 and console 0.8.4 stay frozen. M10 remains Awaiting verification;
M11 Planned. Paired 500 ms RECON, final two-second completion observation,
manual/one-board/continuous modes and faster-search/statistics work are separately
approved future app 2.6.108/xSAR 0.6.102 work, not part of this correction.
[Verification](docs/Verification-2.5.107.md),
[fixture provenance](tests/fixtures/pyramid-recon/native-v9/README.md) and
[live-test brief](docs/Live-Test-2.5.107.md) retain the evidence boundary.


## v2.5.106 — remove the Pyramid application memory cutoff, host verified

- Share one application-owned search policy between the worker and UI diagnostics.
  Set its byte allowance to `usize::MAX`, represented honestly as JSON null with
  `allocator_checked`; this is neither reserved storage nor measured available RAM.
- Retain the existing 2,000,000-history/2,000,000-expansion work bounds, 120-second
  deadline, cancellation and checked allocation failures. xSAR and console sources
  and their defaults remain unchanged.
- Exercise real collection growth/reconstruction beyond the former cutoff:
  524,290 retained histories and 524,289 actions. Keep separate work-limit and
  honest-diagnostic regressions. All three focused tests passed and overlap the
  full 968-test suite, zero failed/ignored.
- Reproduce MemoryLimit at 524,288 histories on the exact reported deal, then solve
  its 42-action route with the new policy at 1,979,388 histories. Independent clear
  replay passed. This is offline evidence without guest input or live acceptance.

Required locked check, strict private rustdoc, Clippy, scoped formatting, native
release and headless help passed. One correction build/no minor is earned;
xSAR 0.5.101 and console 0.8.4 remain frozen. No receipt retry, Daily Challenge
completion-rule or input-authority change is included. M10 remains Awaiting
verification; M11 Planned. The final seven-document/109-link/13-path preservation audit passed.
[Verification](docs/Verification-2.5.106.md) and
[live-test brief](docs/Live-Test-2.5.106.md) record the exact candidate and bounds.


## v2.5.105 — three-step Pyramid workflow and failure packets, host verified

- Show **1. Recon**, **2. Plan route** and **3. Solve board**, concise phase/status,
  preview and STOP for Pyramid Shortest Path. Hide technical fields, generic
  capture/manual controls, output and already-open technical windows during
  normal use. Other games and Computer Vision keep their existing controls.
- Keep bundled native-v8 and automatic evidence names/handover. Remove visible
  per-run consent checkboxes: an explicit Recon/Solve press arms only that
  immediate context-bound operation; worker receipt/evidence/STOP gates stay
  authoritative. Plan sends no guest input and never starts Solve automatically.
- Expose actual errors, dispatch fields, session log, packet path and literal
  overrides after failure. Save bounded private diagnostic packets from existing
  source bytes/frames on the worker, without extra capture/input. Preserve partial
  writes and omissions as visible warnings. Saved evidence grants no resume right.
  This is the authorised failure-only PNG exception; successful ordinary captures
  and the `--image` GAME WIN sequence policy are unchanged.
- Match diagnostic context, phase and request/local-failure identities so late
  packets cannot reopen errors for a newer step. Reconcile matching computed-run
  completion to a terminal Ready/Uncertain state while retaining Detached/STOP
  semantics and single-use plans.

Focused UI13/writer8/backend2 filters passed, zero failed/ignored in each; they
overlap the full suite. The first complete required sequence passed **965 tests,
zero failed/ignored**, locked check, strict private rustdoc, Clippy, six scoped
format modules/four parent shadow checks, native release and headless help.
Application 2.5.105 earns exactly one correction build, without a new minor.
Prior 2.5.104/942-test evidence remains separate. The exact native artifact SHA-256
is `77eb7763f01f7698399582466f879e726689059697c556abf7e1cbaa96600e7a`.
All three indices, frozen siblings, compiler inputs and unrelated evidence were
preserved; automatic `.trigger-tree` telemetry was classified and left untouched.
The final seven-document/103-link/19-path preservation audit passed. Read-only preflight still observed
older PID 683122 at a different executable path/checksum; no process replacement.

This slice does not increase search caps, preserve consumed planning receipts or
fix Daily Challenge completion/progress rules. Retain the reported clear versus
40/41 controller-verified actions and the later MemoryLimit as unresolved live
boundaries. xSAR 0.5.101 and accepted console 0.8.4 stay frozen. M01–M09 remain
accepted, **M10 Awaiting verification and M11 Planned**. See
[Verification-2.5.105.md](docs/Verification-2.5.105.md) and
[Live-Test-2.5.105.md](docs/Live-Test-2.5.105.md). No launch, installation,
guest trial, commit, push or publication was performed for this correction.

## v2.5.104 — stable Pyramid scene anchors, host verified

- Introduce native-v8 with the profile ID and only `board.scene[0]`/`[1]`
  changed from immutable v7. Use complete-mask right-border artwork at
  `[1796,116,109,18]` and `[1796,388,109,20]`, tolerance 4; retain the third
  arrow patch, all card readers/caps, modal signatures and geometry.
- Default to `ReconProfileSource::BundledNativeV8` with exact-byte provenance;
  retain `File(PathBuf)` custom profiles, automatic path/Deal-ID handover,
  unchecked context-bound consent and private same-session restored receipts.
- Retain the later selected-9♦ PNG, original restored baseline, exact acquired
  deal and profile provenance. Diagnose the old K-chart crop's 222/2355
  mismatching pixels without weakening glyph or scene matching. The interrupted
  41-action route verified ten actions and refused action 11; its plan stays consumed.
- Report the exact zero-based scene index and native `[x,y,width,height]` on
  execution mismatch; distinguish a fully matched Undo All modal. Keep the
  original first-mismatch retention, STOP, per-gesture guards and no-resume policy.

Thirteen focused v8 tests passed. Preserve the first full result (941 passed,
one failed, zero ignored): an old route-observer assertion expected the generic
error instead of the new exact scene index/bounds. The assertion is corrected
without a production change. The complete rerun passed **942 tests, zero
failed/ignored**, locked check, strict private rustdoc, Clippy, scoped formatting,
native release and headless help. Preserve the exact release checksum and older
PID 667842 preflight separately: rebuilding did not replace that process. All
three indices, frozen siblings and unrelated source/evidence are preserved;
automatic `.trigger-tree` telemetry remains intact. Prior 929/907/256 totals
remain historical. This qualifies one correction build, with no new minor milestone. xSAR stays 0.5.101 and console
stays frozen 0.8.4. M01–M09 remain accepted, **M10 Awaiting verification and
M11 Planned**. See [Verification-2.5.104.md](docs/Verification-2.5.104.md),
[the v8 fixture record](tests/fixtures/pyramid-recon/native-v8/README.md) and
[the live-test brief](docs/Live-Test-2.5.104.md). No corrected-build live
success, installation, commit, push or publication is claimed.

## v2.5.103 — Solver consent and automatic Pyramid setup, host verified

- Remove the visual Solver-state gate from reconnaissance, route-position and
  final-new-board recognition. A legitimate grey unavailable Solver control,
  including a Daily Challenge, no longer rejects an otherwise supported board.
  Explicit untouched/restored foreground-board and no-interference consent with
  Solver not engaged remains mandatory. These operations never enable Solver.
- Keep schema 1 compatible: require and validate the existing `solver_off`
  profile field, then discard it instead of storing or matching those pixels.
  Preserve scene/modal signatures, glyph samples, geometry, caps and all prior
  profiles. Retain the exact daily-challenge first-refusal PNG and report without
  reclassifying the old zero-input attempt as a successful acquisition.
- Bundle the exact native-v7 profile as the default, retaining an explicit custom
  file override and typed source provenance. Generate an unused-looking output
  pathname and Deal ID without creating or reserving files. Preserve manual
  overrides independently and retain private no-overwrite worker publication.
- Hand the exact successful export path to Calculate automatically; rotate only
  untouched automatic fields for the next acquisition. Calculate requires a
  successful current-context acquisition, and both consent checkboxes remain
  unchecked until the operator confirms them. STOP, context changes and failed
  or uncertain execution retain their existing revocation and no-resume rules.

Application 2.5.103 passed all required host checks and earns exactly one build
increment from host-verified 2.5.102, with no new minor milestone. All 929 tests
passed, zero failed/ignored, with locked check, strict private docs, Clippy,
scoped formatting, native release and headless help. Preserve the first Clippy
failure and complete successful rerun. xSAR stays 0.5.101 and console 0.8.4 stays
frozen. The earlier 907/256 results below remain historical evidence. See
[Verification-2.5.103.md](docs/Verification-2.5.103.md) and the
[current live-test brief](docs/Live-Test-2.5.103.md).

M01–M09 remain accepted, **M10 Awaiting verification and M11 Planned**. Actual
daily stock progression, Undo All/restoration availability, selected-card tint
and terminal behaviour remain live evidence questions. The visible `1 deal left`
caption does not prove the offline reference model's recycle allowance. No
whole-challenge completion, installation, commit, push or publication is claimed.

## v2.5.102 — guarded Rust-calculated Pyramid execution, host verified

- Add separate Calculate route and Execute route controls for Pyramid Shortest
  Path with Solver off. Consume xSAR's generic search/execution modules and pure
  Pyramid adapter directly; leave accepted console 0.8.4 byte-unchanged and add
  no application-to-console dependency.
- Bind input-free calculation to a private same-session successful restored
  acquisition receipt and exact published JSON bytes, allowing a moved filename.
  Independently replay the A* result before retaining a single-use plan.
- Require separate no-interference confirmation, two origin observations and
  fresh expected-position/STOP/context/transport checks before every gesture,
  including between MATCH selections. Commit each logical action only on fresh
  effect evidence, then stop after one tableau clear with no terminal/next-board
  input. Failed or uncertain runs have no resume or cleanup input.
- Keep finite action/gesture counts, six ordinary observations per phase, twenty
  final observations and a 1,200-second cooperative deadline. Correct the total
  capture allowance to `max(2 + 6*(G + A) + 20, 12*A + G + 16)` so both
  precondition and effect phases fit without changing their individual limits.
- Reject known non-regular route paths, including FIFOs, before opening; recheck
  the opened handle's metadata before the bounded read. Filesystem operations
  remain blocking/cooperative rather than subject to a hard I/O deadline.
- Retain dedicated monotonic UI run ownership through STOP and generic worker
  states until the matching completion arrives. Revoke stale plans/consent on
  context, path, reconnaissance and STOP changes; ignore unrelated completions.
- Preserve calibration signatures, accepted five-game behaviour, original
  evidence and GAME WIN image boundaries. Intermediate selected-card rendering
  is unverified and may be refused by the strict guard; tint is a possible live
  blocker, not an observed failed live trial.

M01–M09 remain accepted; **M10 is Awaiting verification and M11 Planned**.
Application 2.5.102 and supporting xSAR 0.5.101 are host verified, each applying
one agreed minor increase and exactly one build increment. Complete tests passed:
907 application and 256 xSAR, including ignored tests, with zero failed/ignored.
Full feature/consumer checks, strict private docs, Clippy, releases, executable
help and scoped new-module formatting passed. Parent-range formatting retains
two inherited required blank-line differences; package-wide baseline differences
remain explicit. [Verification-2.5.102.md](docs/Verification-2.5.102.md) records
commands, first failures and preservation evidence; the
[live-test brief](docs/Live-Test-2.5.102.md) defines the outstanding Beast trial.
No installation, automated guest success, commit, push or publication is claimed.

## v2.4.101 — native v7 calibration candidate, host verified

- Append one byte-original rank-2 glyph and one spade glyph to both native reader
  banks in the separately named v7 profile. Use already v6-recognised source slots
  7 and 25, preserving the failed rank/suit slots as held-out regression evidence.
- Retain the original refused baseline, native crop hashes, provenance and
  diagnostic evidence. Increase bank totals from 34/36 to 35/37 rank/suit variants;
  retain every existing variant, ROI, mask, 24/8 distance cap, alignment 1,
  colour/paper check and scene/dialog signature.
- Keep the safe first refusal explicit: one capture, no attempted input, output
  publication not attempted. Undo All/restoration was unnecessary for that run.
  Preserve the accepted v6 deal/CSV and the existing five-game input/CV boundaries.
- Verified 2.4.101 applies the two previously accepted M01/M03 application minor
  credits once from 2.2.100 and earns one build increment to 101 through the
  successful full test scope. Console 0.8.4 and xSAR 0.4.100 remain unchanged.

Package formatting fails on the same complete normalised differences as pristine
HEAD; the new module was formatted separately and no legacy/sibling source was
reformatted. This known baseline limitation is not a passing package check.
Locked check, all 876 tests including ignored tests, 10 focused v7 regressions,
strict private docs, Clippy, standalone new-module formatting, native release,
executable `--help` and final preservation checks passed in
[Verification-2.4.101.md](docs/Verification-2.4.101.md). Fresh v7 live acquisition/
restoration and M09 human console/game comparison remain pending. Final checks
found no running collector; PID 521945/2.2.100 is historical, with exit time/cause
unknown. V7 remains compatible with the old collector's per-request profile
loading, but an absent application requires an operator launch. Existing images
are privately backed up with all hashes checked and originals unchanged. Rust
production logic is unchanged. Record the actual new process,
profile hash and export producer separately from the built candidate. No commit,
push, deployment or guest input is authorised by this calibration repair.

## v2.2.100 — Numeric migration and late OK verification repair

- Permit the existing fourth, input-free OK observation when normal observation
  three first measures a settled-size body whose current exact-black glyph is
  absent and whose geometry still needs confirmation. Use 200 ms only once that
  final reserve is granted; earlier glyphless observations keep normal spacing.
- Keep current-frame exact RGB(0,0,0), consecutive settled geometry and fresh
  coordinates as click requirements. Preserve three normal observations, one
  irreversible OK-only reserve, no fifth capture, New Game/Play budgets, STOP,
  foreground priority and uncertain-input safeguards. No coordinate or colour
  threshold changes are made.
- Retain the four byte-original Spider captures from session 1792258073269.
  Add shared five-profile and production Spider regressions for a controlled
  fourth-frame glyph, persistent washout, missing/moved body, alternative menu
  rejection and cancellation. A controlled fourth response is not a live result.
- Correct the Free Cell stale-glyph regression: keep clean-to-washed-to-washed
  rejection, and separately verify the existing late-body fourth observation
  after gameplay or Score clears geometry. Require the fourth frame's own
  distinct glyph coordinate, one reserve only, no input and retained evidence.
- Retain five byte-original Spider captures from session 1792264703614 and a
  production-driver regression for persistent interrupted-glyph washout. It
  records a known safe refusal after four OK observations, not a recovery fix;
  current exact-black evidence remains mandatory and no fifth frame is allowed.
- Adopt Charlie's approved application migration checkpoint `2.2.100`, without
  claiming a historical build count. Independently migrate the sibling xSAR
  crate to `0.4.100` and align its application lockfile entry. The dependency
  remains path-only; dependency requirements and registry resolutions do not
  change. Future ordinary verified revisions increment build automatically;
  milestone/minor and major transitions require Charlie's direction or approval.
  The build counter never resets.
- Preserve application transport, Solving mode, production timing and recognition,
  input safeguards, GAME WIN recording and Windows server sources. The earlier
  candidate.15 verifier compiled successfully, then reported 863 passed tests
  and one obsolete Free Cell assertion failure. These revised sources remain
  provisional until focused and complete tests, strict docs/Clippy and the
  release build pass; no Rust toolchain is available in the preparation host.

## v2.2.2-candidate.14 — Reconnect after observed idle EOF

- Before sending a capture request, discard a retained socket whose peer has
  cleanly closed. Reset its session evidence and establish one bounded fresh
  connection, reporting that connection as new. Never replay a request after
  sending begins; errors and unsolicited bytes still stop the capture.
- Integrate the repair through the existing sibling xSAR client without an API,
  wire protocol, dependency or server change. Keep native RGBA pixels, full-frame
  capture, game policies and input authority unchanged.
- Add deterministic socket regressions for idle EOF after a successful capture,
  cancellation, unsolicited idle data and response failure without request replay.
  Native Windows and live guest acceptance remain separate from host tests.

## v2.2.2-candidate.13 — Combined capture transport

- Use matching xSAR 0.4.2-candidate.2 and server 0.1.2 with explicit version-3
  full-screen BGR8 requests, validated response identities and opaque native RGBA
  reconstruction. Pixel payload is 25% smaller; latency remains to be measured.
- Retain one demand-driven TCP session per worker context. Drop failed sessions
  without retrying the capture or guest input; clear them on context changes,
  disconnect and shutdown. Reject non-native frames before analysis or recording.
- Expose explicit v2, v3 one-frame and v3 persistent comparison modes, separate
  monotonic stage timings and bounded rolling summaries. Numeric endpoints avoid
  unbounded DNS; connection cancellation can wait for the five-second connect.
- Preserve QMP input policy, current OK recovery, full Capture PNG and GAME WIN
  recording semantics. Windows compilation, native GDI behaviour and live
  five-game acceptance require the operator's separate verification.

## v2.2.2-candidate.12 — Interrupted OK confirmation

- Reserve the existing fourth, input-free OK observation when a valid second
  observation loses its exact-black central glyph on the third observation,
  while its settled body and consecutive geometry remain valid. The final
  observation waits 200 ms and must provide its own current glyph and matching
  geometry. Persistent washout still stops safely; recovery is not guaranteed.
- Preserve three ordinary observations, one irreversible OK-only reserve,
  exact RGB(0,0,0), current-frame click points, the initial 1000 ms wait and
  ordinary retry spacing. No fifth observation, extra score skip, alternative
  fourth-frame menu input or replay of uncertain input is authorised.
- Clear predecessor eligibility across gameplay, missing or moved bodies,
  malformed captures, resets and error/cancellation exits. Preserve current
  foreground OK evidence that blocks underlying New Game controls.
- Exercise production classifiers with existing native captures and explicitly
  controlled washout/fourth-response cases across all five controllers. The
  candidate.11 failure is corroborated by its retained log; its four original
  PNGs were unavailable, so recorded-sequence pixel verification remains pending.
- Keep capture transport, xSAR/server sources, dependencies, recording and
  mouse timings unchanged. Native/live acceptance is a separate Gate 3 check.

## v2.2.2-candidate.11 — Verification regression repair

- Correct the Klondike reserved-OK regression's expected New Game search count.
  Its K91 fixture is not classified as Level Up, so the confirmed-win fallback
  legitimately searches New Game on all three normal observations. Its correct
  totals are three searches and two recaptures; OK alone gets four and three.
- Assert the fixture precondition and that all three New Game searches precede
  the fourth OK search. Retain rejection of fourth-frame Score Counting and
  New Game, no input, and an unread fifth frame. Include logs on assertion failure.
- Preserve candidate.10 production behaviour, capture transport, dependencies,
  timing, recognition and input safeguards. Only the regression and application
  version change. No gameplay failure investigation is included in this repair.
- Run the previously failing regression first in the supplied verifier, before
  the full suite. Candidate.10's reported host run passed 825 tests and failed
  this assertion; verification stopped before the release build, leaving the
  previous candidate.9 executable in place.

## v2.2.2-candidate.10 — Bounded transition and late OK recovery

- Recover specifically from a TriPeaks post-action `ClickOffsetMismatch` with
  at most three delayed fresh captures, without card, Draw or Solver input.
  Keep the exact canonical-centre check, detector bounds and colour thresholds.
  Persistent mismatches and unrelated errors still stop; STOP remains active.
- Retain three normal control observations and add one final 200 ms OK-only
  confirmation when observation three already has a settled-size body and a
  current exact-black central glyph awaiting consecutive geometry agreement.
  The fourth frame must independently qualify. New Game and Play retain three
  observations; no fifth OK observation or additional click is permitted.
- Apply that bounded final confirmation consistently across all five game types.
  Keep the 1000 ms first post-score wait, existing early 200 ms confirmation,
  independent win evidence, foreground priority and uncertain-input safeguards.
- Retain the recorded Free Cell third-frame failure and native TriPeaks left
  tableau/DRAW captures. Add regressions for current-frame success, loss of
  evidence, persistent geometry rejection, cancellation and no input replay.
- Preserve TCP/QMP acquisition, matching xSAR, dependencies and GAME WIN-only
  diagnostic PNG writes. GAME WIN capture costs remain separately logged.
- This candidate installs over the exact candidate.9 source, with full-file
  backups and rollback. Local static, pixel and delivery evidence is included;
  Rust checks and live acceptance require the supplied Beast verifier.

## v2.2.2-candidate.9 — Earlier fresh OK confirmation

- Across all five game types, choose a 200 ms confirmation wait only when the
  current Level Up button has settled dimensions and a current exact-black
  central glyph, but still needs consecutive geometry agreement. Other missing,
  entering, tinted or pointer-only observations retain their normal spacing.
- Keep the 1000 ms first post-score wait and three total observations. The next
  frame must independently pass readiness before the sole OK click; the shorter
  wait creates no extra capture, click budget or replay of acknowledged input.
- Log the selected and normal retry intervals and confirmation eligibility.
  Clear eligibility on a new search, recapture consumption, reset or error.
- Retain native Pyramid and TriPeaks entering, clean and tinted captures. Add
  shared-policy and controller regressions for conditional waits, fresh points,
  lost evidence, STOP, bounded failure and no replay. Controlled repeated clean
  captures test decisions; they are not recorded live 200 ms outcomes.
- Preserve the integrated TCP capture path, legacy QMP option, GAME WIN-only
  image recording, independent win evidence, dependencies and matching xSAR.
  The separate TriPeaks board-transition issue is outside this change.
- Local syntax, pixel replay and delivery checks are recorded in the package.
  Rust build, tests, strict documentation, Clippy, release and live acceptance
  require the supplied Beast verifier and live GAME WIN runs.

## Unreleased — Guest TCP capture integration

- Merge the guest TCP capture path with current controls, bounded readiness and
  GAME WIN diagnostics. Use version-2 xSAR/server framing with `u16` stride.
- Default to `QMP_CAPTURE_BACKEND=tcp` and `QMP_CAPTURE_ADDR=127.0.0.1:9999`.
  Preserve legacy QMP capture with explicit `QMP_CAPTURE_BACKEND=qmp`; TCP errors
  stop the request without silently switching transports.
- Keep ordinary TCP capture in memory without PNG encoding when diagnostics are
  disabled. Encode manual snapshots from the same frame and retain exact bytes
  for Save. With `--image`, cache the latest PNG for entry evidence and preserve
  GAME WIN-only writes, acquisition timestamps and stopped/error evidence.
- Preserve QMP VM/pointer probes, input ownership, STOP, game/strategy context
  checks and existing control-search limits. Report TCP and PNG-encoding stages
  separately from detection and diagnostic file writes.
- Add local socket/recording regressions; live Windows capture, visual behaviour
  and latency remain to be validated on the intended QEMU guest.

## v2.2.2-candidate.8 — Diagnostic-only Level Up caption

- Remove the fully white fixed-caption veto from shared OK readiness across
  all five games. Keep caption measurements in diagnostics, explicitly logged
  with `caption_gate=false`.
- Continue requiring settled button dimensions, geometry agreeing within one
  pixel across consecutive fresh observations and a current five-pixel exact
  RGB(0,0,0) glyph in the central band, with pointer exclusion and rescan.
- Reproduce the supplied Spider entrance, near-black and grey-caption frames.
  The near-black RGB(1,1,1) glyph remains a miss; the third frame's stable body
  and exact-black glyph permit the sole OK click despite its fading caption.
- Add controller regressions for that third-frame decision, bounded failure,
  STOP and no replay after acknowledged OK. Preserve independent GAME WIN
  evidence, the 1000 ms first wait, three total observations and foreground
  modal precedence over New Game.
- Preserve GAME WIN-only image recording, capture transport, dependencies and
  matching xSAR `6765e0a` TCP protocol v2 code. No sibling source is changed.
- Local static, pixel and package verification is recorded in the delivery
  package. Rust compilation, tests, strict documentation, Clippy, release and
  live acceptance remain separate Beast verification gates.

## v2.2.2-candidate.7 — Settled Level Up readiness

- Require stable measured OK button geometry across consecutive fresh frames
  and a fully presented fixed “You've Reached” caption on the current frame
  before the single Level Up click, across all five games.
- Keep the 1000 ms first post-score wait and three total control observations.
  Pending animation consumes that existing budget; no nested retry loop or
  extra input attempt is introduced. A missing black glyph may still supply
  geometry evidence, but cannot authorise input.
- Use the unchanged xSAR exact-black scanner within the settled button's glyph
  band for the final coordinate, excluding the pointer-outline match observed
  in the supplied Free Cell failure. Preserve separate game-win evidence,
  fresh probes, STOP checks and forward-only menu input.
- Prevent a pending foreground OK from exposing New Game underlay ink or
  renewing the shared controller's score-skip allowance.
- Add byte-original entrance, fading and settled native captures and controller
  regressions for third-frame readiness, bounded failure and no OK replay.
  Log visual readiness separately from the existing black-scan metrics.
- Include the three candidate.5 diagnostics files omitted from published
  401a35e; GAME WIN-only image scope, capture transport and dependencies remain.
- Local syntax/pixel/installer checks are recorded in the delivery package.
  Rust compilation, all tests, private rustdoc, Clippy, release and live
  acceptance require the supplied Beast verifier and live GAME WIN runs.

## v2.2.2-candidate.6 — Strict Clippy verification repair

- Use `inspect_err` for four cancellation paths which update the control-search
  outcome and propagate the original error unchanged.
- Combine the nested recapture condition without changing when a capture is counted.
- Document private control-search counters and terminal-search test helpers.
- Merge identical unsupported-frame branches in the mock-QMP scenario source,
  retaining the same conditions, frame bytes and accounting.
- Preserve GAME WIN-only image recording, the 1000 ms post-score wait, three
  search attempts, detector geometry, input safeguards and dependencies.
- Candidate.5 passed Beast Cargo check, all tests including ignored socket tests
  and strict private rustdoc. Clippy stopped verification before the release build.
  The repaired candidate requires its own strict verification and live acceptance.

## v2.2.2-candidate.5 — GAME WIN image evidence

- Add `--image` to retain the entry PNG and every subsequent capture only during
  GAME WIN transitions, through freshly verified next-game Solver/HALO readiness.
  Ordinary gameplay captures are not written to the diagnostic image directory.
- Reset `/home/charlie/tmp/images` at each application start. Recreate it only
  with `--image`; retain all transition PNGs until the next application start,
  including evidence from stopped or failed transitions.
- Save original QMP PNG bytes with elapsed-time filenames and a unique serial.
  Correlate each filename, acquisition interval and write duration in the text
  log using the same monotonic startup clock. Timestamp worker messages when
  emitted so delayed UI polling cannot distort the sequence timing.
- Stop transition input if recording fails; add regressions for startup cleanup,
  directory boundaries, scoped recording, failed transitions and timestamping.
- Preserve the 1000 ms post-score wait, three control searches, game recognition,
  input safeguards, dependencies, xSAR and capture transport.
- Beast compilation, all tests and strict private documentation passed on
  14 October 2026. Strict Clippy failed; release build and live acceptance were
  not reached by that verifier run. Candidate.6 repairs the identified lint issues.

## v2.2.2-candidate.4 — Earlier Level Up capture and three control searches

- Reduce the shared score-skip-to-Level-Up wait from 3000 ms to 1000 ms.
- Give absent OK, New Game and Play controls three total searches across all
  five game types: the initial frame plus at most two fresh recaptures.
- Centralise the attempt count and update Params and controller diagnostics.
  Preserve existing retry spacing, click budgets, positive reward/gameplay
  recovery, independent win evidence and uncertain-input safeguards.
- Extend regressions for third-attempt success, exhausted three-frame lookups,
  latest-frame retention, cancellation and no input replay. A synthetic washout
  removes only exact-black OK ink from native Level Up pixels after confirmation.
- Keep xSAR, the Windows screen-grab server and capture transport unchanged.
- Rust compilation, tests and live game acceptance remain pending; the delivery
  record separates completed local checks from required Beast verification.

## v2.2.2-candidate.3 — Forward-only menu transitions and diagnostic previews

- Fix Pyramid's detected New Game being overridden by an incidental black run
  in the previous OK region. After a delivered menu click, search the next
  expected control without replaying earlier menu inputs.
- Prefer New Game when the optional Level Up step is absent. Preserve separately
  verified foreground TriPeaks modal context before accepting underlying ink.
- Keep one input-free recapture for an absent expected control, with accurate
  failure wording and the latest restart frame explicitly marked diagnostic
  for display after an uncertain stop.
- Add mock-QMP coverage for New Game plus an OK-region distractor, missing New
  Game with a distractor, and direct New Game entry without Level Up.
- Carry forward the candidate.2 warning cleanup. Search calibration and xsar
  remain unchanged. Rust tests and live acceptance of this fix remain pending.

## v2.2.2-candidate.2 — Shared terminal controls and search metrics

- Use the existing xsar black-run scanner for expected OK, New Game and Play
  controls across TriPeaks, Pyramid, Klondike, Free Cell and Spider. Carry the
  measured full-frame point through to input rather than retaining fixed clicks.
- Centralise bounds and run lengths in per-game constant blocks in
  `terminal_controls/profiles.rs`. Keep a wider vertical OK window, tighten
  static New Game/Play windows per game, and use the selected block for search,
  retry, logs and the Params panel. Keep xsar independent of all game layouts.
- Log each attempted scan's bounds, result, pixel/row counts and scan duration.
  Emit a per-lookup summary of searches, attempted recaptures, second-grab flag,
  accumulated scan time, elapsed wall time and outcome through both log sinks.
- Give missing expected controls one fresh capture retry and retain the latest
  diagnostic image on failure. Keep independent game-win evidence and existing
  positive transition/recovery policies separate from control location.
- Add native control, telemetry, retry and measured-input regressions. Python
  calibration passes all 28 available terminal captures; application Rust tests
  and live acceptance of the expanded integration remain pending.
- Reuse xsar `5fa8a1a` unchanged. TriPeaks/Pyramid New Game and Play lack native
  positive screenshots in the repository and require live verification.
- Remove obsolete dialog-gold detectors, their calibration constants and their
  retired test coverage. Remove unused worker imports and import observation
  limits directly in tests. Search bounds, matching and input behaviour are unchanged.

## v2.2.2-candidate.1 — Spider Level Up black-run search

- Search only Spider's expected OK region for five consecutive exact-black RGB
  pixels, visiting rows bottom-up and each row left-to-right. Click the returned
  full-frame coordinate instead of a fixed Level Up point.
- Retry a missing post-score OK once with a fresh capture, while still allowing
  the optional Level Up screen to be absent when New Game is ready. Retain the
  latest failed capture in the egui diagnostic preview when the retry fails.
- Send every OK search boundary, result and attempt through the session and
  on-screen log path. Keep other games and capture transport unchanged.
- Consume the sibling xsar 0.4.2-candidate.1 helper and add the native Spider
  level-505 screenshot plus detector and worker regressions.
- Charlie confirmed successful live Spider operation on 13 October 2026 and
  published the application checkpoint as `7c6c6fb` with xsar `5fa8a1a`.

## v2.2.0 — Explicit card values and schema-2 export (development)

- Consume xSAR 0.4.0's schema-2 writer with exact `card`, integer `value` and
  `suit` records. A=1 through K=13; suits remain part of every captured identity.
- Keep numeric mapping and strict schema validation in xSAR, including schema-1
  reading and refusal of contradictory labels, values, suits or mixed shapes.
- Preserve the original accepted v2.1.0 live export; a separately published
  schema-2 conversion changes representation only, not acquisition provenance.
- Preserve all five CV controllers, native profile v4, STOP/context checks,
  restoration and no-overwrite publication. Console intake and search stay separate.
- Pin the promoted xSAR 0.4.0 implementation at
  `ab226a65d3bbc2b4914a5cecfb2d3247364c7536` and record its Git source in the
  lockfile. Ordinary builds require no local source override.

## v2.1.0 — Pyramid reconnaissance to JSON (development)

- Add a separately confirmed, bounded Pyramid scan; ordinary independent
  preparation remains read-only and all five CV controllers retain their policy.
- Read native rank/suit evidence in logical tableau order and first-draw stock
  order, require proven advancement and positive exhaustion, then verify the
  complete Undo All modal and original-board restoration before publication.
- Preserve STOP/context revocation, uncertain-input non-replay, original PNG
  diagnostics and the existing coalesced capture preview.
- Consume xSAR 0.3.0's checked complete-deal model, strict JSON codec and atomic
  no-overwrite publisher through an external paired-development override.
  Crate promotion, exact consumer repin and live acceptance remain separate gates.
- Retain original native calibration provenance and explicitly distinguish
  fixture/controller tests from successful live acquisition. No route search,
  console changes or Spider XP change is included.

## v2.0.10 — TriPeaks terminal acquisition margin

- Give qualified TriPeaks final-card terminal acquisition a separate allowance of
  30 total post-action captures instead of the shared 20-round limit.
- Preserve terminal masks, two same-kind positives, the four-capture confirmation
  allowance, STOP, effect proof and uncertain-input non-replay; other game and
  shared post-game limits remain unchanged.
- Retain the original TP03 Level Up screenshot as positive evidence with accurate
  late-capture provenance, and cover late candidates, exhaustion and cancellation.
- Deliver a guarded upgrade from the exact installed v2.0.9 state at its established
  base; dependency pins and the xSAR boundary remain unchanged.

## v2.0.9 — TriPeaks direct Level Up transition

- Recognise the supplied Level Up modal covering a TriPeaks reward panel after a qualifying final-card action.
- Require the existing material effect proof and two fresh confirmations of the same terminal type before entering the guarded restart flow.
- Enter an already visible Level Up stage at its existing raised OK control without a preceding centre click; skip a subsequently revealed reward panel only after fresh positive confirmation.
- Retain bounded observation, fresh input checks, STOP and uncertain-delivery refusal; add native screenshot and controller regressions.
- Bump only the application version from 2.0.8 to 2.0.9; dependency pins remain unchanged.

## v2.0.8 — TriPeaks reward transition and solving-method documentation

- Recognise the supplied TriPeaks Congratulations reward overlay after a final top-row card action, retaining the existing effect threshold and requiring two consecutive fresh terminal observations.
- Enter the bounded post-game sequence from confirmed reward evidence instead of rejecting the overlay as an unsupported gameplay scene. Require fresh recognised reward evidence before each reward-skip attempt; preserve STOP and uncertain-input non-retry.
- Restructure the README around Computer Vision solving and Shortest path route calculation, with their current capabilities, a general process diagram and a closing description of each of the five game types.
- Use plain release version 2.0.8 in Cargo metadata, window title and session logs. The lockfile changes only the application version; the xsar revision and all dependency versions remain pinned.

## v2.0.0, candidate 7 — CLASSIC baseline and Solver startup recovery

- Share the established CLASSIC-blue colour predicate between Klondike and Spider, including Spider stock occupancy and Draw checks. Keep card-back colour separate from scene, Solver and closed-HALO authority.
- Permit explicitly requested TriPeaks and Pyramid runs from a no-HALO preview, with bounded input-free observations and at most one Solver activation on a positively recognised, non-empty board.
- Preserve concrete-preview validation, game-specific action effect checks, STOP and uncertain-input non-retry. Initial strategy, mode-change and manual captures remain read-only.
- Order the Game Type menu as Klondike, Spider, Free Cell, Pyramid, TriPeaks.
- Correct RGB literal formatting in Klondike rustdoc. Report back/felt counts for an unclassified highlighted stock rather than claiming its game profile is missing.
- Establish CLASSIC as the acceptance baseline for all five game types. Pyramid and Free Cell require no card-back detector. Native CLASSIC-blue Spider Draw evidence remains pending; synthetic palette regressions do not claim live calibration.

## v2.0.0, candidate 6 — Klondike red card backs

- Recognise the supplied red card-back artwork in existing Klondike tableau scene strips and highlighted stock interiors, alongside the accepted blue backs.
- Restore the existing bounded Solver recovery for recognised red-backed gameplay without a HALO, and recognise the supplied column-seven Jack HALO without changing its thresholds or geometry.
- Retain read-only initial captures, candidate 5 live previews, STOP, existing action budgets and Free Cell shifted OK recognition.

## v2.0.0, candidate 5 — live capture previews

- Publish intermediate TriPeaks, Pyramid and Klondike QMP captures to the existing bounded latest-frame display slot, including delayed and post-game observations.
- Publish diagnostic pixels before analysis without granting input authority; retain candidate 4 Free Cell OK recognition.

## v2.0.0 — candidate 4

- Recognise Free Cell's expected local Level Up OK at its original location,
  19 pixels higher (FC22) and 12 pixels lower (FC23). Preserve the caption/body
  thresholds, fixed click, optional Level Up flow and independent win entry.
- Keep Spider's accepted OK checks explicitly at its existing offsets 0/-19,
  so Free Cell's lower layout does not change Spider policy.
- Add original supplied PNGs and focused terminal-control regressions. Preserve
  all gameplay, strategy selection, timings, xSAR pin and exact-byte PNG saving.

## v2.0.0 — candidate 3

- Give the two startup strategy boxes the same available width and height,
  with shared padding and wrapping descriptions when the window is resized.
- Retain the promoted xsar dependency, strategy choices and controller behaviour.

## v2.0.0 — candidate 2

- Pin xsar v0.2.0 to Charlie's promoted Git commit
  `1a719b359a51d1e1e3113224193a779c76de82f8` and record the actual Git source
  in Cargo.lock. Ordinary Cargo builds no longer need a local override.
- Update current build instructions and release identity; retain the startup
  strategy split, shared primitives and all accepted game-controller behaviour.
- Deliver complete application files against the same exact base, accepting
  the recorded installed candidate 1 as the guarded preimage and rollback state.

## v2.0.0 — candidate 1

- Add startup strategy selection before any QMP request. Keep the five CV game
  types separate from a read-only independent Pyramid preparation workspace.
- Tag requests, frame/snapshot mailboxes and status/run events with strategy,
  game, socket and generation; reject stale results after idle switches.
- Retain exact original PNG preview/save and shared capture/logging controls;
  preparation does not perform detection or permit guest input.
- Extract actual QMP, geometry, PNG/RGBA and predicate-scan implementations
  into optional xsar features and use them through application adapters.
- Keep accepted game controllers, calibration, timings and terminal policies.
  Recognition, reconnaissance and route search remain later work.
- Deliver a guarded local paired-validation override; the final application
  dependency pin follows Charlie's promotion of the tested xsar commit.

## v1.4.0 — candidate 4

- Move Spider's pointer to the measured clear-felt point at (20, 500)
  after an acknowledged source-click release, before the existing editable
  settle and fresh capture. DRAW does not park; one source remains one action.
- Add a checked movement-only QmpClient helper. The auxiliary park sends one
  QMP command containing two absolute-axis events, counted separately from
  the source click; it sends no button/key event and adds no delay.
- Treat active Spider Solver as disabled: no refresh click. Missing HALOs use
  bounded input-free captures and independent completion checks. Inactive
  Solver retains one start wait, fresh capture and one activation.
- Retain candidate 3's Free Cell side-margin presence checks and Spider's
  higher local LEVEL UP OK, with all timing defaults and other modes unchanged.
  Initial/read-only/manual captures remain input-free and PNG saving exact.
- Record the confirmed cursor-overlap report and supplied-frame simulation;
  the exact rejected runtime PNG is unavailable. No card/effect matching or
  gold-threshold relaxation is introduced. Native Rust and live QMP remain
  pending Beast validation.
- Pin this cumulative complete-file update to the same pushed candidate 2 at
  `aae11dee2792967cb375b368270e46c8a95e8c95`.

## v1.4.0 — candidate 3

- Correct Free Cell toolbar-clipped source detection when a central card pip
  covers the paper probe. Use the visible left and right card margins instead;
  retain closed tops, connected exterior rails and the Y947 exclusive cutoff.
- Add byte-identical FC20/FC21 evidence for a five-card PLAY 6 run and a single
  PLAY 7 card. Both bottom cards are eight of diamonds; their centre paper
  probes contained 139/289 bright pixels, just below the old 50% requirement.
- Keep source clicks, action timings, Solver recovery, deterministic terminal
  flow and all other modes unchanged. No card identity or move-effect matching.
- Accept Spider's supplied LEVEL UP OK caption 19 pixels above the original
  Free Cell location. Reuse only the local button/word check; retain the existing
  click point, optional-level branch and one-shot terminal progression.
- Add original SP20 evidence and restart coverage for both OK positions. Level,
  medal, title, panel colours and fireworks remain outside readiness checks.
- Pin this complete-file update to pushed candidate 2 at
  `aae11dee2792967cb375b368270e46c8a95e8c95`.

## v1.4.0 — candidate 2

- Raise the editable Spider card/run settle default from 750 to 1250 ms at
  Charlie's request after successful Beast play, allowing stack animation to
  finish before fresh HALO detection. Keep DRAW/start/observation defaults.
- Correct the continuous restart test's equal-duration wait count: score-skip
  and default game-start waits are both 3000 ms and serve different purposes.
  Check the ordered waits using a distinct configured start delay, with or
  without LEVEL UP and with inactive or already-active Solver.
- Preserve the accepted runtime detector, terminal sequence, source actions,
  cancellation and capture behaviour; no card-effect matching is introduced.
- Pin this complete-file update to pushed candidate 1 at
  `5a0f730a4248d030ae98fb466a00b092b8083133`. Native verification remains a
  Beast check; delivery records distinguish it from executed source audits.

## v1.4.0 — candidate 1

- Add Solver-led Spider with ten dynamic PLAY columns, DRAW via D and one click
  per highlighted card/run. Completed runs pack automatically; collapsed suits
  are never input targets. No card identity or changed-pixel effect proof.
- Handle complete and clipped source outlines above the measured Y947 toolbar
  boundary, including connected overlapping outlines and changing compression.
  Disappearing stock does not imply a win or retain a permanent input region.
- Add independent editable card, DRAW, observation and game-start waits, initially
  750/2000/1000/3000 ms. DRAW reflects Charlie's observed 1–2 second deal wait.
  Multi-Step starts at continuous 0; observations and recovery remain bounded.
- Reuse accepted local terminal controls for score, optional OK and New Game;
  Spider owns its Play button 100 px lower and its one-board restart policy.
- Include twenty unchanged supplied PNGs, hash manifest and focused regressions.
  Preserve earlier modes, STOP, uncertain-input non-replay and exact PNG saving.
- Pin complete-file installation to `4298fe96079f9283159a0b1200876f97de047d7f`.
  Native Rust and live QMP checks remain mandatory Beast validation.

## v1.3.0 — candidate 5

- Accept fully closed Free Cell PLAY outlines ending at the Y947 exclusive
  cutoff. Their rounded rails terminate before the closing bottom crossbar;
  remove the gap between complete-source and toolbar-clipped-source policies
  without changing gold thresholds or reading toolbar pixels.
- Set the independent editable Free Cell game-start default to 3000 ms,
  reflecting Charlie's live deal timing. Other timings and recovery flow stay
  unchanged.
- Add unchanged supplied FC18/FC19 PNGs and focused boundary regressions.
  Preserve candidate 4 clipped sources, optional LEVEL UP and Solver recovery.
- Pin the guarded installer to pushed candidate 4 at `2345898e8a59e37f065d93ebda51726af0a8887f`.

## v1.3.0 — candidate 4

- Enable evidenced toolbar-clipped Free Cell PLAY sources using only visible
  source geometry and card paper above Y=947; never infer the hidden lower edge
  or read toolbar pixels for source authority. Connected runs remain one click.
- Add an independent editable game-start wait, default 1000 ms as confirmed by
  Charlie, followed by fresh capture before inactive Solver activation.
- Restore one bounded Solver refresh for a supported no-HALO board after an
  input-free re-observation and independent GAME WIN check. Report observed
  Solver state; uncertain input is not replayed.
- Add original FC14/FC16/FC17 evidence and focused regression coverage. Preserve
  candidate 3 optional LEVEL UP, shared capture and accepted earlier modes.
- Pin the guarded installer to pushed candidate 3 at `676afb477495a166a67132a5735b2347b343585d`.

## v1.3.0 — candidate 3

- Make Free Cell LEVEL UP optional after score counting. Each fresh frame checks
  the local OK and New Game controls; New Game can proceed directly when no
  level was earned. Ambiguous or absent controls do not authorise input.
- Preserve one-shot terminal clicks, bounded input-free observations, STOP and
  uncertain-input non-replay; New Game → Play → fresh board remains ordered.
- Correct the test-only constant RGBA chunk loop to `as_chunks_mut::<4>()`,
  addressing the reported Clippy lint without suppressing warnings.
- Add byte-identical FC15 evidence and focused optional-LEVEL-UP regressions.
- Pin the guarded installer to pushed candidate 2 at `656881ee3f906347bd6f5665c733892621b1cc98`.
  Runtime gameplay detectors, other modes, timings and capture are unchanged.

## v1.3.0 — candidate 2

- Enable the approved Free Cell Solver-led slice from original FC01–FC13 PNGs:
  CELL sources first, then PLAY sources bottom-up, with one bottom-card click
  per solid highlighted block. Dashed landing guides are not actionable.
- Observe automatic SUIT transfers without card identity, changed-pixel proof
  or source-position comparison. Activate visibly inactive Solver once; an
  active Solver with no HALO receives bounded input-free captures.
- Add an editable delayed-observation allowance, initially 20, bounded 1–100.
  Preserve independent 750/1000 ms Free Cell timings and continuous budget 0.
- Recognise independent one-board GAME WIN entry from the score/skip panel or
  completed-game New Game panel. Continuous play follows expected local
  OK → New Game → Play controls, fresh board and Solver activation. Finite runs
  do not restart. Level, medal, fireworks and surrounding panel colours do not
  gate these expected controls.
- Keep source probes and clicks above Y=947. Optional FC14 is absent, so a run
  with no complete visible lower outline remains unsupported. No SUIT-source,
  Draw, Recycle or Solve input is enabled.
- Preserve existing modes, STOP, uncertain-input non-replay and exact-byte PNG
  saving. Pin the installer to `44727193d4e620a749925e9cc46c9d9d4df1700f`.
- Native Rust validation is unavailable in the delivery workspace; the review
  distinguishes pixel rehearsals and installer checks from unrun Rust tests.

## v1.3.0 — candidate 1

- Add a selectable read-only Free Cell calibration mode and independent profile.
  Agree CELL, PLAY and SUIT names for future calibration. No Free Cell HALO detector,
  gameplay input, Solver activation or completion/restart automation is enabled.
- Add independent future Free Cell settle/re-observation settings, defaulting
  to 750/1000 ms, and a zero continuous-action budget, with existing bounds.
  Editing these values does not authorise guest input.
- Preserve the accepted Klondike live Y<947 tableau boundary and Solver-led
  action cycle, Pyramid's highlighted MOVE/Recycle D shortcut, TriPeaks delayed
  no-HALO recovery, STOP and original-byte PNG saving.
- Consolidate current architecture and input contracts while retaining dated
  candidate evidence. Add the FC01–FC13 native screenshot intake checklist.
- Base the candidate on `146fe173f6e123a92e2eadf518523715d042cd66`. Charlie
  reported successful candidate-11 overlap recovery and a complete Klondike
  GAME WIN sequence before pushing that base on 4 October 2026.

## v1.2.6 — candidate 11

- End live Klondike tableau HALO detection before the measured guest toolbar
  boundary at Y=947. Undo All, Hint and other artwork below that row no longer
  participate in source recognition.
- Recognise a boundary-clipped source from its visible closed top, connected
  opposing rails and card paper. Keep one upper-card click for the whole block;
  record only its visible bounds rather than guessing its hidden lower edge.
- Add the supplied five-card source fixture K95 and live detector/controller
  checks. Retain historical pixel-effect diagnostics separately in tests; they
  do not gate gameplay or restore toolbar-dependent live detection.
- Preserve candidate 10's Pyramid/TriPeaks corrections, one-board restart
  sequence, timings, STOP and original-byte capture/save contract.
- Rebuild the guarded candidate package against pushed candidate 10 at
  `66c779920a1a57d183f07725943e881f689ed3c6`.

## v1.2.6 — candidate 10

- Pyramid's freshly highlighted MOVE/Recycle target sends D. Both operations
  share the confirmed shortcut; card and Left/Right targets retain their clicks.
- TriPeaks uses three delayed input-free no-HALO observations before one Solver
  recovery, then three further delayed observations before stopping. Its new
  Params interval defaults to 1000 ms and is frozen for each run.
- Once a one-board Klondike win is independently confirmed, expected OK,
  New Game and Play controls use local button readiness. Rank, medal, title,
  frame and fireworks signatures do not gate those ordered clicks.
- Handle the measured Undo All overlay with one bounded lower-border envelope
  and positive icon/source evidence, retaining ordinary border requirements
  outside that envelope. No card-pixel effect checks gate Klondike play.
- Add output-only text sizing while preserving Copy Output and existing
  2000-entry retention and three-game visible-log rollover.
- Retain candidate 9's Hint-shadow correction. Installation supports exact
  base or the complete pinned candidate 9 predecessor, with guarded rollback.

## v1.2.6 — candidate 9

- Recognise both measured Hint-shadow lower-edge positions in column four.
  Keep all 96 lower-edge samples positive, existing rail brackets, closed top,
  card interior, scene context and safe upper-card click.
- Add two complete uploaded PNG fixtures and source/controller regressions.
- Keep fresh-HALO execution, Solve, one-board completion/restart, timings and
  accepted TriPeaks/Pyramid behaviour. No live card-pixel effect check is added.
- Deliver complete affected files against pushed candidate 8 at `cafdd609`.

## v1.2.6 — candidate 8

- Measure the clear seven-pixel column-five/six scene gutter core for the three
  captured column-five source HALOs, preserving196 samples and the 95% felt threshold.
  Source-outline, click, capture and HALO-driven execution policy are unchanged.
- Recognise the independently measured Master medal LEVEL UP layout using
  complete semantic/control/frame/completed-board evidence and the existing
  OK point. Preserve both earlier layouts and input timing; add diagnostics.
- Add seven original-byte native fixtures and adverse/controller regressions;
  restore originalK69 because its old sparse mask omitted part of the new probe.
- Deliver complete affected files against pushed candidate7 at exact449004d.
  No live card matching, formatter, warning suppression or other-mode change.

## v1.2.6 — candidate 7

- Derive Klondike source-height capacity from the existing rows 332..998 scan
  envelope instead of the earlier 604-pixel observed maximum. K80's complete
  ten-card source is 620 pixels tall and passes every other existing guard.
- Preserve scan coordinates, colour predicates, connected rails, closed edges,
  paper/scene checks and one upper-card click. No card-effect matcher or
  controller policy is added; Solve, one-board restart and other modes are unchanged.
- Add two byte-identical native PNG fixtures, detector boundary/adverse tests
  and finite-run controller coverage using fresh recommendations. Keep manual
  Undo evidence separate from unsaved automatic worker frames.
- Deliver complete affected files against committed candidate 6 at exact
  329d806; accept only that base or a complete installed candidate 7 state.

## v1.2.6 — candidate 6

- Follow fresh canonical Klondike Solver recommendations without source/recipient
  card matching. The preview is advisory; each acknowledged action consumes one
  budget slot, settles and captures anew. Keep previous effects explicitly unproven.
- Continue repeated fresh HALOs as new logical actions; finite Step Once stops
  after one action and fresh result. Missing HALOs use bounded read-only captures,
  independent completion and one scene-validated Solver refresh per context.
- Move the previous card-effect matcher to test-only diagnostics; retain its
  historical native/adverse tests without a live execution gate or warning allow.
- Recognise K75's thin column-4 HALO through the measured Hint toolbar shade.
  Require positive shadow-gold on the bounded rail band and two lower-edge
  samples, ordinary paired bracketing and the complete remaining outline.
- Recognise the fresh post-Play deal using positive card-paper perimeter
  geometry, preserving fresh-deal topology, inactive Solver and scene guards.
  The King-diamonds artwork no longer prevents the existing one Solver click.
- Add six original-byte PNG fixtures and native/adverse controller regressions.
  Keep the first no-HALO report separate from the earlier post-click source
  verification stall; its source thresholds no longer gate live Klondike play.
- Deliver against the same committed candidate-2 base at exact 7d869c2.
  Accept complete base, delivered candidate 5 or candidate 6 content states,
  preserving unrelated work and rollback to the actual backed-up before-state.


## v1.2.6 — candidate 5

- Add separate tableau-to-SUIT continuation when the old source becomes the
  next landing guide. Require independently located exposed header, opposed
  positive source strips and uniquely received old artwork in a bright SUIT face.
- Reject neutral guide whitening, retained old print, ambiguous receivers and
  incomplete geometry. Keep ordinary effect thresholds and clipped-corner refusal.
- Add four original-byte PNG fixtures and native/adverse controller regressions.
  Preserve the distinction between the first manual pair and its logged frames.
- Retain candidate 4's source, SUIT-return and Pro Level Up corrections; input,
  timing, Solve, one-board restart and accepted other-game policies are unchanged.
- Deliver against the same exact 7d869c2 base, accepting complete base or
  delivered candidate 4 contents and restoring the actual backed-up before-state.


## v1.2.6 — candidate 4

- Add separately logged SUIT-return continuation when nearly identical foundation
  faces hide most source changes inside the existing cursor exclusion. Require
  paired opposite-corner print, stable paper and independently located newly
  received tableau artwork. Complete action effect remains unverified.
- Recognise the separately evidenced Pro Level Up artwork with complete dialog
  and completed-board guards. Reuse the existing measured OK point and timing;
  the reported failure had sent no OK input.
- Recognise the newly evidenced column-3 and column-7 source borders through
  row 997; bound shadow-gold rail support to its measured column-7 band.
  Keep input/effect limits, complete outline guards and existing height limit.
- Add K65-K69 native regressions and adverse/controller cases; preserve ordinary
  material thresholds, input non-replay and the confirmed one-board restart flow.
- Deliver against committed candidate 2 at exact 7d869c2, accepting a complete
  base or installed candidate 3 r2. Back up and restore the actual before-state.


## v1.2.6 — candidate 3

- Packaging revision r2 targets committed candidate 2 at 7d869c2 after the
  previous f916-based package refused installation. Rebuild all base/content
  records and report expected/actual HEAD; Rust code and fixtures are unchanged.

- Recognise the measured long column-6 source with two positively shadowed gold
  edge samples beside Undo All, retaining the complete visible-edge requirement.
- Add bounded card-relative proof for clipped single-card stack reflow; retain
  material/printed/paper/cursor guards and log continuation separately from
  complete effect verification.
- Support the measured ordinary replacement face after a clipped-card transfer,
  using unique geometry and independent recipient print for continuation only.
- Add separate positive neutral-paper to chromatic-felt continuation for the
  queen and King transfers; combine disjoint measured shade ranges under the
  unchanged source/destination bounds, leaving full effect unverified.
- Reposition one lower scene probe inside the measured clear gutter; retain its
  196-pixel area, 95% felt requirement and all previous scene classifications.
- Add K54-K64 native regressions. Packaging r2 applies to committed candidate 2
  at exact base 7d869c2 and supports rollback to that base. Preserve confirmed
  GAME WIN flow.


## v1.2.6 candidate 2 — 3 October 2026

- Rebuild complete-file installer authority against f9167a0, whose package remains
  v1.2.5. Candidate 1 had refused installation after HEAD changed; later tests and
  the launched binary still used the old source.
- Include the prior measured New Game pointer-probe correction and bounded
  pending-Solve recapture policy, with a distinct candidate 2 runtime label.
- Preserve Charlie's committed README purpose edits. Add read-only installed
  candidate verification before build/test/run commands.
- Record the separate sparse hearts source failure and transient Undo evidence;
  no gameplay effect threshold is relaxed. Beast acceptance remains pending.

## 1.2.6 — 2026-10-03 (candidate 1; Beast verification pending)

- Move one Home button-body colour probe outside the measured pointer shadow
  after Level Up OK; preserve six required samples, tolerance and click points.
- Defer lower-priority tableau/SUIT input during a strongly recognised pending
  Solve face, using at most three editable delayed read-only captures.
  The original full Solve availability check remains the only click authority.
- Preserve fresh initial preview approval, action budgets, STOP and uncertain
  input handling; add native K53 and controlled adverse/controller regressions.
- Deliver complete files against pushed main
  8125c9df1dd87c7d8e9ed0e928fbb7fdd7bae82b with guarded preview/backups/rollback.

## 1.2.5 — 2026-10-03 (candidate 1; Beast verification pending)

- Recognise Solve from stable interior/lettering with empty-stock and scene guards,
  independently of its animated outer ring; retain full artwork diagnostics.
- Add source-supported continuation for the measured bottom 4-clubs transfer
  followed by a different tableau source, with stable paper/printed-detail guards.
  The prior complete effect remains unverified; generic thresholds are unchanged.
- Recognise the coherent two-pixel Level Up label shift and log terminal guards.
- Continue active Multi-Step 0 from a fresh target after one initial Solver
  activation; Single Step/finite recovery still returns the preview for review.
- Add K50–K52 native-coordinate regression evidence and adverse/controller cases.
- Deliver full affected files and a guarded installer against exact pushed main
  eb18478b59b29b00bd5c1e11f1a30348226a5f61. No predecessor candidate authority.

## 1.2.4 — 2026-10-03 (candidate 2; Beast verification pending)

- Recognise K44's measured horizontal tableau-source displacement using a second
  bounded scan with the existing closed-outline and safe-click requirements.
- Re-observe unsupported post-action scenes input-free within the existing shared
  three-capture recovery budget, preserving the pending action and STOP.
- Allow the measured contracted bottom-card source to support qualified fresh-HALO
  continuation through spatially aggregated bidirectional print evidence; keep
  complete-effect verification unchanged and the previous move unverified.
- Replace a variable Level Up text-shadow probe with stable artwork, permit the
  measured warm fireworks only at four context points, and require mixed OK
  lettering/background evidence.
- Record exact Solve stock/artwork/glyph counts and selected target on each run
  observation. K48 already passes the recogniser; no further colour relaxation
  or claim that the unseen live refusal is resolved.
- Add K44–K49 native-coordinate regressions and adverse/controller coverage.
- Deliver complete files against the same exact base or complete candidate 1,
  with guarded preview, backups, content verification and rollback.

## 1.2.4 — 2026-10-02 (candidate 1; Beast verification pending)

- Recognise the evidenced 604-pixel ten-card source and its closing edge beneath
  Undo All, keeping click/effect bounds above the toolbar.
- Accept the measured positive red warming of the Solve border while preserving
  inner artwork, glyph, scene, empty-stock and selection-priority checks.
- Use the existing post-game observation bound after acknowledged Solve instead
  of the short card-recovery budget; require two consecutive fresh win positives.
- Add native K41–K43 regression evidence and negative/controller coverage.
- Preserve timings, one-board Klondike completion, existing terminal controls,
  TriPeaks/Pyramid behaviour, STOP, uncertain-input handling and original PNGs.
- Deliver complete files against d9ad2cdf536165bb99ae4e02dabc49f7a9fa7227 with
  guarded preview, backups, content verification and rollback.

## 1.2.3 — 2026-10-01 (candidate 5; Beast verification pending)

- Continue from the recorded bottom-card replacement after the remaining tableau
  stack spreads, using two full visible proof patches above the toolbar.
- Preserve ordinary clipped-corner refusal, directional/paper thresholds and
  the 512-pixel source gate; keep this new route separate from complete-effect
  verification because the next HALO's shading is not receiver proof.
- Add the native before/result pair and adverse source, receiver, patch, shading,
  cursor, gold, toolbar, material and geometry checks.
- Preserve candidate 4 continuation, Solve, budgets, timings, STOP, shared modes
  and exact-byte PNG saving; deliver exact candidate 4 upgrade and rollback.

## 1.2.3 — 2026-10-01 (candidate 4; Beast verification pending)

- Continue from a supported fresh Klondike recommendation after an acknowledged
  card-source action when its existing source proof passes; keep the previous
  effect unverified if independent receiver evidence is insufficient.
- Permit an independently recognised next Solve control to become the next plan,
  retaining one-shot input, its separate settle and independent completion checks.
- Count each acknowledged action against finite limits and report verified and
  continued operations separately; unchanged cards and uncertain input stop.
- Recognise the measured nine-card run closing at rows 988–990, accounting only
  for the exact Undo All overlap while preserving complete-border and input bounds.
- Add the real black-pip RIGHT pair, Solve priority and deeper-run pixel fixtures
  with adverse controller/geometry checks and explicit runtime limits.
- Deliver complete guarded candidate 3 upgrade and rollback to its actual contents;
  preserve timings, continuous defaults, shared modes and original PNG saving.

## 1.2.3 — 2026-10-01 (candidate 3; Beast verification pending)

- Recognise the measured 3-hearts foundation recipient change under a persistent
  dark landing guide, preserving the ordinary source/destination thresholds.
- Extend fixed upper-card replacement-corner evidence to existing SUIT-source
  returns, adding guarded bidirectional printed-detail changes over stable bright
  paper with separate destination proof and explicit synthetic-test limits.
- Add the actual reconstructed RIGHT pair with native-pixel provenance and
  adverse guide, mask, source-only and destination-only checks.
- Retain candidate 2 completion, continuous defaults, editable timings, STOP,
  warning fixes and established TriPeaks/Pyramid behaviour.
- Deliver a guarded exact candidate 2 upgrade and rollback to its actual contents.
- Record FreeCell Gate 1 evidence; FreeCell implementation remains a later gate.

## 1.2.3 — 2026-10-01 (candidate 2; Beast verification pending)

- Recognise an evidenced tableau transfer with automatic white-face replacement
  in the same column, retaining both source identity and independent destination
  evidence without lowering the existing material-change thresholds.
- Return a changed initial preview for review with zero guest input; preserve
  exact fresh approval before gameplay or no-HALO Solver recovery.
- Default Klondike Actions per Multi-Step to 0, continuous until STOP or a guarded
  stop. Add an independently editable Solve animation interval.
- Detect one-board Klondike game completion from two fresh positive observations;
  require intact full-gold/zero-black progress or completed-game artwork.
- Enable the evidenced score-skip, Level Up OK, New Game, Draw 1 Play and fresh
  Solver sequence only for continuous runs, one guarded control input per stage.
- Preserve shared TriPeaks/Pyramid completion policies and input coordinates.
- Correct compiler/Clippy diagnostics without new warning suppression or a
  formatter, preserving Charlie's source spacing and nightly configuration.
- Supply complete-file installation against the same committed base, accepting
  the exact installed candidate 1 and backing up the actual starting contents.

## 1.2.3 — 2026-09-30 (candidate 1; Beast verification pending)

- Recognise tableau card removal exposing green felt dimmed beneath a dashed
  destination guide, retaining independent source and destination evidence.
- Log dimmed-felt removal counts separately from bright replacement pixels.
- Add original-pixel regression scenes for the repeated three-spades refusal
  and the later empty-column result, with explicit evidence limits.
- Preserve the accepted tall-run geometry, editable timings, continuous mode,
  bounded observations, STOP and one-shot Solve result review.
- Build guarded complete-file installation records for committed v1.2.2 at
  `bcce7515979e2c4520b02f39e9c4d4d1bb019d7f`.

## 1.2.2 — 2026-09-30 (candidate 2; Beast verification pending)

- Correct tall Klondike source bounds where a run overlaps the dimmed toolbar.
  Reject internal card edges while connected side rails continue below them.
- Keep the source click and effect evidence above toolbar controls.
- Add original-pixel regressions for the reconstructed Draw pair, the unchanged
  RIGHT failure scene and the tall highlighted run.
- Log existing pointer/hold/settle intervals and measured input/capture times
  to support diagnosis of intermittent no-op inputs without changing timings.
- Support guarded installation over the exact uncommitted candidate 1 contents,
  with backups and rollback to the actual pre-install state.

## 1.2.2 — 2026-09-30 (candidate 1; Beast verification pending)

- Restore Klondike Multi-Step `0 = continuous`, with bounded recovery per action,
  STOP checks and checked operation counters.
- Set the editable Klondike action-settle default to 750 ms; retain the independent
  1000 ms recapture default and update control wording.
- Correct a demonstrated RIGHT-pile effect-verification blind spot using two
  opposite card-corner comparisons plus independent destination evidence.
- Add source/destination proof diagnostics for unresolved effects.
- Recognise the separately evidenced Solve button; request auto-finish once,
  capture its result and stop for review without claiming completion or restart.
- Preserve Charlie's committed Rust minimum 1.101.0, nightly settings, source
  spacing, existing-game policies and original PNG saving contract.

## 1.2.1 — 2026-09-30 (candidate 1; Beast verification pending)

- Add the Solver-driven Klondike Draw 1 profile, dynamic source-block detection,
  three-position RIGHT fan, stock draw/recycle and SUIT-source priority.
- Add an independent bounded Klondike controller, configurable settle/recapture
  values, finite Multi-Step, and one immediate-capture Solver recovery per
  unresolved context. Unknown results never enter shared completion/restart.
- Preserve the established TriPeaks/Pyramid policies and exact-byte PNG saving.
- Add two blank lines before definitions and statement blocks throughout the
  existing and new Rust source without running a formatter.
- Keep nightly, rustfmt configuration and dependency versions unchanged.

## 1.0.1 — 2026-09-26 (candidate; Beast verification pending)

- Record all 28 Pyramid card envelopes and hit points, plus Left, Move/Recycle and Right, from Issue #1's 1920x1080 screenshots.
- Replace broad Pyramid preview envelopes with labelled target rectangles and hit-point markers (Issue #3). Draw overlays only on the main preview; preserve original saved PNG bytes. Keep Pyramid input disabled pending behaviour and detector validation.
- Name Solver, Undo All and Undo as shared toolbar controls and display them in both mode previews.
- Update architecture, socket-path and capture documentation for Issue #2; remove stale project-stage labels while preserving calibration provenance.
- Synchronise the package version in Cargo.toml and Cargo.lock without changing dependencies.

## 1.0.0 — 2026-09-26

The following behaviour was already present in the 1.0.0 base commit:

- Allow the Level Up screen three seconds to settle after the score panel click. If a fresh frame already shows New Game, require a second capture of that stage before acting on it.
- Capture a fresh QMP PNG when Capture PNG is clicked, preview that image, and save precisely those original bytes when Save PNG or Enter is used. Add field focus and a Cut/Copy/Paste context menu.
- Record the Challenge Complete Continue button geometry for future challenge support, without enabling automated input.
- Remove inherited application and artefact names; default to `qmp-qemu-socket.sock`. `QMP_SOCKET_PATH` supports an existing QEMU socket while the launch configuration is migrated.

- Establish `qmp-qemu-socket` with guarded TriPeaks actions and read-only Pyramid calibration.
- Use QMP to capture the VM display, verify each guarded action, and retain private session logs and original PNG evidence.
- Keep the application version in `Cargo.toml` as the source for the window title and Parameters display.

[Approved DL-first presentation clarification](</home/charlie/Documents/Pyramid/Validation/no-challenge-readiness-20261022/goal-supplement.md>) supersedes the immutable goal’s earlier DR-first display clause; all control and acceptance requirements remain.

[Retained DL-first clarification](</home/charlie/Documents/Pyramid/Validation/no-challenge-readiness-20261022/goal-supplement.md>) preserves DL: card details | DR: card details and honest single/exhaustion feedback.
