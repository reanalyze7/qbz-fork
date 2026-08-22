//! Track-level data models

use super::audio_format::AudioFormat;
use serde::{Deserialize, Serialize};

/// A track from the local library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTrack {
    pub id: i64,
    pub file_path: String,

    // Metadata
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: Option<String>,
    pub album_group_key: String,
    pub album_group_title: String,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub catalog_number: Option<String>,

    // Audio properties
    pub duration_secs: u64,
    pub format: AudioFormat,
    pub bit_depth: Option<u32>,
    pub sample_rate: f64, // Changed from u32 to f64 to support fractional rates (44.1kHz = 44100Hz)
    pub channels: u8,
    pub file_size_bytes: u64,

    // CUE support
    pub cue_file_path: Option<String>,
    pub cue_start_secs: Option<f64>,
    pub cue_end_secs: Option<f64>,

    // Artwork
    pub artwork_path: Option<String>,

    // Indexing
    pub last_modified: i64,
    pub indexed_at: i64,

    // Download tracking
    pub source: Option<String>,
    pub qobuz_track_id: Option<i64>,

    /// True when the file lives on a network-backed filesystem (NFS,
    /// CIFS/SMB, SSHFS, etc.). Detected at index time by inspecting
    /// /proc/mounts. Consumed by the UI to mark the track as
    /// unreachable under forced offline mode — cable unplugged means
    /// the mount is gone even if the path string still says /home/…,
    /// which is common under Flatpak / Snap sandboxes.
    #[serde(default)]
    pub is_network_mount: bool,
}

impl Default for LocalTrack {
    fn default() -> Self {
        Self {
            id: 0,
            file_path: String::new(),
            title: String::new(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            album_artist: None,
            album_group_key: String::new(),
            album_group_title: String::new(),
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            catalog_number: None,
            duration_secs: 0,
            format: AudioFormat::Unknown,
            bit_depth: None,
            sample_rate: 44100.0, // Now f64
            channels: 2,
            file_size_bytes: 0,
            cue_file_path: None,
            cue_start_secs: None,
            cue_end_secs: None,
            artwork_path: None,
            last_modified: 0,
            indexed_at: 0,
            source: None,
            qobuz_track_id: None,
            is_network_mount: false,
        }
    }
}
