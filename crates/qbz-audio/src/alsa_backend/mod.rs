//! ALSA audio backend (direct hardware access)
//!
//! Provides direct access to ALSA hardware devices for:
//! - True exclusive mode (blocks device for other apps)
//! - Bit-perfect playback (no resampling)
//! - Low-latency audio output
//!
//! Uses CPAL's ALSA host with specific device selection.
//! Device enumeration reads directly from /proc/asound (no alsa-utils dependency).

mod device_id;
mod device_id_public;
mod device_list;
mod direct_stream;
mod direct_stream_hw;
mod direct_stream_plughw;
mod dsd_streams;
mod enumerate;
mod output_stream;
mod output_stream_device;
mod output_stream_rate;
mod output_stream_sink;
mod pipewire_suspend;
mod proc_cards;
mod proc_pcm;
mod proc_rates;
mod sample_rates;
#[cfg(test)]
mod tests_device_id;
#[cfg(test)]
mod tests_hw_fallback;

pub use device_id_public::{
    device_supports_sample_rate, get_device_supported_rates, normalize_device_id_to_stable,
    resolve_stable_to_current_hw,
};
pub use dsd_streams::{create_dop_stream, create_native_dsd_stream};
pub use pipewire_suspend::resume_suspended_sink;

// ============================================================================
// ALSA Backend Implementation
// ============================================================================

pub struct AlsaBackend {
    host: rodio::cpal::Host,
}
