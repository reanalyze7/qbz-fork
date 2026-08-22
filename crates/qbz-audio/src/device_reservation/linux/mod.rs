//! Linux implementation of `DeviceReservation`.
//!
//! Implements a client of the `org.freedesktop.ReserveDevice1` D-Bus protocol
//! (specified by the PulseAudio project, also implemented by PipeWire,
//! WirePlumber, JACK, MPD, Roon Bridge, etc.). For an ALSA device string of
//! the form `hw:N,M` (or `plughw:`, or `hw:CARD=Name`), we map the *card*
//! index `N` to the well-known bus name `org.freedesktop.ReserveDevice1.AudioN`
//! and request ownership of it.
//!
//! Acquisition algorithm (matches the spec at
//! `qbz-nix-docs/specs/2026-05-07-alsa-exclusive-hardening-design.md`):
//!
//! 1. `RequestName` with `DO_NOT_QUEUE`.
//! 2. `PrimaryOwner` / `AlreadyOwner` -> we own it. Done.
//! 3. `Exists` / `InQueue` -> someone else holds it. Read their `Priority`
//!    property; if our priority is higher, call `RequestRelease(our_priority)`
//!    on the holder. If that returns `true`, retry `RequestName` with
//!    `DO_NOT_QUEUE | REPLACE_EXISTING`.
//! 4. Refusal or equal-or-greater priority -> `HigherPriorityHolder` (the
//!    only fatal outcome). Any other failure (bus unavailable, name denied,
//!    holder unreachable, lost re-acquire race) -> degraded no-op guard;
//!    the PCM open + EBUSY backoff ladder arbitrate the device (#508/#534).
//!
//! On `Drop`, an active guard releases the bus name. A *degraded* guard
//! (returned when the session bus is unavailable) is a no-op on `Drop`.

mod acquire;
mod acquire_reply;
mod contention;
mod dbus_ops;
mod device_name;
mod drop_impl;
mod error;
mod reservation;
#[cfg(test)]
mod tests;

pub use error::ReservationError;
pub use reservation::DeviceReservation;

/// Priority QBZ takes when acquiring a `ReserveDevice1` reservation.
///
/// Rationale (from the design spec): PulseAudio and PipeWire run at `0`, pro
/// audio software (Ardour, Bitwig, Roon Bridge) runs at `10`-`30`. We pick `5`:
/// above the system mixer so we can pre-empt PipeWire when the user toggles
/// exclusive mode, well below pro DAW software so we never stomp on a
/// recording session.
pub(crate) const QBZ_PRIORITY: i32 = 5;

/// Application name advertised over D-Bus when QBZ publishes the
/// `ReserveDevice1` interface as a server. Deferred to a future commit — see
/// `qbz-nix-docs/specs/2026-05-07-alsa-exclusive-hardening-design.md`,
/// section "The org.freedesktop.ReserveDevice1 protocol", subsections
/// "Note on `app_device_name`" / "Note on `ApplicationName`".
#[allow(dead_code)]
pub(crate) const QBZ_APPLICATION_NAME: &str = "QBZ";

/// D-Bus interface every `ReserveDevice1` holder publishes under
/// `/org/freedesktop/ReserveDevice1/AudioN`.
const RESERVE_DEVICE1_INTERFACE: &str = "org.freedesktop.ReserveDevice1";
