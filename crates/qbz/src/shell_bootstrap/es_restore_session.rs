use crate::*;

/// Session persistence: restore the last queue + current track PAUSED
/// (gated on `persist_session`). No audio is loaded — playback stays
/// stopped until the user hits play.
pub(crate) async fn es_restore_session(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    // Session persistence: restore the last queue + current track PAUSED (gated
    // on `persist_session`). set_queue_with_order emits QueueUpdated so the queue
    // sidebar repaints itself; the now-playing bar reads current_track, so we
    // refresh its metadata explicitly. No audio is loaded — playback stays
    // stopped until the user hits play (Phase B then seeks to the saved
    // position when `resume_playback_position` is on).
    if crash_chain_level() >= 3 {
        // Crash-chain level >=3: two consecutive starts died even after the
        // view-restore reset — bypass the queue restore for THIS boot only
        // (the persisted queue stays on disk; a healthy boot restores it).
        log::warn!("[crash-chain] session-persist queue restore bypassed this boot");
    } else if session_persist::restore(&runtime).await {
        playback::refresh_now_playing_meta(&runtime, &weak).await;
        // Repaint the queue sidebar/list — set_queue_with_order emits
        // QueueUpdated, but the queue UI repaints from explicit refreshes.
        playback::refresh_sidebar(true);
        // Seed the seek bar + timers to the resume position so they show it
        // immediately (refresh_now_playing_meta above reset them to 0; the poll
        // loop only catches up once playback starts). Peeks — the actual resume
        // still fires on first play.
        //
        // KNOWN ISSUE / NEEDS WORK: this seed does NOT visibly stick — at rest
        // the bar + timer still read 0:00 and only jump to the resume position
        // once the user presses play (the audio resume itself works correctly).
        // Something repaints NowPlayingState position/progress back to 0 after
        // this runs (a later refresh_now_playing_meta closure, the poll loop's
        // idle tick reporting position 0 while no audio is loaded, or the bar
        // binding not reflecting a paused non-loaded position). Left as-is on
        // purpose — revisit the pre-play seek-bar seed for paused restore.
        let resume_pos = session_persist::pending_resume_position();
        if resume_pos > 0 {
            if let Some(track) = runtime.core().current_track().await {
                let dur = track.duration_secs;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    playback::seed_seek_display(&w, resume_pos, dur);
                });
            }
        }
    }
}
