# Development conventions

## 3.1.20 — host-qualified dark controls and 100 ms playback

Dark demonstration styling and shared pressable buttons now cover application controls and dropdowns. SINGLE STEP and MULTI STEP use 100 ms ABS intervals, with one SINGLE STEP action. Active operations expose Stop beside Pause/Continue and Cancel. Calculation Stop cancels its search. MULTI Stop finishes its current proved step and retains resumable progress. Cancel uses existing immediate cancellation.

All 50 fresh host gates passed, including 1,911 application, 381 library and 80 console tests. Complete suites have zero failed, ignored, measured or filtered tests. Charlie reported successful 3.1.19 guest playback at 100 ms. Fresh Windows acceptance of this styled 3.1.20 build remains pending.

See [the control contract](Application-Style-3.1.20.md) and [verification](Verification-3.1.20.md).


## 3.1.19 — preserve pickup anchor at Free Cell drop targets (host-qualified)

A pickup at the recognised card corner plus 15 px horizontally and 20 px vertically now uses the same offset at an empty CELL, SUIT or PLAY destination. Occupied destinations retain their observed recognised-corner point. The route no longer replaces CELL/SUIT observations with slot centres. This aligns the planned cursor endpoints with the existing source anchor; it does not change calibration, recognition thresholds or physical-effect proof.

The six diagnostic inputs remain source → LEFT DOWN → source minus 45 px vertically → native floor midpoint of the ORIGINAL source and effective destination → exact destination → LEFT UP. Standard SINGLE STEP retains five 500 ms gesture intervals, fast SINGLE STEP five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. Those intentional gesture waits total 2.5 s, 500 ms and 1 s respectively, excluding transport, scheduling, logging and capture costs. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

Calculated A* routes retain one `RecordedMove` card identity per transfer. Rules and independent replay require an exposed source and remove/place exactly one card. The user confirmed individual single-card transfers suffice. No carried-stack feature or stack regression is added; legacy Solver/HALO handling remains separate.

All six fresh application gates passed with 1,908 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence; no fresh standalone sibling gates are claimed. Fresh Windows acceptance remains pending. See [goal](FREE_CELL_goal_20261111J.md), [contract](FreeCell-Drop-Anchor-3.1.19.md) and [verification](Verification-3.1.19.md).


## 3.1.18 — playback timing, capture cadence and graceful stop (host-qualified)

SINGLE STEP keeps five 500 ms gesture intervals. SINGLE STEP fast uses five 100 ms intervals, and MULTI STEP freezes ABS with five 200 ms intervals. The six inputs are source → LEFT DOWN → source-up45 → native floor midpoint of ORIGINAL source/effective destination → exact destination → LEFT UP. Every newly reached recorded AUTO edge retains its 500 ms animation wait.

The first post-transfer wait is min(configured settle, 200 ms). After one eligible endpoint match, the confirmation wait is min(configured settle, 100 ms). Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact endpoint matches are required within six captures. The sixth rejection ends immediately. At the default 750 ms configuration, an all-unmatched run reaches its six effect captures after 200, 950, 1700, 2450, 3200 and 3950 ms of intentional waiting. The prior sequence reached them after 750 through 4500 ms. The total observation window is therefore 550 ms shorter before capture/processing time, and the former final 750 ms rejection wait is removed.

During active MULTI playback its button becomes STOP AFTER STEP. A direct request is bound to the exact plan and run. The current transfer, two-frame effect proof, recorded AUTO follow-ups and animation waits finish before StepVerified retains the proved cursor for resumption. A request before the next transfer, including one received at its PAUSE checkpoint, prevents that transfer. PAUSE stays independent and requires CONTINUE to proceed. Existing immediate STOP remains available for immediate revocation and release-only recovery.

Successful playback dispatch releases completed RECON preview ownership for the same context. Each acquired playback frame already enters the bounded latest-frame slot before semantic interpretation, so the preview can show the newest acquired readiness, effect and failure pixels. Pending RECON and stale-context fences remain required. Frames coalesce in one slot. During a held gesture the preview displays the latest acquisition until the next existing capture, without inserting an in-flight capture or capture-only QMP connection.

All six fresh application gates passed with 1,905 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence. Fresh Windows acceptance remains pending. See [goal](FREE_CELL_goal_20261111I.md), [contract](FreeCell-Responsiveness-3.1.18.md) and [verification](Verification-3.1.18.md).



## 3.1.17 — continuous ABS playback and AUTO animation (host-qualified)

Use source → DOWN → source-up45 → native original-endpoint midpoint → exact destination → UP. SINGLE STEP and MULTI STEP apply five 500 ms gesture intervals, while fast SINGLE STEP uses zero inserted gesture delay. MULTI STEP continues after each physically verified play. Every newly reached recorded AUTO step adds 500 ms for animation across all options. Adopt the additive interval-taking xSAR 0.16.2 path API. All 51 fresh paired host gates passed with 1,896 application, 381 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests. See [contract](FreeCell-Playback-3.1.17.md) and [verification](Verification-3.1.17.md).


## 3.1.16 — source-up and native midpoint ABS trial (host-qualified)

ABS Single Step holds LEFT through source-up45, the native floor midpoint of the original source and effective destination, destination-up45 and exact destination. Both two-second and half-second choices use six timing gaps across seven inputs. CELL/SUIT targets keep their canonical centres. All 51 fresh paired host gates passed with 1,893 application, 366 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests. See [contract](FreeCell-Source-Midpoint-3.1.16.md) and [verification](Verification-3.1.16.md).

## 3.1.15 — quarter-card ABS trial (host-qualified)

ABS Single Step now holds LEFT through destination →45nativepx up →34nativepx left →exactdestination, then releases. Seven input packets have six selected2s or500ms minimum gaps. Upper CELL/SUIT drops use the slot centre to keep the path inside it; PLAY keeps the observed destination. Adopt additive xSAR0.16.0 four-point transport; legacy APIs, normal drag and REL retain their contracts. Recognition, relevant effect proofs, sourceACK/STOP and release-only recovery remain unchanged. All 51 fresh paired host gates passed: 1,891 application, 365 library and 80 console tests, zero failed, ignored, measured or filtered. Fresh Windows acceptance remains separate. See [contract](FreeCell-Quarter-3.1.15.md) and [verification](Verification-3.1.15.md).

## 3.1.14 — relevant printed identities (host-qualified)

Fresh original-52 binding uses the same independent printed-identity reader as RECON. Decorative card rails and full-face edges no longer veto playback. Subsequent preparation and effect observations inspect the source, destination and recorded AUTO-affected piles; unrelated columns are excluded. Compressed columns use observed printed corners rather than a fixed fan pitch or hidden-card count.

All six fresh application gates passed, including all 1,887 tests with zero failed, ignored or filtered. xSAR 0.15.1, console 0.8.5, input gestures/timings and other game policies are unchanged; sibling qualifications are retained historical evidence only. Free Cell uses one-board card/rank-count goals without score targets; independently replayed partial routes remain permitted. Fresh Windows acceptance remains separate. See [contract](FreeCell-Identity-3.1.14.md) and [verification](Verification-3.1.14.md).


## 3.1.13 / xSAR 0.15.1 — destination approach (host-qualified)

Replace midpoint with checked native destination-y-minus-four then exact destination. Keep both selected timing profiles and all five inputs. All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered. Fresh Windows acceptance remains separate. See [contract](FreeCell-Approach-3.1.13.md) and [verification](Verification-3.1.13.md).

## 3.1.12 — midpoint and frozen timing requests

The approved ABS midpoint trial adds two-second and 500-millisecond step buttons. Both queue immutable typed timing and emit exactly five inputs, with source acknowledgement and the four held replies after UP. xSAR 0.15.0 adds the timing enum/method while retaining its diagnostic entry point. All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered. Fresh Windows acceptance remains separate. See [contract](FreeCell-Midpoint-3.1.12.md) and [verification](Verification-3.1.12.md).

## 3.1.11 — one-second five-input ABS observation

Four one-second pauses retain both destination packets and preserve source ACK, fresh routing, STOP, release-only recovery and exact effect proof. Normal drag, REL, recognition, cached retry and AUTO policy are unchanged. All 51 fresh paired host gates passed: 1,882 application, 353 library and 80 console tests, zero failed/ignored/filtered. Fresh Windows acceptance remains separate; see [contract](FreeCell-Pacing-3.1.11.md) and [verification](Verification-3.1.11.md).

## Application 3.1.10 / xSAR 0.14.2 — approved corrections

The current [contract](FreeCell-Corrections-3.1.10.md) replaces the stepped ABS SINGLE STEP experiment with source ACK, LEFT down, ABS destination twice and LEFT up without intentional gesture waits. Keep normal playback and REL pacing unchanged. Immutable solution cache and live permission are separate: ordinary failure enables a new explicit UNDO ALL retry with full fresh original-board binding, never automatic replay. Native 6D prototype and gold 5H right-rail calibration preserve recognition margins and full geometry. [Qualification](Verification-3.1.10.md) passed all 51 fresh paired gates.

## Application 3.1.9 — verified AUTO follow-ups (10 November 2026)

