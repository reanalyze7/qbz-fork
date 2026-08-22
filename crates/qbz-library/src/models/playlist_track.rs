//! Track wrapper used within playlist listings

use super::track::LocalTrack;
use serde::{Deserialize, Serialize};

/// A local track within a playlist, including its position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistLocalTrack {
    #[serde(flatten)]
    pub track: LocalTrack,
    /// Position in the combined playlist (Qobuz + local tracks)
    pub playlist_position: i32,
}
