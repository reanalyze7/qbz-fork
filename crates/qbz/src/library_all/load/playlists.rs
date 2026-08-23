//! Playlists: owned/hearted = favorites, followed = following.

use crate::favorites::{self, FavData, FavTab};

use super::super::feed::{rank, Feed};
use super::Runtime;

pub(super) async fn load(runtime: &Runtime, feed: &mut Vec<Feed>) {
    let Ok(FavData::Playlists {
        favorites: fav_pl,
        following: fol_pl,
    }) = favorites::load_favorites(runtime, FavTab::Playlists).await
    else {
        return;
    };

    let n = fav_pl.len();
    for (i, p) in fav_pl.into_iter().enumerate() {
        let image_url = p.cover_urls.iter().next().cloned().unwrap_or_default();
        feed.push(Feed {
            kind: "playlist".into(),
            group: "favorites".into(),
            source: "qobuz".into(),
            subtitle: p.subtitle,
            artist: String::new(),
            artist_id: String::new(),
            album: String::new(),
            album_id: String::new(),
            image_url,
            quality_tier: String::new(),
            quality_detail: String::new(),
            is_favorite: true,
            playlist_owned: p.is_owned,
            playlist_following: p.is_following,
            playlist_copied: p.is_copied,
            added_rank: rank(i, n),
            id: p.id,
            title: p.title,
            ..Default::default()
        });
    }
    let n = fol_pl.len();
    for (i, p) in fol_pl.into_iter().enumerate() {
        let image_url = p.cover_urls.iter().next().cloned().unwrap_or_default();
        feed.push(Feed {
            kind: "playlist".into(),
            group: "following".into(),
            source: "qobuz".into(),
            subtitle: p.subtitle,
            artist: String::new(),
            artist_id: String::new(),
            album: String::new(),
            album_id: String::new(),
            image_url,
            quality_tier: String::new(),
            quality_detail: String::new(),
            is_favorite: false,
            playlist_owned: p.is_owned,
            playlist_following: p.is_following,
            playlist_copied: p.is_copied,
            added_rank: rank(i, n),
            id: p.id,
            title: p.title,
            ..Default::default()
        });
    }
}
