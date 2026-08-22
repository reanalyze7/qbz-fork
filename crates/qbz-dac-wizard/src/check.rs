//! Slice 6 (check step): runs the frontend-agnostic audio-stack probes
//! (`qbz_audio::health`) on open, maps them to per-distro copy-paste
//! remediations the check step renders, and recomputes them when the user
//! overrides the distro. Read-only — nothing here writes a system file or
//! opens a stream.

mod install;
pub(crate) mod remediation;
mod state;

pub use state::{apply_health, open_immediate, set_distro, set_init};
