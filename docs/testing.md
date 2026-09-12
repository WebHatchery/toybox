# Test coverage review

The five-case target is the default for a major feature. The current suites
are deliberately split by responsibility:

- `tests/data.rs`: six checks for embedded data, tool reachability, touch-first
  copy, preference ranges, layout references, and touch-first game-page
  recovery controls.
- `tests/audio.rs`: five checks for waveform generation, cue duration, ambience,
  mute behavior, and rate limiting.
- `tests/state_tests/`: 93 normal gameplay checks plus two intentionally
  ignored replay diagnostics. The state suites cover movement, targeting,
  placement, repairs, upgrades, persistence migration, records, clocks, zone
  progress, and deterministic replays.
- `tests/preferences.rs`, `tests/tutorial.rs`, `tests/part_accents.rs`, and
  `tests/scene3d.rs`: narrow adapter, progression, identity-accent, and render
  math checks. They are smaller than five because each covers one compact seam;
  the broader gameplay behavior is covered by the state and replay suites.

The six on-disk persistence checks remain normal tests rather than being
silently skipped. The read-only missing-file check passes here, while the five
write cases report `Access is denied` because this managed workspace denies the
test process write access to the Windows `dirs::data_local_dir()` root. Run the
same suite on an unrestricted Windows checkout to exercise those five cases.
The two ignored replay reports are diagnostic balance sweeps and are run
explicitly when retuning `toy_count` or the full-shift route.
