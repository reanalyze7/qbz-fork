use qbz_models::{Artist, Playlist};

use crate::search::pure::playlist_cover_urls;
use crate::search::rows::{ArtistRow, PlaylistRow};

pub fn map_artist(artist: &Artist, following: bool) -> ArtistRow {
    ArtistRow {
        id: artist.id.to_string(),
        name: artist.name.clone(),
        subtitle: match artist.albums_count {
            Some(n) if n > 0 => qbz_i18n::tf("{} album", "{} albums", n as i64, &[&n.to_string()]),
            _ => String::new(),
        },
        artwork_url: artist
            .image
            .as_ref()
            .and_then(|i| i.best().cloned())
            .unwrap_or_default(),
        following,
    }
}

pub fn map_playlist(playlist: Playlist) -> PlaylistRow {
    let cover_urls = playlist_cover_urls(&playlist);
    let mut subtitle = playlist.owner.name.clone();
    if playlist.tracks_count > 0 {
        let count = playlist.tracks_count;
        let tracks_label = qbz_i18n::tf("{} track", "{} tracks", count as i64, &[&count.to_string()]);
        if subtitle.is_empty() {
            subtitle = tracks_label;
        } else {
            subtitle = format!("{}   •   {}", subtitle, tracks_label);
        }
    }
    let is_owned = crate::library_db::current_user_id()
        .map(|uid| uid == playlist.owner.id)
        .unwrap_or(false);
    PlaylistRow {
        id: playlist.id.to_string(),
        title: playlist.name,
        subtitle,
        cover_urls,
        is_owned,
        is_following: false,
        is_copied: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_artist_builds_album_count_subtitle() {
        let artist = Artist {
            id: 7,
            name: "Metallica".into(),
            image: None,
            albums_count: Some(12),
            biography: None,
            albums: None,
            tracks_appears_on: None,
            playlists: None,
        };
        let row = map_artist(&artist, true);
        assert_eq!(row.id, "7");
        assert_eq!(row.name, "Metallica");
        assert_eq!(row.subtitle, "12 albums");
        assert!(row.following);
    }
}
