use crate::*;

// Local Library album detail reuses AlbumPageView. Route its play actions to
// local playback — guarded to the album view + is-local so Qobuz album/track
// play is untouched. Returns true when it fully handled the event (the
// caller must not fall through to the ordinary dispatch match).
pub(crate) fn media_action_local_album_redirect(
    weak: &slint::Weak<AppWindow>,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    handle: &tokio::runtime::Handle,
    kind: &str,
    id: &str,
    action: &str,
) -> bool {
    if action == "play" && (kind == "album" || kind == "track") {
        if let Some(w) = weak.upgrade() {
            let album_state = w.global::<AlbumState>();
            if matches!(w.global::<NavState>().get_view(), ContentView::Album)
                && album_state.get_is_local()
            {
                let album_id = album_state.get_id().to_string();
                let start = if kind == "track" {
                    id.parse::<i64>().ok()
                } else {
                    None
                };
                playback::play_local_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    album_id,
                    start,
                );
                return true;
            }
        }
    }
    false
}

// === Capa B feedback (intelligent search) ==================================
// Feed the ranking model from RESULTS-PAGE clicks, gated to the Search view
// inside `record_search_interaction` so the same global media-action handler
// fired from other views never mis-attributes. Only QOBUZ result clicks are
// recorded; the search results page never carries local rows (D1/D2), so no
// source check is needed.
//   - track play              -> Play
//   - album play               -> Play (an album-card play is still a play
//                                 interaction with the entity)
//   - album favorite (toggle) -> Favorite ONLY when transitioning to
//                                 favorited (the card heart arm is a toggle
//                                 since 2026-07; Favorite weight must only
//                                 ADD)
//   - artist follow (add)     -> Favorite (search artist cards show "Follow"
//                                 only when NOT following, so this action is
//                                 always an add)
//   - track favorite (toggle) -> Favorite ONLY when transitioning to
//                                 favorited (Favorite weight must only ADD —
//                                 never record on un-favorite)
pub(crate) fn media_action_record_search_feedback(
    weak: &slint::Weak<AppWindow>,
    kind: &str,
    id: &str,
    action: &str,
) {
    if let Some(w) = weak.upgrade() {
        use crate::search_service::InteractionAction;
        match (kind, action) {
            ("track", "play") | ("album", "play") => {
                record_search_interaction(&w, kind, id, InteractionAction::Play);
            }
            ("album", "favorite") => {
                if !crate::fav_cache::is_album_favorite(id) {
                    record_search_interaction(&w, kind, id, InteractionAction::Favorite);
                }
            }
            ("artist", "follow") => {
                record_search_interaction(&w, kind, id, InteractionAction::Favorite);
            }
            ("track", "favorite") => {
                if !crate::fav_cache::is_favorite(id) {
                    record_search_interaction(&w, kind, id, InteractionAction::Favorite);
                }
            }
            _ => {}
        }
    }
}
