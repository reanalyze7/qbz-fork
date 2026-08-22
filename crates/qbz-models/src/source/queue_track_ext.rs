//! `QueueTrack` extension methods for source-aware playback.

use crate::playback::QueueTrack;

use super::{ArtworkRef, PlaybackSource};

impl QueueTrack {
    /// The track's playback source, parsed from its `source` field.
    pub fn source_kind(&self) -> PlaybackSource {
        PlaybackSource::from_source_str(self.source.as_deref())
    }

    /// A uniform reference to this track's cover art.
    ///
    /// The heuristic is source-agnostic (it does not trust `source` to be
    /// set): an `http(s)://` value is [`ArtworkRef::Remote`]; a `file://`
    /// value or a bare filesystem path is [`ArtworkRef::LocalFile`] (local
    /// library + offline-cache covers live on disk).
    pub fn artwork_ref(&self) -> ArtworkRef {
        let raw = self.artwork_url.as_deref().unwrap_or("");
        if raw.is_empty() {
            return ArtworkRef::None;
        }
        if raw.starts_with("http://") || raw.starts_with("https://") {
            ArtworkRef::Remote(raw.to_string())
        } else if let Some(path) = raw.strip_prefix("file://") {
            ArtworkRef::LocalFile(path.to_string())
        } else {
            ArtworkRef::LocalFile(raw.to_string())
        }
    }
}
