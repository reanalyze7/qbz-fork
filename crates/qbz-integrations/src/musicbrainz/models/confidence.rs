//! Confidence and classification enums shared across MusicBrainz models

use serde::{Deserialize, Serialize};

/// Match confidence levels for MusicBrainz lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchConfidence {
    Exact,  // ISRC/UPC exact match
    High,   // Score >= 95
    Medium, // Score >= 80
    Low,    // Score >= 60
    None,   // No match found
}

impl MatchConfidence {
    pub fn from_score(score: Option<i32>) -> Self {
        match score {
            Some(s) if s >= 100 => Self::Exact,
            Some(s) if s >= 95 => Self::High,
            Some(s) if s >= 80 => Self::Medium,
            Some(s) if s >= 60 => Self::Low,
            _ => Self::None,
        }
    }
}

/// Artist type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtistType {
    Person,
    Group,
    Orchestra,
    Choir,
    Character,
    Other,
}

impl Default for ArtistType {
    fn default() -> Self {
        Self::Other
    }
}

impl From<Option<&str>> for ArtistType {
    fn from(s: Option<&str>) -> Self {
        match s.map(|s| s.to_lowercase()).as_deref() {
            Some("person") => Self::Person,
            Some("group") => Self::Group,
            Some("orchestra") => Self::Orchestra,
            Some("choir") => Self::Choir,
            Some("character") => Self::Character,
            _ => Self::Other,
        }
    }
}

/// Musician confidence level for MusicBrainz <-> Qobuz matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MusicianConfidence {
    Confirmed,
    Contextual,
    Weak,
    None,
}

impl MusicianConfidence {
    pub fn level(&self) -> u8 {
        match self {
            Self::Confirmed => 3,
            Self::Contextual => 2,
            Self::Weak => 1,
            Self::None => 0,
        }
    }
}

impl Default for MusicianConfidence {
    fn default() -> Self {
        Self::None
    }
}
