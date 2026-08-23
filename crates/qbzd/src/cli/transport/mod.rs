// crates/qbzd/src/cli/transport/ — the 10 transport verbs (02-cli-and-api.md
// §2.2): `now play pause toggle stop next prev seek volume mute`. Each
// networked verb is exactly one HTTP request (§1.1); the pure argument
// parsers/renderers below are unit-tested without a running daemon.
mod advance;
mod format;
mod mute;
mod now;
mod seek;
mod state_verbs;
#[cfg(test)]
mod tests;
mod volume;

pub use advance::{next, prev};
pub use mute::{mute, mute_body};
pub use now::now;
pub use seek::{parse_seek_arg, seek, seek_body, SeekArg};
pub use state_verbs::{pause, play, stop, toggle};
pub use volume::{fraction_to_pct, parse_volume_arg, pct_to_fraction, volume, volume_body, VolumeArg};
