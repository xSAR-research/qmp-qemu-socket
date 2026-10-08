# Klondike red card backs — v2.0.0, candidate 6

Base: `98f974083246044d04db5cf4298fd07590733250`, promoted candidate 5.

## Problem and resulting behaviour

The two supplied native 1920×1080 captures show a red-backed Klondike deal. The existing seven-column scene check accepts white paper, blue card artwork and green empty-slot outlines. It recognises only one tableau strip in this deal: each of the six red-backed columns contributes 184/552 white border pixels, below the unchanged 70% threshold. All four felt gutters and all four foundation strips already pass.

The failed scene gate suppresses both HALO scanning and the existing bounded Solver recovery. The active Solver capture contains a valid column-seven Jack of spades HALO with bounds (1398,443,132,189); the HALO geometry and colour checks need no adjustment. The stock interior also uses the blue-only card-back predicate, so merely repairing the tableau strips would leave later red-stock Draw highlights unsupported.

The patch adds the measured red card-back colour relation to both existing artwork checks: red ≥150, green <70, blue ≤100, with red exceeding both other channels by at least 80. The strict green bound separates this artwork from every accepted gold predicate. Some antialiased top-strip pixels fall outside that band; each red column still supplies 460/552 accepted pixels (83.33%), above the unchanged 70% requirement. All scene fractions, minimum column/foundation counts, clear gutters, Solver-banner requirement and source-outline rules remain in place. Existing blue artwork remains accepted. The controller's action, recovery and terminal sequence logic is unchanged.

## Input policy

Strategy selection, Game Type changes and manual Capture remain input-free. During an explicitly requested Step Once or Multi-Step run, a supported no-HALO gameplay scene receives the existing bounded delayed captures and completion review, followed by at most one Solver activation/refresh and fresh evidence. A visible valid HALO directly authorises the canonical action; the patch does not toggle Solver first.

This patch recognises the two evidenced back palettes. Other unmeasured artwork remains outside its support contract.

## Code Analysis

Threat surfaces: captured pixels reach the existing bounded PNG decoder and fixed scene/source probes. Supporting a second colour palette expands accepted artwork but retains independent layout, felt, Solver banner and HALO gates.
Memory safety: safe Rust, checked frame access and widened fractional arithmetic; no new unsafe code or transport authority.
Boundary limits: native 1920×1080 frame contract, existing fixed probe sizes, delayed-capture bounds and one-Solver-refresh policy remain unchanged.
Failure modes: unsupported palettes, malformed frames, missing scene structure, overlays or missing required banner still reject input. Initial read-only captures do not activate Solver. Live guest acceptance is required on the Beast.
