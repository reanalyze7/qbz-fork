use crate::*;

thread_local! {
    /// Debounce timer for the header live search — restarted on every
    /// keystroke, fires the search 300 ms after typing stops.
    pub(crate) static SEARCH_DEBOUNCE: slint::Timer = slint::Timer::default();

    /// Debounce timer for the cortinilla (live dropdown) network load —
    /// restarted on every keystroke so the skeleton shows while typing and a
    /// single clean result paint lands ~220 ms after typing stops (no cached
    /// instant-paint, so results never "jump" from a cached to a fresh state).
    pub(crate) static CORTINILLA_DEBOUNCE: slint::Timer = slint::Timer::default();

    /// Snapshot of the cortinilla payload currently shown, so a
    /// `cortinilla-row-clicked(flat_index)` can resolve the flat index back to
    /// the concrete row (kind/id/source) and dispatch. UI thread only; set
    /// whenever `apply_cortinilla` writes a new payload.
    pub(crate) static LAST_CORTINILLA: std::cell::RefCell<Option<search::CortinillaData>> =
        const { std::cell::RefCell::new(None) };

    /// Snapshot of the raw `LocalTrack` rows that backed the cortinilla's "On
    /// this device" section currently shown. The click router resolves a local
    /// row (`source == "local"`) to its concrete `LocalTrack` here (the row's
    /// `id` is the library row id) so it can play through
    /// `playback::play_local_tracks`. Updated in lockstep with `LAST_CORTINILLA`
    /// whenever a cortinilla payload is applied. UI thread only.
    pub(crate) static LAST_CORTINILLA_LOCAL: std::cell::RefCell<Vec<qbz_library::LocalTrack>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Stash for the "Duplicate tracks" confirm sub-modal. Slint can't hold a
    /// `Vec<u64>` ergonomically, so when a Qobuz→Qobuz add finds duplicates we
    /// park the full context here and the DuplicateConfirmActions handlers read
    /// it back. Cleared on add-all / add-new-only / cancel. The tuple is
    /// `(playlist_id, all_track_ids, duplicate_track_ids, playlist_name)`.
    pub(crate) static DUP_CONFIRM_STASH: std::cell::RefCell<
        Option<(u64, Vec<u64>, std::collections::HashSet<u64>, String)>
    > = const { std::cell::RefCell::new(None) };
}
