// crates/qbzd/src/api/queue/ — routes 13-16 (02-cli-and-api.md §3.3.13-16):
// GET /api/queue, POST /api/queue/{add,remove,clear}.
//
// INDEX CONVENTION (normative, §3.3.13/§3.3.15, cross-doc-fixed): every index
// on the wire is 0-based, the SAME space as `current_index` in the
// `GET /api/queue` response. The CLI is the only place a 1-based position
// exists (§2.2's `queue list` table, `queue remove <INDEX>`) — the translation
// happens ONLY at the CLI boundary (`cli/queue.rs`), never here. This file
// speaks 0-based exclusively, straight from `QueueState.current_index`
// (`crates/qbz-models/src/playback.rs:94-103`) with no shift.
//
// 409 needs_auth is gated per-route by the §3.3.14-16 Errors columns, same
// discipline as api/playback.rs's header comment: `add` gates (needs a
// session to resolve tracks via `core.get_track`); `list`/`remove`/`clear`
// carry no needs_auth in their Errors column and act on whatever queue state
// already exists regardless of auth.
//
// Server-side materialization (brief, non-negotiable): `add` NEVER accepts a
// client-built `QueueTrack`. It resolves each `track_id` via `core.get_track`
// (the Qobuz catalog `Track`) and maps it to a `QueueTrack` with
// `track_to_queue_track` below — the same shape the desktop's single-track
// play path builds server-side (`crates/qbz/src/playback.rs:2028-2073`,
// off-limits here since `qbz` is the Slint crate; this is an independent,
// Slint-free re-derivation from `qbz_models::Track`, not a copy of that file).
//
// Response shape note on `add`: 02 §3.3.14's sketch is `{"added","total_tracks"}`.
// This handler additively includes `"tracks"`: the materialized `QueueTrack`
// objects, in request order — the same reasoning T7 used for `next`/`previous`
// returning the full landing `QueueTrack` (02 §3.3.9-10) rather than a bare id:
// the CLI's documented human line (`added: Spain – Chick Corea (next)`, §2.2)
// needs title/artist, and §1.1's one-request-per-verb rule forbids a second
// GET to fetch them. §3.1.4 explicitly allows additive fields within
// api_version 1; `total_tracks`/`added` are unchanged and still exactly what
// §3.3.14 documents.
mod add;
mod clear_reorder;
mod jump_stop_after;
mod list;
mod mapping;
mod remove;
mod shared;
#[cfg(test)]
mod tests;

pub use add::add;
pub use clear_reorder::{clear, reorder};
pub use jump_stop_after::{jump, stop_after};
pub use list::list;
pub(crate) use mapping::track_to_queue_track;
pub use remove::remove;
