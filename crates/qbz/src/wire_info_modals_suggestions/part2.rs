use crate::*;

pub(crate) fn wire_info_modals_suggestions_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Label releases sub-view toolbar — sort / Hi-Res filter /
    // group-by-artist / search. The markup updates the bound LabelState
    // property first; each callback just re-derives the rendered list
    // (local filter over the loaded catalog).
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_sort(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_hires(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_group(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_search(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }


    // Immersive Suggestions panel actions (Checkpoint D — split-panel == 2).
    {
        // load(track-id) — entry + now-playing-change refresh. Reads the
        // artist-id + title off NowPlayingState (the panel only has the track
        // id) and kicks the live artist load (mirror of navigate_award).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SuggestionsActions>()
            .on_load(move |track_id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let np = w.global::<NowPlayingState>();
                let artist_id = np.get_artist_id().to_string();
                let track_id = track_id.to_string();
                let track_name = np.get_title().to_string();
                // Dedup: skip a reload when the panel already shows this artist
                // for this seed track (the changed-watcher can refire on
                // unrelated NowPlayingState churn).
                let ss = w.global::<SuggestionsState>();
                if ss.get_artist_id().as_str() == artist_id
                    && ss.get_seed_track_id().as_str() == track_id
                    && !track_id.is_empty()
                {
                    return;
                }
                navigate_suggestions(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    track_id,
                    track_name,
                );
            });
    }
    {
        // play / queue / play-next a curated artist playlist by id — reuse the
        // existing playback seams (same paths the playlist cards use).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::play_playlist(runtime.clone(), weak.clone(), handle.clone(), id);
            });
    }
}
