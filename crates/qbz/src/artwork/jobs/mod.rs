//! The job-dispatch layer: `ArtworkJob`, decode-size scaling, the
//! pinned-carousel job builder, and (in `spawn.rs`) the semaphore-bounded
//! spawn functions.

mod spawn;

use super::target::ArtworkTarget;

pub use spawn::{spawn_loads, spawn_local_loads, spawn_search_loads};

/// Target decode size. Cards display at 220px; 264px keeps them crisp at
/// modest DPI without holding full ~600px source textures in memory.
pub(in crate::artwork) const DECODE_SIZE: u32 = 264;

/// Interface-size preset multiplier for decode targets, set ONCE at startup
/// (main.rs, before any artwork job runs). Under a scaled UI every card gets
/// `preset ×` more physical pixels, so decode sizes must grow with it or
/// covers go soft at Large/XL; Small shrinks them, saving decoded-cache RAM.
static UI_SCALE_FACTOR: std::sync::OnceLock<f32> = std::sync::OnceLock::new();

pub fn set_ui_scale_factor(factor: f32) {
    let _ = UI_SCALE_FACTOR.set(factor);
}

/// Scale a base decode size by the interface-size preset, rounded up.
pub(in crate::artwork) fn scaled_decode(base: u32) -> u32 {
    let factor = UI_SCALE_FACTOR.get().copied().unwrap_or(1.0);
    (base as f32 * factor).ceil() as u32
}

/// An artwork download job: which card, and the image URL.
pub struct ArtworkJob {
    pub target: ArtworkTarget,
    pub url: String,
}

/// Artwork jobs for the mixed Pinned carousel (`PinnedState.items`). One job
/// per row with art, reading the URL from the sub-struct the row's `kind`
/// selects (album / artist `artwork-url`; playlist single-cover `url1` —
/// SearchPlaylistItem has no artwork-url field). Rows without art are
/// skipped. Build ONLY from the freshly-pushed row set (see `PinnedCard`).
pub fn pinned_artwork_jobs(rows: &[crate::PinnedItem]) -> Vec<ArtworkJob> {
    rows.iter()
        .enumerate()
        .filter_map(|(idx, row)| {
            let url = match row.kind.as_str() {
                "album" => row.album.artwork_url.as_str(),
                "artist" => row.artist.artwork_url.as_str(),
                "playlist" => row.playlist.url1.as_str(),
                _ => "",
            };
            (!url.is_empty()).then(|| ArtworkJob {
                target: ArtworkTarget::PinnedCard { idx },
                url: url.to_string(),
            })
        })
        .collect()
}