Charlie approved application 3.1.9 with unchanged xSAR 0.14.1. The default transfer policy now accepts twice-confirmed current/recorded-prefix readiness, explicitly drags one next record even AUTO, proves its recorded AUTO closure twice and stops Single Step. UI labels describe application policy rather than Solitaire settings.

Six fresh application gates passed against 1,033 frozen compiler/configuration/test inputs: scoped format, locked all-target check, strict private rustdoc, all-target Clippy, full tests including ignored cases and release. Tests: 1,871 passed, zero failed/ignored/filtered. Ten focused regressions passed. xSAR 347 and console 80 tests are retained with exact source/configuration/toolchain bindings, not rerun. No dependency/public API/transport change; all four indices/history and sibling tracked diffs preserved.

See [verification](Verification-3.1.9.md), [contract](FreeCell-Auto-Followups-3.1.9.md), and `Evidence/FreeCell/Validation/freecell-auto-followups-319-20261110/qualification-summary.json`. The input seal is `42ca3d5829dbd455558cd67ee52e8e57dc2aceb420d8b9a5c935d194a13a2e5a`. The mutable 3.1.9 executable SHA256 is `9e9637e275578e18b4d2de44f8dcf2ded3093939eeaac05eab3b2cd1265a54da`. Independent reviews passed; first focused fixture failures and the closeout-helper correction remain retained. Fresh live acceptance is pending; no application launch, guest operation, installation or promotion was performed.

## 3.1.8 — held-motion and native halo correction (host-qualified)

Explicit queried-pointer stepped ABS and bounded REL probe, positive native card/control geometry, and exact delivery/effect separation. All 51 fresh gates passed (1,862 app / 347 library / 80 console tests); see [contract](QMP-Drag-Diagnostics-3.1.8.md).

## 3.1.7 qualification

Application **3.1.7** with xSAR **0.14.0** is host-qualified for explicit ABS/REL Free Cell A* SINGLE STEP diagnostics. Each request freezes one mode; both use at least two seconds between source positioning, LEFT down, held movement and LEFT up. No standalone source or destination click follows. Full-route playback retains its original gesture.

Exact outgoing JSON appears unescaped on its own line in the existing visible LOG and private session, after a QMP_TX identity header. Separate QMP_WIRE metadata retains microsecond/source clocks, owner, primary/recovery connection, QMP ID and write outcomes. Neither flushing nor acknowledgement proves guest effect.

All **51 fresh required gates passed**, bound to **1,032 compiler inputs** at SHA256 `a28d0d145b00d0807146f28fbfcb85f88dd4230e28d1a8d5366f4a7ba6f8f5c5` and the recorded toolchain/configuration. Full tests: **1,850 application, 345 library and 80 console**, zero failed, ignored or filtered. The matrix includes dependency-free defaults, sixteen isolated features, strict private rustdoc, Clippy and release builds for both owners and the real console consumer.

REL first queries the pointer inventory, positions the source absolutely, selects/verifies a unique relative pointer and rechecks it before pressing. Signed native deltas are approximate relative device units. Original-pointer restoration requires acknowledged UP and continuing authority. The explicitly approved failure exception selects the captured relative pointer before old/fresh release-UP; no DOWN or motion is replayed. STOP/error may leave REL selected. Concurrent routing changes remain outside a pinned-device guarantee.

Qualified mutable Cargo executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, **3.1.7**, 16806144 bytes, mode 0755, SHA256 `3c1ecef2d0eec235d868f4284daa7e33dffa0e855270b63d663474526db11567`. No protected copy, install, launch, guest input or Git promotion. Four histories/index trees/staged changes and independent server work are preserved. No fresh Windows/Solitaire drag acceptance or QEMU root cause is claimed.

## 3.1.6 qualification

Application **3.1.6** with xSAR **0.13.0** is host-qualified for the approved QMP diagnostic. Complete outgoing JSON packets, including `execute`, `arguments` and `id`, are recorded under `QMP_WIRE` in the existing private session log. Capability negotiation and primary/fresh release-only recovery are covered. `detail.json` on each `write-attempt` contains the exact compact packet string; separate flushed/failed records distinguish write outcomes from acknowledgement and guest effect.

A* Free Cell SINGLE STEP uses one-second source/down/held-move/up intervals, then a cancellable three-second wait, one fresh terminal/physical-authority frame and a separately guarded current-pointer LEFT click. Normal route playback retains its existing gesture. STOP after confirmed UP produces no later capture, probe, click or recovery connection. The usual two fresh physical endpoint proofs within six observations remain necessary for cursor advancement.

All **51 required gate outcomes passed**: **47 fresh gates** bind the final **1,031 inputs** at seal SHA256 `34b5cce8ddb9dadc28d1806e322af7146488d2eac418d126a12e11620e81aa65`; four library format/check/private-doc/Clippy gates are retained against identical xSAR source, configuration and toolchain. The intervening source difference was a consumer-private Clippy simplification; `retained-library-gates.json` records the comparison. The matrix covers dependency-free defaults, all sixteen isolated xSAR features, both owning packages and the real console consumer. Full tests passed **1,845 application, 338 library, 80 console**, zero failed, ignored or filtered. All eleven new 3.1.6 tests passed in the full suites.

Application base is `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`; xSAR base is `59c5639eed1b57e5be51b97c3f35065f92c22b42`. Exact dirty-file pins bind qualification. All four Git histories, indices and staged changes, plus independent capture-server edits, are preserved. Console stays at 0.8.5 with approved lock alignment. No installation, application launch, guest input or promotion was performed.

Windows/Solitaire drag acceptance and a QEMU root cause remain unverified. Supplied xHCI/tablet, listener and actual `/run/user/1000/qmp-qemu-socket.sock` configuration show no discrepancy requiring a setup change. Logging remains best effort if the existing session log is disabled, fails or the process terminates abruptly.

## 3.1.5 qualification

Application **3.1.5** and xSAR **0.12.0** are host-qualified. Accepted RECON automatically selects its exact JSON. Opaque challenges use complete-first unlimited planning and an independently replayed exhaustive maximum-foundation fallback. Full clearance, partial exhaustion, game-reported completion, cancellation and failure remain distinct. Positive measured highlighted-card geometry supports fresh same-source binding; ordinary glyph confidence and white-paper policy remain.

All **51 recorded gates passed** against the same **1,029 inputs**, seal SHA256 774f44714cb545ade4b1be8ebf283f3b234b77fdcd92050e00b38d9ca0273602: both owning format/check/private-rustdoc/Clippy/full-test/release suites, dependency-free defaults and all sixteen isolated xSAR features, and console check/private-rustdoc/Clippy/full-tests/release. Full tests: **1,839 application, 333 library and 80 console**, zero failed, ignored or filtered. The final focused family passed 92 cases; library fallback regressions passed ten. See [Verification-3.1.5.md](Verification-3.1.5.md).

Current app base is c541e878440b19e3f7aaa289d7e3c92ed895fe2c; xSAR base is 59c5639eed1b57e5be51b97c3f35065f92c22b42. These bases plus exact dirty-file pins establish qualification; HEAD alone does not. Console remains 0.8.5 with only its approved lock alignment. All four histories/indices/staged states and independent server changes are preserved. Source remains unstaged/uncommitted. No installation, launch, guest input or promotion occurred.

The unlimited graph can be expensive; a maximum is proved only under the approved irreversible-foundation, single-card/Horne rules. Complete routes have no shortest-route claim. Native gold evidence covers the exposed retained 9D; full lower faces and occupied cells are labelled synthetic. Shared terminal appearances and local peers prove passive retirement/transport behaviour, not fresh Free Cell guest acceptance. Positive completion retires input before recording and never advances an unproved cursor, clicks Continue or starts another board.

## 3.1.4 qualification

Application 3.1.4 corrects read-only RECON acceptance: native frame/crop bounds, challenge retention and all 52 fixed rank/suit readings with exact unique-deck validation. Felt, unrelated scene pixels and grey borders no longer veto acquisition. Local contrast permits the readable warm-highlighted 9♦; shape confidence and ambiguity limits remain unchanged. Live playback retains a separate strict recogniser and its existing input-authority policy. xSAR remains 0.11.1 at `59c5639eed1b57e5be51b97c3f35065f92c22b42`.

All six fresh host gates passed against the same 1,024 compiler inputs and stable nightly 1.102.0: owning-file format, locked all-target check, strict private rustdoc, strict all-target Clippy, **1,810 tests with zero failed/ignored/filtered**, and release build. The seven new regressions include the exact retained highlighted board, every irrelevant pixel changed, native ambiguity, isolated paper-policy separation, low contrast, invalid frames and local TCP export with an off-card pop-up. Source seal SHA256 `8b3adc9ce4a242c489ff412fdd77d1395e7b6806b341f06a417e4a01637c4a71`. See [Verification-3.1.4.md](Verification-3.1.4.md).

The app base remains published `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`; qualification binds the changed tree, not HEAD alone. All four Git histories/indices and the independent sibling staged/unstaged changes match the starting receipt. Source remains uncommitted and unstaged. Original incidents and sealed 3.1.3 evidence are preserved. No installation, application launch, guest input or promotion occurred; fresh guest/operator acceptance remains unperformed.

## 3.1.3 qualification

