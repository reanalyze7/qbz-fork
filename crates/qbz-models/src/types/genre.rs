//! Genre model + `/genre` list response types.

use serde::{Deserialize, Serialize};

/// Genre model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: u64,
    pub name: String,
    /// Full ancestor id chain (top-level first, self last) as sent by the
    /// discover endpoints. Absent on older cached payloads → None.
    #[serde(default)]
    pub path: Option<Vec<u64>>,
}

/// Genre info with full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreInfo {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub path: Option<Vec<u64>>,
}

/// Genre list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreListResponse {
    pub genres: GenreListContainer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreListContainer {
    pub items: Vec<GenreInfo>,
}
