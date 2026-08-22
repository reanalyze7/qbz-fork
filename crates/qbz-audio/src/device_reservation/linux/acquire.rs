use zbus::blocking::Connection;

use super::acquire_reply::handle_request_name_reply;
use super::dbus_ops::try_acquire_name;
use super::device_name::{bus_name_for_card, object_path_for_card, parse_card_index};
use super::error::ReservationError;
use super::reservation::{DeviceReservation, ReservationState};

impl DeviceReservation {
    /// Acquire a D-Bus device reservation for the given ALSA `hw:` device.
    ///
    /// Returns:
    /// - `Ok(active_guard)` once we own the bus name.
    /// - `Ok(degraded_guard)` for every non-cooperative failure: unparseable
    ///   device strings, an unreachable session bus, a denied `RequestName`
    ///   (confined sandbox bus, #534), or contention resolution failing for
    ///   any reason other than the holder refusing. The caller treats these
    ///   as "no reservation, but proceed normally" — the PCM open and its
    ///   EBUSY backoff ladder arbitrate the device (#508).
    /// - `Err(HigherPriorityHolder { .. })` — the only fatal outcome: a
    ///   cooperative holder refused to release, or holds at
    ///   equal-or-greater priority.
    ///
    /// # Tight coupling rule (load-bearing — do not violate)
    ///
    /// A `DeviceReservation` MUST always be created in tight coupling with an
    /// immediate consumer of the underlying ALSA device:
    ///
    /// - Lifetime A: held inside [`AlsaDirectStream`] for as long as the PCM
    ///   is open. Acquired before `PCM::new`, dropped after the PCM is closed.
    /// - Lifetime B: held inside the application's `AppState` for as long as
    ///   the QBZ process is running, gated by the `reserve_dac_while_running`
    ///   setting. Acquired at startup or on toggle, dropped at process exit.
    ///
    /// **Never construct a `DeviceReservation` in isolation, hold it briefly,
    /// and drop it without a real device consumer in between.** The pattern
    ///
    /// ```ignore
    /// let r = DeviceReservation::acquire("hw:1,0", "test")?;
    /// std::thread::sleep(Duration::from_secs(2));
    /// drop(r);
    /// ```
    ///
    /// triggers WirePlumber to release-and-reacquire the device over an idle
    /// PCM, and some USB DACs (Cambridge DacMagic Plus confirmed, others
    /// suspected) get stranded by that transition and require a hardware
    /// power-cycle to recover. Validated 2026-05-07. Tests must always go
    /// through `AlsaDirectStream::new()` (Lifetime A) or hold the reservation
    /// for the entire process lifetime (Lifetime B).
    ///
    /// # Connection lifecycle
    ///
    /// `acquire` opens a fresh `zbus::blocking::Connection::session()` per
    /// call. zbus 4.4 does not internally pool session-bus connections, so
    /// each call pays a SASL handshake cost (~1-5 ms on a healthy bus). For
    /// per-stream (Lifetime A) acquisition this is acceptable. For the
    /// nested-inside-Lifetime-B pattern landing in Task 5, prefer reusing an
    /// existing connection via the future `acquire_with_connection` overload.
    pub fn acquire(hw_device: &str, app_device_name: &str) -> Result<Self, ReservationError> {
        // Parse failures must not propagate — the caller (AlsaDirectStream)
        // will treat any Err as fatal and abort stream creation, regressing
        // playback for devices we can't introspect. Names that don't target
        // a single card (`default`, `pulse`) and any future plugin alias we
        // can't decode degrade to a no-op guard so PCM open proceeds.
        let card = match parse_card_index(hw_device) {
            Ok(idx) => idx,
            Err(e) => {
                log::warn!(
                    "[reservation] cannot determine ALSA card index for '{}': {}. \
                     Proceeding without D-Bus reservation (degraded).",
                    hw_device,
                    e
                );
                return Ok(Self {
                    state: ReservationState::Degraded,
                });
            }
        };
        let bus_name = bus_name_for_card(card);
        let object_path = object_path_for_card(card);

        // Connect to the session bus. Failure here is *not* an error from the
        // caller's perspective — we degrade and let playback proceed.
        let connection = match Connection::session() {
            Ok(c) => c,
            Err(e) => {
                log::warn!(
                    "[reservation] D-Bus session bus unavailable, degrading: {}",
                    e
                );
                return Ok(Self {
                    state: ReservationState::Degraded,
                });
            }
        };

        // RequestName itself can be denied by the bus policy — under Snap
        // confinement the mediated session bus refuses ownership of the
        // org.freedesktop.ReserveDevice1.* names (#534). That must not be
        // fatal: degrade to no reservation and let the PCM open proceed,
        // exactly like the bus-unavailable case above.
        let reply = match try_acquire_name(&connection, &bus_name, false) {
            Ok(reply) => reply,
            Err(e) => {
                log::warn!(
                    "[reservation] could not request name {} (confined session \
                     bus?): {}. Proceeding without D-Bus reservation (degraded).",
                    bus_name,
                    e
                );
                return Ok(Self {
                    state: ReservationState::Degraded,
                });
            }
        };

        handle_request_name_reply(reply, connection, bus_name, object_path, app_device_name)
    }
}
