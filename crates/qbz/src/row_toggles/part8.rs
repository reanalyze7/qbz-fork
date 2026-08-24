use crate::*;

/// Open the global "Add to Mixtape/Collection" picker for `items` (mirrors
/// Tauri's `openAddToMixtape`). Hops onto the event loop to show the modal,
/// then loads the picker rows (kind-restricted + recency-sorted +
/// `item_exists`-resolved) on a blocking worker. Empty `items` is a no-op
/// (the controller guards too). Callable from any thread.
pub(crate) fn open_add_to_mixtape(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    items: Vec<myqbz_add::AddItem>,
) {
    if items.is_empty() {
        return;
    }
    let restrict = items.iter().any(|it| it.item_type != "album");
    let items_for_open = items.clone();
    let _ = weak.upgrade_in_event_loop(move |w| {
        myqbz_add::open(&w, items_for_open);
    });
    handle.spawn(async move {
        let rows =
            tokio::task::spawn_blocking(move || myqbz_add::load_rows(restrict, &items))
                .await
                .unwrap_or_default();
        let _ = weak.upgrade_in_event_loop(move |w| {
            myqbz_add::apply_rows(&w, rows);
        });
    });
}

/// Build "Add to Mixtape" track payloads from Qobuz `Track` objects (the
/// Favorites, Label and Mix bulk bars — issue #446). `item_type: "track"`
/// auto-restricts the picker to mixtapes; title/artist/artwork enrich the
/// picker rows. Rows without a numeric id are dropped (they cannot be added).
pub(crate) fn mixtape_items_from_qobuz_tracks(tracks: &[qbz_models::Track]) -> Vec<myqbz_add::AddItem> {
    tracks
        .iter()
        .filter(|t| t.id != 0)
        .map(|t| {
            let subtitle = t
                .performer
                .as_ref()
                .map(|a| a.name.clone())
                .filter(|s| !s.is_empty());
            let artwork_url = t
                .album
                .as_ref()
                .and_then(|a| a.image.best().cloned())
                .filter(|u| !u.is_empty());
            myqbz_add::AddItem {
                item_type: "track".into(),
                source: "qobuz".into(),
                source_item_id: t.id.to_string(),
                title: t.title.clone(),
                subtitle,
                artwork_url,
                year: None,
                track_count: None,
            }
        })
        .collect()
}

/// Build "Add to Mixtape" track payloads from the Artist Popular-Tracks
/// selection (issue #446). The artist view only exposes `selected_ids`, so we
/// read the selected `TrackItem` rows directly for their display fields.
pub(crate) fn mixtape_items_from_artist_selection(window: &AppWindow) -> Vec<myqbz_add::AddItem> {
    use slint::Model;
    let model = window.global::<ArtistState>().get_top_tracks();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .filter(|t| t.selected && t.id.as_str().parse::<u64>().is_ok())
        .map(|t| {
            let subtitle = (!t.artist.is_empty()).then(|| t.artist.to_string());
            let artwork_url = (!t.artwork_url.is_empty()).then(|| t.artwork_url.to_string());
            myqbz_add::AddItem {
                item_type: "track".into(),
                source: "qobuz".into(),
                source_item_id: t.id.to_string(),
                title: t.title.to_string(),
                subtitle,
                artwork_url,
                year: None,
                track_count: None,
            }
        })
        .collect()
}

