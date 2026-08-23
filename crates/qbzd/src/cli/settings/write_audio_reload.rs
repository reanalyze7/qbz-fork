// crates/qbzd/src/cli/settings/write_audio_reload.rs — the Reload-class
// `audio.*` write arms (struct refresh only, no audible gap).

use crate::paths::ProfileRoots;

use super::codec_bool::parse_bool;
use super::codec_value::{parse_f32, parse_quality_fallback_behavior, parse_stream_buffer_seconds};
use super::store::open_audio;
use super::write::SetError;

/// Returns `Ok(true)` if `key` was one of the Reload-class audio keys
/// (handled here), `Ok(false)` if not (the caller tries the next domain).
pub(super) fn write_audio_reload(roots: &ProfileRoots, key: &str, raw: &str) -> Result<bool, SetError> {
    match key {
        "audio.stream_first_track" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_stream_first_track(v)
                .map_err(SetError::Io)?
        }
        "audio.stream_buffer_seconds" => {
            let v = parse_stream_buffer_seconds(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_stream_buffer_seconds(v)
                .map_err(SetError::Io)?
        }
        "audio.streaming_only" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_streaming_only(v).map_err(SetError::Io)?
        }
        "audio.limit_quality_to_device" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_limit_quality_to_device(v)
                .map_err(SetError::Io)?
        }
        "audio.allow_quality_fallback" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_allow_quality_fallback(v)
                .map_err(SetError::Io)?
        }
        "audio.quality_fallback_behavior" => {
            let v = parse_quality_fallback_behavior(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_quality_fallback_behavior(&v)
                .map_err(SetError::Io)?
        }
        "audio.gapless_enabled" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_gapless_enabled(v).map_err(SetError::Io)?
        }
        "audio.normalization_enabled" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_normalization_enabled(v)
                .map_err(SetError::Io)?
        }
        "audio.normalization_target_lufs" => {
            let v = parse_f32(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_normalization_target_lufs(v)
                .map_err(SetError::Io)?
        }
        "audio.pw_force_bitperfect" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_pw_force_bitperfect(v)
                .map_err(SetError::Io)?
        }
        "audio.reserve_dac_while_running" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_reserve_dac_while_running(v)
                .map_err(SetError::Io)?
        }
        "audio.sync_audio_on_startup" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_sync_audio_on_startup(v)
                .map_err(SetError::Io)?
        }
        _ => return Ok(false),
    }
    Ok(true)
}
