//! Production `ItemResolver`: dispatches to async Qobuz calls or a
//! caller-supplied sync local closure.

use qbz_models::mixtape::{AlbumSource, ItemType, MixtapeCollectionItem};
use qbz_models::QueueTrack as CoreQueueTrack;

use super::qobuz::{resolve_qobuz_album, resolve_qobuz_playlist, resolve_qobuz_track};
use super::ItemResolver;

/// Production resolver. Holds a reference to the shared Qobuz client and a
/// caller-supplied `local` closure that resolves local items
/// synchronously.
///
/// `&qbz_library::LibraryDatabase` is `!Send`/`!Sync` (it wraps a rusqlite
/// `Connection`), so it cannot be stored here without breaking the
/// `ItemResolver: Send + Sync` bound. Instead the caller supplies a
/// `Send + Sync` closure that performs the DB access in its own synchronous
/// scope (e.g. Slint's `with_db(|db| resolve_local_album_tracks(db, key))`).
/// This keeps the crate free of any frontend's DB-handle type.
pub struct ProdItemResolver<'a, L>
where
    L: Fn(&MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> + Send + Sync,
{
    pub client: &'a qbz_qobuz::QobuzClient,
    pub local: L,
}

impl<'a, L> ProdItemResolver<'a, L>
where
    L: Fn(&MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> + Send + Sync,
{
    /// Build a production resolver from the shared Qobuz client and a local
    /// resolver closure. The closure is invoked only for `AlbumSource::Local`
    /// items and must perform its DB access synchronously.
    pub fn new(client: &'a qbz_qobuz::QobuzClient, local: L) -> Self {
        Self { client, local }
    }
}

#[async_trait::async_trait]
impl<'a, L> ItemResolver for ProdItemResolver<'a, L>
where
    L: Fn(&MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> + Send + Sync,
{
    async fn resolve(&self, item: &MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> {
        match (item.item_type, item.source) {
            (ItemType::Album, AlbumSource::Qobuz) => {
                resolve_qobuz_album(self.client, &item.source_item_id).await
            }
            (ItemType::Track, AlbumSource::Qobuz) => {
                let track_id: u64 = item
                    .source_item_id
                    .parse()
                    .map_err(|_| format!("invalid qobuz track id: {}", item.source_item_id))?;
                resolve_qobuz_track(self.client, track_id).await
            }
            (ItemType::Playlist, AlbumSource::Qobuz) => {
                let playlist_id: u64 = item
                    .source_item_id
                    .parse()
                    .map_err(|_| format!("invalid qobuz playlist id: {}", item.source_item_id))?;
                resolve_qobuz_playlist(self.client, playlist_id).await
            }
            // All local resolution is delegated to the caller-supplied
            // synchronous closure (no `&LibraryDatabase` held across `.await`).
            (_, AlbumSource::Local) => (self.local)(item),
        }
    }
}
