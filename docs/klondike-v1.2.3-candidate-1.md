# Klondike v1.2.3, candidate 1

Committed base: v1.2.2 candidate 2 at
`bcce7515979e2c4520b02f39e9c4d4d1bb019d7f`.
Prepared: 30 September 2026, Australia/Brisbane.

## Evidence and diagnosis

The extended `qmp-qemu-socket-1790772674485(1).log` contains two continuous
attempts and one Single Step attempt at the first refusal, then a later second
refusal. All four input operations were acknowledged and all delayed captures
completed. The subsequent source HALOs were detected. No QMP transport error
or failure to scan the higher source explains these stops.

The repeated first move is 3-spades from column 5 to the spade foundation. The
Undo reconstruction supplies an actual before/after board-state pair despite
the later before-state capture timestamps. `Move-before-fail` shows the
highlighted 3-spades in the tableau; `Move-fail` shows it in the foundation and
the new highlighted 3-clubs in column 6. The former source region has become a
dark dashed destination below 4-diamonds.

The existing effect check counted 8434 settled source changes and 13531
destination changes, but no positive bright replacement pixels. The first
result capture briefly counted 8441 source changes before settling to 8434.
The same refusal occurred in Single Step. Additional observations could not
satisfy a colour predicate that excludes this legitimate result.

The second result shows column 6 empty under a dashed guide and King-spades in
column 4 as the next source. Its log reports 8912 changed source pixels, zero
positive replacements and 14299 destination changes. No before frame for that
specific input was supplied. Moving 4-clubs to column 2 is consistent with the
result, but its card identity is not independently established by a pair.

## Narrow source-removal correction

Ordinary exposed green felt and bright card replacement retain their existing
checks. Tableau removal gains a separate measured transition from white card
paper to green felt dimmed beneath a destination guide. The source must still
change materially outside gold and cursor masks, and an independent destination
must also change. The check does not identify a rank or suit.

The measured vacated felt is very dark but remains green-dominant. White card
paper dimmed by the same guide remains neutral grey; opaque black supplies no
green evidence. Accepting arbitrary darkening would allow a changed Solver
recommendation to masquerade as a move, so those cases remain refusals.

The removal count is logged separately from ordinary positive replacement
pixels. This preserves the useful distinction between a missing effect, a
bright replacement, dimmed vacated felt and RIGHT card-corner replacement.
No threshold is lowered for the existing action policies.

The new evidence is local to tableau source effects. Draw, recycle, RIGHT and
SUIT input semantics remain as in the committed base. The existing tall-source
geometry and source/destination toolbar exclusions are retained. Fresh HALOs
alone cannot establish the previous effect, and uncertain gameplay inputs are
never replayed.

## Fixture and test boundaries

K20-K23 retain original RGBA pixels at native coordinates, with original names,
attachment hashes, fixture hashes and retained rectangles in the manifest.
K20/K21 reconstructs the RIGHT 2-spades transfer; K21/K22 reconstructs the
tableau 3-spades transfer. K23 is only a result scene. Any constructed preceding
K23 scene is labelled synthetic and does not claim another recorded pair.

Regression coverage includes the genuine first refusal, unchanged frames,
recommendation-only guide darkening, source-only changes, cursor/gold/toolbar
changes and the separately constructed empty-column transition. Existing
target, effect, STOP, recovery and Solve review cases remain applicable.

Attachment PNG byte lengths differ from the saved lengths in the session log;
provenance records the uploaded bytes without claiming byte identity with the
Beast's saved files. The app's manual PNG capture/save implementation itself
is unchanged.

## Beast verification

Keep the editable 750 ms action-settle and 1000 ms re-observation defaults.
These repeatable failures establish an effect-check blind spot, not a new
timing requirement. Verify the 3-spades foundation transfer in Single Step,
then continuous mode, and a last-card transfer leaving an empty column under
a guide. Check the new removal counts and the independent destination count.
If a refusal returns, retain the first original result PNG and complete log
before Undo or further manual play.

Also verify a short finite run, Multi-Step 0 and STOP. Smoke-test accepted
TriPeaks/Pyramid behaviour and mode/socket invalidation. Solve remains one
authorised click followed by result review; completion and restart are still
unverified. Independent card search and file-free capture remain separate work.

The candidate installer uses the exact committed base and recognised content,
preserves unrelated edits, calibration, untracked AGENTS.md, ignore rules and
the committed workspace file, and backs up the actual pre-install bytes/modes.
The separate candidate review records actual validation results, limitations
and the complete numbered Beast command sequence.
