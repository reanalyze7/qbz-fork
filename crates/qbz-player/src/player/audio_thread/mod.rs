use super::*;

mod commands;
mod ctx;
mod ctx_device;
mod ctx_device_legacy;
mod ctx_source;
mod loop_run;

pub(crate) use ctx::{ThreadCtx, MAX_SINK_FAILURES, PAUSE_SUSPEND_DELAY_MS};

/// Spawn the dedicated audio thread and return the command channel used to
/// talk to it. Mirrors the old `Player::new` closure, but the command
/// handling and command loop now live in `commands` / `loop_run` as plain
/// functions over `&mut ThreadCtx` instead of one giant closure.
pub(crate) fn spawn(
    device_name: Option<String>,
    settings: Arc<Mutex<AudioSettings>>,
    viz_tap: Option<VisualizerTap>,
    diagnostic: AudioDiagnostic,
    state: SharedState,
) -> Sender<AudioCommand> {
    let (tx, rx) = mpsc::channel::<AudioCommand>();

    thread::spawn(move || {
        let mut ctx = ThreadCtx::new(device_name, settings, viz_tap, diagnostic, state);
        log::info!("Audio thread ready and waiting for commands");
        loop_run::run(&mut ctx, &rx);
    });

    tx
}
