//! `load`: rebuild the rail and push it into `OfflineFavoritesState`.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, OfflineFavoritesState, SlimItem};

use super::gather::gather;
use super::state::RAIL_QUEUE;

/// Rebuild the rail. Fired by the Slint rail's `init` on every mount of
/// the Favorites offline placeholder (ADR-010 conditional mount, so each
/// entry re-reads the three local stores — all cheap local reads).
pub fn load(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let (rows, queue) = gather().await;

        if let Ok(mut q) = RAIL_QUEUE.lock() {
            *q = queue;
        }
        let _ = weak.upgrade_in_event_loop(move |w| {
            let items: Vec<SlimItem> = rows
                .into_iter()
                .map(|rd| SlimItem {
                    id: rd.id.into(),
                    title: rd.title.into(),
                    subtitle: rd.artist.into(),
                    rank: "".into(),
                    artwork_url: "".into(),
                    artwork: rd
                        .cover
                        .map(|(px, pw, ph)| crate::artwork::pixels_to_image(&px, pw, ph))
                        .unwrap_or_default(),
                    following: false,
                    // Track slims render pin-less rows — tracks are not
                    // pinnable.
                    is_pinned: false,
                })
                .collect();
            w.global::<OfflineFavoritesState>()
                .set_tracks(ModelRc::new(VecModel::from(items)));
        });
    });
}
