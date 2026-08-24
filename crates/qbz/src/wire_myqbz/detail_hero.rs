use crate::*;

use MyQbzDetailActions as Act;

/// Hero PLAY / SHUFFLE / DJ-mix CTA (+ the still-stubbed sync CTA) and the
/// DJ-mix modal's own slider / cancel / confirm actions.
pub(crate) fn wire_myqbz_detail_hero(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = image_cache;
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_play_all(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_play::play_all(runtime.clone(), weak.clone(), handle.clone(), id);
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<Act>().on_shuffle(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_play::shuffle(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id,
            );
        });
    }

    // --- Hero DJ-mix CTA — open the "Random queue" sampler modal --------
    // Resolves the collection in-order + counts unique tracks (the slider max),
    // then the modal samples + replace-plays on confirm (myqbz_mix).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_dj_mix(move || {
            let Some(w) = weak.upgrade() else { return };
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            if id.is_empty() {
                return;
            }
            myqbz_mix::open(runtime.clone(), weak.clone(), handle.clone(), id);
        });
    }

    // --- STILL-STUBBED hero CTA: discography sync -----------------------
    // Sync: artist_discography has NO sync impl (spec §8) — no-op stub (the
    // hero button is shown only for artist_collection for Tauri parity).
    {
        let weak = window.as_weak();
        window.global::<Act>().on_sync(move || {
            let id = weak
                .upgrade()
                .map(|w| w.global::<MyQbzDetailState>().get_id().to_string())
                .unwrap_or_default();
            log::info!("[qbz-slint] myqbz_detail sync({id}) — no discography sync impl (spec §8)");
        });
    }

    // --- DJ-mix modal actions (slider / cancel / confirm) ---------------
    {
        let weak = window.as_weak();
        window.global::<MyQbzMixActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_mix::close(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzMixActions>().on_set_index(move |index| {
            if let Some(w) = weak.upgrade() {
                myqbz_mix::apply_index(&w, index);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<MyQbzMixActions>().on_shuffle(move || {
            let Some(w) = weak.upgrade() else { return };
            let ms = w.global::<MyQbzMixState>();
            let id = w.global::<MyQbzDetailState>().get_id().to_string();
            let size = ms.get_selected_size();
            if id.is_empty() || size <= 0 {
                return;
            }
            myqbz_mix::shuffle(runtime.clone(), weak.clone(), handle.clone(), id, size);
        });
    }
}
