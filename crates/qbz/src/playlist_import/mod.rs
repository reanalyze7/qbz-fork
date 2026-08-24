//! Playlist Importer controller — the Rust side of the "Import Playlist"
//! modal (PlaylistImportModal.slint), a 1:1 port of Tauri's
//! PlaylistImportModal.svelte driven by the headless `qbz-playlist-import`
//! crate. Every interpolated string (log lines, status line, summary
//! block) is formatted HERE and pushed into PlaylistImportState
//! pre-formatted; provider detection lives here too (Slint 1.16 strings
//! have no `.contains`, so every URL keystroke round-trips through
//! `url-edited`).
//!
//! Close-mid-import semantics (spec §1.8): closing the modal never cancels
//! the tokio import task. On completion the toast + sidebar refresh still
//! fire (main.rs arm); navigation happens only while the modal is still
//! open AND the run's generation is current. [`GENERATION`] is bumped on
//! every open() and execute(), so a stale run's sink events / completion
//! can never touch a reopened modal's fresh state.

mod complete;
mod events;
mod execute;
mod fetch;
mod format;
mod open;
mod session;

pub use complete::{apply_execute_err, apply_execute_ok};
pub use events::SlintSink;
pub use execute::begin_execute;
pub use fetch::{apply_preview_err, apply_preview_ok, begin_fetch};
pub use open::{on_name_edited, on_url_edited, open};
pub use session::current_generation;
