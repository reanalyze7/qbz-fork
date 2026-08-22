//! Self-contained Odesli/song.link client.
//!
//! Ported from `src-tauri/src/share/{songlink,models,errors}.rs`. This is a
//! frontend-agnostic copy so the resolver does not depend on the Tauri `share`
//! module. The Odesli endpoint is `https://api.song.link/v1-alpha.1/links`.

mod client;
mod error;
mod models;
mod simplified;

use std::time::Duration;

pub use client::SongLinkClient;
pub use error::ShareError;
pub use simplified::ContentType;

#[allow(unused_imports)] // wire-shape types kept reachable for fidelity/debugging
pub use models::{Entity, OdesliResponse, PlatformLink};
#[allow(unused_imports)]
pub use simplified::SongLinkResponse;

const ODESLI_API_URL: &str = "https://api.song.link/v1-alpha.1/links";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour
