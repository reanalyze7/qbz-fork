//! `custom_artwork_path` read/write through `qbz_mixtape::repo`.

/// Read the stored `custom_artwork_path` for a collection (used to delete the
/// previous file after a new one persists). Runs synchronously via `with_db`.
pub(super) fn get_prev_path(id: &str) -> Option<String> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            qbz_mixtape::repo::get_custom_artwork(conn, id).unwrap_or(None)
        }))
    })
    .flatten()
}

/// Persist `path` (or clear with None) into `custom_artwork_path`. Returns true
/// on success. Runs synchronously via `with_db`.
pub(super) fn set_custom_artwork(id: &str, path: Option<&str>) -> bool {
    let id = id.to_string();
    let path = path.map(|p| p.to_string());
    crate::library_db::with_db(move |db| {
        db.with_connection(|conn| {
            qbz_mixtape::repo::set_custom_artwork(conn, &id, path.as_deref())
        })
        .map_err(|e| {
            qbz_library::LibraryError::Database(format!("set_custom_artwork failed: {e}"))
        })
    })
    .is_some()
}
