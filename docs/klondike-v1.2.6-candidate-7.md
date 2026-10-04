# Klondike v1.2.6, candidate 7

## Base and scope

Exact base: `329d8065f4780d7db1b4ee8a52ed3a53b6252adc`, committed candidate 6.
One fresh main checkout was verified clean before edits; all 139 files in the
supplied candidate 6 source freeze matched by bytes and modes. The refreshed,
separate untracked AGENTS.md was read. No later remote polling was performed.
Cargo package version remains 1.2.6; the application label becomes candidate 7.

This correction addresses one captured complete long source HALO. Charlie
reports several subsequent complete live games without another failure, so
broader gameplay and terminal policies are retained.

## Evidence and cause

K80 is the manual FAIL capture. K81 is its later Undo predecessor. Both preserve
original 1920×1080 PNG bytes. The log 1791078466492 records an acknowledged RIGHT
King click, repeated no-HALO observations, one Solver refresh and a guarded stop.
No long-stack gameplay input was attempted. The manual FAIL capture is later
than the final worker observation; neither PNG is asserted to be an unsaved
historical worker frame.

On K80, the existing detector measures source `(1398,372,132,620)`, ending at
exclusive row 992. The lower edge has 96/96 positive samples, rail traversal has
609 matched rows with a largest gap 1, closed top and bright card paper are
present, row 997 has terminated rails, and gameplay/Solver checks pass. Only
`MAXIMUM_SOURCE_HEIGHT=604` rejects it. The canonical action also uses that cap.

## Change

A shared tableau scan-top constant represents the existing row 332. The maximum
source height derives from existing exclusive bottom 998 minus that top: 666.
The source finder and canonical action use the derived capacity. Every other
outline and input guard stays intact. This removes an empirical stack-size cap
while keeping recognition inside the existing native scan envelope.

K80 now produces one source in column 7, rows 372..992, with the existing canonical
click `(1464,412)` within its upper card. Recognition below the translucent toolbar
does not extend click or legacy diagnostic content bounds into it. This is no
permission to click a dashed destination, clipped outline or unknown scene.

The production controller is unchanged. Step Once sends one logical action;
finite runs consume acknowledged actions; continuous 0 follows fresh valid
recommendations. Card matching remains outside live execution. STOP, VM/tablet
probes, mode/socket invalidation, uncertain input non-replay, editable timing,
Solve and independently recognised one-board terminal stages remain in place.

## Validation and Beast acceptance

The delivery review records actual local check results, tool versions and any
environment limitation; it separates detector/controller replay from live guest
acceptance. No formatter or global toolchain change is part of this candidate.

On the Beast, confirm application label v1.2.6, candidate 7 and rebuilt executable
identity. At the captured long-run state, a read-only Capture Frame should select
column 7 rows 372..992 rather than no highlight. Step Once should click the upper
source card once, settle and display a fresh result. Check a finite run, STOP and
continuous play through Solve and the single-board win/new-game sequence. Keep
first-failure native PNG and complete log if a new refusal occurs. Recheck an
accepted TriPeaks/Pyramid run before committing.

## Code Analysis

Guest image bytes remain untrusted and decoded through the existing bounded
RGBA frame contract. Rust uses checked native frame geometry and existing pixel
access checks; no unsafe code, allocation or pointer arithmetic is introduced.
The derived 666-pixel capacity stays inside existing 332..998 recognition rows.
Complete edges, connected positive rails, card paper and scene evidence still
control input; native click bounds and QMP integer conversion remain unchanged.
Malformed, clipped, unsupported and uncertain scenes retain guarded refusal.

Installer inputs are pinned and bounded, and exact HEAD/content/index guards
protect local edits. Complete files are backed up before apply; rollback refuses
unknown edits or changed HEAD. Installation success does not establish guest
behaviour. Charlie supplies live acceptance and performs any commit/push.
