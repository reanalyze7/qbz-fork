//! Name acquisition + fallback-to-existing-primary logic.

use zbus::blocking::fdo::DBusProxy;
use zbus::blocking::Connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::names::WellKnownName;

use super::iface::SingleInstanceIface;
use super::{BUS_NAME, CONN, IFACE_NAME, OBJECT_PATH};

pub(super) fn probe() -> zbus::Result<bool> {
    let conn = Connection::session()?;
    // Export the Present interface BEFORE requesting the name: the moment a
    // second launch sees Exists, the object must already be callable (no
    // window where the name is owned but Present() isn't served yet).
    conn.object_server().at(OBJECT_PATH, SingleInstanceIface)?;
    let proxy = DBusProxy::new(&conn)?;
    let name: WellKnownName<'_> = BUS_NAME.try_into().map_err(zbus::Error::from)?;
    match proxy.request_name(name, RequestNameFlags::DoNotQueue.into())? {
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => {
            let _ = CONN.set(conn);
            Ok(true)
        }
        // Exists (or the DO_NOT_QUEUE-unreachable InQueue): another instance
        // runs. Ask it to present itself; both calls are best-effort — the
        // duplicate still must not start.
        RequestNameReply::Exists | RequestNameReply::InQueue => {
            // Carrying a deep link from argv (captured at process start)?
            // Forward it: OpenUrl makes the primary present itself AND
            // navigate. An older primary without the method errors → the
            // bare-Present ladder below (version-skew tolerance); the URL
            // is then lost with this exiting process, same as a failed emit
            // in the Tauri era.
            let forwarded = match crate::deep_link::take_pending() {
                Some(url) => conn
                    .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "OpenUrl", &url)
                    .is_ok(),
                None => false,
            };
            let presented = forwarded
                || conn
                    .call_method(Some(BUS_NAME), OBJECT_PATH, Some(IFACE_NAME), "Present", &())
                    .is_ok();
            if !presented {
                // Older primary (≤2.0.x) without the SingleInstance interface:
                // fall back to MPRIS Raise. Full MPRIS name =
                // "org.mpris.MediaPlayer2." + BUS_SUFFIX, and
                // qbz-media-controls registers with BUS_SUFFIX = the app id
                // (linux.rs), NOT "qbz".
                let _ = conn.call_method(
                    Some("org.mpris.MediaPlayer2.com.blitzfc.qbz"),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2"),
                    "Raise",
                    &(),
                );
            }
            Ok(false)
        }
    }
}
