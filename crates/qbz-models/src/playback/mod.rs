//! Playback-related types for QBZ
//!
//! This module contains types related to audio playback:
//! - Queue track representation
//! - Repeat mode
//! - Queue state snapshots
//! - Playback state
//!
//! Note: Audio backend types (AudioBackendType, AudioDevice, etc.) are defined
//! in qbz-audio crate to keep the audio module self-contained and immutable.

mod queue_state;
mod queue_track;
mod status;

pub use queue_state::{QueueState, RepeatMode};
pub use queue_track::QueueTrack;
pub use status::{PlaybackState, PlaybackStatus};
