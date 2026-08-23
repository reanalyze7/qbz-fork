//! Discover > For You controller.
//!
//! Loads the personalized For You sections and pushes them into
//! `ForYouState`. Each section reuses an existing card component (album
//! Carousel, SlimCarousel, artist ArtistCarousel).
//!
//! ## Progressive, parallel loading
//!
//! The tab loads once on first open ([`spawn_for_you`]). Rather than
//! awaiting one long sequential chain of API calls and applying every
//! section at the very end (the old behaviour — up to ~9 serialized
//! round-trips before anything painted), the loader now:
//!
//!   1. Paints the local/static sections instantly (recently-played
//!      tracks + albums) before any network call.
//!   2. Fans the independent API calls out into concurrent branches
//!      (release-watch ∥ favorite-artists ∥ favorite-albums ∥
//!      album-suggest), each applying its own section the moment its
//!      data resolves, via `upgrade_in_event_loop`.
//!   3. Latches `ForYouState.loaded = true` ONLY after every branch has
//!      resolved, so the one-shot re-entry guard in `main.rs`
//!      (`ensure_for_you_loaded`) can never strand a partially-loaded
//!      tab.
//!
//! Backed sections: Release Watch (get_release_watch), Recently Played
//! Tracks / Albums (local play-history), Your Top Artists (favorites),
//! Artists to Follow (similar artists seeded from favorites), Rediscover
//! (favorite albums), More From Your Library (album/suggest),
//! Spotlight (a rotated favorite artist's page).

mod apply_misc;
mod apply_sections;
mod build;
mod build_albums;
mod fetch;
mod follow;
mod jobs;
mod mappers;
mod models;
mod orchestrator;
mod spotlight;

pub(crate) use build::top_artist_cards;
pub(crate) use build_albums::most_played_album_cards;
pub(crate) use build_albums::favorite_album_cards;
pub(crate) use fetch::fetch_release_watch;
pub(crate) use mappers::{artist_items, section};
pub use models::{AlbumCard, ArtistSlim, SpotlightData, TrackSlim};
pub use orchestrator::{reset_loading, spawn_for_you};

const ARTIST_SEEDS: usize = 4;
const SIMILAR_PER_SEED: u32 = 10;
const FOLLOW_MAX: usize = 18;
