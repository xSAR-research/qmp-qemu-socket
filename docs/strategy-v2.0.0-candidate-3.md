# Startup layout — v2.0.0, candidate 3

Application base: `9ccf444d9cfebd05ce017456f05e965094bf08a1`.
The fresh committed base is v2.0.0, candidate 2.
The promoted xsar dependency remains unchanged.

Both startup strategy boxes now use the available width with the same frame
padding. The longer route-preparation box establishes the shared height;
the Computer Vision box reserves that same height. Descriptions explicitly
wrap inside the boxes when the window is resized. Labels, selection controls,
startup input gating and all game controllers retain their existing behaviour.

The package supplies five complete affected files. Its guarded installer checks the exact
committed base and affected-file bytes/modes, and requires the new candidate 3
notes to be absent. Unknown edits stop; unrelated files and the
untracked AGENTS.md are preserved. Rollback restores those candidate 2 files
and removes only the candidate 3 notes.

Check the start screen at the normal size and minimum window size. Both
frames should have aligned sides and equal heights, with contained text.
Confirm either strategy still opens its usual workspace. No guest coordinates,
input timing, HALO thresholds or dependency versions are changed.
