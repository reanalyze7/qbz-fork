//! Converter for MusicBrainz search results (`crate::musicbrainz` API client
//! types, not to be confused with this `remote_metadata` module) into the
//! unified `RemoteAlbumSearchResult` DTO.

use super::models::{RemoteAlbumSearchResult, RemoteProvider};

pub fn musicbrainz_release_to_search_result(
    release: &crate::musicbrainz::ReleaseResult,
) -> RemoteAlbumSearchResult {
    // Extract artist from artist-credit
    let artist = release
        .artist_credit
        .as_ref()
        .map(|credits| {
            credits
                .iter()
                .map(|c| {
                    format!(
                        "{}{}",
                        c.name.as_deref().unwrap_or(&c.artist.name),
                        c.joinphrase.as_deref().unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    // Extract year from date (YYYY or YYYY-MM-DD)
    let year = release
        .date
        .as_ref()
        .and_then(|d| d.split('-').next().and_then(|y| y.parse::<u16>().ok()));

    // Extract label and catalog number
    let (label, catalog_number) = release
        .label_info
        .as_ref()
        .and_then(|info| info.first())
        .map(|li| {
            (
                li.label.as_ref().map(|l| l.name.clone()),
                li.catalog_number.clone(),
            )
        })
        .unwrap_or((None, None));

    // Get track count - either from direct field or sum from media
    let track_count = release.track_count.or_else(|| {
        release
            .media
            .as_ref()
            .map(|media| media.iter().filter_map(|m| m.track_count).sum())
    });

    // Get format from first medium
    let format = release
        .media
        .as_ref()
        .and_then(|m| m.first())
        .and_then(|m| m.format.clone());

    RemoteAlbumSearchResult {
        provider: RemoteProvider::MusicBrainz,
        provider_id: release.id.clone(),
        title: release.title.clone(),
        artist,
        year,
        track_count,
        country: release.country.clone(),
        label,
        catalog_number,
        confidence: release.score.map(|s| s.min(100) as u8),
        format,
    }
}
