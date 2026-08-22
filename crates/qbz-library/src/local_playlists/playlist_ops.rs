//! Playlist-header CRUD (create/rename/flags/delete).

use rusqlite::{params, Connection, Result};
use uuid::Uuid;

use super::model::{now_ms, LOCAL_PLAYLIST_PREFIX};

/// Create a playlist; returns its `local:<uuid>` id.
pub fn create(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    offline_only: bool,
) -> Result<String> {
    let id = format!("{LOCAL_PLAYLIST_PREFIX}{}", Uuid::new_v4());
    let ts = now_ms();
    conn.execute(
        "INSERT INTO local_playlists (id, name, description, offline_only, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, name, description, offline_only as i64, ts],
    )?;
    Ok(id)
}

pub fn rename(conn: &Connection, id: &str, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_name, now_ms(), id],
    )?;
    Ok(())
}

pub fn set_description(conn: &Connection, id: &str, description: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET description = ?1, updated_at = ?2 WHERE id = ?3",
        params![description, now_ms(), id],
    )?;
    Ok(())
}

pub fn set_offline_only(conn: &Connection, id: &str, offline_only: bool) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET offline_only = ?1, updated_at = ?2 WHERE id = ?3",
        params![offline_only as i64, now_ms(), id],
    )?;
    Ok(())
}

/// B3: flip the manager's favorite flag (local twin of
/// `playlist_settings.is_favorite`).
pub fn set_favorite(conn: &Connection, id: &str, favorite: bool) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET favorite = ?1, updated_at = ?2 WHERE id = ?3",
        params![favorite as i64, now_ms(), id],
    )?;
    Ok(())
}

/// B3: flip the manager's hidden flag (local twin of
/// `playlist_settings.hidden`). Hidden playlists drop from the sidebar.
pub fn set_hidden(conn: &Connection, id: &str, hidden: bool) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET hidden = ?1, updated_at = ?2 WHERE id = ?3",
        params![hidden as i64, now_ms(), id],
    )?;
    Ok(())
}

/// Move a local playlist into a folder (`Some(folder_id)`) or back to the
/// sidebar root (`None`). The folder lives in the shared `playlist_folders`
/// table — the same folders Qobuz playlists use.
pub fn move_to_folder(conn: &Connection, id: &str, folder_id: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET folder_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![folder_id, now_ms(), id],
    )?;
    Ok(())
}

/// Null the `folder_id` of every local playlist that pointed at `folder_id`.
/// Called when a folder is deleted: the schema's `ON DELETE SET NULL` only
/// fires when the `foreign_keys` pragma is on, and the app's connections keep
/// it off, so do it explicitly (the same reason `delete` clears tracks by hand).
pub fn clear_folder(conn: &Connection, folder_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET folder_id = NULL WHERE folder_id = ?1",
        params![folder_id],
    )?;
    Ok(())
}

pub fn set_custom_artwork(conn: &Connection, id: &str, path: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE local_playlists SET custom_artwork_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![path, now_ms(), id],
    )?;
    Ok(())
}

/// Delete the playlist. Membership rows are removed explicitly as well as
/// by the FK cascade — `PRAGMA foreign_keys` is connection-scoped and the
/// app's connections don't enable it, so don't rely on the cascade alone.
pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM local_playlist_tracks WHERE playlist_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM local_playlists WHERE id = ?1", params![id])?;
    Ok(())
}
