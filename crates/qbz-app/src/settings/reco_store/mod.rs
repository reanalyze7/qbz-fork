//! Recommendation event store (headless, frontend-agnostic).
//!
//! Cleanroom port (ADR-006) of Tauri's `src-tauri/src/reco_store/{mod,db,helpers}.rs`
//! into `qbz-app` so the Slint frontend (and, eventually, Tauri) can drive the
//! Discover recommendation seeds without any `tauri::State` dependency. This
//! module does NOT wrap legacy — it owns the logic and runs headless.
//!
//! ## DB file shared with Tauri
//!
//! Tauri's `RecoState::init_at` opens `<base_dir>/reco/events.db`
//! (`src-tauri/src/reco_store/mod.rs:162-167`; the non-per-user path is
//! `dirs::data_dir()/qbz/reco/events.db`, see `mod.rs:140-148`). To share a
//! user's existing Tauri recommendation history cross-frontend, `new_at(base)`
//! opens the SAME file: `<base>/reco/events.db`. The schema is created with
//! `CREATE TABLE IF NOT EXISTS` (+ idempotent column/index migrations), so the
//! store coexists with a DB that Tauri already created.
//!
//! ## What is ported
//!
//! - `reco_events` schema + the `genre_id` migration (Tauri's base schema omits
//!   `genre_id` and adds it via `ALTER TABLE`; we create it inline AND keep the
//!   idempotent migration so an old Tauri DB without the column is upgraded).
//! - `reco_scores` companion table (written by `train()`, read by `get_home_seeds`).
//! - `reco_album_meta` (needed by `get_top_genres`, which LEFT JOINs it for the
//!   genre name).
//! - Event logging (`log_play_event` / `log_favorite_event` / generic `insert_event`).
//! - Read APIs: `get_recent_track_ids`, `get_recent_track_ids_since` (NEW —
//!   time-windowed, for WeeklyQ's 7-day window), `get_favorite_track_ids`,
//!   `get_top_genres`, `get_home_seeds` (mirrors `get_home_seeds_internal`).
//! - `train()` — the decay/weight scorer from Tauri's `v2_reco_train_scores`,
//!   ported verbatim (same default lookback 90d / half-life 21d / max 5000
//!   events / 200 per type, same event + item weights, same exponential decay).
//!
//! Album/artist *metadata resolution* (the 3-tier Qobuz-API cache in Tauri's
//! `helpers.rs`) is intentionally NOT ported here: it depends on the Qobuz HTTP
//! client + API cache and belongs in the frontend layer that has those. This
//! module returns IDs (seeds); the caller resolves them.

mod events;
mod home_seeds;
mod meta;
mod reads;
mod reads_agg;
mod reads_albums;
mod schema;
mod schema_migrations;
mod scores;
mod scores_write;
#[cfg(test)]
mod tests;
mod train;
mod train_entries;
mod train_weights;
mod types;

use std::time::{SystemTime, UNIX_EPOCH};

pub use meta::{create_empty_reco_store_state, RecoStoreState};
pub use schema::RecoStore;
pub use types::{
    HomeSeedLimits, HomeSeeds, RecoEventInput, RecoEventType, RecoItemType, TopArtistSeed,
    TrainParams,
};

pub(super) fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
