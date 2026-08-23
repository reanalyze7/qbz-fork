//! Local favorites (source "local"; gated by show-local in derive).
//! group "local" — bypasses the Qobuz source switches.

use super::super::feed::{rank, Feed};

pub(super) fn load(feed: &mut Vec<Feed>) {
    let locals = crate::local_favorites::list();
    let n = locals.len();
    for (i, lf) in locals.into_iter().enumerate() {
        feed.push(Feed {
            kind: lf.kind,
            group: "local".into(),
            source: lf.source,
            subtitle: lf.subtitle,
            artist: lf.artist.clone(),
            artist_id: String::new(),
            album: String::new(),
            album_id: String::new(),
            image_url: lf.artwork_url,
            quality_tier: String::new(),
            quality_detail: String::new(),
            is_favorite: true,
            added_rank: rank(i, n),
            id: lf.id,
            title: lf.title,
            ..Default::default()
        });
    }
}
