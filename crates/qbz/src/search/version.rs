//! Monotonic version counters guarding stale async loads.

thread_local! {
    /// Monotonic search-attempt counter. Each `navigate_search` captures the
    /// current value; a stale async load whose version is no longer current
    /// must not overwrite a newer search's results. UI thread only.
    static SEARCH_VERSION: std::cell::Cell<u64> = std::cell::Cell::new(0);

    /// Monotonic cortinilla-attempt counter — SEPARATE from `SEARCH_VERSION`.
    /// The live dropdown fires far more often than the results page (one bump
    /// per debounced keystroke + the instant cached paint), and a stale
    /// revalidation must not overwrite a newer query's dropdown. UI thread only.
    static CORTINILLA_VERSION: std::cell::Cell<u64> = std::cell::Cell::new(0);

    /// Monotonic immersive-search-attempt counter — SEPARATE from both the
    /// results-page (`SEARCH_VERSION`) and the main-header (`CORTINILLA_VERSION`)
    /// counters, so the in-immersive dropdown's stale-load guard never collides
    /// with the main cortinilla's (both surfaces can be open across a session).
    /// UI thread only.
    static IMMERSIVE_SEARCH_VERSION: std::cell::Cell<u64> = std::cell::Cell::new(0);
}

/// Bump the search version and return the new value.
pub fn next_search_version() -> u64 {
    SEARCH_VERSION.with(|c| {
        let v = c.get() + 1;
        c.set(v);
        v
    })
}

/// Whether `version` is still the most recent search attempt.
pub fn is_current_version(version: u64) -> bool {
    SEARCH_VERSION.with(|c| c.get() == version)
}

/// Bump the cortinilla version and return the new value.
pub fn next_cortinilla_version() -> u64 {
    CORTINILLA_VERSION.with(|c| {
        let v = c.get() + 1;
        c.set(v);
        v
    })
}

/// Whether `version` is still the most recent cortinilla attempt.
pub fn is_current_cortinilla_version(version: u64) -> bool {
    CORTINILLA_VERSION.with(|c| c.get() == version)
}

/// Bump the immersive-search version and return the new value.
pub fn next_immersive_search_version() -> u64 {
    IMMERSIVE_SEARCH_VERSION.with(|c| {
        let v = c.get() + 1;
        c.set(v);
        v
    })
}

/// Whether `version` is still the most recent immersive-search attempt.
pub fn is_current_immersive_search_version(version: u64) -> bool {
    IMMERSIVE_SEARCH_VERSION.with(|c| c.get() == version)
}
