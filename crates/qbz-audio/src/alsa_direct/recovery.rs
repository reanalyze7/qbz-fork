//! Shared error-recovery / rate-verification helpers used by every
//! constructor and every write path in this module.

use alsa::pcm::{HwParams, PCM};

/// Log a PCM recovery and record it as a network-throttle underrun signal.
///
/// Each call to ALSA's `pcm.recover()` that returns successfully indicates
/// that the writer thread fell behind the kernel's playback buffer — i.e.
/// an audio underrun. The network throttle treats this as the strongest
/// possible "slow down" signal and immediately drops the prefetch cap to
/// zero for `PANIC_WINDOW_SECS`, so the live stream gets the full pipe to
/// recover.
pub(super) fn log_pcm_recovery(suffix: &str) {
    if suffix.is_empty() {
        log::warn!("[ALSA Direct] Recovered from PCM error");
    } else {
        log::warn!("[ALSA Direct] Recovered from PCM error {}", suffix);
    }
    crate::network_throttle::state().record_underrun();
}

/// Recover a failed write. `snd_pcm_recover` handles EPIPE/ESTRPIPE but NOT
/// EBADFD on this stack (observed on the Pi: recover itself returns 77) — and
/// EBADFD is what a write gets when it races a still-DRAINING pcm (natural-end
/// drain + a late append of the next track) or a stream left in limbo by a
/// failed prepare. For EBADFD, cancel the drain (drop) and prepare explicitly:
/// the stream is writable again and the NEW track's following chunks flow (the
/// one rejected chunk, ~50 ms, is lost — the drain was about to be cut anyway).
/// `snd_pcm_recover` still gets first try: on stacks where it DOES handle
/// EBADFD this behaves exactly as before.
pub(super) fn recover_write_error(pcm: &PCM, errno: i32, suffix: &str) -> Result<(), String> {
    // libc::EBADFD — pcm not in a writable state (e.g. mid-drain).
    const EBADFD: i32 = 77;
    match pcm.recover(errno, false) {
        Ok(()) => {
            log_pcm_recovery(suffix);
            Ok(())
        }
        Err(recover_err) if errno == EBADFD => {
            log::warn!(
                "[ALSA Direct] recover(EBADFD) unsupported ({recover_err}); drop+prepare to cancel the drain"
            );
            // UFCS: `pcm.drop()` resolves to `Drop::drop` — name the inherent
            // method explicitly (same gotcha as in `stop()`).
            PCM::drop(pcm).map_err(|e| format!("drop after EBADFD failed: {e}"))?;
            pcm.prepare()
                .map_err(|e| format!("prepare after EBADFD failed: {e}"))?;
            log_pcm_recovery(suffix);
            Ok(())
        }
        Err(recover_err) => Err(format!("Failed to recover from error: {recover_err}")),
    }
}

/// Fail closed when ALSA selected a different rate than requested (exclusive /
/// bit-perfect paths must not silently nearest-neighbor).
pub(super) fn ensure_exact_rate(hwp: &HwParams<'_>, requested: u32, kind: &str) -> Result<(), String> {
    let actual = hwp
        .get_rate()
        .map_err(|e| format!("Failed to read back {kind} sample rate: {e}"))?;
    if actual != requested {
        return Err(format!(
            "ALSA {kind} rate mismatch: requested {requested} Hz, device selected {actual} Hz (refusing non-bit-perfect nearest)"
        ));
    }
    Ok(())
}
