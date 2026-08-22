//! Area search/lookup responses and their supporting relation types

use serde::Deserialize;

use super::artist::LifeSpan;

/// Area search response
#[derive(Debug, Deserialize)]
pub struct AreaSearchResponse {
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub areas: Vec<AreaResult>,
}

/// Single area in search results
#[derive(Debug, Deserialize)]
pub struct AreaResult {
    pub id: String,
    pub score: Option<i32>,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
    #[serde(rename = "iso-3166-1-codes", default)]
    pub iso_codes: Option<Vec<String>>,
    #[serde(rename = "life-span")]
    pub life_span: Option<LifeSpan>,
}

/// Area detail response (from /area/{id}?inc=area-rels)
#[derive(Debug, Deserialize)]
pub struct AreaDetailResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
    #[serde(rename = "iso-3166-1-codes", default)]
    pub iso_codes: Option<Vec<String>>,
    #[serde(rename = "iso-3166-2-codes", default)]
    pub iso_3166_2_codes: Option<Vec<String>>,
    #[serde(default)]
    pub relations: Option<Vec<AreaRelation>>,
}

/// Area relationship entry
#[derive(Debug, Deserialize)]
pub struct AreaRelation {
    #[serde(rename = "type")]
    pub relation_type: String,
    pub direction: Option<String>,
    pub area: Option<AreaRelTarget>,
}

/// Target area in an area relationship
#[derive(Debug, Deserialize)]
pub struct AreaRelTarget {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
    #[serde(rename = "iso-3166-1-codes", default)]
    pub iso_codes: Option<Vec<String>>,
    #[serde(rename = "iso-3166-2-codes", default)]
    pub iso_3166_2_codes: Option<Vec<String>>,
}
