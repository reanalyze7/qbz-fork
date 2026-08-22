//! Unified DTOs for remote metadata providers (MusicBrainz, Discogs)
//!
//! These structs provide a provider-neutral interface for the Tag Editor
//! to consume album metadata from different sources.

mod album;
mod error;
mod provider;
mod request;

#[cfg(test)]
mod tests;

pub use album::{RemoteAlbumMetadata, RemoteAlbumSearchResult, RemoteTrackMetadata};
pub use error::RemoteMetadataError;
pub use provider::RemoteProvider;
pub use request::{RemoteSearchRequest, RemoteSearchResponse};
