//! Hero overflow play-mode toggle + convert-kind handlers.

use qbz_models::mixtape::{CollectionKind, CollectionPlayMode};

use crate::artwork::ImageCache;
use crate::AppWindow;

use super::super::modal::finish;
use super::super::reload::reload;
use super::super::with_repo;

/// Hero overflow play-mode toggle: flip in_order <-> album_shuffle. Reads the
/// current mode from `MyQbzDetailState.play_mode`, persists the OTHER, reloads.
pub fn toggle_play_mode(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
    current_mode: String,
) {
    if id.is_empty() {
        return;
    }
    let next = if current_mode == "in_order" {
        CollectionPlayMode::AlbumShuffle
    } else {
        CollectionPlayMode::InOrder
    };
    handle.clone().spawn(async move {
        let write_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| qbz_mixtape::repo::set_play_mode(conn, &write_id, next))
        })
        .await
        .unwrap_or_else(|e| Err(format!("play-mode task panicked: {e}")));

        finish(&weak, &handle, &image_cache, id, result, None, qbz_i18n::t("Failed to change play mode"));
    });
}

/// Hero overflow convert-kind: flip mixtape <-> collection. The repo rejects
/// any artist_collection conversion -> "Cannot convert this kind"; success ->
/// "Converted". Reads the current kind from `MyQbzDetailState.kind`.
pub fn convert_kind(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    id: String,
    current_kind: String,
) {
    if id.is_empty() {
        return;
    }
    // artist_collection is non-convertible (the menu item is hidden for it, but
    // guard here too).
    let next = match current_kind.as_str() {
        "mixtape" => CollectionKind::Collection,
        "collection" => CollectionKind::Mixtape,
        _ => {
            crate::toast::error_weak(&weak, qbz_i18n::t("Cannot convert this kind"));
            return;
        }
    };
    handle.clone().spawn(async move {
        let write_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_repo(|conn| qbz_mixtape::repo::set_kind(conn, &write_id, next))
        })
        .await
        .unwrap_or_else(|e| Err(format!("convert-kind task panicked: {e}")));

        match result {
            Ok(()) => {
                crate::toast::success_weak(&weak, qbz_i18n::t("Converted"));
                reload(&weak, &handle, &image_cache, id);
            }
            Err(_) => {
                // The repo's only rejection here is the artist_collection guard.
                crate::toast::error_weak(&weak, qbz_i18n::t("Cannot convert this kind"));
            }
        }
    });
}
