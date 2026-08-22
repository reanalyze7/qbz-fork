//! Source-aware playback types.
//!
//! Playable tracks reach the queue from multiple origins: Qobuz streaming,
//! the offline cache (downloaded Qobuz), and local files. These types let
//! every frontend reason about a track's origin and resolve its cover art
//! uniformly, instead of branching on stringly-typed `source` values at each
//! call site.
//!
//! This is the frontend-agnostic contract behind the source-aware playback
//! context: the now-playing bar, the queue, and the artwork pipeline consume
//! `PlaybackSource` + [`ArtworkRef`] and never special-case a source themselves.
//! The same contract drives the Qobuz Connect queue gate (only castable tracks
//! may be cast — see [`PlaybackSource::is_castable_to_qconnect`]).

mod artwork_ref;
mod playback_source;
mod queue_track_ext;

pub use artwork_ref::ArtworkRef;
pub use playback_source::{PlaybackSource, TrackOriginTag};

#[cfg(test)]
mod tests;
