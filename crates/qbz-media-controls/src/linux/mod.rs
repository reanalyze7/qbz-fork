//! Linux MPRIS backend via `mpris-server`.
//!
//! The whole reason this exists instead of souvlaki: `mpris-server`'s
//! `RootInterface::desktop_entry()` lets us publish the
//! `org.mpris.MediaPlayer2.DesktopEntry` property as `"io.github.reanalyze7.qoqobuz"`,
//! which is the ONLY mechanism GNOME Shell uses to resolve the application
//! icon for its media widget (DesktopEntry → `<name>.desktop` → `Icon=`).
//! souvlaki never sets it, so GNOME shows no icon. (KDE is lenient and works
//! either way.) `mpris:artUrl` is album art — separate and unaffected.
//!
//! The server runs on a dedicated thread with its own current-thread tokio
//! runtime (the workspace forces zbus 4's `tokio` feature via qbz-audio, so a
//! tokio context must be present); state updates arrive over an async channel.

use std::sync::{atomic::AtomicU64, Arc};

use mpris_server::{PlaybackStatus as MprisStatus, Time, Volume};

use crate::types::{MediaEvent, MediaIntegration, PlaybackStatus, TrackMeta};

mod apply;
mod metadata;
mod player_iface;
mod root_iface;
mod spawn;

use metadata::{build_metadata, map_status};

pub use spawn::spawn;

const BUS_SUFFIX: &str = "io.github.reanalyze7.qoqobuz";
const DESKTOP_ENTRY: &str = "io.github.reanalyze7.qoqobuz";
const IDENTITY: &str = "Qoqobuz";

/// Monotonic counter so each track gets a distinct `mpris:trackid` object path
/// (helps clients detect track changes).
static TRACK_SEQ: AtomicU64 = AtomicU64::new(1);

type EventCb = Arc<dyn Fn(MediaEvent) + Send + Sync>;

/// Shared, mutable now-playing state. Read by the MPRIS getter methods (on the
/// zbus task) and written by the update loop (on the same runtime). Never held
/// across an `.await`.
struct State {
    metadata: mpris_server::Metadata,
    status: MprisStatus,
    volume: Volume,
    position: Time,
}

/// Update commands sent from the app to the server thread.
enum Update {
    Metadata(mpris_server::Metadata),
    Playback {
        status: MprisStatus,
        position: Option<Time>,
    },
    Volume(Volume),
}

/// The cloneable handle returned to the app. Pushing state is a non-blocking
/// channel send from any thread/context.
pub struct LinuxHandle {
    tx: async_channel::Sender<Update>,
}

impl MediaIntegration for LinuxHandle {
    fn set_metadata(&self, meta: &TrackMeta) {
        let _ = self.tx.try_send(Update::Metadata(build_metadata(meta)));
    }

    fn set_playback(&self, status: PlaybackStatus, position: Option<std::time::Duration>) {
        let _ = self.tx.try_send(Update::Playback {
            status: map_status(status),
            position: position.map(|d| Time::from_micros(d.as_micros() as i64)),
        });
    }

    fn set_volume(&self, vol: f64) {
        let _ = self.tx.try_send(Update::Volume(vol.clamp(0.0, 1.0)));
    }
}
