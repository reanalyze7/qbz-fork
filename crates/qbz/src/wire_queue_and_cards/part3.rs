use crate::*;

pub(crate) fn wire_queue_and_cards_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Per-disc "Disc N" header ⋯ menu (Qobuz album) — each action is scoped to
    // that disc's tracks only, resolved from the album's stashed raw catalog
    // tracks. Reuses the SAME queue ops as the album-header buttons (play_tracks
    // / play_album_shuffled's xorshift / enqueue_tracks), just over the disc
    // subset rather than the whole album.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<AlbumActions>()
            .on_disc_action(move |disc, action| {
                let mut tracks = album::disc_play_tracks(disc);
                if tracks.is_empty() {
                    return;
                }
                match action.as_str() {
                    "play" => {
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                    "shuffle" => {
                        // Same SystemTime-seeded xorshift Fisher-Yates as the
                        // album-header Shuffle (playback::play_album_shuffled),
                        // applied to the disc subset before play_tracks.
                        let mut seed = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(1)
                            | 1;
                        for i in (1..tracks.len()).rev() {
                            seed ^= seed << 13;
                            seed ^= seed >> 7;
                            seed ^= seed << 17;
                            let j = (seed % (i as u64 + 1)) as usize;
                            tracks.swap(i, j);
                        }
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                    "queue" => {
                        playback::enqueue_tracks(
                            runtime.clone(),
                            handle.clone(),
                            tracks,
                            false,
                        );
                    }
                    "play-next" => {
                        playback::enqueue_tracks(
                            runtime.clone(),
                            handle.clone(),
                            tracks,
                            true,
                        );
                    }
                    other => {
                        log::warn!("[qbz-slint] album disc-action: unknown action {other}");
                    }
                }
            });
    }

    // Album external-database links (Last.fm / Discogs / MusicBrainz) — open
    // the prebuilt url (AlbumState.*-url) in the system browser. Mirrors the
    // ArtworkActions open-in-browser handler.
    window
        .global::<AlbumActions>()
        .on_open_external_link(|url| {
            if url.is_empty() {
                return;
            }
            if let Err(e) = open::that(url.as_str()) {
                log::error!("[qbz-slint] album external link open failed: {e}");
            }
        });

    // Booklet reader removed — the album booklet button now downloads the PDF
    // (booklet::download_booklet via the ("album","booklet") media action). The
    // BookletActions/BookletState globals + AlbumBookletModal.slint are unused
    // now (left in place; remove in a UI cleanup pass that recompiles qbz-ui).

    // Artist in-page search — client-side filter over Popular Tracks
    // and every release-section album.
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    artist::filter_artist(&w, query.as_str());
                }
            });
    }
}
