//! Reset / clear the ephemeral UI session.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, EphemeralAlbum, LocalLibraryState};

/// Reset the ephemeral UI state to its closed defaults.
pub(crate) fn reset_ephemeral_state(window: &AppWindow) {
    let s = window.global::<LocalLibraryState>();
    s.set_ephemeral_active(false);
    s.set_ephemeral_loading(false);
    s.set_ephemeral_name("".into());
    s.set_ephemeral_path("".into());
    s.set_ephemeral_track_count(0);
    s.set_ephemeral_multi_album(false);
    s.set_ephemeral_albums(ModelRc::new(VecModel::from(Vec::<EphemeralAlbum>::new())));
}

/// Clear the ephemeral session: drop the pane, the in-memory store, and the
/// persisted path.
pub fn clear_ephemeral(window: &AppWindow) {
    reset_ephemeral_state(window);
    crate::ephemeral::clear();
    crate::locallibrary_prefs::save_ephemeral_path(None);
}
