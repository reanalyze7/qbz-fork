//! CMAF streaming pipeline for Qobuz.
//!
//! Qobuz's modern mobile client uses CMAF (Common Media Application Format)
//! segmented streaming over Akamai CDN, with AES-CTR per-frame encryption.
//! This is the pipeline that the v9.7.0.3 Android app uses and the one we
//! need to match if we want to stay compatible as Qobuz deprecates the
//! legacy `/track/getFileUrl` nginx path.
//!
//! # Pipeline shape
//!
//! 1. `/file/url` returns `{ url_template, key (wrapped), n_segments, ... }`
//! 2. `/session/start` returns `{ session_id, infos }` — the `infos` string
//!    is the HKDF salt needed to derive the per-session AES key
//! 3. Session key = `HKDF(CMAF_SEED, infos)`
//! 4. Content key = unwrap(session_key, key) — this is the per-track AES key
//! 5. Fetch init segment (s=0) → parse FLAC header + segment table
//! 6. For each s=1..n_segments: fetch → parse crypto boxes → decrypt frames
//!    in place → emit decrypted FLAC frames to the consumer
//!
//! # Why live in `qbz-qobuz` and not `qbz-cmaf`
//!
//! `qbz-cmaf` is pure parsing + crypto primitives (no I/O, no Qobuz client).
//! This module is the Qobuz-specific orchestration: it calls `/file/url`,
//! `/session/start`, owns the Akamai HTTP client, and returns ready-to-play
//! or ready-to-store bundles.
//!
//! # Why two variants
//!
//! - [`download_full`] — returns the fully decrypted FLAC as `Vec<u8>`. Used
//!   by the playback pipeline for in-memory cache writes and eager downloads.
//! - [`download_raw`] — returns a [`CmafRawBundle`] of **encrypted** segments
//!   plus key material. Used by the offline cache so we can persist
//!   bit-identical bytes to what Qobuz delivered, and decrypt only at
//!   playback time. This is the security-sensitive path.

mod decrypt;
mod download_full;
mod download_raw;
mod fetch;
mod setup;

pub use decrypt::decrypt_segments_into;
pub use download_full::{
    download_full, download_full_with_progress, download_full_with_quality,
    download_full_with_quality_progress,
};
pub use download_raw::{download_raw, download_raw_with_progress};
pub use setup::setup_streaming;

/// Concurrency cap for the full-download path. 3 segments in flight is the
/// empirically-determined sweet spot — Akamai CDN rate-limits with 1s windows
/// past ~5 parallel requests per client IP.
pub const CMAF_PREFETCH_CONCURRENCY: usize = 3;

/// Progress callback shape for the download helpers. Each call reports
/// "k of n segments complete" with the bytes received for that segment,
/// so the caller can emit UI progress events without knowing the CMAF
/// internals. Callbacks must be `Send + Sync` because segments are
/// fetched in parallel.
pub type CmafProgressCallback = std::sync::Arc<dyn Fn(CmafProgressUpdate) + Send + Sync>;

/// A single progress tick. `segments_completed` is cumulative (1..=n),
/// `n_segments` is the total including the init segment if you count it.
#[derive(Debug, Clone, Copy)]
pub struct CmafProgressUpdate {
    pub segments_completed: u32,
    pub n_segments: u32,
    pub bytes_this_segment: u64,
}

/// Info gathered from the CMAF init segment, enough to start streaming
/// playback. The caller is expected to fetch audio segments 1..n_segments
/// and feed them through [`qbz_cmaf::parse_segment_crypto`] +
/// [`qbz_cmaf::decrypt_frame`].
pub struct CmafStreamingInfo {
    pub url_template: String,
    pub n_segments: u8,
    pub content_key: [u8; 16],
    pub flac_header: Vec<u8>,
    pub segment_table: Vec<qbz_cmaf::SegmentTableEntry>,
    pub format_id: u32,
    pub sampling_rate: Option<u32>,
    pub bit_depth: Option<u32>,
    /// How long the init segment fetch took (ms), for speed estimation.
    pub init_fetch_ms: u64,
}

/// Raw (encrypted) CMAF bundle suitable for offline storage.
///
/// Everything in this struct is **bit-identical** to what Qobuz's CDN
/// returned. In particular:
///
/// - `init_bytes` is the raw init segment (unencrypted mp4 box with the
///   FLAC header inside — cheap to store).
/// - `segments` are the raw encrypted segment mp4 files, one per
///   `s=1..=n_segments`. These are useless without `content_key` and
///   without running them through the CMAF decrypt pipeline.
/// - `content_key` is the 16-byte AES key unwrapped from the session key;
///   it must be stored **encrypted at rest** on the caller's side.
/// - `infos` is the original `session/start` infos string. With the
///   `CMAF_SEED` constant this is enough to re-derive `session_key` and
///   re-unwrap the content key if we ever need to audit or migrate.
///
/// The intent is that an attacker who copies the user's offline directory
/// out without also extracting the OS-keyring wrapped `content_key` gets
/// nothing usable — the segments are encrypted, the `infos` is just a
/// salt, and the seed alone isn't enough.
pub struct CmafRawBundle {
    pub init_bytes: Vec<u8>,
    pub segments: Vec<Vec<u8>>,
    pub content_key: [u8; 16],
    pub infos: String,
    pub format_id: u32,
    pub sampling_rate: Option<u32>,
    pub bit_depth: Option<u32>,
    pub n_segments: u8,
}

// Silence "unused imports" if we end up not using everything at some point;
// the Result alias is kept for future variants that want to surface ApiError.
#[allow(dead_code)]
fn _type_assertions() {
    let _: fn() -> crate::error::Result<()> = || Ok(());
}
