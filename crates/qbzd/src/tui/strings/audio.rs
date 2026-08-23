// ============================ Audio (§3.2) ============================

pub const AUDIO_TITLE: &str = "Audio";
pub const AUDIO_GROUP_OUTPUT: &str = "OUTPUT";
pub const AUDIO_GROUP_BITPERFECT: &str = "BIT-PERFECT";
pub const AUDIO_GROUP_TRANSPORT: &str = "STREAMING TRANSPORT";

pub const A_BACKEND: &str = "Backend";
pub const A_DEVICE: &str = "Output device";
pub const A_ALSA_PLUGIN: &str = "ALSA plugin";
pub const A_HW_VOLUME: &str = "Hardware volume";
pub const A_DSD: &str = "DSD playback";
pub const A_EXCLUSIVE: &str = "Exclusive mode";
pub const A_RESERVE: &str = "Reserve DAC";
pub const A_PASSTHROUGH: &str = "DAC passthrough";
pub const A_FORCE_BP: &str = "Force bit-perfect";
pub const A_LOCK_OUTPUT: &str = "Lock output device";
pub const A_STREAM_UNCACHED: &str = "Stream uncached";
pub const A_BUFFER: &str = "Initial buffer";
pub const A_STREAMING_ONLY: &str = "Streaming only";

// Disabled-row reasons (rendered dim in parentheses, §3).
pub const R_ALSA_ONLY: &str = "ALSA only";
pub const R_PIPEWIRE_ONLY: &str = "PipeWire only";
pub const R_PASSTHROUGH_OFF: &str = "off while DAC passthrough on";

pub const DSD_CONVERT: &str = "Convert to PCM (works everywhere)";
pub const DSD_DOP: &str = "DoP — DSD over PCM (bit-perfect)";
pub const DSD_NATIVE: &str = "Native DSD (kernel support required)";

pub const ALSA_HW: &str = "hw (Direct Hardware)";
pub const ALSA_PLUGHW: &str = "plughw (Auto-convert)";
pub const ALSA_PCM: &str = "pcm (Most compatible)";

/// DSD guard (§3.2.4). Verbatim-in-spirit of the desktop warning.
pub const DSD_GUARD_TITLE: &str = "DSD direct mode";
pub const DSD_GUARD_BODY: &str = "Choose DoP or Native only if your DAC supports it. On any other DAC they play\nas LOUD NOISE. Volume is fixed and seeking is disabled in DoP/Native; Native\nadditionally needs kernel support.";
pub const DSD_GUARD_HINT: &str = "Enter confirm · Esc revert";

pub const AUDIO_SCANNING: &str = "scanning…";
pub const DEVICE_PICKER_TITLE: &str = "Output device";

/// No-devices hint panel (§5.1).
pub const NO_DEVICES: &str = "no output devices found — is the DAC plugged in and powered? is your user in\nthe 'audio' group? PipeWire backend: is pipewire running?  (r to re-scan)";

pub const JACK_WARNING: &str = "JACK is not bit-perfect (routes through the JACK graph, resamples)";
pub const BP_BADGE: &str = "[BP]";
