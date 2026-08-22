//! Artist image metadata model

use serde::{Deserialize, Serialize};

/// Information about an artist's image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistImageInfo {
    pub artist_name: String,
    pub image_url: Option<String>,
    pub source: Option<String>,
    pub custom_image_path: Option<String>,
    pub canonical_name: Option<String>,
}
