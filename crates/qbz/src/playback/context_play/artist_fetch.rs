//! Fetching the artist page's Popular tracks and mapping them to
//! `QueueTrack`s.

use super::super::recent_blacklist::filter_blacklisted_queue;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Fetch the artist page and build the Popular-tracks play queue. Shared by
/// `play_artist_top_tracks` (start at 0) and `play_artist_top_from` (start at
/// a clicked track id). Returns None and toasts on failure / no top tracks.
pub(super) async fn fetch_artist_top_for_play(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    artist_id: &str,
) -> Option<Vec<QueueTrack>> {
    let id: u64 = match artist_id.parse() {
        Ok(id) => id,
        Err(_) => {
            log::warn!("[qbz-slint] play-top: invalid artist id {artist_id}");
            return None;
        }
    };
    let page = match runtime.core().get_artist_page(id, None).await {
        Ok(page) => page,
        Err(e) => {
            log::error!("[qbz-slint] play-top: get_artist_page failed: {e}");
            crate::toast::error_weak(weak, qbz_i18n::t("Couldn't load this artist"));
            return None;
        }
    };
    let artist_name = page.name.display.clone();
    let raw: Vec<QueueTrack> = page
        .top_tracks
        .unwrap_or_default()
        .into_iter()
        .map(|track| make_top_track_queue(track, &artist_name))
        .collect();
    if raw.is_empty() {
        log::warn!("[qbz-slint] play-top: artist {artist_id} has no top tracks");
        crate::toast::error_weak(weak, qbz_i18n::t("No top tracks available for this artist"));
        return None;
    }
    // Drop blacklisted top tracks (a featured/blacklisted performer can appear
    // in another artist's Popular list). Silent early-return when 0 remain.
    let tracks = filter_blacklisted_queue(raw);
    if tracks.is_empty() {
        log::warn!("[qbz-slint] play-top: artist {artist_id} fully filtered by blacklist");
        return None;
    }
    Some(tracks)
}

/// Build a QueueTrack from a /artist/page top_tracks entry. The page
/// response carries a thinner audio_info than /album/get tracks; fall
/// back to sensible defaults when fields are absent.
pub(super) fn make_top_track_queue(
    track: qbz_models::PageArtistTrack,
    artist_fallback: &str,
) -> QueueTrack {
    let audio = track.audio_info.as_ref();
    let album_id = track.album.as_ref().map(|a| a.id.clone());
    let album_title = track.album.as_ref().map(|a| a.title.clone()).unwrap_or_default();
    let artwork_url = track
        .album
        .as_ref()
        .and_then(|a| a.image.as_ref())
        .and_then(|img| img.best().cloned());
    let artist_name = track
        .artist
        .as_ref()
        .map(|a| a.name.display.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| artist_fallback.to_string());
    let artist_id = track.artist.as_ref().map(|a| a.id);
    let hires = audio.and_then(|a| a.maximum_bit_depth).map(|b| b > 16).unwrap_or(false);
    QueueTrack {
        id: track.id,
        title: track.title,
        version: track.version,
        artist: artist_name,
        album: album_title,
        album_version: None,
        duration_secs: track.duration.unwrap_or(0) as u64,
        artwork_url,
        hires,
        bit_depth: audio.and_then(|a| a.maximum_bit_depth),
        sample_rate: audio.and_then(|a| a.maximum_sampling_rate),
        is_local: false,
        album_id: album_id.clone(),
        artist_id,
        streamable: track.rights.as_ref().and_then(|r| r.streamable).unwrap_or(true),
        source: Some("qobuz".to_string()),
        parental_warning: track.parental_warning.unwrap_or(false),
        source_item_id_hint: album_id,
        // Stamped "artist" by the artist play paths; unset here.
        context_kind: None,
        context_id: None,
    }
}
