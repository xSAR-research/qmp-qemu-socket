# Full Codex-goal acceptance checklist — 2.9.151

Canonical goal: [Codex-Goal.md](Codex-Goal.md). Covers all 13 standing outcomes, retained TracksA/B and151. Earlier Track A/Track B acceptance remains recorded; all rows below are future151 operator checks, not new live passes. See [verification](Verification-2.9.151.md) for host proof and composed-fixture limits.

Start C01–C03, then cover remaining cases on suitable games. Use the guarded `--image` command in [the live brief](Live-Test-2.9.151.md). Record Pass/Fail/Not observed, session/run, capture IDs, export/packet paths and notes in the private results template accompanying the bundle. An unobserved required situation is not a pass.

## C01 — Automatic normal GAME WIN to next Recon

**Do:** Continuous / Shortest route / Repeat normal games. Let two complete three-board games finish without manual START or RECON between them.

**Accept when:** After Play, pending dealing frames cause read-only captures. Two consecutive complete equal normal-one boards precede fresh entry and Recon connections. New Recon confirms all 28 plus24 stock, restores twice and resumes planning.

**Goal mapping:** outcomes 11, 13; tracks A, B3, 151. **Result:** PENDING.

## C02 — Stop during new-game settling

**Do:** After Play, press STOP while the status is waiting for the new board to settle; test a selection/context change in a separate episode.

**Accept when:** No further control click, Recon input, counter acceptance or continuation occurs. Retain current frame, log and failure packet; acquire fresh before another execution.

**Goal mapping:** outcomes 7, 11; tracks A, 151. **Result:** PENDING.

## C03 — Readiness identity and timeout

**Do:** Review the host regressions for all-incomplete6frames, a pending interruption, different full boards, another ordinal, duplicate identities, occupied RIGHT and missing static/legend roles.

**Accept when:** Pending resets pair history. Only equal complete normal-one evidence succeeds. Unsupported or contradictory evidence refuses. No input is replayed and the counter stays at3 on refusal.

**Goal mapping:** outcomes 8, 11; tracks 151. **Result:** PENDING.

## C04 — Normal reward and optional LevelUp

**Do:** Complete a normal series with reward/counting and, when available, LevelUp OK. Also retain an episode with no LevelUp.

**Accept when:** Use the existing receipt-bound reward / optional OK / NewGame / Play sequence. A* never activates guest Solver or HALO. Absent optional OK does not block the next valid stage.

**Goal mapping:** outcomes 11; tracks A. **Result:** PENDING.

## C05 — Reward persists after OK

**Do:** Where encountered, let the reward/counting scene persist after the acknowledged OK.

**Accept when:** Positive supported reward may use its bounded shared skip allowance. Retired OK is not pressed again; unknown scenes cannot authorise clicks.

**Goal mapping:** outcomes 11; tracks A. **Result:** PENDING.

## C06 — Automatic first and second board handoffs

**Do:** Run normal Continuous through board1→2 and2→3; also observe a multi-board challenge.

**Accept when:** Full-ACK expiry permits observation only. A distinct equal fresh pair and correct next counter precede Recon; the same old pair cannot bind. Third-board ACK alone cannot authorise restart.

**Goal mapping:** outcomes 1, 10, 11; tracks B3. **Result:** PENDING.

## C07 — Immutable challenge binding

**Do:** Start Boards, Score and a card-rank challenge. Observe progress on successive boards, including a nonzero entry counter.

**Accept when:** Kind, target and Kings policy remain fixed. Actual numerator can increase at safe boundaries; denominator changes and regression refuse. Unreadable effect evidence gives no next-input permission.

**Goal mapping:** outcomes 10; tracks A. **Result:** PENDING.

## C08 — Challenge target reached mid-route

**Do:** Observe a card-rank/Score target reached before the planned board route ends; include the retained Queens12/12 case when reproducible.

**Accept when:** Positive bound Challenge Complete retires gameplay input immediately and releases operator controls. Record any timeout as a failure; the earlier Queens case is separate and has no new151live pass.

**Goal mapping:** outcomes 10, 11; tracks A. **Result:** PENDING.

## C09 — Kings, Manual and Single board

**Do:** Check authenticated Kings defaults, Manual planning and Single board execution, then explicitly START a new run.

**Accept when:** Kings keeps its authenticated policy. Manual planning does not execute. Single board does not auto-restart or gain normal-series ACK handoff authority.

**Goal mapping:** outcomes 6, 10, 12; tracks A. **Result:** PENDING.

## C10 — Maximum to stock to REDEAL

**Do:** Use a partial Maximum route which drains legal DRAW cycles and reaches newly appearing REDEAL; include occupied RIGHT and both-empty endpoints when encountered.

**Accept when:** Keep exact tableau/piles and verified pre-input48×63 origin. At least512 pixels with inclusive channel delta20 authorise the existing centre click once. Fresh Recon then returns to Shortest.

**Goal mapping:** outcomes 1, 9, 13; tracks B4. **Result:** PENDING.

## C11 — Exhausted stock without appearance

**Do:** Use an exhausted legal partial endpoint where no REDEAL appearance is proved.

**Accept when:** Verify the endpoint, stop safely and release for inspection. No invented REDEAL permission, completion claim, cleanup input or uncertain replay.

**Goal mapping:** outcomes 1, 9; tracks B4. **Result:** PENDING.

## C12 — White, cream, Retro and Western cards

**Do:** Check representative supported Classic/cream/Retro/Western boards and visible DL/DR cards. Review retained football29-face and Western58-face host regressions.

**Accept when:** Every visible identity is independently recognised. Unreadable occupied cards stay Unknown. Supported material below45% in139×187 means Empty; aged material still vetoes unsupported absence.

