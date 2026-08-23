//! Custom (manual) order: the `CUSTOM_ORDER` thread-local, its DB-facing
//! load/persist (`io`), and the pure in-memory reordering ops (`reorder`).

mod io;
mod reorder;

pub use io::{load_or_init_custom, persist_custom};
pub use reorder::{
    apply_custom_order, custom_seed_keys, full_item_ids, move_full_item, move_track,
    reorder_track, swap_full_items,
};

thread_local! {
    /// Custom order positions keyed `(track id, is_local)` — the same
    /// keying `playlist_track_custom_order` uses (Seam E), so local
    /// rows of a mixed playlist can hold an order without colliding with
    /// Qobuz catalog ids. Empty until the custom sort is entered
    /// (loaded/initialized from library.db).
    pub(super) static CUSTOM_ORDER: std::cell::RefCell<std::collections::HashMap<(u64, bool), i32>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}
