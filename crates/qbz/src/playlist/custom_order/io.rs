//! DB-facing custom-order load/persist (blocking; run on a worker thread).

/// Load the playlist's custom order from library.db, seeding it from
/// `seed` keys (see [`super::custom_seed_keys`]) if none exists. Returns
/// `((track_id, is_local), position)` rows — `is_local` is kept (Seam E;
/// the old reader dropped it, which collides once mixed rows exist).
/// Blocking — run on a worker thread.
pub fn load_or_init_custom(
    playlist_id: u64,
    seed: Vec<(i64, bool)>,
) -> Vec<((u64, bool), i32)> {
    crate::library_db::with_db(|db| {
        let has = db.has_playlist_custom_order(playlist_id)?;
        if !has {
            db.init_playlist_custom_order(playlist_id, &seed)?;
        }
        db.get_playlist_custom_order(playlist_id)
    })
    .unwrap_or_default()
    .into_iter()
    .map(|(id, is_local, pos)| ((id as u64, is_local), pos))
    .collect()
}

/// Persist the full custom order (DELETE + INSERT — self-healing),
/// `is_local` per row (Seam E — bidirectionally compatible with Tauri's
/// `playlist_track_custom_order`). Blocking.
pub fn persist_custom(playlist_id: u64, orders: Vec<(u64, bool, i32)>) {
    let rows: Vec<(i64, bool, i32)> = orders
        .into_iter()
        .map(|(id, is_local, pos)| (id as i64, is_local, pos))
        .collect();
    crate::library_db::with_db(|db| db.set_playlist_custom_order(playlist_id, &rows));
}
