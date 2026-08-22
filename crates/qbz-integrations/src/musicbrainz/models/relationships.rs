//! Artist relationship types (members, groups, collaborators)

use serde::{Deserialize, Serialize};

/// Related artist (for relationships)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedArtist {
    pub mbid: String,
    pub name: String,
    pub role: Option<String>,
    pub period: Option<Period>,
    pub ended: bool,
}

/// Time period for a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    pub begin: Option<String>,
    pub end: Option<String>,
}

/// Artist relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistRelationships {
    pub members: Vec<RelatedArtist>,
    pub past_members: Vec<RelatedArtist>,
    pub groups: Vec<RelatedArtist>,
    pub collaborators: Vec<RelatedArtist>,
}

impl ArtistRelationships {
    pub fn empty() -> Self {
        Self {
            members: Vec::new(),
            past_members: Vec::new(),
            groups: Vec::new(),
            collaborators: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
            && self.past_members.is_empty()
            && self.groups.is_empty()
            && self.collaborators.is_empty()
    }
}
