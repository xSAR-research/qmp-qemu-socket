# v2.0.0, candidate 5: live capture previews

TriPeaks and Pyramid shared capture paths now publish decoded pixels before analysis. Klondike does the same for normal and diagnostic acquisitions. Delayed HALO observations and post-game transitions use the same display hand-off. The UI consumes the newest pending frame; frames superseded before a UI refresh are deliberately coalesced.

Diagnostic frames carry no predicted action and do not grant input authority. Accepted observations retain their existing prediction publication. No additional QMP capture, PNG encoding or file write is introduced. Publishing clones the decoded frame using the existing representation; this adds transient memory-copy work and is not zero-copy.

Candidate 4 Free Cell shifted OK recognition is retained. The xSAR dependency, timing settings, game rules and input sequences are unchanged.

## Verification

Focused regressions cover intermediate pixels surviving analysis rejection, latest-only delivery while waiting for HALO, preservation of controller prediction without preview authority, and delayed TriPeaks stock observations. Full native validation results are supplied in the package.

## Code Analysis

Threat surfaces: the existing bounded capture/decode path and local UI mailbox remain the boundaries; previews cannot approve guest input.
Memory safety: safe Rust and checked image analysis; no unsafe code added.
Boundary limits: one pending full-resolution frame, replacing older unpublished frames. Cloning adds temporary allocation and copy cost; no unbounded image queue is added.
Failure modes: analysis failure leaves the acquired diagnostic pixels available. Capture/decode failure cannot supply a new frame. Existing context invalidation and STOP handling remain in force. Live visual acceptance requires the Beast.
