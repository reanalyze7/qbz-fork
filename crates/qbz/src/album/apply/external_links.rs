//! External-music-database links (Last.fm / Discogs / MusicBrainz), built
//! from artist + title. Mirrors Tauri's `AlbumExternalLinks`.

use crate::AlbumState;

use super::super::map::lastfm_segment;

/// Push the external links (or clear them when artist/title is missing) onto
/// `state`, and set `show-external-links`. `apply_album` is the Qobuz path
/// (local albums load through `LocalAlbumState`), so `is_local` is always
/// false here — gate on having both an artist and a title.
pub(super) fn apply_external_links(state: &AlbumState, artist: &str, title: &str) {
    let show_external = !artist.is_empty() && !title.is_empty();
    if show_external {
        let lastfm = format!(
            "https://www.last.fm/music/{}/{}",
            lastfm_segment(artist),
            lastfm_segment(title),
        );
        // `{artist}+{album}` query (spaces as `+`, each part percent-encoded).
        let query = format!(
            "{}+{}",
            urlencoding::encode(artist),
            urlencoding::encode(title)
        );
        let discogs = format!("https://www.discogs.com/search/?q={query}&type=release");
        let musicbrainz =
            format!("https://musicbrainz.org/search?query={query}&type=release&method=indexed");
        state.set_lastfm_url(lastfm.into());
        state.set_discogs_url(discogs.into());
        state.set_musicbrainz_url(musicbrainz.into());
    } else {
        state.set_lastfm_url("".into());
        state.set_discogs_url("".into());
        state.set_musicbrainz_url("".into());
    }
    state.set_show_external_links(show_external);
}
