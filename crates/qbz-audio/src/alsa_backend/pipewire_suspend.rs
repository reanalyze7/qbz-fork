//! PipeWire sink suspend/resume for exclusive-mode ALSA access.

use std::sync::Mutex;

/// The PipeWire sink QBZ suspended to take a device exclusively (ALSA-direct
/// EBUSY retry / CPAL-exclusive). Recorded by *resolved name* so
/// `resume_suspended_sink` wakes the exact sink that was suspended — PipeWire
/// sink names are deterministic, so resume-by-name is reliable even if the sink
/// was vacated and re-created while QBZ held the device. Issue #263: QBZ
/// suspended the default sink but never resumed it, leaving the rest of the
/// system stuck on a suspended sink after exclusive playback.
static SUSPENDED_SINK: Mutex<Option<String>> = Mutex::new(None);

/// Suspend the current default PipeWire sink so an exclusive ALSA device open
/// can grab the hardware, recording the resolved sink name for later resume.
/// Falls back to the `@DEFAULT_SINK@` alias if the name can't be resolved.
pub(super) fn suspend_default_sink_for_exclusive() {
    let resolved = std::process::Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let target = resolved.unwrap_or_else(|| "@DEFAULT_SINK@".to_string());
    match std::process::Command::new("pactl")
        .args(["suspend-sink", &target, "1"])
        .output()
    {
        Ok(output) if output.status.success() => {
            log::info!(
                "[ALSA Backend] Suspended PipeWire sink '{}' for exclusive access",
                target
            );
            if let Ok(mut guard) = SUSPENDED_SINK.lock() {
                *guard = Some(target);
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("[ALSA Backend] Failed to suspend PipeWire sink: {}", stderr);
        }
        Err(e) => log::warn!("[ALSA Backend] Error suspending PipeWire sink: {}", e),
    }
}

/// Resume the PipeWire sink QBZ suspended for exclusive access (issue #263 leak
/// fix). No-op if QBZ did not suspend one — so it is safe to call on every
/// stop/teardown. Call it once the exclusive device has actually been released
/// so the rest of the system can use the sink again.
pub fn resume_suspended_sink() {
    let target = match SUSPENDED_SINK.lock() {
        Ok(mut guard) => guard.take(),
        Err(_) => None,
    };
    if let Some(sink) = target {
        let _ = std::process::Command::new("pactl")
            .args(["suspend-sink", &sink, "0"])
            .output();
        log::info!("[ALSA Backend] Resumed PipeWire sink '{}'", sink);
    }
}
