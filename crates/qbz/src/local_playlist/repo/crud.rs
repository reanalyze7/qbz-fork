use qbz_library::local_playlists as repo;

pub fn list_blocking() -> Vec<repo::LocalPlaylist> {
    crate::library_db::with_db(|db| Ok(db.with_connection(repo::list)))
        .and_then(|r| r.ok())
        .unwrap_or_default()
}

pub fn get_blocking(id: &str) -> Option<repo::LocalPlaylist> {
    crate::library_db::with_db(|db| Ok(db.with_connection(|conn| repo::get(conn, id))))
        .and_then(|r| r.ok())
        .flatten()
}

pub fn get_tracks_blocking(id: &str) -> Vec<repo::LocalPlaylistTrack> {
    crate::library_db::with_db(|db| Ok(db.with_connection(|conn| repo::get_tracks(conn, id))))
        .and_then(|r| r.ok())
        .unwrap_or_default()
}

pub fn create_blocking(name: &str, description: Option<&str>, offline_only: bool) -> Option<String> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::create(conn, name, description, offline_only)))
    })
    .and_then(|r| r.ok())
}

pub fn update_blocking(id: &str, name: &str, description: Option<&str>, offline_only: bool) -> bool {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            repo::rename(conn, id, name)?;
            repo::set_description(conn, id, description)?;
            repo::set_offline_only(conn, id, offline_only)
        }))
    })
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

pub fn delete_blocking(id: &str) -> bool {
    crate::library_db::with_db(|db| Ok(db.with_connection(|conn| repo::delete(conn, id))))
        .map(|r| r.is_ok())
        .unwrap_or(false)
}

/// B3: persist the manager's favorite flag for a local playlist.
pub fn set_favorite_blocking(id: &str, favorite: bool) -> bool {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::set_favorite(conn, id, favorite)))
    })
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

/// B3: persist the manager's hidden flag for a local playlist (hidden
/// locals drop from the sidebar list).
pub fn set_hidden_blocking(id: &str, hidden: bool) -> bool {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| repo::set_hidden(conn, id, hidden)))
    })
    .map(|r| r.is_ok())
    .unwrap_or(false)
}
