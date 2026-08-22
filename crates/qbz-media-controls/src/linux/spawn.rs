use std::sync::{Arc, Mutex};

use mpris_server::{Metadata, PlaybackStatus as MprisStatus, Server, Time};

use crate::inhibit::SleepInhibitor;

use super::apply::apply;
use super::root_iface::QbzMpris;
use super::{EventCb, LinuxHandle, State, Update, BUS_SUFFIX, DESKTOP_ENTRY};

/// Spawn the MPRIS server on a dedicated thread. Returns `None` if the thread
/// or runtime can't start (the bus registration happens async on the thread;
/// failures there are logged, not surfaced — the app keeps running).
pub fn spawn(on_event: EventCb) -> Option<LinuxHandle> {
    let (tx, rx) = async_channel::unbounded::<Update>();

    let spawned = std::thread::Builder::new()
        .name("qbz-mpris".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("[mpris] runtime build failed: {e}");
                    return;
                }
            };
            rt.block_on(async move {
                let state = Arc::new(Mutex::new(State {
                    metadata: Metadata::new(),
                    status: MprisStatus::Stopped,
                    volume: 1.0,
                    position: Time::ZERO,
                }));
                let imp = QbzMpris {
                    on_event,
                    state: state.clone(),
                };
                let server = match Server::new(BUS_SUFFIX, imp).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("[mpris] failed to register org.mpris.MediaPlayer2.{BUS_SUFFIX}: {e}");
                        return;
                    }
                };
                log::info!(
                    "[mpris] registered org.mpris.MediaPlayer2.{BUS_SUFFIX} (DesktopEntry={DESKTOP_ENTRY})"
                );
                // Sleep/idle inhibitor (#522): held while Playing, dropped on
                // Paused/Stopped. Piggybacks on the same playback updates the
                // MPRIS server consumes, so it can never disagree with what
                // the desktop widget shows.
                let mut inhibitor = SleepInhibitor::new();
                while let Ok(update) = rx.recv().await {
                    if let Update::Playback { status, .. } = &update {
                        inhibitor
                            .set_playing(matches!(status, MprisStatus::Playing))
                            .await;
                    }
                    apply(&server, &state, update).await;
                }
                log::debug!("[mpris] update channel closed, server shutting down");
            });
        });

    match spawned {
        Ok(_) => Some(LinuxHandle { tx }),
        Err(e) => {
            log::error!("[mpris] failed to spawn server thread: {e}");
            None
        }
    }
}
