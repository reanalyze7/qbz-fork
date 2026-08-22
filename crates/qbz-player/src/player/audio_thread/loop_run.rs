use super::*;
use std::sync::mpsc::{Receiver, RecvTimeoutError};

mod pause_idle;
mod tick;

/// The audio thread's main command loop: while playing, poll with a short
/// timeout and run periodic housekeeping (`tick::on_timeout`) between
/// commands; while paused, defer to `pause_idle` (which suspends the
/// stream after a delay to save CPU).
pub(crate) fn run(ctx: &mut ThreadCtx, rx: &Receiver<AudioCommand>) {
    loop {
        if ctx.state.is_playing.load(Ordering::SeqCst) {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(command) => super::commands::dispatch(ctx, command),
                Err(RecvTimeoutError::Timeout) => tick::on_timeout(ctx),
                Err(RecvTimeoutError::Disconnected) => {
                    log::info!("Audio thread: channel closed, exiting");
                    break;
                }
            }
        } else if pause_idle::handle_idle(ctx, rx) {
            break;
        }
    }
}
