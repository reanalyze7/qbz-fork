//! The per-key persistence dispatch for `handle_bool` — one key at a time,
//! no cascades (those live in `bool.rs`, right before this is called).

use std::sync::Arc;

use qbz_app::settings::playback::AutoplayMode;
use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::store::{with_audio, with_playback, Apply, SettingsCtx};

/// Persist one boolean settings key and report what the live `Player`
/// needs applied. `Apply::None` keys resolve outside the audio store
/// (discover prefs, musicbrainz, playback prefs) and are handled inline.
pub(super) async fn persist_bool_key(
    ctx: &SettingsCtx,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    key: &str,
    value: bool,
) -> Result<Apply, String> {
    match key {
        // --- Audio toggles -------------------------------------------------
        "limit-quality-to-device" => {
            with_audio(&ctx.audio, |s| s.set_limit_quality_to_device(value)).map(|_| Apply::Reload)
        }
        "alsa-hardware-volume" => {
            with_audio(&ctx.audio, |s| s.set_alsa_hardware_volume(value)).map(|_| Apply::Reinit)
        }
        "exclusive-mode" => {
            with_audio(&ctx.audio, |s| s.set_exclusive_mode(value)).map(|_| Apply::Reinit)
        }
        "reserve-dac" => with_audio(&ctx.audio, |s| s.set_reserve_dac_while_running(value))
            .map(|_| Apply::Reload),
        "dac-passthrough" => {
            with_audio(&ctx.audio, |s| s.set_dac_passthrough(value)).map(|_| Apply::Reinit)
        }
        "pw-force-bitperfect" => {
            with_audio(&ctx.audio, |s| s.set_pw_force_bitperfect(value)).map(|_| Apply::Reload)
        }
        "allow-quality-fallback" => {
            with_audio(&ctx.audio, |s| s.set_allow_quality_fallback(value)).map(|_| Apply::Reload)
        }
        "sync-audio-on-startup" => {
            with_audio(&ctx.audio, |s| s.set_sync_audio_on_startup(value)).map(|_| Apply::Reload)
        }
        "skip-sink-switch" => {
            with_audio(&ctx.audio, |s| s.set_skip_sink_switch(value)).map(|_| Apply::Reinit)
        }
        // --- Playback toggles backed by AudioSettings ----------------------
        "gapless" => with_audio(&ctx.audio, |s| s.set_gapless_enabled(value)).map(|_| Apply::Reload),
        "normalization" => {
            // Loudness leveling; the shared player applies/bypasses it (and
            // skips it entirely under bit-perfect). Reload (not Reinit) — the
            // audio thread re-reads the settings struct, no device re-init.
            with_audio(&ctx.audio, |s| s.set_normalization_enabled(value)).map(|_| Apply::Reload)
        }
        "stream-uncached" => {
            with_audio(&ctx.audio, |s| s.set_stream_first_track(value)).map(|_| Apply::Reload)
        }
        "streaming-only" => {
            with_audio(&ctx.audio, |s| s.set_streaming_only(value)).map(|_| Apply::Reload)
        }
        // --- Playback toggles backed by PlaybackPreferences ----------------
        "continue-playback" => {
            // On = ContinueWithinSource, off = PlayTrackOnly.
            let mode = if value {
                AutoplayMode::ContinueWithinSource
            } else {
                AutoplayMode::PlayTrackOnly
            };
            with_playback(&ctx.playback, |s| s.set_autoplay_mode(mode)).map(|_| Apply::None)
        }
        "show-context-icon" => {
            with_playback(&ctx.playback, |s| s.set_show_context_icon(value)).map(|_| Apply::None)
        }
        "show-recommendations" => {
            crate::discover_prefs::set_show_recommendations(value);
            Ok(Apply::None)
        }
        "musicbrainz" => {
            // Opt-out toggle (default ON). Persist to ui_prefs (Option B,
            // mirrors system_notifications) and drive the core client's
            // in-memory enabled flag so the artist Network/Scene sidebar and
            // playlist Suggested-Songs gate immediately.
            let mut prefs = crate::ui_prefs::load();
            prefs.musicbrainz_enabled = value;
            crate::ui_prefs::save(&prefs);
            runtime.core().musicbrainz_set_enabled(value).await;
            Ok(Apply::None)
        }
        "persist-session" => {
            let r =
                with_playback(&ctx.playback, |s| s.set_persist_session(value)).map(|_| Apply::None);
            if let Ok(p) = with_playback(&ctx.playback, |s| s.get_preferences()) {
                crate::session_persist::set_gates(p.persist_session, p.resume_playback_position);
            }
            r
        }
        "resume-position" => {
            let r = with_playback(&ctx.playback, |s| s.set_resume_playback_position(value))
                .map(|_| Apply::None);
            if let Ok(p) = with_playback(&ctx.playback, |s| s.get_preferences()) {
                crate::session_persist::set_gates(p.persist_session, p.resume_playback_position);
            }
            r
        }
        other => Err(format!("unknown settings bool key: {other}")),
    }
}
