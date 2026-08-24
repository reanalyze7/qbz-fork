use crate::*;

/// Copy a Qobuz playlist (by id) into the user's own playlists: fetch the
/// source, create a new owned playlist, add every track, carry the cover over
/// and mark the SOURCE id copied. Shared by the playlist header AND the card
/// menus (Discover / Search / Favorites / Library All). `is_open` = this id is
/// the currently-open detail, so its PlaylistState.is-copied flips too.
pub(crate) fn playlist_copy_by_id(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    pid: u64,
    is_open: bool,
) {
    handle.spawn(async move {
        let source = match runtime.core().get_playlist(pid).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("[qbz-slint] copy playlist {pid}: get source failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to copy playlist"));
                return;
            }
        };
        let track_ids: Vec<u64> = source
            .tracks
            .as_ref()
            .map(|t| t.items.iter().map(|track| track.id).collect())
            .unwrap_or_default();
        if track_ids.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("Playlist has no tracks to copy"));
            return;
        }
        let attribution = format!(
            "\n\n---\nOriginally curated by {} on Qobuz",
            source.owner.name
        );
        let new_description = match source.description {
            Some(ref d) if !d.is_empty() => Some(format!("{d}{attribution}")),
            _ => Some(attribution.trim_start().to_string()),
        };
        let new_playlist = match runtime
            .core()
            .create_playlist(&source.name, new_description.as_deref(), false)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log::error!("[qbz-slint] copy playlist {pid}: create failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Failed to copy playlist"));
                return;
            }
        };
        if let Err(e) = runtime
            .core()
            .add_tracks_to_playlist(new_playlist.id, &track_ids)
            .await
        {
            log::error!("[qbz-slint] copy playlist {pid}: add tracks failed: {e}");
        }
        if let Some(img) = source.images.as_ref().and_then(|i| i.first()).cloned() {
            let new_id = new_playlist.id;
            let _ = tokio::task::spawn_blocking(move || {
                crate::library_db::with_db(|db| {
                    db.update_playlist_artwork(new_id, Some(img.as_str()))
                });
            })
            .await;
        }
        let _ = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| db.mark_playlist_copied(pid));
        })
        .await;
        let _ = weak.upgrade_in_event_loop(move |w| {
            if is_open {
                w.global::<PlaylistState>().set_is_copied(true);
            }
            crate::toast::success(&w, qbz_i18n::t("Copied to your library"));
        });
        crate::playback::refresh_sidebar(true);
    });
}

/// Follow / unfollow a Qobuz playlist (by id). `is_open` = this id is the
/// currently-open detail (only then do we flip PlaylistState optimistically +
/// revert on error). Shared by the header follow toggle and the card overlay.
pub(crate) fn playlist_set_follow_by_id(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    pid: u64,
    follow: bool,
    is_open: bool,
) {
    if is_open {
        let weak_opt = weak.clone();
        let _ = weak_opt.upgrade_in_event_loop(move |w| {
            w.global::<PlaylistState>().set_is_following(follow);
        });
    }
    handle.spawn(async move {
        let res = if follow {
            runtime.core().subscribe_playlist(pid).await
        } else {
            runtime.core().unsubscribe_playlist(pid).await
        };
        if let Err(e) = res {
            log::error!("[qbz-slint] playlist {pid} follow={follow} failed: {e}");
            if is_open {
                let _ = weak.upgrade_in_event_loop(move |w| {
                    w.global::<PlaylistState>().set_is_following(!follow);
                });
            }
        } else {
            // Live-flip the follow chip on every visible playlist card.
            let ids = pid.to_string();
            let _ = weak.upgrade_in_event_loop(move |w| {
                set_playlist_row_following(&w, &ids, follow);
            });
            crate::playback::refresh_sidebar(true);
        }
    });
}

