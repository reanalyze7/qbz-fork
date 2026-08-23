//! Ephemeral append-to-queue + the play-or-prompt dialog gate, split out of
//! `ephemeral.rs` to keep both files under the line budget.

use super::ephemeral::ephemeral_play;
use super::queue_track::local_queue_track;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;

/// Append an ephemeral selection to the CURRENT queue (no replace).
pub fn ephemeral_enqueue(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    kind: String,
    arg: String,
) {
    handle.spawn(async move {
        let tracks = match kind.as_str() {
            "all" => crate::ephemeral::tracks_snapshot(),
            "album" => crate::ephemeral::album_tracks(&arg),
            "track" => arg
                .parse::<i64>()
                .ok()
                .and_then(crate::ephemeral::get_track)
                .into_iter()
                .collect(),
            _ => Vec::new(),
        };
        if tracks.is_empty() {
            return;
        }
        let queue: Vec<qbz_models::QueueTrack> = tracks.iter().map(local_queue_track).collect();
        runtime.core().add_tracks(queue).await;
        refresh_sidebar(true);
        crate::toast::success_weak(&weak, qbz_i18n::t("Added to queue"));
    });
}

/// Either play the ephemeral selection now, or — if a queue is already active —
/// prompt add-to-queue vs clear-and-play. Only the ephemeral pane uses this
/// (user decision 2026-06-06: ephemeral-only, dialog-on-play).
pub fn ephemeral_play_or_prompt(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    kind: String,
    arg: String,
) {
    let rt = runtime.clone();
    let wk = weak.clone();
    let hd = handle.clone();
    handle.spawn(async move {
        let active = rt.core().current_track().await.is_some();
        if active {
            // "Add to queue" only when the existing queue is itself all-ephemeral
            // (no mixing ephemeral with persistent tracks).
            let (queue, _) = rt.core().get_all_queue_tracks().await;
            let enqueue_allowed = !queue.is_empty()
                && queue.iter().all(|t| {
                    crate::ephemeral::is_ephemeral_id(t.id as i64)
                        || t.source.as_deref() == Some("ephemeral")
                });
            let k = kind.clone();
            let a = arg.clone();
            let _ = wk.upgrade_in_event_loop(move |w| {
                let s = w.global::<crate::EphemeralPlayChoiceState>();
                s.set_intent_kind(k.into());
                s.set_intent_arg(a.into());
                s.set_enqueue_allowed(enqueue_allowed);
                s.set_open(true);
            });
        } else {
            ephemeral_play(rt, wk, hd, kind, arg);
        }
    });
}
