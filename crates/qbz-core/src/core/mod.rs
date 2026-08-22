//! QBZ Core Orchestrator
//!
//! The main orchestrator that connects all QBZ subsystems and provides
//! a unified API for frontends. Split by domain (see each submodule's
//! doc comment); every `impl<A: FrontendAdapter + Send + Sync + 'static>
//! QbzCore<A>` block below is one of several such blocks across this
//! module tree — legal in Rust since they all live in the same crate.

use std::sync::Arc;
use tokio::sync::RwLock;

use qbz_integrations::musicbrainz::cache::MusicBrainzCache;
use qbz_integrations::musicbrainz::MusicBrainzClient;
use qbz_models::FrontendAdapter;
use qbz_player::{Player, QueueManager};
use qbz_qobuz::QobuzClient;

mod auth;
mod events;
mod favorite_ids;
mod favorites;
mod filters;
mod helpers;
mod labels;
mod login;
mod musicbrainz;
mod playback;
mod playlists;
mod queue;
mod search;
mod streaming;
mod setup;

pub use filters::{album_blacklisted, discover_album_blacklisted, track_blacklisted};
pub(crate) use filters::parse_search_all;
pub use helpers::normalize_artist_name;

/// Set of blacklisted artist ids. Built per call from the live blacklist store
/// (`qbz-app`); empty only under fail-open (no session bound / feature off).
pub type BlacklistFilter = std::collections::HashSet<u64>;

/// Set of blocked album ids (Qobuz album ids are alphanumeric `String`s, so a
/// separate, parallel axis from the `u64` artist filter). An album is hidden by
/// its OWN id regardless of artist — the surgical fix for Qobuz same-name
/// artist merges. Empty under fail-open.
pub type AlbumBlacklistFilter = std::collections::HashSet<String>;

/// Core orchestrator for QBZ
///
/// This is the main entry point for any frontend (Tauri, Slint, Iced, CLI, etc.)
/// It provides a unified API and emits events through the FrontendAdapter.
pub struct QbzCore<A: FrontendAdapter> {
    /// Frontend adapter for event emission
    adapter: Arc<A>,
    /// Qobuz API client
    client: Arc<RwLock<Option<QobuzClient>>>,
    /// Queue manager
    queue: Arc<RwLock<QueueManager>>,
    /// Audio player
    player: Arc<Player>,
    /// MusicBrainz client (always present; enable/disable toggle lives inside)
    musicbrainz: Arc<MusicBrainzClient>,
    /// Persistent MB cache. Opened by the frontend (which owns the
    /// data-dir path) via `set_musicbrainz_cache`. Methods read the
    /// cache before hitting the network and persist on miss.
    musicbrainz_cache: Arc<std::sync::Mutex<Option<MusicBrainzCache>>>,
    /// Per-user artist-vector store for the playlist "Suggested Songs" engine.
    /// Opened by the frontend (owns the data dir) via `set_artist_vectors`.
    /// tokio Mutex because the suggestions engine holds it across `.await`s.
    artist_vectors: Arc<tokio::sync::Mutex<Option<qbz_reco::ArtistVectorStore>>>,
    /// Whether the core is initialized
    initialized: Arc<RwLock<bool>>,
    /// D8 guard: true when the current queue was built from an OFFLINE-ONLY
    /// local playlist — such a queue must never be pushed to the Qobuz
    /// Connect cloud. Cleared by every queue REPLACEMENT (`set_queue` /
    /// `set_queue_with_order` / `clear_queue`); append-style ops preserve it.
    /// Set explicitly by the frontend's local-playlist play path right after
    /// its `set_queue`.
    queue_offline_only: Arc<std::sync::atomic::AtomicBool>,
}

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Create a new QbzCore instance with the given frontend adapter and player
    ///
    /// The Player must be created by the frontend with appropriate audio settings.
    /// QbzCore orchestrates playback through this player.
    pub fn new(adapter: A, player: Player) -> Self {
        Self {
            adapter: Arc::new(adapter),
            client: Arc::new(RwLock::new(None)),
            queue: Arc::new(RwLock::new(QueueManager::new())),
            player: Arc::new(player),
            musicbrainz: Arc::new(MusicBrainzClient::new()),
            musicbrainz_cache: Arc::new(std::sync::Mutex::new(None)),
            artist_vectors: Arc::new(tokio::sync::Mutex::new(None)),
            initialized: Arc::new(RwLock::new(false)),
            queue_offline_only: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}
