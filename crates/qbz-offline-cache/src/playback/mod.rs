//! Offline-cache → audio-bytes bridge for playback.
//!
//! When a track is played back from the offline cache (either via the
//! main playback path or via the Local Library path when the track is a
//! Qobuz-cached offline entry), this module converts the stored row
//! into a `Vec<u8>` ready for `player.play_data`.
//!
//! For `cache_format = 2` (v2 CMAF bundle) this means:
//! 1. Read init.mp4 + segments.bin + manifest.json from disk
//! 2. Unwrap the content_key via the secret vault
//! 3. Decrypt the encrypted frames and prepend the FLAC header
//!
//! For `cache_format = 1` (legacy plain FLAC) the caller should just
//! `std::fs::read(file_path)` directly — this module doesn't handle v1
//! since v1 needs no extra work.
//!
//! Split into `resolve` (the PURE resolution — no Tauri, no events) and
//! `ui_events` (the async wrapper that emits unlock-start/end through the
//! `CacheEventSink`).

mod decrypt;
mod resolve;
mod ui_events;

pub use resolve::load_cmaf_bundle;
pub use ui_events::load_cmaf_bundle_with_ui_events;
