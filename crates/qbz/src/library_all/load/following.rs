//! Following: artists + labels (group "following").

use crate::favorites::{self, FavData, FavTab};

use super::super::feed::{rank, Feed};
use super::Runtime;

pub(super) async fn load(runtime: &Runtime, feed: &mut Vec<Feed>) {
    if let Ok(FavData::Artists { items, .. }) =
        favorites::load_favorites(runtime, FavTab::Artists).await
    {
        let n = items.len();
        for (i, ar) in items.into_iter().enumerate() {
            feed.push(Feed {
                kind: "artist".into(),
                group: "following".into(),
                source: "qobuz".into(),
                subtitle: String::new(),
                artist: String::new(),
                artist_id: ar.id.clone(),
                album: String::new(),
                album_id: String::new(),
                image_url: ar.image_url,
                quality_tier: String::new(),
                quality_detail: String::new(),
                is_favorite: true,
                added_rank: rank(i, n),
                id: ar.id,
                title: ar.name,
                ..Default::default()
            });
        }
    }
    if let Ok(FavData::Labels { items, .. }) =
        favorites::load_favorites(runtime, FavTab::Labels).await
    {
        let n = items.len();
        for (i, l) in items.into_iter().enumerate() {
            feed.push(Feed {
                kind: "label".into(),
                group: "following".into(),
                source: "qobuz".into(),
                subtitle: l.albums_line,
                artist: String::new(),
                artist_id: String::new(),
                album: String::new(),
                album_id: String::new(),
                image_url: l.image_url,
                quality_tier: String::new(),
                quality_detail: String::new(),
                is_favorite: true,
                added_rank: rank(i, n),
                id: l.id,
                title: l.name,
                ..Default::default()
            });
        }
    }
}
