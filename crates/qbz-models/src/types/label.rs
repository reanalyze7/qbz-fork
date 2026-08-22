//! Label model + `/label/page` and `/label/get*` response types.

use serde::{Deserialize, Serialize};

/// Label model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
}

/// Top-level response from /label/page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelPageData {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub image: Option<serde_json::Value>,
    #[serde(default)]
    pub releases: Option<Vec<LabelPageContainer>>,
    #[serde(default)]
    pub playlists: Option<LabelPageGenericList>,
    #[serde(default)]
    pub top_tracks: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub top_artists: Option<LabelPageGenericList>,
}

/// A container within label page (e.g. releases category)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelPageContainer {
    pub id: Option<String>,
    pub data: Option<LabelPageGenericList>,
}

/// Generic list with has_more and items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelPageGenericList {
    pub has_more: Option<bool>,
    pub items: Option<Vec<serde_json::Value>>,
}

/// Response from /label/explore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelExploreResponse {
    pub has_more: Option<bool>,
    pub items: Option<Vec<serde_json::Value>>,
}

// ============ Label Sub-resource Types (v9.7.0.3 API) ============
//
// The label page (/label/page) returns an aggregated snapshot; the
// getAlbums / getPlaylists / getTopArtists / getNextReleases /
// getAwardedReleases endpoints return paginated lists for each
// sub-resource. Per Qobuz convention these use the V2 list envelope
// { has_more, items: [...] }. Deserialized shapes are best-effort: if
// the server wraps items in e.g. { albums: { items: ... } }, the
// Optional fallbacks still keep the call non-fatal.

/// Generic paginated response from /label/get* endpoints.
///
/// `T` is typed (`Album`, `Playlist`, `Artist`) but all fields are
/// tolerant so the same struct works if the server returns a bare
/// items list or a nested one.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelListPage<T> {
    #[serde(default)]
    pub has_more: Option<bool>,
    #[serde(default = "Vec::new")]
    pub items: Vec<T>,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Response from /label/story.
///
/// Shape inferred from `c30/b.java` — returns editorial / story content
/// for a label. Actual fields beyond the label identity are not fully
/// known; everything past `id` / `name` / `description` is kept open.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStoryResponse {
    pub id: Option<u64>,
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub story: Option<String>,
    #[serde(default)]
    pub image: Option<serde_json::Value>,
    #[serde(default)]
    pub has_more: Option<bool>,
    #[serde(default)]
    pub items: Option<Vec<serde_json::Value>>,
}

/// Response from /label/getList (POST). Bulk lookup that hydrates
/// label metadata for a set of label IDs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelGetListResponse {
    #[serde(default = "Vec::new")]
    pub labels: Vec<Label>,
    /// Fallback for unknown envelope shape — preserved as raw JSON if
    /// the server wraps differently than expected.
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}
