# Block 4 — guarded Free Cell route playback (3.1.5)

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


## Application 3.1.13 amendment — destination approach (host-qualified)

Both ABS SINGLE STEP buttons use the point four native pixels above the verified destination then the exact destination, with their existing four 2 s or 500 ms waits. Source binding, parent/effect proof, STOP, AUTO policy and cached manual UNDO ALL retry remain. All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered. Fresh Windows acceptance remains separate; see [contract](FreeCell-Approach-3.1.13.md) and [verification](Verification-3.1.13.md).

## Current amendment — 3.1.12

ABS SINGLE STEP now selects the native midpoint then the exact destination, with four two-second waits. The additional ABS-only 0.5-second button freezes the shorter timing per request. Normal remaining-route drag, REL, recorded AUTO proof, fresh physical binding and cached manual UNDO ALL retry retain their contracts. All 51 fresh paired host gates passed: 1,886 application, 354 library and 80 console tests, zero failed/ignored/filtered. Fresh Windows acceptance remains separate; see [contract](FreeCell-Midpoint-3.1.12.md) and [verification](Verification-3.1.12.md).

## Current amendment — 3.1.11

ABS SINGLE STEP retains the five 3.1.10 inputs with four one-second pauses. No extra click is added. Additional protocol probes, normal remaining-route drag, REL, cached manual UNDO ALL recovery and recorded AUTO follow-up proof retain their contracts. This supersedes the earlier no-delay diagnostic timing only. All 51 fresh paired host gates passed: 1,882 application, 353 library and 80 console tests, zero failed/ignored/filtered. See [contract](FreeCell-Pacing-3.1.11.md) and [verification](Verification-3.1.11.md); operator acceptance remains separate.

## Current amendment — application 3.1.10

The [3.1.10 correction contract](FreeCell-Corrections-3.1.10.md) governs current cached recovery and SINGLE STEP ABS input. After ordinary failure, keep the original verified route available for a separately requested manual UNDO ALL restart, with cursor zero and fresh all-52-card physical binding before input. Source/context/STOP invalidation still clears it. ABS repeats the identical destination without intentional gesture waits; normal drag and REL remain unchanged. HALOs mark challenge goals only and never select route input. Historical planning text below is retained; current qualification is recorded in [Verification-3.1.10.md](Verification-3.1.10.md).

## Current 3.1.9 amendment

The approved [AUTO follow-up contract](FreeCell-Auto-Followups-3.1.9.md) governs current Single Step: one explicit transfer, even AUTO, followed by two fresh exact combined proofs of its recorded AUTO follow-ups, then stop. Readiness may accept the unchanged board or an observed recorded AUTO prefix. The UI policy does not change Solitaire settings. Missing effects stop without replay or assumed progress. xSAR remains 0.14.1.

## Retained 3.1.5 design and delivery notes

CALCULATE accepts freshly exported `objective=full-board` and `objective=unresolved` acquisition markers. Legacy or malformed provenance requires re-acquisition. The strict shared JSON schema is unchanged. Acquired challenge text remains opaque: calculation seeks a complete board first, and only true exhaustive search failure releases an independently replayed maximum-foundation partial route. Cancellation, resource failures and invalid replay release no route. See [the current goal](FREE_CELL_goal_20261109C.md) and [verification](Verification-3.1.5.md).

An accepted successful RECON automatically selects its exact published JSON for the next CALCULATE and retires the previous source/report/playback. Stale, duplicate and failed completions cannot replace the selection. Read-only acquisition never grants input authority.

Playback freshly binds all 52 physical identities and proves the supported scene, empty slots and card geometry. Ordinary cards retain neutral borders and white-paper glyph checks. On a fresh nonblank objective, a complete measured gold outline permits contrast-based identity reads for that outlined card; partial highlights, ambiguity and obstructed geometry still refuse. Retained native and labelled synthetic evidence are distinguished in verification.

Ordinary movement still requires two fresh exact physical endpoint observations within six observations. A replay-verified partial endpoint retires its route without a full-board claim. After a fresh same-source 52-card bind, a positive common Challenge Complete title and scaffold retire gameplay immediately, including when observed before the next input. This passive terminal event does not advance the card cursor or claim all 52 foundations. No Continue, New Game, redeal or next-board input follows.

### Step 4.1: Coordinate Mapping for FreeCell Zones
Extend `src/execution.rs` to translate `Zone` variants into screen click/drag coordinates via `src/geometry.rs`:
    * `Zone::Cascade(col)` $\to$ Current bottom-most card coordinate of column `col` (tracked dynamically as column lengths change during playback).
    * `Zone::FreeCell(slot)` $\to$ Center coordinate of FreeCell `0..=3`.
    * `Zone::Foundation(suit)` $\to$ Center coordinate of the target Suit Foundation pile `0..=3`.
    
### Step 4.2: Handling `is_auto` Moves vs. Microsoft In-Game Autoplay
In *Microsoft Solitaire & Casual Games*, the client UI automatically flies Horne-safe cards to the Foundations after a deliberate move completes.
    * **Configurable Execution Policy (`FreeCellExecutionOptions`):**
    * `skip_horne_auto_moves: true` (Default when Microsoft UI Autoplay is active): The executor performs only deliberate moves (`!mv.is_auto`) and inserts an animation settle delay (`auto_settle_delay_ms`) whenever one or more `mv.is_auto` moves immediately follow a deliberate move, updating internal column heights to stay synchronized.
    * `skip_horne_auto_moves: false`: Explicitly clicks/drags every `is_auto` move if in-game autoplay is disabled or if a card requires an explicit click.
