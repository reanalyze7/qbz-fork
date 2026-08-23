//! DB read path — thin wrappers over `qbz_mixtape::repo`.

use qbz_models::mixtape::{CollectionKind, MixtapeCollection};

/// List collections of the given kind filter from the per-user library.db.
/// `None` returns all kinds. Items come hydrated (the repo loads them) so
/// counts + mosaic artwork are accurate. Returns an empty Vec when the DB is
/// unavailable (logged by `with_db`).
pub fn list_collections(kind: Option<CollectionKind>) -> Vec<MixtapeCollection> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            qbz_mixtape::repo::list_collections(conn, kind).unwrap_or_else(|e| {
                log::warn!("[qbz-slint] myqbz list_collections failed: {e}");
                Vec::new()
            })
        }))
    })
    .unwrap_or_default()
}

/// Create a new manual collection of `kind` named `name` in the per-user
/// library.db and return the created row (mirrors the store's `createCollection`
/// → `v2_create_mixtape_collection`). The repo shifts existing positions down
/// so the new row lands at the TOP of its kind (position 0). `source_type` is
/// always `manual` and `source_ref`/`description` are null — artist_collections
/// are created via the Discography Builder, not this path. Runs on a blocking
/// thread; returns None if the DB is unavailable or the insert fails (logged).
pub fn create_collection(kind: CollectionKind, name: &str) -> Option<MixtapeCollection> {
    // Cap at 80 chars (Tauri's `<input maxlength="80">`), counting Unicode
    // scalar values to match HTML maxlength semantics.
    let name: String = name.chars().take(80).collect();
    crate::library_db::with_db(move |db| {
        db.with_connection(|conn| {
            qbz_mixtape::repo::create_collection(
                conn,
                kind,
                &name,
                None,
                qbz_models::mixtape::CollectionSourceType::Manual,
                None,
            )
        })
        .map_err(|e| {
            qbz_library::LibraryError::Database(format!("myqbz create_collection failed: {e}"))
        })
    })
}

/// Parse the modal's `kind` string ("mixtape" | "collection") into a
/// `CollectionKind`. `artist_collection` is intentionally NOT creatable here
/// (Discography Builder only); unknown values default to `Mixtape`.
pub fn kind_from_str(s: &str) -> CollectionKind {
    match s {
        "collection" => CollectionKind::Collection,
        "artist_collection" => CollectionKind::ArtistCollection,
        _ => CollectionKind::Mixtape,
    }
}
