//! `AlsaDirectStream::drain` and `stop` — the bounded-drain polling loop and
//! the drop+prepare stop ritual. Both carry critical hardware-quirk comments;
//! keep them attached to their functions verbatim.

use super::AlsaDirectStream;
use alsa::pcm::PCM;

impl AlsaDirectStream {
    /// Drain and stop playback
    pub fn drain(&self) -> Result<(), String> {
        log::info!("[ALSA Direct] Draining PCM");
        let pcm = self.pcm.lock().unwrap();
        // BOUNDED drain — a bare `snd_pcm_drain` blocks until every queued
        // frame clocks out, and on this driver (snd-rpi-hifiberry/PCM5122) it
        // can block FOREVER when the device stops clocking (observed on the
        // Pi: natural track end -> drain never returned -> the writer thread
        // wedged -> no "engine empty" -> playback died at the transition,
        // "dos tracks y pausa"). Poll the state instead: while frames clock
        // out the pcm is Running; when the tail finishes it underruns to XRun
        // (no more writes are coming) — that IS the drained end state, so
        // drop+prepare and return. If it neither drains nor underruns within
        // the deadline, cancel with drop+prepare so the transition survives.
        const DRAIN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);
        let start = std::time::Instant::now();
        loop {
            match pcm.state() {
                alsa::pcm::State::Running | alsa::pcm::State::Draining => {
                    if start.elapsed() >= DRAIN_DEADLINE {
                        log::warn!(
                            "[ALSA Direct] drain deadline (10s) hit — drop+prepare to unstick the pcm"
                        );
                        // UFCS: `pcm.drop()` resolves to `Drop::drop` on the
                        // MutexGuard — name the inherent method explicitly.
                        PCM::drop(&pcm).map_err(|e| format!("drop after stuck drain: {e}"))?;
                        return pcm
                            .prepare()
                            .map_err(|e| format!("prepare after stuck drain: {e}"));
                    }
                    // Sleep in 100 ms slices waiting for the device to clock.
                    let _ = pcm.wait(Some(100));
                }
                alsa::pcm::State::XRun => {
                    // Tail finished (natural underrun at end-of-stream) or the
                    // frames are gone either way — reset for the next track.
                    PCM::drop(&pcm).map_err(|e| format!("drop after drain XRUN: {e}"))?;
                    return pcm
                        .prepare()
                        .map_err(|e| format!("prepare after drain XRUN: {e}"));
                }
                // Already drained (Setup/Prepared), Paused, or anything else:
                // nothing to wait for.
                _ => return Ok(()),
            }
        }
    }

    /// Stop PCM immediately (prepare for next playback)
    pub fn stop(&self) -> Result<(), String> {
        log::info!("[ALSA Direct] Stopping PCM");
        let pcm = self.pcm.lock().unwrap();
        // Standard immediate-stop ritual: DROP (halt now, discard queued
        // frames) THEN prepare. prepare() alone on a RUNNING or DRAINING
        // stream fails with EBUSY on drivers that require an explicit drop
        // first (snd-rpi-hifiberry / PCM5122 — every stop on the Pi logged
        // "Device or resource busy (16)"), and each failed prepare left the
        // PCM in a limbo the NEXT stream's write surfaced as unrecoverable
        // EBADFD, killing the track transition. drop() from a non-running
        // state returns EBADFD — nothing was playing; harmless, ignore.
        // UFCS: `pcm.drop()` would resolve to `Drop::drop` on the MutexGuard
        // (the guard is the first deref step with a `drop` candidate) — the
        // inherent PCM method must be named explicitly.
        if let Err(e) = PCM::drop(&pcm) {
            // libc::EBADFD — the PCM was not in a running-ish state.
            const EBADFD: i32 = 77;
            if e.errno() as i32 != EBADFD {
                log::warn!("[ALSA Direct] drop on stop failed (continuing to prepare): {}", e);
            }
        }
        pcm.prepare()
            .map_err(|e| format!("Failed to prepare PCM after stop: {}", e))
    }
}
