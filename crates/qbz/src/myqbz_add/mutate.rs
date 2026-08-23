//! The DB-writing / mutation half: add items, create a collection, and
//! surface the outcome via a toast.

use qbz_models::mixtape::CollectionKind;

use super::{item_type_from_str, pending_snapshot, source_from_str, AddItem};
use crate::{AppWindow, ToastKind};

/// Result of a batch insert: how many were inserted and how many were skipped
/// as duplicates.
pub struct AddOutcome {
    pub added: usize,
    pub skipped: usize,
}

/// Insert every pending item into `collection_id` with `allow_duplicate=false`.
/// Blocking (DB). Returns the added/skipped tally (a `false` return from the
/// repo = a dedup-rejected duplicate, NOT an error).
pub fn add_items(collection_id: &str, items: &[AddItem]) -> AddOutcome {
    let mut added = 0usize;
    let mut skipped = 0usize;
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            for it in items {
                match qbz_mixtape::repo::add_item_with(
                    conn,
                    collection_id,
                    item_type_from_str(&it.item_type),
                    source_from_str(&it.source),
                    &it.source_item_id,
                    &it.title,
                    it.subtitle.as_deref(),
                    it.artwork_url.as_deref(),
                    it.year,
                    it.track_count,
                    false,
                ) {
                    Ok(true) => added += 1,
                    Ok(false) => skipped += 1,
                    Err(e) => {
                        log::warn!("[qbz-slint] myqbz_add add_item failed: {e}");
                    }
                }
            }
        }))
    });
    AddOutcome { added, skipped }
}

/// Surface the add outcome via a toast (matches Tauri's `toastBatchResult` net
/// behavior). `name` is the collection name.
pub fn toast_outcome(window: &AppWindow, name: &str, outcome: &AddOutcome) {
    let msg = if outcome.added == 0 {
        // Nothing inserted -> everything was a duplicate ("Already in {name}").
        qbz_i18n::t_args("Already in {}", &[name])
    } else if outcome.skipped > 0 {
        let skipped_label = qbz_i18n::tf(
            "{} duplicate skipped",
            "{} duplicates skipped",
            outcome.skipped as i64,
            &[&outcome.skipped.to_string()],
        );
        qbz_i18n::t_args(
            "Added {} to {} ({})",
            &[&outcome.added.to_string(), name, &skipped_label],
        )
    } else {
        qbz_i18n::t_args("Added {} to {}", &[&outcome.added.to_string(), name])
    };
    let kind = if outcome.added == 0 {
        ToastKind::Info
    } else {
        ToastKind::Success
    };
    crate::toast::show(window, msg, kind);
}

/// Take a snapshot of the pending items (clone). Used by the action handlers in
/// `main.rs` to hand the payload to a blocking worker.
pub fn take_pending() -> Vec<AddItem> {
    pending_snapshot()
}

/// Build `track` payloads from a batch of LocalTracks (source "local").
/// Subtitle =
/// "artist · album"; no artwork_url / year / track_count (1:1 PSD §R).
pub fn track_items_from_local(tracks: &[qbz_library::LocalTrack]) -> Vec<AddItem> {
    tracks
        .iter()
        .map(|t| {
            let subtitle = [t.artist.clone(), t.album.clone()]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" · ");
            AddItem {
                item_type: "track".into(),
                source: "local".into(),
                source_item_id: t.id.to_string(),
                title: t.title.clone(),
                subtitle: (!subtitle.is_empty()).then_some(subtitle),
                artwork_url: None,
                year: None,
                track_count: None,
            }
        })
        .collect()
}

/// Create a new manual collection of `kind` named `name`, returning
/// `(id, name)` on success. Blocking (DB). `kind` is "mixtape" | "collection".
pub fn create_collection(kind: &str, name: &str) -> Option<(String, String)> {
    let kind = match kind {
        "collection" => CollectionKind::Collection,
        _ => CollectionKind::Mixtape,
    };
    crate::myqbz::create_collection(kind, name).map(|c| (c.id, c.name))
}
