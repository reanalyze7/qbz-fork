//! Resolves a MixtapeCollection's items into a flat Vec<QueueTrack>, then the
//! caller applies it to the queue per the enqueue mode.
//!
//! The resolver is split into a trait so tests can use a mock without real
//! API / DB calls.
//!
//! Frontend-agnostic notes (ADR-006):
//! - The Qobuz resolvers are async free fns over `qbz_qobuz::QobuzClient`.
//! - The local resolvers are SYNCHRONOUS free fns over
//!   `qbz_library::LibraryDatabase`. `&LibraryDatabase`
//!   wraps a `rusqlite::Connection`, which is `!Sync`, so a `&LibraryDatabase`
//!   is `!Send` and must NEVER be held across an `.await`. Keeping local
//!   resolution in a synchronous free fn enforces that at the type level: the
//!   caller does its own DB access (e.g. Slint's `with_db(|db| ...)`) and the
//!   crate bakes in no specific handle type.

mod boundary;
mod local;
mod mapping;
mod prod_resolver;
mod qobuz;

pub use boundary::{next_item_index, previous_item_index};
pub use local::{resolve_local_album, resolve_local_album_tracks, resolve_local_item, resolve_local_track};
pub use mapping::{local_track_to_queue_track, track_to_queue_track_from_api};
pub use prod_resolver::ProdItemResolver;
pub use qobuz::{resolve_qobuz_album, resolve_qobuz_playlist, resolve_qobuz_track};

use qbz_models::mixtape::{CollectionPlayMode, MixtapeCollectionItem};
use qbz_models::QueueTrack as CoreQueueTrack;

/// Trait for expanding a single Mixtape item into its tracks. Implementations:
/// - `ProdItemResolver`    — uses the Qobuz client + a caller-supplied local
///   resolver (the real production path)
/// - mocks in `#[cfg(test)]`
#[async_trait::async_trait]
pub trait ItemResolver: Send + Sync {
    async fn resolve(&self, item: &MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String>;
}

/// Apply play_mode to item ordering, then resolve each item and flatten.
/// Failed items are logged and skipped (partial playback > total failure).
/// Every track produced by a single item has its `source_item_id_hint`
/// stamped with the owning item's `source_item_id` for skip-to-item boundary
/// detection downstream.
pub async fn resolve_collection_tracks(
    items: Vec<MixtapeCollectionItem>,
    play_mode: CollectionPlayMode,
    resolver: &dyn ItemResolver,
) -> Vec<CoreQueueTrack> {
    let items = if matches!(play_mode, CollectionPlayMode::AlbumShuffle) {
        shuffle_items(items)
    } else {
        items
    };

    let mut out = Vec::new();
    for item in items {
        match resolver.resolve(&item).await {
            Ok(mut tracks) => {
                let hint = item.source_item_id.clone();
                for track in &mut tracks {
                    track.source_item_id_hint = Some(hint.clone());
                }
                out.extend(tracks);
            }
            Err(e) => {
                log::warn!(
                    "[Mixtape/enqueue] skipping item {:?}/{}: {}",
                    item.source,
                    item.source_item_id,
                    e
                );
            }
        }
    }
    out
}

/// Shuffle the ITEM order for `album_shuffle` play mode. Time-seeded ⇒ a
/// different order every play. Each item later expands to its tracks IN ORDER,
/// so albums stay contiguous and internally ordered.
pub fn shuffle_items(mut items: Vec<MixtapeCollectionItem>) -> Vec<MixtapeCollectionItem> {
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    items.shuffle(&mut rng);
    items
}

#[cfg(test)]
mod tests;
