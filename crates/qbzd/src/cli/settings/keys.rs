// crates/qbzd/src/cli/settings/keys.rs — the canonical dotted-key table
// (NORMATIVE for this build) and its classification.

/// Whether a key's write is Reinit-class (closes/reopens the output device),
/// Reload-class (struct refresh only, no audible gap), or affects nothing the
/// live `Player` reads immediately (playback prefs — applies next play).
/// Purely this CLI's own
/// bookkeeping for the success-line hint; the daemon decides for itself via
/// `daemon::audio_routing_changed` — the two are independent copies of the
/// same table, per 03-setup-tui.md §4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyClass {
    Reinit,
    Reload,
    None,
}

/// `settings show` lists exactly these, in this order; `settings set` accepts
/// exactly these keys and nothing else. Domains: `audio.*` (`AudioSettingsStore`),
/// `playback.*` (daemon_prefs.streaming_quality + `PlaybackPreferencesStore`).
pub(super) const KEY_TABLE: &[(&str, ApplyClass)] = &[
    // --- audio (Reinit — 03-setup-tui.md §4.3's 9-field list) -------------
    ("audio.backend", ApplyClass::Reinit),
    ("audio.device", ApplyClass::Reinit),
    ("audio.alsa_plugin", ApplyClass::Reinit),
    ("audio.alsa_hardware_volume", ApplyClass::Reinit),
    ("audio.exclusive_mode", ApplyClass::Reinit),
    ("audio.dac_passthrough", ApplyClass::Reinit),
    ("audio.skip_sink_switch", ApplyClass::Reinit),
    ("audio.dsd_mode", ApplyClass::Reinit),
    ("audio.device_max_sample_rate", ApplyClass::Reinit),
    // --- audio (Reload) -----------------------------------------------------
    ("audio.stream_first_track", ApplyClass::Reload),
    ("audio.stream_buffer_seconds", ApplyClass::Reload),
    ("audio.streaming_only", ApplyClass::Reload),
    ("audio.limit_quality_to_device", ApplyClass::Reload),
    ("audio.allow_quality_fallback", ApplyClass::Reload),
    ("audio.quality_fallback_behavior", ApplyClass::Reload),
    ("audio.gapless_enabled", ApplyClass::Reload),
    ("audio.normalization_enabled", ApplyClass::Reload),
    ("audio.normalization_target_lufs", ApplyClass::Reload),
    ("audio.pw_force_bitperfect", ApplyClass::Reload),
    ("audio.reserve_dac_while_running", ApplyClass::Reload),
    ("audio.sync_audio_on_startup", ApplyClass::Reload),
    // --- playback (daemon_prefs + PlaybackPreferencesStore) ----------------
    ("playback.quality", ApplyClass::None),
    ("playback.autoplay", ApplyClass::None),
    ("playback.persist_session", ApplyClass::None),
    ("playback.resume_playback_position", ApplyClass::None),
    ("playback.show_context_icon", ApplyClass::None),
    ("playback.mpris", ApplyClass::None),
];

pub(super) fn classify(key: &str) -> Option<ApplyClass> {
    KEY_TABLE.iter().find(|(k, _)| *k == key).map(|(_, c)| *c)
}

/// The fault + fix for an unknown key (02 §1.4 error voice: name the fault,
/// then the fix). No "error:" prefix — `set()` adds it uniformly at the print
/// site, matching every other value-parse error in this file.
pub(super) fn unknown_key_error(key: &str) -> String {
    let mut out = format!("unknown setting key '{key}'\n  → valid keys:\n");
    for (k, _) in KEY_TABLE {
        out.push_str(&format!("      {k}\n"));
    }
    out
}
