# Block 4 — guarded Free Cell route playback (3.1.5)

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
