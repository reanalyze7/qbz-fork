//! `ArtworkRef` — a reference to a piece of cover art, resolvable regardless
//! of origin.

use serde::{Deserialize, Serialize};

/// A reference to a piece of cover art, resolvable regardless of origin.
///
/// The artwork loaders historically handled only remote HTTP URLs, which is
/// why local-file artwork failed to reach the UI. This enum is the uniform
/// contract: a frontend's artwork pipeline matches on it and fetches the
/// bytes the right way for each variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtworkRef {
    /// An HTTP(S) URL (Qobuz covers).
    Remote(String),
    /// A path to a cover image on the local filesystem.
    LocalFile(String),
    /// Cover bytes already in memory (e.g. embedded tags).
    Embedded(Vec<u8>),
    /// No artwork available.
    None,
}

impl ArtworkRef {
    /// True when there is effectively nothing to load (explicit `None` or an
    /// empty Remote/LocalFile string).
    pub fn is_empty(&self) -> bool {
        match self {
            ArtworkRef::None => true,
            ArtworkRef::Remote(s) | ArtworkRef::LocalFile(s) => s.is_empty(),
            ArtworkRef::Embedded(b) => b.is_empty(),
        }
    }

    /// A URL suitable for the MPRIS `mpris:artUrl` property (and other OS media
    /// controls). Mirrors the Tauri frontend's `normalizeCoverUrlForMetadata`:
    /// - **Remote** HTTP(S) covers (Qobuz) pass through — clients fetch them.
    /// - **LocalFile** bare paths become a proper percent-encoded `file://`
    ///   URI (MPRIS clients cannot read a bare path or an `asset://` URL).
    /// - **Embedded** bytes / **None** have no URL (`None`).
    pub fn to_mpris_url(&self) -> Option<String> {
        match self {
            ArtworkRef::Remote(s) if !s.is_empty() => Some(s.clone()),
            ArtworkRef::LocalFile(p) if !p.is_empty() => {
                // Already a file URL? keep it. Otherwise build one (absolute
                // paths only — `from_file_path` rejects relative, → None).
                if p.starts_with("file://") {
                    Some(p.clone())
                } else {
                    url::Url::from_file_path(p).ok().map(|u| u.to_string())
                }
            }
            _ => None,
        }
    }
}
