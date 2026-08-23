//! Rename + description modal submit handlers.

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::modal::{close_modal, finish, set_busy};
use super::super::with_repo;

/// Rename modal submit: trim the draft; empty -> close without writing. Else
/// `repo::rename_collection` -> reload -> close. Caps at 80 chars (Tauri
/// `<input maxlength=80>`).
pub fn rename(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
    raw_name: String,
) {
    let name: String = raw_name.trim().chars().take(80).collect();
    if id.is_empty() || name.is_empty() {
        close_modal(&weak);
        return;
    }
    set_busy(&weak, true);
    handle.clone().spawn(async move {
        let write_id = id.clone();
        let write_name = name.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| qbz_mixtape::repo::rename_collection(conn, &write_id, &write_name))
        })
        .await
        .unwrap_or_else(|e| Err(format!("rename task panicked: {e}")));

        finish(&weak, &handle, &image_cache, id, result, None, qbz_i18n::t("Failed to rename"));
    });
}

/// Description modal submit: trimmed empty -> NULL (clear). Else set it.
/// `repo::set_description` -> reload -> close.
pub fn set_description(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
    raw_description: String,
) {
    if id.is_empty() {
        close_modal(&weak);
        return;
    }
    let trimmed = raw_description.trim().to_string();
    let desc: Option<String> = if trimmed.is_empty() { None } else { Some(trimmed) };
    set_busy(&weak, true);
    handle.clone().spawn(async move {
        let write_id = id.clone();
        let write_desc = desc.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| {
                qbz_mixtape::repo::set_description(conn, &write_id, write_desc.as_deref())
            })
        })
        .await
        .unwrap_or_else(|e| Err(format!("description task panicked: {e}")));

        finish(&weak, &handle, &image_cache, id, result, None, qbz_i18n::t("Failed to save description"));
    });
}
