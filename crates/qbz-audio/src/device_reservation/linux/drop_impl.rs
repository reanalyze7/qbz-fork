use zbus::fdo::ReleaseNameReply;

use super::dbus_ops::release_name;
use super::reservation::{DeviceReservation, ReservationState};

impl Drop for DeviceReservation {
    fn drop(&mut self) {
        if let ReservationState::Active {
            connection,
            bus_name,
            ..
        } = &self.state
        {
            match release_name(connection, bus_name) {
                Ok(ReleaseNameReply::Released) => {
                    log::debug!("[reservation] released {}", bus_name);
                }
                Ok(ReleaseNameReply::NonExistent) => {
                    // We thought we owned it but the bus daemon disagrees.
                    // Almost always indicates a logic bug in our state
                    // tracking — surface loudly.
                    log::warn!(
                        "[reservation] release_name returned NonExistent for {} \
                         (we believed we owned it)",
                        bus_name
                    );
                }
                Ok(ReleaseNameReply::NotOwner) => {
                    log::warn!(
                        "[reservation] release_name returned NotOwner for {} \
                         (we believed we owned it)",
                        bus_name
                    );
                }
                Err(e) => {
                    log::warn!("[reservation] release_name failed for {}: {}", bus_name, e);
                }
            }
        }
    }
}
