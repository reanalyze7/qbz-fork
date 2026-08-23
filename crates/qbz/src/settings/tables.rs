//! Static dropdown-option tables shared by the snapshot builder and the
//! select-handler.
//!
//! Labels in these tables are `mark`ed so the extractor registers the
//! English literals; they are translated once with `t(l)` where the
//! snapshot is built.

use qbz_audio::backend::AlsaPlugin;

/// DSD delivery modes (DSD plan Phases 2-3). Value strings are the
/// AudioSettings.dsd_mode contract ("convert" | "dop" | "native").
pub(super) const DSD_MODES: &[(&str, &str)] = &[
    (qbz_i18n::mark("Convert to PCM (works everywhere)"), "convert"),
    (qbz_i18n::mark("DoP — DSD over PCM (bit-perfect)"), "dop"),
    (qbz_i18n::mark("Native DSD (kernel support required)"), "native"),
];

/// ALSA-plugin dropdown options.
pub(super) const ALSA_PLUGINS: &[(&str, AlsaPlugin)] = &[
    (qbz_i18n::mark("hw (Direct Hardware)"), AlsaPlugin::Hw),
    (qbz_i18n::mark("plughw (Auto-convert)"), AlsaPlugin::PlugHw),
    (qbz_i18n::mark("pcm (Most compatible)"), AlsaPlugin::Pcm),
];

/// "When quality retries fail" dropdown options. The value is the
/// `quality_fallback_behavior` DB string.
pub(super) const RETRY_BEHAVIORS: &[(&str, &str)] = &[
    (qbz_i18n::mark("Ask me"), "ask"),
    (qbz_i18n::mark("Always try lowest quality"), "always_fallback"),
    (qbz_i18n::mark("Always skip track"), "always_skip"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alsa_plugin_table_first_is_hw() {
        assert_eq!(ALSA_PLUGINS[0].1, AlsaPlugin::Hw);
        assert_eq!(ALSA_PLUGINS.len(), 3);
    }

    #[test]
    fn retry_behavior_table_first_is_ask() {
        assert_eq!(RETRY_BEHAVIORS[0].1, "ask");
        assert_eq!(RETRY_BEHAVIORS[1].1, "always_fallback");
        assert_eq!(RETRY_BEHAVIORS[2].1, "always_skip");
    }
}
