//! Statics owned by the now-playing metadata sync, plus their accessors.

/// De-dupe guard for the desktop "now playing" notification: `refresh_now_playing_meta`
/// runs on resume/seek too, so we de-dupe to only notify on an actual track
/// change. `u64::MAX` = "nothing notified yet" (no real track id collides).
pub(super) static NOTIFY_LAST_TRACK: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(u64::MAX);

/// User gate for desktop "now playing" notifications (Settings › Appearance ›
/// System Notifications). Seeded from `ui_prefs.system_notifications` at startup
/// and flipped live by the toggle. Default ON. Read off the poll thread, so an
/// atomic (not the UI-thread AppearanceState) is the source of truth here.
pub static NOTIFICATIONS_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// Last `(track id, resolved art URL)` pushed to the OS media controls'
/// metadata. `refresh_now_playing_meta` re-runs on resume/seek/quality-patch,
/// so metadata is only re-pushed when this key actually changes — the
/// track-id dedupe extended to the art field (B11). `None` = nothing pushed
/// yet / cleared.
pub(super) static MPRIS_LAST_META: std::sync::Mutex<Option<(u64, Option<String>)>> =
    std::sync::Mutex::new(None);

/// F25 hydration cache (#638 fix 1c): catalog max fetched at play time for
/// tracks queued WITHOUT catalog params (`track_item_to_queue` leaves both
/// fields None on every search-surface play). `HYDRATED_TRACK_ID` keys the
/// two values: a `refresh_now_playing_meta` re-run for the same track
/// (resume/seek re-entry) reuses them instead of re-fetching, and a
/// different track ignores them. Rate in Hz, 0 = unknown (same conventions
/// as TRACK_MAX_*).
pub(super) static HYDRATED_TRACK_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub(super) static HYDRATED_RATE_HZ: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);
pub(super) static HYDRATED_BITS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The hydrated catalog params for `track_id`, if the F25 hydration already
/// ran for it — `(bit_depth, sample_rate)` in the `QueueTrack` conventions
/// (rate as f64, Hz here; None = unknown / hydrated for another track).
pub(super) fn hydrated_catalog_quality(track_id: u64) -> (Option<u32>, Option<f64>) {
    // Acquire pairs with the hydration task's Release store of the id, so a
    // matching id guarantees the value stores before it are visible.
    if HYDRATED_TRACK_ID.load(std::sync::atomic::Ordering::Acquire) != track_id {
        return (None, None);
    }
    let bits = HYDRATED_BITS.load(std::sync::atomic::Ordering::Relaxed);
    let rate = HYDRATED_RATE_HZ.load(std::sync::atomic::Ordering::Relaxed);
    ((bits > 0).then_some(bits), (rate > 0).then_some(rate as f64))
}

/// Compare-and-record the MPRIS metadata dedupe key. Returns `true` when
/// `key` differs from the last pushed value (→ caller pushes now), recording
/// it as the new last-pushed value. A poisoned lock falls back to pushing.
pub(super) fn mpris_meta_changed(key: &(u64, Option<String>)) -> bool {
    match MPRIS_LAST_META.lock() {
        Ok(mut last) => {
            if last.as_ref() == Some(key) {
                false
            } else {
                *last = Some(key.clone());
                true
            }
        }
        Err(_) => true,
    }
}
