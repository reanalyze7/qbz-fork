//! Tidal playlist import (OpenAPI v2)

mod auth;
mod duration;
mod playlist;
mod track_ids;
mod tracks;
mod tracks_map;
mod url;

#[cfg(test)]
mod tests;

pub use playlist::fetch_playlist;
pub use url::{detect_resource, parse_playlist_id};

const RATE_LIMIT_DELAY_MS: u64 = 200; // Delay between API calls to avoid 429
const TIDAL_API_BASE: &str = "https://openapi.tidal.com/v2";
const DEFAULT_COUNTRY_CODE: &str = "US";
