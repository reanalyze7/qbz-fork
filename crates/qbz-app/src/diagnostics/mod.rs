//! Frontend-agnostic diagnostics snapshot builders.
//!
//! This module is a faithful COPY of the Tauri diagnostics backend
//! (`src-tauri/src/commands_v2/diagnostics.rs`) so the Slint `qbz` bin can
//! produce byte-identical `RuntimeDiagnostics` + `SystemInfo` snapshots
//! headlessly. The Tauri source stays untouched as the reference copy.
//!
//! Everything here is pure: std + `/proc` + `/sys` + `/etc/os-release` +
//! `/proc/self/maps`, plus three settings structs passed in by the caller.
//! No `tauri::` types, no `crate::runtime::RuntimeError`. Both builders are
//! infallible and return their struct directly.
//!
//! The `#[serde(rename_all = "camelCase")]` on both structs is load-bearing:
//! the exported JSON keys (and the existing Svelte TS interface) depend on it,
//! so the shared struct keeps the same derive + rename for a byte-identical
//! export.

mod runtime;
mod system;

pub use runtime::*;
pub use system::*;
