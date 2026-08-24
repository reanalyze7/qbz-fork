use crate::*;

// Cortinilla row-click dispatch for a Qobuz row: album/artist/playlist
// navigate (+ history record), track plays immediately (single-track
// queue), matching the results-row "play". Split out of the single
// `on_cortinilla_row_clicked` callback (wire_search_part7, part7.rs) to
// stay under the 130-line file cap.
pub(crate) fn handle_cortinilla_qobuz_row(
    row: search::CortRow,
    w: &AppWindow,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
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
                        update_nav_flags(w);
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
                        update_nav_flags(w);
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
                        update_nav_flags(w);
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
}
