//! Send row structs merged from Qobuz + local + folder data, and the
//! session-scoped caches every submodule reads/writes.

use std::sync::{LazyLock, Mutex};

/// Send view of one playlist merged with its local settings + stats.
#[derive(Clone)]
pub(super) struct PmPlaylist {
    pub(super) id: u64,
    pub(super) name: String,
    /// Remote (Qobuz) track count.
    pub(super) tracks_count: u32,
    /// Total playlist duration in seconds (Qobuz `duration`).
    pub(super) duration: u32,
    /// Local (non-Qobuz) track count.
    pub(super) local_count: u32,
    pub(super) play_count: u32,
    pub(super) is_favorite: bool,
    pub(super) is_hidden: bool,
    pub(super) folder_id: Option<String>,
    pub(super) position: i32,
    /// Up to four de-duplicated cover URLs (same scheme as the sidebar).
    pub(super) cover_urls: Vec<String>,
    /// B8: >= 1 snapshot track playable offline (snapshot ∩ cached,
    /// grace-gated). Extends the D11.b offline filter; false while online.
    pub(super) offline_available: bool,
}

impl PmPlaylist {
    pub(super) fn total_count(&self) -> u32 {
        self.tracks_count + self.local_count
    }
}

/// Send view of one LOCAL playlist (library.db entity, id `local:<uuid>`).
/// Listed alongside the Qobuz set with a hard-drive marker. Favorite /
/// hidden live on the `local_playlists` row itself (B3) and participate in
/// the manager's filter + card actions; folder membership stays a
/// Qobuz-side concept (the folder tables are u64-keyed) and doesn't apply.
#[derive(Clone)]
pub(super) struct PmLocalPlaylist {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) offline_only: bool,
    pub(super) track_count: u32,
    pub(super) is_favorite: bool,
    pub(super) is_hidden: bool,
}

#[derive(Clone, Default)]
pub struct PmData {
    pub(super) playlists: Vec<PmPlaylist>,
    pub(super) folders: Vec<crate::folders::FolderFull>,
    pub(super) locals: Vec<PmLocalPlaylist>,
}

/// Last-loaded data (so toolbar changes rebuild from cache, no refetch).
pub(super) static CACHE: LazyLock<Mutex<PmData>> = LazyLock::new(|| Mutex::new(PmData::default()));
/// Session folder-expand state for the tree view (Tauri: not persisted).
pub(super) static EXPANDED: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
/// True once the tree has auto-expanded folders on first open.
pub(super) static TREE_INIT: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Pick up to four de-duplicated cover URLs (images150 > images300 >
/// images), mirroring `crate::sidebar::playlist_cover_urls`.
pub(super) fn cover_urls(p: &qbz_models::Playlist) -> Vec<String> {
    let source = [&p.images300, &p.images150, &p.images]
        .into_iter()
        .flatten()
        .find(|v| !v.is_empty());
    let mut out: Vec<String> = Vec::new();
    if let Some(list) = source {
        for url in list {
            if !url.is_empty() && !out.contains(url) {
                out.push(url.clone());
            }
            if out.len() == 4 {
                break;
            }
        }
    }
    out
}
