use super::super::*;

mod rebuild;

/// Handle `AudioCommand::Resume`.
pub(crate) fn handle(ctx: &mut ThreadCtx) {
    ctx.pause_suspend_deadline = None;
    if ctx.current_engine.is_none() {
        rebuild::resume_from_scratch(ctx);
        return;
    }

    if let Some(ref engine) = ctx.current_engine {
        engine.play();
        let current_pos = ctx.state.position.load(Ordering::SeqCst);
        ctx.state.start_playback_timer(current_pos);
        ctx.state.is_playing.store(true, Ordering::SeqCst);
        log::info!("Audio thread: resumed");
    }
}
