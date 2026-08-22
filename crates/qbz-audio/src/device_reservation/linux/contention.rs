use zbus::blocking::Connection;
use zbus::fdo::RequestNameReply;

use super::dbus_ops::{
    open_holder_proxy, read_application_name, read_priority, request_release, try_acquire_name,
};
use super::error::ReservationError;
use super::reservation::{DeviceReservation, ReservationState};
use super::QBZ_PRIORITY;

/// We tried to acquire a held bus name and got `Exists` (or `InQueue`).
/// Inspect the holder's `Priority`, decide whether to ask them to release,
/// and either retry or return a `HigherPriorityHolder` error.
pub(super) fn resolve_contention(
    conn: &Connection,
    bus_name: &str,
    object_path: &str,
    app_device_name: &str,
) -> Result<DeviceReservation, ReservationError> {
    // One Proxy serves all reads + the RequestRelease call against the
    // current holder. zbus's Proxy is cheap to keep alive, but constructing
    // it twice for back-to-back property reads has been observed to cost an
    // extra GetAll round trip on some bus daemons; one proxy is both faster
    // and clearer.
    let holder_proxy = open_holder_proxy(conn, bus_name, object_path)
        .map_err(|e| ReservationError::DbusError(format!("Proxy::new for holder failed: {}", e)))?;

    // Default to 0 if the holder is uncooperative or doesn't expose Priority.
    // Rationale: PulseAudio/PipeWire are the most common holders and run at
    // priority 0; treating an unreadable priority as 0 lets us still pre-empt
    // them. Pro apps that *do* publish at higher priority will refuse via
    // RequestRelease anyway, which we honour below.
    let holder_priority = read_priority(&holder_proxy).unwrap_or(0);

    if QBZ_PRIORITY <= holder_priority {
        let holder_name = read_application_name(&holder_proxy)
            .unwrap_or_else(|| "another application".to_string());
        log::info!(
            "[reservation] {} held by {} at priority {} (>= ours {}); refusing",
            bus_name,
            holder_name,
            holder_priority,
            QBZ_PRIORITY
        );
        return Err(ReservationError::HigherPriorityHolder {
            holder_name,
            holder_priority,
        });
    }

    log::debug!(
        "[reservation] {} held at priority {}; calling RequestRelease({})",
        bus_name,
        holder_priority,
        QBZ_PRIORITY
    );

    let released = request_release(&holder_proxy, QBZ_PRIORITY)?;
    if !released {
        let holder_name = read_application_name(&holder_proxy)
            .unwrap_or_else(|| "another application".to_string());
        log::info!(
            "[reservation] {} held by {} refused RequestRelease",
            bus_name,
            holder_name
        );
        return Err(ReservationError::HigherPriorityHolder {
            holder_name,
            holder_priority,
        });
    }

    // Holder agreed to release. Retry with REPLACE_EXISTING.
    match try_acquire_name(conn, bus_name, true)? {
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => {
            log::debug!("[reservation] acquired {} after RequestRelease", bus_name);
            Ok(DeviceReservation {
                state: ReservationState::Active {
                    connection: conn.clone(),
                    bus_name: bus_name.to_string(),
                    app_device_name: app_device_name.to_string(),
                },
            })
        }
        RequestNameReply::Exists | RequestNameReply::InQueue => {
            // Someone slipped in between the holder releasing and us
            // re-requesting. Rare; surfaces as a generic D-Bus error.
            Err(ReservationError::DbusError(format!(
                "lost race after holder released {}",
                bus_name
            )))
        }
    }
}
