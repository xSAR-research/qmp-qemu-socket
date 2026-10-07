# Free Cell and Spider v1.4.0, candidate 3

## Result and exact base

This correction starts from Charlie's pushed v1.4.0 candidate 2 at
`aae11dee2792967cb375b368270e46c8a95e8c95`. One fresh main checkout matched
the announced commit, package/lockfile version 1.4.0 and runtime candidate 2,
with a clean initial tree. No later remote poll is used. The runtime label is
candidate 3; the package version stays 1.4.0.

The two supplied failure logs identify candidate 1. Candidate 2 changed Spider
settling and its restart test, leaving Free Cell detection unchanged. The
current pushed Free Cell detector reproduces both failures from the original
PNG pixels, so this version difference does not explain the stalls.

## Evidenced cause and correction

| Evidence | Visible source | Closed top / exclusive cutoff | Old centre paper probe |
| --- | --- | --- | --- |
| FC20 / 083555 | PLAY 6, Q-hearts/J-clubs/10-hearts/9-clubs/8-diamonds | 574 / 947 | 139/289, 48.1% |
| FC21 / 084153 | PLAY 7, single 8-diamonds | 786 / 947 | 139/289, 48.1% |

Both outlines have a closed visible top and continuous opposing exterior rails
reaching the toolbar cutoff. The old clipped-source path rejected both because
the diamond pip at the click point left less than 50% bright paper in its
17-by-17 centre probe. This was single-frame card-presence detection, not a
card-transfer or changed-pixel effect check. Both logs retain a supported
gameplay scene; neither reports a QMP failure.

Use symmetric visible left/right card margins for that paper-presence check.
Both margins contain 289/289 bright pixels in each supplied frame. Keep closed
tops, connected exterior rails, source priority, geometry and the Y947
exclusive cutoff. A clipped source still clicks its visible bottom card at
Y907. No hidden bottom, rank, destination or previous move is inferred.

FC20 and FC21 are byte-identical copies of the full 1920-by-1080 RGBA captures.
The fixture manifest records original names and SHA-256 hashes. Focused tests
cover both original sources, source clicks, excluded toolbar pixels, and the
absence of either card margin. Prior complete and clipped-source tests remain.

## Spider LEVEL UP correction

The newly supplied Spider log identifies v1.4.0 candidate 2. It independently
recognises GAME WIN and clicks to skip score counting at +368.889 seconds.
Twenty delayed post-score observations find neither OK nor New Game ready;
no OK input is attempted. The original SP20 frame shows a standard LEVEL UP
screen with a gold OK button, not a special card-back award.

Its OK caption lies exactly 19 pixels above FC10's original local position.
The original caption window allows only eight pixels of vertical alignment,
so it misses this word. The existing click point (959, 812) remains within the
gold button. Spider now checks the original local OK location or this measured
higher location, reusing the same button coverage and coarse word contrast.
Free Cell keeps its original control region. No level, medal, title, panel
artwork or fireworks matching is introduced; the worker sequence, click point,
one-shot input and timings are unchanged.

SP20 is an unchanged full native PNG. Tests cover this expected OK control,
reject isolated OK as independent win entry, retain original control layouts,
and exercise continuous restart with both OK positions and without LEVEL UP.

## Preserved behaviour and evidence limits

No production controller, QMP, capture, snapshot, logging or other game detector
changes. Free Cell remains fresh source HALO, one source click, editable settle,
fresh capture, with bounded input-free observations and one reserved Solver
refresh when required. No card identity or move-effect verification gates play.
All timings stay unchanged, including Free Cell's 3000 ms game-start wait and
Spider's 1250 ms card/run settle. Optional LEVEL UP and one-board restart stay.

The supplied Spider frame establishes the higher OK position; the candidate
still requires live Beast validation over repeated wins. Rare card-back award
screens remain outside the supported sequence. Supplied attachment byte sizes
differ from earlier saved-PNG sizes in the logs; fixture hashes establish exact
attachment equality only, and the reason for the encoding difference is unknown.

Edition 2024, minimum Rust 1.101.0, floating nightly, Clippy/Rustfmt components,
formatting configuration and existing double blank lines remain. No formatter
or warning suppression is added. The Beast's root AGENTS.md remains untracked
and protected; no refreshed companion was available in this intake.

## Validation and Beast acceptance

The delivery review records executed evidence, source and guarded-installer
checks. Rust/Cargo/rustup are absent in this workspace; Python pixel rehearsals
are not native Rust tests, and no native build, Clippy or live QMP success is
claimed. Run the complete supplied Beast command list, stopping at the first
error. Confirm the installed and launched label is v1.4.0, candidate 3.

Check FC20/FC21 from their native PNGs in the focused Rust tests and exercise
the same clipped run/single-card situations live when available. Verify a
single source click above the toolbar, resumed play and GAME WIN with/without
LEVEL UP. Retain a frame/log for any fresh miss. This delivery stops at Gate 2;
Charlie verifies, commits and pushes.

## Code Analysis

- **Threats:** broad central card artwork previously denied valid sources. Both
  side margins now establish visible card presence, while closed tops and
  connected exterior rails distinguish solid sources from dashed destinations.
  Spider's higher local OK check applies only inside its independently entered
  win sequence. No new action class or recovery input is authorised.
- **Memory safety:** safe existing frame reads and bounded rectangle iteration
  are reused. No unsafe code, dependencies or frame ownership changes.
- **Bounds:** both probes remain inside the measured card face and above Y947.
  Source scan/click limits, canonical geometry and action/observation budgets
  stay unchanged. Toolbar contents cannot supply source authority. The shared
  OK offset helper rejects positions outside the native frame before reads.
- **Failures:** malformed frames, unsupported scenes, missing outlines, absent
  card margins, STOP and uncertain QMP input retain their existing bounded
  stop/recovery policies. Inputs whose delivery is uncertain are not replayed.
  The installer checks exact base, affected contents, modes and index, retains
  backups and refuses rollback over subsequent edits.
