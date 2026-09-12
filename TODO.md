# TODO - Toybox After Hours

- [ ] Move tests from `src/**/tests.rs` into `tests/`; add `src/lib.rs` and expose only intentional public APIs.
- [ ] Add touch controls for every gameplay, menu, pause, and recovery action; remove the keyboard and pointer-lock requirement; update `game_page.json` and tutorial prompts; verify mobile browser captures.
- [ ] Replace the tool-rack button with a toolkit-backed widget that supports the shared release semantics and touch input.
- [ ] Split near-limit modules and refactor `Game::update` to comply with the documented function-size limit.
- [ ] Move remaining tunables and player-facing copy into typed JSON under `assets/`.
- [ ] Add load-time validation for IDs, names, references, capacities, unlocks, effects, and positive configuration values.
- [ ] Remove unused texture-manifest plumbing and correct the source-gate comment.
- [ ] Review feature test suites against the five-case target; consolidate related cases or document justified exceptions.
- [ ] Align Project Roost slug guidance across the canonical docs, templates, publisher, and project copies.
- [ ] Complete browser boot and WebGL performance verification; resolve the remote-browser or loopback access blocker.
- [ ] Coordinate cleanup of stale deployed shared-runtime files in `rust_management`.
