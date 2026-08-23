//! The final `NowPlayingState` push for a track change, plus kicking off
//! the (async) artwork loads.

use super::artwork::{load_now_playing_artwork, load_now_playing_artwork_large};
use super::fields_types::MetaFields;
use super::super::quality::{fmt_remaining, set_viz_paused};
use super::super::state::FORCE_UI_REPUSH;
use super::super::Runtime;
use crate::{AppWindow, NowPlayingState};

/// Push `fields` onto `NowPlayingState` (optimistic seed: position 0,
/// playing true — the poll loop corrects it once real audio starts, see
/// `FORCE_UI_REPUSH`), then kick off the bar + hover-preview artwork loads.
pub(super) fn finish_meta_push(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    shuffle_seed: bool,
    repeat_seed: i32,
    fields: MetaFields,
) {
    // The bar is seeded playing=true below — wake the visualizer producer with
    // it (stored BEFORE the UI post so the drain gate never opens while the
    // producer is still marked paused).
    set_viz_paused(runtime, false);
    let duration = fields.duration;
    let sample_rate = fields.sample_rate;
    let bit_depth = fields.bit_depth;
    let bar_artwork = fields.bar_artwork.clone();
    let preview_artwork = fields.preview_artwork.clone();
    let _ = weak.upgrade_in_event_loop(move |w| {
        let np = w.global::<NowPlayingState>();
        np.set_has_track(true);
        np.set_shuffle(shuffle_seed);
        np.set_repeat_mode(repeat_seed);
        np.set_title(fields.title.into());
        np.set_artist(fields.artist.into());
        np.set_album(fields.album_display.into());
        np.set_album_id(fields.album_id.into());
        np.set_album_favorite(fields.album_favorite);
        np.set_artist_id(fields.artist_id.into());
        // Re-publish the "playing from" origin for THIS track every change — the
        // authoritative source for the song-card layers button (no stale global).
        np.set_context_kind(fields.context_kind.into());
        np.set_context_id(fields.context_id.into());
        np.set_track_id(fields.track_id.into());
        np.set_local_track_id(fields.local_track_id.into());
        np.set_is_ephemeral(fields.is_ephemeral);
        np.set_source(fields.source.into());
        np.set_quality_tier(fields.quality_tier.into());
        np.set_quality_detail(fields.quality_detail.into());
        // Numeric stream params for the Spectral Ribbon overlay. `sample_rate`
        // (the merged catalog/hydrated value) is Hz when >= 1000, else kHz —
        // normalize to Hz.
        np.set_sample_rate_hz(sample_rate.map_or(0, |sr| {
            if sr >= 1000.0 {
                sr as i32
            } else {
                (sr * 1000.0) as i32
            }
        }));
        np.set_bit_depth(bit_depth.unwrap_or(0) as i32);
        // Reset the EFFECTIVE (delivered) quality on every track change — the
        // poll loop re-derives it from the engine's PlaybackEvent once the new
        // stream opens (the old track's downgrade state must not linger).
        np.set_effective_sample_rate_hz(0);
        np.set_effective_bit_depth(0);
        np.set_quality_downgraded(false);
        np.set_quality_true_detail("".into());
        np.set_quality_limit_cause(0);
        np.set_quality_effective_tier("".into());
        np.set_duration_secs(duration as i32);
        np.set_position_secs(0);
        np.set_progress(0.0);
        np.set_cache(0.0);
        np.set_elapsed("0:00".into());
        np.set_remaining(fmt_remaining(0, duration).into());
        np.set_playing(true);
        // Clear the previous cover so it does not linger while the new
        // one resolves.
        np.set_artwork(slint::Image::default());
        // Clear the hover-preview cover too, exactly like the bar art, so the
        // floating preview never shows the previous track while the new high-res
        // cover resolves.
        np.set_artwork_large(slint::Image::default());
        // Do NOT clear the immersive atmosphere bg here. Blanking it caused a
        // visible BACKGROUND FLICKER on a click-driven track change (the async
        // 300px decode + blur takes a beat, so the bg went blank then back).
        // Let the previous blurred ambient bg persist until
        // load_now_playing_artwork_large swaps in the new one — a brief stale
        // blur is imperceptible; a blank/raw-cover fallback is not.
    });

    // The bar was just seeded optimistically — make the poll loop's next tick
    // re-push engine/peer truth even if the raw snapshot is unchanged (see
    // FORCE_UI_REPUSH). Set AFTER the closure post above: the corrective push
    // is also an event-loop post, so FIFO ordering keeps it after the seed.
    FORCE_UI_REPUSH.store(true, std::sync::atomic::Ordering::Relaxed);

    load_now_playing_artwork(weak.clone(), bar_artwork);
    load_now_playing_artwork_large(weak.clone(), preview_artwork);
}
