//! Discover > Recommendations (the 4th tab) controller.
//!
//! Wires the `qbz-external-reco` engine to Slint: a RecoCatalog over QbzCore,
//! the per-user resolution-cache lifecycle, the scrobbler-username gate, and a
//! PROGRESSIVE apply — each row paints the moment its builder resolves (the For
//! You branch pattern), so the tab fills in incrementally instead of all at once.
//!
//! Lineup: Recommended Artists + Recommended Albums (Last.fm), Fresh Releases +
//! Weekly Exploration/Jams (ListenBrainz), Deep-cut albums, and a Qobuz editorial
//! cold-start fallback.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use qbz_external_reco::ArtistReco;

mod album_similar;
mod apply_albums;
mod apply_all;
mod apply_artists_tracks;
mod apply_rows;
mod artist_dismiss;
mod artist_rails;
mod catalog;
mod loader;
mod row_kinds;

pub use album_similar::load_similar_albums_seeded;
pub(crate) use apply_rows::album_card;
pub use artist_dismiss::apply_artist_dismissal;
pub use loader::{ensure_loaded, force_reload};
pub use row_kinds::list_track_ids;

use catalog::CoreRecoCatalog;

static CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Retained per-rail overflow (validated candidates past the visible cap) for
/// the two Recommended-Artist rails — (common, recent). Kept for live
/// backfill after a "not interested" dismissal, so a replacement card needs
/// no extra network. Rewritten on every rail paint (fresh build AND cached
/// blob paint; old blobs carry no overflow, so they simply don't backfill
/// until they expire).
static ARTIST_OVERFLOW: Mutex<(Vec<ArtistReco>, Vec<ArtistReco>)> =
    Mutex::new((Vec::new(), Vec::new()));

pub fn init_for_user(base_dir: &Path) {
    if let Ok(mut g) = CACHE_DIR.lock() {
        *g = Some(base_dir.to_path_buf());
    }
    match qbz_external_reco::RecoCache::open_at(base_dir) {
        Ok(cache) => {
            let _ = cache.cleanup_expired();
            log::info!("[reco] cache initialized at {}", base_dir.display());
        }
        Err(e) => log::warn!("[reco] cache open failed at {}: {e}", base_dir.display()),
    }
}

#[allow(dead_code)]
pub fn teardown() {
    if let Ok(mut g) = CACHE_DIR.lock() {
        *g = None;
    }
}

fn rotation_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}
