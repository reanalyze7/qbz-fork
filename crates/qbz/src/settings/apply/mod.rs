//! Pushing persisted audio settings into the live `Player`, plus the
//! device-cap / conditional-flags / snapshot-rebuild glue used after a
//! cross-setting cascade.

mod audio;
mod bitperfect;
mod device_cap;
mod rebuild;

pub use bitperfect::apply_startup_bitperfect_volume;
pub use device_cap::refresh_device_cap;

pub(super) use audio::apply_audio;
pub(super) use bitperfect::maybe_force_bitperfect_volume;
pub(super) use device_cap::push_conditional_flags;
pub(super) use rebuild::rebuild_and_push;
