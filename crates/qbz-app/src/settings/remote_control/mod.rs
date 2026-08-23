//! Remote control settings and allowed-origin persistence.
//!
//! This module owns portable storage only. Server startup, TLS certificate
//! management, QR generation, router/CORS application, and live restarts remain
//! host-owned behavior.

mod origins;
mod rc;
#[cfg(test)]
mod tests;

pub use origins::{AllowedOrigin, AllowedOriginsState, AllowedOriginsStore};
pub use rc::{RemoteControlSettings, RemoteControlSettingsState, RemoteControlSettingsStore};

pub(crate) const DEFAULT_ALLOWED_ORIGINS: &[&str] = &[
    "vicrodh.github.io",
    "control.qbz.lol",
    "www.control.qbz.lol",
];
