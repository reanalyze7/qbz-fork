# crates/qbz-audio/src/analysis/spectral_ribbon.rs (205 lines)

## Summary
`SpectralAnalyzer`: a progressive FFT-based spectral analyzer feeding the
immersive Spectral Ribbon visualizer — pre-allocated buffers, a Hann window,
log-spaced band bin ranges, and exponential smoothing with fast attack /
slower release, gated to a configurable update cadence.

## Proposed split
By responsibility — convert the file into an `spectral_ribbon/` directory
with a thin `mod.rs` holding the struct + constructor.

- `spectral_ribbon/mod.rs` (~95 lines) — `MIN_FREQ_HZ`/`MAX_FREQ_HZ` consts,
  the `SpectralAnalyzer` struct definition, `SpectralAnalyzer::new(...)`,
  `get_latest_bands()`. Declares `mod process; mod bands; #[cfg(test)] mod tests;`
  and needs no `pub use` since `SpectralAnalyzer` itself is defined here
  (the struct's other methods attach via additional `impl SpectralAnalyzer`
  blocks in the sibling files — Rust allows multiple `impl` blocks for one
  struct across files in the same module).
- `spectral_ribbon/process.rs` (~75 lines) — `impl SpectralAnalyzer { pub fn process_audio_frame(...) -> bool }`,
  the per-frame hot path: windowing, FFT, magnitude computation, per-band RMS
  energy, exponential smoothing.
- `spectral_ribbon/bands.rs` (~45 lines) — `impl SpectralAnalyzer { fn rebuild_window(...); fn rebuild_band_ranges(...) }`,
  the Hann-window and log-spaced band-bin-range setup, called from `new()`
  and re-called from `process_audio_frame` when the sample rate changes.
- `spectral_ribbon/tests.rs` (~15 lines) — the existing
  `#[cfg(test)] mod tests` block (`spectral_analyzer_returns_expected_band_count`),
  included via `#[cfg(test)] mod tests;` from `mod.rs`.

## Re-export surface
`spectral_ribbon/mod.rs` is the public-API surface: `pub struct SpectralAnalyzer`
is defined there directly (not re-exported from elsewhere), so
`crate::analysis::spectral_ribbon::SpectralAnalyzer` (or however
`analysis/mod.rs` currently declares `mod spectral_ribbon;`) keeps its exact
import path — only the file becomes a directory (`spectral_ribbon.rs` →
`spectral_ribbon/mod.rs`), which is transparent to `mod spectral_ribbon;` in
the parent.

## Coupling / watch out
- All private fields (`fft`, `fft_input`, `magnitudes`, `band_bin_ranges`,
  `bands_raw`, `bands_smoothed`, `latest_bands`, `window`, `sample_rate_hz`,
  `num_bands`, `frame_interval`, `last_update`) are read/written from both
  `process.rs` and `bands.rs` — since both are `impl` blocks for the same
  struct within the same module (`spectral_ribbon`), private-field access
  from sibling files works fine in Rust (privacy is module-scoped, not
  file-scoped), so no visibility changes are needed here — just make sure
  the struct definition itself stays in `mod.rs` since that's the single
  source of truth for the field list.
- `rebuild_band_ranges` is called both from `new()` (in `mod.rs`) and mid-stream
  from `process_audio_frame` (in `process.rs`) when `sample_rate_hz` changes
  — keep this cross-file call working via `self.rebuild_band_ranges(...)`
  (same-struct method call, no explicit import needed).
- The clamping logic in `new()` (`fft_size` must be one of 512/1024/2048/
  4096/8192, `num_bands` clamped 48..=1024, `update_rate_hz` clamped 20..=60,
  `smoothing_factor` clamped 0.0..=0.98) must stay exactly as-is in `mod.rs`
  — these are documented invariants relied on by `process.rs`/`bands.rs`
  (e.g. `magnitudes` sized `fft_size / 2`, `band_bin_ranges` sized
  `num_bands`).

## Verify after split
- `cargo build -p qbz-audio`
- `cargo test -p qbz-audio spectral_ribbon` (the existing
  `spectral_analyzer_returns_expected_band_count` test must stay green,
  unchanged assertion).
- Grep for `SpectralAnalyzer` usage across `qbz-audio`/`qbz-ui` (the
  immersive visualizer feed) to confirm no import path broke.
- Manual smoke test: enable the Spectral Ribbon visualizer during playback
  and confirm bands still animate plausibly (no panics, no all-zero output)
  across a sample-rate change (e.g. switching between a 44.1kHz and 96kHz
  track) since that's the one runtime branch (`rebuild_band_ranges` on
  sample-rate change) not covered by the existing unit test.
