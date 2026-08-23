//! My QBZ — Collection / Mixtape DETAIL **playback** (Phase-2 Slice 5).
//!
//! Wires the detail view's hero Play / Shuffle CTAs, the per-row Play action,
//! and the per-row context menu (play / play-next / add-to-queue) to the
//! shared `qbz-mixtape` ENQUEUE resolver, then drives the already-shared
//! `qbz-core` queue + `qbz-app` `RuntimeManager` queue-source stamp.
//!
//! Behavior is 1:1 with Tauri's `v2_enqueue_collection` /
//! `v2_enqueue_collection_item` (spec 40 §5/§6, gotchas §9):
//! - **Resolve all** uses the collection's persisted `play_mode`; the hero
//!   Shuffle forces `AlbumShuffle` ordering (time-seeded, whole-item shuffle).
//! - Failed items are logged + skipped (partial playback > total failure) —
//!   that is `resolve_collection_tracks`' own contract; the per-item path
//!   mirrors it manually.
//! - `play_next` inserts in **REVERSE** so the first resolved track lands
//!   immediately after the current track.
//! - The queue-source-collection stamp is set **only on replace** (hero
//!   play/shuffle + per-row replace-play); append/play_next preserve context.
//! - `touch_play` is best-effort and runs **only** on the whole-collection
//!   replace paths (hero play/shuffle), never per-row.
//!
//! Frontend-agnostic (ADR-005/006): the `qbz-mixtape` crate holds all the
//! resolution logic; this module only builds a `ProdItemResolver` (Qobuz client
//! + a `Send + Sync` local closure that runs `with_db` synchronously — no
//! `&LibraryDatabase` is ever held across an `.await`) and applies the result
//! to the queue.

use std::sync::Arc;

use qbz_models::mixtape::MixtapeCollectionItem;
use qbz_models::QueueTrack;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

mod bulk;
mod bulk_ids;
mod hero;
mod hero_shuffle;
mod inline_track;
mod load;
mod resolve;
mod resolve_item;
mod row_actions;
mod skip;

pub use bulk::bulk_enqueue;
pub use bulk_ids::resolve_bulk_qobuz_track_ids;
pub use hero::play_all;
pub use hero_shuffle::shuffle;
pub use inline_track::play_inline_track;
pub(crate) use load::load_collection;
pub use row_actions::{item_action, play_item};
pub use skip::{skip_to_next_item, skip_to_previous_item};

pub(crate) use hero::play_all_tracks;
pub(crate) use resolve::resolve_collection;
pub(crate) use resolve_item::fetch_item_tracks;

/// Convenience alias for the runtime handle threaded through every call
/// (mirrors `playback::Runtime`).
type Runtime = Arc<AppRuntime<SlintAdapter>>;

/// The per-row context-menu mode parsed from the Slint `action` string.
enum RowMode {
    /// Replace-play this single item (queue + start at 0). No queue-source
    /// stamp, no `touch_play` (per-row action, not "play the whole collection").
    Play,
    /// Insert the item's resolved tracks immediately after the current track.
    PlayNext,
    /// Append the item's resolved tracks at the end of the queue.
    AddToQueue,
}

impl RowMode {
    fn parse(action: &str) -> Option<Self> {
        match action {
            "play" => Some(Self::Play),
            "play-next" | "play_next" => Some(Self::PlayNext),
            "add-to-queue" | "add_to_queue" | "append" => Some(Self::AddToQueue),
            _ => None,
        }
    }
}

/// The synchronous local-item resolver closure handed to `ProdItemResolver`.
///
/// `with_db` is synchronous (it opens the per-user `library.db` fresh on the
/// current blocking thread), so `&LibraryDatabase` never crosses an `.await`.
/// Error semantics are preserved: the crate's `resolve_local_item` error (the
/// load-bearing user-meaningful messages,
/// the local-playlist hard error) is surfaced verbatim so
/// `resolve_collection_tracks` logs + skips the item exactly as it would for a
/// Qobuz failure. A DB-open failure becomes its own `Err` string (the item is
/// then skipped too, not silently dropped as success).
fn resolve_local(item: &MixtapeCollectionItem) -> Result<Vec<QueueTrack>, String> {
    // with_db -> Option<Result<.., String>>: Some(inner) when the DB opened
    // (inner carries the resolver's own Ok/Err); None when the DB could not be
    // opened at all. We map None to an Err so the item is skipped, not treated
    // as an empty success.
    crate::library_db::with_db(|db| Ok(qbz_mixtape::enqueue::resolve_local_item(db, item)))
        .unwrap_or_else(|| Err("library database unavailable".to_string()))
}
