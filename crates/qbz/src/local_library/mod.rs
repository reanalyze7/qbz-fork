//! Local Library controller (Slint) — greenfield port of Tauri's
//! `LocalLibraryView`.
//!
//! This module owns the per-tab navigation and (slice by slice) the data
//! loading for the four browse tabs: Albums / Artists / Folders / Tracks.
//! It reads the shared per-user `library.db` through the already
//! frontend-agnostic `qbz-library` crate (see `library_db::with_db`).
//!
//! Folder management, scan, maintenance, and the danger zone do NOT live in
//! this view — they belong under Settings > Local Library. The view's gear
//! button routes there.
//!
//! Split by tab/domain: `albums`, `tracks`, `album_detail`, `folders_flat`,
//! `folders_tree`, `artists`, `ephemeral`, plus `shared` for the handful of
//! cross-tab helpers. Every `pub fn`/`pub` item stays reachable at its
//! original `crate::local_library::X` path via the `pub use` below.

mod album_detail;
mod albums;
mod artists;
mod cover;
mod ephemeral;
mod folders_flat;
mod folders_tree;
mod shared;
mod tracks;

pub use album_detail::*;
pub use albums::*;
pub use artists::*;
pub use cover::*;
pub use ephemeral::*;
pub use folders_flat::*;
pub use folders_tree::*;
pub use shared::*;
pub use tracks::*;

/// The four browse tabs. Order mirrors Tauri's default tab order
/// (`tracks / folders / albums / artists`); the visible order is a user
/// preference layered on top later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibTab {
    Tracks,
    Folders,
    Albums,
    Artists,
}

impl LibTab {
    /// Parse a HeaderBar menu route (`"local-albums"` etc.).
    pub fn from_route(route: &str) -> Option<Self> {
        match route {
            "local-tracks" => Some(Self::Tracks),
            "local-folders" => Some(Self::Folders),
            "local-albums" => Some(Self::Albums),
            "local-artists" => Some(Self::Artists),
            _ => None,
        }
    }

    /// Parse a tab id (`"albums"` etc.) as carried by a `NavEntry` and by
    /// `LocalLibraryState.active-tab`.
    pub fn from_tab_id(id: &str) -> Option<Self> {
        match id {
            "tracks" => Some(Self::Tracks),
            "folders" => Some(Self::Folders),
            "albums" => Some(Self::Albums),
            "artists" => Some(Self::Artists),
            _ => None,
        }
    }

    /// The canonical tab id used in nav entries + `LocalLibraryState.active-tab`.
    pub fn tab_id(self) -> &'static str {
        match self {
            Self::Tracks => "tracks",
            Self::Folders => "folders",
            Self::Albums => "albums",
            Self::Artists => "artists",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_and_tab_id_round_trip() {
        for (route, id, tab) in [
            ("local-tracks", "tracks", LibTab::Tracks),
            ("local-folders", "folders", LibTab::Folders),
            ("local-albums", "albums", LibTab::Albums),
            ("local-artists", "artists", LibTab::Artists),
        ] {
            assert_eq!(LibTab::from_route(route), Some(tab));
            assert_eq!(LibTab::from_tab_id(id), Some(tab));
            assert_eq!(tab.tab_id(), id);
        }
        assert_eq!(LibTab::from_route("favorites-albums"), None);
        assert_eq!(LibTab::from_tab_id("bogus"), None);
    }
}
