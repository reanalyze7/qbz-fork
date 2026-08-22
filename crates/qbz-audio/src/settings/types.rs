//! `AudioSettings` data struct: all persisted user preferences for audio
//! output device, exclusive mode, DAC passthrough, quality, normalization,
//! gapless, DSD, and crossfade. Pure data — no IO here.

use crate::{AlsaPlugin, AudioBackendType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub output_device: Option<String>, // None = system default
    pub exclusive_mode: bool,
    pub dac_passthrough: bool,
    pub preferred_sample_rate: Option<u32>,     // None = auto
    pub backend_type: Option<AudioBackendType>, // None = auto-detect
    pub alsa_plugin: Option<AlsaPlugin>,        // Only used when backend is ALSA
    pub alsa_hardware_volume: bool,             // Use ALSA mixer for volume (only with hw: devices)
    /// When true, uncached tracks start playing via streaming instead of waiting for full download
    pub stream_first_track: bool,
    /// Initial buffer size in seconds before starting streaming playback (1-10, default 3)
    pub stream_buffer_seconds: u8,
    /// When true, skip L1+L2 cache writes (streaming-only mode). Offline cache still works.
    pub streaming_only: bool,
    /// When true, cap the REQUESTED streaming quality tier at the local output
    /// device's detected ceiling (#638 fix 3; consumed by the desktop's
    /// request-time resolution, never by the audio backends). Applies to local
    /// playback only — never to casting, where the local DAC is not in the
    /// signal path. Default: false (opt-in).
    pub limit_quality_to_device: bool,
    /// Cached max sample rate of the selected device (set when device is selected)
    /// Used when limit_quality_to_device is true
    pub device_max_sample_rate: Option<u32>,
    /// Per-device sample rate limits: device_id -> max_sample_rate
    /// Allows different DACs to have independent max sample rate configurations
    #[serde(default)]
    pub device_sample_rate_limits: HashMap<String, u32>,
    /// When true, apply volume normalization using ReplayGain metadata.
    /// When false (default), the audio pipeline is 100% bit-perfect — no sample modification.
    pub normalization_enabled: bool,
    /// Target loudness in LUFS for normalization.
    /// Common values: -14.0 (Spotify/YouTube), -18.0 (audiophile), -23.0 (EBU broadcast)
    pub normalization_target_lufs: f32,
    /// When true, consecutive same-format tracks play without gap.
    /// Works on Rodio (PipeWire/Pulse) and ALSA Direct backends. Requires cached tracks.
    pub gapless_enabled: bool,
    /// When true, force PipeWire clock.force-quantum alongside clock.force-rate for bit-perfect.
    /// Reset both to 0 on stop. PipeWire-only, requires dac_passthrough.
    pub pw_force_bitperfect: bool,
    /// When true, reload audio settings from DB into the player on app startup.
    /// Useful when Player::new() may hold stale settings (e.g., after Flatpak updates).
    /// Default: false (most users don't need this).
    pub sync_audio_on_startup: bool,
    /// User preference for what happens when all quality retries fail.
    /// Values: "ask" (default), "always_fallback", "always_skip"
    /// Protected by ADR-003: must survive reset_all() and migrations.
    pub quality_fallback_behavior: String,
    /// When true, skip `pactl set-default-sink` on stream creation.
    /// Preserves external routing (JACK, qjackctl, Reaper).
    /// Mutually exclusive with dac_passthrough.
    pub skip_sink_switch: bool,
    /// When true, automatically try lower quality tiers if the requested one fails.
    /// When false (default), playback or download fails if the exact quality is unavailable.
    pub allow_quality_fallback: bool,
    /// When true, hold a per-process ALSA device reservation (Lifetime B) for the
    /// configured output device while QBZ is running, so other PulseAudio/PipeWire
    /// clients won't grab the DAC and break exclusive playback. Off by default.
    /// See `qbz-nix-docs/specs/2026-05-07-alsa-exclusive-hardening-design.md`.
    #[serde(default)]
    pub reserve_dac_while_running: bool,
    /// DSD delivery mode: "convert" (default — DSD→PCM, works everywhere),
    /// "dop" (DSD over PCM, opt-in: NOT detectable, wrong DAC = loud noise),
    /// or "native" (ALSA DSD_U32 formats, needs a kernel quirk for the DAC).
    /// Only takes effect on the ALSA direct backend with stereo tracks;
    /// everything else converts.
    #[serde(default = "default_dsd_mode")]
    pub dsd_mode: String,
    /// Crossfade duration in seconds (0 = off, gapless sequential hand-off
    /// as before; up to 10). Rodio (PipeWire/Pulse) backend ONLY — ALSA
    /// Direct/JACK/DoP stay strictly gapless: crossfading requires mixing
    /// two overlapping sources, which is fundamentally incompatible with
    /// bit-perfect/exclusive-mode playback (owner decision, 2026-08-21).
    #[serde(default)]
    pub crossfade_seconds: f32,
}

pub(crate) fn default_dsd_mode() -> String {
    "convert".to_string()
}
