use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_search_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Cortinilla: a row was activated (click or Enter on a highlight). Resolve
    // the flat index against the controller snapshot, then dispatch to the SAME
    // nav/play seams the results page uses.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_row_clicked(move |flat_index| {
                let Some(w) = weak.upgrade() else { return };
                // Resolve flat_index -> the concrete row from the snapshot.
                let row = LAST_CORTINILLA.with(|c| {
                    let snap = c.borrow();
                    let data = snap.as_ref()?;
                    if let Some(top) = &data.top {
                        if top.flat_index as i32 == flat_index {
                            return Some(top.clone());
                        }
                    }
                    data.sections
                        .iter()
                        .flat_map(|s| s.rows.iter())
                        .find(|r| r.flat_index as i32 == flat_index)
                        .cloned()
                });
                let Some(row) = row else { return };

                // Capture the live cortinilla query BEFORE dismissing so the
                // ranking feedback (Capa B) is keyed off the query that produced
                // this row.
                let cort_query = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .to_string();

                // Dismiss the dropdown before acting AND clear the header input —
                // once the user activates a row, leftover text would otherwise
                // re-invoke the cortinilla when focus bounces back to the field.
                {
                    let st = w.global::<SearchState>();
                    st.set_cortinilla_open(false);
                    st.set_header_search_text("".into());
                }

                // Feed Capa B: a clicked QOBUZ row is an interaction with the
                // search-surfaced entity. action = Play for tracks (they play on
                // click), Open for album/artist/playlist (they navigate). LOCAL
                // rows are intentionally NOT recorded — local entities use a
                // different id space (D4) and are skipped in v1. record() no-ops
                // when the module is disabled, so the unconditional call is safe.
                if row.source != "local" {
                    let action = if row.kind == "track" {
                        crate::search_service::InteractionAction::Play
                    } else {
                        crate::search_service::InteractionAction::Open
                    };
                    crate::search_service::record(&cort_query, &row.kind, &row.id, action);
                }

                if row.source == "local" {
                    // On-device rows route by kind (the "links go to LocalLibrary"
                    // requirement): a local ALBUM opens the LocalAlbum view by its
                    // group key; a local ARTIST opens the LocalLibrary Artists tab
                    // by NAME (local artists have no id); a local TRACK plays
                    // through the LOCAL seam.
                    match row.kind.as_str() {
                        "album" => {
                            // `row.id` is the album_group_key (a local album key).
                            let key = row.id.clone();
                            nav::record(nav::NavEntry::LocalAlbum(key.clone()));
                            navigate_local_album(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                key,
                            );
                            update_nav_flags(&w);
                        }
                        "artist" => {
                            // Local artists are keyed by NAME (`row.title`).
                            open_local_artist(
                                &runtime,
                                &weak,
                                &handle,
                                &image_cache,
                                row.title.clone(),
                            );
                        }
                        _ => {
                            // Track: play this on-device row + its siblings (so the
                            // queue continues down the list), starting at the
                            // clicked one. `row.id` is the library row id.
                            let tracks = LAST_CORTINILLA_LOCAL.with(|c| c.borrow().clone());
                            let start = tracks
                                .iter()
                                .position(|t| t.id.to_string() == row.id)
                                .unwrap_or(0);
                            if !tracks.is_empty() {
                                playback::play_local_tracks(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    tracks,
                                    start,
                                    false,
                                );
                            }
                        }
                    }
                    return;
                }

                match row.kind.as_str() {
                    "album" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Album(id.clone()));
                        navigate_album(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "artist" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Artist(id.clone()));
                        navigate_artist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "playlist" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Playlist(id.clone()));
                        navigate_playlist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "track" => {
                        // A clicked Qobuz track plays immediately (single-track
                        // queue), matching the results-row "play".
                        if let Ok(track_id) = row.id.parse::<u64>() {
                            playback::play_track_now(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                track_id,
                            );
                        }
                    }
                    _ => {}
                }
            });
    }
}