**Goal mapping:** outcomes 8; tracks B1, B2. **Result:** PENDING.

## C13 — Unsupported or unbound entry

**Do:** Try only ordinary available unsupported/partial scenes; review host damage/missing-counter negatives rather than manufacturing transport faults in the guest.

**Accept when:** No normal three-board count is inferred from absent ordinal artwork. Unsupported identities/scenes or missing binding refuse before acquisition/input.

**Goal mapping:** outcomes 8, 10; tracks A, B2. **Result:** PENDING.

## C14 — Pause each A* phase

**Do:** PAUSE then CONTINUE separately in Recon, active search and gameplay.

**Accept when:** Acknowledgement occurs at a safe boundary; UI/log remain responsive; elapsed suspension is excluded and resume refreshes evidence. No parking between a MATCH pair, input/effect or bounded restoration.

**Goal mapping:** outcomes 7; tracks A. **Result:** PENDING.

## C15 — Explicit Shortest to Maximum

**Do:** During a genuinely active Shortest calculation, use Maximum in Continuous and Manual episodes.

**Accept when:** Reuse the same authenticated deal/export bytes with no new capture. Discard the old result, preserve the operator-switch reason, and use existing auto/manual execution policy.

**Goal mapping:** outcomes 12; tracks A. **Result:** PENDING.

## C16 — Paused switch and stale results

**Do:** Pause active search, request Maximum, then CONTINUE; separately STOP before replacement can finish.

**Accept when:** Pause request persists while the retired search acknowledgement clears and the replacement parks/reacknowledges. STOP wins, stale results do not bind, and cancellation is not exhaustive no-win.

**Goal mapping:** outcomes 7, 12; tracks A. **Result:** PENDING.

## C17 — Defaults and idle RESET

**Do:** Check startup Continuous / Shortest / Challenge, allow Maximum fallback, STOP, then idle RESET.

**Accept when:** RESET restores only those three defaults, forcibly retires plans/planning/challenge updates, retains logs/diagnostics and does not clear STOP or send game input.

**Goal mapping:** outcomes 6, 10; tracks A. **Result:** PENDING.

## C18 — Controls and popup chrome

**Do:** Check840×600 and1120×820, long options, PARAMS/LOG/ANALYSIS and Failure Details; collapse, maximise, restore and close.

**Accept when:** START precedes RECON; LOG is white/cyan, available ANALYSIS white/orange, RESET left of STOP/EXIT. Popup chrome contrasts; in-app maximise/restore preserves geometry; closed LOG retains logging.

**Goal mapping:** outcomes 5; tracks A. **Result:** PENDING.

## C19 — Header, badge and graceful EXIT

**Do:** At both sizes check startup, waiting, Recon, search, gameplay, error and closing.

**Accept when:** Heading/options wrap left; fixed240×80 badge remains right; SOLVING is centred and fully visible. WAITING/ERROR remain solid; active halo alone changes. STOP/EXIT remain visible; EXIT is cooperative.

**Goal mapping:** outcomes 5, 7; tracks A. **Result:** PENDING.

## C20 — Independent live LOG

**Do:** Keep LOG open while Recon/search/gameplay run; copy all text, close/reopen and clear while idle.

**Accept when:** Log output remains live and scroll/copy usable. Closing does not stop recording; clear remains idle gated.

**Goal mapping:** outcomes 4; tracks A. **Result:** PENDING.

## C21 — Frozen current failure details

**Do:** At a genuine failure copy details, STOP and reopen. For Spider Uncertain confirm the source-owned current game/run/settings/error/time.

**Accept when:** Details remain the failure-time record and latest packet paths; a prior Klondike record or later UI edits cannot replace current Spider/source data.

**Goal mapping:** outcomes 3; tracks A, FIX06. **Result:** PENDING.

## C22 — Complete preview and coordinates

**Do:** Resize both supported window sizes and inspect the full guest image, overlay positions and a known one-based row|column.

**Accept when:** Image fits in both dimensions with aspect ratio; overlays and native input mapping remain aligned.

**Goal mapping:** outcomes 2; tracks A. **Result:** PENDING.

## C23 — Direct completed Recon handoff

**Do:** Use manual RECON and automatic START/next-game Recon; compare completion and planning/execution timestamps.

**Accept when:** Mandatory1000 ms UndoAll/OK settle and two restoration captures stay inside Recon. No artificial delay is added after completed Recon.

**Goal mapping:** outcomes 13; tracks A. **Result:** PENDING.

## C24 — STOP, context and uncertain delivery

**Do:** Check STOP/EXIT at ordinary safe checkpoints and review the retained local-socket uncertainty/revocation regressions.

**Accept when:** STOP/context/fresh scene→fresh VM+absolute probe→STOP precede input. ACK proves delivery only. Unknown effect/uncertain transport never replays non-idempotent input; failed routes cannot resume.

**Goal mapping:** outcomes 7, 9, 10, 11; tracks A, B3. **Result:** PENDING.

## C25 — Exhausted LEFT final winning route

**Do:** When available, exercise exact RIGHT plus final cards, including DR5♥+8♠ at2|2 then K♠ at1|1, and a Boards/Perfect overlay endpoint.

**Accept when:** Exact final absence bands, LEFT-X and actual RIGHT identity compose correctly without re-running overlay-obscured ordinary occupancy; count/completion binding remains required.

**Goal mapping:** outcomes 1; tracks B3, FIX05. **Result:** PENDING.

## Host-only negatives

Use existing local-socket regressions for malformed transport, uncertain acknowledgements, stale results, missing controls, duplicates and damaged rasters. Deliberately breaking live guest transport is not required. Safe refusal is expected for unsupported evidence and does not establish route/challenge completion. STOP/context/no-replay and failure evidence remain mandatory.
