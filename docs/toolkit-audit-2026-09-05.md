# Toolkit audit — 5 September 2026

The original review had no named finding. Current inspection found and migrated:

- Generated cue and ambience storage, decoding, playback, stopping and volume
  updates now use SoundManager. Voices, seeds, clamps, loop state and cue
  cooldown rules remain unchanged. A missing cue still consumes no cooldown.
- `src/ui/widgets.rs` delegates word wrapping to the shared measured wrapper,
  retaining the line cap, baseline spacing, per-line clipping and return value.
- The remaining texture-manifest parse now uses labeled toolkit loading.

The game already uses shared asset packs, settings/preferences, save slots,
SeededRng, EventBus, notifications, debug overlay, text, raster and color helpers.
Fixed seed mixing and coordinate hashes preserve existing layout identity;
gameplay randomness uses the toolkit. Save schemas, toy repair, placement,
gallery rendering and authored cooldown rules remain game-owned.

Also fixed an unrelated needless borrow in gallery setup. Final validation:
109 checks pass, two existing tests ignored; formatting, strict all-target/
all-feature Clippy and Rust source-size limits pass. Default `publish.ps1`
passed Windows/WebGL release builds, asset packaging, Preview deployment and
Project Roost tracking.
