//! ListenBrainz API models
//!
//! Types for ListenBrainz submission payloads and responses

mod playlists;
mod recommendations;
mod status;
mod submit;

pub use playlists::{LbFreshRelease, LbPlaylistMeta, LbPlaylistTrack};
pub use recommendations::{CfRecommendation, LbListen, LbRecordingMeta};
pub use status::{ListenBrainzStatus, QueuedListen, TokenValidationResponse, UserInfo};
pub use submit::{AdditionalInfo, Listen, ListenType, SubmitListensPayload, TrackMetadata};
