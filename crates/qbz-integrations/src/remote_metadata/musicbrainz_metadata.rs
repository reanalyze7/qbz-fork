//! Converter for full MusicBrainz release responses (`crate::musicbrainz` API
//! client types, not to be confused with this `remote_metadata` module) into
//! the unified `RemoteAlbumMetadata` DTO.

use super::models::{RemoteAlbumMetadata, RemoteProvider, RemoteTrackMetadata};

/// Convert MusicBrainz full release to unified metadata DTO
pub fn musicbrainz_full_to_metadata(
    release: &crate::musicbrainz::ReleaseFullResponse,
) -> RemoteAlbumMetadata {
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

    // Extract year from date
    let year = release
        .date
        .as_ref()
        .and_then(|d| d.split('-').next().and_then(|y| y.parse::<u16>().ok()));

    // Extract genres from tags (sorted by count, take top 5)
    let genres: Vec<String> = release
        .tags
        .as_ref()
        .map(|tags| {
            let mut sorted: Vec<_> = tags.iter().collect();
            sorted.sort_by(|a, b| b.count.cmp(&a.count));
            sorted.iter().take(5).map(|t| t.name.clone()).collect()
        })
        .unwrap_or_default();

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

    // Count discs
    let disc_count = release.media.as_ref().map(|m| m.len() as u8).unwrap_or(1);

    // Convert tracks
    let tracks: Vec<RemoteTrackMetadata> = release
        .media
        .as_ref()
        .map(|media| {
            let mut all_tracks = Vec::new();
            for medium in media {
                if let Some(tracks) = &medium.tracks {
                    for track in tracks {
                        all_tracks.push(RemoteTrackMetadata {
                            disc_number: medium.position.unwrap_or(1),
                            track_number: track.position.unwrap_or(1),
                            title: track
                                .title
                                .clone()
                                .or_else(|| track.recording.as_ref().and_then(|r| r.title.clone()))
                                .unwrap_or_default(),
                            duration_ms: track.length.map(|l| l as u32).or_else(|| {
                                track
                                    .recording
                                    .as_ref()
                                    .and_then(|r| r.length.map(|l| l as u32))
                            }),
                        });
                    }
                }
            }
            all_tracks
        })
        .unwrap_or_default();

    RemoteAlbumMetadata {
        provider: RemoteProvider::MusicBrainz,
        provider_id: release.id.clone(),
        title: release.title.clone(),
        artist,
        year,
        genres,
        label,
        catalog_number,
        country: release.country.clone(),
        barcode: release.barcode.clone(),
        tracks,
        disc_count,
        source_url: Some(format!("https://musicbrainz.org/release/{}", release.id)),
    }
}
