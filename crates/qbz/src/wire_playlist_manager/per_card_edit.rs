use crate::*;

// --- Per-card playlist actions: edit modal + add-to-mixtape --------------
pub(crate) fn wire_pm_per_card_edit(window: &AppWindow, tokio_rt: &tokio::runtime::Runtime) {
    {
        // Open the shared edit-playlist modal, prefilled from the card.
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_edit_playlist(move |id| {
                use slint::Model;
                let Some(w) = weak.upgrade() else { return };
                let model = w.global::<PlaylistManagerState>().get_playlists();
                let name = (0..model.row_count())
                    .filter_map(|i| model.row_data(i))
                    .find(|it| it.id == id)
                    .map(|it| it.name)
                    .unwrap_or_default();
                let es = w.global::<EditPlaylistState>();
                es.set_id(id);
                es.set_name(name);
                es.set_description("".into());
                es.set_open(true);
            });
    }
    {
        // Add a whole playlist to a Mixtape/Collection (callsite O). Builds the
        // `playlist` payload from the PM grid row (id / name / track count /
        // first cover); the owner subtitle isn't carried in the PM model, so it
        // is omitted (optional in the contract).
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_add_to_mixtape(move |id| {
                use slint::Model;
                let Some(w) = weak.upgrade() else { return };
                let model = w.global::<PlaylistManagerState>().get_playlists();
                let Some(row) = (0..model.row_count())
                    .filter_map(|i| model.row_data(i))
                    .find(|it| it.id == id)
                else {
                    return;
                };
                let artwork = row.url1.to_string();
                let item = myqbz_add::AddItem {
                    item_type: "playlist".into(),
                    source: "qobuz".into(),
                    source_item_id: id.to_string(),
                    title: row.name.to_string(),
                    subtitle: None,
                    artwork_url: (!artwork.is_empty()).then_some(artwork),
                    year: None,
                    track_count: (row.total_count > 0).then_some(row.total_count),
                };
                open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
            });
    }
}
