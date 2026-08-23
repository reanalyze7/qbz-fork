//! Session state for the playlist "Suggested Songs" section: the live pool +
//! pagination for the currently-open playlist, held behind a process-global
//! `Mutex`. Every other submodule reads/writes `SESSION` — this is the
//! central piece of shared mutable state for the whole controller.

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

/// Which fetch we are running — drives the merge-vs-replace + error handling.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Phase {
    /// First load for the open playlist: replaces the pool, surfaces errors.
    Initial,
    /// Background pool growth (cycle-wrap load-more / variety): merges, silent.
    Merge,
}

/// The live suggestions session for the open playlist. Held in Rust (the UI
/// only ever sees the projected rows + flags on `PlaylistSuggestionsState`).
#[derive(Default)]
pub(super) struct Session {
    /// Open playlist id (Qobuz catalog id). 0 = no active session.
    pub(super) playlist_id: u64,
    /// Seed artists sent to the engine — stable across load-more within a
    /// session (Svelte: the `artists` prop only recomputes on track change).
    pub(super) artists: Vec<(Option<u64>, String)>,
    /// Track ids already in the playlist (excluded from suggestions). Grows as
    /// the user adds suggested tracks.
    pub(super) exclude_ids: HashSet<u64>,
    /// `title|artist` keys of existing playlist tracks (de-dupe vs the playlist).
    pub(super) existing_keys: HashSet<String>,
    /// The full fetched pool (de-duped on merge by id).
    pub(super) pool: Vec<qbz_reco::SuggestedTrack>,
    /// Current visible page (0-based; window of VISIBLE_COUNT).
    pub(super) page: usize,
    /// How many full cycles through the pages the user has completed.
    pub(super) completed_cycles: usize,
    /// True once the first fetch has returned.
    pub(super) loaded_once: bool,
    /// A foreground (initial) fetch is in flight.
    pub(super) loading: bool,
    /// A background pool expansion (load-more / variety) is in flight.
    pub(super) loading_more: bool,
    /// True once a MAX_POOL request has been issued — prevents auto-expand from
    /// looping when the engine returns fewer than MAX_POOL tracks.
    pub(super) max_requested: bool,
}

pub(super) static SESSION: LazyLock<Mutex<Session>> = LazyLock::new(|| Mutex::new(Session::default()));
