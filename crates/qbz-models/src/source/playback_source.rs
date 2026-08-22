//! `PlaybackSource` / `TrackOriginTag` — where a playable track comes from.

use serde::{Deserialize, Serialize};

/// Where a playable track comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackSource {
    /// The Qobuz streaming catalog.
    Qobuz,
    /// A Qobuz track downloaded into the offline cache (`qobuz_download`).
    OfflineCache,
    /// A local file indexed in the user's library.
    Local,
}

impl PlaybackSource {
    /// Parse the stringly-typed `source` value carried by `QueueTrack` /
    /// `LocalTrack`. Unknown or absent values default to [`Qobuz`] — every
    /// pre-existing queue track was Qobuz, so this preserves history.
    ///
    /// [`Qobuz`]: PlaybackSource::Qobuz
    pub fn from_source_str(s: Option<&str>) -> Self {
        match s {
            Some("local") => Self::Local,
            Some("qobuz_download") => Self::OfflineCache,
            _ => Self::Qobuz,
        }
    }

    /// The canonical string written to `source` fields.
    pub fn as_source_str(self) -> &'static str {
        match self {
            Self::Qobuz => "qobuz",
            Self::OfflineCache => "qobuz_download",
            Self::Local => "local",
        }
    }

    /// Whether this source streams live from the Qobuz catalog. NOTE: NOT the
    /// cast gate — offline-cache also carries a valid Qobuz id and IS castable.
    /// Use is_castable_to_qconnect for the Qobuz Connect gate.
    pub fn is_qobuz_streamable(self) -> bool {
        matches!(self, Self::Qobuz)
    }

    /// The admission-side cast predicate. Offline-cache maps to castable (the
    /// offline copy carries a valid Qobuz track id). This is the method the
    /// QConnect gate consults; is_qobuz_streamable stays "streams live from Qobuz".
    ///
    /// Shared QConnect-admission gate primitive: this is the single predicate
    /// both the Tauri and the upcoming Slint frontends call to gate casting.
    pub fn is_castable_to_qconnect(self) -> bool {
        matches!(self, Self::Qobuz | Self::OfflineCache)
    }

    /// Strict parse for the admission path: unknown/absent → ExternalUnknown (blocked).
    ///
    /// Shared QConnect-admission gate primitive consumed by the Slint port (it
    /// feeds the cast gate, where unknown origins must block, not default to Qobuz).
    pub fn from_source_str_strict(s: Option<&str>) -> TrackOriginTag {
        match s {
            Some("qobuz") => TrackOriginTag::Qobuz,
            Some("qobuz_download") => TrackOriginTag::OfflineCache,
            Some("local") => TrackOriginTag::Local,
            _ => TrackOriginTag::ExternalUnknown,
        }
    }
}

/// Admission-only origin tag. Unlike PlaybackSource, this has ExternalUnknown
/// so the Qobuz Connect gate can default unknown/absent to *blocked* not *Qobuz*.
///
/// Shared QConnect-admission gate primitive consumed by the Slint port (its
/// strict-parse companion for the cast gate); kept intentionally for that use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackOriginTag {
    Qobuz,
    OfflineCache,
    Local,
    ExternalUnknown,
}

impl TrackOriginTag {
    pub fn is_castable_to_qconnect(self) -> bool {
        matches!(self, Self::Qobuz | Self::OfflineCache)
    }
}
