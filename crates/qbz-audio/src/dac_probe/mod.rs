//! DAC hardware-state probe (HiFi wizard Slice 8b / N6).
//!
//! Reads the ACTUAL negotiated rate the DAC is running at, independent of what
//! QBZ *requested* — this is the ground truth for the wizard's playback test.
//! On Linux, `/proc/asound/cardN/pcm*p/sub0/hw_params` reports the live hardware
//! rate while a stream is open (`closed` when idle). The ALSA card number is
//! resolved from the PipeWire `node.name` via `pw-dump` (robust; needs only
//! `pipewire-bin`, not pactl/pipewire-pulse).
//!
//! Read-only: this never opens or reconfigures a stream, so the protected
//! bit-perfect / sample-rate-passthrough path is untouched.

mod parse;
mod probe;

use serde::{Deserialize, Serialize};

pub use parse::{parse_alsa_card_for_node, parse_hw_params};
pub use probe::{negotiated_active_rate, negotiated_stream_rate};

/// The DAC's live, negotiated hardware state — what the card is REALLY clocked
/// at, not what QBZ asked for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedRate {
    /// Hardware sample rate the DAC is actually running at, in Hz.
    pub sample_rate: u32,
    /// ALSA hardware format string as reported (e.g. "S32_LE", "S24_3LE").
    /// Note: 24-bit audio is commonly carried in an `S32_LE` container — this is
    /// the ALSA container format, so the wizard verdict keys on the RATE.
    pub format: String,
    /// Channel count (e.g. 2).
    pub channels: u32,
}
