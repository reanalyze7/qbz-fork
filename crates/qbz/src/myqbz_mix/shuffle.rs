//! Confirm/shuffle flow: sample, then replace-play the queue.

use slint::ComponentHandle;

use qbz_models::QueueTrack;

use super::open_close::{close, close_with_error};
use super::Runtime;
use crate::{AppWindow, MyQbzMixState};

/// Confirm: sample `sample_size` songs from the collection and replace-play the
/// queue. Re-resolves the collection in-order, runs dedup+sample with the thread
/// RNG (confined to a sync scope — see the module doc), then replaces the queue
/// (stamp queue-source + touch_play via [`crate::myqbz_play::play_all_tracks`]).
/// `requested < actual` ⇒ a "Playing N of M" info toast (spec §9.16).
pub fn shuffle(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    collection_id: String,
    sample_size: i32,
) {
    if collection_id.is_empty() || sample_size <= 0 {
        return;
    }
    // Disable the Shuffle button while in flight.
    {
        let weak = weak.clone();
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<MyQbzMixState>().set_busy(true);
        });
    }

    handle.spawn(async move {
        let Some(collection) = crate::myqbz_play::load_collection(&collection_id).await else {
            close_with_error(&weak, qbz_i18n::t("Couldn't load this collection"));
            return;
        };
        // Resolve in-order (await completes BEFORE the RNG is created).
        let resolved = crate::myqbz_play::resolve_collection(&runtime, &collection, false).await;

        // ── RNG-confined sync scope (spec 40 §6): create, use, and DROP the
        // !Send ThreadRng entirely here — it never crosses the `.await` below.
        let requested = sample_size as usize;
        let sampled: Vec<QueueTrack> = {
            let mut rng = rand::rng();
            let deduped = qbz_mixtape::shuffle::dedup_by_similarity(resolved, &mut rng);
            qbz_mixtape::shuffle::hybrid_sample(deduped, requested, &mut rng)
        };
        // `rng` is dropped; from here on the future is `Send` again.

        let actual = sampled.len();

        // Close the modal before playback starts (mirrors handleConfirmMix).
        {
            let weak = weak.clone();
            let _ = weak.upgrade_in_event_loop(|w: AppWindow| close(&w));
        }

        // Replace-play: set_queue + start at 0 + stamp queue-source + touch_play.
        crate::myqbz_play::play_all_tracks(&runtime, &weak, &collection_id, sampled).await;

        // DJ-mix actualCount can be < requested (per-album cap) — surface it.
        if actual > 0 && actual < requested {
            crate::toast::info_weak(
                &weak,
                qbz_i18n::t_args("Playing {} of {}", &[&actual.to_string(), &requested.to_string()]),
            );
        }
    });
}
