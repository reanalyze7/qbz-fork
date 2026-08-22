use zbus::blocking::Connection;
use zbus::fdo::RequestNameReply;

use super::contention::resolve_contention;
use super::error::ReservationError;
use super::reservation::{DeviceReservation, ReservationState};

/// Handle the reply to the initial (non-`REPLACE_EXISTING`) `RequestName`
/// call in [`DeviceReservation::acquire`]. Split out purely to keep
/// `acquire` itself under the file's line budget; the decision logic is
/// otherwise unchanged from the original single-function implementation.
pub(super) fn handle_request_name_reply(
    reply: RequestNameReply,
    connection: Connection,
    bus_name: String,
    object_path: String,
    app_device_name: &str,
) -> Result<DeviceReservation, ReservationError> {
    match reply {
        // Either we just took ownership, or we already owned this name on
        // this same connection (idempotent for Lifetime-A nested under
        // Lifetime-B in Task 5).
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => {
            log::debug!("[reservation] acquired {}", bus_name);
            Ok(DeviceReservation {
                state: ReservationState::Active {
                    connection,
                    bus_name,
                    app_device_name: app_device_name.to_string(),
                },
            })
        }
        // Someone else holds it (or is queued). Check their priority and
        // ask them to step aside.
        //
        // DO_NOT_QUEUE makes InQueue unreachable in practice, but
        // RequestNameReply is exhaustively matched and the contention
        // logic handles it identically to Exists.
        RequestNameReply::Exists | RequestNameReply::InQueue => {
            match resolve_contention(&connection, &bus_name, &object_path, app_device_name) {
                Ok(res) => Ok(res),
                // A cooperative higher-priority holder (DAW / pro-audio app)
                // refused to release: honor it — the one deliberate fatal case.
                Err(e @ ReservationError::HigherPriorityHolder { .. }) => Err(e),
                // Anything else (holder proxy unreachable, RequestRelease call
                // failed, lost re-acquire race): degrade — the PCM open plus its
                // EBUSY backoff ladder is the real arbiter, exactly like the
                // request_name-failure path above (#508/#534).
                Err(e) => {
                    log::warn!(
                        "[reservation] contention resolution for {} failed: {}. \
                         Proceeding without D-Bus reservation (degraded).",
                        bus_name,
                        e
                    );
                    Ok(DeviceReservation {
                        state: ReservationState::Degraded,
                    })
                }
            }
        }
    }
}
