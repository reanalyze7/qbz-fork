//! Converters for the Discogs API client types (`crate::discogs`, not to be
//! confused with this `remote_metadata` module) into the unified
//! `RemoteAlbumSearchResult` / `RemoteAlbumMetadata` DTOs.

use super::discogs_parse::{parse_discogs_duration, parse_discogs_position};
use super::models::{RemoteAlbumMetadata, RemoteAlbumSearchResult, RemoteProvider, RemoteTrackMetadata};

/// Convert Discogs extended search result to unified DTO
pub fn discogs_extended_to_search_result(
    result: &crate::discogs::DiscogsSearchResultExtended,
) -> RemoteAlbumSearchResult {
    // Discogs title format is usually "Artist - Album"
    let (artist, title) = if let Some(pos) = result.title.find(" - ") {
        let (a, t) = result.title.split_at(pos);
        (a.to_string(), t.trim_start_matches(" - ").to_string())
    } else {
        ("Unknown Artist".to_string(), result.title.clone())
    };

    // Parse year from string
    let year = result.year.as_ref().and_then(|y| y.parse::<u16>().ok());

    // Get first label
    let label = result.label.as_ref().and_then(|l| l.first().cloned());

    // Get format as string
    let format = result.format.as_ref().map(|f| f.join(", "));

    RemoteAlbumSearchResult {
        provider: RemoteProvider::Discogs,
        provider_id: result.id.to_string(),
        title,
        artist,
        year,
        track_count: None,
        country: result.country.clone(),
        label,
        catalog_number: result.catno.clone(),
        confidence: None,
        format,
    }
}

/// Convert Discogs full release to unified metadata DTO
pub fn discogs_full_to_metadata(release: &crate::discogs::DiscogsReleaseMetadata) -> RemoteAlbumMetadata {
    // Combine artists with join phrases
    let artist = release
        .artists
        .as_ref()
        .map(|artists| {
            artists
                .iter()
                .map(|a| format!("{}{}", a.name.clone(), a.join.as_deref().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    // Combine genres and styles
    let genres: Vec<String> = {
        let mut combined = Vec::new();
        if let Some(g) = &release.genres {
            combined.extend(g.clone());
        }
        if let Some(s) = &release.styles {
            combined.extend(s.clone());
        }
        combined
    };

    // Get first label and catalog number
    let (label, catalog_number) = release
        .labels
        .as_ref()
        .and_then(|labels| labels.first())
        .map(|l| (Some(l.name.clone()), l.catno.clone()))
        .unwrap_or((None, None));

    // Convert tracklist
    let tracks: Vec<RemoteTrackMetadata> = release
        .tracklist
        .as_ref()
        .map(|tracklist| {
            tracklist
                .iter()
                .filter(|t| {
                    // Filter out headings (disc separators)
                    t.track_type.as_deref() != Some("heading")
                })
                .map(|t| {
                    let (disc_number, track_number) = parse_discogs_position(&t.position);
                    RemoteTrackMetadata {
                        disc_number,
                        track_number,
                        title: t.title.clone(),
                        duration_ms: t.duration.as_ref().and_then(|d| parse_discogs_duration(d)),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Count unique discs
    let disc_count = tracks.iter().map(|t| t.disc_number).max().unwrap_or(1);

    RemoteAlbumMetadata {
        provider: RemoteProvider::Discogs,
        provider_id: release.id.to_string(),
        title: release.title.clone(),
        artist,
        year: release.year.map(|y| y as u16),
        genres,
        label,
        catalog_number,
        country: release.country.clone(),
        barcode: None, // Discogs doesn't include barcode in release details
        tracks,
        disc_count,
        source_url: release.uri.clone(),
    }
}
