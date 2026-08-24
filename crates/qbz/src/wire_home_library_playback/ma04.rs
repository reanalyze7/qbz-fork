use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch04(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let _image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
                ("album", "add-to-mixtape") => {
                    // The cassette button on the album header. Local albums
                    // build the payload
                    // from AlbumState + the loaded tracks; Qobuz albums resolve
                    // via get_album (the proven fail-safe resolver).
                    let Some(w) = weak.upgrade() else { return };
                    let st = w.global::<AlbumState>();
                    if st.get_is_local() {
                        let item = myqbz_add::AddItem {
                            item_type: "album".into(),
                            source: "local".into(),
                            source_item_id: st.get_id().to_string(),
                            title: st.get_title().to_string(),
                            subtitle: {
                                let a = st.get_artist().to_string();
                                (!a.is_empty()).then_some(a)
                            },
                            artwork_url: None, // local albums omit artwork_url (1:1 PSD)
                            year: None,
                            track_count: {
                                use slint::Model;
                                let n = st.get_tracks().row_count();
                                (n > 0).then_some(n as i32)
                            },
                        };
                        open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
                    } else {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        let album_id = id.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_album(&album_id).await {
                                Ok(album) => {
                                    let artwork_url = album
                                        .image
                                        .thumbnail
                                        .clone()
                                        .or_else(|| album.image.small.clone());
                                    let year = album
                                        .release_date_original
                                        .as_deref()
                                        .and_then(|d| d.get(0..4))
                                        .and_then(|y| y.parse::<i32>().ok());
                                    let track_count = album
                                        .tracks_count
                                        .or(album.track_count)
                                        .map(|n| n as i32);
                                    myqbz_add::AddItem {
                                        item_type: "album".into(),
                                        source: "qobuz".into(),
                                        source_item_id: album.id.clone(),
                                        title: album.title.clone(),
                                        subtitle: {
                                            let a = album.artist.name.clone();
                                            (!a.is_empty()).then_some(a)
                                        },
                                        artwork_url,
                                        year,
                                        track_count,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_album {album_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
        _ => {}
    }
}
