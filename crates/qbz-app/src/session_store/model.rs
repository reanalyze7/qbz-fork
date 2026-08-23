use serde::{Deserialize, Serialize};

fn default_streamable() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedQueueTrack {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u64,
    pub artwork_url: Option<String>,
    #[serde(default)]
    pub hires: bool,
    pub bit_depth: Option<u32>,
    pub sample_rate: Option<f64>,
    #[serde(default)]
    pub is_local: bool,
    pub album_id: Option<String>,
    pub artist_id: Option<u64>,
    #[serde(default = "default_streamable")]
    pub streamable: bool,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub parental_warning: bool,
    #[serde(default)]
    pub source_item_id_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedPlaybackSession {
    pub queue_tracks: Vec<PersistedQueueTrack>,
    pub current_index: Option<usize>,
    pub current_position_secs: u64,
    pub volume: f32,
    pub shuffle_enabled: bool,
    pub repeat_mode: String,
    pub was_playing: bool,
    pub saved_at: i64,
}

impl Default for PersistedPlaybackSession {
    fn default() -> Self {
        Self {
            queue_tracks: Vec::new(),
            current_index: None,
            current_position_secs: 0,
            volume: 0.75,
            shuffle_enabled: false,
            repeat_mode: "off".to_string(),
            was_playing: false,
            saved_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedShellViewState {
    #[serde(default = "default_last_view")]
    pub last_view: String,
    #[serde(default)]
    pub view_context_id: Option<String>,
    #[serde(default)]
    pub view_context_type: Option<String>,
}

fn default_last_view() -> String {
    "home".to_string()
}

impl Default for PersistedShellViewState {
    fn default() -> Self {
        Self {
            last_view: "home".to_string(),
            view_context_id: None,
            view_context_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PersistedSessionSnapshot {
    pub playback: PersistedPlaybackSession,
    pub shell_view: PersistedShellViewState,
}
