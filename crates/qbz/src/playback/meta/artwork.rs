//! Now-playing cover resolution: the bar-size decode.
//!
//! The higher-res decode that also feeds the immersive-view ambient
//! background/glow lives in the sibling `artwork_large` module — it is
//! four times the size of this one and pulls in the whole ambient
//! derivation, which has nothing to do with the bar. It is re-exported
//! from here so `push_ui` keeps importing both from one path.
use slint::ComponentHandle;

use crate::{AppWindow, NowPlayingState};

pub(super) use super::artwork_large::load_now_playing_artwork_large;

/// Resolve the now-playing cover and apply it to `NowPlayingState`.
///
/// Takes a source-aware [`qbz_models::ArtworkRef`] so local-library covers
/// reach the now-playing bar, not just remote Qobuz URLs.
pub(super) fn load_now_playing_artwork(weak: slint::Weak<AppWindow>, art: qbz_models::ArtworkRef) {
    if art.is_empty() {
        return;
    }
    let Some(cache) = crate::artwork::shared_cache() else {
        return;
    };
    tokio::spawn(async move {
        let Some((pixels, w, h)) = crate::artwork::fetch_and_decode_ref(&art, &cache, 160).await
        else {
            return;
        };
        let _ = weak.upgrade_in_event_loop(move |win| {
            let img = crate::artwork::pixels_to_image(&pixels, w, h);
            win.global::<NowPlayingState>().set_artwork(img);
        });
    });
}
