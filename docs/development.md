# Development conventions for the 1.1.0 baseline

This release keeps the Solver-driven TriPeaks and Pyramid behaviour established
in v1.0.4 candidate 6. The refactor prepares clear boundaries for additional
game modes. It does not implement rank recognition, Dijkstra, A*, another game
mode or a file-free capture backend.

## Naming and documentation

- Use `snake_case` for modules, functions, methods, fields and local bindings.
- Use `UpperCamelCase` for types, traits and enum variants.
- Use `SCREAMING_SNAKE_CASE` for constants and statics.
- Prefer module-qualified names where two modules expose the same concept,
  such as `pyramid::TABLEAU_CARD_COUNT`, rather than a renamed import.
- Numeric `as` casts are conversions, not name aliases. Review their bounds
  and meaning; do not replace them as a cosmetic naming change.
- Place meaningful `///` documentation on functions, methods, constants,
  statics and type definitions. Explain the contract and non-obvious parameters,
  result, failure conditions or authority granted. Document important fields
  and enum variants. Place `//!` documentation at module boundaries.
- Ordinary local bindings and implementation explanations use `//`; attaching
  item documentation to a local `let` does not provide a function contract.
- Keep test helpers and regression cases documented too. A test-only helper
  belongs under `#[cfg(test)]`; do not silence dead-code warnings globally.

These conventions follow the [Rust API naming guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
and [Rust Reference documentation-comment rules](https://doc.rust-lang.org/reference/comments.html).
The crate enables missing-documentation warnings, including Clippy's private
item check. These checks complement review of what each comment actually says.

## Responsibilities and behaviour preservation

`game.rs` defines typed profiles/actions, while each mode owns its detection
and effect policy. The worker owns the QMP connection and cancellation state.
Its `post_game` submodule shares the existing terminal-screen sequence; its
`pyramid_execution` submodule contains Pyramid-specific result observation.
The UI delegates persistent log I/O to `session_log.rs`.

When adding a mode, supply its profile/detection policy and preserve the shared
input and capture contracts. Do not scatter new mode checks through QMP or
session logging. A new mode's special rules should remain visible at the game
policy boundary.

The unused provisional `card_reader.rs`, inactive card-rank/history types and
their reader-only thresholds were removed. The provisional reader never
returned a recognised visible rank: it deliberately reported incomplete
calibration. The old implementation is recoverable from Git commit
`dccb7f9ae7ff11acae615f029f6e183b15484a63` if useful during future self-solving
work. Active card geometry and its validation coverage remain.

Preserve capture dimensions, calibrated probes, click coordinates, timing
defaults, QMP command policy and manual PNG byte identity during a structural
refactor. Keep fresh-HALO continuation distinct from proven action effect and
from board/game completion. A progress-bar reading alone is not a win.

## Local verification

Run from the repository root using the pinned toolchain in `rust-toolchain.toml`.
Each command is a separate gate; stop on errors.

```zsh
echo 'STEP 1: Check formatting'
cargo fmt --all -- --check
```

```zsh
echo 'STEP 2: Check production compilation'
cargo check --locked
```

```zsh
echo 'STEP 3: Run regression tests'
cargo test --locked
```

```zsh
echo 'STEP 4: Generate private-item documentation'
cargo doc --locked --no-deps --document-private-items
```

```zsh
echo 'STEP 5: Check Clippy diagnostics'
cargo clippy --locked --all-targets
```

```zsh
echo 'STEP 6: Build the release application'
cargo build --locked --release
```

Rustdoc output includes private modules, so IDE hover documentation and generated
documentation describe the same implementation contracts. `cargo doc` does not
start the GUI or send guest input.

After structural changes, verify both game modes on the Beast: Capture Frame,
manual Capture PNG, one-step and bounded multi-step input, repeated pile pairs,
STOP, board redeal and the post-game sequence. Check the output panel expanded
and collapsed, and verify that Copy Output retains complete session history.
Charlie signs off the candidate and pushes the tested source to main. That push
becomes the exact base for the next patch cycle.
