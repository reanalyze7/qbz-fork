//! Shared `MockResolver` used by several enqueue tests.

use qbz_models::mixtape::AlbumSource;
use qbz_models::QueueTrack as CoreQueueTrack;

use crate::enqueue::ItemResolver;
use qbz_models::mixtape::MixtapeCollectionItem;

pub(super) struct MockResolver;

#[async_trait::async_trait]
impl ItemResolver for MockResolver {
    async fn resolve(&self, item: &MixtapeCollectionItem) -> Result<Vec<CoreQueueTrack>, String> {
        let n = item.track_count.unwrap_or(1).max(1) as usize;
        Ok((0..n)
            .map(|i| CoreQueueTrack {
                id: i as u64,
                title: format!("{}-t{}", item.title, i),
                version: None,
                artist: item.subtitle.clone().unwrap_or_default(),
                album: item.title.clone(),
                album_version: None,
                duration_secs: 180,
                artwork_url: None,
                hires: false,
                bit_depth: Some(16),
                sample_rate: Some(44.1),
                is_local: matches!(item.source, AlbumSource::Local),
                album_id: Some(item.source_item_id.clone()),
                artist_id: None,
                streamable: true,
                source: Some(match item.source {
                    AlbumSource::Qobuz => "qobuz".into(),
                    AlbumSource::Local => "local".into(),
                }),
                parental_warning: false,
                source_item_id_hint: None, // stamped by resolve_collection_tracks
                context_kind: None,
                context_id: None,
            })
            .collect())
    }
}
