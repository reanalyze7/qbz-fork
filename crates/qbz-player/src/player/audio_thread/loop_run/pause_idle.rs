use super::super::*;
use std::sync::mpsc::{Receiver, RecvTimeoutError};

/// Drop the output stream after the pause-suspend delay to save CPU while
/// idle. DoP streams are never suspended (the writer keeps the DAC locked
/// in DSD mode with 0x69 silence, and there's no audio data to resume from).
fn suspend_stream(ctx: &mut ThreadCtx) {
    if let Some(engine) = ctx.current_engine.take() {
        engine.stop();
    }
    drop(ctx.stream_opt.take());
    ctx.pause_suspend_deadline = None;
    // Self-gating: no-ops unless QBZ forced/suspended these (#263).
    #[cfg(target_os = "linux")]
    qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    #[cfg(target_os = "linux")]
    qbz_audio::alsa_backend::resume_suspended_sink();
    log::info!("Audio thread: suspended stream after pause");
}

/// Handle one iteration of the "not playing" branch of the main loop.
/// Returns `true` when the caller should break out of the loop (channel
/// disconnected).
pub(super) fn handle_idle(ctx: &mut ThreadCtx, rx: &Receiver<AudioCommand>) -> bool {
    if let Some(deadline) = ctx.pause_suspend_deadline {
        let dop_active = ctx
            .current_engine
            .as_ref()
            .map(|e| e.is_dop())
            .unwrap_or(false);
        if ctx.stream_opt.is_some() && !dop_active {
            let now = Instant::now();
            if now >= deadline {
                suspend_stream(ctx);
                return false;
            }

            let wait = deadline.saturating_duration_since(now);
            let wait = std::cmp::min(wait, Duration::from_millis(250));
            return match rx.recv_timeout(wait) {
                Ok(command) => {
                    super::super::commands::dispatch(ctx, command);
                    false
                }
                Err(RecvTimeoutError::Timeout) => false,
                Err(RecvTimeoutError::Disconnected) => {
                    log::info!("Audio thread: channel closed, exiting");
                    true
                }
            };
        }
        ctx.pause_suspend_deadline = None;
    }

    match rx.recv() {
        Ok(command) => {
            super::super::commands::dispatch(ctx, command);
            false
        }
        Err(_) => {
            log::info!("Audio thread: channel closed, exiting");
            true
        }
    }
}
