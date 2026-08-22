//! System capability detection.
//!
//! Probes the host environment at startup to derive a runtime profile that
//! tunes resource-heavy behaviors (prefetch depth, streaming buffer size,
//! prefetch quality cap) for memory-constrained machines like the
//! Raspberry Pi 3B (1 GB RAM, issue #331).
//!
//! Detection is one-shot, cached in a `OnceLock`, and pure once given the
//! `/proc/meminfo` contents — making it trivial to test by passing
//! synthetic input.

use std::sync::OnceLock;

mod meminfo;
mod pressure;
mod profile;

pub use meminfo::*;
pub use pressure::*;
pub use profile::{MemoryClass, MemoryProfile};

/// Process-wide cached profile. Detection runs once on first access.
static PROFILE: OnceLock<MemoryProfile> = OnceLock::new();

/// Return the cached memory profile, running detection on first call.
/// Logs the resolved profile at info level on the initial detection.
pub fn memory_profile() -> &'static MemoryProfile {
    PROFILE.get_or_init(|| {
        let profile = meminfo::detect_profile();
        match profile.class {
            MemoryClass::LowMemory => {
                log::info!(
                    "[system] Low-memory profile active: {} MB total RAM, prefetch={}, max_initial_buffer={}KB, hires_prefetch=disabled",
                    profile.mem_total_kb / 1024,
                    profile.prefetch_count,
                    profile.max_initial_buffer_bytes / 1024,
                );
            }
            MemoryClass::Normal => {
                log::info!(
                    "[system] Normal memory profile: {} MB total RAM",
                    profile.mem_total_kb / 1024
                );
            }
        }
        profile
    })
}
