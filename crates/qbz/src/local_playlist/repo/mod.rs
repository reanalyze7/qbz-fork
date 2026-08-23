//! Blocking repo wrappers. All open the per-user library.db fresh on the
//! calling (blocking) thread via `library_db::with_db` — never call on the
//! UI/event-loop thread.

mod add_tracks;
mod covers;
mod crud;

pub use add_tracks::{
    add_drag_tracks_blocking, add_local_refs_blocking, add_qobuz_tracks_blocking,
};
pub(crate) use add_tracks::local_row_input;
pub use covers::{clear_custom_artwork_blocking, resolve_cover_urls, set_custom_artwork_blocking};
pub use crud::{
    create_blocking, delete_blocking, get_blocking, get_tracks_blocking, list_blocking,
    set_favorite_blocking, set_hidden_blocking, update_blocking,
};
