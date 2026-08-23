/// Fetch up to `limit` local-library tracks matching `query`, off the UI thread
/// (the rusqlite read is sync + blocking, so it runs inside `spawn_blocking`).
/// Returns an empty Vec when the module is gated off, the library is empty, or no
/// row matches — the caller then simply adds no local section. `gated` makes the
/// fetch respect the intelligent-search toggle (main cortinilla); the immersive
/// search passes `false` since it has its own enable.
///
/// Independent of the Qobuz search: callers `tokio::join!` this with
/// `core.search_all`, so an offline / slow Qobuz never blocks the on-device
/// results (and vice-versa).
pub async fn load_cortinilla_local(
    query: &str,
    limit: u64,
    gated: bool,
) -> Vec<qbz_library::LocalTrack> {
    // Gate: the MAIN cortinilla only touches the DB when the intelligent-search
    // module is enabled (`gated = true`). The immersive search is governed by its
    // own "search action" enable instead, so it passes `gated = false`.
    if gated && !crate::search_service::is_enabled() {
        log::info!("[qbz-slint] cortinilla local: gated off (intelligent-search disabled)");
        return Vec::new();
    }
    let q = query.trim().to_string();
    if q.chars().count() < 2 {
        return Vec::new();
    }
    let exclude_network = crate::local_library::exclude_network_folders_now();
    let q_log = q.clone();
    let rows: Vec<qbz_library::LocalTrack> = tokio::task::spawn_blocking(move || {
        crate::library_db::with_db(|db| {
            // "default" sort: the cortinilla has no sort control; keep the
            // historical album-grouped order.
            db.search_with_filter_page(q.trim(), 0, limit, true, exclude_network, "default")
        })
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();
    log::debug!(
        "[qbz-slint] cortinilla local: query={q_log:?} limit={limit} -> {} rows",
        rows.len()
    );
    rows
}
