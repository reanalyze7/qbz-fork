//! Two layers: `StreamFetcher` (a retrying HTTP stream-to-file downloader
//! with per-attempt fresh clients to dodge HTTP/2 connection-pool
//! poisoning) and the per-track download orchestration
//! (`try_cmaf_offline_download` for the v2 CMAF-first path,
//! `spawn_track_cache_download` as the shared entry point that tries CMAF
//! then falls back to the legacy plain-FLAC fetch, tags, artwork, and
//! library-row insertion).
//!
//! Split into `fetcher` (the reusable download primitive), `validate` (pure
//! byte-count validation), `cmaf_path` (the CMAF-first strategy), and
//! `spawn` (the shared orchestration wrapper).

mod cmaf_path;
mod fetcher;
mod spawn;
mod validate;

pub use fetcher::StreamFetcher;
pub use spawn::spawn_track_cache_download;
