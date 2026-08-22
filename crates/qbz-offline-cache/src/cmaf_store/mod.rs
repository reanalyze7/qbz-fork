//! On-disk layout and I/O for v2 CMAF-bundle offline cache entries.
//!
//! Layout under `<offline_root>/tracks-cmaf/<track_id>/`:
//!
//! ```text
//! init.mp4        — init segment (unencrypted container + FLAC header)
//! segments.bin    — concatenated encrypted audio segments (s=1..=n)
//! manifest.json   — small recovery manifest (segment offsets + n_segments)
//! ```
//!
//! The `manifest.json` is a belt-and-suspenders convenience: SQLite is the
//! authoritative source of truth, but if the DB is ever lost we can still
//! tell the caller how to slice `segments.bin` back into per-segment
//! buffers from the manifest. It's cheap to write and cheap to read.
//!
//! Everything here is intentionally I/O-only — no network, no crypto. The
//! CMAF download itself happens in qbz-qobuz; this module just persists
//! the bytes in a format the playback path can read back efficiently.
//!
//! Split into `layout` (pure path/naming), `write` (persist), and `read`
//! (read-back + decrypt) — see each submodule's own doc comment.

mod layout;
mod read;
mod write;

pub use layout::{BundleLayout, BundleManifest};
pub use read::{read_bundle, remove_bundle, LoadedBundle};
pub use write::persist_bundle;
