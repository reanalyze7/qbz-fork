//! Row loading: the blocking DB read + kind-restriction + sort +
//! `item_exists` resolution.

use qbz_models::mixtape::{CollectionKind, MixtapeCollection};

use super::{source_from_str, AddItem};

/// A loaded picker row (the collection + whether it already contains every
/// pending item). Built on a worker thread by [`load_rows`].
pub struct LoadedRow {
    pub id: String,
    pub name: String,
    pub kind: CollectionKind,
    pub item_count: usize,
    /// True when EVERY pending item already exists in this collection.
    pub already_has: bool,
}

/// Load the collections offered as targets, kind-restricted + recency-sorted +
/// `item_exists`-resolved. Blocking (DB) — run on a worker thread.
///
/// - `restrict_to_mixtape` → only `kind == mixtape` (excludes collections AND
///   artist_collections, the latter never a user target).
/// - sort = `last_played_at ?? updated_at` DESC (most-recently-played, then
///   most-recently-updated), matching `sortedCollections` in the PSD.
/// - `already_has` = every pending item's `(source, source_item_id)` already in
///   the collection (so the row can show an "already added" hint).
pub fn load_rows(restrict_to_mixtape: bool, items: &[AddItem]) -> Vec<LoadedRow> {
    crate::library_db::with_db(|db| {
        Ok(db.with_connection(|conn| {
            let mut cols: Vec<MixtapeCollection> =
                qbz_mixtape::repo::list_collections(conn, None).unwrap_or_else(|e| {
                    log::warn!("[qbz-slint] myqbz_add list_collections failed: {e}");
                    Vec::new()
                });

            // Kind restriction.
            if restrict_to_mixtape {
                cols.retain(|c| c.kind == CollectionKind::Mixtape);
            } else {
                // An album can be added to ANY collection kind — Mixtape,
                // Collection, OR Artist Collection. Tauri allows adding an album
                // to an artist_collection (the user can augment a built
                // discography), so no kind restriction here.
            }

            // Sort by last_played_at ?? updated_at DESC.
            cols.sort_by(|a, b| {
                let ra = a.last_played_at.unwrap_or(a.updated_at);
                let rb = b.last_played_at.unwrap_or(b.updated_at);
                rb.cmp(&ra)
            });

            cols.into_iter()
                .map(|c| {
                    // already_has = every pending item is already present.
                    let already_has = !items.is_empty()
                        && items.iter().all(|it| {
                            qbz_mixtape::repo::item_exists(
                                conn,
                                &c.id,
                                source_from_str(&it.source),
                                &it.source_item_id,
                            )
                            .unwrap_or(false)
                        });
                    LoadedRow {
                        id: c.id,
                        name: c.name,
                        kind: c.kind,
                        item_count: c.items.len(),
                        already_has,
                    }
                })
                .collect::<Vec<_>>()
        }))
    })
    .unwrap_or_default()
}
