# v1.2.6, candidate 11 — tableau toolbar boundary

Base: `66c779920a1a57d183f07725943e881f689ed3c6`, pushed candidate 10.
Charlie requested that the bottom game toolbar be excluded from tableau HALO
detection. This correction preserves candidate 10's terminal sequence and its
Pyramid, TriPeaks and output-display changes.

## Behaviour

The measured game toolbar starts at guest row 947. Live Klondike tableau source
detection reads only rows above 947. This is the game toolbar boundary, rather
than the Windows taskbar at row 1032. Changing pixels at or below row 947 cannot
change the selected tableau source. Upper stock, RIGHT and SUIT recognition keep
their separate regions; the Solver control and terminal controls keep their own
recognition areas.

A complete source outline above the boundary retains the ordinary detector.
Where a source continues beneath the toolbar, its visible closed gold top,
connected opposing gold rails reaching the boundary, and bright card paper above
the boundary identify the actionable source block. Its reported bottom is the
exclusive visible boundary, 947. The hidden lower edge is not estimated. The
existing click is 40 pixels below the observed top and remains strictly above
the boundary. Dashed destination guides do not acquire source authority.

K95 is the supplied five-card column-six source: 6-hearts, 5-spades, 4-hearts,
3-spades and 2-hearts. Its observed top is 571, its visible bottom is 947, and its
click is (1296, 611). The source continues behind Undo All. Candidate 10's
single-card overlap exception rejected this longer source. Candidate 11 makes
the toolbar pixels irrelevant instead of extending that icon exception.

Klondike still follows a fresh valid Solver recommendation, sends one logical
action, waits the editable settle and captures again. Card-pixel effect proof
does not gate gameplay. STOP, mode/socket invalidation, bounded input-free
re-observation and uncertain-input non-replay remain. Full 1920-by-1080 capture
and original-byte manual PNG saving remain unchanged.

## Evidence and verification

The fixture is an unchanged copy of the supplied native RGBA PNG, 3,434,536 bytes,
SHA-256 `7d5218133a6d5919ea0bd6869d2e28c000d4652bdbf008ce743b485986359853`.
The supplied log records a manually saved PNG of a different byte count; the
fixture is therefore not claimed to be that exact logged result capture. The
log establishes bounded no-HALO recovery and a guarded stop, rather than a QMP
disconnection or card-effect verification failure.

Live regressions cover the native source, every recorded frame with all
below-boundary pixels changed, visible-border refusals and the native Step Once
controller path. Older card-effect diagnostics retain a clearly named test-only
historical classifier so their original full-source measurements remain useful.
That historical classifier is not compiled into the application.

Actual commands, tool versions and results are recorded in the delivery review
and validation archive. No rustfmt or cargo fmt is run. Guest input delivery,
animation timing and complete games remain Beast checks. Charlie's first
successful candidate 10 GAME WIN pass is not a candidate 11 gameplay test.

## Installation

The installer accepts only the exact pinned base or the complete installed
candidate 11. It rebuilds content records against the pushed candidate 10 base;
earlier uncommitted candidate records are not accepted. Preview changes no
repository files. Apply backs up the incoming affected files, verifies the
complete installed candidate and preserves unrelated work and index entries.
Unknown affected edits, staged affected files, new-file collisions and a
different HEAD stop installation. Guarded rollback refuses to overwrite later
local edits. The Beast root AGENTS.md and calibration remain local.

## Code Analysis

- Input authority: only a fresh recognised scene and canonical source permit a
  gameplay action. Toolbar artwork cannot supply tableau evidence. A visible
  clipped source needs a closed top, opposing rails and card paper together.
- Memory safety: safe Rust and existing decoded-frame bounds checks remain.
  Cropping the search region does not crop or duplicate the captured frame.
- Bounds: the tableau boundary is exclusive; the upper source click is checked
  above it. Existing finite capture budgets, action limits and cancellation
  checks remain. Continuous mode remains explicitly stoppable.
- Failures: incomplete visible geometry, unsupported scenes, capture/probe
  errors and uncertain input stop without replay. The latest frame remains
  available. The measured layout assumes the existing 1920-by-1080 guest and
  100% scaling; a changed guest layout needs new evidence.
- Installer: exact HEAD, content, modes and index guards, pinned bounded archives,
  private backups and guarded rollback protect local work. Checksums establish
  agreement with the supplied artifacts, not independent distributor identity.
