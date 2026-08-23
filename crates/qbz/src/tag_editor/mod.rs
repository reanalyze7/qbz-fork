//! Tag editor controller (Slint) — local album metadata.
//!
//! Opens from the local album detail's edit pencil, edits album + per-track
//! fields, and persists via sidecar (default) or opt-in direct file-write
//! (one-time confirm, blocked for CUE albums). The DB index is updated in the
//! same transaction. All DB / lofty work runs on `spawn_blocking`; the rfd
//! confirm runs async. Remote MusicBrainz/Discogs lookup is a follow-up.

mod open;
mod refresh;
mod remote;
mod remote_search;
mod save;
mod save_build;
mod save_index;
mod save_payload;
mod save_write;

use std::sync::atomic::AtomicU64;

pub use open::{close_tag_editor, open_tag_editor};
pub use remote::{apply_remote, open_in_browser, select_result};
pub use remote_search::search_remote;
pub use save::save_tags;

/// kv key for the direct-write one-time acknowledgement (replaces the Tauri
/// localStorage flag; cross-compat not required for an ack bit).
const ACK_KEY: &str = "localLibrary.tagEditor.directWriteAcknowledged";

/// Save generation — a newer save supersedes a slow one on apply.
static SAVE_GEN: AtomicU64 = AtomicU64::new(0);

/// Parse the year input (trim; empty => None = clear; 0..=3000 allowed).
fn parse_year(s: &str) -> Result<Option<u32>, ()> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    match t.parse::<i64>() {
        Ok(y) if (0..=3000).contains(&y) => Ok(Some(y as u32)),
        _ => Err(()),
    }
}

/// Lenient u32 parse for track/disc numbers (empty/invalid => None).
fn parse_num(s: &str) -> Option<u32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<u32>().ok()
}
