//! stop/stop_inner/Drop for [`PlaybackEngine`] (see `play_pause.rs` for
//! play/pause).

use super::super::PlaybackEngine;
use std::sync::atomic::Ordering;

impl PlaybackEngine {
    /// Stop playback and release resources.
    /// For ALSA Direct, signals the writer thread and waits for it to exit.
    /// The Drop impl handles the same cleanup if stop() is not called explicitly.
    pub fn stop(mut self) {
        self.stop_inner();
    }

    /// Internal stop logic shared by stop() and Drop
    pub(crate) fn stop_inner(&mut self) {
        match self {
            Self::Rodio { sink, .. } => {
                sink.stop();
            }
            Self::AlsaDirect {
                stream,
                is_playing,
                should_stop,
                playback_thread,
                ..
            } => {
                if should_stop.load(Ordering::SeqCst) {
                    return; // Already stopped
                }
                log::info!("[ALSA Direct Engine] Stop requested");
                should_stop.store(true, Ordering::SeqCst);
                is_playing.store(false, Ordering::SeqCst);

                if let Some(handle) = playback_thread.take() {
                    let _ = handle.join();
                }

                if let Err(e) = stream.stop() {
                    log::warn!("[ALSA Direct Engine] Stop failed: {}", e);
                }
            }
            #[cfg(target_os = "linux")]
            Self::Jack {
                is_playing,
                should_stop,
                feeder_thread,
                ..
            } => {
                if should_stop.load(Ordering::SeqCst) {
                    return;
                }
                log::info!("[JACK Engine] Stop requested");
                should_stop.store(true, Ordering::SeqCst);
                is_playing.store(false, Ordering::SeqCst);
                if let Some(handle) = feeder_thread.take() {
                    let _ = handle.join();
                }
                // JackStream's Drop deactivates the client + unregisters the ports.
            }
            #[cfg(target_os = "linux")]
            Self::AlsaDop {
                stream,
                is_playing,
                should_stop,
                writer_thread,
                ..
            } => {
                if should_stop.load(Ordering::SeqCst) {
                    return;
                }
                log::info!("[DoP Engine] Stop requested");
                should_stop.store(true, Ordering::SeqCst);
                is_playing.store(false, Ordering::SeqCst);
                if let Some(handle) = writer_thread.take() {
                    let _ = handle.join();
                }
                if let Err(e) = stream.stop() {
                    log::warn!("[DoP Engine] Stop failed: {}", e);
                }
            }
        }
    }
}

impl Drop for PlaybackEngine {
    fn drop(&mut self) {
        self.stop_inner();
    }
}
