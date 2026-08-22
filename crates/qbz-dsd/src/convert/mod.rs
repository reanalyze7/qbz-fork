//! Streaming DSD → 88.2 kHz PCM conversion chain.
//!
//! dsd2pcm decimates 8:1 (one float per DSD byte), leaving DSD64 at
//! 352.8 kHz, DSD128 at 705.6 kHz, … A chain of half-band ÷2 stages then
//! brings every rate down to a uniform 88.2 kHz.
//!
//! Why 88.2 kHz (v1 policy, revised after the first smoke): the original
//! 176.4 kHz target is NOT universally supported — the owner's DacMagic
//! Plus USB interface exposes 44.1/48/88.2/96/192 only, so the player fell
//! back to rodio's linear resampler, which has no anti-alias filter: the
//! DSD ultrasonic noise shelf (huge between 44k–88k at a 176.4k container
//! rate) folded straight into the audible band as loud hiss. At 88.2 kHz
//! every DAC that matters accepts the rate natively (no resampler in the
//! path), it is an exact 44.1k-family division, and the half-band chain —
//! which DOES anti-alias properly — removes all DSD noise above 44.1 kHz.
//! A per-device higher-rate policy can come with the Phase-2 capability
//! model (see qbz-nix-docs/dsd-support/).

mod converter;
mod downmix;
mod halfband;

pub use converter::DsdPcmConverter;

/// Uniform PCM output rate for converted DSD.
pub const OUTPUT_RATE: u32 = 88_200;

/// Default conversion gain. DSD program material can exceed 0 dBFS when
/// low-passed to PCM; the customary −6 dB trim prevents clipping.
pub const DEFAULT_GAIN_DB: f32 = -6.0;
