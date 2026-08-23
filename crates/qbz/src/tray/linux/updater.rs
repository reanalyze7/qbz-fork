//! The mpsc-channel + dedicated-thread pattern for live tray updates.
//!
//! We cannot call `ksni::blocking::Handle::update` directly from a tokio task —
//! `ksni` 0.3 (`feature = "blocking"`) wraps every update in
//! `Runtime::block_on`, which panics inside an existing tokio runtime (the
//! playback poll loop runs on one). The panic is swallowed, so updates appear
//! to succeed but never apply. We serialise updates over a `std::sync::mpsc`
//! channel applied from a dedicated `std::thread` outside any tokio context.

use std::sync::{
    mpsc::{self, Sender},
    Arc, Mutex,
};

use super::icons::decode_tray_icons;
use super::tray_impl::{NowPlaying, QbzTray};

/// Updates dispatched from the rest of the app into the ksni worker.
pub(super) enum TrayUpdate {
    SetTrack {
        title: String,
        artist: String,
        album: String,
    },
    ClearTrack,
    SetPlaying(bool),
    /// Re-decode pixmaps for the requested theme override and push them live
    /// (`NewIcon` SNI signal — panels re-fetch without restart).
    SetIconTheme(String),
}

/// Cross-thread handle to the live ksni tray. Cloneable; mutators forward to
/// the worker thread, safe from any async context. When the tray failed to
/// start the inner sender stays `None` and every mutator is a no-op.
#[derive(Clone)]
pub struct LinuxTrayHandle {
    sender: Arc<Mutex<Option<Sender<TrayUpdate>>>>,
}

impl LinuxTrayHandle {
    pub(super) fn empty() -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn install(&self, handle: ksni::blocking::Handle<QbzTray>) {
        let (tx, rx) = mpsc::channel::<TrayUpdate>();
        std::thread::Builder::new()
            .name("qbz-tray-updater".into())
            .spawn(move || {
                while let Ok(msg) = rx.recv() {
                    match msg {
                        TrayUpdate::SetTrack {
                            title,
                            artist,
                            album,
                        } => {
                            log::debug!("[tray] tooltip update -> {} / {} / {}", title, artist, album);
                            handle.update(move |tray| {
                                tray.now_playing = Some(NowPlaying {
                                    title,
                                    artist,
                                    album,
                                });
                            });
                        }
                        TrayUpdate::ClearTrack => {
                            log::debug!("[tray] tooltip cleared");
                            handle.update(|tray| {
                                tray.now_playing = None;
                                tray.is_playing = false;
                            });
                        }
                        TrayUpdate::SetPlaying(is_playing) => {
                            handle.update(move |tray| {
                                tray.is_playing = is_playing;
                            });
                        }
                        TrayUpdate::SetIconTheme(theme) => {
                            log::info!("[tray] icon theme override -> {}", theme);
                            match decode_tray_icons(Some(&theme)) {
                                Ok(new_icons) => {
                                    handle.update(move |tray| {
                                        tray.icons = new_icons;
                                    });
                                }
                                Err(e) => {
                                    log::error!(
                                        "[tray] failed to decode icons for theme '{}': {}",
                                        theme,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
                log::debug!("[tray] updater thread exiting");
            })
            .expect("spawn tray updater thread");
        if let Ok(mut guard) = self.sender.lock() {
            *guard = Some(tx);
        }
    }

    pub(super) fn send(&self, msg: TrayUpdate) {
        let guard = match self.sender.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(msg);
        }
    }
}
