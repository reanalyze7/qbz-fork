mod extras;
mod releases;

use qbz_models::PageArtistResponse;

use extras::{map_playlists, map_similar_artists};
use releases::map_releases;

use crate::artist::data::ArtistData;
use crate::artist::track_map::map_track;

pub(crate) fn map_artist(page: PageArtistResponse) -> ArtistData {
    let name = page.name.display;

    // Biography: content (HTML-stripped) + source name (when present). The
    // /artist/page biography.source is a raw JSON value because Qobuz
    // sometimes returns a string and sometimes an object; we only care
    // about the string form.
    let (bio, bio_source) = match page.biography {
        Some(biography) => {
            let content = biography
                .content
                .map(|c| crate::strip_html::strip_html(&c))
                .unwrap_or_default();
            // Entity-decode (no tags expected in a source name, but the
            // same CMS that emits `&copy` in bodies feeds this field).
            let source = biography
                .source
                .and_then(|v| v.as_str().map(crate::strip_html::decode_html_entities))
                .unwrap_or_default();
            (content, source)
        }
        None => (String::new(), String::new()),
    };
    let bio_short = truncate_words(&bio, 360);
    let bio_truncated = bio_short != bio;

    let artwork_url = page
        .images
        .and_then(|images| images.portrait)
        .map(|portrait| {
            format!(
                "https://static.qobuz.com/images/artists/covers/large/{}.{}",
                portrait.hash, portrait.format
            )
        })
        .unwrap_or_default();

    let top_tracks = page
        .top_tracks
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, track)| map_track(index, track))
        .collect();

    // Releases: server-driven bucketing (see `releases::map_releases` for the
    // full rationale — D3 faithfulness, label collection, section ordering).
    let (release_sections, labels) = map_releases(page.releases);

    let similar_artists = map_similar_artists(page.similar_artists);

    // Curated playlists featuring this artist (the /artist/page `playlists`
    // section) — rendered as a main-column carousel above the "Other" block.
    let playlists = map_playlists(page.playlists);

    // "Novedad más reciente" — the single latest-release highlight.
    // Drop a blocked "Latest release" at the SOURCE so has_last_release is
    // false (section hidden) and no stale cover job is queued.
    let last_release = page
        .last_release
        .map(crate::artist::track_map::map_release)
        .filter(|c| !crate::artist_blacklist::card_blacklisted(&c.id, &c.artist_id));

    // "Appears On" — tracks where the artist guests (tracks_appears_on).
    // These are TRACKS, not albums; rendered as a flat track section.
    let appears_on = page
        .tracks_appears_on
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, track)| map_track(index, track))
        .collect();

    ArtistData {
        name,
        bio,
        bio_short,
        bio_truncated,
        bio_source,
        artwork_url,
        top_tracks,
        last_release,
        appears_on,
        release_sections,
        labels,
        similar_artists,
        playlists,
    }
}

/// Truncate text at the last word boundary within `max` characters,
/// appending an ellipsis. Returns the text unchanged when it already
/// fits.
pub(crate) fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    format!("{}…", truncated[..cut].trim_end())
}
