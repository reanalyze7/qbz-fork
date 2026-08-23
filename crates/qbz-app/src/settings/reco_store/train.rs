use std::collections::HashMap;

use super::now_ts;
use super::schema::RecoStore;
use super::train_entries::{build_album_entries, build_artist_entries, build_track_entries};
use super::train_weights::{decay_factor, event_weight, item_weight};
use super::types::TrainParams;

impl RecoStore {
    // ---- Scorer ----

    /// Recompute and persist recommendation scores from recent events.
    ///
    /// Faithful port of Tauri's `v2_reco_train_scores`
    /// (`src-tauri/src/commands_v2/library.rs:1771-1911`): same lookback window,
    /// same exponential half-life decay, the same event weights
    /// (play=1.0 / favorite=3.0 / playlist_add=1.2) and item weights
    /// (primary=1.0; non-primary album=0.7 / artist=0.5 / track=0.85 / other=0.6),
    /// the same top-N-per-type cap, and the same `(all, favorite) x (track,
    /// album, artist)` six `replace_scores` writes.
    pub fn train(&mut self, params: TrainParams) -> Result<(), String> {
        let now = now_ts();
        let since_ts = now.saturating_sub(params.lookback_days * 86_400);
        let events = self.get_events_since(since_ts, params.max_events)?;
        let half_life_days = params.half_life_days;

        let build_scores = |favorites_only: bool| {
            let mut tracks: HashMap<u64, f64> = HashMap::new();
            let mut albums: HashMap<String, f64> = HashMap::new();
            let mut artists: HashMap<u64, f64> = HashMap::new();

            for event in &events {
                if favorites_only && event.event_type != "favorite" {
                    continue;
                }
                let age_secs = (now - event.created_at).max(0);
                let base_weight =
                    event_weight(&event.event_type) * decay_factor(age_secs, half_life_days);

                if let Some(track_id) = event.track_id {
                    let weight = base_weight * item_weight("track", event.item_type == "track");
                    *tracks.entry(track_id).or_insert(0.0) += weight;
                }
                if let Some(album_id) = event.album_id.as_ref() {
                    let weight = base_weight * item_weight("album", event.item_type == "album");
                    *albums.entry(album_id.clone()).or_insert(0.0) += weight;
                }
                if let Some(artist_id) = event.artist_id {
                    let weight = base_weight * item_weight("artist", event.item_type == "artist");
                    *artists.entry(artist_id).or_insert(0.0) += weight;
                }
            }
            (tracks, albums, artists)
        };

        let max_per_type = params.max_per_type as usize;
        let (all_tracks, all_albums, all_artists) = build_scores(false);
        let (fav_tracks, fav_albums, fav_artists) = build_scores(true);

        self.replace_scores("all", "track", &build_track_entries(all_tracks, max_per_type))?;
        self.replace_scores("all", "album", &build_album_entries(all_albums, max_per_type))?;
        self.replace_scores("all", "artist", &build_artist_entries(all_artists, max_per_type))?;
        self.replace_scores(
            "favorite",
            "track",
            &build_track_entries(fav_tracks, max_per_type),
        )?;
        self.replace_scores(
            "favorite",
            "album",
            &build_album_entries(fav_albums, max_per_type),
        )?;
        self.replace_scores(
            "favorite",
            "artist",
            &build_artist_entries(fav_artists, max_per_type),
        )?;

        Ok(())
    }
}
