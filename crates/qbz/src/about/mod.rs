//! About modal controller.
//!
//! Seeds the static `AboutState` fields (version, platform label, build date +
//! commit, the release URL and the contributor list) and wires `AboutActions`.
//! The text fields are static for the lifetime of the app; the one async bit is
//! the GitHub avatars (author + contributors), fetched off the UI thread and
//! painted onto their chips as they arrive. `install` is a one-shot seed +
//! callback wire + avatar dispatch, called once at shell setup.
//!
//! App version: the `qbz` binary inherits `version.workspace = true` (currently
//! 1.2.15), so `env!("CARGO_PKG_VERSION")` is the REAL release version, not the
//! 0.1.0 the workspace pins for library crates. The diagnostics panel reads the
//! same source. Build date + commit come from `build.rs` (`QBZ_BUILD_*`).

mod avatars;
mod install;
mod meta;

pub use install::install;
pub use meta::app_version;
