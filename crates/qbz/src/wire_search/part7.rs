use crate::*;
use crate::navigate_album_artist::nav_statics::LAST_CORTINILLA;

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
                    handle_cortinilla_local_row(
                        row,
                        &w,
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        image_cache.clone(),
                    );
                    return;
                }

                handle_cortinilla_qobuz_row(
                    row,
                    &w,
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                );
            });
    }
}
