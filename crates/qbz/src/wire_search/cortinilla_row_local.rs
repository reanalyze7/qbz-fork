use crate::*;

// Cortinilla row-click dispatch for an on-device (`row.source == "local"`)
// row: local ALBUM opens the LocalAlbum view by group key, local ARTIST
// opens the LocalLibrary Artists tab by name, local TRACK plays through the
// LOCAL seam (with its siblings, so the queue continues down the list).
// Split out of the single `on_cortinilla_row_clicked` callback
// (wire_search_part7, part7.rs) to stay under the 130-line file cap.
pub(crate) fn handle_cortinilla_local_row(
    row: search::CortRow,
    w: &AppWindow,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
) {
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
                            update_nav_flags(w);
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
}
