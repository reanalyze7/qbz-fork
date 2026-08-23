//! My QBZ — custom-cover upload / remove (Phase-2 Slice 7).
//!
//! Mirrors Tauri's `v2_mixtape_upload_custom_cover` /
//! `v2_mixtape_remove_custom_cover` (spec 40 §7) 1:1, and the album
//! custom-cover convention (same artwork cache dir, same 1000×1000 Lanczos3
//! resize, same failure-safe "persist before deleting the previous file"
//! ordering):
//!
//! - **Upload:** validate the picked file's extension (png/jpg/jpeg/webp),
//!   read the previous `custom_artwork_path` (to delete after persist), decode
//!   + `resize(1000, 1000, Lanczos3)`, save as `mixtape_custom_{safe_id}_{epoch_secs}.jpg`
//!   in `qbz_library::get_artwork_cache_dir()`, persist via
//!   `repo::set_custom_artwork(Some(dest))`, then delete the previous file only
//!   if it differs.
//! - **Remove:** read previous, `repo::set_custom_artwork(None)`, delete prev.
//!
//! Frontend-agnostic (ADR-005/006): the persistence is `qbz_mixtape::repo`
//! reached directly through `crate::library_db::with_db`; no Tauri command
//! wrappers. All blocking work (file decode/resize/IO + DB) runs on a
//! `spawn_blocking` worker; the reload + toast hop back to the event loop.
//!
//! NOTE on webp: the workspace `image` crate is built with only the `jpeg` +
//! `png` features, so a `.webp` source decodes to an error at runtime (the
//! extension is accepted to match the Tauri picker filter, but the decode
//! surfaces the "upload failed" toast). png/jpg/jpeg are the working formats.

mod actions;
mod db;
mod file_ops;

pub use actions::{remove, upload};
