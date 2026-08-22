//! Fetch complete metadata for a track from the Qobuz API, deriving
//! album_artist/genre/label/year/artwork_url from the track + album
//! responses.

use super::model::CompleteTrackMetadata;

/// Fetch complete metadata for a track from Qobuz API
pub async fn fetch_complete_metadata(
    track_id: u64,
    qobuz_client: &qbz_qobuz::QobuzClient,
) -> Result<CompleteTrackMetadata, String> {
    log::info!("Fetching complete metadata for track {}", track_id);

    let track = qobuz_client
        .get_track(track_id)
        .await
        .map_err(|e| format!("Failed to fetch track: {}", e))?;

    let album = if let Some(album_obj) = &track.album {
        qobuz_client.get_album(&album_obj.id).await.ok()
    } else {
        None
    };

    let album_artist = album
        .as_ref()
        .map(|a| a.artist.name.clone())
        .or_else(|| track.performer.as_ref().map(|p| p.name.clone()));

    let genre = album
        .as_ref()
        .and_then(|a| a.genre.as_ref())
        .map(|g| g.name.clone());

    let label = album
        .as_ref()
        .and_then(|a| a.label.as_ref())
        .map(|l| l.name.clone());

    let year = album
        .as_ref()
        .and_then(|a| a.release_date_original.as_ref())
        .and_then(|date_str| {
            // Parse YYYY-MM-DD or YYYY format
            date_str
                .split('-')
                .next()
                .and_then(|year_str| year_str.parse::<u32>().ok())
        });

    let artwork_url = album
        .as_ref()
        .and_then(|a| a.image.large.clone())
        .or_else(|| track.album.as_ref().and_then(|a| a.image.large.clone()));

    Ok(CompleteTrackMetadata {
        track_id,
        title: track.title,
        artist: track
            .performer
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default(),
        album: track
            .album
            .as_ref()
            .map(|a| a.title.clone())
            .unwrap_or_default(),
        album_artist,
        track_number: Some(track.track_number),
        disc_number: track.media_number,
        year,
        genre,
        isrc: track.isrc.clone(),
        label,
        copyright: None, // Qobuz API doesn't provide copyright in album model
        composer: None,  // Qobuz API doesn't provide composer in track model
        duration_secs: track.duration as u64,
        artwork_url,
    })
}
