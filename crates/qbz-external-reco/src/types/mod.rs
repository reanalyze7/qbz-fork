//! Data types for the external-recommendations engine.

use serde::{Deserialize, Serialize};

mod aggregate;
mod candidates;
mod history;
mod reco;

pub use aggregate::ExternalCarousels;
pub use candidates::{AlbumCandidate, ArtistCandidate, TrackCandidate};
pub use history::{ExtHistory, LocalHistory};
pub use reco::{AlbumReco, ArtistReco, TrackReco};

/// Which source produced a recommendation row item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoSource {
    /// In-house artist-vector engine (`qbz-reco`) — deferred, placeholder.
    Internal,
    LastFm,
    ListenBrainz,
    /// Qobuz editorial (cold-start fallback).
    Editorial,
}
