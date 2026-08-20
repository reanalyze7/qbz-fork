//! "Add to playlist" picker controller. Loads the user's playlists into
//! `PlaylistPickerState` for the global picker modal (`PlaylistAddModal`);
//! the pick handler in `main.rs` toggles the pending track(s) in/out of the
//! chosen playlist (checkbox = membership, per
//! `PLAYLIST-REDESIGN-SPEC.md` §4 — a MULTI state per playlist, not the old
//! single `selected-id`). Split across three files to stay under the
//! 130-line budget: this one (open/PENDING), `playlist_picker_load.rs`
//! (the async load), `playlist_picker_apply.rs` (render into Slint).
//!
//! The pending ids/refs are stashed in [`PENDING`] (mirrors the
//! `myqbz_add::PENDING` pattern) instead of threading them through every one
//! of the ~20 `open_multi` + `load` + `apply` call sites in `main.rs`: `open`
//! / `open_multi` set it synchronously, `load` reads it back on the worker
//! thread. Membership math itself lives in `playlist_membership[_qobuz]`.

use std::sync::{Arc, LazyLock, Mutex};

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, PlaylistPickItem, PlaylistPickerState};

// Re-exported so every existing `playlist_picker::load(...)` /
// `playlist_picker::apply(...)` call site in `main.rs` (~20 of them) keeps
// working unchanged — the split into `playlist_picker_load`/`_apply` is an
// internal-only file-size refactor, not an API change.
pub use crate::playlist_picker_apply::{apply, mark_row_already_has};
pub use crate::playlist_picker_load::load;

pub struct PickPlaylist {
    pub id: String,
    pub name: String,
    pub tracks: u32,
    /// LOCAL playlist (library.db, id `local:<uuid>`) — adds write the
    /// local repo (works offline) instead of the Qobuz endpoint.
    pub is_local: bool,
    /// True when every pending id/ref (see [`PENDING`]) is already a member
    /// — drives the row's checked state.
    pub already_has: bool,
}

/// Pending ids/refs for the currently-open picker, plus whether they are
/// LocalLibrary refs (`true`) or Qobuz catalog ids (`false`). Set by
/// [`open`]/[`open_multi`], read by `load` on a worker thread.
pub(crate) static PENDING: LazyLock<Mutex<(Vec<String>, bool)>> =
    LazyLock::new(|| Mutex::new((Vec::new(), false)));

fn set_pending(ids: &[String], local: bool) {
    if let Ok(mut p) = PENDING.lock() {
        *p = (ids.to_vec(), local);
    }
}

pub(crate) fn pending_snapshot() -> (Vec<String>, bool) {
    PENDING.lock().map(|p| p.clone()).unwrap_or_default()
}

/// Open the picker for `track_id` and mark it loading. UI thread.
pub fn open(window: &AppWindow, track_id: &str) {
    let state = window.global::<PlaylistPickerState>();
    state.set_track_id(track_id.into());
    state.set_track_ids(ModelRc::new(VecModel::from(Vec::<slint::SharedString>::new())));
    state.set_playlists(ModelRc::new(VecModel::from(Vec::<PlaylistPickItem>::new())));
    state.set_filter_matches(0);
    state.set_local_mode(false);
    state.set_loading(true);
    state.set_open(true);
    set_pending(&[track_id.to_string()], false);
}

/// Open the picker for a batch of track refs (bulk add), or an empty batch
/// for the sidebar "+" create-only shortcut. `local` marks the refs as
/// LocalLibrary local-mode refs — `"<i64>"` library row ids (resolved
/// source-aware at insert: local path / offline-copy Qobuz id) or
/// library row ids — routed to the library.db add paths
/// instead of the Qobuz endpoint. UI thread.
pub fn open_multi(window: &AppWindow, ids: &[String], local: bool) {
    let state = window.global::<PlaylistPickerState>();
    state.set_track_id("".into());
    let model: Vec<slint::SharedString> = ids.iter().map(|s| s.clone().into()).collect();
    state.set_track_ids(ModelRc::new(VecModel::from(model)));
    state.set_playlists(ModelRc::new(VecModel::from(Vec::<PlaylistPickItem>::new())));
    state.set_filter_matches(0);
    state.set_local_mode(local);
    state.set_loading(true);
    state.set_open(true);
    set_pending(ids, local);
}

/// Open the Add-to-Playlist picker seeded with `ids` and asynchronously
/// populate the user's playlists. Picking an existing playlist toggles the
/// ids in/out of it; the inline "Create new playlist" row create-and-adds
/// them — so this is the single entry point for "create/add a playlist from
/// an arbitrary track-id list" (the queue save-as-playlist + the reco rows).
/// MUST be called on the UI/event-loop thread (it sets Slint globals).
/// `local=false` -> Qobuz u64 ids as strings; `local=true` -> LocalLibrary/
/// local refs.
pub fn open_for_ids<A>(
    window: &AppWindow,
    runtime: Arc<AppRuntime<A>>,
    handle: &tokio::runtime::Handle,
    ids: Vec<String>,
    local: bool,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    open_multi(window, &ids, local);
    let weak = window.as_weak();
    handle.spawn(async move {
        let playlists = load(&runtime).await;
        let _ = weak.upgrade_in_event_loop(move |w| apply(&w, playlists));
    });
}
