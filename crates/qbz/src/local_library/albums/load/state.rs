//! Generation guard + the raw-album cache backing the Albums tab.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use slint::ComponentHandle;

use crate::{AppWindow, LocalLibraryState};

/// Generation guard, bumped on every (re)load. A stale in-flight
/// fetch (older search/sort) is discarded on apply, and an in-flight
/// load-more is dropped once a reload supersedes it.
pub(crate) static ALBUMS_GEN: AtomicU64 = AtomicU64::new(0);

/// True if `gen` is still the current albums generation. The artwork
/// pipeline calls this before applying a decoded cover so an in-flight job
/// from a superseded page (a search/sort/retry replaced the model) doesn't
/// land on a stale row index.
pub fn albums_gen_current(gen: u64) -> bool {
    ALBUMS_GEN.load(Ordering::SeqCst) == gen
}

/// The full metadata-grouped LocalAlbum set — the FILTER SOURCE (the quality/
/// format/source filters need raw bit_depth/format/source, which AlbumCardItem
/// doesn't carry). Loaded once; `derive_albums` filters it by id.
static LOCAL_ALBUMS: LazyLock<Mutex<Vec<qbz_library::LocalAlbum>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub(crate) fn local_albums() -> std::sync::MutexGuard<'static, Vec<qbz_library::LocalAlbum>> {
    LOCAL_ALBUMS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Upper bound for the full-load page. The Albums tab loads the entire set in
/// one shot (search/sort/filter/group are all derived client-side over the
/// cached set in `derive_albums`), so we request a single large page rather
/// than truly paginating. `total` from the page is informational here.
pub(crate) const ALBUMS_FULL_LOAD_LIMIT: u64 = 1_000_000;

/// The user's album-identity mode (Albums tab dropdown / Settings, persisted
/// in locallibrary_ui.json). Read on the UI thread and captured into the
/// blocking DB closures — Slint state is not thread-safe.
pub fn current_group_mode(window: &AppWindow) -> qbz_library::album_grouping::AlbumGroupMode {
    qbz_library::album_grouping::AlbumGroupMode::from_pref(
        &window
            .global::<LocalLibraryState>()
            .get_albums_id_mode(),
    )
}
