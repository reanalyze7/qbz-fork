//! macOS-only `CpalDefaultBackend` internals: exclusive-mode (Hog Mode) stream
//! opening plus sample-rate query/switch helpers used only on macOS. Compiles
//! to nothing on Linux/Windows.
//!
//! Split into `exclusive.rs` (guard + retry-open), `rate_query.rs` (nominal-
//! rate lookups) and `rate_switch.rs` (rate-switch helpers) to stay under the
//! per-file line limit; all are plain `impl CpalDefaultBackend { .. }` blocks
//! (inherent impls may be repeated across files, unlike a single trait impl).

mod exclusive;
mod probe;
mod rate_query;
mod rate_switch;
mod shared_open;

pub(super) const MACOS_SHARED_OPEN_MAX_ATTEMPTS: usize = 2;
pub(super) const MACOS_SHARED_OPEN_RETRY_DELAY: std::time::Duration =
    std::time::Duration::from_millis(50);
