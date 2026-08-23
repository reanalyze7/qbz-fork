//! Favorites: tracks + albums (group "favorites").

use crate::favorites::{self, FavData, FavTab};

use super::super::feed::{rank, Feed};
use super::Runtime;

pub(super) async fn load(runtime: &Runtime, feed: &mut Vec<Feed>) {
    if let Ok(FavData::Tracks { items, .. }) =
        favorites::load_favorites(runtime, FavTab::Tracks).await
    {
        let n = items.len();
        for (i, t) in items.into_iter().enumerate() {
            feed.push(Feed {
                kind: "track".into(),
                group: "favorites".into(),
                source: "qobuz".into(),
                subtitle: t.artist.clone(),
                artist: t.artist,
                artist_id: t.artist_id,
                album: t.album,
                album_id: t.album_id,
                image_url: t.artwork_url,
                quality_tier: t.quality_tier,
                quality_detail: t.quality_detail,
                is_favorite: true,
                genre: t.genre,
                added_rank: rank(i, n),
                id: t.id,
                title: t.title,
                ..Default::default()
            });
        }
    }
    if let Ok(FavData::Albums { items, .. }) =
        favorites::load_favorites(runtime, FavTab::Albums).await
    {
        let n = items.len();
        for (i, a) in items.into_iter().enumerate() {
            feed.push(Feed {
                kind: "album".into(),
                group: "favorites".into(),
                source: "qobuz".into(),
                subtitle: a.artist.clone(),
                artist: a.artist,
                artist_id: a.artist_id,
                album: String::new(),
                album_id: String::new(),
                image_url: a.artwork_url,
                quality_tier: a.quality_tier,
                quality_detail: a.quality_detail,
                is_favorite: true,
                genre: a.genre,
                added_rank: rank(i, n),
                id: a.id,
                title: a.title,
                ..Default::default()
            });
        }
    }
}
