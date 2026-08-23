use super::types::{ContentSource, ContextType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackContext {
    #[serde(rename = "type")]
    pub context_type: ContextType,
    pub id: String,
    pub label: String,
    pub source: ContentSource,
    pub track_ids: Vec<u64>,
    pub current_position: usize,
}

impl PlaybackContext {
    pub fn new(
        context_type: ContextType,
        id: String,
        label: String,
        source: ContentSource,
        track_ids: Vec<u64>,
        start_position: usize,
    ) -> Self {
        Self {
            context_type,
            id,
            label,
            source,
            track_ids,
            current_position: start_position,
        }
    }

    pub fn next_track_id(&self) -> Option<u64> {
        let next_pos = self.current_position + 1;
        if next_pos < self.track_ids.len() {
            self.track_ids.get(next_pos).copied()
        } else {
            None
        }
    }

    pub fn upcoming_track_ids(&self, count: usize) -> Vec<u64> {
        let start_pos = self.current_position + 1;
        self.track_ids
            .iter()
            .skip(start_pos)
            .take(count)
            .copied()
            .collect()
    }

    pub fn advance(&mut self) -> bool {
        let next_pos = self.current_position + 1;
        if next_pos < self.track_ids.len() {
            self.current_position = next_pos;
            true
        } else {
            false
        }
    }

    pub fn has_next(&self) -> bool {
        self.current_position + 1 < self.track_ids.len()
    }

    pub fn total_tracks(&self) -> usize {
        self.track_ids.len()
    }

    pub fn display_info(&self) -> String {
        let type_str = match self.context_type {
            ContextType::Album => "Album",
            ContextType::Playlist => "Playlist",
            ContextType::ArtistTop => "Artist Top Songs",
            ContextType::LabelTop => "Label Top Songs",
            ContextType::HomeList => "Home List",
            ContextType::DailyQ => "DailyQ",
            ContextType::WeeklyQ => "WeeklyQ",
            ContextType::FavQ => "FavQ",
            ContextType::TopQ => "TopQ",
            ContextType::Favorites => "Favorites",
            ContextType::LocalLibrary => "Local Library",
            ContextType::Radio => "Radio",
            ContextType::Search => "Search Results",
        };
        format!("{} · {}", type_str, self.label)
    }
}
