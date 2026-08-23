//! Resolve the selected items' Qobuz track IDs for the bulk "Add to
//! playlist" flow.

use qbz_models::mixtape::MixtapeCollectionItem;
use qbz_mixtape::enqueue::ProdItemResolver;

use super::{resolve_local, Runtime};

/// Resolve the selected items' Qobuz track IDs for the bulk "Add to playlist"
/// flow (spec 12 §13.1). Qobuz playlists only accept Qobuz track ids, so each
/// item is resolved and only `source == "qobuz"` tracks contribute their ids
/// (local tracks are skipped — same constraint the Local Library bulk
/// add-to-playlist applies). Returns the ids in resolution order; an empty
/// result means nothing playable-to-a-Qobuz-playlist was selected.
pub async fn resolve_bulk_qobuz_track_ids(
    runtime: &Runtime,
    items: &[MixtapeCollectionItem],
) -> Vec<String> {
    use qbz_mixtape::enqueue::ItemResolver;

    let client_lock = runtime.core().client();
    let client = {
        let guard = client_lock.read().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => {
                log::warn!("[qbz-slint] myqbz_play: no Qobuz client; cannot resolve bulk ids");
                return Vec::new();
            }
        }
    };
    let resolver = ProdItemResolver::new(&client, resolve_local);

    let mut ids: Vec<String> = Vec::new();
    for item in items {
        match resolver.resolve(item).await {
            Ok(tracks) => {
                for t in tracks {
                    // Qobuz-only: a local track id is not a Qobuz playlist
                    // member. `source` is the resolver's per-track stamp.
                    let is_qobuz = t.source.as_deref() == Some("qobuz")
                        || (t.source.is_none() && !t.is_local);
                    if is_qobuz {
                        ids.push(t.id.to_string());
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "[qbz-slint] myqbz_play: bulk add-to-playlist resolve {}/{} failed: {}",
                    item.source_item_id,
                    item.title,
                    e
                );
            }
        }
    }
    ids
}
