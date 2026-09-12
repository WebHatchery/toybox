# TODO - Toybox After Hours

- [x] Move tests from `src/**/tests.rs` into `tests/`; add `src/lib.rs` and expose only intentional public APIs.
- [blocked] Add touch controls for every gameplay, menu, pause, and recovery action; remove the keyboard and pointer-lock requirement; update `game_page.json` and tutorial prompts; verify mobile browser captures. The implementation and generated page are touch-first, but the managed browser denied loopback access for the final mobile capture.
- [x] Replace the tool-rack button with a toolkit-backed widget that supports the shared release semantics and touch input.
- [x] Split near-limit modules and refactor `Game::update` to comply with the documented function-size limit.
- [x] Move remaining tunables and player-facing copy into typed JSON under `assets/`.
- [x] Add load-time validation for IDs, names, references, capacities, unlocks, effects, and positive configuration values.
- [x] Remove unused texture-manifest plumbing and correct the source-gate comment.
- [x] Review feature test suites against the five-case target; consolidate related cases or document justified exceptions in `docs/testing.md`.
- [blocked] Align Project Roost slug guidance across the canonical docs, templates, publisher, and project copies. The project copy now documents the publisher's observed canonical `rust_<project_directory>` behavior, but the external canonical/template files are read-only in this session.
- [blocked] Complete browser boot and WebGL performance verification; resolve the remote-browser or loopback access blocker. Windows/WebGL builds and a native 10-second benchmark pass; Preview deployment and loopback browser access are blocked by the managed environment.
- [x] Coordinate cleanup of stale deployed shared-runtime files in `rust_management`; the audit found no divergent bridge files requiring deletion. See `docs/verification/shared-runtime-audit-2026-09-12.md`.
