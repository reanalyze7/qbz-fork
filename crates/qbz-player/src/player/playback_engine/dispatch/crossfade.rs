//! `crossfade_to` for [`PlaybackEngine`].

use super::super::PlaybackEngine;
use rodio::{Player as RodioPlayer, Source};
use std::thread;
use std::time::Duration;

impl PlaybackEngine {
    /// Start `source` playing IMMEDIATELY on a SECOND player connected to
    /// the same mixer — overlapping the current (outgoing) source instead
    /// of `append`'s sequential queue — then cross-fades between them over
    /// `fade`. `self` becomes the incoming player right away (so
    /// subsequent play/pause/volume/position calls target the new track,
    /// matching what the user now perceives as "playing"); the outgoing
    /// one keeps sounding, fading out, until it's dropped.
    ///
    /// The incoming source's fade-IN rides rodio's own `Source::fade_in`
    /// adapter (applied sample-accurately as it decodes — no timer/thread
    /// needed, can't drift out of sync with playback). The outgoing
    /// player's fade-OUT has no rodio equivalent for an ALREADY-PLAYING
    /// sink, so it's ramped via `set_volume` on a short-lived detached
    /// thread — this method must return immediately, not block the
    /// audio-command thread that calls it for `fade`'s duration.
    pub fn crossfade_to<S>(&mut self, source: S, fade: Duration) -> Result<(), String>
    where
        S: Source<Item = f32> + Send + 'static,
    {
        let Self::Rodio { sink, mixer } = self else {
            return Err("crossfade is only supported on the Rodio engine".to_string());
        };
        let incoming = RodioPlayer::connect_new(mixer);
        incoming.append(source.fade_in(fade));
        incoming.play();

        let outgoing = std::mem::replace(sink, incoming);
        let steps: u32 = 40; // ~25ms ticks at a typical 0-10s fade — smooth, not wasteful.
        let step_dur = fade / steps.max(1);
        thread::spawn(move || {
            for i in (0..=steps).rev() {
                outgoing.set_volume(i as f32 / steps as f32);
                thread::sleep(step_dur);
            }
            // `outgoing` drops here — its output stops.
        });
        Ok(())
    }
}
