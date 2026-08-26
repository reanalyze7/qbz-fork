//! Slint-state glue: build `SuggestionCard`s from the payload and push into
//! `SuggestionsState`.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppWindow, SuggestionCard, SuggestionsState, TrackItem};

use super::types::{PlaylistCard, SuggestionsPayload};

/// Build a `SuggestionCard` for a playlist (book collage).
fn playlist_to_card(card: &PlaylistCard) -> SuggestionCard {
    SuggestionCard {
        kind: "playlist".into(),
        title: card.name.clone().into(),
        subtitle: qbz_i18n::tf(
            "{} track",
            "{} tracks",
            card.track_count as i64,
            &[&card.track_count.to_string()],
        )
        .into(),
        cover_urls: ModelRc::new(VecModel::from(
            card.cover_urls
                .iter()
                .map(|s| slint::SharedString::from(s.as_str()))
                .collect::<Vec<_>>(),
        )),
        cover0: slint::Image::default(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        playlist_id: card.id.clone().into(),
        seed_track_id: "".into(),
        seed_track_name: "".into(),
        seed_artist_id: "".into(),
        badge: "qobuz".into(),
        loading: false,
    }
}

/// Build the seed "Song Radio" card (diamond collage).
fn radio_card(payload: &SuggestionsPayload) -> SuggestionCard {
    SuggestionCard {
        kind: "radio".into(),
        title: qbz_i18n::t("Song Radio").into(),
        subtitle: payload.seed_track_name.clone().into(),
        cover_urls: ModelRc::new(VecModel::from(
            payload
                .radio_cover_urls
                .iter()
                .map(|s| slint::SharedString::from(s.as_str()))
                .collect::<Vec<_>>(),
        )),
        cover0: slint::Image::default(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        playlist_id: "".into(),
        seed_track_id: payload.seed_track_id.clone().into(),
        seed_track_name: payload.seed_track_name.clone().into(),
        seed_artist_id: payload.seed_artist_id.clone().into(),
        badge: "qbz".into(),
        loading: false,
    }
}

/// Apply the assembled suggestions to `SuggestionsState`. Runs on the event loop.
pub fn apply_suggestions(window: &AppWindow, payload: SuggestionsPayload) {
    let mut cards: Vec<SuggestionCard> =
        payload.playlist_cards.iter().map(playlist_to_card).collect();
    // The radio card always trails the playlist cards (Tauri order).
    if !payload.seed_track_id.is_empty() {
        cards.push(radio_card(&payload));
    }
    let tracks: Vec<TrackItem> = payload
        .rec_tracks
        .iter()
        .map(crate::playlist::to_item)
        .collect();

    let state = window.global::<SuggestionsState>();
    state.set_artist_id(payload.artist_id.into());
    state.set_seed_track_id(payload.seed_track_id.into());
    state.set_cards(ModelRc::new(VecModel::from(cards)));
    state.set_tracks(ModelRc::new(VecModel::from(tracks)));
    state.set_error(if payload.error { "error".into() } else { "".into() });
    state.set_loading(false);
}

/// Clear the suggestions state before a (re)load. Runs on the event loop.
pub fn reset_suggestions(window: &AppWindow) {
    let state = window.global::<SuggestionsState>();
    state.set_cards(ModelRc::new(VecModel::from(Vec::<SuggestionCard>::new())));
    state.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
    state.set_error("".into());
    state.set_loading(true);
}
