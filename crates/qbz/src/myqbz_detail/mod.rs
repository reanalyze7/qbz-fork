//! My QBZ — Collection / Mixtape DETAIL view controller (read-only slice).
//!
//! Mirrors `crate::playlist` (a cached full item list backs a client-side
//! filter -> search -> sort that re-derives the visible model) and reuses the
//! grid controller's mosaic + URL-downscale helpers from `crate::myqbz`. It
//! loads ONE `MixtapeCollection` (items come hydrated) via
//! `qbz_mixtape::repo::get_collection` through `library_db::with_db` +
//! `with_connection`, precomputes every display string (type label, source
//! kind, quality detail, tracks/year columns, downscaled `_50` row artwork
//! URL, up-to-9 hero-mosaic URLs), and pushes ready-to-render
//! `MixtapeDetailItem`s into `MyQbzDetailState`. The view does NO per-row
//! lookups.
//!
//! READ-ONLY SCOPE (Phase-2 Slice 3): nav-in (the grid card click) routes here
//! and loads real data — that is the testable path. The hero CTAs
//! (play/shuffle/dj-mix/edit/delete/sync), per-row context-menu items, and the
//! select-mode bulk bar are VISIBLE 1:1 but their handlers are logging stubs
//! (wired in main.rs). DEFERRED to a later slice: the live source/quality
//! `resolveItems` resolution (so quality badges + local source kinds are
//! placeholders here, derived only from the stored `source`), the per-item
//! inline track expansion (the "expanded" view-mode renders its toggle + shell
//! only), the rename/description/delete/cover/DJ-mix modals, and persisted
//! per-collection view-prefs.
//!
//! The backend (`qbz-mixtape`) is reused directly — no Tauri command wrappers
//! (ADR-005), headless (ADR-006).

mod artwork;
mod hero;
mod inline_tracks;
mod lifecycle;
mod model;
mod resolve;
mod resolved_item;
mod selection;
mod strings;
mod toolbar;

pub use artwork::{artwork_jobs, dispatch_artwork, set_row_artwork};
pub use hero::set_hero_cover;
pub use inline_tracks::ensure_expanded;
pub use lifecycle::{get_collection, navigate};
pub use selection::{
    clear_selection, selected_full_items, selected_positions, toggle_item_select,
    toggle_select_mode,
};
pub use strings::{item_type_str, source_str};
pub use toolbar::{
    reset_filters, search, set_sort, set_type_filter, set_view_mode,
    toggle_source_filter,
};

/// Process-global runtime handle, set ONCE during startup wiring. The
/// mutation-reload paths (cover upload/remove, rename/description/convert/
/// remove-selected) re-run `navigate` to refresh the open detail, and
/// `navigate`'s resolveItems pass needs the runtime; rather than thread the
/// `Arc<AppRuntime>` through every one of those entry points + their main.rs
/// callsites, they pull it from here. The primary nav-in path still passes the
/// runtime explicitly. Set by `set_runtime` at wiring time.
static GLOBAL_RUNTIME: std::sync::OnceLock<
    std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>,
> = std::sync::OnceLock::new();

/// Store the shared runtime for the global reload paths (idempotent — a second
/// call is ignored). Called once during startup wiring.
pub fn set_runtime(
    runtime: std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>,
) {
    let _ = GLOBAL_RUNTIME.set(runtime);
}

/// The shared runtime for the reload paths. `None` only before wiring (never in
/// practice, since reloads happen after the UI is up).
pub fn global_runtime(
) -> Option<std::sync::Arc<qbz_app::shell::AppRuntime<crate::adapter::SlintAdapter>>> {
    GLOBAL_RUNTIME.get().cloned()
}

thread_local! {
    /// The full, original-order item list for the open collection — the
    /// canonical source the toolbar derives the visible list from. UI thread
    /// only (mirrors `playlist::FULL_ITEMS`).
    pub(in crate::myqbz_detail) static FULL_ITEMS: std::cell::RefCell<Vec<qbz_models::mixtape::MixtapeCollectionItem>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Expanded-mode inline-tracks cache, keyed by a STABLE per-item key
    /// (`source|source_item_id`). Populated ONCE per item when its inline tracks
    /// are first resolved (spec 12 §8). It is the durable home of the resolved
    /// tracks: `refresh_view` rebuilds the `MixtapeDetailItem` render rows on
    /// every filter/sort/search (so the per-row `inline_tracks` would be wiped),
    /// so after each re-derive we re-hydrate the rows from THIS cache instead of
    /// re-fetching. The cached `Vec<TrackItem>` carries `slint::Image`s (`!Send`)
    /// — safe here because the cache lives on the UI thread only. Cleared on
    /// `reset` (a fresh collection open). Mirrors the Tauri per-item track cache
    /// that survives the client-side re-derive.
    pub(in crate::myqbz_detail) static INLINE_CACHE: std::cell::RefCell<std::collections::HashMap<String, Vec<crate::TrackItem>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// Per-collection view-prefs "hydrated" gate (mirrors Tauri's
    /// `prefsHydrated`). `false` from `reset` until `apply` has restored the
    /// stored prefs; while `false` every toolbar persist is suppressed so an
    /// early setter can't clobber the about-to-be-restored prefs. UI thread.
    pub(in crate::myqbz_detail) static PREFS_HYDRATED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    /// resolveItems cache, keyed by the STABLE per-item key
    /// (`source|source_item_id`). Holds the LIVE-resolved per-row display values
    /// that `MixtapeCollectionItem` alone can't carry: the resolved source kind
    /// (qobuz / local), the album-level quality tier + detail (derived
    /// from the item's first resolved track), and the resolved TYPE label
    /// (album -> EP/Single/Album by track count). Populated once per item by the
    /// `resolve_items` pass (spawned after `apply`), re-hydrated in `to_item` on
    /// every filter/sort/search re-derive so the columns stay populated without
    /// re-fetching. Cleared on `reset`. UI thread only.
    pub(in crate::myqbz_detail) static RESOLVE_CACHE: std::cell::RefCell<std::collections::HashMap<String, resolved_item::ResolvedItem>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}
