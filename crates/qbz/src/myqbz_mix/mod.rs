//! My QBZ — DJ-mix "Random queue" sampler (Phase-2 Slice 10).
//!
//! The Rust side of the [`crate::MyQbzMixState`] modal (spec 21 §C / spec 12
//! §12). Replaces Tauri's `v2_collection_unique_track_count` +
//! `v2_collection_shuffle_tracks` commands (spec 40 §6) with direct calls into
//! the shared `qbz-mixtape` shuffle pipeline + the `qbz-core` queue — no Tauri
//! wrappers (ADR-005/006).
//!
//! ## Data path
//!
//! - **On open** ([`open`]): resolve the collection's items **in-order**
//!   (`play_mode` is ignored for DJ-mix, spec §4.B — always InOrder) via the
//!   shared resolver reused from [`crate::myqbz_play::resolve_collection`], then
//!   run the DETERMINISTIC [`qbz_mixtape::shuffle::unique_track_count`] (no RNG)
//!   to get the slider's max. The discrete size set is [`build_size_options`]
//!   (50,100,150,…,All(N)); the slider indexes into it. A "Loading…" state shows
//!   while resolving.
//! - **On shuffle** ([`shuffle`]): re-resolve in-order, then — in a SYNC scope
//!   that ends BEFORE any `.await` — run [`qbz_mixtape::shuffle::dedup_by_similarity`]
//!   + [`qbz_mixtape::shuffle::hybrid_sample`] with `rand::rng()` (the thread
//!   RNG). The sampled queue is then handed to
//!   [`crate::myqbz_play::play_all_tracks`] (replace + start at 0 + stamp the
//!   queue-source collection + best-effort `touch_play`). When the sampled
//!   `actual` count is below the requested size (the per-album cap can shrink the
//!   pool, spec §9.16) a "Playing N of M" info toast is shown.
//!
//! ## RNG confinement (load-bearing, spec 40 §6)
//!
//! `rand::rng()` returns a `ThreadRng`, which is `!Send`. Holding it across an
//! `.await` would make the spawned future non-`Send` and fail to compile. So the
//! resolve (`.await`) happens FIRST into an owned `Vec<QueueTrack>`; the
//! dedup+sample then run inside a plain synchronous block `{ … }` that creates,
//! uses, and DROPS the `ThreadRng` entirely before the next `.await`
//! (`play_all_tracks`). The RNG never crosses an await point.

mod open_close;
mod options;
mod shuffle;

pub use open_close::{close, open};
pub use options::apply_index;
pub use shuffle::shuffle;

use std::sync::Arc;

use crate::adapter::SlintAdapter;
use qbz_app::shell::AppRuntime;

pub(super) type Runtime = Arc<AppRuntime<SlintAdapter>>;
