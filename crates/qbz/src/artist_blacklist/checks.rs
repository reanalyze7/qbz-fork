//! Fail-open predicates: id checks, the row/queue stamp, and the enabled flag.

use super::lifecycle::with_service;

/// True when the artist id is blacklisted (and the feature is enabled).
/// Fail-open `false` when no session is bound.
pub fn is_blacklisted(artist_id: u64) -> bool {
    with_service(false, |s| s.is_blacklisted(artist_id))
}

/// True when the string-form artist id parses and is blacklisted. Non-numeric
/// ids (local artists) are never blacklisted. For row code that carries string
/// ids.
pub fn is_blacklisted_id_str(artist_id: &str) -> bool {
    let Ok(id) = artist_id.parse::<u64>() else {
        return false;
    };
    is_blacklisted(id)
}

/// True when the album id is blocked (and the feature is enabled). Album ids
/// are alphanumeric strings; an empty id never matches. Fail-open `false` when
/// no session is bound. Orthogonal to artist blocking — an album is hidden by
/// its OWN id regardless of its artist.
pub fn is_album_blacklisted(album_id: &str) -> bool {
    if album_id.is_empty() {
        return false;
    }
    with_service(false, |s| s.is_album_blacklisted(album_id))
}

/// Card-grid predicate: `true` when an album-card-shaped row (string album id +
/// string primary-artist id) should be hidden from a grid/carousel. Honors the
/// enabled gate + no-session fail-open via the underlying checks. Album axis
/// (own id) OR artist axis (primary-artist id). Use for the read-only album
/// grids that map an `AlbumCard`-like row rather than a typed `Album`.
pub fn card_blacklisted(album_id: &str, artist_id: &str) -> bool {
    is_album_blacklisted(album_id) || is_blacklisted_id_str(artist_id)
}

/// Stamp value for a `TrackItem.is-blacklisted` cell (Task 6). The single
/// rule every in-scope track controller (album / playlist / favorites / the
/// four Q-mixes) reuses so render and the Task 7 queue filters agree on what
/// "blacklisted" means per row:
///
/// - **HARD local guard** — a non-Qobuz `source` is NEVER blacklisted
///   (Codex guardrail; local copies with a numeric Qobuz id must still stay
///   playable). `qobuz_download` rows render `source == "qobuz"`, so they are
///   treated as Qobuz here — that matches Tauri (VTL keys on `!isLocal`).
/// - Resolve the artist from the candidate string ids in order; the first
///   non-empty, numeric, blacklisted id wins (D-FEAT: performer OR composer;
///   album rows that lack a performer fall back to the album's primary artist).
/// - Missing / zero / non-numeric ids => fail-open (`false`).
///
/// The enabled-flag gate and the no-session fail-open live in
/// [`is_blacklisted`], so this never blocks when the feature is off or no
/// session is bound.
///
/// Live re-stamp contract (Step B): there is no change-notify here (the
/// fav_cache pattern). Every in-scope controller already calls this at LOAD
/// time, so navigating to a view always shows correct state. To refresh the
/// CURRENTLY-loaded lists after a blacklist mutation (Task 9 artist toggle /
/// Task 11 manager), the mutation site re-runs that controller's existing
/// reload / re-push path (which re-invokes `stamp_row` per row) — same as how
/// favorites re-push after a `fav_cache` change. There is intentionally no
/// global listener/observer.
pub fn stamp_row(source: &str, artist_ids: &[&str], album_id: Option<&str>) -> bool {
    // Local / ephemeral rows are protected — never blacklisted.
    if source != "qobuz" {
        return false;
    }
    // Album axis (orthogonal): the row's own album id being blocked drops it
    // regardless of artist. Then the artist axis (performer/composer/primary).
    if album_id.is_some_and(is_album_blacklisted) {
        return true;
    }
    artist_ids.iter().any(|id| is_blacklisted_id_str(id))
}

/// THE single queue/playback predicate (Task 7). Returns `true` when this track
/// should be DROPPED from any play / shuffle / queue-next / queue-later / radio
/// builder. Implemented in terms of the exact same source-guard + per-id check
/// as [`stamp_row`] so the queue filter and the row greyout can NEVER diverge:
/// a row that greys out is the row that drops from the queue, and vice versa.
///
/// - **HARD local guard** — a non-Qobuz `source` is NEVER blacklisted
///   (local copies with a numeric Qobuz id stay playable). Delegates to
///   [`stamp_row`]'s guard by passing `source` through.
/// - D-FEAT: performer OR composer — pass both numeric ids; either one being
///   blacklisted drops the track.
/// - Missing / `None` ids => fail-open (`false`), so id-less tracks always play.
///
/// The enabled-flag gate + the no-session fail-open live in [`is_blacklisted`],
/// so this returns `false` when the feature is off or no session is bound.
pub fn is_track_blacklisted(
    source: &str,
    performer_id: Option<u64>,
    composer_id: Option<u64>,
    album_id: Option<&str>,
) -> bool {
    // Reuse stamp_row's guard + checks by funneling the numeric ids through the
    // same string path — the SINGLE underlying predicate shared with rendering.
    // The album id (a blocked album hides all its tracks) is passed straight
    // through, so render greyout and queue-drop can never diverge.
    let performer = performer_id.map(|id| id.to_string()).unwrap_or_default();
    let composer = composer_id.map(|id| id.to_string()).unwrap_or_default();
    stamp_row(source, &[performer.as_str(), composer.as_str()], album_id)
}

/// True when the blacklist feature is enabled. Default-enabled (`true`) when no
/// session is bound.
pub fn is_enabled() -> bool {
    with_service(true, |s| s.is_enabled())
}
