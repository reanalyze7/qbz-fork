//! CoreAudio direct access for macOS
//!
//! Provides device capability probing and sample rate switching on macOS
//! using the coreaudio-rs safe wrappers.
//!
//! Phase 1: Device probing + nominal sample rate switching (shared mode)
//! Phase 2 (future): Hog mode + integer mode + IO proc for bit-perfect playback

#![cfg_attr(target_os = "macos", allow(deprecated))]

#[cfg(target_os = "macos")]
mod devices;
#[cfg(target_os = "macos")]
mod guard;
#[cfg(target_os = "macos")]
mod hog_mode;
#[cfg(target_os = "macos")]
mod sample_rate;
#[cfg(not(target_os = "macos"))]
mod stub;
#[cfg(target_os = "macos")]
mod volume;

#[cfg(target_os = "macos")]
pub use devices::{
    find_device_by_name, get_default_output_device, get_device_name,
    get_device_transport_type, get_output_device_ids, resolve_output_device_id,
    resolve_output_device_name,
};
#[cfg(target_os = "macos")]
pub use guard::CoreAudioExclusiveGuard;
#[cfg(target_os = "macos")]
pub use hog_mode::{get_hogging_pid, set_hog_mode};
#[cfg(target_os = "macos")]
pub use sample_rate::{
    get_nominal_sample_rate, query_supported_sample_rates, set_nominal_sample_rate,
};
#[cfg(not(target_os = "macos"))]
pub use stub::{
    get_nominal_sample_rate_by_name, query_supported_sample_rates, set_nominal_sample_rate_by_name,
    CoreAudioExclusiveGuard,
};
#[cfg(target_os = "macos")]
pub use volume::{get_hardware_volume, set_hardware_volume};

/// CoreAudio device ID (re-exported so callers don't need objc2_core_audio)
#[cfg(target_os = "macos")]
pub type AudioDeviceID = u32;

// CoreAudio transport type constants (FourCC values from AudioHardware.h)
#[cfg(target_os = "macos")]
mod transport_types {
    pub const BUILT_IN: u32 = 0x626c746e; // 'bltn'
    pub const USB: u32 = 0x75736220; // 'usb '
    pub const BLUETOOTH: u32 = 0x626c7565; // 'blue'
    pub const BLUETOOTH_LE: u32 = 0x626c6561; // 'blea'
    pub const HDMI: u32 = 0x68646d69; // 'hdmi'
    pub const DISPLAY_PORT: u32 = 0x64707274; // 'dprt'
    pub const THUNDERBOLT: u32 = 0x7468756e; // 'thun'
    pub const FIREWIRE: u32 = 0x31333934; // '1394'
    pub const VIRTUAL: u32 = 0x76697274; // 'virt'
    pub const AGGREGATE: u32 = 0x67727570; // 'grup'
}

/// Common audio sample rates to check against device capabilities
#[cfg(target_os = "macos")]
const COMMON_SAMPLE_RATES: &[u32] = &[
    44100, 48000, 88200, 96000, 176400, 192000, 352800, 384000, 705600, 768000,
];
