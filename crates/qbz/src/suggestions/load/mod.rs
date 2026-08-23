//! The async data-assembly pipeline: artist-detail fetch, then delegate to
//! `rec_tracks` and `playlist_cards` for the two data products.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::types::SuggestionsPayload;

mod playlist_cards;
mod rec_tracks;

/// Build the suggestions payload for `artist_id` + `current_track_id`. All
/// queries are live Qobuz artist calls (NEVER reco_store). On the top-level
/// artist-detail failure, returns an error payload (drives the panel's error
/// branch); individual playlist-cover fetch failures are tolerated.
pub async fn load_suggestions<A>(
    runtime: &Arc<AppRuntime<A>>,
    artist_id: u64,
    current_track_id: u64,
    seed_track_name: String,
) -> SuggestionsPayload
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let artist = match runtime.core().get_artist_detail(artist_id, None, None).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("[qbz-slint] suggestions get_artist_detail({artist_id}) failed: {e}");
            return SuggestionsPayload {
                artist_id: artist_id.to_string(),
                seed_track_id: current_track_id.to_string(),
                seed_track_name,
                seed_artist_id: artist_id.to_string(),
                playlist_cards: Vec::new(),
                rec_tracks: Vec::new(),
                radio_cover_urls: Vec::new(),
                error: true,
            };
        }
    };

    let rec = rec_tracks::build(runtime, artist_id, current_track_id, &artist).await;
    let playlist_cards = playlist_cards::build(runtime, &artist).await;

    SuggestionsPayload {
        artist_id: artist_id.to_string(),
        seed_track_id: current_track_id.to_string(),
        seed_track_name,
        seed_artist_id: artist_id.to_string(),
        playlist_cards,
        rec_tracks: rec.tracks,
        radio_cover_urls: rec.radio_cover_urls,
        error: false,
    }
}
