//! Low-level D-Bus call wrappers. Pure plumbing, no business logic.

use zbus::blocking::fdo::DBusProxy;
use zbus::blocking::{Connection, Proxy};
use zbus::fdo::{ReleaseNameReply, RequestNameFlags, RequestNameReply};
use zbus::names::WellKnownName;

use super::error::ReservationError;
use super::RESERVE_DEVICE1_INTERFACE;

/// Issue `RequestName` for `bus_name`. Always sets `DO_NOT_QUEUE`; sets
/// `REPLACE_EXISTING` when `replace` is true (used after a successful
/// `RequestRelease` on the previous holder).
///
/// Returns the `RequestNameReply` on success. zbus errors are surfaced as
/// `ReservationError::DbusError`.
pub(super) fn try_acquire_name(
    conn: &Connection,
    bus_name: &str,
    replace: bool,
) -> Result<RequestNameReply, ReservationError> {
    let proxy = DBusProxy::new(conn)
        .map_err(|e| ReservationError::DbusError(format!("DBusProxy::new failed: {}", e)))?;
    let well_known: WellKnownName<'_> = bus_name
        .try_into()
        .map_err(|e| ReservationError::DbusError(format!("invalid bus name '{}': {}", bus_name, e)))?;
    let flags = if replace {
        RequestNameFlags::DoNotQueue | RequestNameFlags::ReplaceExisting
    } else {
        RequestNameFlags::DoNotQueue.into()
    };
    proxy
        .request_name(well_known, flags)
        .map_err(|e| ReservationError::DbusError(format!("request_name failed: {}", e)))
}

/// Release the bus name. Pure forward of the zbus reply variant; the caller
/// (`Drop`) decides what to log. Returns `zbus::fdo::Error` which already
/// implements `Display`.
pub(super) fn release_name(conn: &Connection, bus_name: &str) -> zbus::fdo::Result<ReleaseNameReply> {
    let proxy = DBusProxy::new(conn)?;
    // .map_err(zbus::Error::from) bridges names crate -> zbus::Error;
    // ? then bridges zbus::Error -> fdo::Error.
    let well_known: WellKnownName<'_> = bus_name.try_into().map_err(zbus::Error::from)?;
    proxy.release_name(well_known)
}

/// Open a `Proxy` for the current holder's `ReserveDevice1` interface. Used
/// for property reads (`Priority`, `ApplicationName`) and the `RequestRelease`
/// call; the same proxy handle serves all three so we only pay one
/// construction cost per contention case.
pub(super) fn open_holder_proxy<'a>(
    conn: &'a Connection,
    bus_name: &'a str,
    object_path: &'a str,
) -> zbus::Result<Proxy<'a>> {
    Proxy::new(conn, bus_name, object_path, RESERVE_DEVICE1_INTERFACE)
}

/// Read the holder's `Priority` property via `org.freedesktop.DBus.Properties.Get`.
/// Returns `None` if the holder is uncooperative (no such property, type
/// mismatch, etc.); the caller treats that as priority 0.
pub(super) fn read_priority(proxy: &Proxy<'_>) -> Option<i32> {
    proxy.get_property::<i32>("Priority").ok()
}

/// Read the holder's `ApplicationName` property. Used purely for human-readable
/// error messages in `HigherPriorityHolder`.
pub(super) fn read_application_name(proxy: &Proxy<'_>) -> Option<String> {
    proxy.get_property::<String>("ApplicationName").ok()
}

/// Call `RequestRelease(priority)` on the current holder via the shared
/// proxy. Returns the holder's reply (`true` = will release, `false` =
/// refuses).
pub(super) fn request_release(proxy: &Proxy<'_>, priority: i32) -> Result<bool, ReservationError> {
    proxy
        .call::<_, _, bool>("RequestRelease", &(priority,))
        .map_err(|e| ReservationError::DbusError(format!("RequestRelease failed: {}", e)))
}
