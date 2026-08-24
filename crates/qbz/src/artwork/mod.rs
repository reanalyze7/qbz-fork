//! Album artwork pipeline.
//!
//! Cover images go through the shared QBZ image cache (`qbz_cache`), the
//! same disk cache the Tauri app uses — covers are never re-downloaded
//! once cached. Fetch and decode run off the UI thread; each decoded
//! cover is applied to its own `AlbumCardItem` row on the Slint event
//! loop, so artwork arriving never resets a list (POC ADR 16 and 18).

mod apply;
mod cache;
mod decode;
mod fetch;
mod jobs;
mod target;

pub use cache::{
    cached_file_url_for, cached_path_for, open_cache, open_cache_blocking, set_shared_cache,
    shared_cache, spawn_evict, ImageCache, MAX_CACHE_BYTES,
};
pub use decode::{decode_local_pixels, decoded_pixels, header_tint, load_local_cover, pixels_to_image, DecodedPixels};
pub use fetch::{fetch_and_decode, fetch_and_decode_ref};
pub use jobs::{
    pinned_artwork_jobs, scaled_decode, set_ui_scale_factor, spawn_loads, spawn_local_loads,
    spawn_search_loads, ArtworkJob,
};
pub use target::ArtworkTarget;