Application 3.1.3 implements all seven correction items: request-bound fresh RECON pixels and original-cause FAILURE DETAILS, variable objective acquisition with honest planning/playback refusal, Free Cell as the A* entry default, aligned wrapping workflow controls, and scoped passive development telemetry. xSAR remains 0.11.1 at `59c5639eed1b57e5be51b97c3f35065f92c22b42`; no sibling source, schema, dependency or transport-policy change was needed.

All six fresh host gates passed on the same 1,022 pinned compiler inputs: owning-file format, locked all-target check, strict private rustdoc, strict all-target Clippy, **1,803 tests with zero failed/ignored/filtered**, and release build. Seal SHA256 `fc3687822599e4f8af2e1c0f5d7c6f82759bfe9c8a920289138c2fde1b90da3a`; toolchain and complete commands are recorded in [Verification-3.1.3.md](Verification-3.1.3.md). First failures and their corrections remain retained.

The application base is published `c541e878440b19e3f7aaa289d7e3c92ed895fe2c`. Baseline qualification covers the pinned changed tree, not HEAD alone. Four histories and indices, and independent server staged/unstaged work, match this correction's starting receipt. Source remains uncommitted; no staging, promotion, installation, launch or guest input was performed. Host/native-retained/synthetic/local-peer qualification is complete; fresh operator/guest acceptance remains unperformed. Unresolved or legacy/unmarked objectives require supported acquisition before CALCULATE/playback; they never silently become a full-board objective.


## 3.1.2 qualification

Free Cell playback now uses one guarded source-to-destination drag per recorded transfer. The source movement is acknowledged before button-down; held movement and release are sent before collecting their replies. Interrupted or uncertain delivery permits release-only recovery and never replays the transfer. Two fresh semantic endpoint observations, rather than delivery acknowledgements, advance the cursor. CALCULATE remains offline; autoplay policy, Single Step scope and permanent revocation rules remain unchanged.

All 38 source commands passed: dependency-free defaults and eleven isolated xSAR features, both owning-package six-gate suites and console check/tests. Full tests: **323 xSAR / 1,789 application / 80 console**, zero failed/ignored/filtered. Fourteen focused application tests include local TCPv3/Unix-QMP peers, immutable failure presentation and actual headless Restore clicks at wide/narrow workspace sizes. Seven aligned commands passed after exactly five approved Cargo version edits; **1,013 of 1,018** compiler inputs stayed identical. xSAR is **0.11.1**, application **3.1.2**, console package unchanged.

Host/retained-frame/local-peer evidence does not establish fresh guest drag effects, autoplay timing or full-route acceptance. Native support remains 1920×1080 CLASSIC BLUE with Solver off and complete supported geometry; toolbar-clipped ten-card columns may refuse. A mismatched or uncertain position stops playback and requires CALCULATE again. No guest actions, installation, executable copy, commit or push were performed for this release.

The frozen source and version-only lineage are recorded in [Verification-3.1.2.md](Verification-3.1.2.md).

## 3.1.1 qualification

Adds explicit Free Cell A* route playback from the retained independently replayed complete route. CALCULATE remains offline. Single Step performs at most one deliberate transfer with its leading/trailing automatic transfers; SOLVE remaining route continues from the verified cursor. Fresh physical-board evidence binds the saved original deal before input, and observed semantic effects advance the cursor independently of QMP acknowledgements.

Keep Solver off and choose the policy matching game autoplay: ON observes automatic transfers without clicking them; OFF executes every recorded transfer. The policy freezes at first playback. PAUSE/CONTINUE retain verified work at safe boundaries. STOP, uncertain delivery, failure, or changed source/context/policy revokes playback and requires CALCULATE again. See the [Block 4 playback contract](free-cell-block-4.md).

All six application source gates passed, including 1,789 tests with zero failures, ignored or filtered tests. The 28 focused tests include synthetic board effects and local TCPv3/Unix-QMP peers. App-only version alignment passed locked check, release build and direct help; 1,015 of 1,017 inputs are unchanged across the two Cargo version edits. xSAR stays at 0.11.0 with its existing qualification retained. Fresh operator-checked guest/UI acceptance remains pending; no guest actions were performed during this implementation.

Native support requires 1920×1080 CLASSIC BLUE, Solver off and complete supported card geometry. Hidden identities are carried only from the authenticated original through verified same-run moves; contradictory visible evidence refuses. Complete-face checks can stop on toolbar-clipped ten-card columns. Actual source/destination click semantics, autoplay timing and complete clean native midgame acceptance still need guest validation. See [verification](Verification-3.1.1.md).


## 3.1.0 qualification

Adds an offline Free Cell complete-route calculation from a strict saved 52-card deal. The dedicated worker freezes the loaded input, searches without configured memory/time/state/expansion limits, records root and interleaved Horne automatic transfers, and independently replays every transfer before publishing a usable complete route. Foundations are irreversible. Guidance orders the search; no shortest or optimal route is promised.

The UI provides a direct Pyramid/Free Cell selector, JSON file selection, CALCULATE, PAUSE/CONTINUE and CANCEL, actual counters and measured paused duration. Request/context/source-generation fencing rejects stale results. Calculation performs no capture, QMP or guest input; later move execution and partial optimisation remain separate work.

Qualification passed 38 source commands: xSAR dependency-free defaults and eleven isolated optional features, both owning six-gate suites, and the console consumer check/tests. Full tests passed: xSAR 319, controller 1,761, console 80, with zero failures or ignored tests. The unchanged library's 30 passing rows are retained from the first source attempt; the app's narrow Clippy correction is followed by fresh six-gate qualification. The original failure and preimages remain preserved.

Approved five-file version alignment to app 3.1.0/xSAR 0.11.0 passed library/app/console locked checks, console tests, both release builds and direct help. All 1,009 compiler inputs are sealed; 1,004 are identical across version alignment. Console package version remains 0.8.5. No new Miri or fresh guest/GUI acceptance is claimed.

See [verification](Verification-3.1.0.md) and [Block 2 contract](free-cell-block-2.md).

## 3.0.2 qualification

All six source gates passed, including 1,758 tests with no failures, ignored or filtered tests; six focused regressions passed. Approved version-only alignment passed locked check, release and bounded help. Source/aligned sets contain 996 inputs, with 994 unchanged after the two Cargo version edits. No fresh guest execution is claimed.

Run from this repository root: `cargo run --release`. No secondary executable copy was created. See [verification](Verification-3.0.2.md) and [telemetry contract](klondike-failure-telemetry.md).

## 3.0.1 diagnostic build

3.0.1 adds exact Klondike failure telemetry. It saves the first classified scene rejection and final stopping frame already acquired by the worker, with all fifteen probe counts, same-response TCP identities and attempted/acknowledged input. Scene decisions, thresholds, input, STOP and observation budgets are unchanged. No extra capture is taken.

Run from the repository base with `cargo run --release`. [Verification](Verification-3.0.1.md) · [Telemetry and reproduction](klondike-failure-telemetry.md).

## 3.0.0 qualified host build

3.0.0 adds independent read-only Free Cell RECON/export. One explicit request acquires one fresh native TCPv3 frame, requires eight PLAY columns [7,7,7,7,6,6,6,6], four positively empty CELLs/four SUITs and 52 unique recognised rank-and-suit identities, then publishes bounded JSON without overwrite. Unsupported, incomplete or ambiguous evidence refuses export. Free Cell A* planning is deferred to 3.1.0.

Retained source qualification: xSAR 20 gates/311 full tests; controller six gates/1,748 full tests. Approved version alignment passed locked checks for xSAR, controller and console, the console tests, controller release and bounded help. Only five authorised manifests/locks changed after qualification: 987 sealed inputs, 982 unchanged. No secondary executable, guest/GUI action, installation or Git publication occurred.

Native acceptance remains pending: FC01/FC13 are evaluation originals, not blind threshold hold-outs; native PLAY-bottom Two, authentic dealing/unsupported-scale/artwork negatives and fresh operator-checked guest RECON/export remain unverified. Foundation Twos establish typography only. Source/socket tests do not establish live acceptance or completion of all Block 1 acceptance rows. See [RECON contract](free-cell-block-1-recon.md) and [verification](Verification-3.0.0.md).

## 2.9.183 — Cooperative STOP and fresh preview

The accepted STOP correction is qualified as **2.9.183**. A causally recorded operator cancellation shows Waiting and admits one token/context-correlated raw preview after matching run ownership retires. Genuine failures, stale authority, shutdown and uncertain delivery retain Error. STOP stays latched; failed or stopped plans cannot resume. Existing failure history is retained.

Run from `/home/charlie/repo/RUST/qmp-qemu-socket`:

```sh
cargo run --release
```

The normal Cargo target is used; no secondary executable copy was created. All six source gates passed, including **1,736 tests** with zero failed, ignored or filtered; 11 focused STOP regressions passed. After Charlie's explicit milestone/version approval, aligned locked check, release and direct `--help` passed. Fresh guest STOP/preview acceptance remains pending. See [verification](Verification-2.9.183.md) and [game operations](game-operations.md).

## Qualified 2.9.182 — images enabled by default

GAME WIN image recording is now enabled by default. Run `cargo run --release` from the repository root; use `cargo run --release -- --no-image` to opt out. The former `--image` flag is rejected. Startup retains the existing fixed image-directory reset in both modes; help and invalid arguments return before that reset.

All six source gates passed, including 1725 tests with zero failures, ignored, measured or filtered tests. The approved version-only alignment then passed locked check, release and protected help. Of 961 sealed inputs, 959 remained identical after alignment; the 56 xSAR inputs are unchanged. [Verification](Verification-2.9.182.md); [image policy](game-win-images.md).

## Qualified 2.9.181 — reusable game operations

Added a private reusable game-operation layer for fresh native capture, guarded mouse/key input, pure card and option recognition, and bounded semantic Undo protocols. Pyramid Recon now uses its shared Undo All protocol with the existing capture, wait, confirmation, STOP and publication sequence preserved.

All six application gates passed, including 1725 tests with zero failed, ignored, measured or filtered, followed by the approved 2.9.181 locked check, release and protected help. The 56 unchanged xSAR inputs retain cached qualification. Fresh live guest acceptance remains pending. [Verification](Verification-2.9.181.md); [operation contracts](game-operations.md).

## Qualified 2.9.180 — foundation verification and INITIAL readiness

Verified the existing four-SUIT foundation source handling in Free Cell and Klondike. Pyramid INITIAL source checking now waits for the bundled native stock/Draw scene anchor within its existing six-check budget. All static scene, legend, layout, modal, header and original-source checks must pass on the supported frame before input; POST remains strict.

All six application gates passed, including 1713 tests with zero failed, ignored, measured or filtered, followed by aligned locked check, release and protected help. Five new readiness regressions and eight existing foundation checks passed. The 56 unchanged xSAR inputs retain cached sixteen-gate qualification. Fresh live guest acceptance remains pending. [Verification](Verification-2.9.180.md).

## Qualified 2.9.179 — foundation source HALOs

Free Cell now checks all four foundation stacks for a complete closed solid source HALO after its existing CELL and lowest-PLAY priorities. The measured native slots use checked one-click mapping. Dashed destinations and incomplete sources remain refused. Klondike already scanned all four foundations; its production detector and controller remain unchanged, with stronger coordinate/invalid-column regression coverage.

All six application gates passed, including 1708 tests with zero failed, ignored, measured or filtered, followed by aligned locked check, release and protected help. The 56 unchanged xSAR inputs retain cached sixteen-gate qualification. Foundation source tests translate retained native source pixels; no new Free Cell foundation-source screenshot or fresh guest acceptance is claimed. [Verification](Verification-2.9.179.md). The editable [gameplay function catalogue](gameplay-function-work-tasks.md) is a planning deliverable; its proposed composition module has not been implemented.

## Qualified 2.9.178 — additive Coral Diamonds recognition

One bounded Underwater Diamonds suit supplement is decoded from the byte-original retained Five of Diamonds at 6|5. All thirteen original rank corners and every previous suit variant remain unchanged. The supplement must already match the complete original family as Five and Diamonds before only its suit glyph is appended. Quality, gap, colour, geometry and ambiguity rules remain unchanged.

All six controller gates passed, including 1703 tests with zero failed, ignored, measured or filtered, plus aligned locked check/release/protected help. The 56 xSAR inputs and package 0.9.1 are unchanged; its sixteen-gate qualification remains cached. Saved-frame recognition is offline evidence; fresh guest acceptance of this revision remains pending. [Verification](Verification-2.9.178.md).

## Qualified 2.9.176 — Coral entry and RESET preview

Coral rank recognition now uses an independently retained black Four corner within the unchanged 13-rank calibration. Boards headers match one coherent existing background appearance. RESET shows WAITING and requests one token-correlated read-only preview while retaining STOP, inventory, logs and historical diagnostics.

All six controller gates passed, including 1697 tests with zero failed, ignored or filtered, plus aligned locked check/release/protected help. The 56 xSAR inputs and package0.9.1 are unchanged; prior sixteen-gate library qualification remains cached. Fresh guest acceptance is pending. [Verification](Verification-2.9.176.md).

## Current 2.9.174 delivery

2.9.174 corrects native Boards challenge-header recognition using the union of
the five existing validated Boards literal ink supports, restricted to independently
proven complete word spans. All reference geometries and lengths are checked before
indexing. RGB tolerance 9, blue floor 6, word distance, letter cores, banner anchors,
boundaries and counter parsing remain unchanged. No new reference bank or dilation
is introduced. Score/Card recognition and input authority are unchanged.

