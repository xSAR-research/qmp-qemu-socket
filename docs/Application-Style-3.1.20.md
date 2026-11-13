# Free Cell controls and application styling — 3.1.20

The application adopts the local egui demonstration's dark palette and pressable buttons. The reference source and its digest are retained in this evidence capsule. The reference application is read-only.

The shared palette uses background #1E2124, panels #282C30, text #E6E8EA and ordinary accent #3DAEE9. Buttons use proportional 15-point text, 18×8-point padding, a six-point corner radius and a one-point accent border. Hover tint animates over 120 ms, pointer-down darkens the fill and keyboard focus has a two-point outline. Native egui interaction retains disabled, keyboard, focus and accessibility behaviour. Existing warning/action accents remain available. Dropdowns use the same dark theme and a 220-point preferred width, bounded by their available row space.

SINGLE STEP and MULTI STEP use the named 100 ms ABS gesture interval. The separate fast action and selectable timing state are removed. MULTI freezes ABS at dispatch. The existing explicit REL single-step movement diagnostic retains its own two-second transport pacing and probe completion semantics.

The six ABS events remain source positioning, LEFT down, source-minus-45-pixel vertical lift, native floor midpoint of original source and destination, exact destination and LEFT up. Five 100 ms waits contribute 500 ms of intentional gesture waiting. Source positioning is acknowledged before down, fresh routing is checked, and held replies are collected after up. Destination coordinates retain the authenticated 15×20 pickup inset. This request changes the ABS action interval. Recorded AUTO animation waits remain 500 ms per newly reached edge.

During calculation the action becomes Stop, beside Pause/Continue and Cancel. Stop and Cancel use the existing cooperative calculation cancellation path. During MULTI, Stop requests the existing exact-plan/run stop-after-step boundary. Cancel uses immediate existing playback STOP. During SINGLE, Stop uses immediate playback STOP. Pause remains an independent control. Pyramid planning/session actions also expose their active Stop with Pause/Continue and Cancel in the action row through the existing worker controls.

Fresh original 52-card binding, relevant physical identity proofs, VM/pointer probes, source/context/run/STOP fences and two fresh exact effect confirmations within six observations remain required. Acknowledgement proves transport delivery. Uncertain gestures are never replayed. Post-down recovery is release-only. Graceful MULTI Stop preserves only the proved cursor after its current effect and recorded AUTO waits. The existing post-transfer 200 ms and eligible-confirmation 100 ms caps, configured unmatched retry gaps, acquired-frame preview coalescing and failure evidence remain in force.

xSAR 0.16.3 is a requested metadata-only patch release. Its Rust sources, public APIs, feature defaults, dependencies, edition and MSRV are preserved. The application uses its existing interval-taking path API. The console remains 0.8.5 with only its xSAR locked resolution aligned. Qualification includes default and all 16 isolated library features, strict private rustdoc, Clippy and real application/console consumers.

All 50 fresh host gates passed, including 1,911 application, 381 library and 80 console tests. Complete suites have zero failed, ignored, measured or filtered tests. Charlie reported successful 3.1.19 guest playback at 100 ms. Fresh Windows acceptance of this styled 3.1.20 build remains pending.

## Code Analysis

UI events route through existing scoped request methods. Native egui owns interaction and accessibility. The new styling performs bounded painting and interpolation, with no guest I/O. Transport retains checked coordinates and release recovery. No unsafe Rust, dependencies, async runtime or new input authority are introduced. Session logging and abrupt-process failure retain their existing durability limits.
