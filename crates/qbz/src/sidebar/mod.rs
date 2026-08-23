//! Sidebar playlists + folders controller. Builds the flattened
//! left-nav list (folder headers with their playlists + root
//! playlists) from the user's Qobuz playlists and the local folder
//! organization (library.db). The loaded data is cached so expand /
//! move operations rebuild the list without re-hitting the network.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

use slint::ComponentHandle;

use crate::folders::FolderInfo;
use crate::{AppWindow, SidebarState};

mod artwork;
mod entry_build;
mod folder_popup;
mod load;
mod load_meta;
mod load_offline;
mod load_playlists;
mod lookups;
mod mutate;
mod offline_filter;
mod rebuild;
mod rebuild_folder;
mod sort_search;

pub use artwork::artwork_jobs;
pub use folder_popup::load_folder_popup;
pub use load::load;
pub use lookups::{
    local_playlist_meta, playlist_name_desc, playlist_track_count, search_menu_folders, set_active,
};
pub use mutate::{apply, move_local_playlist_local, move_playlist_local, rename_entry, toggle_folder};
pub use rebuild::rebuild;
pub use sort_search::{set_search, set_sort};

#[derive(Clone)]
pub struct SidebarPlaylist {
    pub id: u64,
    pub name: String,
    /// Playlist description (Qobuz). Empty when none. Used to prefill the
    /// edit-playlist modal opened from the sidebar context menu.
    pub description: String,
    /// Total track count (Qobuz). Used by the "# of tracks" sort.
    pub tracks_count: u32,
    /// Up to four cover-art URLs for the micro-collage, sourced from the
    /// `get_user_playlists()` payload (images300 / images150 / images).
    /// No extra fetch — same as Tauri's `images150 ?? images300 ?? images`.
    pub cover_urls: Vec<String>,
    /// Custom-sort position (from `playlist_settings.position`); the
    /// `Custom` sort orders by this ascending.
    pub position: i32,
}

/// One LOCAL playlist (library.db entity, id `local:<uuid>`) listed
/// alongside the Qobuz playlists. Always available — including offline.
#[derive(Clone)]
pub struct LocalSidebarPlaylist {
    pub id: String,
    pub name: String,
    pub description: String,
    pub offline_only: bool,
    /// Sidebar folder membership (shared `playlist_folders.id`); None = root.
    pub folder_id: Option<String>,
    /// Up to four cover refs for the micro-collage, resolved from the
    /// playlist's tracks' artwork (local file paths / cached
    /// Qobuz covers — no network). Empty = render the hard-drive glyph.
    pub cover_urls: Vec<String>,
}

#[derive(Clone, Default)]
pub struct SidebarData {
    pub playlists: Vec<SidebarPlaylist>,
    pub folders: Vec<FolderInfo>,
    pub folder_map: HashMap<u64, String>,
    /// Playlist ids the user has hidden from the sidebar (local settings).
    pub hidden_playlists: HashSet<u64>,
    /// First-class local playlists (offline-mode D7), appended as root rows.
    pub local_playlists: Vec<LocalSidebarPlaylist>,
    /// Qobuz playlist id -> local sidecar track count (library.db
    /// `playlist_local_tracks`). The D11.b offline filter keeps only the
    /// MIXED playlists (count > 0); unused while online.
    pub local_counts: HashMap<u64, u32>,
    /// B8: Qobuz playlists whose local SNAPSHOT membership has >= 1 track
    /// playable offline (snapshot ∩ cached, grace-gated). Extends the
    /// D11.b offline filter; empty while online.
    pub snapshot_available: HashSet<u64>,
}

/// Session-only folder expand state (matches Tauri — not persisted).
static EXPANDED: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
/// Last loaded data, so expand/move rebuild without a refetch.
static CACHE: LazyLock<Mutex<SidebarData>> = LazyLock::new(|| Mutex::new(SidebarData::default()));
/// Active sort option, mirrored from `SidebarState.sort-option`. Session
/// scope (Tauri persists in localStorage; we have no equivalent store
/// here yet, so this matches the in-session behavior).
static SORT: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("name".to_string()));
/// Active playlist-name search query (lowercased), mirrored from
/// `SidebarState.search-query`. Filters the rebuilt list recursively.
static SEARCH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
/// playlist id -> (name, description) from the last loaded payload, so the
/// sidebar context menu can prefill the edit-playlist modal without a
/// refetch.
static NAME_DESC: LazyLock<Mutex<HashMap<u64, (String, String)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_loading(window: &AppWindow, loading: bool) {
    window.global::<SidebarState>().set_loading(loading);
}