The six controller gates passed, including 1687 tests with zero failures,
ignored or filtered tests. Qualification sealed 942 inputs; after qualification only
the application's manifest/lock version pair changed from 2.9.173 to 2.9.174,
leaving 940 inputs identical. Aligned locked check, release and protected help passed.
The 56 unchanged xSAR inputs retain cached sixteen-gate qualification (303 full
tests, 14 default tests and two Miri tests); xSAR remains modified/unreleased 0.9.1.
User profiles and CPU/hardening configuration are preserved. Evidence: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-challenge-174-proposal-9yca399e`.

[Verification](../Evidence/release-verification/Verification-2.9.174.md).

## Retained 2.9.173 delivery

2.9.173 places exact INITIAL source identity readiness inside the existing
six-check computed-route precondition budget. The immediately retained boundary
frame is checked first. A supported identity nonmatch remains pending; subsequent
checks use fresh captures and the existing re-observation delay (50 ms for ordinary
actions; final-action pacing retains its existing owner). Both expected MATCH
sources must match in the SAME frame. At most six checks are made before refusal.
A ready boundary adds no capture. Input remains withheld until a source proof and
its exact replayed after-state bind the prepared once-only action.

Structural frame/reader errors, unsupported source scenes and STOP still refuse
input. The two successive fresh POST source-disappearance checks remain unchanged;
a different card or undetermined rank/suit can qualify only for the already-issued
action. No input replay, RGB coverage veto or pixel-quietness test is introduced.
The next action independently identifies its own expected sources. Completion,
RECON inventory/restoration and REDEAL keep their separate semantic authority.

Failure packets and the window now carry an optional typed current-stage snapshot,
independent of the retained frame phase. It records INITIAL checks and fresh
captures, expected/observed sources, current action attempts/acknowledgements,
completed actions and separate earlier observations. Current POST confirmations
are distinct from the previous action's retained confirmations. Matching
persistence-only updates preserve the displayed stage. Unsupported fresh frames
are counted as received; a failed transport attempt is not called a retained image.
RECON reports its available acquisition/restoration counters; owners without a
detailed snapshot explicitly report that it is unavailable.

Dependencies, user Cargo profiles, CPU/hardening configuration and modified,
unreleased xSAR 0.9.1 remain unchanged. No guest operation, new live capture,
installation, staging, commit or push was performed. Fresh Coral and standard
live acceptance remains pending.

All six controller gates pass, including 1683 include-ignored tests with zero failures/ignored/filtered (149.37 seconds). Aligned locked check, native release and protected --help pass. Library qualification is PASSED (Cached): 56 unchanged inputs, 303 full tests, 14 default tests and two focused Miri checks.

[Verification](../Evidence/release-verification/Verification-2.9.173.md).

## Retained 2.9.172 delivery

2.9.172 verifies the expected original card at its exact source location before
sending the logical action once. Two successive fresh supported observations must
then stop identifying that same source card. A different recognised card or an
undetermined rank/suit (`None`) qualifies; structural reader/layout errors do not.
An original card reappearing or an unsupported observation resets the pair.

RGB coverage, replacement/ancestor identity matching and pixel quietness no longer
veto MOVE/MATCH/KING action acceptance. The next action independently identifies
its own expected source on the retained frame before input. The existing capture
budgets, probes, STOP and no-replay rules remain. RECON uses two supported source
identity checks within its existing observation loop, while ordered identities,
unique inventory, exhaustion, Undo All, confirmation and restored-board publication
remain separate requirements. Final-clear, terminal and next-board claims retain
their semantic checks on the same captured frames; source disappearance alone is
not reported as an observed board clear.

REDEAL retains its separate appearance policy. PARAMS labels its RGB controls for
that scope. Glyph recognition, package dependencies, user Cargo profiles, host CPU
and hardening configuration are preserved. xSAR remains the unchanged modified,
unreleased 0.9.1 path dependency. No guest operation, new capture, installation,
staging, commit or push was performed. Fresh guest acceptance remains pending.

All six controller gates pass, including 1678 include-ignored tests with zero failures/ignored/filtered (154.10 seconds). Aligned locked check, native release and protected --help pass. Library qualification is PASSED (Cached): 56 unchanged inputs, 303 full tests, 14 default tests and two focused Miri checks.

[Verification](../Evidence/release-verification/Verification-2.9.172.md).

## 2.9.171 qualified tolerance correction

2.9.171 changes the default Pyramid RGB tolerance from 13 to 5 intensity
levels (5/255, about 1.96 percent). A pixel counts only when all three
same-coordinate absolute RGB differences are strictly greater than 5; alpha
is ignored. A difference of 5 is excluded and 6 qualifies. Joint coverage stays
strictly greater than 5.0 percent in the card-relative (3,4), 66-by-89 effect
quarter: 293 of 5,874 eligible pixels refuse and 294 pass. Existing guided cursor
exclusions retain their adjusted eligible denominator.

Live PARAMS still publishes a complete pair frozen per action or paired-D batch;
edits affect the next boundary. Glyph recognition and independent quiet settling
remain 31 by 49. The quiet byte tolerance 12, per-channel 1-percent/mean-2 limits,
two-capture requirement, exact identities, stock order, inventory, restoration,
terminal, STOP and no-replay checks are unchanged. No input, capture or timing
policy changed. User optimisation profiles and CPU/hardening flags are preserved.

The reported 170 Coral 4-spades/4-diamonds refusal was 289/5,874 (4.92 percent)
at tolerance 13. A later different-theme diamond report was 140/5,874 (2.38 percent),
also at 13. Immediate pre-batch pixels are missing for both reports; their actual
tolerance-5 counts and complete Recon/restoration outcomes cannot be reconstructed.
The earlier independent Coral settling stop is not established resolved.

All seven retained picture-card controls pass the tightened default, while their
exact historical counts remain tested explicitly at 13/5. These are aligned
artwork comparisons, not natural temporal transitions. Suitable retained evidence
for four other Coral picture-card suit pairs is still unavailable. Fresh complete
standard/Coral guest Recon, restoration and broader theme acceptance remain pending.

All six controller gates pass, including **1,675 include-ignored tests**, zero failed, ignored or filtered; test time 149.57 seconds. Aligned locked check, release and protected --help pass. xSAR remains the unchanged modified/unreleased 0.9.1 path dependency; its sixteen-gate receipt (303 full tests, 14 default tests, feature checks and two focused Miri checks) is cached over 56 unchanged library inputs.

[Verification](../Evidence/release-verification/Verification-2.9.171.md).

## 2.9.170 qualified quarter comparison

2.9.170 enlarges Pyramid card-effect comparison to the top-left quarter of
the common card interior: card-relative (3, 4), 66 by 89 pixels. All three
same-coordinate absolute RGB differences must still be strictly greater than
13; alpha is ignored, and joint coverage must remain strictly greater than
5.0 percent. With all 5,874 pixels eligible, 293 changes refuse and 294 pass.
The glyph reader and independent after-action quiet-settling sample stay
31 by 49. Guided pile comparisons keep their existing cursor-exclusion union,
so their eligible denominator can be smaller. REDEAL keeps its separate region.

Live PARAMS still publishes a complete validated pair, frozen for the current
action or paired-D batch; edits affect the next boundary. Expected identities,
stock order, inventory, restoration, terminal checks, STOP and no-replay rules
remain independent. This change does not replace the separate quiet-settling
policy or establish that the previously reported settling stop is resolved.

Seven exposed same-theme picture-card comparisons pass the quarter policy.
The two retained Coral controls are J-clubs/J-spades 1,025/5,874 (17.45 percent)
and K-hearts/K-diamonds 564/5,874 (9.60 percent). The other four Coral picture
suit-pairs lack suitable exposed retained evidence. These are aligned artwork
controls, not natural before/after transitions. The exact failed Coral 7-clubs
to 7-spades transition lacks its immediate pre-D frame, so its quarter coverage
cannot be reconstructed. Fresh complete standard/Coral live Recon, restoration
and broader theme acceptance remain pending.

All six controller gates pass, including 1,674 tests in the required
include-ignored scope. The user optimisation profiles and CPU configuration
are preserved. xSAR stays unchanged at the existing modified, unreleased 0.9.1
path dependency; its sixteen-gate receipt is cached, not a new library run.

## 2.9.169 qualified coverage default

2.9.169 changes the starting Pyramid joint-change coverage from strictly more
than 10.0 percent to strictly more than **5.0 percent**. The RGB tolerance stays
13: a pixel counts only when all three absolute R, G and B differences at the
same coordinate are strictly greater than 13; alpha is ignored. On the existing
31 by 49 corner, 75 pixels refuse and 76 of 1,519 pixels pass. Live PARAMS still
publishes one complete pair, frozen for the current action or paired-D batch;
an in-flight edit takes effect at the next established boundary.

The reported Recon counts of 102 (6.71 percent, standard-theme 9D to 5D as
clarified by Charlie) and 131 (8.62 percent, Coral) clear this pixel gate under
the new default. Synthetic regressions check both counts against 5 and 10
percent. Immediate runtime pre-batch pixel pairs were not retained, so these
tests do not reproduce those guest transitions or prove complete Recon.
Expected identities, stock order, full inventory, restoration, quiet settling,
STOP/context revocation and no-replay rules remain in force. Fresh live
standard/Coral Recon acceptance remains pending.

All six controller gates pass, including 1,670 tests with the required
include-ignored scope, locked checks, strict private rustdoc, Clippy, scoped
formatting and release. xSAR is unchanged at the modified, unreleased 0.9.1
path dependency; its 16-gate receipt (303 full tests, 14 default tests, feature
checks and two focused Miri checks) is PASSED (Cached). User Cargo profiles
and CPU configuration are preserved. No install or guest launch was performed.

## 2.9.168 qualified joint RGB occurrence

2.9.168 is host qualified with the modified xSAR 0.9.1 path dependency.
All 1,669 controller tests and 303 full xSAR tests pass. Controller format,
locked check, strict private rustdoc, Clippy and release pass. xSAR's default
14 tests, seven individual feature checks, all-feature checks, strict docs,
Clippy, release and two new focused Miri checks pass. The qualified xSAR working tree retains 0.9.1; its added comparator remains an unreleased path-dependency change.

Computed tableau removals keep the occupied pre-action corner, recognised-card
veto, two fresh quiet after-observations and original-slot history. Computed
stock-only KING/MATCH, MOVE/RECYCLE and stock continuation also require RGB
occurrence on the pile corners whose expected identities change. Full scene,
remaining-card and exact pile identities remain mandatory. Ordinary position
and frame-guard checks precede removal-history commitment; the later bound
challenge-counter check still precedes progression and new source authority.

Native Recon keeps one original stock baseline across paired D and stale
observations, with no intermediate capture. Recognised stock advancement must
also satisfy the joint RGB rule. UndoAll/Confirm and pause revoke these pixel
baselines; their specific dialog/restored-arrangement proofs remain authoritative.
Guided/Solver pile comparisons preserve the inset and union of cursor exclusions,
the unique highlighted partner rules and semantic terminal exceptions. REDEAL
appearance uses the shared predicate, while one acknowledged click still hands
off to fresh Recon. Challenge completion, terminal/final-apex and restoration
proofs remain explicit semantic exceptions. Colour change alone grants no input.

[Receipt](../Evidence/release-verification/Verification-2.9.168.md).

## Retained earlier development

## Current paired slice: 2.9.167 / xSAR 0.9.1

2.9.167 and xSAR 0.9.1 are host qualified for route-owned temporal card-removal
verification and the reusable safe packed-byte comparator. All 1,660 controller
tests and 299 xSAR tests pass, including the required ignored-test scope. xSAR's
dependency-free default, seven individual feature checks, strict private
rustdoc, Clippy and two new focused Miri checks also pass.

The computed route uses the actual occupied rank/suit corner and two settled
after-observations. Prior removed slots and final clear use the same committed
history rather than exposed-background special cases. Static readers retain
their existing rules. Fresh Coral, Xbox and Wild West temporal acceptance
remains pending.

Only the four approved package manifest/lock files changed after qualification;
901 other sealed inputs remained identical. The path dependency remains local,
not a Git pin. Both aligned locked checks and release builds pass. The help
branch exits before image cleanup or GUI startup. [Verification](../Evidence/release-verification/Verification-2.9.167.md)
records the preserved packaging assertion and authoritative Cargo artifact IDs.

## Retained development records

## Current schema-2 slice: v2.2.0

Charlie approved schema-2 integration after successful live acquisition and
restoration on 11 October 2026. Local application HEAD is
`57215a90fd2836cb13a0d1b14400c5d5d417e9f9`; Charlie promoted the complete xSAR
implementation at `ab226a65d3bbc2b4914a5cecfb2d3247364c7536`. Preserve all existing
uncommitted reconnaissance work. Package versions are application 2.2.0 and
xSAR 0.4.0, with manifest and root lock entries updated together and no unrelated
dependency changes. Capture, recognition and all controller policies are unchanged.

xSAR owns the const rank/value mapping and strict schema-1/2 reader. Writers
emit schema 2 with exactly `card`, `value` and `suit` for every card. Numeric
values never replace suit identity. The root document schema is independent of
the native calibration profile's unchanged schema 1/profile ID v4.

The manifest and lockfile select xSAR 0.4.0 at that exact Git revision. Normal
validation uses `--locked` without an external config or local source patch;
`--offline` additionally requires the committed source and dependencies to be
cached. Earlier paired validation used
`/tmp/pyramid-schema2.4FXYLJL6/local-xsar.toml`; retain that historical evidence
outside promotion files. The application is promoted after Git-backed validation
and consumer acceptance.

The accepted schema-1 export remains unchanged. Explicit conversion reads it
through xSAR, writes to a distinct no-overwrite destination and preserves its
original producer/source/restoration metadata. That conversion does not execute
a new scan or grant input authority. Console intake and search remain separate.
See [the v2.2.0 delivery record](../History/documentation-index.md#item-fbda618d2c15b509b436).

## Historical reconnaissance slice: v2.1.0

Charlie authorised the complete Pyramid reconnaissance-to-JSON implementation
on 11 October 2026. The promoted application base is
`ba1a82beb3ea39242d91ead39ebdbd1198847f84` (2.0.11). Paired development consumes
the reviewed, uncommitted xSAR 0.3.0 model/JSON foundation and publisher from
`/home/charlie/repo/RUST/xsar`; its promoted HEAD remains
`987856a0b5cb95aec2788551e050e18f7469b1ef`. Console intake and search are deferred.

The external validation config is
`/tmp/pyramid-recon-build.5qifGKcG/local-xsar.toml`, supplied to Cargo with
`--config` alongside `--locked`. It contains a source patch for the existing
xSAR Git URL and is intentionally outside the checkout. Its local source and
the corresponding paired lock state are not independently promotable. Promote
xSAR first, repin the consumer to that exact commit, regenerate only the
affected dependency source and rerun locked validation before app promotion.

See [reconnaissance and restoration](pyramid-reconnaissance.md) and
[native profile validation](pyramid-recon-profile.md). No build result is proof
of a real guest scan or restoration; those require Charlie's Gate 3 acceptance.

## Historical two-frame comparison slice: v2.0.11

The accepted application base is `e59e06c49f7112b2dac1a7634999ae6f6fcd7344`,
originally using xSAR `1a719b359a51d1e1e3113224193a779c76de82f8`. Gate 2 adds
shared RGB measurement in xSAR 0.2.1 and replaces only the two application
counting loops. Keep game thresholds, cursor geometry and input/effect policy
local. Pyramid's three optional mask entries contain two actual rectangles:
Move uses D and contributes no cursor mask. Do not add one during extraction.

Charlie promoted xSAR 0.2.1 at `987856a0b5cb95aec2788551e050e18f7469b1ef`.
The application manifest and lockfile now select that exact Git revision with
defaults off and the existing qmp/png/image_matching features. No host-only path
or source override is needed for normal builds. Record Cargo's selected source
and use locked checks after narrowly resolving a dependency-source change.
The earlier sibling-path and isolated paired builds remain historical evidence.
For future paired development, use an isolated temporary override and keep it
outside promotion files. Promote the crate first, update the consumer's exact
Git pin without unrelated upgrades, revalidate and obtain consumer acceptance.
Registry publication is separate; the commit subject does not set Cargo's version.

As authorised on 10 October 2026, rustfmt and cargo fmt, including `--check`,
are permitted. Preserve existing toolchain/formatter configuration and keep
formatting within the approved change. Preserve the established two-blank-line
layout. The formatter prohibitions in the dated records below are superseded.
Use explicit installed nightly selection for xSAR when the host default is
below its unchanged MSRV. Required implementation checks remain locked check,
tests, strict private rustdoc, Clippy and release build, with feature coverage
for the crate and explicit include-ignored host evidence for the application.

See [v2.0.11 implementation and evidence](../History/documentation-index.md#item-acb084196d46f392b9aa).

## Historical v2.0.0 candidate 2 conventions

The exact application base is `78df8f1fc994442a06aa9eaa411414462e169727`.
The xsar Git dependency is pinned to Charlie's promoted commit
`1a719b359a51d1e1e3113224193a779c76de82f8`; Cargo.lock records the same source.
Keep the root companion AGENTS.md untracked. Do not run rustfmt/cargo fmt.
Keep two blank lines before item documentation and statement blocks.

Use ordinary Cargo check, test, doc, Clippy and release-build commands with
`--locked`. The application no longer requires the paired candidate wrapper
or a local xsar checkout. No machine-specific path or Cargo patch is committed.
Crates.io publication remains a separate decision.

Preparation must not call a HALO controller or accept ExecuteSteps. Tests
cover raw capture, zero guest input, unsupported combinations and late
socket/game/strategy/generation rejection. Shared transport tests move into
the crate, alongside checked frame/ROI and optional-feature coverage. Existing
five-game evidence remains application regression coverage.

Preserve the accepted earlier modes' fresh-evidence cycles and policies.
Pyramid's highlighted MOVE/Recycle uses D; TriPeaks retains delayed input-free
no-HALO recovery. The following gameplay notes remain current.

Spider's twenty native originals cover ten PLAY columns, stock dealing,
automatic run packing, heavy compaction and the toolbar boundary. A single
fresh source authorises one card/run click or D. No card/effect proof is added.
Candidate 4 parks the pointer at (20, 500), where the full supplied 33-by-46
cursor fits on measured clear felt outside scene/source probes, only after a
source click's release is acknowledged. A shared checked QmpClient helper
sends one absolute movement
command with two axis events, no button/key input and no added delay. The
existing editable card/run settle and fresh capture follow. Auxiliary movement
is counted separately; the source still consumes one logical action slot.
DRAW does not park. Initial, read-only and manual PNG captures remain input-free.
The independent 2000 ms DRAW wait reflects Charlie's 1–2 second observation;
the card/run default is now 1250 ms following Charlie's Beast tuning. Other
settings remain editable. Spider owns its source geometry and
ordered one-board policy; score, optional OK and New Game use the accepted
local control checks, with a separate Spider Play location 100 px lower.

The continuous restart regression uses a distinctive configured start delay
and checks ordered waits, with and without LEVEL UP and with Solver inactive
or already active. Score-skip and the default game-start wait are both 3000 ms;
counting equal durations conflated the two purposes and produced a false failure
in candidate 1. Candidate 2 changes the test, not the terminal state machine.
Candidate 3 adds native SP20, whose OK caption sits 19 pixels higher than FC10.
Spider accepts the original or this measured local button position after a
confirmed win. Free Cell keeps its original location; no title, level or medal
matching is introduced. Cover both OK positions in continuous restart tests,
retaining the optional direct-New-Game branch and uncertain-input non-replay.
Active Spider Solver is disabled: no refresh click is permitted on a no-HALO
board.
Use bounded input-free recaptures and independent completion checks instead.
Inactive Solver retains one editable start wait, fresh capture and one
activation per unresolved context. Default card/DRAW/observation/start waits
remain 1250/2000/1000/3000 ms; the reported runtime used a configured 2000 ms
observation interval, which does not change the default.

Free Cell's approved runtime slice uses original FC01–FC21 native frames and
its own source geometry and one-board terminal controls. It follows fresh
CELL/PLAY source HALOs without card or effect-pixel matching. Missing HALOs
receive input-free re-observation for automatic SUIT transfers before one
Solver refresh on a supported board. Inactive Solver receives the independent
3000 ms game-start wait and fresh capture before activation. There is no SUIT
source scan, Draw, Recycle or Solve operation. FC14/FC16/FC17 support clipped
PLAY sources using visible closed tops, rails and paper above the toolbar;
no hidden bottom edge or card effect is inferred. FC18/FC19 also cover complete
closed bottoms ending at Y947; the toolbar remains excluded.
FC20/FC21 add a clipped run and single card whose central eight-of-diamonds pip
leaves only 139/289 bright pixels in the old centre probe. Check paper in both
visible card margins instead, preserving the outline conditions and click at
Y907. These are same-frame card-presence checks; they neither compare ranks
nor establish a previous action's effect. Regressions erase each margin and
alter excluded toolbar pixels independently.
The guarded installer pins this exact base, preserves unrelated work and the
Beast's untracked root AGENTS.md, and verifies complete affected contents.

## Naming and documentation

- Use `snake_case` for modules, functions, methods, fields and local bindings.
- Use `UpperCamelCase` for types, traits and enum variants.
- Use `SCREAMING_SNAKE_CASE` for constants and statics.
- Prefer module-qualified names where two modules expose the same concept,
  such as `pyramid::TABLEAU_CARD_COUNT`, rather than a renamed import.
- Numeric `as` casts are conversions, not name aliases. Review their bounds
  and meaning; do not replace them as a cosmetic naming change.
- Place meaningful `///` documentation on functions, methods, constants,
  statics and type definitions. Explain the contract and non-obvious parameters,
  result, failure conditions or authority granted. Document important fields
  and enum variants. Place `//!` documentation at module boundaries.
