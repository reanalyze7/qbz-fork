//! Cross-provider resource/provider enums shared by detection and dispatch.

use serde::{Deserialize, Serialize};

/// Which streaming platform a music link belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicProvider {
    Spotify,
    AppleMusic,
    Tidal,
    Deezer,
}

/// The kind of resource a music URL points to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicResource {
    /// A native Qobuz URL — resolve directly.
    Qobuz,
    /// A single track on a third-party platform.
    Track {
        provider: MusicProvider,
        url: String,
    },
    /// An album on a third-party platform.
    Album {
        provider: MusicProvider,
        url: String,
    },
    /// A playlist — should be redirected to the Playlist Importer.
    Playlist { provider: MusicProvider },
    /// A song.link / album.link / odesli.co URL — resolve via Odesli API.
    SongLink { url: String },
}
