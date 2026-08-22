use zbus::blocking::Connection;

#[derive(Debug)]
pub struct DeviceReservation {
    pub(super) state: ReservationState,
}

#[derive(Debug)]
pub(super) enum ReservationState {
    /// We own the bus name `bus_name` on `connection`. `Drop` releases it.
    /// `app_device_name` is stashed for Task 5 (status payload) — kept private.
    Active {
        connection: Connection,
        bus_name: String,
        #[allow(dead_code)] // Surfaced via Tauri status command in Task 5.
        app_device_name: String,
    },
    /// D-Bus session bus was unreachable, or some other graceful-degrade
    /// path. `is_active()` reports `false`; `Drop` is a no-op.
    Degraded,
}

impl DeviceReservation {
    /// Whether this guard currently holds an active D-Bus reservation.
    pub fn is_active(&self) -> bool {
        matches!(self.state, ReservationState::Active { .. })
    }
}
