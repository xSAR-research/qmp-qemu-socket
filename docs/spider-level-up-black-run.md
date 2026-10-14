# Spider Level Up five-black-pixel search

This records the original Spider candidate, confirmed live by Charlie. The
shared three-control implementation and revised bounds are documented in
`bounded-terminal-controls.md`.

Candidate: qmp-qemu-socket 2.2.2-candidate.1 with xsar 0.4.2-candidate.1.
Source bases: qmp-qemu-socket 9b6288388f4c2a3a1571b926fe8db95c26e32fc7 and
xsar 072cbed6cda31c82289522c384974e2448ffd807.

## Agreed behaviour

The caller supplies an already captured RGBA frame and a rectangular search
area. Search starts on the bottom row and moves upwards. Each row is searched
left-to-right. Five consecutive pixels whose RGB channels are exactly zero
produce a match. Return the first pixel of that run in full-frame coordinates;
return (0,0) when there is no match. The returned point is the mouse click spot.
Alpha is ignored. There is no glyph template, near-black tolerance, gradient
comparison, full-region ranking or screenshot acquisition inside the helper.

The xsar helper accepts a validated FrameView, PixelRect and minimum run
length. The rectangle is a logical crop over borrowed storage, with no pixel
buffer allocation or copy. PixelRect uses an inclusive origin and exclusive
right/bottom edges. Existing row padding is respected.

Spider supplies PixelRect::new(930,700,61,151), meaning inclusive X930..=990 and
Y700..=850, and a minimum length of five. Other games retain their existing
methods pending live acceptance on Spider. The app retains native 1920x1080
coordinate validation before using its fixed QMP conversion geometry.

## Worker integration

The existing Congratulations recognition establishes a win. Only then may the
OK search select a Level Up control. The optional New Game alternative is
checked in the same frame. If neither is ready, wait the configured Spider
reobserve delay and capture once more. A second miss stops; the worker publishes
that latest frame as a diagnostic image for egui. If both controls appear ready,
the existing uncertain-state stop remains in force.

The recognised point and terminal stage travel together to the input adapter.
No second search or stale fixed OK coordinate is used for that click. A (0,0)
result is refused at both the coordinator and QMP input boundary. Other restart
controls and gameplay observation limits retain their previous behaviour.

Every OK search emits its inclusive bounds, scan order, required run length,
attempt and result through WorkerEvent::Log. The UI's normal log receiver writes
the same message into the session file and visible log panel. Example:

    Spider Level Up OK search 1/2: X=930..=990, Y=700..=850 inclusive; bottom-to-top rows, left-to-right pixels; 5 consecutive RGB(0,0,0); result=(944, 771).

## Validation

The native level-505 screenshot is stored as
`tests/fixtures/spider-level-up-505.png`. Its five-pixel result is (944,771).
Older native Level Up captures remain covered. Scanner regressions cover exact
RGB, alpha independence, scan order, lengths four/five/longer, stride padding,
ROI boundaries and invalid arguments. Worker regressions cover the measured
click point, retry success, exactly two misses, logs and newest failed preview.

The candidate package's verification report records tests actually executed in
the delivery environment. Run `verify.zsh` on beast for the required nightly
Rust toolchain and native application tests, then prove the sequence in the VM.
Fixture checks do not establish live QMP input or guest animation behaviour.

## Code Analysis

The scanner is read-only and uses safe Rust with checked frame storage, bounds
and coordinate conversion. It allocates no image data. Invalid inputs remain
typed errors rather than no-match results. The (0,0) sentinel cannot distinguish
a genuine origin hit, so callers must exclude the origin; Spider's ROI does.
Any five exact-black pixels in the selected rectangle satisfy the agreed rule.
The caller therefore owns the expected-screen context and crop calibration.
Search failures never cause a sentinel click, and retry captures send no input.