- Ordinary local bindings and implementation explanations use `//`; attaching
  item documentation to a local `let` does not provide a function contract.
- Keep test helpers and regression cases documented too. A test-only helper
  belongs under `#[cfg(test)]`; do not silence dead-code warnings globally.

These conventions follow the [Rust API naming guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
and [Rust Reference documentation-comment rules](https://doc.rust-lang.org/reference/comments.html).
The crate enables missing-documentation warnings, including Clippy's private
item check. These checks complement review of what each comment actually says.

## Responsibilities and behaviour preservation

`game.rs` defines typed profiles/actions, while each mode owns its detection
and effect policy. The worker owns the QMP connection and cancellation state.
Its `post_game` submodule shares the existing terminal-screen sequence; its
`pyramid_execution` submodule contains Pyramid-specific result observation.
Its `klondike_execution` submodule owns finite or continuous Klondike runs with
bounded per-action recovery and independently verified single-board completion.
`klondike_terminal.rs` owns the evidenced terminal controls. The separate Solve
button is clicked once before bounded completion observations; only continuous
runs may restart through fresh recognised stages. The shared terminal controller
retains the existing TriPeaks/Pyramid policy and coordinates.
`freecell.rs` owns native layout, coarse scene/Solver state and canonical
CELL/PLAY source actions. `freecell_terminal.rs` owns independent score/New Game
win entry and local expected-control readiness. `worker/freecell_execution.rs`
owns finite/continuous execution, automatic-transfer observations and ordered
one-shot terminal progression. Only continuous runs restart; the fresh board
must be recognised before activating Solver and resuming from a new frame.
`spider.rs` owns the stock/source classifier and ten-column dynamic geometry.
`spider_terminal.rs` owns typed expected stages and the measured Spider Play
location, reusing local caption/control checks through a bounded offset helper.
`worker/spider_execution.rs` owns one-action execution, independent DRAW delay,
post-click pointer parking, bounded input-free no-HALO observation and
optional-LEVEL-UP restart. Only continuous 0 restarts; a fresh dealt board
precedes inactive-Solver activation. `qmp.rs` owns the checked movement-only
helper; it does not own Spider's park policy or change capture behaviour.
The UI delegates persistent log I/O to `session_log.rs`.

When adding a mode, supply its profile/detection policy and preserve the shared
input and capture contracts. Do not scatter new mode checks through QMP or
session logging. A new mode's special rules should remain visible at the game
policy boundary.

The unused provisional `card_reader.rs`, inactive card-rank/history types and
their reader-only thresholds were removed. The provisional reader never
returned a recognised visible rank: it deliberately reported incomplete
calibration. The old implementation is recoverable from Git commit
`dccb7f9ae7ff11acae615f029f6e183b15484a63` if useful during future self-solving
work. Active card geometry and its validation coverage remain.

Preserve capture dimensions, calibrated probes, click coordinates, timing
defaults, QMP command policy and manual PNG byte identity during a structural
refactor. Keep fresh-HALO continuation distinct from proven action effect and
from board/game completion. A progress-bar reading alone is not a win.

## Source spacing

Charlie's 30 September 2026 instruction requires two blank lines before function,
struct, enum, trait, type, constant, static, impl, inline-module and macro
definitions, above attached documentation and attributes. Statement blocks
(for, while, loop, if, match and bare scope statements) receive the same spacing
before the enclosing statement. Documentation remains attached to its item.

Apply this convention to existing source as well as new code. Do not run
`rustfmt` or `cargo fmt` for this candidate. Preserve `rust-toolchain.toml` and
`rustfmt.toml`; changing those files or the global toolchain is not part of the
spacing change. A whitespace-only pass must preserve parsed Rust tokens and
comments and introduce no syntax errors.

## Local verification

Run from the repository root with its selected nightly. Record `rustc --version`
and `cargo --version`. Each check is independent; stop and retain diagnostics on
the first error.

```text
cargo check --locked
cargo test --locked
cargo doc --locked --no-deps --document-private-items
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
```

No formatter check is included, by Charlie's explicit instruction. Build/test
results from another host do not prove Beast installation or live guest input.
The candidate delivery notes record the checks actually executed.

On the Beast, confirm that the installed package, launched executable and startup
log identify **v1.4.0, candidate 4**. Select Free Cell and verify read-only Capture
Frame, original-byte Capture PNG/Save PNG, initial Solver activation, CELL/PLAY
single clicks and automatic SUIT-transfer recapture. Adjust the independent
750/1000 ms settings, separate 3000 ms game-start wait, 20-observation allowance
and zero action budget; they must
not alter other modes' settings. Verify GAME WIN → optional OK → New Game →
Play → fresh board → Solver → fresh source, both with and without a level change.
After score counting, one fresh frame checks both permitted controls; only one
ready local control authorises input. Unready controls receive bounded recaptures. Switch to each existing
mode and verify its accepted action/recovery behaviour, STOP and socket/mode
invalidation. Preserve first-failure frames and complete session logs.

Select Spider and verify source click → acknowledged release → movement-only
park → configured settle → fresh capture. Check one source slot and separate
auxiliary counts of one command/two absolute events, with no park after DRAW.
Verify active no-HALO frames use bounded input-free recaptures and completion
checks with zero Solver refresh clicks; inactive Solver gets one start wait,
fresh capture and one activation. Check STOP and uncertain park delivery do
not repeat the source click. Initial Capture Frame and manual PNG operations
must send zero input. Retain candidate 3 Free Cell margin and higher-OK checks.

Use compact tests for actual invariants: native source/scene discrimination,
frame-layout rejection, finite budgets, automatic-transfer recapture, one-shot
GAME WIN controls, STOP, uncertain-input non-replay and independent bounded
timings. Test counts are measured results, not coverage quotas. Historical
Klondike card-effect diagnostics remain explicitly test-only and must not become
Free Cell runtime requirements. The delivery review records actual tool versions,
commands and failures; no old validation run establishes this candidate's results.
The supplied-frame cursor simulation is supporting analysis, not a recovered
runtime PNG or native Rust execution. Rust/Cargo are absent here; native build,
tests, Clippy and live QMP remain pending Beast verification. See
[candidate 4 evidence and limits](../History/documentation-index.md#item-27084918f0c4779a5077).


## Historical candidate verification notes

The following notes retain their original candidate scope and measurements.
Earlier card-effect witnesses, exact preview matching and icon exceptions do not
override the current Solver-led Klondike policy. The candidate-10 terminal
button policy and candidate-11 tableau boundary are retained by this release.

For the inherited tall-run behaviour, verify the complete source bounds and
its single click above the toolbar. The reconstructed Draw pair is not a
successful RIGHT transfer pair. Replay success supports the existing input
contract; an intermittent RIGHT no-op still needs live diagnosis. Logged pointer,
hold, action-settle and measured I/O times are separate quantities.

For v1.2.3, replay the three-spades foundation transfer and a last-card removal
leaving a dashed guide over an empty column. The source-removal check must
recognise dimmed green felt without accepting unchanged cards darkened by a
new recommendation. Verify Single Step, continuous mode and STOP, preserving
the first failure frame and full log. Card and recapture timings remain editable.

Candidate 2 also covers a same-column replacement face after automatic reveal,
the zero-input changed-preview endpoint, the default continuous limit and the
five supplied terminal scenes. Verify the new separate Solve delay, finite-run
stop before terminal input, and continuous score → Level Up → New Game → Play →
Solver progression. Unknown or unchanged stages must stop after bounded read-only
observations without another control click. All five input controls remain live
Beast checks; fixture recognition is not a runtime input test.

Remove compiler/Clippy diagnostics by preserving behaviour in the affected code.
Use compile-time assertions for fixed configuration invariants, named evidence
records for grouped recovery guards and boxed failure payloads for large error
variants. Do not introduce warning suppression or run a formatter. The candidate
review records actual compiler versions and warning counts.

Candidate 3 adds the measured 3-hearts recipient transition under a persistent
foundation guide and extends upper-card replacement evidence to SUIT returns,
including stable-paper changes in both printed-detail directions.
The generic material thresholds remain. Test the real RIGHT pair and adverse
guide/paper/source/destination/mask cases. SUIT return tests are explicitly
synthetic. Its named screenshot arrived after candidate 3 validation, but shows
the source recommendation only; an actual before/result pair remains a Beast
verification requirement. See `klondike-v1.2.3-candidate-3.md`.

FreeCell is evidence intake only in `freecell-gate-1.md`. Its implementation follows
Klondike acceptance/promotion and a new authorised cycle; do not introduce dormant
mode code or claim Klondike calibration establishes FreeCell input behaviour.

Candidate 4 separates source-supported fresh-HALO continuation from strict effect
verification. Test finite budgets and reported counts, repeated-source replacement,
unchanged/cursor-only refusals, independent Solve handoff, and prohibition after
uncertain input or Solver refresh. The real black-pip RIGHT pair changes only 251
recipient pixels; it continues without being called verified. Extend the tall-run
case to the measured closing edge partly covered by Undo All. Require the remaining
edge, paired rails, closed top and white face; test gaps outside the exact overlap,
clipped borders and toolbar input/effect bounds. Preserve K19 recognition.
See `klondike-v1.2.3-candidate-4.md` for provenance and live acceptance limits.

Candidate 5 covers a bottom single-card transfer whose source stays mostly white
as the remaining stack spreads. A normal lower corner crosses the toolbar, so the
ordinary corner function keeps its refusal. A separate pair of full visible
patches must retain both paper/ink directions, positive paper and material source.
This new source proof supports fresh-HALO continuation only; a complete effect
remains unverified. Test restoration of either patch, source-only replacement
without complete verification, receiver-only changes, one-way shading/whitening, masks, insufficient material,
run geometry and malformed frames. No toolbar pixels may supply effect proof.
The controller, action budgets, timing defaults and completion policy are unchanged.
See `klondike-v1.2.3-candidate-5.md` for native measurements and Beast checks.

Version 1.2.4 candidate 1 adds K41–K43. Verify the 604-pixel source, the
warmer Solve border and input-free completion across animation into the settled
Congratulations screen. The longer Solve wait uses the existing 20-round
post-game bound and current editable re-observation timing; this is not a fixed
20-second timeout. Card recovery and terminal-control advancement retain their
existing bounds. All new source keeps the requested spacing without a formatter.

Version 1.2.4 candidate 2 retains the same exact Git base and adds K44–K49.
Test the displaced native source, persistent/transient unsupported results, the
source-supported contracted-card continuation and both early/late Level Up
recognition. The provided earlier rollback screenshot is not the original
planning frame for the last bottom-card action; that regression combines native
result pixels, runtime counts and an explicitly synthetic predecessor.
Do not claim K48 establishes the missing live Solve failure: its settled artwork
already passes. Inspect new per-observation Solve diagnostics in the next run.
No input may be replayed after uncertain delivery or unsupported-scene recovery.

Version 1.2.5 candidate 1 adds K50–K52 and corrects continuous initial Solver
recovery. Preserve the finite-run review endpoint. Test stable Solve lettering
with a removed/changed outer ring, unknown scenes and intact stock refusals,
coherent Level Up label alignment, and source-supported bottom-card continuation.
Retain unchanged/cursor/guide/paper/mask refusals and strict-effect distinctions.
The unrecorded frames after the last Level Up OK click remain unverified.

Version 1.2.6 candidate 1 adds native K53 and fixes the Home body sample covered
by the last OK pointer. Preserve all six required body probes and independent
modal/label/completed-board guards. A pending Solve face grants observation
authority only; it never grants input at the lower 95% interior threshold.
Test full availability after bounded captures, disappearance, persistence,
initial changed-preview review, finite budgets, priority, STOP and input errors.
Early Solve pixels were not saved; controlled test derivatives are not original
worker captures. Verify the complete new-game/deal/Solver sequence on the Beast.


Candidate 2 rebuilds the installer records for f9167a0 after both candidate-1
preview/apply refused a changed HEAD. Preserve Charlie's README purpose edits.
Require `--verify-installed` before every Cargo or launch block; download hashes
do not prove installation. Its read-only check requires complete candidate bytes
and modes, the exact base HEAD and an unchanged affected index, and refuses the
complete old base as well as unknown/mixed content. It does not create evidence
directories or change repository files.

The latest hearts stop occurred on RIGHT 5-hearts to SUIT, exposing 8-hearts;
6-hearts is the next tableau recommendation. Logged source change is only 98
pixels with 7 reverse corner print pixels. The Undo reconstruction contains a
separate gold-outlined 5-hearts over the table, so it is not an accepted fresh
Waste planning frame. No source threshold is changed from this evidence. Obtain
a settled original pre-click PNG and one-action result/log before authorising a
new source-effect policy. See `klondike-v1.2.6-candidate-2.md`.

## v1.2.6 candidate 3 verification

Packaging revision r2 uses exact base 7d869c22a6ca98179cd3efc8f83552d2edd1ada7.
Candidate 2 is now the committed base; no predecessor-package authority is needed.
Manifest schema 4 retains whole-state checks and rollback to that base.
Installed verification requires candidate 3 before
Cargo or launch. Native K54-K64 regressions distinguish outline detection from
card-relative source continuation; complete effect stays unverified for that
continuation route. Keep the full unfiltered checks and Beast end-cycle test.

The ordinary replacement regression independently binds the five-pixel nominal
face offset with unchanged artwork. Test missing/duplicate seams, paper rails,
felt gutters, pure translated print and independent recipient requirements.

Test K59/K60 queen transfer and retained-card guide darkening, with cursor/gold,
source-only and recipient-only refusals. Keep the older dimmed-felt predicate and
strict complete effect verdict unchanged.

Test K61/K62 Draw-to-long-source classification, the previous 60 scene outcomes,
exact 187/186 felt acceptance, independent gutters and invalid outlines. K63/K64
require the disjoint positive-felt union: neither shade component alone reaches
512. Retained-card neutral guide darkening, material/cursor/gold and opposed-half
negatives must still refuse continuation. No stack input was sent in K61/K62.


## v1.2.6 candidate 4 verification

K65/K66 are the later Undo predecessor and stopped result for the SUIT 2 return.
Complete paired source corners and a uniquely located new recipient face support
continuation only. New receiving print must match old source artwork and differ
from the prior recipient; the geometry is selected before the print comparison.
Keep old source material thresholds and the 96-by-96 cursor exclusion intact.
Run native, adverse and production-controller regressions, including source-only,
recipient-only, shading, translated print and ambiguous receiving seams. Require
complete source verification before each Beast build or launch. Current check
results, actual compiler versions and remaining limits are recorded in the
candidate delivery review. Do not run a formatter or suppress warnings.

K67 supplies a separate Pro Level Up panel signature, retaining the completed
foundation/empty-tableau context and the existing OK coordinate/hold/settle. The
full native restart integration uses real completion and terminal classifiers;
no mocked stage approval hides a detector failure. Require only one click per
recognised terminal stage and retain read-only bounds for unknown transitions.


K68/K69 cover the new column-7/column-3 source borders at row 997. Preserve
closed edges, terminated rails, strong rail bracketing around the measured
column-7 shading band, paper above the toolbar and the existing height limit.
K69 is a detection-only sparse fixture, not material-effect evidence. The
original Strange-fail frame already selects its column-5 source; a matching log
and immediately preceding frame are required to diagnose the earlier action.


## Candidate 5 continuation regressions

K70-K73 are complete original-byte manual uploads. Test newly received SUIT ink
and the independently exposed source header together, then remove each separately.
Test neutral paper whitening, retained old source print, masks, ambiguous or
missing geometry and source rows at the toolbar boundary. Native controller
checks distinguish continuation from verified effect/completion, preserve finite
budgets and continuous STOP, and stop before reading evidence after uncertain
input. K70/K71 does not reproduce the logged automatic highlight or effect counts;
K72/K73 reproduces the unavailable source proof in native image analysis.


## Candidate 6 detection and restart regressions

K74/K75 show the same unmoved two-spades source in two HALO intensities. K75
must reproduce the full outline through the measured Hint shadow using only
positive gold samples; remove each rail bracket, edge sample and card/scene
guard separately. The phase pair must not establish a card effect. A repeated fresh target may
authorise another logical action under the current policy; uncertain input may
never be replayed. K76's displaced copy is detection evidence only; no action
semantics are assumed for it.

K77 shows an initial deal after Play with Solver inactive. Check the entire
terminal sequence through one Solver click and fresh observations, and stop
before further input on STOP or uncertain acknowledgement.
Remove card perimeter, stock/waste/foundation and inactive-banner evidence to
prove a generic no-HALO scene cannot authorise SolverReady. The earlier logged source comparison with 454 aligned material and 21 cleared
pixels remains a historical diagnostic; it no longer gates live Klondike input.


## Candidate 6 policy update from Charlie

The current Klondike controller follows fresh valid HALOs after acknowledged
input and settle. Remove old worker assertions that demand card-effect proof;
retain those analyser fixtures as test-only diagnostics. New worker tests must
cover the native K78/K79 SUIT-return then RIGHT recommendation, repeated fresh
HALOs, advisory-preview changes, finite action budgets, bounded no-HALO/Solver
recovery, unknown scenes, STOP and uncertain-input non-replay. Report acknowledged
Solver actions separately from independently verified completion. TriPeaks and
Pyramid retain their accepted shared policies.

## Candidate 10 validation boundary

Validate Pyramid's canonical Move key operation and rejection of forged card/pile
keys, delayed TriPeaks stock-HALO recovery with zero Solver inputs, one-refresh
exhaustion, known-win local terminal readiness on K91-K93 and Undo All source
occlusion on K94. Decorative changes must not affect already-confirmed terminal
progression. A generic gold button must not establish game completion.

Do not run rustfmt or cargo fmt. Preserve the floating nightly toolchain and
record actual compiler/Cargo/Clippy versions. Run check, full tests, rustdoc,
warning-denying Clippy and release build. Retain failed commands and distinguish
sandbox QMP Unix-listener limitations from tests passed. Beast verification is
required for input timing and complete games; synthetic/controller tests cannot
prove guest delivery or animation timing.

Candidate 10's installer pins exact cafdd609f5afcac9738ea7d11bc43b231cb73dfc plus
complete candidate 9 predecessor records. Refuse mixed states, staged affected
files and unknown edits. Back up actual incoming files and restore that incoming
state on rollback. Downloads and validation evidence use /home/charlie/tmp;
root AGENTS.md and calibration remain local and untouched.

## Candidate 11 validation boundary

Validate live tableau detection with the native K95 five-card source and all
recorded frames after changing every pixel at or below row 947. Below-boundary
mutations must leave live source selection unchanged. Remove visible top, rail
and paper evidence separately to retain source/destination discrimination.
The native Step Once regression must send one upper source click, settle and
capture freshly; it must not invoke card-pixel proof or extra Solver input.

Keep the retired card-effect measurements in explicitly named historical
test-only helpers. Do not redirect live fixture or worker regressions through
those helpers. Record focused checks, the unfiltered full suite, check, rustdoc,
warning-denying Clippy and release build using actual private nightly versions.
Retain environmental test failures as failures and do not hide them with filters
or warning suppression. No formatter is run.

The candidate 11 installer pins the pushed candidate 10 commit
66c779920a1a57d183f07725943e881f689ed3c6. Its rebuilt base and complete-candidate
records replace the prior predecessor allowance. Exercise preview, apply,
verification, idempotence and rollback, including affected edits/index conflicts,
new-file collisions, wrong HEAD and later local edits. Preserve unrelated staged
calibration and untracked root AGENTS.md. The delivered review distinguishes
these checks from Charlie's guest gameplay verification.
