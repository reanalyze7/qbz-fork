use serde::{Deserialize, Serialize};

/// Remote metadata provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RemoteProvider {
    MusicBrainz,
    Discogs,
}

impl std::fmt::Display for RemoteProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MusicBrainz => write!(f, "musicbrainz"),
            Self::Discogs => write!(f, "discogs"),
        }
    }
}

impl std::str::FromStr for RemoteProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "musicbrainz" | "mb" => Ok(Self::MusicBrainz),
            "discogs" => Ok(Self::Discogs),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}
