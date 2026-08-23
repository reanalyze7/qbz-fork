//! Playlist Manager — richer folder + settings/stats access. The sidebar
//! only needs id+name (see `folders::sidebar`); the Playlist Manager needs
//! the full folder record (icon preset/color/custom-image/hidden) plus the
//! per-playlist settings/stats and local track counts.

mod read;
mod write;

pub use read::{
    load_folders_full, playlist_local_counts, playlist_play_counts, playlist_settings_map,
    FolderFull, PlaylistSettingsLite,
};
pub use write::{
    create_folder_full, reorder_playlists, set_favorite, set_folder_hidden, set_hidden,
    update_folder_full,
};
