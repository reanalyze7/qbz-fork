//! Playlist Manager controller — the full playlist + folder organization
//! surface (Tauri's `PlaylistManagerView`). Local organization layer over
//! the user's Qobuz playlists: folders (icon + color + hidden), per-playlist
//! favorite / hidden / custom-order / folder membership, search / filter /
//! sort, and three view modes (grid / list / tree).
//!
//! The backend is 100% reusable: playlists come from
//! `QbzCore::get_user_playlists`, and folders / settings / stats / local
//! counts come from the per-user `library.db` via `crate::folders` (the
//! same data the Tauri `v2_*` commands back). All DB ops are blocking, so
//! the loader runs them on `spawn_blocking`.
//!
//! Merged row structs are precomputed in Rust (Send) and pushed as
//! ready-to-render Slint models — the view does NO per-row map lookups.
//! Toolbar state (filter / sort / view / folder-mode) is session-scoped,
//! mirrored in this module's statics so rebuilds don't re-hit the network.

mod artwork;
mod build;
mod build_format;
mod load;
mod mutate;
mod navigate;
mod render;
mod sort_filter;
mod types;

pub use artwork::{artwork_jobs, folder_for_edit, load_editor_custom_image, load_folder_custom_images};
pub use load::load;
pub use mutate::{
    move_down, move_to_folder_local, move_up, toggle_favorite_local, toggle_hidden_local,
    toggle_local_favorite, toggle_local_hidden,
};
pub use navigate::navigate;
pub use render::{apply, rebuild, search_menu_folders, toggle_tree_folder};
