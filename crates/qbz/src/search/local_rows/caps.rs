/// Per-section caps for the LOCAL cortinilla sections. Two profiles: the NORMAL
/// (online + signed-in) profile keeps the on-device block compact since the
/// Qobuz catalog dominates the dropdown; the EXPANDED profile (offline OR not
/// signed in, so the cortinilla is local-only) turns it into a small on-device
/// browser with more rows per section.
#[derive(Debug, Clone, Copy)]
pub struct LocalCaps {
    pub albums: usize,
    pub artists: usize,
    pub tracks: usize,
}

impl LocalCaps {
    /// Normal profile (Qobuz present): compact on-device block.
    const NORMAL: LocalCaps = LocalCaps {
        albums: 3,
        artists: 2,
        tracks: 3,
    };
    /// Expanded profile (offline / not signed in → local-only dropdown).
    const EXPANDED: LocalCaps = LocalCaps {
        albums: 8,
        artists: 4,
        tracks: 8,
    };

    /// Pick the profile for the current session state. `expand` is true when the
    /// session is offline OR unauthenticated (the cortinilla has no Qobuz half).
    pub fn for_session(expand: bool) -> LocalCaps {
        if expand {
            Self::EXPANDED
        } else {
            Self::NORMAL
        }
    }

    /// How many raw local TRACK rows to fetch so the grouped album/artist
    /// sections can be filled. Albums/artists are derived by grouping tracks, so
    /// a single album can swallow many rows — over-fetch well beyond the shown
    /// caps to surface enough distinct groups.
    pub(crate) fn fetch_limit(self) -> u64 {
        ((self.albums.max(self.tracks) * 12) + 40) as u64
    }
}

/// The artwork key for a local cortinilla row: the RAW path, so the search
/// artwork dispatcher (`artwork::spawn_search_loads`) can route it by scheme —
/// http(s) → Qobuz CDN, anything else (an absolute filesystem path) →
/// `LocalFile` (decoded with `fs::read`, so NO `file://` prefix). A stray
/// `file://` is stripped for the same reason.
pub(crate) fn local_artwork_url(path: Option<&str>) -> String {
    path.map(|p| p.strip_prefix("file://").unwrap_or(p).to_string())
        .unwrap_or_default()
}

/// The canonical "artist" attributed to a local track for grouping: the
/// album-artist tag when present, else the track performer. Mirrors the album-
/// grouping helper in `local_library` so cortinilla artists line up with the
/// LocalLibrary Artists tab (which `open_local_artist` selects by NAME).
pub(crate) fn local_album_artist(t: &qbz_library::LocalTrack) -> String {
    t.album_artist
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| t.artist.clone())
}
