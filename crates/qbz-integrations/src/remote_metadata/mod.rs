//! Remote-metadata converters (MusicBrainz + Discogs -> unified DTOs).
//!
//! Frontend-agnostic copy of the pure converters that live in
//! `src-tauri/src/library/remote_metadata/`, so the Slint frontend can do
//! remote album lookup via `qbz_integrations` without depending on the Tauri
//! binary. The Tauri side keeps its own copy + its cache/state orchestration;
//! only these pure adapters are shared here.

mod discogs_convert;
mod discogs_parse;
mod models;
mod musicbrainz_metadata;
mod musicbrainz_search;

pub use discogs_convert::{discogs_extended_to_search_result, discogs_full_to_metadata};
pub use discogs_parse::{parse_discogs_duration, parse_discogs_position};
pub use models::{
    RemoteAlbumMetadata, RemoteAlbumSearchResult, RemoteMetadataError, RemoteProvider,
    RemoteSearchRequest, RemoteSearchResponse, RemoteTrackMetadata,
};
pub use musicbrainz_metadata::musicbrainz_full_to_metadata;
pub use musicbrainz_search::musicbrainz_release_to_search_result;
